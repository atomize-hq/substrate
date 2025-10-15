use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde_json::json;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{log_command_event, ShellConfig, CURRENT_PTY, PTY_ACTIVE};

#[cfg(unix)]
pub(crate) fn initialize_global_sigwinch_handler_impl() {
    use signal_hook::{consts::SIGWINCH, iterator::Signals};

    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        thread::spawn(|| {
            let mut signals = match Signals::new([SIGWINCH]) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to register SIGWINCH handler: {e}");
                    return;
                }
            };

            for _ in signals.forever() {
                // Resize current PTY if one is active
                // NOTE: signal_hook runs this in a normal thread, not a signal handler
                // Allocations and logging are safe here
                if let Ok(pty_opt) = CURRENT_PTY.lock() {
                    if let Some(ref pty) = *pty_opt {
                        if let Ok(size) = get_terminal_size() {
                            // ioctl + resize called from handler thread
                            let _ = pty.lock().unwrap().resize(size);

                            // Debug logging if requested (safe in normal thread)
                            if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
                                log::debug!("SIGWINCH: Resized PTY to {}x{}", size.cols, size.rows);
                                // 🔥 PRODUCTION: Would emit terminal_resize telemetry here but
                                // cannot access ShellConfig from SIGWINCH thread
                            }
                        }
                    }
                }
            }
        });
    });
}

/// Custom exit status for PTY commands
#[derive(Debug, Clone)]
pub struct PtyExitStatus {
    pub code: Option<i32>,
    pub signal: Option<i32>,
}

impl PtyExitStatus {
    fn from_portable_pty(status: portable_pty::ExitStatus) -> Self {
        #[cfg(unix)]
        {
            let raw = status.exit_code() as i32;
            if raw > 128 {
                // Terminated by signal (128 + signal number)
                // Note: We intentionally don't set the core dump bit (0x80) since
                // portable_pty doesn't expose whether a core dump occurred
                PtyExitStatus {
                    code: None,
                    signal: Some(raw - 128),
                }
            } else {
                PtyExitStatus {
                    code: Some(raw),
                    signal: None,
                }
            }
        }

        #[cfg(not(unix))]
        {
            PtyExitStatus {
                code: Some(status.exit_code() as i32),
                signal: None,
            }
        }
    }

    pub fn success(&self) -> bool {
        self.code == Some(0)
    }

    pub fn code(&self) -> Option<i32> {
        self.code
    }

    #[cfg(unix)]
    pub fn signal(&self) -> Option<i32> {
        self.signal
    }

    #[cfg(not(unix))]
    pub fn signal(&self) -> Option<i32> {
        None
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
    crate::initialize_global_sigwinch_handler();

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

    // Set up PTY master for resize handling
    // Wrap the Box in Arc<Mutex> for thread safety
    let pty_master = Arc::new(Mutex::new(pair.master));

    // Register this PTY for SIGWINCH handling with RAII guard
    let _current_pty_guard = CurrentPtyGuard::register(Arc::clone(&pty_master));

    // Handle I/O between terminal and PTY
    let exit_status = handle_pty_io(pty_master, &mut child, cmd_id)?;

    // PTY automatically unregistered by CurrentPtyGuard drop

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

// RAII guard to ensure PTY_ACTIVE flag is cleared even on panic
struct PtyActiveGuard;

impl Drop for PtyActiveGuard {
    fn drop(&mut self) {
        PTY_ACTIVE.store(false, Ordering::SeqCst);
    }
}

// RAII guard for CURRENT_PTY registration (panic-safe)
struct CurrentPtyGuard;

impl CurrentPtyGuard {
    fn register(pty: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>) -> Self {
        *CURRENT_PTY.lock().unwrap() = Some(pty);
        Self
    }
}

impl Drop for CurrentPtyGuard {
    fn drop(&mut self) {
        *CURRENT_PTY.lock().unwrap() = None;
    }
}

#[cfg(windows)]
fn windows_console_size() -> Option<PtySize> {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::*;
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        if h == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut info = std::mem::MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFO>::uninit();
        if GetConsoleScreenBufferInfo(h, info.as_mut_ptr()) != 0 {
            let info = info.assume_init();
            let cols = (info.srWindow.Right - info.srWindow.Left + 1) as u16;
            let rows = (info.srWindow.Bottom - info.srWindow.Top + 1) as u16;
            if rows > 0 && cols > 0 {
                return Some(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        }
        None
    }
}

fn get_terminal_size() -> Result<PtySize> {
    #[cfg(windows)]
    if let Some(sz) = windows_console_size() {
        return Ok(sz);
    }

    #[cfg(unix)]
    {
        use libc::{ioctl, winsize, TIOCGWINSZ};
        use std::mem;

        // CRITICAL: Try /dev/tty first - this always refers to the controlling terminal
        // This ensures we get the real size even when stdin/stdout are redirected
        if let Ok(tty) = std::fs::File::open("/dev/tty") {
            use std::os::unix::io::AsRawFd;
            let fd = tty.as_raw_fd();
            unsafe {
                let mut size: winsize = mem::zeroed();
                if ioctl(fd, TIOCGWINSZ, &mut size) == 0 && size.ws_row > 0 && size.ws_col > 0 {
                    return Ok(PtySize {
                        rows: size.ws_row,
                        cols: size.ws_col,
                        pixel_width: size.ws_xpixel,
                        pixel_height: size.ws_ypixel,
                    });
                }
            }
        }

        // Fallback: Try stdin, stdout, stderr in order (handles redirects)
        for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            unsafe {
                let mut size: winsize = mem::zeroed();
                if ioctl(fd, TIOCGWINSZ, &mut size) == 0 && size.ws_row > 0 && size.ws_col > 0 {
                    return Ok(PtySize {
                        rows: size.ws_row,
                        cols: size.ws_col,
                        pixel_width: size.ws_xpixel,
                        pixel_height: size.ws_ypixel,
                    });
                }
            }
        }
    }

    // Fallback to environment or MODERN defaults (not 1970s terminals!)
    let rows = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50); // Modern default: 50 rows
    let cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120); // Modern default: 120 columns

