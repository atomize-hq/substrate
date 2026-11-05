use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use futures_util::StreamExt;
use tokio::sync::{mpsc, oneshot, Notify};

use super::normalize_request;
use super::settlement::{
    settle_attempt, BackendAttemptCustody, SettlementObservation, SettlementPublisher,
};
use super::{
    BackendDefaults, BackendHarnessAdapter, BackendHarnessErrorPhase, BackendSpawn,
    DynBackendCompletionFuture, DynBackendEventStream, NormalizedRequest,
};

use crate::{
    AgentWrapperCancelHandle, AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent,
    AgentWrapperEventKind, AgentWrapperRunControl, AgentWrapperRunHandle, AgentWrapperRunRequest,
};

#[allow(dead_code)]
type RequestTerminationHook = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Clone)]
#[allow(dead_code)]
struct HarnessCancelSignal {
    inner: Arc<HarnessCancelInner>,
}

#[allow(dead_code)]
struct HarnessCancelInner {
    cancelled: AtomicBool,
    notify: Notify,
}

#[allow(dead_code)]
impl HarnessCancelSignal {
    fn new() -> Self {
        Self {
            inner: Arc::new(HarnessCancelInner {
                cancelled: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    fn cancel(&self) {
        if self.inner.cancelled.swap(true, Ordering::SeqCst) {
            return;
        }
        self.inner.notify.notify_waiters();
    }

    async fn cancelled(&self) {
        let notified = self.inner.notify.notified();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }
}

fn pump_error_event(agent_kind: crate::AgentWrapperKind, message: String) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind,
        kind: AgentWrapperEventKind::Error,
        channel: Some("error".to_string()),
        text: None,
        message: Some(message),
        data: None,
    }
}

#[allow(dead_code)]
fn cancelled_completion_error() -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: "cancelled".to_string(),
    }
}

fn request_termination_best_effort(request_termination: Option<&RequestTerminationHook>) {
    if let Some(request_termination) = request_termination {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (request_termination)();
        }));
    }
}

async fn surface_spawn_failure(
    agent_kind: crate::AgentWrapperKind,
    message: String,
    tx: mpsc::Sender<AgentWrapperEvent>,
    completion_tx: oneshot::Sender<Result<AgentWrapperCompletion, AgentWrapperError>>,
) {
    for bounded in
        crate::bounds::enforce_event_bounds(pump_error_event(agent_kind, message.clone()))
    {
        if tx.send(bounded).await.is_err() {
            break;
        }
    }
    drop(tx);
    let _ = completion_tx.send(Err(AgentWrapperError::Backend { message }));
}

async fn surface_control_startup_failure(
    agent_kind: crate::AgentWrapperKind,
    message: String,
    cancel_signal: HarnessCancelSignal,
    request_termination: Option<RequestTerminationHook>,
    tx: mpsc::Sender<AgentWrapperEvent>,
    completion_tx: oneshot::Sender<Result<AgentWrapperCompletion, AgentWrapperError>>,
) {
    if cancel_signal.is_cancelled() {
        request_termination_best_effort(request_termination.as_ref());
        drop(tx);
        let _ = completion_tx.send(Err(cancelled_completion_error()));
        return;
    }

    for bounded in
        crate::bounds::enforce_event_bounds(pump_error_event(agent_kind, message.clone()))
    {
        tokio::select! {
            biased;
            _ = cancel_signal.cancelled() => {
                request_termination_best_effort(request_termination.as_ref());
                drop(tx);
                let _ = completion_tx.send(Err(cancelled_completion_error()));
                return;
            }
            send_outcome = tx.send(bounded) => {
                if send_outcome.is_err() {
                    break;
                }
            }
        }
    }

    if cancel_signal.is_cancelled() {
        request_termination_best_effort(request_termination.as_ref());
        drop(tx);
        let _ = completion_tx.send(Err(cancelled_completion_error()));
        return;
    }

    drop(tx);
    let _ = completion_tx.send(Err(AgentWrapperError::Backend { message }));
}

