//! Admission of NEW ordinary work population at the existing owner locks.
//! Original selection identity is insufficient after its controller settles.

use super::super::{
    facade::{validate_locked_post_hsa_authority_binding, AuthorityObservationV1},
    schema::{HostSessionPostureV1, HostSessionTransitionModeV1},
    store::{self, BootstrapError},
    store_schema::{
        HostSessionTransitionIntentStateV2, HostSessionTransitionIntentStateV3,
        SessionNamespaceRecordV1, VersionedStateRoot,
    },
};
use crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1;
use anyhow::{ensure, Result};

/// No serialization or caller-field constructor. This binds a genuine current
/// run observation to its exact locked preimage. It grants no World send and is
/// unnecessary for progress/retry of already-captured independent work.
pub(crate) struct AuthenticatedOrdinaryWorldPublicationV1 {
    observation: AuthorityObservationV1,
    root_bytes: Vec<u8>,
    participant: String,
    intent_id: String,
    run_id: String,
    mode: HostSessionTransitionModeV1,
}

pub(crate) fn authenticate_ordinary_world_publication(
    selection: &AuthenticatedOrdinarySelectionV1,
) -> Result<AuthenticatedOrdinaryWorldPublicationV1> {
    use transport_api_types::AgentSelectionPlacementV1;
    selection.require_ordinary_configured_authority()?;
    selection.revalidate()?;
    ensure!(
        selection.native_selection().placement == AgentSelectionPlacementV1::Host
            && selection.requested_selection().placement == AgentSelectionPlacementV1::World,
        "new ordinary World work requires its genuine local controller roles"
    );
    let authority = selection.held_authority();
    let current = authority.resolve_current_exact(selection.original_session_id(), None)?;
    ensure!(
        current.caller.participant_id == selection.current_participant_id()
            && current.authority.lifecycle_posture == HostSessionPostureV1::ActiveAttached,
        "new ordinary World work requires the actual active controller"
    );
    // Source capture must never migrate or otherwise publish authority state.
    let root = store::read_opened_root_v2_or_v3(authority.trusted_root())?;
    ensure!(
        current.observation.root_revision == root.root_revision(),
        "World publication authority moved during current-run capture"
    );
    let original = match &root {
        VersionedStateRoot::V2(root) => root
            .transition_intent_map
            .get(selection.original_start_intent_id()),
        VersionedStateRoot::V3(root) => root
            .transition_intent_map
            .get(selection.original_start_intent_id()),
        VersionedStateRoot::V1(_) => None,
    }
    .ok_or_else(|| anyhow::anyhow!("World publication original Start absent"))?;
    let (intent_id, run_id) = if original.target_authoritative_participant_id
        == current.caller.participant_id
    {
        (original.intent_id.clone(), original.run_id.clone())
    } else {
        let VersionedStateRoot::V3(root) = &root else {
            anyhow::bail!("World publication current successor source absent");
        };
        let candidates = root
            .successor_transition_intent_map
            .values()
            .filter(|intent| {
                intent.orchestration_session_id == current.authority.orchestration_session_id
                    && intent.target_authoritative_participant_id == current.caller.participant_id
                    && matches!(
                        intent.state,
                        HostSessionTransitionIntentStateV3::Applied { .. }
                    )
            })
            .collect::<Vec<_>>();
        let [intent] = candidates.as_slice() else {
            anyhow::bail!("World publication current run is absent or ambiguous");
        };
        (intent.intent_id.clone(), intent.run_id.clone())
    };
    let observed = authority.observe_current_host_gateway_run_v1(
        selection.original_session_id(),
        &current.caller.participant_id,
        &selection.native_selection().backend_id,
        &run_id,
    )?;
    let outcome = (|| {
        let run = observed.current_run();
        ensure!(
            run.intent_id() == intent_id
                && run.run_id() == run_id
                && run.original_start_intent_id() == selection.original_start_intent_id()
                && run.authority_store_id() == selection.authority_store_id()
                && matches!(
                    run.transition_mode(),
                    HostSessionTransitionModeV1::Start | HostSessionTransitionModeV1::ResumeOneTurn
                ),
            "World publication current issued run differs"
        );
        let fresh = authority.resolve_current_exact(selection.original_session_id(), None)?;
        ensure!(
            fresh.observation == current.observation && fresh.caller == current.caller,
            "World publication current authority changed"
        );
        Ok(AuthenticatedOrdinaryWorldPublicationV1 {
            observation: current.observation,
            root_bytes: root.to_canonical_bytes()?,
            participant: current.caller.participant_id,
            intent_id,
            run_id,
            mode: run.transition_mode(),
        })
    })();
    // Finalizers run on refusals too, without carrying a lock across IPC.
    let current_exit = observed.revalidate();
    let source_exit = selection.revalidate();
    current_exit?;
    source_exit?;
    outcome
}

impl AuthenticatedOrdinaryWorldPublicationV1 {
    pub(crate) fn verify_locked_ordinary_world_publication(
        &self,
        root: &VersionedStateRoot,
    ) -> Result<(), BootstrapError> {
        let refusal = || BootstrapError::world_controller_source_guard();
        validate_locked_post_hsa_authority_binding(root, &self.observation, None)
            .map_err(|_| refusal())?;
        if root.to_canonical_bytes().map_err(|_| refusal())? != self.root_bytes {
            return Err(refusal());
        }
        let (sessions, starts) = match root {
            VersionedStateRoot::V2(root) => {
                (&root.session_namespace_map, &root.transition_intent_map)
            }
            VersionedStateRoot::V3(root) => {
                (&root.session_namespace_map, &root.transition_intent_map)
            }
            VersionedStateRoot::V1(_) => return Err(refusal()),
        };
        let Some(SessionNamespaceRecordV1::Authority(authority)) =
            sessions.get(&self.observation.orchestration_session_id)
        else {
            return Err(refusal());
        };
        if authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
            || authority.active_authoritative_participant_id.as_ref() != Some(&self.participant)
        {
            return Err(refusal());
        }
        let active = match self.mode {
            HostSessionTransitionModeV1::Start => {
                starts.get(&self.intent_id).is_some_and(|intent| {
                    intent.run_id == self.run_id
                        && intent.target_authoritative_participant_id == self.participant
                        && matches!(
                            intent.state,
                            HostSessionTransitionIntentStateV2::Applied { .. }
                        )
                })
            }
            HostSessionTransitionModeV1::ResumeOneTurn => match root {
                VersionedStateRoot::V3(root) => root
                    .successor_transition_intent_map
                    .get(&self.intent_id)
                    .is_some_and(|intent| {
                        intent.run_id == self.run_id
                            && intent.target_authoritative_participant_id == self.participant
                            && matches!(
                                intent.state,
                                HostSessionTransitionIntentStateV3::Applied { .. }
                            )
                    }),
                _ => false,
            },
            _ => false,
        };
        if !active {
            return Err(refusal());
        }
        Ok(())
    }
}
