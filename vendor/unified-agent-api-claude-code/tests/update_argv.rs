use claude_code::ClaudeUpdateRequest;

#[test]
fn update_argv() {
    let argv = ClaudeUpdateRequest::new().into_command().argv();
    assert_eq!(argv, ["update"]);
}
