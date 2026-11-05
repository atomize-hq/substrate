//! A finite retained owner joins raw completion and separate checked SDK custody.
use agent_api::{
    AgentWrapperCompletion, AgentWrapperError, DynAgentWrapperCompletion, DynAgentWrapperSettlement,
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
}

/// One provider terminal fact; live payload delivery is independent of replaying
/// its native completion/authority consequences.
#[derive(Default)]
pub(crate) struct PendingProviderTerminal {
    event: Option<(agent_api::AgentWrapperEvent, bool)>,
    conflicted: bool,
}
impl PendingProviderTerminal {
    pub(crate) fn push(
        &mut self,
        event: agent_api::AgentWrapperEvent,
        completed: bool,
    ) -> Result<(), AgentWrapperError> {
        if self.event.is_some() {
            self.conflicted = true;
            return Err(backend_error(
                "wrapper duplicated native terminal evidence before process finality",
            ));
        }
        self.event = Some((event, completed));
        Ok(())
    }
    pub(crate) fn take_after_finality(
        &mut self,
        stream_final: bool,
        native: &NativeTerminalOutcome,
    ) -> Result<Option<agent_api::AgentWrapperEvent>, AgentWrapperError> {
        if !stream_final {
            return Err(backend_error("provider terminal precedes stream finality"));
        }
        native.require_checked()?;
        if self.conflicted {
            return Err(backend_error(
                "wrapper duplicated native terminal evidence before process finality",
            ));
        }
        match (&self.event, &native.raw) {
            (Some((_, true)), Ok(completion)) if completion.status.success() => {}
            (Some((_, true)), _) => return Err(backend_error(
                "submission_outcome_unknown: provider completed without successful native exit",
            )),
            (_, Err(AgentWrapperError::Backend { message })) if message == "cancelled" => {
                return Err(backend_error("submission_outcome_unknown: raw native cancellation is not a provider terminal fact"));
            }
            // An actual provider Failed fact remains a failed turn once native
            // child/stdio settlement is checked, even if raw completion is Err.
            (Some((_, false)), _) => {}
            (None, Err(error)) => return Err(backend_error(format!(
                "submission_outcome_unknown: native completion failed without provider terminal fact: {error}"
            ))),
            (None, Ok(_)) => {}
        }
        Ok(self.event.take().map(|(event, _)| event))
    }
}
impl NativeTerminalObservation {
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
                });
            }
        }
    }
}

