//! Command dispatch helpers extracted from routing.

mod exec;
mod prelude;
mod registry;
mod shim_ops;
mod world_ops;

pub(crate) use prelude::*;

#[cfg(test)]
mod tests;
