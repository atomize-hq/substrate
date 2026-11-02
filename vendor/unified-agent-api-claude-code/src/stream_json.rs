use serde_json::Value;

use crate::StreamJsonLineError;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ClaudeStreamJsonErrorCode {
    JsonParse,
    TypedParse,
    Normalize,
    Unknown,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct ClaudeStreamJsonParseError {
    pub code: ClaudeStreamJsonErrorCode,
    /// Redacted; MUST NOT embed the full raw line.
    pub message: String,
    /// Potentially richer; intended for sinks. v1 keeps this equal to `message`.
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct ClaudeStreamEvent {
    pub event_type: String,
    pub raw: Value,
}

#[derive(Debug, Clone)]
pub enum ClaudeStreamJsonEvent {
    SystemInit {
        session_id: String,
        raw: Value,
    },
    SystemOther {
        session_id: String,
        subtype: String,
        raw: Value,
    },

    UserMessage {
        session_id: String,
        raw: Value,
    },
    AssistantMessage {
        session_id: String,
        raw: Value,
    },

    ResultSuccess {
        session_id: String,
        raw: Value,
    },
    ResultError {
        session_id: String,
        raw: Value,
    },

    StreamEvent {
        session_id: String,
        stream: ClaudeStreamEvent,
        raw: Value,
    },

    Unknown {
        session_id: Option<String>,
        raw: Value,
    },
}

impl ClaudeStreamJsonEvent {
    pub fn raw(&self) -> &Value {
        match self {
            ClaudeStreamJsonEvent::SystemInit { raw, .. } => raw,
            ClaudeStreamJsonEvent::SystemOther { raw, .. } => raw,
            ClaudeStreamJsonEvent::UserMessage { raw, .. } => raw,
            ClaudeStreamJsonEvent::AssistantMessage { raw, .. } => raw,
            ClaudeStreamJsonEvent::ResultSuccess { raw, .. } => raw,
            ClaudeStreamJsonEvent::ResultError { raw, .. } => raw,
            ClaudeStreamJsonEvent::StreamEvent { raw, .. } => raw,
            ClaudeStreamJsonEvent::Unknown { raw, .. } => raw,
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            ClaudeStreamJsonEvent::SystemInit { session_id, .. } => Some(session_id.as_str()),
            ClaudeStreamJsonEvent::SystemOther { session_id, .. } => Some(session_id.as_str()),
            ClaudeStreamJsonEvent::UserMessage { session_id, .. } => Some(session_id.as_str()),
            ClaudeStreamJsonEvent::AssistantMessage { session_id, .. } => Some(session_id.as_str()),
            ClaudeStreamJsonEvent::ResultSuccess { session_id, .. } => Some(session_id.as_str()),
            ClaudeStreamJsonEvent::ResultError { session_id, .. } => Some(session_id.as_str()),
            ClaudeStreamJsonEvent::StreamEvent { session_id, .. } => Some(session_id.as_str()),
            ClaudeStreamJsonEvent::Unknown { session_id, .. } => session_id.as_deref(),
        }
    }

    pub fn into_raw(self) -> Value {
        match self {
            ClaudeStreamJsonEvent::SystemInit { raw, .. } => raw,
            ClaudeStreamJsonEvent::SystemOther { raw, .. } => raw,
            ClaudeStreamJsonEvent::UserMessage { raw, .. } => raw,
            ClaudeStreamJsonEvent::AssistantMessage { raw, .. } => raw,
            ClaudeStreamJsonEvent::ResultSuccess { raw, .. } => raw,
            ClaudeStreamJsonEvent::ResultError { raw, .. } => raw,
            ClaudeStreamJsonEvent::StreamEvent { raw, .. } => raw,
            ClaudeStreamJsonEvent::Unknown { raw, .. } => raw,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClaudeStreamJsonParser {
    last_session_id: Option<String>,
}

impl ClaudeStreamJsonParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.last_session_id = None;
    }

    pub fn parse_line(
        &mut self,
        line: &str,
    ) -> Result<Option<ClaudeStreamJsonEvent>, ClaudeStreamJsonParseError> {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.chars().all(|ch| ch.is_whitespace()) {
            return Ok(None);
        }
        let value: Value = serde_json::from_str(line).map_err(|err| {
            ClaudeStreamJsonParseError::new(
                ClaudeStreamJsonErrorCode::JsonParse,
                format!("invalid JSON: {err}"),
            )
        })?;
        self.parse_json(&value)
    }

    pub fn parse_json(
        &mut self,
        value: &Value,
    ) -> Result<Option<ClaudeStreamJsonEvent>, ClaudeStreamJsonParseError> {
        let obj = value.as_object().ok_or_else(|| {
            ClaudeStreamJsonParseError::new(
                ClaudeStreamJsonErrorCode::TypedParse,
                "expected JSON object".to_string(),
            )
        })?;

        let outer_type = get_required_str(obj, "type").map_err(|msg| {
            ClaudeStreamJsonParseError::new(ClaudeStreamJsonErrorCode::TypedParse, msg)
        })?;

        let known = matches!(
            outer_type.as_str(),
            "system" | "user" | "assistant" | "result" | "stream_event"
        );

        let session_id = if known {
            Some(get_required_session_id(obj)?)
        } else {
            get_optional_session_id(obj)
        };

        match outer_type.as_str() {
            "system" => {
                let session_id = session_id.expect("known type requires session_id");
                let subtype = get_required_str(obj, "subtype").map_err(|msg| {
                    ClaudeStreamJsonParseError::new(ClaudeStreamJsonErrorCode::TypedParse, msg)
                })?;
                self.last_session_id = Some(session_id.clone());
                if subtype == "init" {
                    Ok(Some(ClaudeStreamJsonEvent::SystemInit {
                        session_id,
                        raw: value.clone(),
                    }))
                } else {
                    Ok(Some(ClaudeStreamJsonEvent::SystemOther {
                        session_id,
                        subtype,
                        raw: value.clone(),
                    }))
                }
            }
            "user" => {
                let session_id = session_id.expect("known type requires session_id");
                self.last_session_id = Some(session_id.clone());
                Ok(Some(ClaudeStreamJsonEvent::UserMessage {
                    session_id,
                    raw: value.clone(),
                }))
            }
            "assistant" => {
                let session_id = session_id.expect("known type requires session_id");
                self.last_session_id = Some(session_id.clone());
                Ok(Some(ClaudeStreamJsonEvent::AssistantMessage {
                    session_id,
                    raw: value.clone(),
                }))
            }
            "result" => {
                let session_id = session_id.expect("known type requires session_id");
                let subtype = get_required_str(obj, "subtype").map_err(|msg| {
                    ClaudeStreamJsonParseError::new(ClaudeStreamJsonErrorCode::TypedParse, msg)
                })?;
                let is_error = get_optional_bool(obj, "is_error").map_err(|msg| {
                    ClaudeStreamJsonParseError::new(ClaudeStreamJsonErrorCode::TypedParse, msg)
                })?;

                let event = match subtype.as_str() {
                    "success" => {
                        if matches!(is_error, Some(true)) {
                            return Err(ClaudeStreamJsonParseError::new(
                                ClaudeStreamJsonErrorCode::Normalize,
                                "result subtype success inconsistent with is_error=true"
                                    .to_string(),
                            ));
                        }
                        ClaudeStreamJsonEvent::ResultSuccess {
                            session_id,
                            raw: value.clone(),
                        }
                    }
                    "error" => {
                        if matches!(is_error, Some(false)) {
                            return Err(ClaudeStreamJsonParseError::new(
                                ClaudeStreamJsonErrorCode::Normalize,
                                "result subtype error inconsistent with is_error=false".to_string(),
                            ));
                        }
                        ClaudeStreamJsonEvent::ResultError {
                            session_id,
                            raw: value.clone(),
                        }
                    }
                    _ => {
                        return Err(ClaudeStreamJsonParseError::new(
                            ClaudeStreamJsonErrorCode::TypedParse,
                            "result subtype must be success or error".to_string(),
                        ));
                    }
                };

                if let ClaudeStreamJsonEvent::ResultSuccess { session_id, .. }
                | ClaudeStreamJsonEvent::ResultError { session_id, .. } = &event
                {
                    self.last_session_id = Some(session_id.clone());
                }

                Ok(Some(event))
            }
            "stream_event" => {
                let session_id = session_id.expect("known type requires session_id");
                let event_obj = obj
                    .get("event")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| {
                        ClaudeStreamJsonParseError::new(
                            ClaudeStreamJsonErrorCode::TypedParse,
                            "missing object field event".to_string(),
                        )
                    })?;
                let event_type = get_required_str(event_obj, "type").map_err(|msg| {
                    ClaudeStreamJsonParseError::new(ClaudeStreamJsonErrorCode::TypedParse, msg)
                })?;

                self.last_session_id = Some(session_id.clone());
                Ok(Some(ClaudeStreamJsonEvent::StreamEvent {
                    session_id,
                    stream: ClaudeStreamEvent {
                        event_type,
                        raw: obj.get("event").expect("exists").clone(),
                    },
                    raw: value.clone(),
                }))
            }
            _ => {
                let session_id = session_id.or_else(|| self.last_session_id.clone());
                Ok(Some(ClaudeStreamJsonEvent::Unknown {
                    session_id,
                    raw: value.clone(),
                }))
            }
        }
    }
}

impl ClaudeStreamJsonParseError {
    fn new(code: ClaudeStreamJsonErrorCode, message: String) -> Self {
        Self {
            code,
            details: message.clone(),
            message,
        }
    }
}

fn get_optional_session_id(obj: &serde_json::Map<String, Value>) -> Option<String> {
    obj.get("session_id")
        .or_else(|| obj.get("sessionId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_required_session_id(
    obj: &serde_json::Map<String, Value>,
) -> Result<String, ClaudeStreamJsonParseError> {
    get_optional_session_id(obj).ok_or_else(|| {
        ClaudeStreamJsonParseError::new(
            ClaudeStreamJsonErrorCode::TypedParse,
            "missing string field session_id (or sessionId)".to_string(),
        )
    })
}

fn get_required_str(obj: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing string field {key}"))
}

fn get_optional_bool(
    obj: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, String> {
    let Some(v) = obj.get(key) else {
        return Ok(None);
    };
    v.as_bool()
        .ok_or_else(|| format!("field {key} must be boolean"))
        .map(Some)
}

#[derive(Debug, Clone)]
pub struct StreamJsonLine {
    pub line_number: usize,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub enum StreamJsonLineOutcome {
    Ok {
        line: StreamJsonLine,
        value: Value,
    },
    Err {
        line: StreamJsonLine,
        error: StreamJsonLineError,
    },
}

/// Legacy convenience helper for parsing Claude stream-json output as raw `serde_json::Value`.
///
/// This function is intentionally **not** the normative parser API. Prefer
/// [`ClaudeStreamJsonParser`] to obtain typed [`ClaudeStreamJsonEvent`] values.
pub fn parse_stream_json_lines(text: &str) -> Vec<StreamJsonLineOutcome> {
    let mut out = Vec::new();
    let mut parser = ClaudeStreamJsonParser::new();
    for (idx, raw) in text.lines().enumerate() {
        let line_number = idx + 1;
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if raw.chars().all(|ch| ch.is_whitespace()) {
            continue;
        }
        let line = StreamJsonLine {
            line_number,
            raw: raw.to_string(),
        };
        match parser.parse_line(&line.raw) {
            Ok(Some(event)) => out.push(StreamJsonLineOutcome::Ok {
                line,
                value: event.into_raw(),
            }),
            Ok(None) => {}
            Err(err) => out.push(StreamJsonLineOutcome::Err {
                line,
                error: StreamJsonLineError {
                    line_number,
                    message: err.message,
                },
            }),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_ignores_blank_lines_without_full_trim() {
        let mut parser = ClaudeStreamJsonParser::new();
        assert!(parser.parse_line("   ").unwrap().is_none());
        assert!(parser.parse_line("\t").unwrap().is_none());
    }

    #[test]
    fn parse_json_matches_parse_line_for_typedparse_and_normalize_codes() {
        let mut parser = ClaudeStreamJsonParser::new();

        let value = serde_json::json!({"type":"user"});
        let err = parser.parse_json(&value).unwrap_err();
        assert_eq!(err.code, ClaudeStreamJsonErrorCode::TypedParse);

        let value = serde_json::json!({"type":"result","subtype":"success","session_id":"s","is_error":true});
        let err = parser.parse_json(&value).unwrap_err();
        assert_eq!(err.code, ClaudeStreamJsonErrorCode::Normalize);
    }

    #[test]
    fn unknown_outer_type_is_not_an_error() {
        let mut parser = ClaudeStreamJsonParser::new();
        let line = r#"{"type":"weird","session_id":"s"}"#;
        let ev = parser.parse_line(line).unwrap().unwrap();
        assert!(matches!(ev, ClaudeStreamJsonEvent::Unknown { .. }));
    }
}
