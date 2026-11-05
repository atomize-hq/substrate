//! A finite retained owner joins raw completion and separate checked SDK custody.
use agent_api::{
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperProcessSettlementObservation,
    AgentWrapperProcessTerminationObservation, DynAgentWrapperCompletion,
    DynAgentWrapperProcessSettlement, DynAgentWrapperSettlement,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::{oneshot, watch};

pub(crate) fn backend_error(message: impl Into<String>) -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: message.into(),
    }
}

// The SDK error is deliberately not Clone. Preserve its exact variant and fields
// for independent observation while returning the original raw value untouched.
fn clone_wrapper_error(error: &AgentWrapperError) -> AgentWrapperError {
    use AgentWrapperError::*;
    match error {
        UnknownBackend { agent_kind } => UnknownBackend {
            agent_kind: agent_kind.clone(),
        },
        UnknownRuntimeFamily { runtime_family } => UnknownRuntimeFamily {
            runtime_family: runtime_family.clone(),
        },
        UnsupportedTargetTriple {
            runtime_family,
            target_triple,
        } => UnsupportedTargetTriple {
            runtime_family: runtime_family.clone(),
            target_triple: target_triple.clone(),
        },
        MissingValidatedRuntime {
            runtime_family,
            target_triple,
        } => MissingValidatedRuntime {
            runtime_family: runtime_family.clone(),
            target_triple: target_triple.clone(),
        },
        UnsupportedCapability {
            agent_kind,
            capability,
        } => UnsupportedCapability {
            agent_kind: agent_kind.clone(),
            capability: capability.clone(),
        },
        InvalidAgentKind { message } => InvalidAgentKind {
            message: message.clone(),
        },
        InvalidRequest { message } => InvalidRequest {
            message: message.clone(),
        },
        Backend { message } => Backend {
            message: message.clone(),
        },
    }
}

#[derive(Debug)]
pub(crate) enum NativeSettlementOutcome {
    Checked(Result<(), AgentWrapperError>),
    Unsupported,
}

#[derive(Debug)]
pub(crate) struct NativeTerminalOutcome {
    pub(crate) raw: Result<AgentWrapperCompletion, AgentWrapperError>,
    pub(crate) settlement: NativeSettlementOutcome,
    process_settlement: Option<AgentWrapperProcessSettlementObservation>,
}

/// Actual SDK child-admission/wait facts only. Neither variant retires the
/// enclosing member, gateway, history owner, World or exclusion lease.
pub(crate) enum CheckedNativeProcessCloseout<'a> {
    AdmissionClosedBeforeSpawn,
    Processes(&'a [AgentWrapperProcessTerminationObservation]),
}

impl NativeTerminalOutcome {
    pub(crate) fn require_checked(&self) -> Result<(), AgentWrapperError> {
        match &self.settlement {
            NativeSettlementOutcome::Checked(result) => {
                result.as_ref().map(|_| ()).map_err(clone_wrapper_error)
            }
            NativeSettlementOutcome::Unsupported => {
                Err(backend_error("checked native settlement unavailable"))
            }
        }
    }

    /// Copied wait observations, not permission or proof that no process existed.
    pub(crate) fn require_checked_process_observations(
        &self,
    ) -> Result<&[AgentWrapperProcessTerminationObservation], AgentWrapperError> {
        self.require_checked()?;
        self.process_settlement
            .as_ref()
            .filter(|value| value.acknowledged)
            .and_then(|value| value.observations.as_deref())
            .ok_or_else(|| backend_error("checked native process observations unavailable"))
    }

