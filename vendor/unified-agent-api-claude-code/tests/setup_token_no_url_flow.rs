#[cfg(unix)]
mod unix {
    use std::{fs, time::Duration};

    use claude_code::{ClaudeClient, ClaudeSetupTokenRequest};
    use tempfile::TempDir;

    #[tokio::test]
    async fn setup_token_allows_missing_url_and_exits_cleanly() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let script_path = dir.path().join("fake-claude");

        let script = r#"#!/bin/sh
set -eu
if [ "${1:-}" != "setup-token" ]; then
  echo "expected 'setup-token' arg, got: ${1:-<none>}" >&2
  exit 10
fi
echo "opened browser"
exit 0
"#;

        fs::write(&script_path, script).expect("write script");
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");

        let client = ClaudeClient::builder()
            .binary(&script_path)
            .timeout(Some(Duration::from_secs(2)))
            .build();

        let mut session = client
            .setup_token_start_with(ClaudeSetupTokenRequest::new().timeout(None))
            .await
            .expect("start");

        let url = session
            .wait_for_url(Duration::from_millis(50))
            .await
            .expect("wait url");
        assert!(url.is_none(), "expected no URL, got {url:?}");

        let out = session.wait().await.expect("wait");
        assert!(
            out.status.success(),
            "expected success, got {:?}",
            out.status
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains("opened browser"));
    }
}
