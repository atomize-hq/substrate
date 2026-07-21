//! PTY state helpers and terminal guards.

use crate::execution::PTY_ACTIVE;
use anyhow::Result;
use portable_pty::PtySize;
#[cfg(unix)]
use std::io;
use std::sync::atomic::Ordering;

/// Custom exit status for PTY commands
#[derive(Debug, Clone)]
pub struct PtyExitStatus {
    pub code: Option<i32>,
    pub signal: Option<i32>,
}

impl PtyExitStatus {
    pub(crate) fn from_portable_pty(status: portable_pty::ExitStatus) -> Self {
        #[cfg(unix)]
        {
            let raw = status.exit_code() as i32;
            if raw > 128 {
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

// RAII guard to ensure PTY_ACTIVE flag is cleared even on panic
pub(crate) struct PtyActiveGuard;

impl Drop for PtyActiveGuard {
    fn drop(&mut self) {
        PTY_ACTIVE.store(false, Ordering::SeqCst);
    }
}

// Minimal terminal guard - ONLY sets stdin to raw mode for input forwarding
// Does NOT touch stdout to avoid display corruption
pub(crate) struct MinimalTerminalGuard {
    #[cfg(unix)]
    saved_termios: Option<nix::sys::termios::Termios>,
    #[cfg(windows)]
    saved_stdin_mode: Option<u32>,
}

impl MinimalTerminalGuard {
    pub(crate) fn new() -> Result<Self> {
        #[cfg(unix)]
        {
            use nix::sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg};
            use std::os::unix::io::{AsRawFd, BorrowedFd};

            let raw_fd = io::stdin().as_raw_fd();
            let fd = unsafe { BorrowedFd::borrow_raw(raw_fd) };
            let saved_termios = tcgetattr(fd).ok();

            if let Some(ref orig) = saved_termios {
                let mut raw = orig.clone();
                cfmakeraw(&mut raw);

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
                let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
                if h_stdin != INVALID_HANDLE_VALUE {
                    let mut mode = 0;
                    if GetConsoleMode(h_stdin, &mut mode) != 0 {
                        saved_stdin_mode = Some(mode);

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

            if let Some(mode) = self.saved_stdin_mode {
                unsafe {
                    let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
                    if h_stdin != INVALID_HANDLE_VALUE {
                        let _ = SetConsoleMode(h_stdin, mode);
                    }
                }
            }
        }
    }
}

#[cfg(windows)]
pub(crate) fn windows_console_size() -> Option<PtySize> {
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

pub(crate) fn get_terminal_size() -> Result<PtySize> {
    #[cfg(windows)]
    if let Some(sz) = windows_console_size() {
        return Ok(sz);
    }

    #[cfg(unix)]
    {
        use libc::{ioctl, winsize, TIOCGWINSZ};
        use std::mem;

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

    let rows = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120);

    Ok(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })
}

#[cfg(unix)]
pub(crate) fn verify_process_group(pid: Option<u32>) {
    if let Some(pid) = pid {
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(["-o", "pid,pgid,tpgid,stat", "-p", &pid.to_string()])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            log::debug!("Process group info for {pid}: {output_str}");
        }
    }
}

#[cfg(not(unix))]
#[allow(dead_code)]
pub(crate) fn verify_process_group(_pid: Option<u32>) {
    // No-op on non-Unix platforms
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::atomic::Ordering;

    struct EnvGuard {
        lines: Option<String>,
        cols: Option<String>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                lines: std::env::var("LINES").ok(),
                cols: std::env::var("COLUMNS").ok(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(lines) = self.lines.take() {
                std::env::set_var("LINES", lines);
            } else {
                std::env::remove_var("LINES");
            }

            if let Some(cols) = self.cols.take() {
                std::env::set_var("COLUMNS", cols);
            } else {
                std::env::remove_var("COLUMNS");
            }
        }
    }

    #[test]
    fn exit_status_reports_success_and_code() {
        let success = PtyExitStatus {
            code: Some(0),
            signal: None,
        };
        assert!(success.success());
        assert_eq!(success.code(), Some(0));

        let failure = PtyExitStatus {
            code: Some(1),
            signal: None,
        };
        assert!(!failure.success());
        assert_eq!(failure.code(), Some(1));
    }

    #[cfg(unix)]
    #[test]
    fn exit_status_reports_signal_on_unix() {
        let status = PtyExitStatus {
            code: None,
            signal: Some(9),
        };

        assert!(!status.success());
        assert_eq!(status.signal(), Some(9));
    }

    #[test]
    fn active_guard_resets_flag_on_drop() {
        if crate::execution::run_in_bounded_test_subprocess(
            concat!(
                module_path!(),
                "::",
                stringify!(active_guard_resets_flag_on_drop)
            ),
            "pty_active",
        ) {
            return;
        }
        let previous = PTY_ACTIVE.swap(true, Ordering::SeqCst);

        {
            let _guard = PtyActiveGuard;
            assert!(PTY_ACTIVE.load(Ordering::SeqCst));
        }

        assert!(
            !PTY_ACTIVE.load(Ordering::SeqCst),
            "guard clears PTY_ACTIVE when dropped"
        );

        PTY_ACTIVE.store(previous, Ordering::SeqCst);
    }

    #[test]
    #[serial]
    fn terminal_size_uses_non_zero_dimensions() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let _env_guard = EnvGuard::new();
        std::env::set_var("LINES", "30");
        std::env::set_var("COLUMNS", "100");

        let size = get_terminal_size().unwrap();

        assert!(size.rows > 0);
        assert!(size.cols > 0);
    }

    #[test]
    fn minimal_terminal_guard_handles_creation() {
        #[cfg(unix)]
        {
            use crate::execution::TestSubprocessExpectation;
            use nix::sys::termios::{tcgetattr, LocalFlags};
            use std::os::fd::{AsRawFd, BorrowedFd};
            use std::process::Stdio;

            let test_name = concat!(
                module_path!(),
                "::",
                stringify!(minimal_terminal_guard_handles_creation)
            );
            for role in ["ordinary", "panic", "abort", "timeout"] {
                if crate::execution::is_bounded_test_subprocess_child(test_name, role) {
                    let stdin_fd = std::io::stdin().as_raw_fd();
                    let stdin_fd = unsafe { BorrowedFd::borrow_raw(stdin_fd) };
                    let termios_before = tcgetattr(stdin_fd).expect("read isolated PTY mode");
                    let guard = MinimalTerminalGuard::new()
                        .expect("create terminal guard over isolated PTY slave");
                    let termios_during = tcgetattr(stdin_fd).expect("read isolated raw PTY mode");
                    assert!(
                        !termios_during
                            .local_flags
                            .intersects(LocalFlags::ECHO | LocalFlags::ICANON | LocalFlags::ISIG),
                        "isolated terminal guard must execute the raw-mode mutation"
                    );
                    crate::execution::publish_test_subprocess_state_ready(test_name, role);
                    match role {
                        "ordinary" => {
                            drop(guard);
                            assert_eq!(
                                tcgetattr(stdin_fd).expect("read restored isolated PTY mode"),
                                termios_before,
                                "isolated terminal guard must restore the exact PTY mode"
                            );
                            return;
                        }
                        "panic" => panic!("intentional isolated terminal-mode panic"),
                        "abort" => std::process::abort(),
                        "timeout" => loop {
                            std::thread::park();
                        },
                        _ => unreachable!(),
                    }
                }
            }

            let parent_fd = std::io::stdin().as_raw_fd();
            let parent_fd = unsafe { BorrowedFd::borrow_raw(parent_fd) };
            let parent_mode = tcgetattr(parent_fd).ok();
            let roles = [
                ("ordinary", TestSubprocessExpectation::Success),
                ("panic", TestSubprocessExpectation::ExitCode(101)),
                ("abort", TestSubprocessExpectation::Signal(libc::SIGABRT)),
                (
                    "timeout",
                    TestSubprocessExpectation::Timeout(std::time::Duration::from_secs(1)),
                ),
            ];
            for (role, expectation) in roles {
                let nix::pty::OpenptyResult { master, slave } =
                    nix::pty::openpty(None, None).expect("open isolated PTY pair");
                let _master = master;
                crate::execution::run_bounded_test_subprocess_role(
                    test_name,
                    role,
                    Stdio::from(std::fs::File::from(slave)),
                    expectation,
                    true,
                );
                assert_eq!(
                    tcgetattr(parent_fd).ok(),
                    parent_mode,
                    "isolated terminal-mode role changed the parent terminal: {role}"
                );
            }
        }

        #[cfg(windows)]
        {
            use crate::execution::TestSubprocessExpectation;
            use std::os::windows::io::AsRawHandle;
            use std::process::Stdio;
            use windows_sys::Win32::System::Console::{
                GetConsoleMode, GetStdHandle, SetStdHandle, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
                ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_INPUT, STD_INPUT_HANDLE,
            };

            let test_name = concat!(
                module_path!(),
                "::",
                stringify!(minimal_terminal_guard_handles_creation)
            );
            for role in ["ordinary", "panic", "abort", "timeout"] {
                if crate::execution::is_bounded_test_subprocess_child(test_name, role) {
                    let console_input = std::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open("CONIN$")
                        .expect("open isolated Windows console input");
                    let console_input_handle = console_input.as_raw_handle();
                    assert_ne!(
                        unsafe { SetStdHandle(STD_INPUT_HANDLE, console_input_handle) },
                        0,
                        "install isolated Windows console input"
                    );
                    let mut mode_before = 0;
                    assert_ne!(
                        unsafe { GetConsoleMode(console_input_handle, &mut mode_before) },
                        0,
                        "read isolated Windows console mode"
                    );
                    let guard = MinimalTerminalGuard::new()
                        .expect("create terminal guard over isolated Windows console");
                    let mut mode_during = 0;
                    assert_ne!(
                        unsafe { GetConsoleMode(console_input_handle, &mut mode_during) },
                        0,
                        "read isolated raw Windows console mode"
                    );
                    assert_eq!(
                        mode_during
                            & (ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT),
                        0,
                        "isolated Windows terminal guard must disable cooked input modes"
                    );
                    assert_ne!(
                        mode_during & ENABLE_VIRTUAL_TERMINAL_INPUT,
                        0,
                        "isolated Windows terminal guard must enable virtual terminal input"
                    );
                    crate::execution::publish_test_subprocess_state_ready(test_name, role);
                    match role {
                        "ordinary" => {
                            drop(guard);
                            let mut mode_after = 0;
                            assert_ne!(
                                unsafe { GetConsoleMode(console_input_handle, &mut mode_after) },
                                0,
                                "read restored isolated Windows console mode"
                            );
                            assert_eq!(mode_after, mode_before);
                            return;
                        }
                        "panic" => panic!("intentional isolated Windows terminal-mode panic"),
                        "abort" => std::process::abort(),
                        "timeout" => loop {
                            std::thread::park();
                        },
                        _ => unreachable!(),
                    }
                }
            }

            let parent_stdin = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
            let mut parent_mode_before = 0;
            let parent_has_console =
                unsafe { GetConsoleMode(parent_stdin, &mut parent_mode_before) } != 0;
            let roles = [
                ("ordinary", TestSubprocessExpectation::Success),
                ("panic", TestSubprocessExpectation::ExitCode(101)),
                ("abort", TestSubprocessExpectation::Failure),
                (
                    "timeout",
                    TestSubprocessExpectation::Timeout(std::time::Duration::from_secs(1)),
                ),
            ];
            for (role, expectation) in roles {
                crate::execution::run_bounded_test_subprocess_role(
                    test_name,
                    role,
                    Stdio::null(),
                    expectation,
                    true,
                );
                if parent_has_console {
                    let mut parent_mode_after = 0;
                    assert_ne!(
                        unsafe { GetConsoleMode(parent_stdin, &mut parent_mode_after) },
                        0,
                        "read parent Windows console mode after isolated child"
                    );
                    assert_eq!(
                        parent_mode_after, parent_mode_before,
                        "isolated Windows terminal-mode role changed the parent console: {role}"
                    );
                }
            }
            return;
        }

        #[cfg(not(any(unix, windows)))]
        {
            let result = MinimalTerminalGuard::new();
            assert!(result.is_ok() || result.is_err());
        }
    }
}
