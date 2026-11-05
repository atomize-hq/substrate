//! Attempt-local ownership of real children and their stdio tasks.
use std::{
    future::Future,
    io,
    path::Path,
    process::ExitStatus,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::{Child, ChildStdout, Command},
    sync::{oneshot, Notify},
    time,
};

const WAIT_FAILED: u8 = 1;
const STDIN_FAILED: u8 = 2;
const STDOUT_FAILED: u8 = 4;
const STDERR_FAILED: u8 = 8;
const OWNER_FAILED: u8 = 16;
const KILL_FAILED: u8 = 32;

/// Checked child/stdio settlement failed. Messages contain only failure categories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaudeSettlementError {
    mask: u8,
}
impl std::fmt::Display for ClaudeSettlementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "process settlement failed (categories={:#04x})",
            self.mask
        )
    }
}
impl std::error::Error for ClaudeSettlementError {}

/// Actual child-owner wait data, independent from logical completion. Missing
/// values are unknown, not proof of process absence or successful settlement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessTerminationObservation {
    pub process_id: Option<u32>,
    pub status: Option<ExitStatus>,
    pub recovery_status: Option<ExitStatus>,
}

#[derive(Debug, Default)]
struct State {
    sealed: bool,
    started: bool,
    admission: Option<SpawnAdmission>,
    cancelled: bool,
    active: usize,
    failures: u8,
    terminations: Vec<ProcessTerminationObservation>,
}
#[derive(Debug, Default)]
struct Inner {
    state: Mutex<State>,
    changed: Notify,
    cancellation: Notify,
    #[cfg(test)]
    wait_observer: Mutex<Option<oneshot::Sender<()>>>,
    #[cfg(test)]
    fail_wait: AtomicBool,
    #[cfg(test)]
    fail_stderr: AtomicBool,
    #[cfg(test)]
    stdout_ack_gate: Mutex<Option<oneshot::Receiver<()>>>,
    #[cfg(test)]
    stdout_ack_observer: Mutex<Option<oneshot::Sender<()>>>,
}

// Opaque callbacks never expose captured authority/credentials through Debug.
#[derive(Clone)]
struct SpawnAdmission(Arc<AdmissionCallback>);
type AdmissionFuture = std::pin::Pin<Box<dyn Future<Output = io::Result<()>> + Send>>;
type AdmissionCallback = dyn Fn() -> AdmissionFuture + Send + Sync;
impl std::fmt::Debug for SpawnAdmission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SpawnAdmission")
    }
}

