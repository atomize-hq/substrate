use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs::Metadata;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use substrate_common::macos_retirement_v2::{
    document_sha256_v2, parse_canonical_v2, sha256_hex_v2,
};
use substrate_r3_macos_finalizer::experiment::controls::{
    single_securityagent_terminal_alert_evidence_v2, SecurityAgentArmEvidenceV2,
    SecurityAgentRawReportEvidenceV2,
};
use substrate_r3_macos_finalizer::experiment::publisher_protocol::{
    PublisherSecurityAgentObservationV2, PublisherStageV2, PUBLISHER_PROTOCOL_OWNER_V2,
};
use substrate_r3_macos_finalizer::experiment::{
    RepetitionV2, EXPERIMENT_ID_V2, EXPERIMENT_VERSION_V2, SECURITYAGENT_OBSERVER_PATH_V2,
};

const OBSERVER_READY_DEADLINE: Duration = Duration::from_secs(10);
const OBSERVER_SAMPLE_COUNT: u64 = 101;
const OBSERVER_SAMPLE_INTERVAL_MS: u64 = 50;
const OBSERVED_CHILD_DEADLINE: Duration = Duration::from_millis(4_500);
const OBSERVER_REARM_AFTER: Duration = Duration::from_secs(4);
type RawObserverReportV2 = (String, Vec<u8>);
type RearmedObservedResultV2<T> = (Result<T>, Vec<RawObserverReportV2>);

#[derive(Debug)]
pub(crate) struct ObservedChildV2 {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub securityagent_report_sha256: String,
    pub securityagent_report: Vec<u8>,
    pub unexpected_ui_observed: bool,
}

pub(crate) struct ObservedStartupChildV2<T> {
    pub child: Child,
    pub attestation: T,
    pub securityagent_report_sha256: String,
    pub securityagent_report: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct SecurityAgentTerminalAlertV2 {
    pub raw_report: SecurityAgentRawReportEvidenceV2,
    pub arm_evidence: SecurityAgentArmEvidenceV2,
}

impl std::fmt::Display for SecurityAgentTerminalAlertV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecurityAgent observer emitted a canonical terminal ALERT report")
    }
}

impl std::error::Error for SecurityAgentTerminalAlertV2 {}

/// Run one closed publisher stage entirely inside one fixed observer window. ALERT is deliberately
/// process-fatal: no publisher code is allowed to mutate, publish success, or continue after UI.
pub(crate) fn observe_stage<T, F, A>(
    repetition: RepetitionV2,
    stage: PublisherStageV2,
    prearm_baseline_sha256: &str,
    operation: F,
    persist_terminal_alert: A,
) -> Result<(Result<T>, PublisherSecurityAgentObservationV2)>
where
    T: Send,
    F: FnOnce() -> Result<T> + Send,
    A: FnOnce(&SecurityAgentTerminalAlertV2) -> Result<()>,
{
    let started = monotonic_nanoseconds()?;
    let (result, raw_report_sha256, raw_report) =
        observe_supervised_operation(operation, persist_terminal_alert)?;
    let finished = monotonic_nanoseconds()?;
    let raw = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(&raw_report),
        raw_report_sha256,
        raw_report_byte_length: u64::try_from(raw_report.len())
            .context("publisher SecurityAgent report length exceeds u64")?,
    };
    raw.validate()?;
    let arm_evidence = SecurityAgentArmEvidenceV2 {
        schema_owner: "substrate.r3-macos-securityagent-arm-observation".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        observer_path: SECURITYAGENT_OBSERVER_PATH_V2.to_owned(),
        reports: vec![raw],
        report_set_sha256: String::new(),
        overlap_rearm_count: 0,
        gap_free_rearm_coverage: true,
        unexpected_ui_observed: false,
    };
    let arm_evidence = SecurityAgentArmEvidenceV2 {
        report_set_sha256: document_sha256_v2(&arm_evidence.reports)?,
        ..arm_evidence
    };
    arm_evidence.validate()?;
    let observation = PublisherSecurityAgentObservationV2 {
        schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        stage,
        prearm_baseline_sha256: prearm_baseline_sha256.to_owned(),
        arm_evidence_sha256: document_sha256_v2(&arm_evidence)?,
        arm_evidence,
        observation_started_monotonic_ns: started,
        observation_finished_monotonic_ns: finished,
        unexpected_ui_observed: false,
    };
    observation.validate(repetition, stage)?;
    Ok((result, observation))
}

