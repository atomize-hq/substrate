#[cfg(unix)]
mod unix {
    use std::{fs, time::Duration};

    use claude_code::{ClaudeClient, ClaudeOutputFormat, ClaudePrintRequest};
    use tempfile::TempDir;

    #[tokio::test]
    async fn resume_value_allows_missing_prompt() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let script_path = dir.path().join("fake-claude");

        let script = r#"#!/bin/sh
set -eu
case "${1:-}" in
  --print) shift ;;
  *) echo "expected --print, got: ${1:-<none>}" >&2; exit 10 ;;
esac

resume_value=""
while [ "$#" -gt 0 ]; do
  case "${1:-}" in
    --output-format) shift 2 ;;
    --verbose) shift ;;
    --resume)
      resume_value="${2:-}"
      shift 2
      ;;
    *)
      echo "unexpected arg: ${1:-<none>}" >&2
      exit 11
      ;;
  esac
done

if [ -z "$resume_value" ]; then
  echo "missing resume value" >&2
  exit 12
fi

echo '{"type":"result","subtype":"success","session_id":"s","is_error":false}'
"#;

        fs::write(&script_path, script).expect("write script");
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");

        let client = ClaudeClient::builder()
            .binary(&script_path)
            .timeout(Some(Duration::from_secs(2)))
            .build();

        let req = ClaudePrintRequest::new("ignored")
            .no_prompt()
            .output_format(ClaudeOutputFormat::StreamJson)
            .resume_value("abc");

        let out = client.print(req).await.expect("print");
        assert!(out.output.status.success());
        assert!(out.parsed.is_some());
    }
}
