use std::collections::BTreeSet;

use crate::AgentWrapperBackend;

use super::support::backend_with_env;

#[test]
fn gemini_backend_advertises_only_the_end_to_end_capabilities_it_currently_honors() {
    let backend = backend_with_env(Default::default());

    let ids = backend.capabilities().ids;
    assert_eq!(
        ids,
        BTreeSet::from([
            "agent_api.run".to_string(),
            "agent_api.events".to_string(),
            "agent_api.events.live".to_string(),
            "agent_api.config.model.v1".to_string(),
        ])
    );
    assert!(!ids.contains("agent_api.session.resume.v1"));
    assert!(!ids.contains("agent_api.session.fork.v1"));
}
