use std::future::Future;
use std::pin::Pin;

use super::contract::{
    BackendHarnessAdapter, BackendHarnessErrorPhase, BackendSpawn, NormalizedRequest,
};
use crate::{
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperKind, AgentWrapperRunRequest,
};

pub(super) fn success_exit_status() -> std::process::ExitStatus {
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

pub(super) fn toy_kind() -> AgentWrapperKind {
    AgentWrapperKind::new("toy").expect("toy kind is valid")
}

pub(super) struct ToyAdapter {
    pub(super) fail_spawn: bool,
}

pub(super) struct ToyPolicy;

pub(super) enum ToyEvent {
    Text(String),
}

pub(super) struct ToyCompletion;

#[derive(Debug)]
pub(super) struct ToyBackendError {
    pub(super) secret: String,
}

impl BackendHarnessAdapter for ToyAdapter {
    fn kind(&self) -> AgentWrapperKind {
        toy_kind()
    }

    fn supported_extension_keys(&self) -> &'static [&'static str] {
        &["agent_api.exec.non_interactive", "backend.toy.example"]
    }

    type Policy = ToyPolicy;

    fn validate_and_extract_policy(
        &self,
        _request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError> {
        Ok(ToyPolicy)
    }

    type BackendEvent = ToyEvent;
    type BackendCompletion = ToyCompletion;
    type BackendError = ToyBackendError;

    fn spawn(
        &self,
        _req: NormalizedRequest<Self::Policy>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BackendSpawn<
                            Self::BackendEvent,
                            Self::BackendCompletion,
                            Self::BackendError,
                        >,
                        Self::BackendError,
                    >,
                > + Send
                + 'static,
        >,
    > {
        let fail_spawn = self.fail_spawn;
        Box::pin(async move {
            if fail_spawn {
                return Err(ToyBackendError {
                    secret: "SECRET_SPAWN".to_string(),
                });
            }

            let events = futures_util::stream::iter([
                Ok(ToyEvent::Text("one".to_string())),
                Ok(ToyEvent::Text("two".to_string())),
            ]);

            Ok(BackendSpawn {
                events: Box::pin(events),
                completion: Box::pin(async move { Ok(ToyCompletion) }),
                events_observability: None,
            })
        })
    }
    fn map_event(&self, event: Self::BackendEvent) -> Vec<AgentWrapperEvent> {
        match event {
            ToyEvent::Text(text) => vec![AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(text),
                message: None,
                data: None,
            }],
        }
    }

    fn map_completion(
        &self,
        _completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        Ok(AgentWrapperCompletion {
            status: success_exit_status(),
            final_text: Some("done".to_string()),
            data: None,
        })
    }

    fn redact_error(&self, phase: BackendHarnessErrorPhase, _err: &Self::BackendError) -> String {
        let phase = match phase {
            BackendHarnessErrorPhase::Spawn => "spawn",
            BackendHarnessErrorPhase::Stream => "stream",
            BackendHarnessErrorPhase::Completion => "completion",
        };
        format!("toy backend error (redacted): phase={phase}")
    }
}
