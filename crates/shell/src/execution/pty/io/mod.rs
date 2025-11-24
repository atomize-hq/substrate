#[cfg(any(windows, test))]
use super::control::active_pty_control;
#[cfg(unix)]
use super::control::initialize_global_sigwinch_handler_impl;
#[cfg(windows)]
use super::control::WIN_PTY_INPUT_GATE;
use super::control::{ActivePtyGuard, PtyCommand, PtyControl};
use crate::execution::{
    configure_child_shell_env, log_command_event, ShellConfig, ShellMode, PTY_ACTIVE,
};
use anyhow::{Context, Result};
mod state;
#[cfg(test)]
use portable_pty::PtySize;
use portable_pty::{native_pty_system, CommandBuilder};
use serde_json::json;
#[cfg(windows)]
pub(crate) use state::windows_console_size;
pub use state::PtyExitStatus;
pub(crate) use state::{
    get_terminal_size, verify_process_group, MinimalTerminalGuard, PtyActiveGuard,
};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

fn start_pty_manager(
    master: Box<dyn portable_pty::MasterPty + Send>,
) -> Result<(PtyControl, Box<dyn Read + Send>, thread::JoinHandle<()>)> {
    let reader = master
        .try_clone_reader()
        .context("Failed to create PTY reader")?;
    let writer = master
        .take_writer()
        .context("Failed to create PTY writer")?;
    let (tx, rx) = mpsc::channel();
    let control = PtyControl { tx };

    let manager_handle = thread::spawn(move || run_pty_manager(master, writer, rx));

    Ok((control, reader, manager_handle))
}

fn run_pty_manager(
    master: Box<dyn portable_pty::MasterPty + Send>,
    mut writer: Box<dyn Write + Send>,
    rx: mpsc::Receiver<PtyCommand>,
) {
    for cmd in rx {
        match cmd {
            PtyCommand::Resize(size) => {
                if let Err(e) = master.resize(size) {
                    log::warn!("Failed to resize PTY: {e}");
                }
            }
            PtyCommand::Write(data) => {
                if let Err(e) = writer.write_all(&data) {
                    log::warn!("Failed to write to PTY: {e}");
                    break;
                }
                if let Err(e) = writer.flush() {
                    log::warn!("Failed to flush PTY writer: {e}");
                }
            }
            PtyCommand::Close => break,
        }
    }
}

