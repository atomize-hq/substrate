#[cfg(unix)]
mod unix {
    use std::{fs, time::Duration};

    use claude_code::{ClaudeClient, ClaudeSetupTokenRequest};
    use tempfile::TempDir;

    #[tokio::test]
    async fn setup_token_extracts_url_and_accepts_code() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let script_path = dir.path().join("fake-claude");

        let script = r#"#!/bin/sh
set -eu
if [ "${1:-}" != "setup-token" ]; then
  echo "expected 'setup-token' arg, got: ${1:-<none>}" >&2
  exit 10
fi

cat >&2 <<'EOF'
Browser didn't open? Use the url below to sign in (c to copy)

https://claude.ai/oauth/authorize?code=true&client_id=abc&response_type=c
ode&redirect_uri=https%3A%2F%2Fplatform.claude.com%2Foauth%2Fcode%2Fcallback&scope=user%3Ainference

Paste code here if prompted >
EOF

IFS= read -r code
if [ "$code" != "my-code" ]; then
  echo "unexpected code: $code" >&2
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

        let mut session = client
            .setup_token_start_with(ClaudeSetupTokenRequest::new().timeout(None))
            .await
            .expect("start");

        let url = session
            .wait_for_url(Duration::from_secs(2))
            .await
            .expect("wait url")
            .expect("url present");
        assert!(url.starts_with("https://claude.ai/oauth/authorize?"));

        let out = session.submit_code("my-code").await.expect("submit");
        assert!(out.status.success());
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("ok"),
            "expected stdout to contain ok; got: {stdout:?}"
        );
    }
}
