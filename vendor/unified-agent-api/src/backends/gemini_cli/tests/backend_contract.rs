use crate::{backend_harness::BackendHarnessAdapter, AgentWrapperBackend};

use super::support::backend_with_env;

#[test]
fn gemini_adapter_implements_backend_harness_adapter_contract() {
    fn assert_impl<T: BackendHarnessAdapter>() {}
    assert_impl::<crate::backends::gemini_cli::GeminiCliBackend>();
}

#[test]
fn gemini_backend_advertises_the_minimal_headless_capabilities() {
    let backend = backend_with_env(Default::default());

    assert_eq!(
        backend.capabilities().ids,
        std::collections::BTreeSet::from([
            "agent_api.run".to_string(),
            "agent_api.events".to_string(),
            "agent_api.events.live".to_string(),
            "agent_api.config.model.v1".to_string(),
        ])
    );
    assert_eq!(
        backend.supported_extension_keys(),
        &["agent_api.config.model.v1"]
    );
}