pub(crate) fn retain_native_completion<C: Send + 'static>(
    mut completion: DynAgentWrapperCompletion,
    settlement: Option<DynAgentWrapperSettlement>,
    resources: C,
    terminate: impl FnOnce() + Send + 'static,
    timeout: Option<Duration>,
) -> (DynAgentWrapperCompletion, NativeTerminalObservation) {
    let (mut sender, receiver) = oneshot::channel();
    let (terminal_sender, terminal_receiver) = watch::channel(None);
    tokio::spawn(async move {
        let deadline = async {
            match timeout.filter(|value| !value.is_zero()) {
                Some(value) => tokio::time::sleep(value).await,
                None => std::future::pending().await,
            }
        };
        let raw = tokio::select! {
            value = &mut completion => value,
            _ = sender.closed() => { terminate(); completion.await },
            _ = deadline => { terminate(); completion.await },
        };
        // Concrete settle seals admission: it is polled only after startup and raw
        // completion have ended. UAA observer acquisition itself is observation-only.
        let settlement = match settlement {
            Some(observation) => NativeSettlementOutcome::Checked(observation.await),
            None => NativeSettlementOutcome::Unsupported,
        };
        drop(resources);
        let _ = terminal_sender.send(Some(Arc::new(NativeTerminalOutcome {
            raw: raw.as_ref().cloned().map_err(clone_wrapper_error),
            settlement,
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
        },
    )
}

#[cfg(test)]
mod tests;

// A finite latch distinguishes dropping the bridge awaiter from actual disposal
// of the boxed startup future retained by its independent owner.
#[derive(Default)]
struct StartupState {
    started: bool,
    active: bool,
    cancelled: bool,
    abandoned: bool,
    #[cfg(target_os = "linux")]
    host_admission_closed: bool,
}
#[derive(Default)]
pub(crate) struct StartupLatch {
    #[cfg(target_os = "linux")]
    host_acquisition: std::sync::Mutex<Option<Arc<crate::execution::agent_runtime::native_launch_context::HostGatewayAcquisitionCustodyV1>>>,
    #[cfg(target_os = "linux")]
    host_route: std::sync::Mutex<
        Option<Arc<crate::execution::agent_runtime::native_launch_context::HostGatewayTurnLeaseV1>>,
    >,
    state: std::sync::Mutex<StartupState>,
    changed: tokio::sync::Notify,
    stop: tokio::sync::Notify,
    #[cfg(test)]
    checked_process_settlement: std::sync::atomic::AtomicBool,
}
#[derive(Clone)]
pub(crate) enum NativeAttemptCustody {
    Codex {
        process: codex::ExecProcessCustody,
        startup: Arc<StartupLatch>,
    },
    Claude {
        process: claude_code::ClaudeProcessCustody,
        startup: Arc<StartupLatch>,
    },
}
impl NativeAttemptCustody {
    pub(crate) fn codex() -> Self {
        Self::Codex {
            process: codex::ExecProcessCustody::new(),
            startup: Arc::default(),
        }
    }
    pub(crate) fn claude() -> Self {
        Self::Claude {
            process: claude_code::ClaudeProcessCustody::new(),
            startup: Arc::default(),
        }
    }
    fn latch(&self) -> &StartupLatch {
        match self {
            Self::Codex { startup, .. } | Self::Claude { startup, .. } => startup,
        }
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn begin_host_acquisition(
        &self,
    ) -> Result<
        Arc<
            crate::execution::agent_runtime::native_launch_context::HostGatewayAcquisitionCustodyV1,
        >,
        AgentWrapperError,
    > {
        let state = self
            .latch()
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if state.cancelled || state.host_admission_closed {
            return Err(backend_error(
                "native attempt closed before Host acquisition",
            ));
        }
        let mut slot = self
            .latch()
            .host_acquisition
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if slot.is_some() {
            return Err(backend_error(
                "Host acquisition already attempted; retained uncertainty cannot be rebound",
            ));
        }
        let acquisition=Arc::new(crate::execution::agent_runtime::native_launch_context::HostGatewayAcquisitionCustodyV1::new());
        *slot = Some(Arc::clone(&acquisition));
        Ok(acquisition)
    }

    #[cfg(target_os = "linux")]
    fn host_acquisition(
        &self,
    ) -> Option<
        Arc<
            crate::execution::agent_runtime::native_launch_context::HostGatewayAcquisitionCustodyV1,
        >,
    > {
        self.latch()
            .host_acquisition
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
    #[cfg(target_os = "linux")]
    fn close_host_acquisition(
        &self,
    ) -> Option<
        Arc<
            crate::execution::agent_runtime::native_launch_context::HostGatewayAcquisitionCustodyV1,
        >,
    > {
        // Same lock order as registration. Seal future admission before taking a
        // snapshot, including when no preparation has started yet.
        let mut state = self
            .latch()
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.host_admission_closed = true;
        let acquisition = self.host_acquisition();
        drop(state);
        if let Some(acquisition) = acquisition.as_ref() {
            acquisition.request_close();
        }
        acquisition
    }

    #[cfg(target_os = "linux")]
    fn host_route(
        &self,
    ) -> Option<Arc<crate::execution::agent_runtime::native_launch_context::HostGatewayTurnLeaseV1>>
    {
        self.latch()
            .host_route
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    #[cfg(target_os = "linux")]
    pub(crate) async fn bind_host_route(
        &self,
        route: Arc<crate::execution::agent_runtime::native_launch_context::HostGatewayTurnLeaseV1>,
    ) -> Result<(), AgentWrapperError> {
        if !self
            .host_acquisition()
            .is_some_and(|pending| Arc::ptr_eq(&pending, route.acquisition()))
        {
            let _ = route.close_and_wait().await;
            return Err(backend_error(
                "Host route does not belong to pending acquisition",
            ));
        }
        let callback_route = Arc::clone(&route);
        let callback = move || {
            let route = Arc::clone(&callback_route);
            async move { route.revalidate_before_spawn().await }
        };
        let installed = match self {
            Self::Codex { process, .. } => process.install_spawn_admission(callback),
            Self::Claude { process, .. } => process.install_spawn_admission(callback),
        };
        if installed.is_err() {
            let cleanup = route.close_and_wait().await;
            return Err(backend_error(format!(
                "Host spawn admission installation refused; cleanup checked={}",
                cleanup.is_ok()
            )));
        }
        *self
            .latch()
            .host_route
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(route);
        if self
            .latch()
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cancelled
        {
            if let Some(route) = self.host_route() {
                route.request_close();
            }
        }
        Ok(())
    }

    fn terminate_process(&self) {
        #[cfg(target_os = "linux")]
        if let Some(acquisition) = self.close_host_acquisition() {
            acquisition.request_close();
        }
        match self {
            Self::Codex { process, .. } => process.request_termination(),
            Self::Claude { process, .. } => process.request_termination(),
        }
    }
    pub(crate) fn request_termination(&self) {
        #[cfg(target_os = "linux")]
        if let Some(acquisition) = self.close_host_acquisition() {
            acquisition.request_close();
        }
        let mut state = self
            .latch()
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.cancelled = true;
        let active = state.active;
        drop(state);
        self.latch().stop.notify_waiters();
        if !active {
            self.terminate_process();
        }
    }
    pub(crate) async fn cancelled(&self) {
        loop {
            let stopped = self.latch().stop.notified();
            tokio::pin!(stopped);
            stopped.as_mut().enable();
            if self
                .latch()
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .cancelled
            {
                return;
            }
            stopped.await;
        }
    }
    fn begin_startup(&self) -> Result<StartupOwner, AgentWrapperError> {
        let mut state = self
            .latch()
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if state.started {
            return Err(backend_error("native attempt startup already used"));
        }
        state.started = true;
        state.active = true;
        Ok(StartupOwner {
            custody: self.clone(),
            acknowledged: false,
        })
    }
    async fn process_settlement(&self) -> Result<(), AgentWrapperError> {
        #[cfg(target_os = "linux")]
        let acquisition = self.close_host_acquisition();
        let native = async {
            match self {
                Self::Codex { process, .. } => process
                    .settle()
                    .await
                    .map_err(|error| backend_error(error.to_string())),
                Self::Claude { process, .. } => process
                    .settle()
                    .await
                    .map_err(|error| backend_error(error.to_string())),
            }
        };
        #[cfg(target_os = "linux")]
        let route_cleanup = async {
            match acquisition {
                Some(acquisition) => acquisition
                    .close_and_wait()
                    .await
                    .map(|_| ())
                    .map_err(|error| backend_error(error.to_string())),
                None => Ok(()),
            }
        };
        #[cfg(target_os = "linux")]
        let outcome = {
            // Gateway disposal progresses even while native child/stdio waits.
            let (native, route) = tokio::join!(native, route_cleanup);
            match (native, route) {
                (Ok(()), result) | (result, Ok(())) => result,
                (Err(native), Err(route)) => Err(backend_error(format!(
                    "native acknowledgment: {native}; gateway cleanup: {route}"
                ))),
            }
        };
        #[cfg(not(target_os = "linux"))]
        let outcome = native.await;
        #[cfg(test)]
        self.latch()
            .checked_process_settlement
            .store(outcome.is_ok(), std::sync::atomic::Ordering::SeqCst);
        outcome
    }
    pub(crate) fn settlement(&self) -> DynAgentWrapperSettlement {
        let custody = self.clone();
        Box::pin(async move {
            let abandoned = loop {
                let changed = custody.latch().changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                let ready = {
                    let state = custody
                        .latch()
                        .state
                        .lock()
                        .unwrap_or_else(|error| error.into_inner());
                    (!state.active).then_some(state.abandoned)
                };
                if let Some(abandoned) = ready {
                    break abandoned;
                }
                changed.await;
            };
            let settled = custody.process_settlement().await;
            if abandoned {
                return Err(backend_error(format!(
                    "native startup owner abandoned; process acknowledgment: {}",
                    settled
                        .as_ref()
                        .err()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "checked".into())
                )));
            }
            settled
        })
    }
}

struct StartupOwner {
    custody: NativeAttemptCustody,
    acknowledged: bool,
}
impl StartupOwner {
    fn finish(mut self) {
        self.acknowledged = true;
        let mut state = self
            .custody
            .latch()
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.active = false;
        let cancelled = state.cancelled;
        drop(state);
        if cancelled {
            self.custody.terminate_process();
        }
        self.custody.latch().changed.notify_waiters();
    }
}
impl Drop for StartupOwner {
    fn drop(&mut self) {
        if !self.acknowledged {
            let mut state = self
                .custody
                .latch()
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.active = false;
            state.abandoned = true;
            drop(state);
            self.custody.terminate_process();
            self.custody.latch().changed.notify_waiters();
        }
    }
}

/// Delivery alone does not transfer terminal ownership of selected source/home.
pub(crate) struct NativeStartupResources<C> {
    resources: Arc<std::sync::Mutex<C>>,
    transfer: oneshot::Sender<()>,
}
impl<C: Send + 'static> NativeStartupResources<C> {
    pub(crate) fn with_resource<R>(&self, inspect: impl FnOnce(&C) -> R) -> R {
        let held = self
            .resources
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        inspect(&held)
    }
    pub(crate) fn retain_completion(
        self,
        completion: DynAgentWrapperCompletion,
        settlement: Option<DynAgentWrapperSettlement>,
        terminate: impl FnOnce() + Send + 'static,
        timeout: Option<Duration>,
    ) -> (DynAgentWrapperCompletion, NativeTerminalObservation) {
        let Self {
            resources,
            transfer,
        } = self;
        let retained =
            retain_native_completion(completion, settlement, resources, terminate, timeout);
        // The actual completion task now owns the same resources. A queued channel
        // delivery was not this acknowledgment and cannot release the startup owner.
        let _ = transfer.send(());
        retained
    }
}

/// The task owns the actual boxed startup, not merely a drop notification.
pub(crate) async fn retain_native_startup<T: Send + 'static, C: Send + 'static>(
    startup: impl std::future::Future<Output = Result<T, AgentWrapperError>> + Send + 'static,
    custody: NativeAttemptCustody,
    resources: C,
) -> Result<(T, NativeStartupResources<C>), AgentWrapperError> {
    let owner = custody.begin_startup()?;
    let resources = Arc::new(std::sync::Mutex::new(resources));
    let (mut sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        let mut startup = Box::pin(startup);
        let outcome = tokio::select! {
            biased;
            _ = sender.closed() => None,
            _ = custody.cancelled() => None,
            value = &mut startup => Some(value),
        };
        // This is the actual startup future. External settlement waits for owner.
        drop(startup);
        match outcome {
            Some(Ok(control)) => {
                let (transfer, transferred) = oneshot::channel();
                let delivery = NativeStartupResources {
                    resources: Arc::clone(&resources),
                    transfer,
                };
                let queued = match sender.send(Ok((control, delivery))) {
                    Ok(()) => true,
                    Err(delivery) => {
                        if let Ok((control, delivery)) = delivery {
                            drop(control);
                            drop(delivery);
                        }
                        false
                    }
                };
                let retained = if queued {
                    tokio::select! {
                        result = transferred => result.is_ok(),
                        _ = custody.cancelled() => false,
                    }
                } else {
                    false
                };
                if !retained {
                    custody.terminate_process();
                    let _ = custody.process_settlement().await;
                }
                drop(resources);
            }
            failure => {
                custody.terminate_process();
                let settled = custody.process_settlement().await;
                let error = match (failure, settled) {
                    (Some(Err(error)), Ok(())) => error,
                    (Some(Err(error)), Err(settlement)) => backend_error(format!(
                        "submission_outcome_unknown: native startup failed: {error}; acknowledgment failed: {settlement}")),
                    (_, Err(error)) => backend_error(format!(
                        "submission_outcome_unknown: disposed native startup acknowledgment failed: {error}")),
                    _ => backend_error("native startup observer disposed after checked settlement"),
                };
                drop(resources);
                let _ = sender.send(Err(error));
            }
        }
        owner.finish();
    });
    receiver
        .await
        .map_err(|_| backend_error("submission_outcome_unknown: native startup owner abandoned"))?
}

/// Drive separately owned route/listener disposal alongside the native wait;
/// neither pending nor failed acknowledgment skips disposal before propagating the unchanged
/// native acknowledgment. Disposal cannot turn failed native settlement into success.
pub(crate) async fn finish_after_native_ack(
    native_ack: impl std::future::Future<Output = Result<(), AgentWrapperError>>,
    cleanup: impl std::future::Future<Output = ()>,
) -> Result<(), AgentWrapperError> {
    let (native_ack, ()) = tokio::join!(native_ack, cleanup);
    native_ack
}

#[cfg(all(test, target_os = "linux"))]
mod host_acquisition_binding_tests {
    use super::*;
    #[test]
    fn test_attempt_clones_cannot_reobserve_or_rekey_uncertain_acquisition() {
        for custody in [
            NativeAttemptCustody::codex(),
            NativeAttemptCustody::claude(),
        ] {
            custody.begin_host_acquisition().unwrap();
            assert!(custody.clone().begin_host_acquisition().is_err());
            custody.request_termination();
            assert!(custody.begin_host_acquisition().is_err());
        }
    }
}
