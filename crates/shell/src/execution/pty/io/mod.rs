mod manager;
mod reader;
mod runner;
mod types;

pub use runner::execute_with_pty;
#[cfg(any(unix, test))]
#[allow(unused_imports)]
pub(crate) use types::verify_process_group;
pub use types::PtyExitStatus;
#[allow(unused_imports)]
pub(crate) use types::{get_terminal_size, MinimalTerminalGuard, PtyActiveGuard};

#[cfg(windows)]
pub(crate) use manager::{sleep_input_gate, wake_input_gate};
