//! Builtin command handling for shell routing.

mod shim_actions;
mod utility;
mod world_deps;

pub(crate) use utility::handle_builtin;

#[cfg(test)]
mod tests;
