use super::*;

#[cfg(unix)]
#[tokio::test]
async fn app_server_codegen_maps_overrides_and_prettier() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD"
printf "%s\n" "$@"
"#,
    );

    let workdir = dir.path().join("workdir");
    std_fs::create_dir_all(&workdir).unwrap();
    let out_dir = dir.path().join("out/ts");
    let prettier = dir.path().join("bin/prettier.js");

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .working_dir(&workdir)
        .approval_policy(ApprovalPolicy::OnRequest)
        .search(true)
        .build();

    let result = client
        .generate_app_server_bindings(
            AppServerCodegenRequest::typescript(&out_dir)
                .prettier(&prettier)
                .profile("dev")
                .config_override("features.codegen", "true"),
        )
        .await
        .unwrap();

    let mut lines = result.stdout.lines();
    let pwd = lines.next().unwrap();
    let pwd = std_fs::canonicalize(Path::new(pwd)).unwrap();
    let workdir = std_fs::canonicalize(&workdir).unwrap();
    assert_eq!(pwd, workdir);

    let args: Vec<_> = lines.map(str::to_string).collect();
    assert_eq!(
        args,
        vec![
            "--config",
            "features.codegen=true",
            "--profile",
            "dev",
            "--ask-for-approval",
            "on-request",
            "--search",
            "app-server",
            "generate-ts",
            "--out",
            out_dir.to_string_lossy().as_ref(),
            "--prettier",
            prettier.to_string_lossy().as_ref(),
        ]
    );
    assert!(out_dir.is_dir());
    assert_eq!(result.out_dir, out_dir);
    assert!(result.status.success());
}

#[cfg(unix)]
#[tokio::test]
async fn app_server_codegen_surfaces_non_zero_exit() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "ts error"
echo "bad format" 1>&2
exit 5
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let out_dir = dir.path().join("schema");
    let err = client
        .generate_app_server_bindings(AppServerCodegenRequest::json_schema(&out_dir))
        .await
        .unwrap_err();

    match err {
        CodexError::NonZeroExit { status, stderr } => {
            assert_eq!(status.code(), Some(5));
            assert!(stderr.contains("bad format"));
        }
        other => panic!("expected NonZeroExit, got {other:?}"),
    }
    assert!(out_dir.is_dir());
}

#[cfg(unix)]
#[tokio::test]
async fn app_server_proxy_maps_sock_and_overrides() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("proxy.sock");
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD"
printf "%s\n" "$@"
"#,
    );

    let client_workdir = dir.path().join("client-workdir");
    let request_workdir = dir.path().join("proxy-workdir");
    std_fs::create_dir_all(&client_workdir).unwrap();
    std_fs::create_dir_all(&request_workdir).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .working_dir(&client_workdir)
        .approval_policy(ApprovalPolicy::OnRequest)
        .build();

    let mut child = client
        .start_app_server_proxy(
            AppServerProxyRequest::new()
                .socket_path(&socket_path)
                .working_dir(&request_workdir)
                .profile("dev")
                .config_override("features.proxy", "true")
                .search(true),
        )
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let pwd = lines.next_line().await.unwrap().unwrap();
    let pwd = std_fs::canonicalize(Path::new(&pwd)).unwrap();
    let request_workdir = std_fs::canonicalize(&request_workdir).unwrap();
    assert_eq!(pwd, request_workdir);

    let mut args = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        args.push(line);
    }
    assert_eq!(
        args,
        vec![
            "--config",
            "features.proxy=true",
            "--profile",
            "dev",
            "--ask-for-approval",
            "on-request",
            "--search",
            "app-server",
            "proxy",
            "--sock",
            socket_path.to_string_lossy().as_ref(),
        ]
    );

    let status = child.wait().await.unwrap();
    assert!(status.success());
}

