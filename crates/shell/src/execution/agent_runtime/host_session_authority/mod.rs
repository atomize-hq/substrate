//! A1 host-session authority primitives.
//!
//! Submodules land in the fixed A1.1a-A1.1e order. This module does not expose
//! production authority operations until the complete A1.1 proof wall passes.

// A1.1a lands these primitives before A1.1c-A1.1e adopt them. Keep the staging
// allowance at the module boundary and remove it when the facade is integrated.
#[allow(dead_code)]
pub(crate) mod canonical_json;
#[allow(dead_code)]
pub(crate) mod facade;
#[allow(dead_code)]
pub(crate) mod hash;
#[allow(dead_code)]
pub(crate) mod schema;
#[allow(dead_code)]
pub(crate) mod store;
#[allow(dead_code)]
pub(crate) mod store_format;
#[allow(dead_code)]
pub(crate) mod store_schema;
#[allow(dead_code)]
pub(crate) mod transition;
#[allow(dead_code)]
pub(crate) mod trusted_fs;
#[allow(dead_code)]
pub(crate) mod validation;

#[allow(
    unused_imports,
    reason = "A1.2a lands the bounded read contract before A1.2a-S adopts it"
)]
pub(crate) use facade::{
    AuthorityObservationV1, AuthorityParticipantRoleV1, HostSessionAuthority,
    OpenedBootstrapHomeV1, ResolvedAuthorityCallerV1, ResolvedCurrentAuthorityV1,
    ResolvedSessionAuthorityV1,
};

#[cfg(test)]
mod golden_vectors;
#[cfg(test)]
mod transition_tests;
