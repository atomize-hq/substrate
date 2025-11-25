//! Broker crate entrypoint for policy evaluation, approvals, and profile detection.

mod api;
mod broker;
mod handle;
mod policy_loader;

mod approval;
mod policy;
mod profile;
mod watcher;

pub use api::{
    allowed_domains, detect_profile, evaluate, init, quick_check, reload_policy, set_global_broker,
    set_observe_only,
};
pub use approval::{ApprovalCache, ApprovalContext, ApprovalStatus};
pub use broker::Broker;
pub use handle::BrokerHandle;
pub use policy::{Decision, Policy, Restriction, RestrictionType};
pub use profile::ProfileDetector;
#[cfg(any(test, feature = "policy-watcher"))]
pub use watcher::{spawn_policy_watcher, MultiPolicyWatcher, PolicyWatcher};

#[cfg(test)]
pub(crate) use broker::matches_pattern;

#[cfg(test)]
mod tests;
