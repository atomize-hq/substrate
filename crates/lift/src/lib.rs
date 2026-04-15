#![forbid(unsafe_code)]

#[cfg(test)]
use assert_cmd as _;
#[cfg(test)]
use predicates as _;

pub mod error;

#[cfg(feature = "cli")]
pub use cli::run as run_cli;

#[cfg(feature = "cli")]
pub(crate) mod cli;

pub(crate) mod api;
pub(crate) mod compat;
pub(crate) mod core;
pub(crate) mod detect;
pub(crate) mod graph;
pub(crate) mod languages;
pub(crate) mod policy;
pub(crate) mod repo;
pub(crate) mod resolve;
pub(crate) mod runner;
