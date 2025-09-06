use std::process::Command;

use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[test]
fn test_help_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_dfixxer"))
        .arg("--help")
        .output()
        .expect("Failed to run dfixxer binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));
}

fn create_unique_temp_dir() -> std::path::PathBuf {
    let mut temp_path = env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    temp_path.push(format!("dfixxer_test_{}", unique));
    fs::create_dir_all(&temp_path).unwrap();
    temp_path
}

fn assert_contents_match(actual_content: &str, expected_content: &str, file_name: &str) {
    if actual_content == expected_content {
        return;
    }

    // Show full content comparison like assert_eq!
    let mut diff_info = format!("assertion failed: `(left == right)`\n");
    diff_info.push_str(&format!("Mismatch for file: {}\n", file_name));
    diff_info.push_str(&format!("left:\n{:?}\n", actual_content));
    diff_info.push_str(&format!("right:\n{:?}\n", expected_content));

    // Show detailed diff information
    let actual_lines: Vec<&str> = actual_content.lines().collect();
    let expected_lines: Vec<&str> = expected_content.lines().collect();

    diff_info.push_str(&format!(
        "Actual file has {} lines, expected file has {} lines\n",
        actual_lines.len(),
        expected_lines.len()
    ));

    // Show first differing line
    for (i, (actual_line, expected_line)) in
        actual_lines.iter().zip(expected_lines.iter()).enumerate()
    {
        if actual_line != expected_line {
            diff_info.push_str(&format!(
                "Line {}: \n  Actual:   {:?}\n  Expected: {:?}\n",
                i + 1,
                actual_line,
                expected_line
            ));
            break; // Show only the first difference for brevity
        }
    }

    // Handle case where files have different lengths
    if actual_lines.len() != expected_lines.len() {
        let min_len = actual_lines.len().min(expected_lines.len());
        if actual_lines.len() > expected_lines.len() {
            diff_info.push_str(&format!(
                "Actual file has {} extra lines starting at line {}\n",
                actual_lines.len() - expected_lines.len(),
                min_len + 1
            ));
        } else {
            diff_info.push_str(&format!(
                "Actual file is missing {} lines that should start at line {}\n",
                expected_lines.len() - actual_lines.len(),
                min_len + 1
            ));
        }
    }

    panic!("{}", diff_info);
}

#[test]
fn test_update_smoke() {
    let test_data_dir = Path::new("test-data\\update");
    let temp_dir = create_unique_temp_dir();

    // Ensure configuration files are available in the temp directory by
    // copying all dfixxer.toml files while preserving relative paths.
    for entry in WalkDir::new(test_data_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "dfixxer.toml" {
                let rel_path = path.strip_prefix(test_data_dir).unwrap();
                let temp_file = temp_dir.join(rel_path);
                if let Some(parent) = temp_file.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::copy(&path, &temp_file).expect("Failed to copy config to temp");
            }
        }
    }

    for entry in WalkDir::new(test_data_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".original.test.pas") {
                // To avoid name collisions, preserve relative path in temp dir
                let rel_path = path.strip_prefix(test_data_dir).unwrap();
                let temp_file = temp_dir.join(rel_path);
                if let Some(parent) = temp_file.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::copy(&path, &temp_file).expect("Failed to copy file to temp");

                // Run the update command
                let status = Command::new(env!("CARGO_BIN_EXE_dfixxer"))
                    .arg("update")
                    .arg(&temp_file)
                    .status()
                    .expect("Failed to run update command");
                assert!(
                    status.success(),
                    "Update command failed for {:?}",
                    temp_file
                );

                // Compare with correct file
                let correct_name = name.replace("original", "correct");
                let correct_file = path.with_file_name(correct_name);
                let updated_content =
                    fs::read_to_string(&temp_file).expect("Failed to read updated file");
                let correct_content =
                    fs::read_to_string(&correct_file).expect("Failed to read correct file");

                assert_contents_match(&updated_content, &correct_content, name);
            }
        }
    }

    // Clean up temp dir
    fs::remove_dir_all(&temp_dir).expect("Failed to remove temp dir");
}
