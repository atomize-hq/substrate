use portable_pty::PtySize;
use std::sync::mpsc::Sender;
#[cfg(not(windows))]
use std::sync::Mutex;
#[cfg(windows)]
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use lazy_static::lazy_static;

use super::io::get_terminal_size;

#[derive(Clone)]
pub(crate) struct PtyControl {
    pub(crate) tx: Sender<PtyCommand>,
}

impl PtyControl {
    #[cfg_attr(windows, allow(dead_code))]
    pub(crate) fn resize(&self, size: PtySize) {
        if let Err(err) = self.tx.send(PtyCommand::Resize(size)) {
            log::warn!("Failed to dispatch PTY resize: {err}");
        }
    }

    pub(crate) fn write(&self, data: Vec<u8>) {
        if let Err(err) = self.tx.send(PtyCommand::Write(data)) {
            log::warn!("Failed to dispatch PTY write: {err}");
        }
    }

    pub(crate) fn close(&self) {
        let _ = self.tx.send(PtyCommand::Close);
    }
}

pub(crate) enum PtyCommand {
    #[cfg_attr(windows, allow(dead_code))]
    Resize(PtySize),
    Write(Vec<u8>),
    Close,
}

lazy_static! {
    static ref ACTIVE_PTY: Mutex<Option<PtyControl>> = Mutex::new(None);
}

#[cfg(windows)]
lazy_static! {
    static ref WIN_PTY_INPUT_GATE: Arc<(Mutex<bool>, Condvar)> =
        Arc::new((Mutex::new(false), Condvar::new()));
}

pub(crate) struct ActivePtyGuard;

impl ActivePtyGuard {
    pub(crate) fn register(control: PtyControl) -> Self {
        if let Ok(mut slot) = ACTIVE_PTY.lock() {
            *slot = Some(control);
        }
        #[cfg(windows)]
        wake_input_gate();
        ActivePtyGuard
    }
}

impl Drop for ActivePtyGuard {
    fn drop(&mut self) {
        if let Ok(mut slot) = ACTIVE_PTY.lock() {
            *slot = None;
        }
        #[cfg(windows)]
        sleep_input_gate();
    }
}

pub(crate) fn active_pty_control() -> Option<PtyControl> {
    ACTIVE_PTY.lock().ok().and_then(|slot| slot.clone())
}

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
                if let Some(control) = active_pty_control() {
                    if let Ok(size) = get_terminal_size() {
                        control.resize(size);

                        if std::env::var("SUBSTRATE_PTY_DEBUG").is_ok() {
                            log::debug!("SIGWINCH: Resized PTY to {}x{}", size.cols, size.rows);
                        }
                    }
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use portable_pty::PtySize;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn active_control_dispatches_commands_and_clears_on_drop() {
        let (tx, rx) = mpsc::channel();
        let control = PtyControl { tx };
        let expected_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        {
            let _guard = ActivePtyGuard::register(control.clone());
            let active = active_pty_control().expect("active PTY control is registered");

            active.resize(expected_size);
            active.write(b"payload".to_vec());
            active.close();
            assert!(
                active_pty_control().is_some(),
                "control remains active while guard is held"
            );
        }

        let mut received = Vec::new();
        while let Ok(cmd) = rx.recv_timeout(Duration::from_secs(1)) {
            received.push(cmd);
            if matches!(received.last(), Some(PtyCommand::Close)) {
                break;
            }
        }

        assert!(
            active_pty_control().is_none(),
            "control cleared after guard drop"
        );
        assert!(
            matches!(received.first(), Some(PtyCommand::Resize(size)) if size.rows == expected_size.rows && size.cols == expected_size.cols)
        );
        assert!(matches!(
            received.get(1),
            Some(PtyCommand::Write(bytes)) if bytes == b"payload"
        ));
        assert!(matches!(received.get(2), Some(PtyCommand::Close)));
    }
}
