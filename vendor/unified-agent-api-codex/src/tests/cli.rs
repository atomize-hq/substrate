use super::*;

#[cfg(unix)]
#[tokio::test]
async fn features_list_maps_overrides_and_json_flag() {
    let _guard = env_guard_async().await;
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
echo "$PWD" 1>&2
printf "%s\n" "$@" 1>&2
cat <<'JSON'
[{"name":"json-stream","stage":"stable","enabled":true},{"name":"cloud-exec","stage":"experimental","enabled":false}]
JSON
"#,
    );

    let workdir = dir.path().join("features-workdir");
    std_fs::create_dir_all(&workdir).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .working_dir(&workdir)
        .approval_policy(ApprovalPolicy::OnRequest)
        .search(true)
        .build();

    let output = client
        .list_features(
            FeaturesListRequest::new()
                .json(true)
                .profile("dev")
                .config_override("features.extras", "true"),
        )
        .await
        .unwrap();

    assert_eq!(output.format, FeaturesListFormat::Json);
    assert_eq!(output.features.len(), 2);
    assert_eq!(output.features[0].stage, Some(CodexFeatureStage::Stable));
    assert!(output.features[0].enabled);
    assert!(!output.features[1].enabled);

    let mut lines = output.stderr.lines();
    let pwd = lines.next().unwrap();
    let pwd = std_fs::canonicalize(Path::new(pwd)).unwrap();
    let workdir = std_fs::canonicalize(&workdir).unwrap();
    assert_eq!(pwd, workdir);

    let args: Vec<_> = lines.map(str::to_string).collect();
    assert_eq!(
        args,
        vec![
            "--config",
            "features.extras=true",
            "--profile",
            "dev",
            "--ask-for-approval",
            "on-request",
            "--search",
            "features",
            "list",
            "--json"
        ]
    );
}

#[cfg(unix)]
#[tokio::test]
async fn supports_help_review_fork_resume_and_features_commands() {
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

    let features = client
        .features(FeaturesCommandRequest::new())
        .await
        .unwrap();
    assert_eq!(
        features.stdout.lines().collect::<Vec<_>>(),
        vec!["features"]
    );

    let help = client
        .help(HelpCommandRequest::new(HelpScope::Root).command(["exec", "review"]))
        .await
        .unwrap();
    assert_eq!(
        help.stdout.lines().collect::<Vec<_>>(),
        vec!["help", "exec", "review"]
    );

    let review = client
        .review(
            ReviewCommandRequest::new()
                .base("main")
                .commit("abc123")
                .title("hello")
                .uncommitted(true)
                .prompt("please review"),
        )
        .await
        .unwrap();
    assert_eq!(
        review.stdout.lines().collect::<Vec<_>>(),
        vec![
            "review",
            "--base",
            "main",
            "--commit",
            "abc123",
            "--title",
            "hello",
            "--uncommitted",
            "please review"
        ]
    );

    let exec_review = client
        .exec_review(
            ExecReviewCommandRequest::new()
                .base("main")
                .commit("abc123")
                .title("hello")
                .uncommitted(true)
                .json(true)
                .prompt("please review"),
        )
        .await
        .unwrap();
    assert_eq!(
        exec_review.stdout.lines().collect::<Vec<_>>(),
        vec![
            "exec",
            "review",
            "--base",
            "main",
            "--commit",
            "abc123",
            "--json",
            "--skip-git-repo-check",
            "--title",
            "hello",
            "--uncommitted",
            "please review"
        ]
    );

    let resume = client
        .resume_session(
            ResumeSessionRequest::new()
                .all(true)
                .last(true)
                .session_id("sess-1")
                .prompt("resume prompt"),
        )
        .await
        .unwrap();
    assert_eq!(
        resume.stdout.lines().collect::<Vec<_>>(),
        vec!["resume", "--all", "--last", "sess-1", "resume prompt"]
    );

    let fork = client
        .fork_session(
            ForkSessionRequest::new()
                .all(true)
                .last(true)
                .session_id("sess-1")
                .prompt("fork prompt"),
        )
        .await
        .unwrap();
    assert_eq!(
        fork.stdout.lines().collect::<Vec<_>>(),
        vec!["fork", "--all", "--last", "sess-1", "fork prompt"]
    );
}