#[allow(clippy::too_many_arguments)]
async fn drive_control_backend_startup<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    normalized: NormalizedRequest<A::Policy>,
    agent_kind: crate::AgentWrapperKind,
    cancel_signal: HarnessCancelSignal,
    request_termination: Option<RequestTerminationHook>,
    custody: Option<BackendAttemptCustody>,
    publisher: Option<SettlementPublisher>,
    tx: mpsc::Sender<AgentWrapperEvent>,
    completion_tx: oneshot::Sender<Result<AgentWrapperCompletion, AgentWrapperError>>,
) {
    // Own the actual startup future in an Option so cancellation drops it before awaiting custody.
    // Dropping Pin<&mut Future> would only drop a reference and leave startup admission live.
    let mut spawn = Some(adapter.spawn(normalized));
    tokio::select! {
        biased;
        _ = cancel_signal.cancelled() => {
            drop(spawn.take());
            request_termination_best_effort(request_termination.as_ref());
            if let Some(custody) = &custody { (custody.request_termination)(); }
            drop(tx); // Stream closes promptly; terminal completion still waits for custody.
            if let Some(publisher) = &publisher { publisher.request_disposal(); }
            let settlement = settle_attempt(custody.as_ref()).await;
            if let Some(publisher) = publisher { finish_attempt_settlement(publisher, &settlement, custody.as_ref()); }
            let _ = completion_tx.send(Err(cancelled_completion_error()));
        }
        spawn_outcome = async { spawn.as_mut().expect("startup future present").await } => {
            drop(spawn.take());
            let spawned = match spawn_outcome {
                Ok(spawned) => spawned,
                Err(err) => {
                    if let Some(publisher) = &publisher { publisher.request_disposal(); }
                    let settlement = settle_attempt(custody.as_ref()).await;
                    let message = if settlement.is_err() { "backend process settlement failed".to_string() }
                        else { adapter.redact_error(BackendHarnessErrorPhase::Spawn, &err) };
                    if let Some(publisher) = publisher { finish_attempt_settlement(publisher, &settlement, custody.as_ref()); }
                    surface_control_startup_failure(agent_kind, message, cancel_signal,
                        request_termination, tx, completion_tx).await;
                    return;
                }
            };
            let BackendSpawn { events, completion, events_observability } = spawned;
            tokio::spawn(send_completion_with_custody(
                adapter.clone(), completion, cancel_signal.clone(), request_termination,
                custody, publisher, completion_tx,
            ));
            tokio::spawn(pump_backend_events_with_cancel(adapter, events, tx, cancel_signal, events_observability));
        }
    }
}

fn start_backend_runtime<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    spawned: BackendSpawn<A::BackendEvent, A::BackendCompletion, A::BackendError>,
    tx: mpsc::Sender<AgentWrapperEvent>,
    completion_tx: oneshot::Sender<Result<AgentWrapperCompletion, AgentWrapperError>>,
) {
    let BackendSpawn {
        events,
        completion,
        events_observability,
    } = spawned;

    tokio::spawn({
        let adapter = adapter.clone();
        async move {
            let completion_outcome = completion.await;
            let completion_outcome: Result<AgentWrapperCompletion, AgentWrapperError> =
                match completion_outcome {
                    Ok(typed) => adapter.map_completion(typed),
                    Err(err) => Err(AgentWrapperError::Backend {
                        message: adapter.redact_error(BackendHarnessErrorPhase::Completion, &err),
                    }),
                }
                .map(crate::bounds::enforce_completion_bounds);

            let _ = completion_tx.send(completion_outcome);
        }
    });

    tokio::spawn(pump_backend_events(
        adapter,
        events,
        tx,
        events_observability,
    ));
}

async fn pump_backend_events<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    mut events: DynBackendEventStream<A::BackendEvent, A::BackendError>,
    tx: mpsc::Sender<AgentWrapperEvent>,
    events_observability: Option<super::EventObservabilitySignal>,
) {
    // BH-C04 (SEAM-3) pinned semantics:
    // - Forward mapped + bounds-enforced universal events while the receiver is alive.
    // - Receiver drop MUST be detected only via `tx.send(...).await` returning `Err(_)`.
    // - After the first send failure, stop forwarding entirely (no further mapping/bounds/sends),
    //   but keep draining the typed backend stream until it ends.
    // - Finality signal for DR-0012 gating is the drop of this `Sender`; the sender MUST be
    //   dropped only once the backend stream has ended (receiver drop is not finality).
    let mut forward = true;
    while let Some(outcome) = events.next().await {
        if !forward {
            continue;
        }

        let mapped: Vec<AgentWrapperEvent> = match outcome {
            Ok(ev) => adapter.map_event(ev),
            Err(err) => vec![pump_error_event(
                adapter.kind(),
                adapter.redact_error(BackendHarnessErrorPhase::Stream, &err),
            )],
        };

        for event in mapped {
            for bounded in crate::bounds::enforce_event_bounds(event) {
                if tx.send(bounded).await.is_err() {
                    forward = false;
                    if let Some(signal) = events_observability.as_ref() {
                        signal.signal();
                    }
                    break;
                }
            }
            if !forward {
                break;
            }
        }
    }

    // Finality signal (BH-C04): drop the sender only after the backend stream ends.
    drop(tx);
}