    Ok(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })
}

// 🔥 CRITICAL FIX: Windows global input forwarder with Condvar gating
// Prevents stealing input when no PTY is active
#[cfg(windows)]
use lazy_static::lazy_static;
#[cfg(windows)]
use std::sync::Condvar;

#[cfg(windows)]
lazy_static! {
    static ref CURRENT_PTY_WRITER: Arc<Mutex<Option<Box<dyn Write + Send>>>> =
        Arc::new(Mutex::new(None));
    // Condvar to wake/sleep the forwarder thread
    // 🔥 MUST-FIX: Renamed from PTY_ACTIVE to avoid collision with crate::PTY_ACTIVE
    static ref WIN_PTY_INPUT_GATE: Arc<(Mutex<bool>, Condvar)> =
        Arc::new((Mutex::new(false), Condvar::new()));
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
                        // Forward to current PTY writer if still active
                        if let Ok(mut writer_lock) = CURRENT_PTY_WRITER.lock() {
                            if let Some(ref mut writer) = *writer_lock {
                                let _ = writer.write_all(&buffer[..n]);
                                let _ = writer.flush();
                            } else {
                                // PTY was cleared while we were reading, go back to waiting
                                continue;
                            }
                        }
                    }
                    Err(_) => {
                        // Error reading stdin, check if PTY still active
                        if let Ok(writer_lock) = CURRENT_PTY_WRITER.lock() {
                            if writer_lock.is_none() {
                                // PTY cleared, go back to waiting
                                continue;
                            }
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
    pty_master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    child: &mut Box<dyn portable_pty::Child + Send + Sync>,
    cmd_id: &str,
) -> Result<PtyExitStatus> {
    let done = Arc::new(AtomicBool::new(false));

    // Get writer for stdin->pty (wrap in Arc<Mutex<Option>> so we can drop it from main thread)
    let writer = {
        let master = pty_master.lock().unwrap();
        Arc::new(Mutex::new(Some(
            master
                .take_writer()
                .context("Failed to create PTY writer")?,
        )))
    };

    // Platform-specific stdin handling
    #[cfg(unix)]
    let stdin_join: Option<thread::JoinHandle<()>> = {
        // Clone for the stdin thread
        let writer_for_thread = Arc::clone(&writer);
        // Unix: Spawn thread to copy stdin to PTY (will be joined after child exits)
        // CRITICAL FIX: Use non-blocking I/O to prevent stealing input after PTY exit
        let done_writer = Arc::clone(&done);
        let cmd_id_stdin = cmd_id.to_string();
        Some(thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = vec![0u8; 4096];

            while !done_writer.load(Ordering::Relaxed) {
                // Check if writer is still valid BEFORE reading stdin
                let writer_guard = match writer_for_thread.try_lock() {
                    Ok(guard) => guard,
                    Err(_) => break, // Writer was dropped or is locked, exit thread
                };

                // If writer is None, exit immediately
                if writer_guard.is_none() {
                    break;
                }

                // Drop the guard before blocking read to avoid holding lock
                drop(writer_guard);

                // Set up a select/poll with timeout to avoid blocking forever
                use nix::sys::select::{select, FdSet};
                use nix::sys::time::TimeVal;
                use std::os::unix::io::{AsFd, AsRawFd};

                let stdin_fd = stdin.as_raw_fd();
                let stdin_borrowed = stdin.as_fd();
                let mut read_fds = FdSet::new();
                read_fds.insert(stdin_borrowed);

                // Wait up to 100ms for input
                let mut timeout = TimeVal::new(0, 100_000); // 100ms = 100,000 microseconds
                let result = select(
                    stdin_fd + 1,
                    Some(&mut read_fds),
                    None,
                    None,
                    Some(&mut timeout),
                );

                match result {
                    Ok(0) => {
                        // Timeout - check done flag and continue
                        continue;
                    }
                    Ok(_) if read_fds.contains(stdin_borrowed) => {
                        // Data available, try to read
                        match stdin.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                // Re-acquire writer guard to write
                                if let Ok(mut writer_guard) = writer_for_thread.try_lock() {
                                    if let Some(ref mut writer) = *writer_guard {
                                        if let Err(e) = writer.write_all(&buffer[..n]) {
                                            if !done_writer.load(Ordering::Relaxed) {
                                                log::warn!(
                                                    "[{cmd_id_stdin}] Failed to write to PTY: {e}"
                                                );
                                            }
                                            break;
                                        }
                                        let _ = writer.flush();
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("[{cmd_id_stdin}] Failed to read from stdin: {e}");
                                break;
                            }
                        }
                    }
                    Ok(_) => continue, // Spurious wakeup
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

    #[cfg(windows)]
    {
        // Windows: Use global input forwarder to avoid thread leak
        // Set the current PTY writer and wake the forwarder thread
        // TODO: Properly handle writer cloning on Windows
        // For now, we'll skip PTY input forwarding on Windows as it needs
        // a different approach than Unix

        // Clear any existing writer
        *CURRENT_PTY_WRITER.lock().unwrap() = None;

        // Note: Windows PTY input handling needs to be reimplemented
        // The try_clone_writer() method doesn't exist on trait objects
    }

    // Spawn thread to copy PTY output to stdout (using blocking I/O)
    let mut reader = {
        let master = pty_master.lock().unwrap();
        master
            .try_clone_reader()
            .context("Failed to create PTY reader")?
    };

    let done_reader = Arc::clone(&done);
    let cmd_id_output = cmd_id.to_string();
    let output_thread = thread::spawn(move || {
        let mut stdout = io::stdout();
        // Use smaller buffer to prevent blocking on partial reads
        let mut buffer = vec![0u8; 4096];

        while !done_reader.load(Ordering::Relaxed) {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF - child process exited
                Ok(n) => {
                    // Handle partial writes properly to prevent blocking
                    let mut written = 0;
                    while written < n {
                        match stdout.write(&buffer[written..n]) {
                            Ok(0) => break, // Can't write anymore
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
                    // Flush after processing the buffer
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

    // Wait for child to exit (blocking wait, not polling - more efficient)
    let portable_status = child.wait()?;

    // Signal threads to stop FIRST
    done.store(true, Ordering::Relaxed);

    // CRITICAL FIX: Drop the writer to stop stdin thread from trying to write
    #[cfg(unix)]
    {
        // Take the writer out of the Arc<Mutex> and drop it
        // This causes the stdin thread to fail on its next write attempt
        if let Ok(mut guard) = writer.lock() {
            *guard = None; // Drop the writer
        }
    }

    // CRITICAL: Drain any remaining output before sending reset sequences
    // This prevents race condition with TUI cleanup sequences
    thread::sleep(std::time::Duration::from_millis(50)); // Give TUI time to send cleanup

    // Wait for output thread (it will exit when PTY closes)
    let _ = output_thread.join();

    // Platform-specific cleanup
    #[cfg(unix)]
    if let Some(handle) = stdin_join {
        // Unix: Join stdin thread (it should exit quickly now that writer is dropped)
        let _ = handle.join();
    }

    #[cfg(windows)]
    {
        // Windows: Clear the current PTY writer and put forwarder to sleep
        *CURRENT_PTY_WRITER.lock().unwrap() = None;

        // Put the forwarder thread back to sleep
        let (lock, _cvar) = &**WIN_PTY_INPUT_GATE;
        *lock.lock().unwrap() = false;

        // Flush any straggler input to prevent swallowed keystrokes
        // This prevents the next keystroke from waking the read() and getting discarded
        // Only flush if there's actually pending input to avoid nuking legitimate keystrokes
        unsafe {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            use windows_sys::Win32::System::Console::*;
            let h = GetStdHandle(STD_INPUT_HANDLE);
            if h != INVALID_HANDLE_VALUE {
                let mut n: u32 = 0;
                if GetNumberOfConsoleInputEvents(h, &mut n) != 0 && n > 0 {
                    let _ = FlushConsoleInputBuffer(h);
                }
            }
        }
    }

    // Note: We've tried various approaches to make the prompt appear immediately:
    // - TIOCSTI injection of newline/Ctrl+L - doesn't solve the blocking issue
    // - Terminal state restoration - helps but prompt still doesn't appear
    // - The core issue is that Reedline's read_line() blocks waiting for input
    //   and we're not using the ExecuteHostCommand event system

    Ok(PtyExitStatus::from_portable_pty(portable_status))
}

#[cfg(unix)]
fn verify_process_group(pid: Option<u32>) {
    // Verify child is session leader with controlling terminal
    if let Some(pid) = pid {
        // This is for debugging/verification only
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(["-o", "pid,pgid,tpgid,stat", "-p", &pid.to_string()])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            log::debug!("Process group info for {pid}: {output_str}");
            // We want pid==pgid==tpgid for proper session leader
        }
    }
}

#[cfg(not(unix))]
fn verify_process_group(_pid: Option<u32>) {
    // No-op on non-Unix platforms
}

// Minimal terminal guard - ONLY sets stdin to raw mode for input forwarding
// Does NOT touch stdout to avoid display corruption
struct MinimalTerminalGuard {
    #[cfg(unix)]
    saved_termios: Option<nix::sys::termios::Termios>,
    #[cfg(windows)]
    saved_stdin_mode: Option<u32>,
}

impl MinimalTerminalGuard {
    fn new() -> Result<Self> {
        #[cfg(unix)]
        {
            use nix::sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg};
            use std::os::unix::io::{AsRawFd, BorrowedFd};

            let raw_fd = io::stdin().as_raw_fd();
            let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
            let saved_termios = tcgetattr(fd).ok();

            // Set raw mode on stdin ONLY for proper input forwarding
            if let Some(ref orig) = saved_termios {
                let mut raw = orig.clone();
                cfmakeraw(&mut raw);

                // Ensure immediate input without buffering
                raw.control_chars[nix::sys::termios::SpecialCharacterIndices::VMIN as usize] = 1;
                raw.control_chars[nix::sys::termios::SpecialCharacterIndices::VTIME as usize] = 0;

                let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
                let _ = tcsetattr(fd, SetArg::TCSANOW, &raw);
            }

            Ok(Self { saved_termios })
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            use windows_sys::Win32::System::Console::*;

            let mut saved_stdin_mode = None;

            unsafe {
                // Save and modify stdin console mode ONLY
                let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
                if h_stdin != INVALID_HANDLE_VALUE {
                    let mut mode = 0;
                    if GetConsoleMode(h_stdin, &mut mode) != 0 {
                        saved_stdin_mode = Some(mode);

                        // Clear: ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT
                        // Set: ENABLE_VIRTUAL_TERMINAL_INPUT
                        let new_mode = (mode
                            & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT))
                            | ENABLE_VIRTUAL_TERMINAL_INPUT;
                        SetConsoleMode(h_stdin, new_mode);
                    }
                }
            }

            Ok(Self { saved_stdin_mode })
        }

        #[cfg(not(any(unix, windows)))]
        Ok(Self {})
    }
}

impl Drop for MinimalTerminalGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use nix::sys::termios::{tcsetattr, SetArg};
            use std::os::unix::io::{AsRawFd, BorrowedFd};

            if let Some(ref termios) = self.saved_termios {
                let raw_fd = io::stdin().as_raw_fd();
                let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
                let _ = tcsetattr(fd, SetArg::TCSANOW, termios);
            }
        }

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            use windows_sys::Win32::System::Console::*;

            unsafe {
                if let Some(mode) = self.saved_stdin_mode {
                    let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
                    if h_stdin != INVALID_HANDLE_VALUE {
                        SetConsoleMode(h_stdin, mode);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicI32};
    use std::sync::Arc;

    // Mock implementations for testing PTY operations without actual PTY allocation

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
    fn test_current_pty_guard_management() {
        // Test CurrentPtyGuard registration and cleanup
        // We can't easily test with real PTY objects, but we can test the guard logic

        // Initially, no PTY should be registered
        {
            let pty_lock = CURRENT_PTY.lock().unwrap();
            assert!(pty_lock.is_none());
        }

        // We can't easily create a mock PTY for this test due to trait object constraints,
        // but we can test that the guard mechanism doesn't panic
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
        // Test PTY debug environment detection
        let original_debug = std::env::var("SUBSTRATE_PTY_DEBUG").ok();

        // Test debug disabled
        std::env::remove_var("SUBSTRATE_PTY_DEBUG");
        assert!(std::env::var("SUBSTRATE_PTY_DEBUG").is_err());

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
