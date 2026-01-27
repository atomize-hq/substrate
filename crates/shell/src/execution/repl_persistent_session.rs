//! Pure protocol core for the world-first REPL persistent session (PROTOCOL v1).
//!
//! This is intentionally transport-agnostic so we can unit test fail-closed behavior without
//! standing up a world-agent WebSocket.

#![allow(dead_code)]

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PersistentSessionProtocolError {
    message: String,
    fatal: bool,
}

impl PersistentSessionProtocolError {
    pub fn fatal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fatal: true,
        }
    }

    pub fn is_fatal(&self) -> bool {
        self.fatal
    }
}

impl std::fmt::Display for PersistentSessionProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PersistentSessionProtocolError {}

#[derive(Debug, Default)]
pub struct PersistentSessionClientCore {
    phase: Phase,
    shutdown_initiated: bool,
    latched_fatal: Option<PersistentSessionProtocolError>,
}

#[derive(Debug, Default)]
enum Phase {
    #[default]
    Init,
    WaitingReady,
    Ready,
    InFlight {
        expected_seq: u64,
        expected_token: String,
    },
    Closed,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerFrame {
    Ready {
        session_nonce: String,
        cwd: String,
        protocol_version: u32,
    },
    Stdout {
        data_b64: String,
    },
    CommandComplete {
        seq: u64,
        token_hex: String,
        exit: i32,
        cwd: String,
    },
    Exit {
        code: i32,
    },
    Error {
        code: String,
        message: String,
        fatal: bool,
        #[serde(default)]
        seq: Option<u64>,
    },
    #[serde(other)]
    Unknown,
}

impl PersistentSessionClientCore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn note_start_session_sent(&mut self) {
        if self.latched_fatal.is_some() {
            return;
        }
        self.phase = Phase::WaitingReady;
    }

    pub fn note_shutdown_initiated(&mut self) {
        self.shutdown_initiated = true;
    }

    pub fn note_exec_in_flight(
        &mut self,
        expected_seq: u64,
        expected_token_hex: &str,
    ) -> Result<(), PersistentSessionProtocolError> {
        if let Some(err) = self.latched_fatal.clone() {
            return Err(err);
        }
        match self.phase {
            Phase::Ready => {}
            Phase::InFlight { .. } => {
                return Err(self.latch_fatal("protocol error: attempted exec while in-flight"));
            }
            Phase::WaitingReady => {
                return Err(self.latch_fatal("protocol error: attempted exec before ready"));
            }
            Phase::Init => {
                return Err(self.latch_fatal("protocol error: attempted exec before start_session"));
            }
            Phase::Closed => {
                return Err(self.latch_fatal("protocol error: attempted exec after close"));
            }
        }

        if !is_hex32_lower(expected_token_hex) {
            return Err(self.latch_fatal("protocol error: expected_token_hex must be hex32"));
        }

        self.phase = Phase::InFlight {
            expected_seq,
            expected_token: expected_token_hex.to_string(),
        };
        Ok(())
    }

    pub fn on_server_frame(
        &mut self,
        frame: serde_json::Value,
    ) -> Result<(), PersistentSessionProtocolError> {
        if let Some(err) = self.latched_fatal.clone() {
            return Err(err);
        }

        let frame: ServerFrame = serde_json::from_value(frame)
            .map_err(|e| self.latch_fatal(format!("protocol error: invalid frame JSON: {e}")))?;

        match frame {
            ServerFrame::Ready {
                session_nonce,
                cwd,
                protocol_version,
            } => {
                if !matches!(self.phase, Phase::WaitingReady) {
                    return Err(self.latch_fatal(
                        "protocol error: unexpected ready frame after session start",
                    ));
                }
                if protocol_version != 1 {
                    return Err(self.latch_fatal(format!(
                        "unsupported persistent session protocol_version={} (expected 1)",
                        protocol_version
                    )));
                }
                if !is_hex32_lower(&session_nonce) {
                    return Err(self.latch_fatal(
                        "protocol error: invalid ready.session_nonce (expected hex32)",
                    ));
                }
                if !cwd.starts_with('/') {
                    return Err(self.latch_fatal(format!(
                        "protocol error: ready.cwd must be an absolute world path: {cwd}"
                    )));
                }
                self.phase = Phase::Ready;
                Ok(())
            }
            ServerFrame::Stdout { data_b64 } => {
                // Spec: `stdout` frames are raw PTY bytes (stdout+stderr combined).
                // This core validates base64 but leaves rendering/routing to the caller.
                let _bytes = BASE64.decode(data_b64.as_bytes()).map_err(|e| {
                    self.latch_fatal(format!(
                        "protocol error: stdout.data_b64 invalid base64: {e}"
                    ))
                })?;
                Ok(())
            }
            ServerFrame::CommandComplete {
                seq,
                token_hex,
                exit: _,
                cwd,
            } => {
                if !cwd.starts_with('/') {
                    return Err(self.latch_fatal(format!(
                        "protocol error: command_complete.cwd must be an absolute world path: {cwd}"
                    )));
                }
                if !is_hex32_lower(&token_hex) {
                    return Err(self
                        .latch_fatal("protocol error: command_complete.token_hex must be hex32"));
                }

                match &self.phase {
                    Phase::InFlight {
                        expected_seq,
                        expected_token,
                    } => {
                        if seq != *expected_seq || token_hex != *expected_token {
                            return Err(self.latch_fatal(format!(
                                "protocol error: command_complete mismatch (expected seq={}, token={} got seq={}, token={})",
                                expected_seq,
                                redact_token(expected_token),
                                seq,
                                redact_token(&token_hex),
                            )));
                        }
                        self.phase = Phase::Ready;
                        Ok(())
                    }
                    _ => Err(self.latch_fatal(
                        "protocol error: command_complete received with no command in flight",
                    )),
                }
            }
            ServerFrame::Exit { code } => {
                if self.shutdown_initiated {
                    self.phase = Phase::Closed;
                    Ok(())
                } else {
                    Err(self.latch_fatal(format!(
                        "world session exited unexpectedly with code={} (protocol fail-closed)",
                        code
                    )))
                }
            }
            ServerFrame::Error {
                code,
                message,
                fatal,
                seq,
            } => {
                if !fatal {
                    return Err(self.latch_fatal(
                        "protocol error: error.fatal=false is invalid for protocol_version=1",
                    ));
                }
                let seq_note = seq.map(|s| format!(" seq={s}")).unwrap_or_default();
                Err(self.latch_fatal(format!("world-agent error ({code}{seq_note}): {message}")))
            }
            ServerFrame::Unknown => Err(self
                .latch_fatal("protocol error: unknown server frame type (protocol fail-closed)")),
        }
    }

