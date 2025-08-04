//! Substrate command execution shim
//!
//! This library provides command interception and logging capabilities for tracing
//! command execution in development environments, particularly for AI coding assistants
//! like Claude Code.
//!
//! ## Architecture
//!
//! The shim works by:
//! 1. Being copied to multiple command names (git, npm, python, etc.)
//! 2. Resolving the real binary from a clean PATH (excluding shim directory)
//! 3. Executing the real command while logging structured data
//! 4. Providing session correlation for command chains
//!
//! ## Usage
//!
//! ```rust,no_run
//! use substrate_shim::run_shim;
//!
//! fn main() -> anyhow::Result<()> {
//!     let exit_code = run_shim()?;
//!     std::process::exit(exit_code);
//! }
//! ```

pub use context::ShimContext;
pub use exec::run_shim;

mod context;
mod exec;
mod logger;
mod resolver;

// Re-export commonly used types
pub use anyhow::{Context, Result};
pub use std::path::PathBuf;