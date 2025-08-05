#[cfg(unix)]
pub use unix_pty::*;

#[cfg(unix)]
mod unix_pty {
    use anyhow::Result;
    use nix::unistd::{close, dup2, execvp, fork, ForkResult};
    use nix::sys::wait::{waitpid, WaitStatus};
    use nix::sys::termios::{tcgetattr, tcsetattr, SetArg, Termios};
    use std::ffi::CString;
    use std::io::{self, Read, Write};
    use std::os::unix::io::{AsRawFd, RawFd, FromRawFd};
    use std::os::fd::BorrowedFd;
    use crate::ShellConfig;

    pub struct PtySession {
        pub master: RawFd,
        pub child_pid: nix::unistd::Pid,
    }

    pub fn spawn_pty_shell(shell: &str) -> Result<PtySession> {
        // Use libc directly for PTY operations
        let mut master: RawFd = -1;
        let mut slave: RawFd = -1;
        
        unsafe {
            if libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()) != 0 {
                return Err(anyhow::anyhow!("Failed to open PTY"));
            }
        }

        match unsafe { fork() }? {
            ForkResult::Parent { child } => {
                close(slave)?;
                Ok(PtySession {
                    master,
                    child_pid: child,
                })
            }
            ForkResult::Child => {
                // Child process: set up PTY and exec shell
                close(master)?;

                // Make slave the controlling terminal
                dup2(slave, 0)?; // stdin
                dup2(slave, 1)?; // stdout
                dup2(slave, 2)?; // stderr
                close(slave)?;

                // Set up command logging for interactive bash
                let shell_name = std::path::Path::new(shell)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                
                if shell_name == "bash" || shell_name == "bash.exe" {
                    if let Ok(home) = std::env::var("HOME") {
                        let preexec_path = format!("{}/.substrate_preexec", home);
                        
                        // Create the preexec file
                        let _ = std::fs::write(&preexec_path, crate::BASH_PREEXEC_SCRIPT);
                        
                        // Execute bash with custom rcfile for interactive mode
                        let shell_cstr = CString::new(shell)?;
                        let i_flag = CString::new("-i")?;
                        let rcfile_flag = CString::new("--rcfile")?;
                        let rcfile_path = CString::new(preexec_path.as_str())?;
                        let args = vec![shell_cstr.clone(), i_flag, rcfile_flag, rcfile_path];
                        execvp(&shell_cstr, &args)?;
                    }
                }

                // If HOME missing or non-bash, execute the shell normally (no rcfile trap)
                let shell_cstr = CString::new(shell)?;
                let args = vec![shell_cstr.clone()];
                execvp(&shell_cstr, &args)?;

                unreachable!("execvp should not return");
            }
        }
    }
    

    pub fn handle_pty_io(session: PtySession, _config: &ShellConfig) -> Result<()> {
        use std::thread;
        use std::sync::mpsc;
        use nix::poll::{poll, PollFd, PollFlags};
        use nix::sys::signal::{signal, SigHandler, Signal};
        use std::sync::atomic::{AtomicBool, Ordering};

        // Save and configure terminal
        let stdin_fd = io::stdin().as_raw_fd();
        let stdout_fd = io::stdout().as_raw_fd();
        let original_termios = unsafe {
            let borrowed_fd = BorrowedFd::borrow_raw(stdin_fd);
            tcgetattr(borrowed_fd)?
        };
        
        // Set raw mode
        let mut raw_termios = original_termios.clone();
        nix::sys::termios::cfmakeraw(&mut raw_termios);
        unsafe {
            let borrowed_fd = BorrowedFd::borrow_raw(stdin_fd);
            tcsetattr(borrowed_fd, SetArg::TCSANOW, &raw_termios)?;
        }

        // Restore terminal on exit
        let _guard = TerminalGuard { fd: stdin_fd, termios: original_termios };
        
        // Set up SIGWINCH handler for window resize
        static WINCH_RECEIVED: AtomicBool = AtomicBool::new(false);
        extern "C" fn handle_winch(_: i32) {
            WINCH_RECEIVED.store(true, Ordering::Relaxed);
        }
        unsafe {
            signal(Signal::SIGWINCH, SigHandler::Handler(handle_winch))?;
        }
        
        // Function to update PTY window size
        let update_pty_size = |master_fd: RawFd| -> Result<()> {
            unsafe {
                let mut ws: libc::winsize = std::mem::zeroed();
                if libc::ioctl(stdout_fd, libc::TIOCGWINSZ, &mut ws as *mut _) == 0 {
                    libc::ioctl(master_fd, libc::TIOCSWINSZ, &ws as *const _);
                }
            }
            Ok(())
        };
        
        // Set initial window size
        update_pty_size(session.master)?;

        // Spawn thread to copy stdin to PTY
        let (tx, rx) = mpsc::channel();
        let master_fd = session.master;
        
        thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = [0u8; 1024];
            
            loop {
                match stdin.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buffer[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Main loop: copy PTY output to stdout and handle input
        let mut stdout = io::stdout();
        let mut pty_file = unsafe { std::fs::File::from_raw_fd(master_fd) };
        let mut buffer = [0u8; 4096];

        loop {
            // Check for window resize
            if WINCH_RECEIVED.swap(false, Ordering::Relaxed) {
                update_pty_size(session.master)?;
            }
            
            // Check for input from stdin thread
            if let Ok(data) = rx.try_recv() {
                pty_file.write_all(&data)?;
                pty_file.flush()?;
            }

            // Poll for PTY output - use BorrowedFd for nix 0.29
            unsafe {
                let borrowed_fd = BorrowedFd::borrow_raw(master_fd);
                let mut poll_fds = [PollFd::new(borrowed_fd, PollFlags::POLLIN)];
                if poll(&mut poll_fds, nix::poll::PollTimeout::from(10u8))? > 0 {
                    if poll_fds[0].revents().unwrap().contains(PollFlags::POLLIN) {
                        match pty_file.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                stdout.write_all(&buffer[..n])?;
                                stdout.flush()?;
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                            Err(_) => break,
                        }
                    }
                }
            }

            // Check if child process has exited
            match waitpid(session.child_pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG))? {
                WaitStatus::Exited(_, _) | WaitStatus::Signaled(_, _, _) => break,
                _ => continue,
            }
        }

        Ok(())
    }

    struct TerminalGuard {
        fd: RawFd,
        termios: Termios,
    }

    impl Drop for TerminalGuard {
        fn drop(&mut self) {
            unsafe {
                let borrowed_fd = BorrowedFd::borrow_raw(self.fd);
                let _ = tcsetattr(borrowed_fd, SetArg::TCSANOW, &self.termios);
            }
        }
    }
}