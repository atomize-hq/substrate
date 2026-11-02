use std::{future::Future, path::PathBuf, pin::Pin};

use futures_util::StreamExt;

use crate::{
    backend_harness::{
        BackendHarnessAdapter, BackendHarnessErrorPhase, BackendSpawn, DynBackendEventStream,
        NormalizedRequest,
    },
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperKind,
    AgentWrapperRunRequest, EXT_AGENT_API_CONFIG_MODEL_V1,
};

use super::{mapping::map_stream_json_event, GeminiCliBackend};

const REDACTED_SPAWN_MESSAGE: &str = "gemini backend error: spawn failed";
const REDACTED_MISSING_BINARY_MESSAGE: &str = "gemini backend error: binary not found";
const REDACTED_STREAM_MESSAGE: &str = "gemini backend error: malformed stream-json output";
const REDACTED_COMPLETION_MESSAGE: &str = "gemini backend error: completion failed";
const REDACTED_INVALID_INPUT_MESSAGE: &str = "gemini backend error: invalid input";
const REDACTED_TURN_LIMIT_MESSAGE: &str = "gemini backend error: turn limit exceeded";
const REDACTED_TIMEOUT_MESSAGE: &str = "gemini backend error: timeout";

#[derive(Clone, Debug, Default)]
pub struct GeminiCliPolicy;

#[derive(Debug)]
pub enum GeminiCliBackendError {
    Spawn(gemini_cli::GeminiCliError),
    StreamParse,
    Completion(gemini_cli::GeminiCliError),
}

impl BackendHarnessAdapter for GeminiCliBackend {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind(super::AGENT_KIND.to_string())
    }

    fn supported_extension_keys(&self) -> &'static [&'static str] {
        static SUPPORTED_EXTENSION_KEY: &str = EXT_AGENT_API_CONFIG_MODEL_V1;
        std::slice::from_ref(&SUPPORTED_EXTENSION_KEY)
    }

    type Policy = GeminiCliPolicy;

    fn validate_and_extract_policy(
        &self,
        _request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError> {
        Ok(GeminiCliPolicy)
    }

    type BackendEvent = gemini_cli::GeminiStreamJsonEvent;
    type BackendCompletion = gemini_cli::GeminiStreamJsonCompletion;
    type BackendError = GeminiCliBackendError;

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
            .or_else(|| std::env::var_os("GEMINI_BINARY").map(PathBuf::from));
        let NormalizedRequest {
            prompt,
            model_id,
            working_dir,
            effective_timeout: timeout,
            env,
            ..
        } = req;

        Box::pin(async move {
            let mut builder = gemini_cli::GeminiCliClient::builder();
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

            let mut run_request = gemini_cli::GeminiStreamJsonRunRequest::new(prompt);
            if let Some(model_id) = model_id {
                run_request = run_request.model(model_id);
            }
            if let Some(working_dir) = working_dir {
                run_request = run_request.working_dir(working_dir);
            }

            let handle = client
                .stream_json(run_request)
                .await
                .map_err(GeminiCliBackendError::Spawn)?;
            let gemini_cli::GeminiStreamJsonHandle { events, completion } = handle;

            let events: DynBackendEventStream<Self::BackendEvent, Self::BackendError> =
                Box::pin(events.map(|item| item.map_err(|_| GeminiCliBackendError::StreamParse)));
            let completion =
                Box::pin(
                    async move { completion.await.map_err(GeminiCliBackendError::Completion) },
                );

            Ok(BackendSpawn {
                events,
                completion,
                events_observability: None,
            })
        })
    }

    fn map_event(&self, event: Self::BackendEvent) -> Vec<AgentWrapperEvent> {
        map_stream_json_event(event)
    }

    fn map_completion(
        &self,
        completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        let data = if completion.raw_result.is_some()
            || completion.session_id.is_some()
            || completion.model.is_some()
        {
            Some(serde_json::json!({
                "raw_result": completion.raw_result,
                "session": { "id": completion.session_id },
                "model": completion.model,
            }))
        } else {
            None
        };

        Ok(crate::bounds::enforce_completion_bounds(
            AgentWrapperCompletion {
                status: completion.status,
                final_text: crate::bounds::enforce_final_text_bound(completion.final_text),
                data,
            },
        ))
    }

    fn redact_error(&self, phase: BackendHarnessErrorPhase, err: &Self::BackendError) -> String {
        match (phase, err) {
            (
                BackendHarnessErrorPhase::Spawn,
                GeminiCliBackendError::Spawn(gemini_cli::GeminiCliError::MissingBinary),
            ) => REDACTED_MISSING_BINARY_MESSAGE.to_string(),
            (
                BackendHarnessErrorPhase::Completion,
                GeminiCliBackendError::Completion(gemini_cli::GeminiCliError::RunFailed {
                    exit_code: Some(42),
                    ..
                }),
            ) => REDACTED_INVALID_INPUT_MESSAGE.to_string(),
            (
                BackendHarnessErrorPhase::Completion,
                GeminiCliBackendError::Completion(gemini_cli::GeminiCliError::RunFailed {
                    exit_code: Some(53),
                    ..
                }),
            ) => REDACTED_TURN_LIMIT_MESSAGE.to_string(),
            (
                BackendHarnessErrorPhase::Completion,
                GeminiCliBackendError::Completion(gemini_cli::GeminiCliError::Timeout { .. }),
            ) => REDACTED_TIMEOUT_MESSAGE.to_string(),
            (BackendHarnessErrorPhase::Spawn, GeminiCliBackendError::Spawn(_)) => {
                REDACTED_SPAWN_MESSAGE.to_string()
            }
            (BackendHarnessErrorPhase::Stream, GeminiCliBackendError::StreamParse) => {
                REDACTED_STREAM_MESSAGE.to_string()
            }
            (BackendHarnessErrorPhase::Completion, GeminiCliBackendError::Completion(_)) => {
                REDACTED_COMPLETION_MESSAGE.to_string()
            }
            (_, GeminiCliBackendError::Spawn(gemini_cli::GeminiCliError::MissingBinary)) => {
                REDACTED_MISSING_BINARY_MESSAGE.to_string()
            }
            (
                _,
                GeminiCliBackendError::Completion(gemini_cli::GeminiCliError::RunFailed {
                    exit_code: Some(42),
                    ..
                }),
            ) => REDACTED_INVALID_INPUT_MESSAGE.to_string(),
            (
                _,
                GeminiCliBackendError::Completion(gemini_cli::GeminiCliError::RunFailed {
                    exit_code: Some(53),
                    ..
                }),
            ) => REDACTED_TURN_LIMIT_MESSAGE.to_string(),
            (_, GeminiCliBackendError::Completion(gemini_cli::GeminiCliError::Timeout { .. })) => {
                REDACTED_TIMEOUT_MESSAGE.to_string()
            }
            (_, GeminiCliBackendError::Spawn(_)) => REDACTED_SPAWN_MESSAGE.to_string(),
            (_, GeminiCliBackendError::StreamParse) => REDACTED_STREAM_MESSAGE.to_string(),
            (_, GeminiCliBackendError::Completion(_)) => REDACTED_COMPLETION_MESSAGE.to_string(),
        }
    }
}