/// Observe one short, sealed creator child. The child is killed immediately on ALERT or if its
/// bounded arm outlives the one fixed observer window, allowing the root runner to enter only its
/// compiled rollback path rather than terminating before cleanup.
pub(crate) fn observe_sealed_child(command: Command) -> Result<ObservedChildV2> {
    observe_sealed_child_with(command, |_| Ok(())).map(|(observed, _)| observed)
}

/// Observe a long-lived sealed child from `exec` through its exact post-first-Security-call
/// SIGSTOP rendezvous and caller-supplied process/code attestation.  The child remains stopped
/// until the complete no-UI report is validated, then is returned to the caller for one SIGCONT.
/// ALERT kills the exact child, durably delegates its canonical trigger report, and exits 86.
pub(crate) fn observe_stopped_child_startup<T, F, A>(
    mut command: Command,
    attest_stopped: F,
    persist_terminal_alert: A,
) -> Result<ObservedStartupChildV2<T>>
where
    T: Send,
    F: FnOnce(u32) -> Result<T> + Send,
    A: FnOnce(&SecurityAgentTerminalAlertV2) -> Result<()>,
{
    let (observer, signals) = match start_supervised_observer() {
        Ok(value) => value,
        Err(error) => {
            if let Some(alert) = error.downcast_ref::<SecurityAgentTerminalAlertV2>() {
                persist_terminal_alert(alert)?;
                std::process::exit(86);
            }
            return Err(error);
        }
    };
    let mut child = command
        .spawn()
        .context("spawn long-lived child under startup observer")?;
    let child_pid = child.id();
    let (attestation, report_sha256, report) =
        thread::scope(|scope| -> Result<(T, String, Vec<u8>)> {
            let deadline = std::time::Instant::now() + OBSERVER_READY_DEADLINE;
            loop {
                let mut status = 0;
                // SAFETY: this is the one freshly spawned exact child; WUNTRACED observes its
                // compiled SIGSTOP rendezvous without waiting unboundedly.
                let observed = unsafe {
                    libc::waitpid(
                        i32::try_from(child_pid).unwrap_or(-1),
                        &mut status,
                        libc::WUNTRACED | libc::WNOHANG,
                    )
                };
                if observed == i32::try_from(child_pid)? {
                    if libc::WIFSTOPPED(status) && libc::WSTOPSIG(status) == libc::SIGSTOP {
                        break;
                    }
                    let mut observer = observer;
                    kill_and_wait(&mut observer);
                    bail!("startup child exited or changed state before exact SIGSTOP")
                }
                if observed < 0 {
                    let error = std::io::Error::last_os_error();
                    let mut observer = observer;
                    kill_and_wait(&mut observer);
                    return Err(error).context("observe startup child SIGSTOP");
                }
                match signals.recv_timeout(Duration::from_millis(10)) {
                    Ok(ObserverSignalV2::Alert) => {
                        kill_and_wait(&mut child);
                        let (digest, raw, unexpected) = finish_observer(observer)?;
                        if !unexpected {
                            std::process::exit(78);
                        }
                        let alert = terminal_alert_evidence(digest, raw)?;
                        if persist_terminal_alert(&alert).is_err() {
                            std::process::exit(78);
                        }
                        std::process::exit(86);
                    }
                    Ok(ObserverSignalV2::Invalid)
                    | Ok(ObserverSignalV2::Eof)
                    | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        kill_and_wait(&mut child);
                        std::process::exit(78);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if std::time::Instant::now() >= deadline {
                    kill_and_wait(&mut child);
                    let mut observer = observer;
                    kill_and_wait(&mut observer);
                    bail!("startup child missed exact SIGSTOP inside observer window")
                }
            }

            let (sender, receiver) = mpsc::sync_channel(1);
            scope.spawn(move || {
                let _ = sender.send(attest_stopped(child_pid));
            });
            loop {
                match signals.recv_timeout(Duration::from_millis(10)) {
                    Ok(ObserverSignalV2::Alert) => {
                        kill_and_wait(&mut child);
                        let (digest, raw, unexpected) = finish_observer(observer)?;
                        if !unexpected {
                            std::process::exit(78);
                        }
                        let alert = terminal_alert_evidence(digest, raw)?;
                        if persist_terminal_alert(&alert).is_err() {
                            std::process::exit(78);
                        }
                        std::process::exit(86);
                    }
                    Ok(ObserverSignalV2::Invalid)
                    | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        kill_and_wait(&mut child);
                        std::process::exit(78);
                    }
                    Ok(ObserverSignalV2::Eof) => {
                        let attestation = match receiver.try_recv() {
                            Ok(value) => value.context("attest stopped startup child")?,
                            Err(_) => {
                                kill_and_wait(&mut child);
                                std::process::exit(78);
                            }
                        };
                        let (digest, raw, unexpected) = finish_observer(observer)?;
                        if unexpected {
                            std::process::exit(78);
                        }
                        return Ok((attestation, digest, raw));
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
        })?;
    Ok(ObservedStartupChildV2 {
        child,
        attestation,
        securityagent_report_sha256: report_sha256,
        securityagent_report: report,
    })
}

pub(crate) fn observe_sealed_child_with<T, F>(
    mut command: Command,
    after_spawn: F,
) -> Result<(ObservedChildV2, Option<T>)>
where
    T: Send,
    F: FnOnce(u32) -> Result<T> + Send,
{
    let (observer, signals) = start_supervised_observer()?;
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn sealed observed child")?;
    thread::scope(|scope| -> Result<(ObservedChildV2, Option<T>)> {
        let (after_sender, after_receiver) = mpsc::sync_channel(1);
        let child_pid = child.id();
        scope.spawn(move || {
            let _ = after_sender.send(after_spawn(child_pid));
        });
        let mut after_spawn = None;
        let deadline = std::time::Instant::now() + OBSERVED_CHILD_DEADLINE;
        loop {
            if after_spawn.is_none() {
                match after_receiver.try_recv() {
                    Ok(Ok(value)) => after_spawn = Some(value),
                    Ok(Err(error)) => {
                        kill_and_wait(&mut child);
                        let mut observer = observer;
                        kill_and_wait(&mut observer);
                        return Err(error).context("validate sealed child after exec");
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        kill_and_wait(&mut child);
                        let mut observer = observer;
                        kill_and_wait(&mut observer);
                        bail!("sealed child after-spawn attestation channel disconnected")
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }
            match signals.try_recv() {
                Ok(ObserverSignalV2::Alert) => {
                    // The attestation/rendezvous runs concurrently so ALERT can kill the sealed
                    // child immediately rather than waiting up to its bounded SIGSTOP deadline.
                    kill_and_wait(&mut child);
                    let (securityagent_report_sha256, securityagent_report, unexpected) =
                        finish_observer(observer)?;
                    if !unexpected {
                        bail!("SecurityAgent ALERT lacked its canonical trigger report")
                    }
                    let status = child
                        .try_wait()?
                        .context("alert-killed child did not report one status")?;
                    return Ok((
                        ObservedChildV2 {
                            status,
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                            securityagent_report_sha256,
                            securityagent_report,
                            unexpected_ui_observed: true,
                        },
                        None,
                    ));
                }
                Ok(ObserverSignalV2::Invalid) | Err(mpsc::TryRecvError::Disconnected) => {
                    kill_and_wait(&mut child);
                    let mut observer = observer;
                    kill_and_wait(&mut observer);
                    bail!("SecurityAgent observer stream failed during sealed child")
                }
                Ok(ObserverSignalV2::Eof) => {
                    if child.try_wait()?.is_none() || after_spawn.is_none() {
                        kill_and_wait(&mut child);
                        bail!("SecurityAgent observer ended while sealed child work remained live")
                    }
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
            if child.try_wait()?.is_some() && after_spawn.is_some() {
                break;
            }
            if std::time::Instant::now() >= deadline {
                kill_and_wait(&mut child);
                let mut observer = observer;
                kill_and_wait(&mut observer);
                bail!("sealed child exceeded its fixed SecurityAgent observation window")
            }
            thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output()?;
        let (securityagent_report_sha256, securityagent_report, unexpected_ui_observed) =
            finish_observer(observer)?;
        while let Ok(signal) = signals.try_recv() {
            match signal {
                ObserverSignalV2::Alert => {
                    return Ok((
                        ObservedChildV2 {
                            status: output.status,
                            stdout: output.stdout,
                            stderr: output.stderr,
                            securityagent_report_sha256,
                            securityagent_report,
                            unexpected_ui_observed: true,
                        },
                        None,
                    ));
                }
                ObserverSignalV2::Invalid => {
                    bail!("SecurityAgent observer emitted invalid data during sealed child")
                }
                ObserverSignalV2::Eof => {}
            }
        }
        Ok((
            ObservedChildV2 {
                status: output.status,
                stdout: output.stdout,
                stderr: output.stderr,
                securityagent_report_sha256,
                securityagent_report,
                unexpected_ui_observed,
            },
            after_spawn,
        ))
    })
}

pub(crate) fn observe_idle_baseline_sha256() -> Result<String> {
    finish_observer(start_observer()?).and_then(|(digest, _, unexpected)| {
        if unexpected {
            bail!("SecurityAgent ALERT changed the idle baseline")
        }
        Ok(digest)
    })
}

/// The raw-report variant is reserved for canonical experiment receipts that must retain the
/// observer bytes rather than only their digest.
pub(crate) fn observe_root_operation_with_raw<T, F, A>(
    operation: F,
    persist_terminal_alert: A,
) -> Result<(Result<T>, String, Vec<u8>)>
where
    T: Send,
    F: FnOnce() -> Result<T> + Send,
    A: FnOnce(&SecurityAgentTerminalAlertV2) -> Result<()>,
{
    observe_supervised_operation(operation, persist_terminal_alert)
}

/// Observe a bounded operation that may exceed one 5.05-second observer window.  Each successor
/// is READY before four seconds, leaving more than one second of overlap.  Every raw canonical
/// window is returned for durable archival; no EOF may leave an operation without a live arm.
pub(crate) fn observe_rearmed_root_operation_with_raw<T, F, A>(
    operation: F,
    persist_terminal_alert: A,
) -> Result<RearmedObservedResultV2<T>>
where
    T: Send,
    F: FnOnce() -> Result<T> + Send,
    A: Fn(&SecurityAgentTerminalAlertV2) -> Result<()>,
{
    struct Window {
        observer: Child,
        signals: mpsc::Receiver<ObserverSignalV2>,
        started: std::time::Instant,
        successor_requested: bool,
        successor_ready: bool,
    }

    let (observer, signals) = match start_supervised_observer() {
        Ok(value) => value,
        Err(error) => {
            if let Some(alert) = error.downcast_ref::<SecurityAgentTerminalAlertV2>() {
                persist_terminal_alert(alert)?;
                std::process::exit(86);
            }
            return Err(error);
        }
    };
    thread::scope(move |scope| -> Result<RearmedObservedResultV2<T>> {
        let operation = scope.spawn(operation);
        let mut windows = vec![Window {
            observer,
            signals,
            started: std::time::Instant::now(),
            successor_requested: false,
            successor_ready: false,
        }];
        let mut pending_successor = None;
        let mut reports = Vec::new();
        loop {
            if !operation.is_finished()
                && windows.last().is_some_and(|window| {
                    !window.successor_requested && window.started.elapsed() >= OBSERVER_REARM_AFTER
                })
            {
                windows
                    .last_mut()
                    .expect("one observer window is live before rearm")
                    .successor_requested = true;
                let (sender, receiver) = mpsc::sync_channel(1);
                thread::spawn(move || {
                    let _ = sender.send(start_supervised_observer());
                });
                pending_successor = Some(receiver);
            }
            if let Some(receiver) = pending_successor.as_ref() {
                match receiver.try_recv() {
                    Ok(Ok((observer, signals))) => {
                        windows
                            .last_mut()
                            .expect("predecessor observer remains live until successor READY")
                            .successor_ready = true;
                        windows.push(Window {
                            observer,
                            signals,
                            started: std::time::Instant::now(),
                            successor_requested: false,
                            successor_ready: false,
                        });
                        pending_successor = None;
                    }
                    Ok(Err(error)) => {
                        if let Some(alert) = error.downcast_ref::<SecurityAgentTerminalAlertV2>() {
                            if persist_terminal_alert(alert).is_err() {
                                std::process::exit(78);
                            }
                            std::process::exit(86);
                        }
                        std::process::exit(78)
                    }
                    Err(mpsc::TryRecvError::Disconnected) => std::process::exit(78),
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }

            let mut index = 0;
            while index < windows.len() {
                match windows[index].signals.try_recv() {
                    Ok(ObserverSignalV2::Alert) => {
                        let window = windows.remove(index);
                        let (digest, raw, unexpected) = finish_observer(window.observer)?;
                        if !unexpected {
                            std::process::exit(78);
                        }
                        let alert = terminal_alert_evidence(digest, raw)?;
                        if persist_terminal_alert(&alert).is_err() {
                            std::process::exit(78);
                        }
                        std::process::exit(86);
                    }
                    Ok(ObserverSignalV2::Invalid)
                    | Err(mpsc::TryRecvError::Disconnected) => std::process::exit(78),
                    Ok(ObserverSignalV2::Eof) => {
                        let was_latest = index + 1 == windows.len();
                        let window = windows.remove(index);
                        let (digest, raw, unexpected) = finish_observer(window.observer)?;
                        if unexpected {
                            std::process::exit(78);
                        }
                        reports.push((digest, raw));
                        if was_latest && !operation.is_finished() {
                            // A live operation without an already-READY successor is an
                            // unobserved interval and is process-fatal.
                            std::process::exit(78);
                        }
                        continue;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }
                index += 1;
            }
            if operation.is_finished() && windows.is_empty() {
                let result = operation
                    .join()
                    .map_err(|_| anyhow::anyhow!("rearmed observed operation panicked"))?;
                if reports.is_empty() {
                    bail!("rearmed operation completed without one canonical observer report")
                }
                return Ok((result, reports));
            }
            thread::sleep(Duration::from_millis(10));
        }
    })
}

fn observe_supervised_operation<T, F, A>(
    operation: F,
    persist_terminal_alert: A,
) -> Result<(Result<T>, String, Vec<u8>)>
where
    T: Send,
    F: FnOnce() -> Result<T> + Send,
    A: FnOnce(&SecurityAgentTerminalAlertV2) -> Result<()>,
{
    let (observer, signals) = match start_supervised_observer() {
        Ok(value) => value,
        Err(error) => {
            if let Some(alert) = error.downcast_ref::<SecurityAgentTerminalAlertV2>() {
                persist_terminal_alert(alert)?;
                std::process::exit(86);
            }
            return Err(error);
        }
    };
    thread::scope(move |scope| -> Result<(Result<T>, String, Vec<u8>)> {
        let operation = scope.spawn(operation);
        let mut observer = Some(observer);
        loop {
            match signals.recv_timeout(Duration::from_millis(10)) {
                Ok(ObserverSignalV2::Alert) => {
                    let child = observer.take().expect("observer is consumed only once");
                    let (report_sha256, report, unexpected) = match finish_observer(child) {
                        Ok(value) => value,
                        Err(_) => std::process::exit(78),
                    };
                    if !unexpected {
                        std::process::exit(78);
                    }
                    let alert = terminal_alert_evidence(report_sha256, report)?;
                    if persist_terminal_alert(&alert).is_err() {
                        std::process::exit(78);
                    }
                    // Never wait for or unwind through a still-running effect after ALERT. The
                    // durable trigger cursor is the only authority admitted on the next process.
                    std::process::exit(86);
                }
                Ok(ObserverSignalV2::Invalid) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    std::process::exit(78);
                }
                Ok(ObserverSignalV2::Eof) => {
                    let child = observer.take().expect("observer is consumed only once");
                    let (report_sha256, report, unexpected) = match finish_observer(child) {
                        Ok(value) => value,
                        Err(_) if !operation.is_finished() => std::process::exit(78),
                        Err(error) => return Err(error),
                    };
                    if unexpected || !operation.is_finished() {
                        std::process::exit(78);
                    }
                    let result = operation
                        .join()
                        .map_err(|_| anyhow::anyhow!("observed root operation panicked"))?;
                    return Ok((result, report_sha256, report));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
    })
}

#[derive(Debug, Clone, Copy)]
enum ObserverSignalV2 {
    Alert,
    Invalid,
    Eof,
}

fn start_supervised_observer() -> Result<(Child, mpsc::Receiver<ObserverSignalV2>)> {
    require_root_owned_fixed_observer()?;
    let mut command = Command::new(SECURITYAGENT_OBSERVER_PATH_V2);
    command
        .current_dir("/")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in std::env::vars_os()
        .map(|(key, _)| key)
        .filter(|key| key.to_string_lossy().starts_with("SUBSTRATE_"))
    {
        command.env_remove(key);
    }
    let mut child = command.spawn().context("spawn supervised UI observer")?;
    let stderr = child
        .stderr
        .take()
        .context("supervised observer lacks stderr")?;
    let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
    let (signal_sender, signal_receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        let ready = reader.read_line(&mut line).map(|_| line.clone());
        let _ = ready_sender.send(ready);
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                let _ = signal_sender.send(ObserverSignalV2::Eof);
            }
            Ok(_) if line == "ALERT\n" => {
                let _ = signal_sender.send(ObserverSignalV2::Alert);
            }
            Ok(_) | Err(_) => {
                let _ = signal_sender.send(ObserverSignalV2::Invalid);
            }
        }
    });
    match ready_receiver.recv_timeout(OBSERVER_READY_DEADLINE) {
        Ok(Ok(line)) if line == "READY\n" => Ok((child, signal_receiver)),
        Ok(Ok(line)) if line == "ALERT\n" => {
            let (digest, report, unexpected) = finish_observer(child)?;
            if !unexpected {
                bail!("SecurityAgent ALERT marker lacked its canonical alert report")
            }
            Err(anyhow::Error::new(terminal_alert_evidence(digest, report)?))
        }
        Ok(Ok(_)) => {
            kill_and_wait(&mut child);
            bail!("SecurityAgent observer emitted alternate ready marker")
        }
        Ok(Err(error)) => {
            kill_and_wait(&mut child);
            Err(error).context("read supervised observer ready marker")
        }
        Err(_) => {
            kill_and_wait(&mut child);
            bail!("supervised observer ready deadline expired")
        }
    }
}

fn start_observer() -> Result<Child> {
    require_root_owned_fixed_observer()?;
    let mut command = Command::new(SECURITYAGENT_OBSERVER_PATH_V2);
    command
        .current_dir("/")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in std::env::vars_os()
        .map(|(key, _)| key)
        .filter(|key| key.to_string_lossy().starts_with("SUBSTRATE_"))
    {
        command.env_remove(key);
    }
    let mut child = command
        .spawn()
        .context("spawn fixed SecurityAgent observer")?;
    let stderr = child
        .stderr
        .take()
        .context("fixed SecurityAgent observer lacks stderr")?;
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        let first = reader.read_line(&mut line).map(|_| line.clone());
        let _ = sender.send(first);
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {}
            Ok(_) if line == "ALERT\n" => {}
            Ok(_) | Err(_) => {}
        }
    });
    match receiver.recv_timeout(OBSERVER_READY_DEADLINE) {
        Ok(Ok(line)) if line == "READY\n" => Ok(child),
        Ok(Ok(line)) if line == "ALERT\n" => {
            let (digest, report, unexpected) = finish_observer(child)?;
            if !unexpected {
                bail!("SecurityAgent ALERT marker lacked its canonical alert report")
            }
            Err(anyhow::Error::new(terminal_alert_evidence(digest, report)?))
        }
        Ok(Ok(_)) => {
            kill_and_wait(&mut child);
            bail!("SecurityAgent observer emitted an alternate ready marker")
        }
        Ok(Err(error)) => {
            kill_and_wait(&mut child);
            Err(error).context("read SecurityAgent observer ready marker")
        }
        Err(_) => {
            kill_and_wait(&mut child);
            bail!("SecurityAgent observer ready deadline expired")
        }
    }
}

fn finish_observer(child: Child) -> Result<(String, Vec<u8>, bool)> {
    let output = child
        .wait_with_output()
        .context("wait for fixed SecurityAgent observer")?;
    if !output.status.success() && output.status.code() != Some(86) {
        bail!("fixed SecurityAgent observer did not complete successfully")
    }
    let mut bytes = output.stdout;
    if bytes.pop() != Some(b'\n') || bytes.last() == Some(&b'\n') {
        bail!("SecurityAgent observer output is not one newline-terminated report")
    }
    let report: Value = parse_canonical_v2(&bytes)?;
    let unexpected = report
        .as_object()
        .and_then(|value| value.get("unexpectedUiObserved"))
        .and_then(Value::as_bool)
        .context("SecurityAgent report lacks its UI result")?;
    validate_raw_report(&report, unexpected)?;
    if output.status.success() == unexpected {
        bail!("SecurityAgent report and observer exit classification disagree")
    }
    Ok((sha256_hex_v2(&bytes), bytes, unexpected))
}

fn validate_raw_report(report: &Value, unexpected: bool) -> Result<()> {
    let object = report
        .as_object()
        .context("SecurityAgent observation is not an object")?;
    if object.get("schemaOwner").and_then(Value::as_str)
        != Some("substrate.r3-macos-securityagent-observation")
        || object.get("schemaVersion").and_then(Value::as_u64) != Some(1)
        || !object
            .get("sampleCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| {
                if unexpected {
                    (1..=OBSERVER_SAMPLE_COUNT).contains(&count)
                } else {
                    count == OBSERVER_SAMPLE_COUNT
                }
            })
        || object
            .get("sampleIntervalMilliseconds")
            .and_then(Value::as_u64)
            != Some(OBSERVER_SAMPLE_INTERVAL_MS)
        || object.get("unexpectedUiObserved").and_then(Value::as_bool) != Some(unexpected)
    {
        bail!("SecurityAgent observer report identity or no-UI result changed")
    }
    Ok(())
}

fn terminal_alert_evidence(
    raw_report_sha256: String,
    raw_report: Vec<u8>,
) -> Result<SecurityAgentTerminalAlertV2> {
    let raw_report = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(&raw_report),
        raw_report_sha256,
        raw_report_byte_length: u64::try_from(raw_report.len())
            .context("SecurityAgent terminal report length exceeds u64")?,
    };
    raw_report.validate_terminal_alert()?;
    let arm_evidence = single_securityagent_terminal_alert_evidence_v2(raw_report.clone())?;
    Ok(SecurityAgentTerminalAlertV2 {
        raw_report,
        arm_evidence,
    })
}

fn require_root_owned_fixed_observer() -> Result<()> {
    let path = Path::new(SECURITYAGENT_OBSERVER_PATH_V2);
    let mut current = PathBuf::from("/");
    for component in path.components().skip(1) {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current)
            .with_context(|| format!("inspect observer path component {}", current.display()))?;
        require_immutable_component(&metadata, current == path)?;
    }
    Ok(())
}

fn require_immutable_component(metadata: &Metadata, terminal: bool) -> Result<()> {
    if metadata.file_type().is_symlink() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
        bail!("SecurityAgent observer path is not root-owned immutable no-follow state")
    }
    if terminal {
        if !metadata.file_type().is_file() || metadata.nlink() != 1 {
            bail!("SecurityAgent observer is not one regular-file identity")
        }
    } else if !metadata.is_dir() {
        bail!("SecurityAgent observer parent is not a directory")
    }
    Ok(())
}

fn kill_and_wait(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn monotonic_nanoseconds() -> Result<u64> {
    let mut value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: output is writable and CLOCK_MONOTONIC is process-independent.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut value) } != 0
        || value.tv_sec < 0
        || value.tv_nsec < 0
    {
        return Err(std::io::Error::last_os_error()).context("read monotonic clock");
    }
    let seconds = u64::try_from(value.tv_sec).context("monotonic seconds overflow")?;
    let nanoseconds = u64::try_from(value.tv_nsec).context("monotonic nanos overflow")?;
    seconds
        .checked_mul(1_000_000_000)
        .and_then(|base| base.checked_add(nanoseconds))
        .context("monotonic clock overflow")
}

#[cfg(test)]
mod tests {
    #[test]
    fn observer_surface_hard_exits_on_alert_and_has_one_bounded_window() {
        let source = include_str!("securityagent.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(source.matches("std::process::exit(86)").count() >= 3);
        assert!(source.contains("if !operation.is_finished()"));
        assert!(source.contains("std::process::exit(78)"));
        assert!(source.contains("unexpectedUiObserved"));
        assert!(!production.contains(&["SecurityAgent", ".app"].concat()));
        let root = production
            .split("pub(crate) fn observe_root_operation_with_raw")
            .nth(1)
            .unwrap()
            .split("#[derive(Debug, Clone, Copy)]")
            .next()
            .unwrap();
        assert!(root.contains("observe_supervised_operation"));
        let supervised = production
            .split("fn observe_supervised_operation")
            .nth(1)
            .unwrap()
            .split("#[derive(Debug, Clone, Copy)]")
            .next()
            .unwrap();
        assert!(supervised.contains("signals.recv_timeout"));
        assert!(supervised.contains("persist_terminal_alert(&alert)"));
        assert!(supervised.contains("std::process::exit(86)"));
        let rearmed = production
            .split("pub(crate) fn observe_rearmed_root_operation_with_raw")
            .nth(1)
            .unwrap()
            .split("fn observe_supervised_operation")
            .next()
            .unwrap();
        assert!(rearmed.contains("OBSERVER_REARM_AFTER"));
        assert!(rearmed.contains("thread::spawn(move ||"));
        assert!(rearmed.contains("pending_successor"));
        assert!(rearmed.contains("successor_ready"));
        assert!(rearmed.contains("was_latest && !operation.is_finished()"));
        assert!(rearmed.contains("std::process::exit(78)"));
        let sealed = production
            .split("pub(crate) fn observe_sealed_child_with")
            .nth(1)
            .unwrap()
            .split("pub(crate) fn observe_idle_baseline_sha256")
            .next()
            .unwrap();
        assert!(sealed.contains("scope.spawn"));
        assert!(sealed.contains("after_sender.send(after_spawn(child_pid))"));
        assert!(sealed.contains("signals.try_recv()"));
        assert!(sealed.contains("kill_and_wait(&mut child)"));
        assert!(!sealed.contains("let after_spawn = match after_spawn(child.id())"));
    }
}