#[cfg(unix)]
#[tokio::test]
async fn supports_debug_models_prompt_input_and_plugin_commands() {
    let _guard = env_guard_async().await;
    let dir = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        dir.path(),
        r#"#!/usr/bin/env bash
printf "%s\n" "$@"
"#,
    );

    let image_one = dir.path().join("prompt-1.png");
    let image_two = dir.path().join("prompt-2.png");

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let debug_models = client
        .debug_models(DebugModelsRequest::new().bundled(true))
        .await
        .unwrap();
    assert_eq!(
        debug_models.stdout.lines().collect::<Vec<_>>(),
        vec!["debug", "models", "--bundled"]
    );

    let prompt_input = client
        .debug_prompt_input(
            DebugPromptInputRequest::new()
                .image(&image_one)
                .image(&image_two)
                .prompt("hello"),
        )
        .await
        .unwrap();
    assert_eq!(
        prompt_input.stdout.lines().collect::<Vec<_>>(),
        vec![
            "debug",
            "prompt-input",
            "--image",
            image_one.to_string_lossy().as_ref(),
            "--image",
            image_two.to_string_lossy().as_ref(),
            "hello",
        ]
    );

    let plugin = client.plugin(PluginCommandRequest::new()).await.unwrap();
    assert_eq!(plugin.stdout.lines().collect::<Vec<_>>(), vec!["plugin"]);

    let plugin_help = client
        .plugin_help(PluginHelpRequest::new().command(["marketplace", "add"]))
        .await
        .unwrap();
    assert_eq!(
        plugin_help.stdout.lines().collect::<Vec<_>>(),
        vec!["plugin", "help", "marketplace", "add"]
    );

    let plugin_marketplace = client
        .plugin_marketplace(PluginMarketplaceCommandRequest::new())
        .await
        .unwrap();
    assert_eq!(
        plugin_marketplace.stdout.lines().collect::<Vec<_>>(),
        vec!["plugin", "marketplace"]
    );

    let plugin_marketplace_help = client
        .plugin_marketplace_help(PluginMarketplaceHelpRequest::new().command(["upgrade"]))
        .await
        .unwrap();
    assert_eq!(
        plugin_marketplace_help.stdout.lines().collect::<Vec<_>>(),
        vec!["plugin", "marketplace", "help", "upgrade"]
    );

    let plugin_marketplace_add = client
        .plugin_marketplace_add(
            PluginMarketplaceAddRequest::new("owner/repo")
                .source_ref("main")
                .sparse_path("marketplaces/core"),
        )
        .await
        .unwrap();
    assert_eq!(
        plugin_marketplace_add.stdout.lines().collect::<Vec<_>>(),
        vec![
            "plugin",
            "marketplace",
            "add",
            "--ref",
            "main",
            "--sparse",
            "marketplaces/core",
            "owner/repo",
        ]
    );

    let plugin_marketplace_remove = client
        .plugin_marketplace_remove(PluginMarketplaceRemoveRequest::new("primary"))
        .await
        .unwrap();
    assert_eq!(
        plugin_marketplace_remove.stdout.lines().collect::<Vec<_>>(),
        vec!["plugin", "marketplace", "remove", "primary"]
    );

    let plugin_marketplace_upgrade = client
        .plugin_marketplace_upgrade(
            PluginMarketplaceUpgradeRequest::new().marketplace_name("primary"),
        )
        .await
        .unwrap();
    assert_eq!(
        plugin_marketplace_upgrade
            .stdout
            .lines()
            .collect::<Vec<_>>(),
        vec!["plugin", "marketplace", "upgrade", "primary"]
    );
}
