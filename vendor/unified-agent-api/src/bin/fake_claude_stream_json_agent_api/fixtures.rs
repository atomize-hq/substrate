pub(crate) const SYSTEM_INIT: &str = include_str!("fixtures/system_init.jsonl");
pub(crate) const USER_MESSAGE: &str = include_str!("fixtures/user_message.jsonl");
pub(crate) const STREAM_EVENT_TOOL_USE_START: &str =
    include_str!("fixtures/stream_event_tool_use_start.jsonl");
pub(crate) const STREAM_EVENT_INPUT_JSON_DELTA: &str =
    include_str!("fixtures/stream_event_input_json_delta.jsonl");
pub(crate) const STREAM_EVENT_TOOL_RESULT_START: &str =
    include_str!("fixtures/stream_event_tool_result_start.jsonl");
pub(crate) const ASSISTANT_MESSAGE_TEXT: &str =
    include_str!("fixtures/assistant_message_text.jsonl");

pub(crate) fn first_nonempty_line(text: &str) -> &str {
    text.lines()
        .find(|line| !line.chars().all(|ch| ch.is_whitespace()))
        .expect("fixture contains a non-empty line")
}
