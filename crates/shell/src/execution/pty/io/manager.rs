//! PTY writer/manager loop and Windows input forwarding helpers.

#[cfg(windows)]
use super::super::control::active_pty_control;
#[cfg(windows)]
use super::super::control::WIN_PTY_INPUT_GATE;
use super::super::control::{PtyCommand, PtyControl};
use anyhow::{Context, Result};
use portable_pty::MasterPty;
#[cfg(windows)]
use std::io;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

pub(crate) fn start_pty_manager(
    master: Box<dyn MasterPty + Send>,
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

pub(crate) fn run_pty_manager(
    master: Box<dyn MasterPty + Send>,
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
