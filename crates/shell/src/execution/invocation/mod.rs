//! Invocation planning and execution.

mod plan;
mod runtime;

#[cfg(test)]
mod tests;

pub use plan::{needs_shell, ShellConfig, ShellMode};
pub(crate) use runtime::{run_interactive_shell, run_pipe_mode, run_script_mode, run_wrap_mode};