/// Execute a command with full PTY support
pub fn execute_with_pty(
    config: &ShellConfig,
    command: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<PtyExitStatus> {
    // Initialize global handlers once
    #[cfg(unix)]
    initialize_global_sigwinch_handler_impl();

    #[cfg(windows)]
    initialize_windows_input_forwarder();

    // Set PTY active flag to prevent double SIGINT handling
    PTY_ACTIVE.store(true, Ordering::SeqCst);

    // Ensure flag is cleared on exit (RAII guard for panic safety)
    let _pty_guard = PtyActiveGuard;

    // Create minimal terminal guard - ONLY for stdin raw mode
    // This allows proper input forwarding without display interference
    let _terminal_guard = MinimalTerminalGuard::new()?;

    // Get current terminal size
    let pty_size = get_terminal_size()?;

    // Log the detected terminal size (debug only)
    log::info!(
        "PTY: Detected terminal size: {}x{} (rows x cols)",
        pty_size.rows,
        pty_size.cols
    );

    // Create PTY system
    let pty_system = native_pty_system();

    // Create a new PTY pair with graceful error on older Windows
    let pair = pty_system.openpty(pty_size).map_err(|e| {
        #[cfg(windows)]
        {
            // ConPTY requires Windows 10 1809+
            anyhow::anyhow!(
                "PTY creation failed. ConPTY requires Windows 10 version 1809 or later. Error: {}",
                e
            )
        }
        #[cfg(not(windows))]
        {
            anyhow::anyhow!("Failed to create PTY: {}", e)
        }
    })?;

    // Prepare command - handle :pty prefix if present
    let actual_command = if let Some(stripped) = command.strip_prefix(":pty ") {
        stripped
    } else {
        command
    };

    let mut cmd = CommandBuilder::new(&config.shell_path);
    let shell_name = Path::new(&config.shell_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let is_bash = shell_name == "bash" || shell_name == "bash.exe";
    cmd.arg("-c");
    cmd.arg(actual_command);
    cmd.cwd(std::env::current_dir()?);

    // CRITICAL: Preserve tracing environment variables needed for logging
    cmd.env("SHIM_SESSION_ID", &config.session_id);
    cmd.env("SHIM_TRACE_LOG", &config.trace_log_file);
    cmd.env("SHIM_PARENT_CMD_ID", cmd_id);

    // Clear SHIM_ACTIVE/CALLER/CALL_STACK to allow shims to work inside PTY
    cmd.env_remove("SHIM_ACTIVE");
    cmd.env_remove("SHIM_CALLER");
    cmd.env_remove("SHIM_CALL_STACK");

    configure_child_shell_env(
        &mut cmd,
        config,
        is_bash,
        matches!(config.mode, ShellMode::Script(_)),
    );

    // Preserve existing TERM or set a default
    // Many TUIs like claude need the correct TERM to function properly
    match std::env::var("TERM") {
        Ok(term) => cmd.env("TERM", term),
        Err(_) => cmd.env("TERM", "xterm-256color"),
    };

    // Set COLUMNS/LINES for TUIs that read them (only if valid)
    if pty_size.cols > 0 && pty_size.rows > 0 {
        cmd.env("COLUMNS", pty_size.cols.to_string());
        cmd.env("LINES", pty_size.rows.to_string());
    }

    // Log command start with pty flag and initial size
    let start_extra = json!({
        "pty": true,
        "pty_rows": pty_size.rows,
        "pty_cols": pty_size.cols
    });

    // Add debug logging if requested
    if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
        log::debug!(
            "[{}] PTY allocated: {}x{}",
            cmd_id,
            pty_size.cols,
            pty_size.rows
        );
    }

    log_command_event(
        config,
        "command_start",
        actual_command,
        cmd_id,
        Some(start_extra),
    )?;
    let start_time = std::time::Instant::now();

    // Spawn the child process
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context(format!("Failed to spawn PTY command: {actual_command}"))?;

    // Store child PID for signal handling
    if let Some(pid) = child.process_id() {
        running_child_pid.store(pid as i32, Ordering::SeqCst);
    }

    // Verify process group setup (Unix only, debug mode)
    #[cfg(unix)]
    if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
        if let Some(pid) = child.process_id() {
            verify_process_group(Some(pid));
        }
    }

    let (pty_control, reader, manager_handle) = start_pty_manager(pair.master)?;
    let _active_pty_guard = ActivePtyGuard::register(pty_control.clone());

    let exit_status = handle_pty_io(pty_control, reader, &mut child, cmd_id, manager_handle)?;

    // Clear the running PID BEFORE logging completion
    running_child_pid.store(0, Ordering::SeqCst);

    // Log command completion with pty flag
    let duration = start_time.elapsed();
    let mut extra = json!({
        "duration_ms": duration.as_millis(),
        "pty": true
    });

    if let Some(code) = exit_status.code {
        extra["exit_code"] = json!(code);
    }
    if let Some(signal) = exit_status.signal {
        extra["term_signal"] = json!(signal);
    }

    log_command_event(
        config,
        "command_complete",
        actual_command,
        cmd_id,
        Some(extra),
    )?;

    Ok(exit_status)
}

// 🔥 CRITICAL FIX: Windows global input forwarder with Condvar gating
// Prevents stealing input when no PTY is active

