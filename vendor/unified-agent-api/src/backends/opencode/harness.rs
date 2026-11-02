use std::{future::Future, path::PathBuf, pin::Pin};

use futures_util::StreamExt;

use crate::{
    backend_harness::{
        BackendHarnessAdapter, BackendHarnessErrorPhase, BackendSpawn, DynBackendEventStream,
        NormalizedRequest,
    },
    backends::session_selectors::{
        parse_session_fork_v1, parse_session_resume_v1, validate_resume_fork_mutual_exclusion,
        SessionSelectorV1, EXT_SESSION_FORK_V1, EXT_SESSION_RESUME_V1,
    },
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperKind,
    AgentWrapperRunRequest, EXT_AGENT_API_CONFIG_MODEL_V1,
};

use super::{mapping::map_run_json_event, OpencodeBackend};

const SUPPORTED_EXTENSION_KEYS: [&str; 3] = [
    EXT_AGENT_API_CONFIG_MODEL_V1,
    EXT_SESSION_RESUME_V1,
    EXT_SESSION_FORK_V1,
];
const REDACTED_SPAWN_MESSAGE: &str = "opencode backend error: spawn failed";
const REDACTED_MISSING_BINARY_MESSAGE: &str = "opencode backend error: binary not found";
const REDACTED_STREAM_MESSAGE: &str = "opencode backend error: malformed run output";
const REDACTED_COMPLETION_MESSAGE: &str = "opencode backend error: completion failed";
const REDACTED_TIMEOUT_MESSAGE: &str = "opencode backend error: timeout";

#[derive(Clone, Debug, Default)]
pub struct OpencodeExecPolicy {
    resume: Option<SessionSelectorV1>,
    fork: Option<SessionSelectorV1>,
}

#[derive(Debug)]
pub enum OpencodeBackendError {
    Spawn(opencode::OpencodeError),
    StreamParse,
    Completion(opencode::OpencodeError),
}