/// Retained ownership of the children created by one Claude attempt.
///
/// Construct before polling startup, then retain through cancellation/errors. `settle` seals
/// further child admission and acknowledges checked child waits and all stdio owners. It does
/// not assert termination of descendants or consumer delivery of buffered events.
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let custody = claude_code::ClaudeProcessCustody::new();
/// custody.request_termination();
/// custody.settle().await?;
/// # Ok(()) }
/// ```
#[derive(Clone, Debug, Default)]
pub struct ClaudeProcessCustody {
    inner: Arc<Inner>,
}
impl ClaudeProcessCustody {
    /// Creates an open, empty attempt. No process is launched.
    pub fn new() -> Self {
        Self::default()
    }
    /// Copy known actual wait results without sealing, cancelling, waiting or
    /// launching. Check `settle()` separately before treating this as the settled
    /// attempt's snapshot. A recovery wait does not replace a failed primary wait.
    /// ```no_run
    /// # async fn inspect(custody: &claude_code::ClaudeProcessCustody) -> Result<(), Box<dyn std::error::Error>> {
    /// custody.settle().await?;
    /// let observations = custody.observed_process_terminations();
    /// # let _ = observations; Ok(()) }
    /// ```
    pub fn observed_process_terminations(&self) -> Vec<ProcessTerminationObservation> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .terminations
            .clone()
    }

    /// Observe admission closed before any spawn attempt began. Open empty
    /// custody and failed/abandoned attempts return false. This does not seal,
    /// cancel or wait; check `settle()` and enclosing cleanup separately.
    /// ```
    /// let custody = claude_code::ClaudeProcessCustody::new();
    /// assert!(!custody.observed_closed_before_any_spawn());
    /// custody.request_termination();
    /// assert!(custody.observed_closed_before_any_spawn());
    /// ```
    pub fn observed_closed_before_any_spawn(&self) -> bool {
        let state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.sealed
            && !state.started
            && state.active == 0
            && state.failures == 0
            && state.terminations.is_empty()
    }

    /// Creates custody with a fallible check before every actual spawn, including
    /// capability/help preflights and executable-busy retries. The callback runs
    /// outside the custody lock. Cancellation/sealing is checked again under the
    /// lock immediately before spawn and registration; this is not an atomic lock
    /// across external policy changes.
    ///
    /// ```no_run
    /// let custody = claude_code::ClaudeProcessCustody::with_spawn_admission(|| async { Ok(()) });
    /// # drop(custody);
    /// ```
    pub fn with_spawn_admission<F, Fut>(admission: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = io::Result<()>> + Send + 'static,
    {
        let custody = Self::new();
        custody
            .inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .admission = Some(SpawnAdmission(Arc::new(move || Box::pin(admission()))));
        custody
    }

    /// Installs one check before any spawn. Replacement or installation after
    /// startup/sealing is refused, including through another custody clone.
    ///
    /// ```no_run
    /// let custody = claude_code::ClaudeProcessCustody::new();
    /// custody.install_spawn_admission(|| async { Ok(()) })?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn install_spawn_admission<F, Fut>(&self, admission: F) -> io::Result<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = io::Result<()>> + Send + 'static,
    {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.started || state.sealed || state.cancelled || state.admission.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "process admission installation refused",
            ));
        }
        state.admission = Some(SpawnAdmission(Arc::new(move || Box::pin(admission()))));
        Ok(())
    }

    pub(crate) async fn spawn_admitted(
        &self,
        command: &mut Command,
        binary: &Path,
    ) -> Result<(Child, ChildOwner), crate::ClaudeCodeError> {
        let mut abandoned = self.cancel_on_drop();
        let mut backoff = Duration::from_millis(2);
        for attempt in 0..5 {
            self.ensure_open()?;
            let admission = {
                let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
                if state.sealed || state.cancelled {
                    return Err(crate::ClaudeCodeError::StdoutRead(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "process attempt sealed",
                    )));
                }
                // Freeze callback installation before leaving the lock, even when no
                // callback is configured. A concurrent installer cannot be skipped.
                state.started = true;
                state.admission.clone()
            };
            if let Some(admission) = admission {
                let result = tokio::select! {
                    biased;
                    _ = self.cancelled() => Err(io::Error::new(io::ErrorKind::Interrupted, "process attempt sealed")),
                    result = (admission.0)() => result,
                };
                if result.is_err() {
                    self.request_termination();
                    // Callback error text may contain authority/account material.
                    return Err(crate::ClaudeCodeError::StdoutRead(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "process spawn admission refused",
                    )));
                }
            }
            match self.spawn_once(command, binary) {
                Ok(child) => {
                    abandoned.disarm();
                    return Ok(child);
                }
                Err(crate::ClaudeCodeError::Spawn { source, .. })
                    if (source.kind() == io::ErrorKind::ExecutableFileBusy
                        || source.raw_os_error() == Some(26))
                        && attempt < 4 =>
                {
                    // Release the lock between retries; every retry obtains fresh admission.
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, Duration::from_millis(50));
                }
                Err(error) => return Err(error),
            }
        }
        unreachable!("bounded spawn loop returns on its final attempt")
    }

    fn spawn_once(
        &self,
        command: &mut Command,
        binary: &Path,
    ) -> Result<(Child, ChildOwner), crate::ClaudeCodeError> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.sealed || state.cancelled {
            return Err(crate::ClaudeCodeError::StdoutRead(io::Error::new(
                io::ErrorKind::Interrupted,
                "process attempt sealed",
            )));
        }
        // Register actual child ownership before the syscall, without any intervening await.
        state.started = true;
        state.active += 1;
        let mut owner = ChildOwner {
            custody: self.clone(),
            acknowledged: false,
            process_id: None,
            status: None,
            recovery_status: None,
        };
        let child = command
            .spawn()
            .map_err(|source| crate::ClaudeCodeError::Spawn {
                binary: binary.to_path_buf(),
                source,
            });
        drop(state);
        match child {
            Ok(child) => {
                owner.process_id = child.id();
                Ok((child, owner))
            }
            Err(error) => {
                owner.finish(0);
                Err(error)
            }
        }
    }

    /// Seals the attempt and requests termination of each admitted child. This is only a signal.
    pub fn request_termination(&self) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state.sealed = true;
        state.cancelled = true;
        drop(state);
        self.inner.cancellation.notify_waiters();
    }
    /// Seals child admission and waits for every real child/stdio owner to acknowledge.
    /// A failed or abandoned owner is an error, including after cancellation.
    pub async fn settle(&self) -> Result<(), ClaudeSettlementError> {
        loop {
            let changed = self.inner.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            {
                let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
                state.sealed = true;
                if state.active == 0 {
                    return if state.failures == 0 {
                        Ok(())
                    } else {
                        Err(ClaudeSettlementError {
                            mask: state.failures,
                        })
                    };
                }
            }
            changed.await;
        }
    }
    pub(crate) async fn cancelled(&self) {
        let notified = self.inner.cancellation.notified();
        tokio::pin!(notified);
        // Register before checking the flag: cancellation during admission cannot
        // be lost between the state read and this await.
        notified.as_mut().enable();
        if self
            .inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .cancelled
        {
            return;
        }
        notified.await;
    }
    pub(crate) fn ensure_open(&self) -> Result<(), crate::ClaudeCodeError> {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.sealed || state.cancelled {
            Err(crate::ClaudeCodeError::StdoutRead(io::Error::new(
                io::ErrorKind::Interrupted,
                "process attempt sealed",
            )))
        } else {
            Ok(())
        }
    }
    #[cfg(test)]
    pub(crate) fn spawn(
        &self,
        command: &mut Command,
        binary: &Path,
    ) -> Result<(Child, ChildOwner), crate::ClaudeCodeError> {
        // Admission, registration, spawn and cancellation/sealing are serialized under this lock.
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.sealed || state.cancelled || state.admission.is_some() {
            return Err(crate::ClaudeCodeError::StdoutRead(io::Error::new(
                io::ErrorKind::Interrupted,
                "process attempt sealed",
            )));
        }
        state.started = true;
        state.active += 1;
        let mut owner = ChildOwner {
            custody: self.clone(),
            acknowledged: false,
            process_id: None,
            status: None,
            recovery_status: None,
        };
        let child = crate::process::spawn_with_retry(command, binary);
        drop(state);
        match child {
            Ok(child) => {
                owner.process_id = child.id();
                Ok((child, owner))
            }
            Err(error) => {
                owner.finish(0);
                Err(error)
            }
        }
    }
    pub(crate) fn cancel_on_drop(&self) -> CancelOnDrop {
        CancelOnDrop(Some(self.clone()))
    }
}

