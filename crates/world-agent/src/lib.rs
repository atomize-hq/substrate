//! World agent library for execution inside worlds/VMs.

pub mod gc;
pub mod handlers;
pub mod pty;
pub mod service;

pub use service::WorldAgentService;
