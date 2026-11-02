#[cfg(unix)]
mod unix {
    use std::{fs, time::Duration};

    use claude_code::ClaudeClient;
    use tempfile::TempDir;

    #[tokio::test]
    async fn help_and_version_invoke_root_flags() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let script_path = dir.path().join("fake-claude");

        let script = r#"#!/bin/sh
set -eu
case "${1:-}" in
  --help) echo "help ok" ;;
  --version) echo "version ok" ;;
  *) echo "expected --help or --version, got: ${1:-<none>}" >&2; exit 10 ;;
esac
"#;

        fs::write(&script_path, script).expect("write script");
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");

        let client = ClaudeClient::builder()
            .binary(&script_path)
            .timeout(Some(Duration::from_secs(2)))
            .build();

        let help = client.help().await.expect("help run");
        assert!(help.status.success());
        assert_eq!(String::from_utf8_lossy(&help.stdout).trim(), "help ok");

        let version = client.version().await.expect("version run");
        assert!(version.status.success());
        assert_eq!(
            String::from_utf8_lossy(&version.stdout).trim(),
            "version ok"
        );
    }
}
