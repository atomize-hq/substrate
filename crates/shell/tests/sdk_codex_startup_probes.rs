#![cfg(unix)]

use agent_api::backends::codex::{CodexBackend, CodexBackendConfig};
use agent_api::{AgentWrapperBackend, AgentWrapperRunRequest};
use futures_util::StreamExt;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::test]
async fn test_codex_add_dirs_reuses_one_cwd_bound_probe_pass() {
    for binary_kind in ["absolute", "relative", "path"] {
        let temp = tempfile::tempdir().unwrap();
        let working = fs::canonicalize(temp.path()).unwrap();
        let log = working.join("invocations");
        let binary = working.join("fake-codex");
        fs::write(
            &binary,
            r#"#!/bin/sh
printf '%s\t%s\t%s\n' "$PWD" "$PROBE_CONTEXT" "$*" >> "$PROBE_LOG"
case "$*" in
  --version) echo 'codex 0.9.0' ;;
  'features list --json')
    if [ "$PROBE_ADD_DIR" = 1 ]; then echo '{"features":["add_dir"]}'; else echo '{"features":[]}'; fi ;;
  'features list') echo 'output_schema' ;;
  --help)
    if [ "$PROBE_ADD_DIR" = 1 ]; then echo 'Usage: codex --add-dir'; else echo 'Usage: codex exec'; fi ;;
  *) cat >/dev/null; exit 99 ;;
esac
"#,
        )
        .unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let configured_binary = match binary_kind {
            "absolute" => binary,
            "relative" => PathBuf::from("./fake-codex"),
            _ => PathBuf::from("fake-codex"),
        };
        let backend = CodexBackend::new(CodexBackendConfig {
            binary: Some(configured_binary),
            codex_home: Some(working.join("native-home")),
            ..Default::default()
        });
        for supported in [true, false] {
            fs::write(&log, []).unwrap();
            let request = AgentWrapperRunRequest {
                prompt: "synthetic startup probe".into(),
                working_dir: Some(working.clone()),
                timeout: Some(Duration::from_secs(2)),
                env: BTreeMap::from([
                    ("PROBE_LOG".into(), log.display().to_string()),
                    ("PROBE_CONTEXT".into(), binary_kind.into()),
                    ("PROBE_ADD_DIR".into(), u8::from(supported).to_string()),
                    (
                        "PATH".into(),
                        format!("{}:/usr/bin:/bin", working.display()),
                    ),
                ]),
                extensions: BTreeMap::from([(
                    "agent_api.exec.add_dirs.v1".into(),
                    json!({"dirs": [&working]}),
                )]),
            };
            let handle = backend.run(request).await.unwrap();
            tokio::time::timeout(Duration::from_secs(10), handle.events.collect::<Vec<_>>())
                .await
                .unwrap();
            let completion = tokio::time::timeout(Duration::from_secs(10), handle.completion)
                .await
                .unwrap();
            if supported {
                assert!(
                    !completion.unwrap().status.success(),
                    "fake exec must return exit 99"
                );
            } else {
                assert_eq!(
                    completion.unwrap_err().to_string(),
                    "backend error: add_dirs rejected by runtime"
                );
            }
            let invocations = fs::read_to_string(&log).unwrap();
            let rows: Vec<_> = invocations
                .lines()
                .map(|line| line.splitn(3, '\t').collect::<Vec<_>>())
                .collect();
            assert_eq!(
                rows.len(),
                4,
                "one probe pass and only supported exec: {invocations}"
            );
            for row in &rows {
                assert_eq!(row[0], working.to_str().unwrap(), "{invocations}");
                assert_eq!(row[1], binary_kind, "{invocations}");
            }
            assert_eq!(rows[0][2], "--version");
            assert_eq!(rows[1][2], "features list --json");
            if supported {
                assert_eq!(rows[2][2], "--help");
                assert!(rows[3][2].split_whitespace().any(|arg| arg == "exec"));
            } else {
                assert_eq!(rows[2][2], "features list");
                assert_eq!(rows[3][2], "--help");
            }
        }
    }
}
