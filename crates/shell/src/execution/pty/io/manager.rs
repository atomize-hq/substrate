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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use portable_pty::PtySize;
    use std::io::{self, Read, Write};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{mpsc, Arc, Mutex};

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
        fn resize(&self, size: PtySize) -> Result<()> {
            self.sizes.lock().unwrap().push(size);
            Ok(())
        }

        fn get_size(&self) -> Result<PtySize> {
            Ok(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
        }

        fn try_clone_reader(&self) -> Result<Box<dyn Read + Send>> {
            Ok(Box::new(io::empty()))
        }

        fn take_writer(&self) -> Result<Box<dyn Write + Send>> {
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
            Err(io::Error::other("writer failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn processes_resize_write_and_close_commands() {
        let sizes = Arc::new(Mutex::new(Vec::new()));
        let writes = Arc::new(Mutex::new(Vec::new()));
        let flushes = Arc::new(AtomicUsize::new(0));

        let master = MockMasterPty::new(sizes.clone(), writes.clone(), flushes.clone());
        let writer = master.writer();

        let (tx, rx) = mpsc::channel();
        let control = PtyControl { tx };

        let manager =
            std::thread::spawn(move || run_pty_manager(Box::new(master), Box::new(writer), rx));

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

        manager
            .join()
            .expect("pty manager thread panicked while handling commands");

        assert_eq!(sizes.lock().unwrap().as_slice(), &[resize]);
        assert_eq!(writes.lock().unwrap().as_slice(), b"bytes-more");
        assert!(flushes.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn exits_when_channel_sender_drops() {
        let master = MockMasterPty::new(
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(AtomicUsize::new(0)),
        );
        let writer = master.writer();

        let (tx, rx) = mpsc::channel();
        let manager =
            std::thread::spawn(move || run_pty_manager(Box::new(master), Box::new(writer), rx));

        drop(tx);
        manager
            .join()
            .expect("pty manager thread panicked on channel close");
    }

    #[test]
    fn stops_after_write_failure() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let master = MockMasterPty::new(
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(AtomicUsize::new(0)),
        );
        let writer = FailingWriter::new(attempts.clone());

        let (tx, rx) = mpsc::channel();
        let manager =
            std::thread::spawn(move || run_pty_manager(Box::new(master), Box::new(writer), rx));

        let control = PtyControl { tx };
        control.write(b"should-fail".to_vec());
        drop(control);

        manager
            .join()
            .expect("pty manager thread panicked after writer error");
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
