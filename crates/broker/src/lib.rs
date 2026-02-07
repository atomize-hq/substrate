//! Broker crate entrypoint for policy evaluation, approvals, and profile detection.

mod api;
mod broker;
mod effective_policy;
mod handle;
mod mode;

mod approval;
mod policy;
mod profile;
mod watcher;

#[cfg(test)]
mod test_utils;

pub use api::{
    allowed_domains, detect_profile, evaluate, init, policy_mode, quick_check, reload_policy,
    resolve_effective_policy_with_explain, set_global_broker, set_observe_only, set_policy_mode,
    world_fs_mode, world_fs_policy,
};
pub use approval::{ApprovalCache, ApprovalContext, ApprovalStatus};
pub use broker::Broker;
pub use effective_policy::{EffectivePolicySources, PolicyExplainV1};
pub use handle::BrokerHandle;
pub use mode::PolicyMode;
pub use policy::{
    Decision, Policy, Restriction, RestrictionType, WorldFsDenyEnforcement, WorldFsDimensionPolicy,
    WorldFsEnforcement, WorldFsIsolation, WorldFsPolicy,
};
pub use profile::ProfileDetector;
#[cfg(any(test, feature = "policy-watcher"))]
pub use watcher::{spawn_policy_watcher, MultiPolicyWatcher, PolicyWatcher};

#[cfg(test)]
pub(crate) use broker::matches_pattern;

#[cfg(test)]
mod tests;