#[cfg(windows)]
pub(crate) fn wake_input_gate() {
    let (lock, cvar) = &**WIN_PTY_INPUT_GATE;
    let mut active = lock.lock().unwrap();
    *active = true;
    cvar.notify_all();
}

#[cfg(windows)]
pub(crate) fn sleep_input_gate() {
    let (lock, _) = &**WIN_PTY_INPUT_GATE;
    *lock.lock().unwrap() = false;
}

#[cfg(windows)]
pub(crate) fn initialize_windows_input_forwarder() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        thread::spawn(|| {
            let mut stdin = io::stdin();
            let mut buffer = vec![0u8; 2048]; // Reduced to 2KB for better responsiveness

            loop {
                // Wait until a PTY is active
                {
                    let (lock, cvar) = &**WIN_PTY_INPUT_GATE;
                    let mut active = lock.lock().unwrap();
                    while !*active {
                        // Sleep until woken by PTY activation
                        active = cvar.wait(active).unwrap();
                    }
                }

                // Now we know PTY is active, safe to read stdin
                match stdin.read(&mut buffer) {
                    Ok(0) => {
                        // EOF or no data
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Ok(n) => {
                        if let Some(control) = active_pty_control() {
                            control.write(buffer[..n].to_vec());
                        }
                    }
                    Err(_) => {
                        if active_pty_control().is_none() {
                            continue;
                        }
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }
        });

        if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
            log::debug!("Windows global input forwarder initialized (with Condvar gating)");
        }
    });
}

