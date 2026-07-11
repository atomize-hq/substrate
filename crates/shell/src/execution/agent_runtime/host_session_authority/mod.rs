//! A1 host-session authority primitives.
//!
//! Submodules land in the fixed A1.1a-A1.1e order. This module does not expose
//! production authority operations until the complete A1.1 proof wall passes.

// A1.1a lands these primitives before A1.1c-A1.1e adopt them. Keep the staging
// allowance at the module boundary and remove it when the facade is integrated.
#[allow(dead_code)]
pub(crate) mod canonical_json;
#[allow(dead_code)]
pub(crate) mod hash;
#[allow(dead_code)]
pub(crate) mod schema;
#[allow(dead_code)]
pub(crate) mod trusted_fs;
#[allow(dead_code)]
pub(crate) mod validation;

#[cfg(test)]
mod golden_vectors;
