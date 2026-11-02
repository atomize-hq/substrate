//! Actual MCP/app-server child and stdio custody, independent of protocol replies.
use std::{
    io,
    sync::{Arc, Mutex},
};
use tokio::{
    process::{Child, Command},
    sync::{watch, Notify},
    task::JoinHandle,
};

const WAIT_FAILED: u8 = 1;
const STDIN_FAILED: u8 = 2;
const STDOUT_FAILED: u8 = 4;
const STDERR_FAILED: u8 = 8;
const OWNER_FAILED: u8 = 16;
const KILL_FAILED: u8 = 32;
pub(super) const SETUP_FAILED: u8 = 64;

/// Checked wait or stdio settlement failed. No backend output is included.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpSettlementError {
    failures: u8,
}
impl std::fmt::Display for McpSettlementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MCP process settlement failed (categories={:#04x})",
            self.failures
        )
    }
}
impl std::error::Error for McpSettlementError {}

#[derive(Debug, Default)]
struct State {
    sealed: bool,
    terminate: bool,
    active: usize,
    failures: u8,
}
#[derive(Debug, Default)]
struct Inner {
    state: Mutex<State>,
    changed: Notify,
    termination: Notify,
    #[cfg(test)]
    wait_gate: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    #[cfg(test)]
    wait_observed: Notify,
    #[cfg(test)]
    pipe_gate: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    #[cfg(test)]
    pipes_observed: Notify,
    #[cfg(test)]
    inject_failures: std::sync::atomic::AtomicU8,
    #[cfg(test)]
    before_wait_gate: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    #[cfg(test)]
    before_wait_observed: Notify,
    #[cfg(test)]
    fail_before_wait: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    cleanup_wait_gate: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    #[cfg(test)]
    cleanup_wait_observed: Notify,
    #[cfg(test)]
    cleanup_wait_completed: Notify,
    #[cfg(test)]
    cleanup_wait_succeeded: std::sync::atomic::AtomicBool,
}

/// Attempt-local custody of the actual MCP/app-server child and every stdio owner.
///
/// Construct before startup. Termination is a signal; `settle` seals further
/// child admission and waits for checked child/pipe results, including failures.
/// It does not acknowledge protocol completion, descendants, or event delivery.
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let custody = codex::mcp::McpProcessCustody::new();
/// custody.request_termination();
/// custody.settle().await?;
/// # Ok(()) }
/// ```
#[derive(Clone, Debug, Default)]
pub struct McpProcessCustody {
    inner: Arc<Inner>,
}
impl McpProcessCustody {
    /// Create an open attempt without launching a process.
    pub fn new() -> Self {
        Self::default()
    }
    /// Seal child admission and request termination. This is not acknowledgment.
    pub fn request_termination(&self) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state.sealed = true;
        state.terminate = true;
        drop(state);
        self.inner.termination.notify_waiters();
    }
    /// Seal admission and observe all actual child waits and pipe owners.
    /// Call after startup has ended, or request termination to cancel startup.
    pub async fn settle(&self) -> Result<(), McpSettlementError> {
        loop {
            let changed = self.inner.changed.notified();
            {
                let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
                state.sealed = true;
                if state.active == 0 {
                    return if state.failures == 0 {
                        Ok(())
                    } else {
                        Err(McpSettlementError {
                            failures: state.failures,
                        })
                    };
                }
            }
            changed.await;
        }
    }
    async fn termination_requested(&self) {
        loop {
            let notified = self.inner.termination.notified();
            if self
                .inner
                .state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .terminate
            {
                return;
            }
            notified.await;
        }
    }
    pub(super) fn spawn(&self, command: &mut Command) -> io::Result<(Child, ChildOwner)> {
        // Registration, sealing and actual spawn are serialized. Every successful
        // child is returned with its already registered owner, with no await.
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.sealed {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "MCP attempt sealed",
            ));
        }
        state.active += 1;
        let owner = ChildOwner {
            custody: self.clone(),
            finished: false,
        };
        let result = command.spawn();
        drop(state);
        match result {
            Ok(child) => Ok((child, owner)),
            Err(error) => {
                owner.finish(0);
                Err(error)
            }
        }
    }
}