    pub(crate) fn require_checked_process_closeout(
        &self,
    ) -> Result<CheckedNativeProcessCloseout<'_>, AgentWrapperError> {
        let observations = self.require_checked_process_observations()?;
        let closed_before_any_spawn = self
            .process_settlement
            .as_ref()
            .and_then(|value| value.closed_before_any_spawn);
        if observations.is_empty() {
            return if closed_before_any_spawn == Some(true) {
                Ok(CheckedNativeProcessCloseout::AdmissionClosedBeforeSpawn)
            } else {
                Err(backend_error(
                    "native admission closeout unavailable; empty waits are not absence proof",
                ))
            };
        }
        if closed_before_any_spawn == Some(true)
            || observations
                .iter()
                .any(|value| value.process_id.is_none_or(|pid| pid == 0) || value.status.is_none())
        {
            return Err(backend_error(
                "native process closeout is contradictory or unknown",
            ));
        }
        Ok(CheckedNativeProcessCloseout::Processes(observations))
    }

    /// Convert copied, checked SDK observations without inspecting or rewriting
    /// raw completion. This data conversion supplies no enclosing retirement.
    #[cfg(target_os = "linux")]
    pub(crate) fn initial_bootstrap_process_closeout(
        &self,
    ) -> Result<transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1, AgentWrapperError>
    {
        use std::os::unix::process::ExitStatusExt;
        use transport_api_types::{
            NativeProcessTerminationV1 as Status,
            OrdinaryInitialBootstrapProcessCloseoutV1 as Closeout,
        };
        let termination = |status: std::process::ExitStatus| -> Result<Status, AgentWrapperError> {
            match (status.code(), status.signal()) {
                (Some(exit_code), _) => Ok(Status::Exited { exit_code }),
                (_, Some(signal)) => Ok(Status::Signaled { signal }),
                _ => Err(backend_error(
                    "initial native wait omitted actual termination",
                )),
            }
        };
        let result = match self.require_checked_process_closeout()? {
            CheckedNativeProcessCloseout::AdmissionClosedBeforeSpawn => {
                Closeout::ClosedBeforeAnySpawn {}
            }
            CheckedNativeProcessCloseout::Processes(observed) => Closeout::ProcessesWaited {
                processes: observed
                    .iter()
                    .map(|process| {
                        Ok(transport_api_types::NativeProcessWaitObservationV1 {
                            pid: process.process_id.filter(|pid| *pid != 0).ok_or_else(|| {
                                backend_error("initial native wait omitted actual PID")
                            })?,
                            termination: termination(process.status.ok_or_else(|| {
                                backend_error("initial native primary wait unavailable")
                            })?)?,
                            recovery_termination: process
                                .recovery_status
                                .map(termination)
                                .transpose()?,
                        })
                    })
                    .collect::<Result<Vec<_>, AgentWrapperError>>()?,
            },
        };
        result.validate().map_err(backend_error)?;
        Ok(result)
    }

    /// This is the enclosing Substrate result. The raw universal value stays intact.
    pub(crate) fn enclosing_result(
        &self,
        completion: Result<AgentWrapperCompletion, AgentWrapperError>,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        match &self.settlement {
            NativeSettlementOutcome::Checked(Ok(())) => completion,
            NativeSettlementOutcome::Checked(Err(error)) => Err(backend_error(format!(
                "submission_outcome_unknown: native acknowledgment failed: {error}; raw backend outcome: {}; enclosing cleanup outcome: {}",
                outcome_label(&self.raw), outcome_label(&completion),
            ))),
            NativeSettlementOutcome::Unsupported => {
                tracing::warn!("native settlement unsupported; forwarding backend completion without a checked cleanup claim");
                completion
            }
        }
    }
}

fn outcome_label(result: &Result<AgentWrapperCompletion, AgentWrapperError>) -> String {
    match result {
        Ok(value) => format!("status {}", value.status),
        Err(error) => error.to_string(),
    }
}

#[derive(Clone)]
pub(crate) struct NativeTerminalObservation {
    receiver: watch::Receiver<Option<Arc<NativeTerminalOutcome>>>,
    // Disposal is a request, never a native terminal acknowledgment.
    route_disposal: tokio_util::sync::CancellationToken,
}
impl NativeTerminalObservation {
    pub(crate) fn request_route_disposal(&self) {
        self.route_disposal.cancel();
    }

