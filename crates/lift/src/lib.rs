#![forbid(unsafe_code)]

#[cfg(test)]
use assert_cmd as _;
#[cfg(test)]
use jsonschema as _;
#[cfg(test)]
use predicates as _;
#[cfg(test)]
use serde as _;
#[cfg(test)]
use serde_jcs as _;
use serde_json as _;
#[cfg(test)]
use sha2 as _;
#[cfg(test)]
use thiserror as _;

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
pub mod kernel;
pub(crate) mod lang;
pub(crate) mod pack;
pub(crate) mod patch;
pub(crate) mod query;
pub(crate) mod repo;
pub(crate) mod topo;