pub(super) struct ChildOwner {
    custody: McpProcessCustody,
    finished: bool,
}
impl ChildOwner {
    fn finish(mut self, failures: u8) {
        self.finished = true;
        self.record(failures);
    }
    fn record(&self, failures: u8) {
        let mut state = self
            .custody
            .inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.active -= 1;
        state.failures |= failures;
        drop(state);
        self.custody.inner.changed.notify_waiters();
    }
}
impl Drop for ChildOwner {
    fn drop(&mut self) {
        if !self.finished {
            self.record(OWNER_FAILED);
        }
    }
}

pub(super) type PipeTask = JoinHandle<io::Result<()>>;

/// Transfer Child and all pipe JoinHandles synchronously, before any post-spawn await.
/// The owner task is deliberately not exposed to public cancellation or Drop.
pub(super) fn supervise(
    mut child: Child,
    owner: ChildOwner,
    tasks: Vec<(u8, PipeTask)>,
    stop: watch::Sender<bool>,
    writer_stop: watch::Sender<bool>,
    setup_failures: u8,
) {
    tokio::spawn(async move {
        let custody = owner.custody.clone();
        let mut failures = setup_failures;
        if setup_failures != 0 {
            custody.request_termination();
        }
        #[cfg(test)]
        {
            // Hold before even polling the first wait, while real protocol I/O
            // can establish that the synthetic app-server is running.
            let gate = custody.inner.before_wait_gate.lock().unwrap().take();
            custody.inner.before_wait_observed.notify_one();
            if let Some(gate) = gate {
                let _ = gate.await;
            }
        }
        let child_pid = child.id();
        let mut termination_branch = "natural_wait";
        let mut recovery_wait = None;
        let status = tokio::select! {
            biased;
            result = async {
                #[cfg(test)]
                if custody.inner.fail_before_wait.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    return Err(io::Error::other("injected failure before actual MCP wait"));
                }
                child.wait().await
            } => result,
            _ = custody.termination_requested() => {
                termination_branch = "termination_requested";
                // Stop pipe I/O intentionally, then join the owners below. Do not
                // abort their tasks or infer acknowledgment from process absence.
                stop.send_replace(true);
                writer_stop.send_replace(true);
                if child.start_kill().is_err() { failures |= KILL_FAILED; }
                child.wait().await
            }
        };
        let status = match status {
            Ok(exit) => Ok(exit),
            Err(error) => {
                // A failed wait leaves terminal state unconfirmed. Seal this
                // attempt and retain Child plus registration through a checked
                // kill/recovery wait and all pipe joins, as exec custody does.
                failures |= WAIT_FAILED;
                custody.request_termination();
                stop.send_replace(true);
                writer_stop.send_replace(true);
                if child.start_kill().is_err() {
                    failures |= KILL_FAILED;
                }
                #[cfg(test)]
                {
                    let gate = custody.inner.cleanup_wait_gate.lock().unwrap().take();
                    custody.inner.cleanup_wait_observed.notify_one();
                    if let Some(gate) = gate {
                        let _ = gate.await;
                    }
                }
                let cleanup_wait = child.wait().await;
                recovery_wait = Some((cleanup_wait.is_ok(), cleanup_wait.as_ref().ok().copied()));
                #[cfg(test)]
                {
                    custody
                        .inner
                        .cleanup_wait_succeeded
                        .store(cleanup_wait.is_ok(), std::sync::atomic::Ordering::SeqCst);
                    custody.inner.cleanup_wait_completed.notify_one();
                }
                if cleanup_wait.is_err() {
                    // As with exec custody, a second wait error is failed
                    // settlement, never proof of terminal completion.
                    failures |= WAIT_FAILED;
                }
                // Successful cleanup cannot erase the original failed wait.
                Err(error)
            }
        };
        #[cfg(test)]
        let status = if custody
            .inner
            .inject_failures
            .load(std::sync::atomic::Ordering::SeqCst)
            & WAIT_FAILED
            != 0
        {
            Err(io::Error::other(
                "injected MCP wait result failure after real wait",
            ))
        } else {
            status
        };
        if status.is_err() {
            failures |= WAIT_FAILED;
        }
        // Natural exit must still drain buffered stdout/stderr responses. Only
        // stdin is closed immediately. Forced disposal already stopped all I/O.
        writer_stop.send_replace(true);
        if status.is_err() {
            stop.send_replace(true);
        }
        #[cfg(test)]
        {
            let gate = custody.inner.wait_gate.lock().unwrap().take();
            custody.inner.wait_observed.notify_one();
            if let Some(gate) = gate {
                let _ = gate.await;
            }
        }
        let drain_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        for (category, mut task) in tasks {
            let result = tokio::select! {
                result = &mut task => result,
                _ = tokio::time::sleep_until(drain_deadline) => {
                    // A descendant may retain a pipe. Dispose that owned reader
                    // intentionally, then await its acknowledgment without abort.
                    stop.send_replace(true);
                    task.await
                }
            };
            match result {
                Ok(Ok(())) => {}
                Ok(Err(_)) => {
                    failures |= category;
                }
                Err(_) => {
                    failures |= category | OWNER_FAILED;
                }
            }
        }
        tracing::debug!(target: "codex::mcp::safe_lifecycle", child_pid, termination_branch,
            wait_checked = status.is_ok(), wait_status = ?status.as_ref().ok(),
            wait_error_kind = ?status.as_ref().err().map(io::Error::kind),
            wait_errno = status.as_ref().err().and_then(io::Error::raw_os_error),
            recovery_wait = ?recovery_wait,
            pipe_failures = failures & (STDIN_FAILED | STDOUT_FAILED | STDERR_FAILED | OWNER_FAILED),
            settlement_failures = failures, "owned app-server child and pipe joins settled");
        owner.finish(failures);
    });
}

