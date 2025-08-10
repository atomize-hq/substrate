use anyhow::{Result, Context};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread;
use serde_json::json;

use crate::{ShellConfig, log_command_event, CURRENT_PTY, PTY_ACTIVE};

#[cfg(unix)]
pub(crate) fn initialize_global_sigwinch_handler_impl() {
    use signal_hook::{consts::SIGWINCH, iterator::Signals};
    
    static INIT: std::sync::Once = std::sync::Once::new();
    
    INIT.call_once(|| {
        thread::spawn(|| {
            let mut signals = match Signals::new(&[SIGWINCH]) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to register SIGWINCH handler: {}", e);
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
    
    // Save and prepare terminal for PTY
    let _terminal_guard = TerminalGuard::new()?;
    
    // Get current terminal size
    let pty_size = get_terminal_size()?;
    
    // Create PTY system
    let pty_system = native_pty_system();
    
    // Create a new PTY pair with graceful error on older Windows
    let pair = pty_system
        .openpty(pty_size)
        .map_err(|e| {
            #[cfg(windows)]
            {
                // ConPTY requires Windows 10 1809+
                anyhow::anyhow!("PTY creation failed. ConPTY requires Windows 10 version 1809 or later. Error: {}", e)
            }
            #[cfg(not(windows))]
            {
                anyhow::anyhow!("Failed to create PTY: {}", e)
            }
        })?;
    
    // Prepare command - handle :pty prefix if present
    let actual_command = if command.starts_with(":pty ") {
        &command[5..]
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
        log::debug!("[{}] PTY allocated: {}x{}", cmd_id, pty_size.cols, pty_size.rows);
    }
    
    log_command_event(config, "command_start", actual_command, cmd_id, 
        Some(start_extra))?;
    let start_time = std::time::Instant::now();
    
    // Spawn the child process
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context(format!("Failed to spawn PTY command: {}", actual_command))?;
    
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
    
    log_command_event(config, "command_complete", actual_command, cmd_id, Some(extra))?;
    
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
        if h == INVALID_HANDLE_VALUE { return None; }
        let mut info = std::mem::MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFO>::uninit();
        if GetConsoleScreenBufferInfo(h, info.as_mut_ptr()) != 0 {
            let info = info.assume_init();
            let cols = (info.srWindow.Right - info.srWindow.Left + 1) as u16;
            let rows = (info.srWindow.Bottom - info.srWindow.Top + 1) as u16;
            if rows > 0 && cols > 0 {
                return Some(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
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
        
        // Try stdin, stdout, stderr in order (handles redirects)
        for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            unsafe {
                let mut size: winsize = mem::zeroed();
                if ioctl(fd, TIOCGWINSZ, &mut size) == 0 
                    && size.ws_row > 0 && size.ws_col > 0 {
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
    
    // Fallback to environment or defaults
    let rows = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);
    let cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);
    
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
use std::sync::{Condvar, Mutex};

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
            let mut buffer = vec![0u8; 4096];
            
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
    
    // Get writer for stdin->pty (briefly lock to take writer)
    let mut writer = {
        let master = pty_master.lock().unwrap();
        master.take_writer()
            .context("Failed to create PTY writer")?
    };
    
    // 🔥 CRITICAL FIX: Declare stdin_thread handle outside platform blocks
    // 🔥 MUST-FIX: Initialize to None to avoid uninitialized variable on non-Unix
    let stdin_join: Option<thread::JoinHandle<()>>;
    
    // Platform-specific stdin handling
    #[cfg(unix)]
    {
        // Unix: Spawn thread to copy stdin to PTY (will be joined after child exits)
        // With VMIN=0/VTIME=1, reads timeout every 100ms so thread can check done flag
        let done_writer = Arc::clone(&done);
        let cmd_id_stdin = cmd_id.to_string();
        stdin_join = Some(thread::spawn(move || {
            let mut stdin = io::stdin();
            // Increased buffer size for better performance with complex TUIs
            let mut buffer = vec![0u8; 32768];
            
            while !done_writer.load(Ordering::Relaxed) {
                match stdin.read(&mut buffer) {
                    Ok(0) => {
                        // Could be timeout (VTIME) or actual EOF
                        // Continue looping to check done flag
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Ok(n) => {
                        if let Err(e) = writer.write_all(&buffer[..n]) {
                            log::warn!("[{}] Failed to write to PTY: {}", cmd_id_stdin, e);
                            break;
                        }
                        if let Err(e) = writer.flush() {
                            log::warn!("[{}] Failed to flush PTY writer: {}", cmd_id_stdin, e);
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // Timeout from VTIME, check done flag
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(e) => {
                        log::warn!("[{}] Failed to read from stdin: {}", cmd_id_stdin, e);
                        break;
                    }
                }
            }
        }));
    }
    
    #[cfg(windows)]
    {
        stdin_join = None; // Windows doesn't use per-command threads
        
        // Windows: Use global input forwarder to avoid thread leak
        // Set the current PTY writer and wake the forwarder thread
        *CURRENT_PTY_WRITER.lock().unwrap() = Some(Box::new(writer));
        
        // Wake the forwarder thread
        let (lock, cvar) = &**WIN_PTY_INPUT_GATE;
        *lock.lock().unwrap() = true;
        cvar.notify_all();
    }
    
    // Spawn thread to copy PTY output to stdout (using blocking I/O)
    let mut reader = {
        let master = pty_master.lock().unwrap();
        master.try_clone_reader()
            .context("Failed to create PTY reader")?
    };
    
    let done_reader = Arc::clone(&done);
    let cmd_id_output = cmd_id.to_string();
    let output_thread = thread::spawn(move || {
        let mut stdout = io::stdout();
        // Increased buffer size for better performance with complex TUIs
        let mut buffer = vec![0u8; 32768];
        
        while !done_reader.load(Ordering::Relaxed) {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF - child process exited
                Ok(n) => {
                    if let Err(e) = stdout.write_all(&buffer[..n]) {
                        log::warn!("[{}] Failed to write to stdout: {}", cmd_id_output, e);
                        break;
                    }
                    if let Err(e) = stdout.flush() {
                        log::warn!("[{}] Failed to flush stdout: {}", cmd_id_output, e);
                    }
                }
                Err(e) => {
                    log::error!("[{}] Failed to read from PTY: {}", cmd_id_output, e);
                    break;
                }
            }
        }
    });
    
    // Wait for child to exit (blocking wait, not polling - more efficient)
    let portable_status = child.wait()?;
    
    // Signal threads to stop
    done.store(true, Ordering::Relaxed);
    
    // Wait for output thread (it will exit when PTY closes)
    let _ = output_thread.join();
    
    // Platform-specific cleanup
    #[cfg(unix)]
    if let Some(handle) = stdin_join {
        // Unix: Join stdin thread (with O_NONBLOCK it won't hang)
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
            use windows_sys::Win32::System::Console::*;
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            let h = GetStdHandle(STD_INPUT_HANDLE);
            if h != INVALID_HANDLE_VALUE {
                let mut n: u32 = 0;
                if GetNumberOfConsoleInputEvents(h, &mut n) != 0 && n > 0 {
                    let _ = FlushConsoleInputBuffer(h);
                }
            }
        }
    }
    
    Ok(PtyExitStatus::from_portable_pty(portable_status))
}

#[cfg(unix)]
fn verify_process_group(pid: Option<u32>) {
    // Verify child is session leader with controlling terminal
    if let Some(pid) = pid {
        // This is for debugging/verification only
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(&["-o", "pid,pgid,tpgid,stat", "-p", &pid.to_string()])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            log::debug!("Process group info for {}: {}", pid, output_str);
            // We want pid==pgid==tpgid for proper session leader
        }
    }
}

#[cfg(not(unix))]
fn verify_process_group(_pid: Option<u32>) {
    // No-op on non-Unix platforms
}

struct TerminalGuard {
    #[cfg(unix)]
    saved_termios: Option<nix::sys::termios::Termios>,
    #[cfg(unix)]
    saved_stdin_flags: Option<i32>,
    #[cfg(windows)]
    saved_stdin_mode: Option<u32>,
    #[cfg(windows)]
    saved_stdout_mode: Option<u32>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        #[cfg(unix)]
        {
            use nix::sys::termios::{tcgetattr, tcsetattr, SetArg};
            use std::os::unix::io::{AsRawFd, BorrowedFd};
            
            let raw_fd = io::stdin().as_raw_fd();
            let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
            let saved_termios = tcgetattr(fd).ok();
            
            // Save original file flags for restoration
            let saved_stdin_flags = unsafe {
                let flags = libc::fcntl(raw_fd, libc::F_GETFL);
                if flags != -1 { 
                    Some(flags) 
                } else {
                    if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
                        log::debug!("Failed to get stdin flags with fcntl(F_GETFL): {}", io::Error::last_os_error());
                    }
                    None 
                }
            };
            
            // Set minimal terminal changes for PTY operation
            // Instead of full raw mode, just disable echo and canonical mode
            // This allows TUI apps to manage their own terminal settings
            if let Some(ref orig) = saved_termios {
                let mut new_termios = orig.clone();
                
                // Disable echo and canonical mode (line buffering)
                // But keep other terminal processing intact
                use nix::sys::termios::{LocalFlags, InputFlags};
                new_termios.local_flags.remove(LocalFlags::ECHO | LocalFlags::ICANON);
                
                // Keep signal processing (ISIG) so Ctrl-C/Ctrl-Z work normally
                // This is different from cfmakeraw which disables everything
                
                // Set VMIN=1, VTIME=1 for responsive input with timeout
                // This allows reads to return with partial data
                new_termios.control_chars[nix::sys::termios::SpecialCharacterIndices::VMIN as usize] = 1;
                new_termios.control_chars[nix::sys::termios::SpecialCharacterIndices::VTIME as usize] = 1; // 0.1s timeout
                
                let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
                let _ = tcsetattr(fd, SetArg::TCSANOW, &new_termios);
                
                // Set O_NONBLOCK on stdin to prevent blocking even in canonical mode
                // This ensures the stdin thread can be joined cleanly
                if let Some(flags) = saved_stdin_flags {
                    unsafe {
                        let result = libc::fcntl(raw_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                        if result == -1 && std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
                            log::debug!("Failed to set O_NONBLOCK with fcntl(F_SETFL): {}", io::Error::last_os_error());
                        }
                    }
                }
            }
            
            Ok(Self { saved_termios, saved_stdin_flags })
        }
        
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Console::*;
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            
            let mut saved_stdin_mode = None;
            let mut saved_stdout_mode = None;
            
            unsafe {
                // Save and modify stdin console mode
                let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
                if h_stdin != INVALID_HANDLE_VALUE {
                    let mut mode = 0;
                    if GetConsoleMode(h_stdin, &mut mode) != 0 {
                        saved_stdin_mode = Some(mode);
                        
                        // Clear: ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT
                        // Set: ENABLE_VIRTUAL_TERMINAL_INPUT
                        let new_mode = (mode & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT))
                            | ENABLE_VIRTUAL_TERMINAL_INPUT;
                        SetConsoleMode(h_stdin, new_mode);
                    }
                }
                
                // Save and modify stdout console mode
                let h_stdout = GetStdHandle(STD_OUTPUT_HANDLE);
                if h_stdout != INVALID_HANDLE_VALUE {
                    let mut mode = 0;
                    if GetConsoleMode(h_stdout, &mut mode) != 0 {
                        saved_stdout_mode = Some(mode);
                        
                        // Add: ENABLE_VIRTUAL_TERMINAL_PROCESSING for VT sequences
                        let new_mode = mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
                        SetConsoleMode(h_stdout, new_mode);
                    }
                }
            }
            
            Ok(Self {
                saved_stdin_mode,
                saved_stdout_mode,
            })
        }
        
        #[cfg(not(any(unix, windows)))]
        Ok(Self {})
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::unix::io::{AsRawFd, BorrowedFd};
            let raw_fd = io::stdin().as_raw_fd();
            
            // Restore original termios settings (exits raw mode)
            if let Some(ref termios) = self.saved_termios {
                use nix::sys::termios::{tcsetattr, SetArg};
                let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
                let _ = tcsetattr(fd, SetArg::TCSANOW, termios);
            }
            
            // Restore original file flags (removes O_NONBLOCK)
            if let Some(flags) = self.saved_stdin_flags {
                unsafe {
                    let _ = libc::fcntl(raw_fd, libc::F_SETFL, flags);
                }
            }
        }
        
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Console::*;
            use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
            
            unsafe {
                // Restore original stdin console mode
                if let Some(mode) = self.saved_stdin_mode {
                    let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
                    if h_stdin != INVALID_HANDLE_VALUE {
                        SetConsoleMode(h_stdin, mode);
                    }
                }
                
                // Restore original stdout console mode
                if let Some(mode) = self.saved_stdout_mode {
                    let h_stdout = GetStdHandle(STD_OUTPUT_HANDLE);
                    if h_stdout != INVALID_HANDLE_VALUE {
                        SetConsoleMode(h_stdout, mode);
                    }
                }
            }
        }
    }
}