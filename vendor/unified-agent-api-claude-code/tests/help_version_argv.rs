use claude_code::ClaudeCommandRequest;

#[test]
fn root_help_and_version_argv() {
    assert_eq!(
        ClaudeCommandRequest::root().arg("--help").argv(),
        ["--help"]
    );
    assert_eq!(
        ClaudeCommandRequest::root().arg("--version").argv(),
        ["--version"]
    );
}
