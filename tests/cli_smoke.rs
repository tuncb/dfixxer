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

fn copy_file_to_temp_with_name(src: &Path, temp_dir: &Path, name: &str) -> std::path::PathBuf {
    let dst = temp_dir.join(name);
    fs::copy(src, &dst).expect("Failed to copy fixture file");
    dst
}

fn assert_contents_match(actual_content: &str, expected_content: &str, file_name: &str) {
    if actual_content == expected_content {
        return;
    }

    let normalized_actual = actual_content.replace("\r\n", "\n");
    let normalized_expected = expected_content.replace("\r\n", "\n");
    if normalized_actual == normalized_expected {
        return;
    }

    // Show full content comparison like assert_eq!
    let mut diff_info = "assertion failed: `(left == right)`\n".to_string();
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
fn test_check_correct_files() {
    let check_dir = Path::new("test-data").join("check-correct");
    for entry in WalkDir::new(check_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let status = Command::new(env!("CARGO_BIN_EXE_dfixxer"))
            .arg("check")
            .arg(path)
            .status()
            .expect("Failed to run check command");
        assert!(status.success(), "Check command failed for {:?}", path);
    }
}

#[test]
fn test_check_does_not_modify_file() {
    let temp_dir = create_unique_temp_dir();
    let src = Path::new("test-data")
        .join("update")
        .join("ex1.original.test.pas");
    let temp_file = copy_file_to_temp_with_name(&src, &temp_dir, "check_no_mutation_1.pas");

    let before = fs::read_to_string(&temp_file).expect("Failed to read temp file before check");
    let output = Command::new(env!("CARGO_BIN_EXE_dfixxer"))
        .arg("check")
        .arg(&temp_file)
        .output()
        .expect("Failed to run check command");
    assert!(
        output.status.code().unwrap_or(1) > 0,
        "Expected non-zero replacement count for check command"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("@@"),
        "Expected unified diff output containing hunk markers"
    );

    let after = fs::read_to_string(&temp_file).expect("Failed to read temp file after check");
    assert_eq!(
        before, after,
        "check command modified file contents unexpectedly"
    );

    fs::remove_dir_all(&temp_dir).expect("Failed to remove temp dir");
}

#[test]
fn test_check_multi_does_not_modify_files_and_prints_per_file_output() {
    let temp_dir = create_unique_temp_dir();
    let src1 = Path::new("test-data")
        .join("update")
        .join("ex1.original.test.pas");
    let src2 = Path::new("test-data")
        .join("update")
        .join("ex2.original.test.pas");
    let temp_file1 = copy_file_to_temp_with_name(&src1, &temp_dir, "check_multi_1.pas");
    let temp_file2 = copy_file_to_temp_with_name(&src2, &temp_dir, "check_multi_2.pas");

    let before1 = fs::read_to_string(&temp_file1).expect("Failed to read first file before check");
    let before2 = fs::read_to_string(&temp_file2).expect("Failed to read second file before check");

    let pattern_path = temp_dir.join("*.pas");
    let pattern = pattern_path.to_string_lossy();
    let output = Command::new(env!("CARGO_BIN_EXE_dfixxer"))
        .arg("check")
        .arg(pattern.as_ref())
        .arg("--multi")
        .output()
        .expect("Failed to run check --multi command");
    assert!(
        output.status.code().unwrap_or(1) > 0,
        "Expected non-zero replacement count for check --multi command"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.matches("Processing file:").count() >= 2,
        "Expected per-file output markers in --multi mode"
    );
    assert!(
        stdout.contains("@@"),
        "Expected unified diff output containing hunk markers in --multi mode"
    );

    let after1 = fs::read_to_string(&temp_file1).expect("Failed to read first file after check");
    let after2 = fs::read_to_string(&temp_file2).expect("Failed to read second file after check");
    assert_eq!(
        before1, after1,
        "check --multi modified first file contents unexpectedly"
    );
    assert_eq!(
        before2, after2,
        "check --multi modified second file contents unexpectedly"
    );

    fs::remove_dir_all(&temp_dir).expect("Failed to remove temp dir");
}

#[test]
fn test_update_smoke() {
    let test_data_dir = Path::new("test-data").join("update");
    let temp_dir = create_unique_temp_dir();

    // Ensure configuration files are available in the temp directory by
    // copying all dfixxer.toml files while preserving relative paths.
    for entry in WalkDir::new(&test_data_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name == "dfixxer.toml"
        {
            let rel_path = path.strip_prefix(&test_data_dir).unwrap();
            let temp_file = temp_dir.join(rel_path);
            if let Some(parent) = temp_file.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(path, &temp_file).expect("Failed to copy config to temp");
        }
    }

    for entry in WalkDir::new(&test_data_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.ends_with(".original.test.pas")
        {
            // To avoid name collisions, preserve relative path in temp dir
            let rel_path = path.strip_prefix(&test_data_dir).unwrap();
            let temp_file = temp_dir.join(rel_path);
            if let Some(parent) = temp_file.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(path, &temp_file).expect("Failed to copy file to temp");

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

            assert_contents_match(
                &updated_content,
                &correct_content,
                rel_path.to_string_lossy().as_ref(),
            );
        }
    }

    // Clean up temp dir
    fs::remove_dir_all(&temp_dir).expect("Failed to remove temp dir");
}
