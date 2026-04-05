//! PTY reader loop and process completion handling.

use super::super::control::PtyControl;
use super::types::PtyExitStatus;
use anyhow::Result;
use std::env;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn should_forward_stdin() -> bool {
    if let Ok(value) = env::var("SUBSTRATE_PTY_FORWARD_STDIN") {
        let value = value.to_ascii_lowercase();
        return matches!(value.as_str(), "1" | "true" | "yes" | "on");
    }

    // In CI, PTY harnesses like `script` often pre-buffer the entire input stream. Forwarding
    // stdin during a `:pty <command>` run can consume subsequent REPL commands (like `exit`),
    // leaving the parent session blocked at the next prompt.
    if env::var_os("CI").is_some() || env::var_os("GITHUB_ACTIONS").is_some() {
        return false;
    }

    true
}

pub(crate) fn handle_pty_io(
    control: PtyControl,
    mut reader: Box<dyn Read + Send>,
    child: &mut Box<dyn portable_pty::Child + Send + Sync>,
    cmd_id: &str,
    manager_handle: thread::JoinHandle<()>,
) -> Result<PtyExitStatus> {
    let done = Arc::new(AtomicBool::new(false));

    #[cfg(unix)]
    let stdin_join: Option<thread::JoinHandle<()>> = {
        if !should_forward_stdin() {
            None
        } else {
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
                        Ok(_) if read_fds.contains(stdin_borrowed) => match stdin.read(&mut buffer)
                        {
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
        }
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

    // Ensure the PTY master is closed before joining the output thread so a blocking read can
    // observe EOF and terminate.
    control.close();
    let _ = manager_handle.join();

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

    Ok(PtyExitStatus::from_portable_pty(portable_status))
}