pub(crate) struct CancelOnDrop(Option<ClaudeProcessCustody>);
impl CancelOnDrop {
    pub(crate) fn disarm(&mut self) {
        self.0 = None;
    }
}
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(custody) = &self.0 {
            custody.request_termination();
        }
    }
}

pub(crate) struct ChildOwner {
    custody: ClaudeProcessCustody,
    acknowledged: bool,
    process_id: Option<u32>,
    status: Option<ExitStatus>,
    recovery_status: Option<ExitStatus>,
}
impl ChildOwner {
    fn finish(mut self, failures: u8) {
        self.complete(failures);
    }
    fn finish_with_observation(
        mut self,
        failures: u8,
        status: Option<ExitStatus>,
        recovery_status: Option<ExitStatus>,
    ) {
        self.status = status;
        self.recovery_status = recovery_status;
        self.complete(failures);
    }
    fn complete(&mut self, failures: u8) {
        let mut state = self
            .custody
            .inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.terminations.push(ProcessTerminationObservation {
            process_id: self.process_id,
            status: self.status,
            recovery_status: self.recovery_status,
        });
        state.failures |= failures;
        if failures != 0 {
            // A failed owner cannot authorize fallback probes or other later child effects.
            state.sealed = true;
            state.cancelled = true;
        }
        state.active -= 1;
        self.acknowledged = true;
        drop(state);
        if failures != 0 {
            self.custody.inner.cancellation.notify_waiters();
        }
        self.custody.inner.changed.notify_waiters();
    }
}
impl Drop for ChildOwner {
    fn drop(&mut self) {
        if !self.acknowledged {
            self.complete(OWNER_FAILED);
        }
    }
}

