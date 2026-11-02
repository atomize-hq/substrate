#[cfg(unix)]
mod unix {
    use std::{fs, time::Duration};

    use claude_code::ClaudeClient;
    use tempfile::TempDir;

    #[tokio::test]
    async fn doctor_sends_newline_so_command_exits() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let script_path = dir.path().join("fake-claude");

        let script = r#"#!/bin/sh
set -eu
if [ "${1:-}" != "doctor" ]; then
  echo "expected 'doctor' arg, got: ${1:-<none>}" >&2
  exit 10
fi
echo "Press Enter to continue…" >&2
if ! IFS= read -r _line; then
  echo "stdin closed without newline" >&2
  exit 42
fi
echo "ok"
"#;

        fs::write(&script_path, script).expect("write script");
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");

        let client = ClaudeClient::builder()
            .binary(&script_path)
            .timeout(Some(Duration::from_secs(2)))
            .build();

        let out = client.doctor().await.expect("doctor run");
        assert!(
            out.status.success(),
            "expected success, got {:?}",
            out.status
        );
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "ok");
    }
}