#[cfg(unix)]
#[tokio::test]
async fn responses_api_proxy_maps_flags_and_parses_server_info() {
    let dir = tempfile::tempdir().unwrap();
    let server_info = dir.path().join("server-info.json");
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD"
printf "%s\n" "$@"
info_path=""
while [[ $# -gt 0 ]]; do
  if [[ $1 == "--server-info" ]]; then
info_path=$2
  fi
  shift
done
read -r key || exit 1
echo "key:${key}"
if [[ -n "$info_path" ]]; then
  printf '{"port":4567,"pid":1234}\n' > "$info_path"
fi
"#,
    );

    let workdir = dir.path().join("responses-workdir");
    std_fs::create_dir_all(&workdir).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .working_dir(&workdir)
        .build();

    let mut proxy = client
        .start_responses_api_proxy(
            ResponsesApiProxyRequest::new("sk-test-123")
                .port(8080)
                .server_info(&server_info)
                .http_shutdown(true)
                .upstream_url("https://example.com/v1/responses"),
        )
        .await
        .unwrap();

    assert_eq!(
        proxy.server_info_path.as_deref(),
        Some(server_info.as_path())
    );

    let stdout = proxy.child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let pwd = lines.next_line().await.unwrap().unwrap();
    let pwd = std_fs::canonicalize(Path::new(&pwd)).unwrap();
    let workdir = std_fs::canonicalize(&workdir).unwrap();
    assert_eq!(pwd, workdir);

    let mut args = Vec::new();
    for _ in 0..8 {
        args.push(lines.next_line().await.unwrap().unwrap());
    }
    assert_eq!(
        args,
        vec![
            "responses-api-proxy",
            "--port",
            "8080",
            "--server-info",
            server_info.to_string_lossy().as_ref(),
            "--http-shutdown",
            "--upstream-url",
            "https://example.com/v1/responses",
        ]
    );

    let api_key_line = lines.next_line().await.unwrap().unwrap();
    assert_eq!(api_key_line, "key:sk-test-123");

    let info = proxy.read_server_info().await.unwrap().unwrap();
    assert_eq!(info.port, 4567);
    assert_eq!(info.pid, 1234);

    let status = proxy.child.wait().await.unwrap();
    assert!(status.success());
}

#[tokio::test]
async fn responses_api_proxy_rejects_empty_api_key() {
    let client = CodexClient::builder().build();
    let err = client
        .start_responses_api_proxy(ResponsesApiProxyRequest::new("  "))
        .await
        .unwrap_err();
    assert!(matches!(err, CodexError::EmptyApiKey));
}

#[cfg(unix)]
#[tokio::test]
async fn exec_server_maps_overrides_and_request_workdir() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD"
printf "%s\n" "$@"
"#,
    );

    let request_workdir = dir.path().join("exec-server-workdir");
    std_fs::create_dir_all(&request_workdir).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .approval_policy(ApprovalPolicy::OnRequest)
        .build();

    let mut child = client
        .start_exec_server(
            ExecServerRequest::new()
                .working_dir(&request_workdir)
                .profile("dev")
                .config_override("exec.mode", "server")
                .search(true),
        )
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let pwd = lines.next_line().await.unwrap().unwrap();
    let pwd = std_fs::canonicalize(Path::new(&pwd)).unwrap();
    let request_workdir = std_fs::canonicalize(&request_workdir).unwrap();
    assert_eq!(pwd, request_workdir);

    let mut args = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        args.push(line);
    }
    assert_eq!(
        args,
        vec![
            "--config",
            "exec.mode=server",
            "--profile",
            "dev",
            "--ask-for-approval",
            "on-request",
            "--search",
            "exec-server",
        ]
    );

    let status = child.wait().await.unwrap();
    assert!(status.success());
}

#[cfg(unix)]
#[tokio::test]
async fn stdio_to_uds_maps_args_and_pipes_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("bridge.sock");
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD"
printf "%s\n" "$@"
while read -r line; do
  echo "relay:${line}"
done
"#,
    );

    let workdir = dir.path().join("uds-workdir");
    std_fs::create_dir_all(&workdir).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .working_dir(&workdir)
        .build();

    let request = StdioToUdsRequest::new(&socket_path).working_dir(&workdir);
    let mut child = match client.stdio_to_uds(request.clone()) {
        Ok(child) => child,
        Err(CodexError::Spawn { source, .. }) if source.raw_os_error() == Some(26) => {
            time::sleep(Duration::from_millis(25)).await;
            client.stdio_to_uds(request).unwrap()
        }
        Err(other) => panic!("unexpected spawn error: {other:?}"),
    };

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let pwd = lines.next_line().await.unwrap().unwrap();
    let pwd = std_fs::canonicalize(Path::new(&pwd)).unwrap();
    let workdir = std_fs::canonicalize(&workdir).unwrap();
    assert_eq!(pwd, workdir);

    let arg_one = lines.next_line().await.unwrap().unwrap();
    let arg_two = lines.next_line().await.unwrap().unwrap();
    assert_eq!(arg_one, "stdio-to-uds");
    assert_eq!(arg_two, socket_path.to_string_lossy().as_ref());

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"ping\n").await.unwrap();
    stdin.shutdown().await.unwrap();
    drop(stdin);

    let echoed = lines.next_line().await.unwrap().unwrap();
    assert_eq!(echoed, "relay:ping");

    let status = time::timeout(Duration::from_secs(5), child.wait())
        .await
        .expect("stdio-to-uds wait timed out")
        .unwrap();
    assert!(status.success());
}

#[tokio::test]
async fn stdio_to_uds_rejects_empty_socket_path() {
    let client = CodexClient::builder().build();
    let err = client
        .stdio_to_uds(StdioToUdsRequest::new(PathBuf::new()))
        .unwrap_err();
    assert!(matches!(err, CodexError::EmptySocketPath));
}
