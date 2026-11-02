use crate::{AgentWrapperCompletion, AgentWrapperEvent, AgentWrapperEventKind};

pub(crate) const CHANNEL_BOUND_BYTES: usize = 128;
pub(crate) const TEXT_BOUND_BYTES: usize = 65_536;
pub(crate) const MESSAGE_BOUND_BYTES: usize = 4_096;
pub(crate) const DATA_BOUND_BYTES: usize = 65_536;
#[allow(dead_code)]
pub(crate) const MCP_STDOUT_BOUND_BYTES: usize = TEXT_BOUND_BYTES;
#[allow(dead_code)]
pub(crate) const MCP_STDERR_BOUND_BYTES: usize = TEXT_BOUND_BYTES;
pub(crate) const TRUNCATION_SUFFIX: &str = "…(truncated)";

pub(crate) fn enforce_event_bounds(event: AgentWrapperEvent) -> Vec<AgentWrapperEvent> {
    let mut event = event;
    event.channel = enforce_channel_bound(event.channel);
    event.message = event.message.map(enforce_message_bound);
    event.data = event.data.map(enforce_data_bound);

    if event.kind != AgentWrapperEventKind::TextOutput {
        return vec![event];
    }

    let Some(text) = event.text.clone() else {
        return vec![event];
    };

    if text.len() <= TEXT_BOUND_BYTES {
        return vec![event];
    }

    split_utf8_chunks(&text, TEXT_BOUND_BYTES)
        .into_iter()
        .map(|chunk| {
            let mut e = event.clone();
            e.text = Some(chunk);
            e
        })
        .collect()
}

pub(crate) fn enforce_completion_bounds(
    completion: AgentWrapperCompletion,
) -> AgentWrapperCompletion {
    let mut completion = completion;
    completion.data = completion.data.map(enforce_data_bound);
    completion
}

pub(crate) fn enforce_final_text_bound(text: Option<String>) -> Option<String> {
    let text = text?;
    if text.len() <= TEXT_BOUND_BYTES {
        return Some(text);
    }

    Some(truncate_text_with_suffix(&text, TEXT_BOUND_BYTES))
}

/// Enforces the pinned MM-C04 UTF-8-safe output bound used by MCP command stdout/stderr.
///
/// Backend mappings should pass the bounded captured bytes plus their `saw_more_bytes` signal
/// here instead of re-implementing truncation semantics locally.
#[allow(dead_code)]
pub(crate) fn enforce_mcp_output_bound(
    bytes: &[u8],
    saw_more_bytes: bool,
    bound_bytes: usize,
) -> (String, bool) {
    let decoded = String::from_utf8_lossy(bytes);
    let truncated = saw_more_bytes || decoded.len() > bound_bytes;
    if truncated {
        (
            truncate_text_with_suffix(decoded.as_ref(), bound_bytes),
            true,
        )
    } else {
        (decoded.into_owned(), false)
    }
}

fn enforce_channel_bound(channel: Option<String>) -> Option<String> {
    let channel = channel?;
    if channel.len() <= CHANNEL_BOUND_BYTES {
        Some(channel)
    } else {
        None
    }
}

fn enforce_message_bound(message: String) -> String {
    if message.len() <= MESSAGE_BOUND_BYTES {
        return message;
    }

    truncate_text_with_suffix(&message, MESSAGE_BOUND_BYTES)
}

fn enforce_data_bound(data: serde_json::Value) -> serde_json::Value {
    let bytes = serde_json::to_vec(&data)
        .map(|v| v.len())
        .unwrap_or(usize::MAX);
    if bytes <= DATA_BOUND_BYTES {
        data
    } else {
        serde_json::json!({ "dropped": { "reason": "oversize" } })
    }
}

fn split_utf8_chunks(text: &str, bound_bytes: usize) -> Vec<String> {
    if bound_bytes == 0 {
        return Vec::new();
    }
    if text.len() <= bound_bytes {
        return vec![text.to_string()];
    }

    let mut out = Vec::new();
    let mut start = 0usize;
    while start < text.len() {
        let mut end = std::cmp::min(start + bound_bytes, text.len());
        while end > start && !text.is_char_boundary(end) {
            end -= 1;
        }
        if end == start {
            let ch_len = text[start..]
                .chars()
                .next()
                .map(|ch| ch.len_utf8())
                .unwrap_or(1);
            end = std::cmp::min(start + ch_len, text.len());
        }
        out.push(text[start..end].to_string());
        start = end;
    }
    out
}

fn truncate_text_with_suffix(text: &str, bound_bytes: usize) -> String {
    let suffix_bytes = TRUNCATION_SUFFIX.len();
    if bound_bytes > suffix_bytes {
        let prefix = utf8_truncate_to_bytes(text, bound_bytes - suffix_bytes);
        let mut out = String::with_capacity(bound_bytes);
        out.push_str(prefix);
        out.push_str(TRUNCATION_SUFFIX);
        out
    } else {
        utf8_truncate_to_bytes("…", bound_bytes).to_string()
    }
}