    fn latch_fatal(&mut self, message: impl Into<String>) -> PersistentSessionProtocolError {
        self.phase = Phase::Closed;
        let err = PersistentSessionProtocolError::fatal(message);
        self.latched_fatal = Some(err.clone());
        err
    }
}

fn is_hex32_lower(s: &str) -> bool {
    s.len() == 32 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

fn redact_token(token: &str) -> String {
    if token.len() <= 8 {
        return "***".to_string();
    }
    format!("{}…{}", &token[..4], &token[token.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hex32(ch: char) -> String {
        std::iter::repeat_n(ch, 32).collect()
    }

    #[test]
    fn test_ready_version_mismatch_fails_closed() {
        let mut core = PersistentSessionClientCore::new();
        core.note_start_session_sent();

        let err = core
            .on_server_frame(json!({
                "type": "ready",
                "session_nonce": hex32('a'),
                "cwd": "/",
                "protocol_version": 2,
            }))
            .unwrap_err();
        assert!(err.is_fatal());
        assert!(err.to_string().contains("protocol_version"));

        let err2 = core
            .on_server_frame(json!({
                "type": "stdout",
                "data_b64": "AA==",
            }))
            .unwrap_err();
        assert_eq!(err.to_string(), err2.to_string());
    }

    #[test]
    fn test_no_pipelining_rejected_by_client_core() {
        let mut core = PersistentSessionClientCore::new();
        core.note_start_session_sent();
        core.on_server_frame(json!({
            "type": "ready",
            "session_nonce": hex32('b'),
            "cwd": "/",
            "protocol_version": 1,
        }))
        .unwrap();

        let token = hex32('c');
        core.note_exec_in_flight(1, &token).unwrap();

        let err = core.note_exec_in_flight(2, &hex32('d')).unwrap_err();
        assert!(err.is_fatal());
        assert!(err.to_string().contains("in-flight"));
    }

    #[test]
    fn test_command_complete_seq_token_mismatch_fails_closed() {
        let mut core = PersistentSessionClientCore::new();
        core.note_start_session_sent();
        core.on_server_frame(json!({
            "type": "ready",
            "session_nonce": hex32('e'),
            "cwd": "/",
            "protocol_version": 1,
        }))
        .unwrap();

        let token = hex32('f');
        core.note_exec_in_flight(7, &token).unwrap();

        let err = core
            .on_server_frame(json!({
                "type": "command_complete",
                "seq": 8,
                "token_hex": token,
                "exit": 0,
                "cwd": "/tmp",
            }))
            .unwrap_err();
        assert!(err.is_fatal());
        assert!(err.to_string().contains("mismatch"));
    }

    #[test]
    fn test_unknown_server_frame_is_fatal() {
        let mut core = PersistentSessionClientCore::new();
        core.note_start_session_sent();

        let err = core
            .on_server_frame(json!({
                "type": "wat",
                "x": 1,
            }))
            .unwrap_err();
        assert!(err.is_fatal());
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn test_exit_expected_vs_unexpected() {
        let mut core = PersistentSessionClientCore::new();
        core.note_start_session_sent();
        core.on_server_frame(json!({
            "type": "ready",
            "session_nonce": hex32('1'),
            "cwd": "/",
            "protocol_version": 1,
        }))
        .unwrap();

        let err = core
            .on_server_frame(json!({
                "type": "exit",
                "code": 0,
            }))
            .unwrap_err();
        assert!(err.is_fatal());
        assert!(err.to_string().contains("unexpectedly"));

        let mut core = PersistentSessionClientCore::new();
        core.note_start_session_sent();
        core.on_server_frame(json!({
            "type": "ready",
            "session_nonce": hex32('2'),
            "cwd": "/",
            "protocol_version": 1,
        }))
        .unwrap();

        core.note_shutdown_initiated();
        core.on_server_frame(json!({
            "type": "exit",
            "code": 0,
        }))
        .unwrap();
    }

    #[test]
    fn test_stdout_base64_invalid_is_fatal() {
        let mut core = PersistentSessionClientCore::new();
        core.note_start_session_sent();
        core.on_server_frame(json!({
            "type": "ready",
            "session_nonce": hex32('3'),
            "cwd": "/",
            "protocol_version": 1,
        }))
        .unwrap();

        let err = core
            .on_server_frame(json!({
                "type": "stdout",
                "data_b64": "!!!not b64!!!",
            }))
            .unwrap_err();
        assert!(err.is_fatal());
        assert!(err.to_string().contains("base64"));
    }
}