#[allow(dead_code)]
async fn pump_backend_events_with_cancel<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    mut events: DynBackendEventStream<A::BackendEvent, A::BackendError>,
    tx: mpsc::Sender<AgentWrapperEvent>,
    cancel: HarnessCancelSignal,
    events_observability: Option<super::EventObservabilitySignal>,
) {
    // CA-C02 (SEAM-2) semantics:
    // - Explicit cancellation is orthogonal to receiver drop.
    // - On cancellation: stop forwarding immediately, proactively close the universal stream by
    //   dropping the sender, but keep draining the typed backend stream to completion.
    // - On receiver drop: preserve BH-C04 drain-on-drop posture (stop forwarding on first send
    //   failure, but keep draining; sender is dropped only after stream end).
    let mut tx = Some(tx);
    let mut forward = true;
    let mut cancelled = false;

    loop {
        tokio::select! {
            biased;
            _ = cancel.cancelled(), if !cancelled => {
                cancelled = true;
                forward = false;
                // Close the universal stream (consumer sees `None`) without affecting drain.
                if let Some(signal) = events_observability.as_ref() {
                    signal.signal();
                }
                drop(tx.take());
            }
            maybe_outcome = events.next() => {
                let Some(outcome) = maybe_outcome else { break; };
                if !forward {
                    continue;
                }

                let mapped: Vec<AgentWrapperEvent> = match outcome {
                    Ok(ev) => adapter.map_event(ev),
                    Err(err) => vec![pump_error_event(
                        adapter.kind(),
                        adapter.redact_error(BackendHarnessErrorPhase::Stream, &err),
                    )],
                };

                'mapped: for event in mapped {
                    for bounded in crate::bounds::enforce_event_bounds(event) {
                        let Some(sender) = tx.clone() else {
                            forward = false;
                            break 'mapped;
                        };

                        tokio::select! {
                            biased;
                            _ = cancel.cancelled(), if !cancelled => {
                                cancelled = true;
                                forward = false;
                                if let Some(signal) = events_observability.as_ref() {
                                    signal.signal();
                                }
                                drop(tx.take());
                                break 'mapped;
                            }
                            send_outcome = sender.send(bounded) => {
                                if send_outcome.is_err() {
                                    forward = false;
                                    if let Some(signal) = events_observability.as_ref() {
                                        signal.signal();
                                    }
                                    break 'mapped;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // BH-C04 finality posture: if cancellation never occurred, drop the sender only after typed
    // backend stream end.
    drop(tx);
}

#[allow(dead_code)]
async fn send_completion_with_cancel<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    completion: DynBackendCompletionFuture<A::BackendCompletion, A::BackendError>,
    cancel: HarnessCancelSignal,
    request_termination: Option<RequestTerminationHook>,
    completion_tx: oneshot::Sender<Result<AgentWrapperCompletion, AgentWrapperError>>,
) {
    send_completion_with_custody(
        adapter,
        completion,
        cancel,
        request_termination,
        None,
        None,
        completion_tx,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn send_completion_with_custody<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    mut completion: DynBackendCompletionFuture<A::BackendCompletion, A::BackendError>,
    cancel: HarnessCancelSignal,
    request_termination: Option<RequestTerminationHook>,
    custody: Option<BackendAttemptCustody>,
    publisher: Option<SettlementPublisher>,
    completion_tx: oneshot::Sender<Result<AgentWrapperCompletion, AgentWrapperError>>,
) {
    let (cancelled, completion_outcome) = tokio::select! {
        biased;
        _ = cancel.cancelled() => {
            if let Some(publisher) = &publisher { publisher.request_disposal(); }
            request_termination_best_effort(request_termination.as_ref());
            if let Some(custody) = &custody { (custody.request_termination)(); }
            (true, completion.await)
        }
        outcome = &mut completion => (false, outcome),
    };
    if let Some(publisher) = &publisher {
        publisher.request_disposal();
    }
    let settlement = settle_attempt(custody.as_ref()).await;
    if let Some(publisher) = publisher {
        finish_attempt_settlement(publisher, &settlement, custody.as_ref());
    }
    // Cancellation still selects its exact normative value; separate acknowledgment retains
    // cleanup failure. A successful non-cancel completion cannot hide that failure.
    let outcome = if cancelled || cancel.is_cancelled() {
        Err(cancelled_completion_error())
    } else if let Err(error) = settlement {
        Err(error)
    } else {
        match completion_outcome {
            Ok(typed) => adapter.map_completion(typed),
            Err(err) => Err(AgentWrapperError::Backend {
                message: adapter.redact_error(BackendHarnessErrorPhase::Completion, &err),
            }),
        }
        .map(crate::bounds::enforce_completion_bounds)
    };
    let _ = completion_tx.send(outcome);
}

fn finish_attempt_settlement(
    publisher: SettlementPublisher,
    result: &Result<(), AgentWrapperError>,
    custody: Option<&BackendAttemptCustody>,
) {
    let closed_before_any_spawn = custody
        .and_then(|custody| custody.observe_closed_before_any_spawn.as_ref())
        .map(|observe| observe());
    match custody.and_then(|custody| custody.observe_processes.as_ref()) {
        Some(observe) => publisher.finish_with_process_observations(
            result,
            Some(observe()),
            closed_before_any_spawn,
        ),
        None => publisher.finish(result),
    }
}

#[allow(dead_code)]
pub(crate) async fn run_harnessed_backend_control<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    defaults: BackendDefaults,
    request: AgentWrapperRunRequest,
    request_termination: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
) -> Result<AgentWrapperRunControl, AgentWrapperError> {
    let normalized = normalize_request(adapter.as_ref(), &defaults, request)?;
    let custody = adapter.attempt_custody(&normalized);
    let supports_process_observation = custody
        .as_ref()
        .is_some_and(|custody| custody.observe_processes.is_some());
    let observation = custody.as_ref().map(|_| SettlementObservation::default());
    let publisher = observation.as_ref().map(SettlementObservation::publisher);
    let agent_kind = normalized.agent_kind.clone();
    let (tx, rx) = mpsc::channel::<AgentWrapperEvent>(super::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let cancel_signal = HarnessCancelSignal::new();
    let signal_cancel = {
        let signal = cancel_signal.clone();
        let observation = observation.clone();
        move || {
            if let Some(observation) = &observation {
                observation.request_disposal();
            }
            signal.cancel();
        }
    };
    let cancel = match observation {
        Some(observation) => AgentWrapperCancelHandle::new_with_settlement_and_process_observation(
            signal_cancel,
            observation.factory(),
            observation.disposal_factory(),
            supports_process_observation.then(|| observation.process_factory()),
        ),
        None => AgentWrapperCancelHandle::new(signal_cancel),
    };

    tokio::spawn(drive_control_backend_startup(
        adapter,
        normalized,
        agent_kind,
        cancel_signal,
        request_termination,
        custody,
        publisher,
        tx,
        completion_tx,
    ));

    Ok(AgentWrapperRunControl {
        handle: crate::run_handle_gate::build_gated_run_handle(rx, completion_rx),
        cancel,
    })
}

pub(crate) async fn run_harnessed_backend<A: BackendHarnessAdapter>(
    adapter: Arc<A>,
    defaults: BackendDefaults,
    request: AgentWrapperRunRequest,
) -> Result<AgentWrapperRunHandle, AgentWrapperError> {
    let normalized = normalize_request(adapter.as_ref(), &defaults, request)?;
    let agent_kind = normalized.agent_kind.clone();
    let (tx, rx) = mpsc::channel::<AgentWrapperEvent>(super::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    tokio::spawn(async move {
        let spawn_outcome = adapter.spawn(normalized).await;
        let spawned = match spawn_outcome {
            Ok(spawned) => spawned,
            Err(err) => {
                let message = adapter.redact_error(BackendHarnessErrorPhase::Spawn, &err);
                surface_spawn_failure(agent_kind, message, tx, completion_tx).await;
                return;
            }
        };

        start_backend_runtime(adapter, spawned, tx, completion_tx);
    });

    Ok(crate::run_handle_gate::build_gated_run_handle(
        rx,
        completion_rx,
    ))
}

#[cfg(test)]
mod tests;