fn utf8_truncate_to_bytes(s: &str, bound_bytes: usize) -> &str {
    if s.len() <= bound_bytes {
        return s;
    }
    let mut end = std::cmp::min(bound_bytes, s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentWrapperEventKind, AgentWrapperKind};

    fn success_exit_status() -> std::process::ExitStatus {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(0)
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(0)
        }
    }

    #[test]
    fn channel_over_bound_is_dropped() {
        let event = AgentWrapperEvent {
            agent_kind: AgentWrapperKind("codex".to_string()),
            kind: AgentWrapperEventKind::Status,
            channel: Some("a".repeat(CHANNEL_BOUND_BYTES + 1)),
            text: None,
            message: None,
            data: None,
        };
        let out = enforce_event_bounds(event);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].channel, None);
    }

    #[test]
    fn message_over_bound_is_truncated_with_suffix() {
        let event = AgentWrapperEvent {
            agent_kind: AgentWrapperKind("codex".to_string()),
            kind: AgentWrapperEventKind::Error,
            channel: None,
            text: None,
            message: Some("a".repeat(MESSAGE_BOUND_BYTES + 10)),
            data: None,
        };
        let out = enforce_event_bounds(event);
        assert_eq!(out.len(), 1);
        let message = out[0].message.as_deref().expect("message");
        assert!(message.len() <= MESSAGE_BOUND_BYTES);
        assert!(message.ends_with(TRUNCATION_SUFFIX));
    }

    #[test]
    fn text_over_bound_is_split_deterministically() {
        let text = "a".repeat(TEXT_BOUND_BYTES + 10);
        let event = AgentWrapperEvent {
            agent_kind: AgentWrapperKind("codex".to_string()),
            kind: AgentWrapperEventKind::TextOutput,
            channel: Some("assistant".to_string()),
            text: Some(text.clone()),
            message: None,
            data: None,
        };
        let out = enforce_event_bounds(event);
        assert!(out.len() >= 2);
        for e in out.iter() {
            let t = e.text.as_deref().expect("text");
            assert!(t.len() <= TEXT_BOUND_BYTES);
        }
        let recombined: String = out
            .iter()
            .map(|e| e.text.as_deref().unwrap())
            .collect::<Vec<_>>()
            .join("");
        assert_eq!(recombined, text);
    }

    #[test]
    fn data_over_bound_is_replaced_with_dropped_reason() {
        let large = serde_json::Value::String("a".repeat(DATA_BOUND_BYTES + 10));
        let event = AgentWrapperEvent {
            agent_kind: AgentWrapperKind("codex".to_string()),
            kind: AgentWrapperEventKind::ToolCall,
            channel: None,
            text: None,
            message: None,
            data: Some(large),
        };
        let out = enforce_event_bounds(event);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].data.as_ref().and_then(|v| v.get("dropped")),
            Some(&serde_json::json!({ "reason": "oversize" }))
        );
    }

    #[test]
    fn completion_data_over_bound_is_replaced_with_dropped_reason() {
        let completion = AgentWrapperCompletion {
            status: success_exit_status(),
            final_text: None,
            data: Some(serde_json::Value::String("a".repeat(DATA_BOUND_BYTES + 10))),
        };
        let bounded = enforce_completion_bounds(completion);
        assert_eq!(
            bounded.data.as_ref().and_then(|v| v.get("dropped")),
            Some(&serde_json::json!({ "reason": "oversize" }))
        );
    }

    #[test]
    fn final_text_over_bound_is_truncated_with_suffix_utf8_safely() {
        let text = "💖".repeat((TEXT_BOUND_BYTES / 4) + 100);
        assert!(text.len() > TEXT_BOUND_BYTES);

        let out = enforce_final_text_bound(Some(text)).expect("final_text present");
        assert!(out.len() <= TEXT_BOUND_BYTES);
        assert!(out.ends_with(TRUNCATION_SUFFIX));
    }

    #[test]
    fn mcp_output_under_bound_is_not_truncated() {
        let (out, truncated) = enforce_mcp_output_bound(b"plain output", false, 32);
        assert_eq!(out, "plain output");
        assert!(!truncated);
    }

    #[test]
    fn mcp_output_over_bound_is_truncated_with_suffix() {
        let bytes = vec![b'a'; TEXT_BOUND_BYTES + 1];
        let (out, truncated) = enforce_mcp_output_bound(&bytes, false, MCP_STDOUT_BOUND_BYTES);

        assert!(truncated);
        assert!(out.len() <= MCP_STDOUT_BOUND_BYTES);
        assert!(out.ends_with(TRUNCATION_SUFFIX));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn mcp_output_truncation_preserves_utf8_boundaries() {
        let bytes = "💖".repeat(10).into_bytes();
        let (out, truncated) = enforce_mcp_output_bound(&bytes, false, 20);

        assert!(truncated);
        assert_eq!(out, format!("💖{TRUNCATION_SUFFIX}"));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn mcp_output_uses_lossy_decode_for_invalid_utf8() {
        let bytes = [b'a', 0xff, 0xfe, b'b'];
        let (out, truncated) = enforce_mcp_output_bound(&bytes, false, 32);

        assert!(!truncated);
        assert_eq!(out, "a��b");
        assert!(out.contains('\u{FFFD}'));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn mcp_output_lossy_decode_can_trigger_truncation_without_saw_more_bytes() {
        let bytes = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let (out, truncated) = enforce_mcp_output_bound(&bytes, false, 20);

        assert!(truncated);
        assert_eq!(out, format!("��{TRUNCATION_SUFFIX}"));
        assert!(out.contains('\u{FFFD}'));
        assert!(out.len() <= 20);
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn mcp_output_saw_more_bytes_forces_truncation_even_when_decoded_text_is_under_bound() {
        let (out, truncated) = enforce_mcp_output_bound(b"ok", true, 20);

        assert!(truncated);
        assert_eq!(out, format!("ok{TRUNCATION_SUFFIX}"));
        assert!(out.len() <= 20);
    }

    #[test]
    fn mcp_output_falls_back_to_truncated_ellipsis_when_bound_is_too_small_for_suffix() {
        let (out, truncated) = enforce_mcp_output_bound(b"abcdef", true, 3);

        assert!(truncated);
        assert_eq!(out, "…");
    }
}
