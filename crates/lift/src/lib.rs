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

pub(crate) mod app;
pub(crate) mod derive;
pub(crate) mod export;
pub(crate) mod facts;
pub(crate) mod graph;
pub(crate) mod kernel;
pub(crate) mod lang;
pub(crate) mod pack;
pub(crate) mod patch;
pub(crate) mod query;
pub(crate) mod repo;
pub(crate) mod topo;
