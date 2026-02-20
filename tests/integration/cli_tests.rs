// Integration tests for the flamewasm CLI (end-to-end process invocation)

#[cfg(test)]
mod cli_tests {
    use std::process::Command;

    fn flamewasm_bin() -> std::path::PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") { path.pop(); }
        path.join("flamewasm")
    }

    #[test]
    fn test_help_flag() {
        let bin = flamewasm_bin();
        if !bin.exists() {
            eprintln!("flamewasm binary not found at {bin:?}, skipping");
            return;
        }
        let output = Command::new(&bin).arg("--help").output().unwrap();
        assert!(output.status.success() || output.status.code() == Some(0));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("flamewasm") || stdout.contains("FlameWasm"));
    }

    #[test]
    fn test_version_flag() {
        let bin = flamewasm_bin();
        if !bin.exists() { return; }
        let output = Command::new(&bin).arg("--version").output().unwrap();
        assert!(output.status.success());
    }
}