#[derive(Default)]
struct Stop {
    stopped: AtomicBool,
    notify: Notify,
}
impl Stop {
    fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }
    async fn requested(&self) {
        let notified = self.notify.notified();
        if self.stopped.load(Ordering::SeqCst) {
            return;
        }
        notified.await;
    }
}

pub(crate) struct ChildOutcome<T> {
    pub(crate) status: io::Result<ExitStatus>,
    pub(crate) stdout: io::Result<Option<T>>,
    pub(crate) stderr: io::Result<Option<Vec<u8>>>,
    pub(crate) stdin: io::Result<()>,
    pub(crate) timed_out: bool,
    pub(crate) settlement_failed: bool,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum StderrMode {
    Absent,
    Mirror,
    Capture { mirror: bool },
}
impl StderrMode {
    fn requires_pipe(self) -> bool {
        !matches!(self, Self::Absent)
    }
    fn mirrors(self) -> bool {
        matches!(self, Self::Mirror | Self::Capture { mirror: true })
    }
    fn captures(self) -> bool {
        matches!(self, Self::Capture { .. })
    }
}
async fn read_stderr(reader: tokio::process::ChildStderr, mode: StderrMode) -> io::Result<Vec<u8>> {
    read_stderr_to(reader, tokio::io::stderr(), mode).await
}
async fn read_stderr_to<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    mut reader: R,
    mut console: W,
    mode: StderrMode,
) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 4096];
    loop {
        let count = reader.read(&mut chunk).await?;
        if count == 0 {
            return Ok(bytes);
        }
        if mode.mirrors() {
            console.write_all(&chunk[..count]).await?;
            console.flush().await?;
        }
        if mode.captures() {
            bytes.extend_from_slice(&chunk[..count]);
        }
    }
}
fn join_error(error: tokio::task::JoinError) -> io::Error {
    io::Error::other(error)
}
fn copy_error(error: &io::Error) -> io::Error {
    io::Error::new(error.kind(), error.to_string())
}