struct PipePanicGuard(Option<McpProcessCustody>);
impl Drop for PipePanicGuard {
    fn drop(&mut self) {
        if let Some(custody) = self.0.take() {
            custody.request_termination();
        }
    }
}

pub(super) fn pipe_task(
    category: u8,
    work: impl std::future::Future<Output = io::Result<()>> + Send + 'static,
    custody: McpProcessCustody,
    end_transport: bool,
) -> (u8, PipeTask) {
    (
        category,
        tokio::spawn(async move {
            let mut guard = PipePanicGuard(Some(custody.clone()));
            let result = work.await;
            drop(guard.0.take());
            if result.is_err() || end_transport {
                custody.request_termination();
            }
            #[cfg(test)]
            {
                if category == STDOUT_FAILED {
                    let gate = custody.inner.pipe_gate.lock().unwrap().take();
                    custody.inner.pipes_observed.notify_one();
                    if let Some(gate) = gate {
                        let _ = gate.await;
                    }
                }
                if custody
                    .inner
                    .inject_failures
                    .load(std::sync::atomic::Ordering::SeqCst)
                    & category
                    != 0
                {
                    return Err(io::Error::other("injected MCP stdio failure"));
                }
            }
            result
        }),
    )
}

pub(super) fn stdin_category() -> u8 {
    STDIN_FAILED
}
pub(super) fn stdout_category() -> u8 {
    STDOUT_FAILED
}
pub(super) fn stderr_category() -> u8 {
    STDERR_FAILED
}

pub(super) async fn stopped(stop: &mut watch::Receiver<bool>) {
    loop {
        if *stop.borrow_and_update() {
            return;
        }
        if stop.changed().await.is_err() {
            return;
        }
    }
}

#[cfg(all(test, unix))]
mod tests;