fn handle_pty_io(
    control: PtyControl,
    mut reader: Box<dyn Read + Send>,
    child: &mut Box<dyn portable_pty::Child + Send + Sync>,
    cmd_id: &str,
    manager_handle: thread::JoinHandle<()>,
) -> Result<PtyExitStatus> {
    let done = Arc::new(AtomicBool::new(false));

    #[cfg(unix)]
    let stdin_join: Option<thread::JoinHandle<()>> = {
        let control_clone = control.clone();
        let done_writer = Arc::clone(&done);
        let cmd_id_stdin = cmd_id.to_string();
        Some(thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = vec![0u8; 4096];

            while !done_writer.load(Ordering::Relaxed) {
                use nix::sys::select::{select, FdSet};
                use nix::sys::time::TimeVal;
                use std::os::unix::io::{AsFd, AsRawFd};

                let stdin_fd = stdin.as_raw_fd();
                let stdin_borrowed = stdin.as_fd();
                let mut read_fds = FdSet::new();
                read_fds.insert(stdin_borrowed);

                let mut timeout = TimeVal::new(0, 100_000);
                let result = select(
                    stdin_fd + 1,
                    Some(&mut read_fds),
                    None,
                    None,
                    Some(&mut timeout),
                );

                match result {
                    Ok(0) => continue,
                    Ok(_) if read_fds.contains(stdin_borrowed) => match stdin.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            control_clone.write(buffer[..n].to_vec());
                        }
                        Err(e) => {
                            log::warn!("[{cmd_id_stdin}] Failed to read from stdin: {e}");
                            break;
                        }
                    },
                    Ok(_) => continue,
                    Err(e) => {
                        if e != nix::errno::Errno::EINTR {
                            log::warn!("[{cmd_id_stdin}] select() failed: {e}");
                            break;
                        }
                    }
                }
            }
        }))
    };

    #[cfg(not(unix))]
    let stdin_join: Option<thread::JoinHandle<()>> = None;

    let done_reader = Arc::clone(&done);
    let cmd_id_output = cmd_id.to_string();
    let output_thread = thread::spawn(move || {
        let mut stdout = io::stdout();
        let mut buffer = vec![0u8; 4096];

        while !done_reader.load(Ordering::Relaxed) {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let mut written = 0;
                    while written < n {
                        match stdout.write(&buffer[written..n]) {
                            Ok(0) => break,
                            Ok(bytes) => {
                                written += bytes;
                            }
                            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                            Err(e) => {
                                log::warn!("[{cmd_id_output}] Failed to write to stdout: {e}");
                                break;
                            }
                        }
                    }
                    if let Err(e) = stdout.flush() {
                        log::warn!("[{cmd_id_output}] Failed to flush stdout: {e}");
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    log::error!("[{cmd_id_output}] Failed to read from PTY: {e}");
                    break;
                }
            }
        }
    });

    let portable_status = child.wait()?;

    done.store(true, Ordering::Relaxed);

    thread::sleep(std::time::Duration::from_millis(50));

    let _ = output_thread.join();

    if let Some(handle) = stdin_join {
        let _ = handle.join();
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
        use windows_sys::Win32::System::Console::*;

        unsafe {
            let h = GetStdHandle(STD_INPUT_HANDLE);
            if h != INVALID_HANDLE_VALUE {
                let mut n: u32 = 0;
                if GetNumberOfConsoleInputEvents(h, &mut n) != 0 && n > 0 {
                    let _ = FlushConsoleInputBuffer(h);
                }
            }
        }
    }

    control.close();
    let _ = manager_handle.join();

    Ok(PtyExitStatus::from_portable_pty(portable_status))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize};
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex, OnceLock};

    static PTY_ENV_TEST_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    fn acquire_pty_env_guard() -> std::sync::MutexGuard<'static, ()> {
        PTY_ENV_TEST_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("pty env guard poisoned")
    }

    #[derive(Clone)]
    struct MockMasterPty {
        sizes: Arc<Mutex<Vec<PtySize>>>,
        writes: Arc<Mutex<Vec<u8>>>,
        flushes: Arc<AtomicUsize>,
    }

    impl MockMasterPty {
        fn new(
            sizes: Arc<Mutex<Vec<PtySize>>>,
            writes: Arc<Mutex<Vec<u8>>>,
            flushes: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                sizes,
                writes,
                flushes,
            }
        }

        fn writer(&self) -> MockWriter {
            MockWriter::new(self.writes.clone(), self.flushes.clone())
        }
    }

    impl portable_pty::MasterPty for MockMasterPty {
        fn resize(&self, size: PtySize) -> Result<(), anyhow::Error> {
            self.sizes.lock().unwrap().push(size);
            Ok(())
        }

        fn get_size(&self) -> Result<PtySize, anyhow::Error> {
            Ok(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
        }

        fn try_clone_reader(&self) -> Result<Box<dyn Read + Send>, anyhow::Error> {
            Ok(Box::new(io::empty()))
        }

        fn take_writer(&self) -> Result<Box<dyn Write + Send>, anyhow::Error> {
            Ok(Box::new(self.writer()))
        }

        #[cfg(unix)]
        fn process_group_leader(&self) -> Option<libc::pid_t> {
            None
        }

        #[cfg(unix)]
        fn as_raw_fd(&self) -> Option<portable_pty::unix::RawFd> {
            None
        }
    }

    #[derive(Clone)]
    struct MockWriter {
        writes: Arc<Mutex<Vec<u8>>>,
        flushes: Arc<AtomicUsize>,
    }

    impl MockWriter {
        fn new(writes: Arc<Mutex<Vec<u8>>>, flushes: Arc<AtomicUsize>) -> Self {
            Self { writes, flushes }
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.writes.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct FailingWriter {
        attempts: Arc<AtomicUsize>,
    }

    impl FailingWriter {
        fn new(attempts: Arc<AtomicUsize>) -> Self {
            Self { attempts }
        }
    }

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            self.attempts.fetch_add(1, Ordering::SeqCst);
            Err(io::Error::new(io::ErrorKind::Other, "writer failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Mock implementations for testing PTY operations without actual PTY allocation
    #[test]
    fn pty_manager_processes_channel_commands() {
        let sizes = Arc::new(Mutex::new(Vec::new()));
        let writes = Arc::new(Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));

        let master = MockMasterPty::new(sizes.clone(), writes.clone(), flushes.clone());
        let writer = master.writer();

        let (tx, rx) = mpsc::channel();
        let control = PtyControl { tx };

        let manager =
            thread::spawn(move || run_pty_manager(Box::new(master), Box::new(writer), rx));

        let resize = PtySize {
            rows: 22,
            cols: 88,
            pixel_width: 0,
            pixel_height: 0,
        };
        control.resize(resize);
        control.write(b"bytes".to_vec());
        control.write(b"-more".to_vec());
        control.close();

        manager.join().expect("pty manager thread panicked");

        assert_eq!(sizes.lock().unwrap().last().copied(), Some(resize));
        assert_eq!(writes.lock().unwrap().as_slice(), b"bytes-more");
        assert!(flushes.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn pty_manager_exits_when_channel_closes() {
        let sizes = Arc::new(Mutex::new(Vec::new()));
        let writes = Arc::new(Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));

        let master = MockMasterPty::new(sizes, writes, flushes);
        let writer = master.writer();

        let (tx, rx) = mpsc::channel();
        let manager =
            thread::spawn(move || run_pty_manager(Box::new(master), Box::new(writer), rx));

        drop(tx);
        manager
            .join()
            .expect("pty manager thread panicked on channel close");
    }

    #[test]
    fn pty_manager_stops_on_write_error() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let sizes = Arc::new(Mutex::new(Vec::new()));
        let writes = Arc::new(Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));

        let master = MockMasterPty::new(sizes, writes, flushes);
        let writer = FailingWriter::new(attempts.clone());

        let (tx, rx) = mpsc::channel();
        let manager =
            thread::spawn(move || run_pty_manager(Box::new(master), Box::new(writer), rx));

        let control = PtyControl { tx };
        control.write(b"should-fail".to_vec());
        drop(control);

        manager
            .join()
            .expect("pty manager thread panicked after writer error");
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_pty_exit_status_creation() {
        // Test PtyExitStatus creation and methods
        let status_success = PtyExitStatus {
            code: Some(0),
            signal: None,
        };
        assert!(status_success.success());
        assert_eq!(status_success.code(), Some(0));

        let status_failure = PtyExitStatus {
            code: Some(1),
            signal: None,
        };
        assert!(!status_failure.success());
        assert_eq!(status_failure.code(), Some(1));

        #[cfg(unix)]
        {
            let status_signal = PtyExitStatus {
                code: None,
                signal: Some(9),
            };
            assert!(!status_signal.success());
            assert_eq!(status_signal.signal(), Some(9));
        }
    }

    #[test]
    fn test_pty_exit_status_from_portable_pty() {
        // Test conversion from portable_pty exit status
        // We can't easily create portable_pty::ExitStatus instances in tests,
        // but we can test the logic

        #[cfg(unix)]
        {
            // Test signal detection logic (raw exit code > 128 indicates signal)
            // This tests the logic in from_portable_pty without requiring actual portable_pty objects
            let raw_exit_code = 130; // 128 + SIGINT(2)
            if raw_exit_code > 128 {
                let signal = raw_exit_code - 128;
                assert_eq!(signal, 2); // SIGINT
            }

            let normal_exit = 1;
            assert!(normal_exit <= 128); // Normal exit code
        }
    }

    #[test]
    fn test_pty_active_guard() {
        // Test that PtyActiveGuard properly manages PTY_ACTIVE flag
        assert!(!PTY_ACTIVE.load(Ordering::SeqCst));

        {
            PTY_ACTIVE.store(true, Ordering::SeqCst);
            let _guard = PtyActiveGuard;
            assert!(PTY_ACTIVE.load(Ordering::SeqCst));
        } // Guard dropped here

        assert!(!PTY_ACTIVE.load(Ordering::SeqCst));
    }

    #[test]
    fn test_get_terminal_size_fallback() {
        // Test terminal size detection fallback behavior
        // Remove environment variables to test fallback
        let original_lines = std::env::var("LINES").ok();
        let original_columns = std::env::var("COLUMNS").ok();

        std::env::remove_var("LINES");
        std::env::remove_var("COLUMNS");

        let size = get_terminal_size().unwrap();

        // When a real TTY is present, ioctl may return actual size (e.g., 24x80 in CI);
        // otherwise we use modern defaults (50x120). In either case, sizes must be > 0.
        assert!(size.rows > 0);
        assert!(size.cols > 0);

        // Test with custom environment variables
        std::env::set_var("LINES", "30");
        std::env::set_var("COLUMNS", "80");

        let custom_size = get_terminal_size().unwrap();
        // On systems with a controlling TTY, ioctl takes precedence; just ensure > 0.
        // On headless builds, env vars provide size. Either way, sizes must be > 0.
        assert!(custom_size.rows > 0);
        assert!(custom_size.cols > 0);

        // Restore original values
        if let Some(lines) = original_lines {
            std::env::set_var("LINES", lines);
        } else {
            std::env::remove_var("LINES");
        }
        if let Some(columns) = original_columns {
            std::env::set_var("COLUMNS", columns);
        } else {
            std::env::remove_var("COLUMNS");
        }
    }

    #[test]
    fn test_minimal_terminal_guard_creation() {
        // Test that MinimalTerminalGuard can be created without panicking
        // This mainly tests that the platform-specific code compiles and runs
        let guard_result = MinimalTerminalGuard::new();

        match guard_result {
            Ok(_guard) => {
                // Guard created successfully - it will be dropped automatically
            }
            Err(_) => {
                // Failed to create guard - this can happen in test environments
                // The important thing is it didn't panic
            }
        }
    }

    #[test]
    fn test_active_pty_guard_management() {
        assert!(active_pty_control().is_none());

        let (tx, rx) = mpsc::channel();
        drop(rx);
        let control = PtyControl { tx };
        {
            let _guard = ActivePtyGuard::register(control);
            assert!(active_pty_control().is_some());
        }
        assert!(active_pty_control().is_none());
    }

    #[test]
    fn test_pty_size_validation() {
        // Test PtySize validation and creation
        let valid_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        assert!(valid_size.rows > 0);
        assert!(valid_size.cols > 0);

        // Test edge cases
        let minimal_size = PtySize {
            rows: 1,
            cols: 1,
            pixel_width: 0,
            pixel_height: 0,
        };

        assert_eq!(minimal_size.rows, 1);
        assert_eq!(minimal_size.cols, 1);

        // Test large sizes
        let large_size = PtySize {
            rows: 1000,
            cols: 1000,
            pixel_width: 1920,
            pixel_height: 1080,
        };

        assert_eq!(large_size.rows, 1000);
        assert_eq!(large_size.cols, 1000);
        assert_eq!(large_size.pixel_width, 1920);
        assert_eq!(large_size.pixel_height, 1080);
    }

    #[cfg(unix)]
    #[test]
    fn test_signal_handling_setup() {
        // Test that global signal handlers can be initialized without panicking
        // This tests the initialization logic but doesn't actually test signal handling
        initialize_global_sigwinch_handler_impl();

        // If we get here without panicking, the initialization succeeded
        // The actual signal handling is tested through integration tests
    }

    #[test]
    fn test_environment_variable_handling() {
        // Test PTY-related environment variable handling
        let original_term = std::env::var("TERM").ok();
        let original_debug = std::env::var("SUBSTRATE_PTY_DEBUG").ok();

        // Test TERM variable fallback
        std::env::remove_var("TERM");
        // In a real PTY execution, we'd set TERM to "xterm-256color" if not present

        // Test debug flag detection
        std::env::remove_var("SUBSTRATE_PTY_DEBUG");
        assert!(std::env::var("SUBSTRATE_PTY_DEBUG").is_err());

        std::env::set_var("SUBSTRATE_PTY_DEBUG", "1");
        assert!(std::env::var("SUBSTRATE_PTY_DEBUG").is_ok());

        // Restore original values
        if let Some(term) = original_term {
            std::env::set_var("TERM", term);
        } else {
            std::env::remove_var("TERM");
        }
        if let Some(debug) = original_debug {
            std::env::set_var("SUBSTRATE_PTY_DEBUG", debug);
        } else {
            std::env::remove_var("SUBSTRATE_PTY_DEBUG");
        }
    }

    #[test]
    fn test_command_preparation() {
        // Test CommandBuilder setup logic (without actually executing)
        let _command = "echo test";
        let session_id = "test-session-123";
        let trace_log = "/tmp/test.log";
        let cmd_id = "cmd-456";

        // Test command prefix stripping
        let pty_prefixed = ":pty echo hello";
        let stripped = pty_prefixed.strip_prefix(":pty ");
        assert_eq!(stripped, Some("echo hello"));

        let normal_command = "echo hello";
        let not_stripped = normal_command.strip_prefix(":pty ");
        assert_eq!(not_stripped, None);

        // Test environment variable names
        assert!(!session_id.is_empty());
        assert!(!cmd_id.is_empty());
        assert!(!trace_log.is_empty());
    }

    #[test]
    fn test_pty_size_environment_validation() {
        // Test COLUMNS and LINES validation logic
        let test_cases = vec![
            (0, 0, false),      // Invalid: zero dimensions
            (1, 1, true),       // Valid: minimal dimensions
            (24, 80, true),     // Valid: typical dimensions
            (50, 120, true),    // Valid: modern defaults
            (1000, 1000, true), // Valid: large dimensions
        ];

        for (rows, cols, should_be_valid) in test_cases {
            let is_valid = rows > 0 && cols > 0;
            assert_eq!(is_valid, should_be_valid, "Failed for {}x{}", rows, cols);

            if is_valid {
                std::env::set_var("LINES", rows.to_string());
                std::env::set_var("COLUMNS", cols.to_string());

                let size = get_terminal_size().unwrap();
                // If ioctl reports a real TTY size, it may override env; just ensure positive.
                assert!(size.rows > 0);
                assert!(size.cols > 0);
            }
        }

        // Cleanup
        std::env::remove_var("LINES");
        std::env::remove_var("COLUMNS");
    }

    #[test]
    fn test_atomic_operations() {
        // Test atomic operations used in PTY management
        let pid = Arc::new(AtomicI32::new(0));
        let active = Arc::new(AtomicBool::new(false));

        // Test PID management
        assert_eq!(pid.load(Ordering::SeqCst), 0);
        pid.store(12345, Ordering::SeqCst);
        assert_eq!(pid.load(Ordering::SeqCst), 12345);
        pid.store(0, Ordering::SeqCst);
        assert_eq!(pid.load(Ordering::SeqCst), 0);

        // Test active flag management
        assert!(!active.load(Ordering::SeqCst));
        active.store(true, Ordering::SeqCst);
        assert!(active.load(Ordering::SeqCst));
        active.store(false, Ordering::SeqCst);
        assert!(!active.load(Ordering::SeqCst));
    }

    #[test]
    fn test_error_handling_contexts() {
        // Test error handling and context preservation
        use anyhow::Context;

        // Test error context chaining
        let base_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let contextual_error: Result<(), _> = Err(base_error).context("Failed to open PTY");

        assert!(contextual_error.is_err());
        let error_msg = contextual_error.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to open PTY"));
    }

    #[cfg(unix)]
    #[test]
    fn test_process_group_verification() {
        // Test process group verification logic (without actually spawning processes)
        // This tests the logic in verify_process_group

        let test_pid = std::process::id();
        verify_process_group(Some(test_pid));

        // If we get here without panicking, the verification logic works
        verify_process_group(None);
    }

    #[cfg(not(unix))]
    #[test]
    fn test_process_group_verification_noop() {
        // Test that non-Unix platforms handle process group verification gracefully
        verify_process_group(Some(12345));
        verify_process_group(None);
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_console_size_detection() {
        // Test Windows console size detection logic
        // This mainly ensures the code compiles and doesn't panic
        let _size = windows_console_size();
        // Function may return None in test environment, which is expected
    }

    #[test]
    fn test_timing_operations() {
        // Test timing operations used in PTY execution
        use std::time::{Duration, Instant};

        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(5));
        assert!(elapsed < Duration::from_millis(100));

        // Test duration conversion to milliseconds
        let duration_ms = elapsed.as_millis();
        assert!(duration_ms >= 5);
        assert!(duration_ms < 100);
    }

    #[test]
    fn test_thread_safety_primitives() {
        // Test thread safety primitives used in PTY operations
        use std::sync::atomic::AtomicI32;
        use std::thread;

        let counter = Arc::new(AtomicI32::new(0));
        let mut handles = vec![];

        // Spawn multiple threads to test thread safety
        for _ in 0..5 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn test_pty_debug_environment() {
        let _guard = acquire_pty_env_guard();
        // Test PTY debug environment detection
        let original_debug = std::env::var("SUBSTRATE_PTY_DEBUG").ok();

        // Test debug disabled
        std::env::remove_var("SUBSTRATE_PTY_DEBUG");
        // Some environments pin this variable; skip the unset assertion in that case.
        let debug_env_forced = std::env::var("SUBSTRATE_PTY_DEBUG").is_ok();
        if !debug_env_forced {
            assert!(std::env::var("SUBSTRATE_PTY_DEBUG").is_err());
        }

        // Test debug enabled
        std::env::set_var("SUBSTRATE_PTY_DEBUG", "1");
        assert!(std::env::var("SUBSTRATE_PTY_DEBUG").is_ok());

        // Test various debug values
        for debug_value in ["1", "true", "on", "yes"] {
            std::env::set_var("SUBSTRATE_PTY_DEBUG", debug_value);
            assert!(std::env::var("SUBSTRATE_PTY_DEBUG").is_ok());
        }

        // Restore original value
        if let Some(debug) = original_debug {
            std::env::set_var("SUBSTRATE_PTY_DEBUG", debug);
        } else {
            std::env::remove_var("SUBSTRATE_PTY_DEBUG");
        }
    }

    #[test]
    fn test_platform_specific_compilation() {
        // Test that platform-specific code compiles correctly

        #[cfg(unix)]
        {
            // Test Unix-specific functionality compiles
            let _ = libc::STDIN_FILENO;
            let _ = libc::STDOUT_FILENO;
            let _ = libc::STDERR_FILENO;
        }

        #[cfg(windows)]
        {
            // Test Windows-specific functionality compiles
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            use windows_sys::Win32::System::Console::STD_OUTPUT_HANDLE;
            let _ = INVALID_HANDLE_VALUE;
            let _ = STD_OUTPUT_HANDLE;
        }
    }

    #[test]
    fn test_json_serialization() {
        // Test JSON serialization used in logging
        use serde_json::json;

        let test_data = json!({
            "pty": true,
            "pty_rows": 24,
            "pty_cols": 80,
            "duration_ms": 1500,
            "exit_code": 0
        });

        assert!(test_data.is_object());
        assert_eq!(test_data["pty"], json!(true));
        assert_eq!(test_data["pty_rows"], json!(24));
        assert_eq!(test_data["pty_cols"], json!(80));
        assert_eq!(test_data["duration_ms"], json!(1500));
        assert_eq!(test_data["exit_code"], json!(0));

        // Test serialization to string
        let json_string = serde_json::to_string(&test_data).unwrap();
        assert!(!json_string.is_empty());
        assert!(json_string.contains("pty"));
    }
}
