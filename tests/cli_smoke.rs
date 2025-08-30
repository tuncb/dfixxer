use std::process::Command;

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
