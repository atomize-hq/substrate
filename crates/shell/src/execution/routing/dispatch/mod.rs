//! Command dispatch helpers extracted from routing.

mod exec;
mod prelude;
mod registry;
mod shim_ops;
mod world_ops;
pub(crate) mod world_persistent_session;

pub(crate) use prelude::*;

#[cfg(test)]
mod tests;