impl BackendHarnessAdapter for OpencodeBackend {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind(super::AGENT_KIND.to_string())
    }

    fn supported_extension_keys(&self) -> &'static [&'static str] {
        &SUPPORTED_EXTENSION_KEYS
    }

    type Policy = OpencodeExecPolicy;

    fn validate_and_extract_policy(
        &self,
        request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError> {
        validate_resume_fork_mutual_exclusion(&request.extensions)?;

        let resume = request
            .extensions
            .get(EXT_SESSION_RESUME_V1)
            .map(parse_session_resume_v1)
            .transpose()?;
        let fork = request
            .extensions
            .get(EXT_SESSION_FORK_V1)
            .map(parse_session_fork_v1)
            .transpose()?;

        Ok(OpencodeExecPolicy { resume, fork })
    }

    type BackendEvent = opencode::OpencodeRunJsonEvent;
    type BackendCompletion = opencode::OpencodeRunCompletion;
    type BackendError = OpencodeBackendError;

    fn spawn(
        &self,
        req: NormalizedRequest<Self::Policy>,
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
        let binary = self
            .config
            .binary
            .clone()
            .or_else(|| std::env::var_os("OPENCODE_BINARY").map(PathBuf::from));
        let NormalizedRequest {
            prompt,
            model_id,
            working_dir,
            effective_timeout: timeout,
            env,
            policy,
            ..
        } = req;
        let OpencodeExecPolicy { resume, fork } = policy;

        Box::pin(async move {
            let mut builder = opencode::OpencodeClient::builder();
            if let Some(binary) = binary {
                builder = builder.binary(binary);
            }
            if let Some(timeout) = timeout {
                builder = builder.timeout(timeout);
            }
            for (key, value) in env {
                builder = builder.env(key, value);
            }
            let client = builder.build();

            let mut run_request = opencode::OpencodeRunRequest::new(prompt);
            if let Some(model_id) = model_id {
                run_request = run_request.model(model_id);
            }
            match resume {
                Some(SessionSelectorV1::Last) => {
                    run_request = run_request.continue_session(true);
                }
                Some(SessionSelectorV1::Id { id }) => {
                    run_request = run_request.session(id);
                }
                None => {}
            }
            match fork {
                Some(SessionSelectorV1::Last) => {
                    run_request = run_request.continue_session(true).fork(true);
                }
                Some(SessionSelectorV1::Id { id }) => {
                    run_request = run_request.session(id).fork(true);
                }
                None => {}
            }
            if let Some(working_dir) = working_dir {
                run_request = run_request.working_dir(working_dir);
            }

            let handle = client
                .run_json(run_request)
                .await
                .map_err(OpencodeBackendError::Spawn)?;
            let opencode::OpencodeRunJsonHandle { events, completion } = handle;

            let events: DynBackendEventStream<Self::BackendEvent, Self::BackendError> =
                Box::pin(events.map(|item| item.map_err(|_| OpencodeBackendError::StreamParse)));

            let completion =
                Box::pin(async move { completion.await.map_err(OpencodeBackendError::Completion) });

            Ok(BackendSpawn {
                events,
                completion,
                events_observability: None,
            })
        })
    }

    fn map_event(&self, event: Self::BackendEvent) -> Vec<AgentWrapperEvent> {
        map_run_json_event(event)
    }

    fn map_completion(
        &self,
        completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        Ok(crate::bounds::enforce_completion_bounds(
            AgentWrapperCompletion {
                status: completion.status,
                final_text: crate::bounds::enforce_final_text_bound(completion.final_text),
                data: None,
            },
        ))
    }

    fn redact_error(&self, phase: BackendHarnessErrorPhase, err: &Self::BackendError) -> String {
        match (phase, err) {
            (
                BackendHarnessErrorPhase::Spawn,
                OpencodeBackendError::Spawn(opencode::OpencodeError::MissingBinary),
            ) => REDACTED_MISSING_BINARY_MESSAGE.to_string(),
            (
                BackendHarnessErrorPhase::Completion,
                OpencodeBackendError::Completion(opencode::OpencodeError::SelectionFailed {
                    message,
                }),
            ) => message.clone(),
            (
                BackendHarnessErrorPhase::Completion,
                OpencodeBackendError::Completion(opencode::OpencodeError::RunFailed {
                    message,
                    ..
                }),
            ) => message.clone(),
            (BackendHarnessErrorPhase::Spawn, OpencodeBackendError::Spawn(_)) => {
                REDACTED_SPAWN_MESSAGE.to_string()
            }
            (BackendHarnessErrorPhase::Stream, OpencodeBackendError::StreamParse) => {
                REDACTED_STREAM_MESSAGE.to_string()
            }
            (
                BackendHarnessErrorPhase::Completion,
                OpencodeBackendError::Completion(opencode::OpencodeError::Timeout { .. }),
            ) => REDACTED_TIMEOUT_MESSAGE.to_string(),
            (BackendHarnessErrorPhase::Completion, OpencodeBackendError::Completion(_)) => {
                REDACTED_COMPLETION_MESSAGE.to_string()
            }
            (_, OpencodeBackendError::Spawn(opencode::OpencodeError::MissingBinary)) => {
                REDACTED_MISSING_BINARY_MESSAGE.to_string()
            }
            (
                _,
                OpencodeBackendError::Completion(opencode::OpencodeError::SelectionFailed {
                    message,
                }),
            ) => message.clone(),
            (
                _,
                OpencodeBackendError::Completion(opencode::OpencodeError::RunFailed {
                    message,
                    ..
                }),
            ) => message.clone(),
            (_, OpencodeBackendError::Spawn(_)) => REDACTED_SPAWN_MESSAGE.to_string(),
            (_, OpencodeBackendError::StreamParse) => REDACTED_STREAM_MESSAGE.to_string(),
            (_, OpencodeBackendError::Completion(opencode::OpencodeError::Timeout { .. })) => {
                REDACTED_TIMEOUT_MESSAGE.to_string()
            }
            (_, OpencodeBackendError::Completion(_)) => REDACTED_COMPLETION_MESSAGE.to_string(),
        }
    }
}