/// The spawned caller transfers Child (including all pipe handles) and owner here without await.
/// This task is never aborted when a public future is dropped.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn drive_child<T, F, Fut>(
    mut child: Child,
    owner: ChildOwner,
    input: Option<Vec<u8>>,
    ignore_broken_pipe: bool,
    timeout: Option<Duration>,
    stderr_mode: StderrMode,
    ready: oneshot::Sender<io::Result<()>>,
    stdout_reader: F,
) -> ChildOutcome<T>
where
    T: Send + 'static,
    F: FnOnce(ChildStdout) -> Fut + Send + 'static,
    Fut: Future<Output = io::Result<T>> + Send + 'static,
{
    let requires_stderr = stderr_mode.requires_pipe();
    let stop = Arc::new(Stop::default());
    let custody = owner.custody.clone();
    let setup_failures = (if child.stdout.is_none() {
        STDOUT_FAILED
    } else {
        0
    }) | (if requires_stderr && child.stderr.is_none() {
        STDERR_FAILED
    } else {
        0
    }) | (if input.is_some() && child.stdin.is_none() {
        STDIN_FAILED
    } else {
        0
    });
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let mut stdin_task = Some(tokio::spawn({
        let stop = Arc::clone(&stop);
        async move {
            let write = async move {
                if let Some(input) = input {
                    let mut stdin = stdin.ok_or_else(|| io::Error::other("missing child stdin"))?;
                    let result = async {
                        stdin.write_all(&input).await?;
                        stdin.shutdown().await
                    }
                    .await;
                    match result {
                        Err(e) if ignore_broken_pipe && e.kind() == io::ErrorKind::BrokenPipe => {
                            Ok(())
                        }
                        other => other,
                    }
                } else {
                    drop(stdin);
                    Ok(())
                }
            };
            tokio::select! { biased; _ = stop.requested() => Ok(false), result = write => result.map(|()| true) }
        }
    }));
    let mut stdout_task = Some(tokio::spawn({
        let stop = Arc::clone(&stop);
        #[cfg(test)]
        let observation = custody.clone();
        async move {
            let stdout = stdout.ok_or_else(|| io::Error::other("missing child stdout"))?;
            let result = tokio::select! { biased; _ = stop.requested() => Ok(None), result = stdout_reader(stdout) => result.map(Some) };
            #[cfg(test)]
            {
                let gate = observation.inner.stdout_ack_gate.lock().unwrap().take();
                if let Some(gate) = gate {
                    if let Some(tx) = observation.inner.stdout_ack_observer.lock().unwrap().take() {
                        let _ = tx.send(());
                    }
                    let _ = gate.await;
                }
            }
            result
        }
    }));
    let mut stderr_task = Some(tokio::spawn({
        let stop = Arc::clone(&stop);
        async move {
            match stderr {
                Some(stderr) => {
                    tokio::select! { biased; _ = stop.requested() => Ok(None), result = read_stderr(stderr, stderr_mode) => result.map(Some) }
                }
                None if requires_stderr => Err(io::Error::other("missing child stderr")),
                None => Ok(None),
            }
        }
    }));
    let mut ready = Some(ready);
    let mut input_outcome = None;
    let mut stdout_outcome = None;
    let mut stderr_outcome = None;
    let mut status = None;
    let mut recovery_status = None;
    let mut failures = setup_failures;
    let mut stopping = setup_failures != 0;
    if stopping {
        stop.stop();
    }
    let mut timed_out = false;
    let deadline = timeout.map(|duration| time::Instant::now() + duration);
    while status.is_none() || stdin_task.is_some() || stdout_task.is_some() || stderr_task.is_some()
    {
        tokio::select! {
            biased;
            _ = custody.cancelled(), if !stopping => { stopping = true; stop.stop(); }
            _ = async { match deadline { Some(d) => time::sleep_until(d).await, None => std::future::pending().await } }, if !stopping => {
                // Preserve real process status separately from the run's pipe-disposal deadline.
                if status.is_none() {
                    match child.try_wait() {
                        Ok(Some(exit)) => {
                            #[cfg(test)]
                            if let Some(tx) = custody.inner.wait_observer.lock().unwrap().take() { let _ = tx.send(()); }
                            status = Some(Ok(exit));
                        }
                        Ok(None) => { timed_out = true; }
                        Err(_) => { failures |= WAIT_FAILED; }
                    }
                }
                // Expiration still disposes and joins every outstanding stdio owner, including
                // when the process has exited. No disposed reader is claimed to have reached EOF.
                // Claude preserves an already-exited status, as its original wait helper did.
                stopping = true; stop.stop();
            }
            result = async { stdin_task.as_mut().expect("guarded stdin task").await }, if stdin_task.is_some() => {
                stdin_task = None;
                let result = result.map_err(join_error).and_then(|result| result);
                let result = match result {
                    Err(error) => { failures |= STDIN_FAILED; stopping = true; stop.stop(); Err(error) }
                    Ok(false) => Err(io::Error::new(io::ErrorKind::Interrupted, "stdin disposed during cleanup")),
                    Ok(true) => {
                        // A later stop cannot erase successful checked setup. Readiness is not
                        // terminal completion: actual wait and all pipe ACKs remain required.
                        if setup_failures == 0 { if let Some(tx) = ready.take() { let _ = tx.send(Ok(())); } }
                        Ok(())
                    }
                };
                input_outcome = Some(result);
            }
            result = async { stdout_task.as_mut().expect("guarded stdout task").await }, if stdout_task.is_some() => {
                stdout_task = None;
                let result = result.map_err(join_error).and_then(|result| result);
                if result.is_err() { failures |= STDOUT_FAILED; stopping = true; stop.stop(); }
                stdout_outcome = Some(result);
            }
            result = async { stderr_task.as_mut().expect("guarded stderr task").await }, if stderr_task.is_some() => {
                stderr_task = None;
                let result = result.map_err(join_error).and_then(|result| result);
                #[cfg(test)]
                let result = if custody.inner.fail_stderr.swap(false, Ordering::SeqCst) {
                    Err(io::Error::other("injected stderr failure"))
                } else { result };
                if result.is_err() { failures |= STDERR_FAILED; stopping = true; stop.stop(); }
                stderr_outcome = Some(result);
            }
            result = child.wait(), if status.is_none() => {
                #[cfg(test)]
                let result = if custody.inner.fail_wait.swap(false, Ordering::SeqCst) {
                    Err(io::Error::other("injected wait failure after real wait"))
                } else { result };
                let result = match result {
                    Ok(exit) => Ok(exit),
                    Err(error) => {
                        failures |= WAIT_FAILED; stopping = true; stop.stop();
                        if child.start_kill().is_err() { failures |= KILL_FAILED; }
                        // Keep the original failed wait visible even if this cleanup wait succeeds.
                        match child.wait().await {
                            Ok(exit) => recovery_status = Some(exit),
                            Err(_) => failures |= WAIT_FAILED,
                        }
                        Err(error)
                    }
                };
                #[cfg(test)]
                if result.is_ok() {
                    if let Some(tx) = custody.inner.wait_observer.lock().unwrap().take() { let _ = tx.send(()); }
                }
                status = Some(result);
            }
        }
        if stopping && status.is_none() {
            if let Err(error) = child.start_kill() {
                // A child can exit between the selected branch and the kill request.
                match child.try_wait() {
                    Ok(Some(exit)) => status = Some(Ok(exit)),
                    Err(_) => {
                        let _ = error;
                        failures |= WAIT_FAILED | KILL_FAILED;
                    }
                    Ok(None) => {
                        let _ = error;
                        failures |= KILL_FAILED;
                    }
                }
            }
        }
    }
    let stdin = input_outcome.expect("stdin owner joined");
    let status = status.expect("child wait attempted");
    let stdout = stdout_outcome.expect("stdout owner joined");
    let stderr = stderr_outcome.expect("stderr owner joined");
    // All awaitable owners have been checked before releasing this registration or any error.
    owner.finish_with_observation(failures, status.as_ref().ok().copied(), recovery_status);
    if let Some(tx) = ready {
        let error = match &stdin {
            Err(error) => copy_error(error),
            Ok(()) => io::Error::new(
                io::ErrorKind::Interrupted,
                "process setup cancelled or failed",
            ),
        };
        let _ = tx.send(Err(error));
    }
    ChildOutcome {
        status,
        stdout,
        stderr,
        stdin,
        timed_out,
        settlement_failed: failures != 0,
    }
}

