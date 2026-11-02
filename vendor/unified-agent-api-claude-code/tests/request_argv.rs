use claude_code::{ClaudeInputFormat, ClaudeOutputFormat, ClaudePrintRequest};

#[test]
fn argv_orders_flags_before_prompt() {
    let req = ClaudePrintRequest::new("hello")
        .output_format(ClaudeOutputFormat::StreamJson)
        .input_format(ClaudeInputFormat::Text)
        .json_schema(r#"{"type":"object"}"#)
        .model("sonnet")
        .debug(true);

    let argv = req.argv();
    assert!(argv.starts_with(&[
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--input-format".to_string(),
        "text".to_string(),
        "--json-schema".to_string(),
        r#"{"type":"object"}"#.to_string(),
        "--model".to_string(),
        "sonnet".to_string(),
        "--debug".to_string(),
    ]));
    assert_eq!(argv.last().unwrap(), "hello");
}
