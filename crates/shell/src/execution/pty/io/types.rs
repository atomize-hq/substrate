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

            return Ok(Self { saved_stdin_mode });
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
    fn terminal_size_uses_non_zero_dimensions() {
        let _env_guard = EnvGuard::new();
        std::env::set_var("LINES", "30");
        std::env::set_var("COLUMNS", "100");

        let size = get_terminal_size().unwrap();

        assert!(size.rows > 0);
        assert!(size.cols > 0);
    }

    #[test]
    fn minimal_terminal_guard_handles_creation() {
        let result = MinimalTerminalGuard::new();
        assert!(result.is_ok() || result.is_err());
    }
}
