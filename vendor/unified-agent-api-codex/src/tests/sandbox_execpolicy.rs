use super::*;

#[cfg(unix)]
#[tokio::test]
async fn sandbox_maps_platform_flags_and_command() {
    let _guard = env_guard_async().await;
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD"
printf "%s\n" "$@"
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let request = SandboxCommandRequest::new(
        SandboxPlatform::Linux,
        [OsString::from("echo"), OsString::from("hello world")],
    )
    .full_auto(true)
    .log_denials(true)
    .config_override("foo", "bar")
    .enable_feature("alpha")
    .disable_feature("beta");

    let run = client.run_sandbox(request).await.unwrap();
    let mut lines = run.stdout.lines();
    let pwd = lines.next().unwrap();
    assert_eq!(Path::new(pwd), env::current_dir().unwrap().as_path());

    let args: Vec<_> = lines.map(str::to_string).collect();
    assert!(!args.contains(&"--log-denials".to_string()));
    assert_eq!(
        args,
        vec![
            "sandbox",
            "linux",
            "--full-auto",
            "--config",
            "foo=bar",
            "--enable",
            "alpha",
            "--disable",
            "beta",
            "--",
            "echo",
            "hello world"
        ]
    );
    assert!(run.status.success());
}

#[cfg(unix)]
#[tokio::test]
async fn sandbox_includes_log_denials_on_macos() {
    let _guard = env_guard_async().await;
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

    let run = client
        .run_sandbox(SandboxCommandRequest::new(SandboxPlatform::Macos, ["ls"]).log_denials(true))
        .await
        .unwrap();
    let args: Vec<_> = run.stdout.lines().collect();
    assert!(args.contains(&"--log-denials"));
    assert_eq!(args[0], "sandbox");
    assert_eq!(args[1], "macos");
}

#[cfg(unix)]
#[tokio::test]
async fn sandbox_honors_working_dir_precedence() {
    let _guard = env_guard_async().await;
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD"
"#,
    );

    let request_dir = dir.path().join("request_cwd");
    let builder_dir = dir.path().join("builder_cwd");
    std_fs::create_dir_all(&request_dir).unwrap();
    std_fs::create_dir_all(&builder_dir).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .working_dir(&builder_dir)
        .build();

    let run_request = client
        .run_sandbox(
            SandboxCommandRequest::new(SandboxPlatform::Windows, ["echo", "cwd"])
                .working_dir(&request_dir),
        )
        .await
        .unwrap();
    let request_pwd = run_request.stdout.lines().next().unwrap();
    let request_pwd = std_fs::canonicalize(Path::new(request_pwd)).unwrap();
    let request_dir = std_fs::canonicalize(&request_dir).unwrap();
    assert_eq!(request_pwd, request_dir);

    let run_builder = client
        .run_sandbox(SandboxCommandRequest::new(
            SandboxPlatform::Windows,
            ["echo", "builder"],
        ))
        .await
        .unwrap();
    let builder_pwd = run_builder.stdout.lines().next().unwrap();
    let builder_pwd = std_fs::canonicalize(Path::new(builder_pwd)).unwrap();
    let builder_dir = std_fs::canonicalize(&builder_dir).unwrap();
    assert_eq!(builder_pwd, builder_dir);

    let client_default = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();
    let run_default = client_default
        .run_sandbox(SandboxCommandRequest::new(
            SandboxPlatform::Windows,
            ["echo", "default"],
        ))
        .await
        .unwrap();
    let default_pwd = run_default.stdout.lines().next().unwrap();
    assert_eq!(
        Path::new(default_pwd),
        env::current_dir().unwrap().as_path()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn sandbox_returns_non_zero_status_without_error() {
    let _guard = env_guard_async().await;
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "failing"
exit 7
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();
    let run = client
        .run_sandbox(SandboxCommandRequest::new(
            SandboxPlatform::Linux,
            ["false"],
        ))
        .await
        .unwrap();

    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(7));
    assert_eq!(run.stdout.trim(), "failing");
}