pub(crate) struct OwnedCompletion<T> {
    pub(crate) receiver: oneshot::Receiver<T>,
    pub(crate) cancel: CancelOnDrop,
}
impl<T> Unpin for OwnedCompletion<T> {}
impl<T> Future for OwnedCompletion<T> {
    type Output = Result<T, oneshot::error::RecvError>;
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let poll = std::pin::Pin::new(&mut self.receiver).poll(cx);
        if poll.is_ready() {
            self.cancel.disarm();
        }
        poll
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_os = "linux"))]
mod spawn_admission_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn test_actual_preflight_uses_admission_and_denial_prevents_fallback_spawns() {
        let temp = tempfile::tempdir().unwrap();
        let binary = temp.path().join("preflight-executable");
        let marker = temp.path().join("spawned");
        std::fs::write(
            &binary,
            format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
        )
        .unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let checked = calls.clone();
        let custody = ClaudeProcessCustody::with_spawn_admission(move || {
            checked.fetch_add(1, Ordering::SeqCst);
            async { Err(io::Error::new(io::ErrorKind::PermissionDenied, "revoked")) }
        });
        let client = crate::ClaudeClient::builder()
            .binary(&binary)
            .working_dir(temp.path())
            .build();
        assert!(client.help_with_custody(&custody).await.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!marker.exists());
        assert!(custody.ensure_open().is_err());
        custody.settle().await.unwrap();
    }

    #[tokio::test]
    async fn test_denial_seals_all_clones_before_actual_spawn() {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("spawned");
        let custody = ClaudeProcessCustody::with_spawn_admission(|| async {
            Err(io::Error::other("private credential canary"))
        });
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg("touch \"$1\"").arg("sh").arg(&marker);
        let error = custody
            .spawn_admitted(&mut command, Path::new("/bin/sh"))
            .await
            .err()
            .unwrap();
        assert!(!error.to_string().contains("private credential canary"));
        assert!(!marker.exists());
        assert!(custody.clone().ensure_open().is_err());
        custody.settle().await.unwrap();
    }

    #[tokio::test]
    async fn test_cancellation_during_admission_prevents_spawn_after_ack() {
        let custody = ClaudeProcessCustody::new();
        let (entered, observed) = oneshot::channel();
        let entered = Arc::new(Mutex::new(Some(entered)));
        custody
            .install_spawn_admission(move || {
                let entered = entered.clone();
                async move {
                    if let Some(entered) = entered.lock().unwrap().take() {
                        let _ = entered.send(());
                    }
                    std::future::pending::<io::Result<()>>().await
                }
            })
            .unwrap();
        let spawning = custody.clone();
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("spawned");
        let path = marker.clone();
        let task = tokio::spawn(async move {
            let mut command = Command::new("/bin/sh");
            command.arg("-c").arg("touch \"$1\"").arg("sh").arg(path);
            spawning
                .spawn_admitted(&mut command, Path::new("/bin/sh"))
                .await
                .err()
                .unwrap()
        });
        observed.await.unwrap();
        custody.request_termination();
        custody.settle().await.unwrap();
        assert!(task.await.is_ok());
        assert!(!marker.exists());
    }

    #[tokio::test]
    async fn test_executable_busy_retry_requires_new_admission() {
        let temp = tempfile::tempdir().unwrap();
        let binary = temp.path().join("busy-executable");
        let marker = temp.path().join("spawned");
        std::fs::write(
            &binary,
            format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
        )
        .unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        // Linux refuses exec while this real executable has a writer, with ETXTBSY.
        let _writer = std::fs::OpenOptions::new()
            .write(true)
            .open(&binary)
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let custody = ClaudeProcessCustody::with_spawn_admission(move || {
            let attempt = count.fetch_add(1, Ordering::SeqCst);
            async move {
                if attempt == 0 {
                    Ok(())
                } else {
                    Err(io::Error::other("revoked"))
                }
            }
        });
        let mut command = Command::new(&binary);
        assert!(custody.spawn_admitted(&mut command, &binary).await.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert!(!marker.exists());
        assert!(custody.ensure_open().is_err());
        custody.settle().await.unwrap();
    }
}
