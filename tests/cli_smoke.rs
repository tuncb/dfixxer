use std::process::Command;

use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[test]
fn test_update_smoke() {
    let test_data_dir = Path::new("test-data");
    let temp_dir = create_unique_temp_dir();

    for entry in fs::read_dir(test_data_dir).expect("Failed to read test-data dir") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".original.test.pas") {
                let temp_file = temp_dir.join(name);
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
                let correct_file = test_data_dir.join(correct_name);
                let updated_content =
                    fs::read_to_string(&temp_file).expect("Failed to read updated file");
                let correct_content =
                    fs::read_to_string(&correct_file).expect("Failed to read correct file");
                assert_eq!(
                    updated_content, correct_content,
                    "Mismatch for file: {}",
                    name
                );
            }
        }
    }

    // Clean up temp dir
    fs::remove_dir_all(&temp_dir).expect("Failed to remove temp dir");
}