#[cfg(unix)]
#[tokio::test]
async fn execpolicy_maps_policies_and_overrides() {
    let _guard = env_guard_async().await;
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("codex-execpolicy");
    std_fs::write(
        &script_path,
        r#"#!/usr/bin/env bash
printf "%s\n" "$PWD" "$@" 1>&2
cat <<'JSON'
{"match":{"decision":"prompt","rules":[{"name":"rule1","decision":"forbidden"}]}}
JSON
"#,
    )
    .unwrap();
    let mut perms = std_fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    std_fs::set_permissions(&script_path, perms).unwrap();

    let workdir = dir.path().join("workdir");
    std_fs::create_dir_all(&workdir).unwrap();
    let policy_one = dir.path().join("policy_a.codexpolicy");
    let policy_two = dir.path().join("policy_b.codexpolicy");
    std_fs::write(&policy_one, "").unwrap();
    std_fs::write(&policy_two, "").unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .working_dir(&workdir)
        .approval_policy(ApprovalPolicy::OnRequest)
        .build();

    let result = client
        .check_execpolicy(
            ExecPolicyCheckRequest::new([
                OsString::from("bash"),
                OsString::from("-lc"),
                OsString::from("echo ok"),
            ])
            .policies([&policy_one, &policy_two])
            .pretty(true)
            .profile("dev")
            .config_override("features.execpolicy", "true"),
        )
        .await
        .unwrap();

    assert_eq!(result.decision(), Some(ExecPolicyDecision::Prompt));
    let match_result = result.evaluation.match_result.unwrap();
    assert_eq!(match_result.rules.len(), 1);
    assert_eq!(match_result.rules[0].name.as_deref(), Some("rule1"));
    assert_eq!(
        match_result.rules[0].decision,
        Some(ExecPolicyDecision::Forbidden)
    );

    let mut lines = result.stderr.lines();
    let pwd = lines.next().unwrap();
    let pwd = std_fs::canonicalize(Path::new(pwd)).unwrap();
    let workdir = std_fs::canonicalize(&workdir).unwrap();
    assert_eq!(pwd, workdir);

    let args: Vec<_> = lines.map(str::to_string).collect();
    assert_eq!(
        args,
        vec![
            "--config",
            "features.execpolicy=true",
            "--profile",
            "dev",
            "--ask-for-approval",
            "on-request",
            "execpolicy",
            "check",
            "--policy",
            policy_one.to_string_lossy().as_ref(),
            "--policy",
            policy_two.to_string_lossy().as_ref(),
            "--pretty",
            "--",
            "bash",
            "-lc",
            "echo ok"
        ]
    );
}

#[tokio::test]
async fn execpolicy_rejects_empty_command() {
    let _guard = env_guard_async().await;
    let client = CodexClient::builder().build();
    let request = ExecPolicyCheckRequest::new(Vec::<OsString>::new());
    let err = client.check_execpolicy(request).await.unwrap_err();
    assert!(matches!(err, CodexError::EmptyExecPolicyCommand));
}

#[cfg(unix)]
#[tokio::test]
async fn execpolicy_surfaces_parse_errors() {
    let _guard = env_guard_async().await;
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("codex-execpolicy-bad");
    std_fs::write(
        &script_path,
        r#"#!/usr/bin/env bash
echo "not-json"
"#,
    )
    .unwrap();
    let mut perms = std_fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    std_fs::set_permissions(&script_path, perms).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let err = client
        .check_execpolicy(
            ExecPolicyCheckRequest::new([OsString::from("echo"), OsString::from("noop")])
                .policy(dir.path().join("policy.codexpolicy")),
        )
        .await
        .unwrap_err();

    match err {
        CodexError::ExecPolicyParse { stdout, .. } => assert!(stdout.contains("not-json")),
        other => panic!("expected ExecPolicyParse, got {other:?}"),
    }
}

#[tokio::test]
async fn sandbox_rejects_empty_command() {
    let _guard = env_guard_async().await;
    let client = CodexClient::builder().build();
    let request = SandboxCommandRequest::new(SandboxPlatform::Linux, Vec::<OsString>::new());
    let err = client.run_sandbox(request).await.unwrap_err();
    assert!(matches!(err, CodexError::EmptySandboxCommand));
}
