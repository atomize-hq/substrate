#![deny(unsafe_op_in_unsafe_fn)]

pub mod ambient;
pub mod authority;
pub mod contract;
pub mod engine;
pub mod experiment;
pub mod fixed_inbox;
pub mod frame;
pub mod journal;
#[cfg(target_os = "macos")]
pub mod native_effects;
pub mod targets;

#[cfg(target_os = "macos")]
mod disposable_capability;

#[cfg(target_os = "macos")]
pub mod darwin;

pub mod frozen_identity {
    include!(concat!(env!("OUT_DIR"), "/frozen_identity.rs"));
}