    pub(crate) async fn route_disposal_requested(&self) {
        self.route_disposal.cancelled().await;
    }

    pub(crate) async fn wait(&self) -> Arc<NativeTerminalOutcome> {
        let mut receiver = self.receiver.clone();
        loop {
            let ready = receiver.borrow().clone();
            if let Some(ready) = ready {
                return ready;
            }
            if receiver.changed().await.is_err() {
                let error =
                    backend_error("submission_outcome_unknown: native terminal owner abandoned");
                return Arc::new(NativeTerminalOutcome {
                    raw: Err(clone_wrapper_error(&error)),
                    settlement: NativeSettlementOutcome::Checked(Err(error)),
                    process_settlement: None,
                });
            }
        }
    }
}

pub(crate) fn retain_native_completion<C: Send + 'static>(
    completion: DynAgentWrapperCompletion,
    settlement: Option<DynAgentWrapperSettlement>,
    resources: C,
    terminate: impl FnOnce() + Send + 'static,
    timeout: Option<Duration>,
    disposal_requested: Option<agent_api::DynAgentWrapperDisposalRequested>,
) -> (DynAgentWrapperCompletion, NativeTerminalObservation) {
    retain_native_completion_with_process_observation(
        completion,
        settlement,
        None,
        resources,
        terminate,
        timeout,
        disposal_requested,
    )
}

pub(crate) fn retain_native_completion_with_process_observation<C: Send + 'static>(
    mut completion: DynAgentWrapperCompletion,
    settlement: Option<DynAgentWrapperSettlement>,
    process_settlement: Option<DynAgentWrapperProcessSettlement>,
    resources: C,
    terminate: impl FnOnce() + Send + 'static,
    timeout: Option<Duration>,
    disposal_requested: Option<agent_api::DynAgentWrapperDisposalRequested>,
) -> (DynAgentWrapperCompletion, NativeTerminalObservation) {
    let (mut sender, receiver) = oneshot::channel();
    let (terminal_sender, terminal_receiver) = watch::channel(None);
    let route_disposal = tokio_util::sync::CancellationToken::new();
    let dispose = route_disposal.clone();
    tokio::spawn(async move {
        let deadline = async {
            match timeout.filter(|value| !value.is_zero()) {
                Some(value) => tokio::time::sleep(value).await,
                None => std::future::pending().await,
            }
        };
        tokio::pin!(deadline);
        let disposal_requested = async move {
            match disposal_requested {
                Some(requested) => requested.await,
                None => std::future::pending().await,
            }
        };
        tokio::pin!(disposal_requested);
        let mut requested = false;
        let raw = loop {
            tokio::select! {
                value = &mut completion => break value,
                _ = &mut disposal_requested, if !requested => {
                    requested = true;
                    dispose.cancel();
                },
                _ = sender.closed() => { dispose.cancel(); terminate(); break completion.await },
                _ = &mut deadline => { dispose.cancel(); terminate(); break completion.await },
            }
        };
        dispose.cancel();
        // Concrete settle seals admission: it is polled only after startup and raw
        // completion have ended. UAA observer acquisition itself is observation-only.
        let settlement = match settlement {
            Some(observation) => NativeSettlementOutcome::Checked(observation.await),
            None => NativeSettlementOutcome::Unsupported,
        };
        let process_settlement = match process_settlement {
            Some(observation) => Some(observation.await),
            None => None,
        };
        drop(resources);
        let _ = terminal_sender.send(Some(Arc::new(NativeTerminalOutcome {
            raw: raw.as_ref().cloned().map_err(clone_wrapper_error),
            settlement,
            process_settlement,
        })));
        let _ = sender.send(raw);
    });
    let raw = Box::pin(async move {
        receiver.await.map_err(|_| {
            backend_error("submission_outcome_unknown: native completion owner abandoned")
        })?
    });
    (
        raw,
        NativeTerminalObservation {
            receiver: terminal_receiver,
            route_disposal,
        },
    )
}

#[cfg(test)]
mod tests;
