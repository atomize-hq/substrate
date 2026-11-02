use super::*;

#[cfg(unix)]
#[tokio::test]
async fn cloud_list_parses_json_and_maps_args() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
printf "%s\n" "$@" 1>&2
cat <<'JSON'
{"tasks":[],"cursor":null}
JSON
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let output = client
        .cloud_list(
            CloudListRequest::new()
                .json(true)
                .env_id("env-1")
                .limit(3)
                .cursor("cur-1"),
        )
        .await
        .unwrap();

    assert_eq!(output.json, Some(json!({"tasks": [], "cursor": null})));
    assert_eq!(
        output.stderr.lines().collect::<Vec<_>>(),
        vec!["cloud", "list", "--env", "env-1", "--limit", "3", "--cursor", "cur-1", "--json"]
    );
}

#[cfg(unix)]
#[tokio::test]
async fn cloud_exec_maps_args_and_rejects_empty_env_id() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
printf "%s\n" "$@"
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let output = client
        .cloud_exec(
            CloudExecRequest::new("env-1")
                .attempts(2)
                .branch("main")
                .query("hello"),
        )
        .await
        .unwrap();
    assert_eq!(
        output.stdout.lines().collect::<Vec<_>>(),
        vec![
            "cloud",
            "exec",
            "--env",
            "env-1",
            "--attempts",
            "2",
            "--branch",
            "main",
            "hello"
        ]
    );

    let err = client
        .cloud_exec(CloudExecRequest::new("  "))
        .await
        .unwrap_err();
    assert!(matches!(err, CodexError::EmptyEnvId));
}
