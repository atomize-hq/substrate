use std::sync::Arc;

use crate::{
    AgentInventorySourceMaterialV1, ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1,
    ConfigProjectionRegistryV1, ConfiguredAcceptedHomeAuthorityV1,
    EffectiveSubstrateConfigSourceV1, Timestamp,
};

pub enum E3PreparationAbandonmentTargetV1<'a> {
    Live(&'a mut E3PreparedRetainedLaunchPublicationV1),
    Recovered(&'a crate::ConfigProjectionPreparationMetadataV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::large_enum_variant,
    reason = "the contract requires owned, unboxed readiness observations"
)]
pub enum E3ManagedGatewayActivationObservationV1 {
    Delivered {
        delivered_at: Timestamp,
    },
    Ready {
        gateway_process_identity: crate::GatewayProcessIdentityV1,
        child_security_attestation: crate::E3ChildSecurityAttestationV1,
        observed_at: Timestamp,
    },
}

#[derive(Default)]
struct E3GatewayActivationPublicationProgressV1 {
    delivered: Option<crate::ConfigProjectionSecretHandoffRevisionV1>,
    delivered_ref: Option<crate::SecretHandoffRefV1>,
    ready_observation: Option<E3ManagedGatewayActivationObservationV1>,
    consumed: Option<crate::ConfigProjectionSecretHandoffRevisionV1>,
    consumed_ref: Option<crate::SecretHandoffRefV1>,
    ack: Option<crate::ManagedGatewayActivationAckV1>,
    ready_record: Option<crate::AgentConfigProjectionRecordV1>,
    ready_ref: Option<crate::ConfigProjectionRefV1>,
}

pub struct AgentConfigProjectionServiceV1 {
    registry: Arc<ConfigProjectionRegistryV1>,
    accepted_home: Arc<ConfiguredAcceptedHomeAuthorityV1>,
}

#[cfg(test)]
pub(crate) fn exercise_activation_progress_for_test(
    service: &AgentConfigProjectionServiceV1,
    mut publication: E3PreparedRetainedLaunchPublicationV1,
    lease: crate::ConfigProjectionConsumerLeaseV1,
    ready: E3ManagedGatewayActivationObservationV1,
) -> crate::ConfigProjectionRefV1 {
    // The fixture has already published the prepared chain and acquired its original lease.
    // Only this colocated test can install those otherwise private prepare-return fields.
    let dormant = publication
        .launch_input
        .as_ref()
        .unwrap()
        .dormant_projection_ref
        .clone();
    publication.published_head = Some(dormant.clone());
    publication.held_consumer_lease = Some(lease.clone());
    let input = publication.launch_input.as_ref().unwrap();
    transport_api_types::ConfigProjectionActivationCarrierV1 {
        authority_store_id: dormant.authority_store_id.clone(),
        series_id: dormant.series_id.clone(),
        dormant_projection_ref: dormant.clone(),
        activation_intent_ref: input.activation_intent_ref.clone(),
        expected_gateway_ref: input.gateway_ref.clone(),
        fence_id: publication
            .activation_intent
            .as_ref()
            .unwrap()
            .fence_id
            .clone(),
        consumer_id: lease.consumer_id.clone(),
        consumer_lease_revision: lease.revision,
        consumer_lease_hash: lease.lease_hash.clone(),
    }
    .validate()
    .expect("fixture transport carrier");
    let carrier = build_prepared_response_v1(&publication)
        .unwrap()
        .config_projection;
    assert_eq!(
        service
            .resolve_activation_carrier(&publication, &carrier)
            .unwrap(),
        *publication.launch_input.as_ref().unwrap()
    );
    assert_eq!(
        service.activate_managed_gateway(&mut publication, ready.clone()),
        Err(ConfigProjectionFailureV1::WrongBinding)
    );
    let delivered_at = publication.prepared_handoff.handoff.created_at.clone();
    let delivered = E3ManagedGatewayActivationObservationV1::Delivered { delivered_at };
    assert_eq!(
        service
            .activate_managed_gateway(&mut publication, delivered.clone())
            .unwrap(),
        None
    );
    let frozen = publication.activation_progress.delivered.clone().unwrap();
    assert_eq!(
        service
            .activate_managed_gateway(&mut publication, delivered.clone())
            .unwrap(),
        None
    );
    assert_eq!(
        publication.activation_progress.delivered.as_ref(),
        Some(&frozen)
    );
    let reference = service
        .activate_managed_gateway(&mut publication, ready.clone())
        .unwrap()
        .unwrap();
    let frozen_ack = publication.activation_progress.ack.clone().unwrap();
    let frozen_record = publication
        .activation_progress
        .ready_record
        .clone()
        .unwrap();
    assert_eq!(
        service
            .activate_managed_gateway(&mut publication, ready.clone())
            .unwrap(),
        Some(reference.clone())
    );
    assert_eq!(
        publication.activation_progress.ack.as_ref(),
        Some(&frozen_ack)
    );
    assert_eq!(
        publication.activation_progress.ready_record.as_ref(),
        Some(&frozen_record)
    );
    assert_eq!(publication.published_head(), Some(&dormant));
    assert_eq!(publication.held_consumer_lease(), Some(&lease));
    assert_eq!(
        service.activate_managed_gateway(&mut publication, delivered),
        Err(ConfigProjectionFailureV1::WrongBinding)
    );
    let mut unequal = ready;
    if let E3ManagedGatewayActivationObservationV1::Ready {
        gateway_process_identity,
        ..
    } = &mut unequal
    {
        gateway_process_identity.pidfd_inode += 1;
    }
    assert_eq!(
        service.activate_managed_gateway(&mut publication, unequal),
        Err(ConfigProjectionFailureV1::Conflict)
    );
    assert!(service
        .resolve_activation_carrier(&publication, &carrier)
        .is_err());
    reference
}

impl AgentConfigProjectionServiceV1 {
    pub fn new(
        registry: Arc<ConfigProjectionRegistryV1>,
        accepted_home: Arc<ConfiguredAcceptedHomeAuthorityV1>,
    ) -> Result<Self, ConfigProjectionFailureV1> {
        accepted_home.revalidate()?;
        let store = registry.recover(None).map(|readback| readback.store)?;
        if store.accepted_home != *accepted_home.accepted_home() {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(Self {
            registry,
            accepted_home,
        })
    }

    pub fn resolve_preparation_subject_v1(
        &self,
        subject: &crate::ConfigProjectionIdentityV1,
    ) -> Result<crate::ConfigProjectionSubjectReadbackV1, ConfigProjectionFailureV1> {
        self.accepted_home.revalidate()?;
        let readback = self.registry.recover(Some(subject))?;
        if readback.store.accepted_home != *self.accepted_home.accepted_home() {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        readback
            .subject
            .ok_or(ConfigProjectionFailureV1::PartialPublication)
    }

    pub fn publish_retained_launch_inputs(
        &self,
        effective_config: &EffectiveSubstrateConfigSourceV1,
        agent_inventory: &AgentInventorySourceMaterialV1,
        created_at: Timestamp,
    ) -> Result<ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1> {
        self.accepted_home.revalidate()?;
        self.registry
            .import_runtime_artifacts(effective_config, agent_inventory, created_at)
    }

    pub fn publish_retained_fork_inputs(
        &self,
        effective_config: &EffectiveSubstrateConfigSourceV1,
        agent_inventory: &AgentInventorySourceMaterialV1,
        created_at: Timestamp,
    ) -> Result<ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1> {
        self.publish_retained_launch_inputs(effective_config, agent_inventory, created_at)
    }

    pub fn publish_prepared_retained_launch(
        &self,
        authoring_input_ref: &ConfigProjectionAuthoringInputRefV1,
        authenticated_e2_launch_activation: &transport_api_types::E2MemberLaunchActivationCarrierV1,
        publication: &mut E3PreparedRetainedLaunchPublicationV1,
    ) -> Result<
        (
            transport_api_types::E3ConfigProjectionPrepareResponseV1,
            crate::PublishedConfigProjectionCapabilityV1,
        ),
        ConfigProjectionFailureV1,
    > {
        use ConfigProjectionFailureV1::{
            PartialPublication, RetiredSeries, StaleRevision, WrongBinding,
        };
        self.accepted_home.revalidate()?;
        validate_prepared_retained_launch_publication_v1(publication)?;
        let record = publication.record.as_ref().ok_or(PartialPublication)?;
        let (_, inputs) = self.registry.resolve(None, Some(authoring_input_ref))?;
        let (effective, source, manifest) = inputs.ok_or(PartialPublication)?;
        let identity = &record.identity;
        let expected_sources = vec![
            crate::LogicalConfigSourceRefV1::EffectiveSubstrateConfig {
                authority_store_id: effective.authority_store_id.clone(),
                source_revision: effective.source_revision.clone(),
                source_hash: effective.source_hash.clone(),
            },
            crate::LogicalConfigSourceRefV1::AgentInventory {
                authority_store_id: effective.authority_store_id.clone(),
                inventory_scope: source.inventory_scope.clone(),
                source_revision: source.source_revision.clone(),
                source_hash: source.source_hash.clone(),
            },
        ];
        if identity.accepted_home != *self.accepted_home.accepted_home()
            || identity.accepted_home != effective.accepted_home
            || identity.workspace_root != effective.workspace_root
            || identity.authority_store_id != effective.authority_store_id
            || identity.authority_store_id != authoring_input_ref.authority_store_id
            || record.logical.sources != expected_sources
            || record.logical.backend_id != effective.values.default_backend_id
            || record.logical.backend_id != identity.backend_id
            || record.logical.runtime_family != identity.runtime_family
            || record.logical.placement != "world"
            || record.logical.execution_scope != "world"
        {
            return Err(WrongBinding);
        }
        for (artifact, role) in [
            (
                &identity.runtime_artifacts.codex,
                crate::RuntimeArtifactAuthorityRoleV1::Codex0125,
            ),
            (
                &identity.runtime_artifacts.world_entry_wrapper,
                crate::RuntimeArtifactAuthorityRoleV1::WorldEntryWrapper,
            ),
            (
                &identity.runtime_artifacts.managed_gateway,
                crate::RuntimeArtifactAuthorityRoleV1::ManagedGateway,
            ),
        ] {
            let reference = &artifact.authority_ref;
            let entry = manifest
                .entries
                .iter()
                .find(|entry| entry.manifest_entry_id == reference.manifest_entry_id)
                .ok_or(WrongBinding)?;
            if reference.authority_store_id != manifest.authority_store_id
                || reference.manifest_id != manifest.manifest_id
                || reference.manifest_revision != manifest.revision
                || reference.manifest_hash != manifest.manifest_hash
                || reference.entry_hash != entry.entry_hash
                || entry.authority_role != role
                || artifact.configured_absolute_path != entry.configured_absolute_path
                || artifact.device_id != entry.device_id
                || artifact.inode != entry.inode
                || artifact.mode != entry.mode
                || artifact.owner_uid != entry.owner_uid
                || artifact.byte_length != entry.byte_length
                || artifact.sha256 != entry.sha256
                || artifact.provenance != entry.provenance
                || artifact.runtime_support != entry.runtime_support
            {
                return Err(WrongBinding);
            }
        }
        let e2 = authenticated_e2_launch_activation;
        e2.validate().map_err(|_| WrongBinding)?;
        let subject = match e2.launch_kind {
            crate::E2MemberLaunchKindV1::FreshSpawn => {
                crate::ConfigProjectionE2SubjectV1::RetainedWorkerLaunch {
                    retained_participant_id: e2.retained_participant_id.clone(),
                    bootstrap_run_id: e2.bootstrap_run_id.clone(),
                }
            }
            crate::E2MemberLaunchKindV1::Fork => {
                crate::ConfigProjectionE2SubjectV1::RetainedWorkerFork {
                    source_participant_id: e2.source_participant_id.clone().ok_or(WrongBinding)?,
                    child_participant_id: e2.retained_participant_id.clone(),
                    bootstrap_run_id: e2.bootstrap_run_id.clone(),
                }
            }
        };
        let cap = crate::ConfigProjectionE2CapLinkV1 {
            e2_activation_id: e2.activation_id.clone(),
            e2_launch_kind: e2.launch_kind,
            commitment_ref: e2.commitment_ref.clone(),
            commitment_subject: subject,
            immutable_worker_cap_ref: e2.immutable_worker_cap_ref.clone(),
            immutable_worker_cap_created_revision: e2.immutable_worker_cap_created_revision,
            immutable_worker_cap_application_revision: e2.immutable_worker_cap_application_revision,
            policy_snapshot_ref: e2.policy_snapshot_ref.clone(),
            policy_snapshot_hash: e2.policy_snapshot_hash.clone(),
            policy_snapshot_revision: e2.policy_snapshot_revision.clone(),
            request_id: e2.request_id.clone(),
            idempotency_key: e2.idempotency_key.clone(),
            caller_participant_id: e2.caller_participant_id.clone(),
            caller_backend_id: e2.caller_backend_id.clone(),
            target_backend_id: e2.target_backend_id.clone(),
            target_world: e2.target_world.clone(),
            registry_publication_revision: e2.registry_publication_revision,
        };
        if identity.immutable_launch_cap != cap
            || record.effective.accepted_policy != cap
            || e2.orchestration_session_id != identity.orchestration_session_id
            || e2.retained_participant_id != identity.retained_participant_id
            || e2.bootstrap_run_id != identity.bootstrap_run_id
            || e2.target_backend_id != identity.backend_id
            || e2.target_world.world_id != identity.world_id
            || e2.target_world.world_generation != identity.world_generation
        {
            return Err(WrongBinding);
        }
        if publication.published_head.is_none() {
            let head = self.registry.publish_dormant(
                record,
                Some((
                    &publication.prepared_handoff,
                    &publication.gateway,
                    publication
                        .dormant_boundary
                        .as_ref()
                        .ok_or(PartialPublication)?,
                    publication
                        .activation_intent
                        .as_ref()
                        .ok_or(PartialPublication)?,
                    publication
                        .launch_input
                        .as_ref()
                        .ok_or(PartialPublication)?,
                    publication
                        .ordered_child_cgroups
                        .as_ref()
                        .ok_or(PartialPublication)?,
                )),
            )?;
            // Store durable progress before any later fallible operation, including lease creation.
            publication.published_head = Some(head);
        }
        let head = publication
            .published_head
            .as_ref()
            .ok_or(PartialPublication)?;
        if publication.held_consumer_lease.is_none() {
            let held = self.registry.acquire_consumer_lease(
                head,
                crate::ConfigProjectionConsumerKindV1::MemberDispatchV2,
                record.created_at.clone(),
                Some(&publication.consumer_id),
            )?;
            publication.held_consumer_lease = Some(held);
        }
        let lease = publication
            .held_consumer_lease
            .as_ref()
            .ok_or(PartialPublication)?;
        let (resolution, _) = self.registry.resolve(Some((identity, lease)), None)?;
        let capability = match resolution.ok_or(PartialPublication)? {
            crate::ConfigProjectionResolutionV1::Current {
                identity: resolved_identity,
                projection_ref,
                record: resolved_record,
                capability,
            } if resolved_identity == *identity
                && projection_ref == *head
                && resolved_record == *record =>
            {
                capability
            }
            crate::ConfigProjectionResolutionV1::Retired { .. } => return Err(RetiredSeries),
            _ => return Err(StaleRevision),
        };
        let response = build_prepared_response_v1(publication)?;
        Ok((response, capability))
    }

    pub fn resolve_activation_carrier(
        &self,
        publication: &E3PreparedRetainedLaunchPublicationV1,
        carrier: &transport_api_types::ConfigProjectionActivationCarrierV1,
    ) -> Result<crate::ManagedGatewayLaunchInputV1, ConfigProjectionFailureV1> {
        use ConfigProjectionFailureV1::{PartialPublication, StaleRevision, WrongBinding};
        self.accepted_home.revalidate()?;
        validate_gateway_activation_publication_v1(publication)?;
        let expected = build_prepared_response_v1(publication)?;
        if expected.config_projection != *carrier {
            return Err(WrongBinding);
        }
        let record = publication.record.as_ref().ok_or(PartialPublication)?;
        let lease = publication
            .held_consumer_lease
            .as_ref()
            .ok_or(PartialPublication)?;
        let (resolved, _) = self
            .registry
            .resolve(Some((&publication.identity, lease)), None)?;
        match resolved.ok_or(PartialPublication)? {
            crate::ConfigProjectionResolutionV1::Current {
                record: actual,
                projection_ref,
                ..
            } if actual == *record
                && Some(&projection_ref) == publication.published_head.as_ref() => {}
            _ => return Err(StaleRevision),
        }
        let crate::ConfigProjectionSubjectReadbackV1::Bound(current) =
            self.resolve_preparation_subject_v1(&publication.identity)?
        else {
            return Err(PartialPublication);
        };
        if current.record != *record
            || current.prepared_handoff != publication.prepared_handoff
            || current.consumer_lease.as_ref() != Some(lease)
        {
            return Err(WrongBinding);
        }
        publication.launch_input.clone().ok_or(PartialPublication)
    }

    pub fn activate_managed_gateway(
        &self,
        publication: &mut E3PreparedRetainedLaunchPublicationV1,
        observation: E3ManagedGatewayActivationObservationV1,
    ) -> Result<Option<crate::ConfigProjectionRefV1>, ConfigProjectionFailureV1> {
        use ConfigProjectionFailureV1::{Conflict, PartialPublication, WrongBinding};
        self.accepted_home.revalidate()?;
        validate_gateway_activation_publication_v1(publication)?;
        let head = publication
            .published_head
            .as_ref()
            .ok_or(PartialPublication)?;
        let lease = publication
            .held_consumer_lease
            .as_ref()
            .ok_or(PartialPublication)?;
        let intent = publication
            .activation_intent
            .as_ref()
            .ok_or(PartialPublication)?;
        match &observation {
            E3ManagedGatewayActivationObservationV1::Delivered { delivered_at } => {
                let successor = build_gateway_handoff_successor_v1(
                    &publication.prepared_handoff,
                    crate::SecretHandoffStateV1::Delivered,
                    delivered_at,
                )?;
                if let Some(frozen) = &publication.activation_progress.delivered {
                    if *frozen != successor {
                        return Err(Conflict);
                    }
                } else {
                    publication.activation_progress.delivered = Some(successor);
                }
                if publication.activation_progress.ready_observation.is_some() {
                    return Err(WrongBinding);
                }
                let reference = self.registry.publish_handoff_transition_v1(
                    &publication.identity,
                    &intent.fence_id,
                    publication
                        .activation_progress
                        .delivered
                        .as_ref()
                        .ok_or(PartialPublication)?,
                    Some(head),
                    Some(lease),
                )?;
                publication.activation_progress.delivered_ref = Some(reference);
                Ok(None)
            }
            E3ManagedGatewayActivationObservationV1::Ready { observed_at, .. } => {
                let delivered = publication
                    .activation_progress
                    .delivered
                    .as_ref()
                    .ok_or(WrongBinding)?;
                if publication.activation_progress.delivered_ref.as_ref()
                    != Some(&secret_handoff_reference_v1(delivered)?)
                {
                    return Err(WrongBinding);
                }
                if let Some(frozen) = &publication.activation_progress.ready_observation {
                    if *frozen != observation {
                        return Err(Conflict);
                    }
                } else {
                    let consumed = build_gateway_handoff_successor_v1(
                        delivered,
                        crate::SecretHandoffStateV1::Consumed,
                        observed_at,
                    )?;
                    let (ack, ready) =
                        build_gateway_ready_closed_v1(publication, &consumed, &observation)?;
                    // Freeze all identity/timestamp/byte choices before the first fallible write.
                    publication.activation_progress.ready_observation = Some(observation.clone());
                    publication.activation_progress.consumed = Some(consumed);
                    publication.activation_progress.ack = Some(ack);
                    publication.activation_progress.ready_record = Some(ready);
                }
                let progress = &mut publication.activation_progress;
                if progress.consumed_ref.is_none() {
                    let reference = self.registry.publish_handoff_transition_v1(
                        &publication.identity,
                        &intent.fence_id,
                        progress.consumed.as_ref().ok_or(PartialPublication)?,
                        Some(head),
                        Some(lease),
                    )?;
                    progress.consumed_ref = Some(reference);
                }
                // Always exact-readback ReadyClosed even on a successful earlier return. No
                // second capability is created and the original Dormant/lease are untouched.
                let ready = self.registry.publish_ready_closed(
                    head,
                    progress.ready_record.as_ref().ok_or(PartialPublication)?,
                    progress.ack.as_ref().ok_or(PartialPublication)?,
                    lease,
                )?;
                progress.ready_ref = Some(ready.clone());
                Ok(Some(ready))
            }
        }
    }

    pub fn publish_preparation_abandonment(
        &self,
        target: E3PreparationAbandonmentTargetV1<'_>,
        terminal_state: crate::SecretHandoffStateV1,
        released_at: Timestamp,
        failure_diagnostic_ref: Option<crate::RedactedDiagnosticRefV1>,
    ) -> Result<Option<crate::SecretHandoffRefV1>, ConfigProjectionFailureV1> {
        self.accepted_home.revalidate()?;
        if !matches!(
            terminal_state,
            crate::SecretHandoffStateV1::Failed | crate::SecretHandoffStateV1::Expired
        ) {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_rfc3339_micros_utc_timestamp_v1(&released_at)?;
        let publication = match target {
            E3PreparationAbandonmentTargetV1::Live(publication) => publication,
            E3PreparationAbandonmentTargetV1::Recovered(expected) => {
                let crate::ConfigProjectionSubjectReadbackV1::Bound(current) =
                    self.resolve_preparation_subject_v1(&expected.record.identity)?
                else {
                    return Err(ConfigProjectionFailureV1::MissingPreparation);
                };
                if current.record != expected.record
                    || current.projection_ref != expected.projection_ref
                    || current.prepared_handoff != expected.prepared_handoff
                {
                    return Err(ConfigProjectionFailureV1::StaleRevision);
                }
                if current.record.managed_gateway.posture
                    == crate::ManagedGatewayProjectionPostureV1::Active
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                // Recovery preserves an existing terminal successor exactly, including Expired
                // versus Failed and its diagnostic. The caller cannot rewrite retained history.
                let (state, diagnostic) = if matches!(
                    current.current_handoff.handoff.state,
                    crate::SecretHandoffStateV1::Failed | crate::SecretHandoffStateV1::Expired
                ) {
                    (
                        current.current_handoff.handoff.state,
                        current
                            .current_handoff
                            .handoff
                            .failure_diagnostic_ref
                            .clone(),
                    )
                } else {
                    (terminal_state, failure_diagnostic_ref)
                };
                let fence = match &current.record.activation.publication_fence {
                    crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } => {
                        fence_id
                    }
                    _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                };
                let terminal = if current.current_handoff.handoff.state
                    == crate::SecretHandoffStateV1::Consumed
                {
                    // Consumed is terminal. Cleanup proves quiescence separately and preserves
                    // the existing immutable revision instead of attempting a forbidden edge.
                    let readback = self.registry.recover(Some(&current.record.identity))?;
                    if readback.kernel_effects.iter().any(|effect| {
                        effect.intent.series_id == current.record.identity.series_id
                            && effect.intent.fence_id == *fence
                            && effect.resolution.is_none()
                    }) {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                    secret_handoff_reference_v1(&current.current_handoff)?
                } else {
                    let successor = build_preparation_abandonment_successor_v1(
                        &current.current_handoff,
                        state,
                        diagnostic,
                    )?;
                    self.registry.publish_handoff_transition_v1(
                        &current.record.identity,
                        fence,
                        &successor,
                        Some(&current.projection_ref),
                        None,
                    )?
                };
                match current.consumer_lease {
                    Some(lease)
                        if lease.posture
                            == crate::ConfigProjectionConsumerLeasePostureV1::Released =>
                    {
                        // The registry read validated the immutable release chain; its original time wins.
                        if lease.released_at.is_none() {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        }
                    }
                    lease => {
                        let held = if let Some(held) = lease {
                            held
                        } else {
                            let acquisition_ref = match current.record.managed_gateway.posture {
                                crate::ManagedGatewayProjectionPostureV1::Dormant => {
                                    &current.projection_ref
                                }
                                crate::ManagedGatewayProjectionPostureV1::ReadyClosed => current
                                    .record
                                    .predecessor_ref
                                    .as_ref()
                                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                                _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                            };
                            self.registry.acquire_consumer_lease(
                                acquisition_ref,
                                crate::ConfigProjectionConsumerKindV1::MemberDispatchV2,
                                current.prepared_handoff.handoff.created_at.clone(),
                                Some(&crate::registry::prepared_member_dispatch_consumer_id_v1(
                                    &current
                                        .prepared_handoff
                                        .handoff
                                        .credential_source_ref
                                        .preparation_id,
                                )?),
                            )?
                        };
                        self.registry.release_consumer_lease(&held, released_at)?;
                    }
                }
                return Ok(Some(terminal));
            }
        };
        let Some(record) = publication.record.as_ref().cloned() else {
            validate_preparation_abandonment_v1(
                publication,
                terminal_state,
                &released_at,
                &failure_diagnostic_ref,
            )?;
            return Ok(None);
        };
        if publication.activation_progress.delivered.is_some() {
            let crate::ConfigProjectionSubjectReadbackV1::Bound(current) =
                self.resolve_preparation_subject_v1(&publication.identity)?
            else {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            };
            let exact_head = current.record == record
                || publication.activation_progress.ready_record.as_ref() == Some(&current.record);
            if !exact_head
                || current.prepared_handoff != publication.prepared_handoff
                || current.consumer_lease.as_ref().is_some_and(|lease| {
                    publication
                        .held_consumer_lease
                        .as_ref()
                        .is_none_or(|original| {
                            lease.consumer_id != original.consumer_id
                                || lease.acquired_projection_ref != original.acquired_projection_ref
                        })
                })
            {
                return Err(ConfigProjectionFailureV1::StaleRevision);
            }
            return self.publish_preparation_abandonment(
                E3PreparationAbandonmentTargetV1::Recovered(&current),
                terminal_state,
                released_at,
                failure_diagnostic_ref,
            );
        }
        validate_preparation_abandonment_v1(
            publication,
            terminal_state,
            &released_at,
            &failure_diagnostic_ref,
        )?;
        if publication.published_head.is_none() {
            let head = self.registry.publish_dormant(
                &record,
                Some((
                    &publication.prepared_handoff,
                    &publication.gateway,
                    publication
                        .dormant_boundary
                        .as_ref()
                        .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                    publication
                        .activation_intent
                        .as_ref()
                        .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                    publication
                        .launch_input
                        .as_ref()
                        .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                    publication
                        .ordered_child_cgroups
                        .as_ref()
                        .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                )),
            )?;
            publication.published_head = Some(head);
        }
        let head = publication
            .published_head
            .as_ref()
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let expected_head = crate::ConfigProjectionRefV1 {
            authority_store_id: record.identity.authority_store_id.clone(),
            series_id: record.identity.series_id.clone(),
            record_id: record.record_id.clone(),
            revision: record.revision,
            record_hash: record.record_hash.clone(),
        };
        if *head != expected_head {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        if publication.held_consumer_lease.is_none() {
            let held = self.registry.acquire_consumer_lease(
                head,
                crate::ConfigProjectionConsumerKindV1::MemberDispatchV2,
                record.created_at.clone(),
                Some(&publication.consumer_id),
            )?;
            publication.held_consumer_lease = Some(held);
        }
        let successor = build_preparation_abandonment_successor_v1(
            &publication.prepared_handoff,
            terminal_state,
            failure_diagnostic_ref,
        )?;
        let fence_id = match &record.activation.publication_fence {
            crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } => fence_id,
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        };
        let terminal_ref = self.registry.publish_handoff_transition_v1(
            &publication.identity,
            fence_id,
            &successor,
            Some(head),
            None,
        )?;
        let held = publication
            .held_consumer_lease
            .as_ref()
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        self.registry.release_consumer_lease(held, released_at)?;
        Ok(Some(terminal_ref))
    }
}

/// Nonsecret, attempt-owned publication progress. Live resources stay in world-service.
pub struct E3PreparedRetainedLaunchPublicationV1 {
    identity: crate::ConfigProjectionIdentityV1,
    gateway: crate::InWorldGatewayIdentityV1,
    prepared_handoff: crate::ConfigProjectionSecretHandoffRevisionV1,
    prepared_handoff_ref: crate::SecretHandoffRefV1,
    preparation_idempotency_key: String,
    consumer_id: String,
    record: Option<crate::AgentConfigProjectionRecordV1>,
    ordered_child_cgroups: Option<[crate::E3ChildCgroupRegistrationV1; 3]>,
    dormant_boundary: Option<crate::GatewayAccessBoundaryV1>,
    activation_intent: Option<crate::ManagedGatewayActivationIntentV1>,
    launch_input: Option<crate::ManagedGatewayLaunchInputV1>,
    published_head: Option<crate::ConfigProjectionRefV1>,
    held_consumer_lease: Option<crate::ConfigProjectionConsumerLeaseV1>,
    activation_progress: E3GatewayActivationPublicationProgressV1,
}

impl E3PreparedRetainedLaunchPublicationV1 {
    pub fn new(
        identity: &crate::ConfigProjectionIdentityV1,
        gateway: &crate::InWorldGatewayIdentityV1,
        credential_source_ref: &crate::CredentialSourceRefV1,
        handoff_id: String,
        preparation_idempotency_key: String,
        created_at: Timestamp,
        expires_at: Timestamp,
    ) -> Result<Self, ConfigProjectionFailureV1> {
        let digest = preparation_idempotency_key
            .strip_prefix("e3pik_")
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        let prepared_handoff = build_prepared_secret_handoff_revision_v1(
            identity,
            gateway,
            credential_source_ref,
            handoff_id,
            created_at,
            expires_at,
        )?;
        let handoff = &prepared_handoff.handoff;
        let prepared_handoff_ref = crate::SecretHandoffRefV1 {
            authority_store_id: identity.authority_store_id.clone(),
            handoff_id: handoff.handoff_id.clone(),
            orchestration_session_id: identity.orchestration_session_id.clone(),
            retained_participant_id: identity.retained_participant_id.clone(),
            runtime_family: identity.runtime_family.clone(),
            world_id: identity.world_id.clone(),
            world_generation: identity.world_generation,
            receiving_gateway_identity_hash: gateway.gateway_identity_hash.clone(),
            handoff_state_revision: 1,
            handoff_hash: prepared_handoff.revision_hash.clone(),
        };
        Ok(Self {
            identity: identity.clone(),
            gateway: gateway.clone(),
            prepared_handoff,
            prepared_handoff_ref,
            preparation_idempotency_key,
            consumer_id: crate::registry::prepared_member_dispatch_consumer_id_v1(
                &credential_source_ref.preparation_id,
            )?,
            record: None,
            ordered_child_cgroups: None,
            dormant_boundary: None,
            activation_intent: None,
            launch_input: None,
            published_head: None,
            held_consumer_lease: None,
            activation_progress: E3GatewayActivationPublicationProgressV1::default(),
        })
    }

    pub fn prepared_handoff_ref(&self) -> &crate::SecretHandoffRefV1 {
        &self.prepared_handoff_ref
    }

    pub fn bind_prepared_chain(
        &mut self,
        record: crate::AgentConfigProjectionRecordV1,
        ordered_child_cgroups: [crate::E3ChildCgroupRegistrationV1; 3],
        dormant_boundary: crate::GatewayAccessBoundaryV1,
        activation_intent: crate::ManagedGatewayActivationIntentV1,
        launch_input: crate::ManagedGatewayLaunchInputV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if self.record.is_some()
            || self.published_head.is_some()
            || self.held_consumer_lease.is_some()
        {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        let candidate = Self {
            identity: self.identity.clone(),
            gateway: self.gateway.clone(),
            prepared_handoff: self.prepared_handoff.clone(),
            prepared_handoff_ref: self.prepared_handoff_ref.clone(),
            preparation_idempotency_key: self.preparation_idempotency_key.clone(),
            consumer_id: self.consumer_id.clone(),
            record: Some(record),
            ordered_child_cgroups: Some(ordered_child_cgroups),
            dormant_boundary: Some(dormant_boundary),
            activation_intent: Some(activation_intent),
            launch_input: Some(launch_input),
            published_head: None,
            held_consumer_lease: None,
            activation_progress: E3GatewayActivationPublicationProgressV1::default(),
        };
        validate_prepared_retained_launch_publication_v1(&candidate)?;
        *self = candidate;
        Ok(())
    }

    pub fn published_head(&self) -> Option<&crate::ConfigProjectionRefV1> {
        self.published_head.as_ref()
    }
    pub fn held_consumer_lease(&self) -> Option<&crate::ConfigProjectionConsumerLeaseV1> {
        self.held_consumer_lease.as_ref()
    }
}

fn validate_gateway_activation_publication_v1(
    publication: &E3PreparedRetainedLaunchPublicationV1,
) -> Result<(), ConfigProjectionFailureV1> {
    validate_prepared_retained_launch_publication_v1(publication)?;
    let head = publication
        .published_head
        .as_ref()
        .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
    let lease = publication
        .held_consumer_lease
        .as_ref()
        .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
    if lease.revision != 1
        || lease.acquired_projection_ref != *head
        || lease.consumer_id != publication.consumer_id
        || lease.consumer_kind != crate::ConfigProjectionConsumerKindV1::MemberDispatchV2
        || lease.posture != crate::ConfigProjectionConsumerLeasePostureV1::Held
    {
        return Err(ConfigProjectionFailureV1::WrongBinding);
    }
    Ok(())
}

fn build_gateway_handoff_successor_v1(
    predecessor: &crate::ConfigProjectionSecretHandoffRevisionV1,
    state: crate::SecretHandoffStateV1,
    timestamp: &Timestamp,
) -> Result<crate::ConfigProjectionSecretHandoffRevisionV1, ConfigProjectionFailureV1> {
    use crate::SecretHandoffStateV1::{Consumed, Delivered, Prepared};
    use ConfigProjectionFailureV1::{Malformed, WrongBinding};
    validate_rfc3339_micros_utc_timestamp_v1(timestamp)?;
    if !matches!(
        (predecessor.handoff.state, state),
        (Prepared, Delivered) | (Delivered, Consumed)
    ) || timestamp.0 < predecessor.handoff.created_at.0
        || timestamp.0 >= predecessor.handoff.expires_at.0
        || predecessor
            .handoff
            .delivered_at
            .as_ref()
            .is_some_and(|t| timestamp.0 < t.0)
    {
        return Err(WrongBinding);
    }
    let mut next = predecessor.clone();
    next.predecessor_ref = Some(secret_handoff_reference_v1(predecessor)?);
    next.handoff.state_revision = next
        .handoff
        .state_revision
        .checked_add(1)
        .ok_or(Malformed)?;
    next.handoff.state = state;
    match state {
        Delivered => next.handoff.delivered_at = Some(timestamp.clone()),
        Consumed => next.handoff.consumed_at = Some(timestamp.clone()),
        _ => return Err(WrongBinding),
    }
    let mut value = serde_json::to_value(&next).map_err(|_| Malformed)?;
    value
        .as_object_mut()
        .ok_or(Malformed)?
        .remove("revision_hash");
    next.revision_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.config-projection-secret-handoff-revision.v1",
        &serde_json::json!({"revision": value}),
    )?;
    Ok(next)
}

fn build_gateway_ready_closed_v1(
    publication: &E3PreparedRetainedLaunchPublicationV1,
    consumed: &crate::ConfigProjectionSecretHandoffRevisionV1,
    observation: &E3ManagedGatewayActivationObservationV1,
) -> Result<
    (
        crate::ManagedGatewayActivationAckV1,
        crate::AgentConfigProjectionRecordV1,
    ),
    ConfigProjectionFailureV1,
> {
    use ConfigProjectionFailureV1::{Malformed, PartialPublication, WrongBinding};
    let E3ManagedGatewayActivationObservationV1::Ready {
        gateway_process_identity,
        child_security_attestation,
        observed_at,
    } = observation
    else {
        return Err(WrongBinding);
    };
    let input = publication
        .launch_input
        .as_ref()
        .ok_or(PartialPublication)?;
    let mut ack = crate::ManagedGatewayActivationAckV1 {
        schema_version: 1,
        authority_store_id: publication.identity.authority_store_id.clone(),
        activation_ack_id: format!("gaa_{}", uuid::Uuid::now_v7()),
        activation_intent_ref: input.activation_intent_ref.clone(),
        config_projection_identity_hash: publication.identity.identity_hash.clone(),
        dormant_projection_ref: input.dormant_projection_ref.clone(),
        gateway_ref: input.gateway_ref.clone(),
        gateway_process_identity: gateway_process_identity.clone(),
        child_security_attestation: child_security_attestation.clone(),
        listener_identity: input.listener_identity.clone(),
        access_boundary_ref: input.access_boundary_ref.clone(),
        secret_handoff_ref: secret_handoff_reference_v1(consumed)?,
        secret_handoff_terminal_state: crate::SecretHandoffStateV1::Consumed,
        gateway_ready_revision: 1,
        readiness_nonce: input.readiness_nonce.clone(),
        launch_input_ref: crate::ManagedGatewayLaunchInputRefV1 {
            authority_store_id: input.authority_store_id.clone(),
            launch_input_id: input.launch_input_id.clone(),
            launch_input_hash: input.launch_input_hash.clone(),
        },
        observed_at: observed_at.clone(),
        ack_hash: String::new(),
    };
    let mut value = serde_json::to_value(&ack).map_err(|_| Malformed)?;
    value.as_object_mut().ok_or(Malformed)?.remove("ack_hash");
    ack.ack_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.managed-gateway-activation-ack.v1",
        &serde_json::json!({"ack": value}),
    )?;
    let reference = crate::ManagedGatewayActivationAckRefV1 {
        authority_store_id: ack.authority_store_id.clone(),
        activation_ack_id: ack.activation_ack_id.clone(),
        ack_hash: ack.ack_hash.clone(),
    };
    let mut ready = publication.record.clone().ok_or(PartialPublication)?;
    ready.record_id = format!("cpr_{}", uuid::Uuid::now_v7());
    ready.revision = ready.revision.checked_add(1).ok_or(Malformed)?;
    ready.predecessor_ref = publication.published_head.clone();
    ready.created_at = observed_at.clone();
    ready.managed_gateway.posture = crate::ManagedGatewayProjectionPostureV1::ReadyClosed;
    ready.managed_gateway.activation_ack_ref = Some(reference.clone());
    ready.nonsecret_handoff.secret_handoff_ref = ack.secret_handoff_ref.clone();
    ready.nonsecret_handoff.observed_state = crate::SecretHandoffStateV1::Consumed;
    ready.nonsecret_handoff.activation_ack_ref = Some(reference.clone());
    ready.activation.gateway_activation_ack_ref = Some(reference);
    let mut gateway_value = serde_json::to_value(&ready.managed_gateway).map_err(|_| Malformed)?;
    gateway_value
        .as_object_mut()
        .ok_or(Malformed)?
        .remove("projection_hash");
    ready.managed_gateway.projection_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.managed-gateway-projection.v1",
        &serde_json::json!({"projection": gateway_value}),
    )?;
    let mut handoff_value =
        serde_json::to_value(&ready.nonsecret_handoff).map_err(|_| Malformed)?;
    handoff_value
        .as_object_mut()
        .ok_or(Malformed)?
        .remove("projection_hash");
    ready.nonsecret_handoff.projection_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.nonsecret-handoff-projection.v1",
        &serde_json::json!({"projection": handoff_value}),
    )?;
    let mut value = serde_json::to_value(&ready).map_err(|_| Malformed)?;
    value
        .as_object_mut()
        .ok_or(Malformed)?
        .remove("record_hash");
    ready.record_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.agent-config-projection-record.v1",
        &serde_json::json!({"record": value}),
    )?;
    Ok((ack, ready))
}

fn build_prepared_secret_handoff_revision_v1(
    identity: &crate::ConfigProjectionIdentityV1,
    gateway: &crate::InWorldGatewayIdentityV1,
    credential: &crate::CredentialSourceRefV1,
    handoff_id: String,
    created_at: Timestamp,
    expires_at: Timestamp,
) -> Result<crate::ConfigProjectionSecretHandoffRevisionV1, ConfigProjectionFailureV1> {
    use ConfigProjectionFailureV1::{HashInvalid, Malformed, WrongBinding};
    let uuid = |value: &str, prefix: &str| -> Result<(), ConfigProjectionFailureV1> {
        let suffix = value.strip_prefix(prefix).ok_or(Malformed)?;
        let parsed = uuid::Uuid::parse_str(suffix).map_err(|_| Malformed)?;
        if parsed.get_version_num() != 7 || parsed.to_string() != suffix {
            return Err(Malformed);
        }
        Ok(())
    };
    for (value, prefix) in [
        (&identity.authority_store_id, "cpa_"),
        (&identity.series_id, "cps_"),
        (&gateway.gateway_instance_id, "cgi_"),
        (&gateway.access_boundary_id, "gab_"),
        (&credential.credential_source_id, "crs_"),
        (&credential.preparation_id, "e3p_"),
        (&handoff_id, "gsh_"),
    ] {
        uuid(value, prefix)?;
    }
    let start = chrono::DateTime::parse_from_rfc3339(&created_at.0).map_err(|_| Malformed)?;
    let end = chrono::DateTime::parse_from_rfc3339(&expires_at.0).map_err(|_| Malformed)?;
    if created_at.0 != start.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
        || expires_at.0 != end.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
        || !created_at.0.ends_with('Z')
        || !expires_at.0.ends_with('Z')
    {
        return Err(Malformed);
    }
    if end.signed_duration_since(start) != chrono::Duration::seconds(120)
        || credential.issued_at != created_at
        || credential.expires_at != expires_at
    {
        return Err(WrongBinding);
    }
    if identity.schema_version != 1 || gateway.schema_version != 1 || credential.schema_version != 1
    {
        return Err(Malformed);
    }
    if gateway.authority_store_id != identity.authority_store_id
        || gateway.config_projection_identity_hash != identity.identity_hash
        || gateway.orchestration_session_id != identity.orchestration_session_id
        || gateway.retained_participant_id != identity.retained_participant_id
        || gateway.backend_id != identity.backend_id
        || gateway.world_id != identity.world_id
        || gateway.world_generation != identity.world_generation
        || gateway.gateway_artifact_sha256 != identity.runtime_artifacts.managed_gateway.sha256
        || credential.selected_backend_id != identity.backend_id
        || credential.bundle_backend_id != "cli:codex"
        || identity.runtime_family != "codex"
        || identity.orchestration_session_id.is_empty()
        || identity.retained_participant_id.is_empty()
        || identity.world_id.is_empty()
        || identity.world_generation == 0
    {
        return Err(WrongBinding);
    }
    let mut names = vec!["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN".to_string()];
    if credential.optional_account_id_present {
        names.push("SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID".to_string());
    }
    if credential.ordered_field_names != names {
        return Err(WrongBinding);
    }
    for (domain, key, field, object) in [
        (
            "substrate.e3.config-projection-identity.v1",
            "identity",
            "identity_hash",
            serde_json::to_value(identity),
        ),
        (
            "substrate.e3.in-world-gateway-identity.v1",
            "gateway",
            "gateway_identity_hash",
            serde_json::to_value(gateway),
        ),
        (
            "substrate.e3.credential-source-ref.v1",
            "credential_source",
            "ref_hash",
            serde_json::to_value(credential),
        ),
    ] {
        let mut object = object.map_err(|_| Malformed)?;
        let hash = object
            .as_object_mut()
            .ok_or(Malformed)?
            .remove(field)
            .ok_or(Malformed)?;
        if hash.as_str()
            != Some(
                crate::ConfigProjectionCodecV1::domain_sha256(
                    domain,
                    &serde_json::json!({key: object}),
                )?
                .as_str(),
            )
        {
            return Err(HashInvalid);
        }
    }
    let mut revision = crate::ConfigProjectionSecretHandoffRevisionV1 {
        schema_version: 1,
        authority_store_id: identity.authority_store_id.clone(),
        predecessor_ref: None,
        revision_hash: String::new(),
        handoff: crate::LaunchTimeSecretHandoffV1 {
            schema_version: 1,
            handoff_id,
            orchestration_session_id: identity.orchestration_session_id.clone(),
            world_id: identity.world_id.clone(),
            world_generation: identity.world_generation,
            retained_participant_id: Some(identity.retained_participant_id.clone()),
            runtime_family: identity.runtime_family.clone(),
            credential_source_ref: credential.clone(),
            receiving_gateway_ref: crate::InWorldGatewayRefV1 {
                authority_store_id: gateway.authority_store_id.clone(),
                gateway_instance_id: gateway.gateway_instance_id.clone(),
                gateway_identity_hash: gateway.gateway_identity_hash.clone(),
            },
            delivery: crate::SecretDeliveryMechanismV1::SecureFd {
                fd_name: "SUBSTRATE_LLM_AUTH_BUNDLE_FD".into(),
                one_time: true,
                gateway_receiver_only: true,
                deny_child_inheritance: true,
                close_after_consume: true,
            },
            created_at,
            expires_at,
            delivered_at: None,
            consumed_at: None,
            state_revision: 1,
            state: crate::SecretHandoffStateV1::Prepared,
            failure_diagnostic_ref: None,
        },
    };
    let mut value = serde_json::to_value(&revision).map_err(|_| Malformed)?;
    value
        .as_object_mut()
        .ok_or(Malformed)?
        .remove("revision_hash");
    revision.revision_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.config-projection-secret-handoff-revision.v1",
        &serde_json::json!({"revision": value}),
    )?;
    Ok(revision)
}

fn validate_preparation_abandonment_v1(
    publication: &E3PreparedRetainedLaunchPublicationV1,
    terminal_state: crate::SecretHandoffStateV1,
    released_at: &Timestamp,
    failure_diagnostic_ref: &Option<crate::RedactedDiagnosticRefV1>,
) -> Result<(), ConfigProjectionFailureV1> {
    use ConfigProjectionFailureV1::{Malformed, WrongBinding};

    if !matches!(
        terminal_state,
        crate::SecretHandoffStateV1::Failed | crate::SecretHandoffStateV1::Expired
    ) {
        return Err(WrongBinding);
    }
    validate_rfc3339_micros_utc_timestamp_v1(released_at)?;
    let handoff = &publication.prepared_handoff.handoff;
    let expected = build_prepared_secret_handoff_revision_v1(
        &publication.identity,
        &publication.gateway,
        &handoff.credential_source_ref,
        handoff.handoff_id.clone(),
        handoff.created_at.clone(),
        handoff.expires_at.clone(),
    )?;
    if expected != publication.prepared_handoff
        || secret_handoff_reference_v1(&expected)? != publication.prepared_handoff_ref
    {
        return Err(WrongBinding);
    }
    if let Some(diagnostic) = failure_diagnostic_ref {
        if diagnostic.schema_version != 1
            || diagnostic.authority_store_id != publication.identity.authority_store_id
            || diagnostic.diagnostic_id.is_empty()
            || diagnostic.diagnostic_hash.len() != 64
            || !diagnostic
                .diagnostic_hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(Malformed);
        }
    }
    match (
        &publication.record,
        &publication.published_head,
        &publication.held_consumer_lease,
    ) {
        (None, None, None) => Ok(()),
        (None, _, _) => Err(WrongBinding),
        (Some(_), _, _) => validate_prepared_retained_launch_publication_v1(publication),
    }
}

fn build_preparation_abandonment_successor_v1(
    predecessor: &crate::ConfigProjectionSecretHandoffRevisionV1,
    terminal_state: crate::SecretHandoffStateV1,
    failure_diagnostic_ref: Option<crate::RedactedDiagnosticRefV1>,
) -> Result<crate::ConfigProjectionSecretHandoffRevisionV1, ConfigProjectionFailureV1> {
    if matches!(
        predecessor.handoff.state,
        crate::SecretHandoffStateV1::Failed | crate::SecretHandoffStateV1::Expired
    ) {
        return if predecessor.handoff.state == terminal_state
            && predecessor.handoff.failure_diagnostic_ref == failure_diagnostic_ref
        {
            Ok(predecessor.clone())
        } else {
            Err(ConfigProjectionFailureV1::Conflict)
        };
    }
    if !matches!(
        predecessor.handoff.state,
        crate::SecretHandoffStateV1::Prepared | crate::SecretHandoffStateV1::Delivered
    ) {
        return Err(ConfigProjectionFailureV1::WrongBinding);
    }
    let mut successor = predecessor.clone();
    successor.predecessor_ref = Some(secret_handoff_reference_v1(predecessor)?);
    successor.handoff.state_revision = successor
        .handoff
        .state_revision
        .checked_add(1)
        .ok_or(ConfigProjectionFailureV1::Malformed)?;
    successor.handoff.state = terminal_state;
    successor.handoff.failure_diagnostic_ref = failure_diagnostic_ref;
    successor.revision_hash.clear();
    let mut value =
        serde_json::to_value(&successor).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
    value
        .as_object_mut()
        .ok_or(ConfigProjectionFailureV1::Malformed)?
        .remove("revision_hash");
    successor.revision_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.config-projection-secret-handoff-revision.v1",
        &serde_json::json!({"revision": value}),
    )?;
    Ok(successor)
}

fn secret_handoff_reference_v1(
    revision: &crate::ConfigProjectionSecretHandoffRevisionV1,
) -> Result<crate::SecretHandoffRefV1, ConfigProjectionFailureV1> {
    let handoff = &revision.handoff;
    Ok(crate::SecretHandoffRefV1 {
        authority_store_id: revision.authority_store_id.clone(),
        handoff_id: handoff.handoff_id.clone(),
        orchestration_session_id: handoff.orchestration_session_id.clone(),
        retained_participant_id: handoff
            .retained_participant_id
            .clone()
            .ok_or(ConfigProjectionFailureV1::WrongBinding)?,
        runtime_family: handoff.runtime_family.clone(),
        world_id: handoff.world_id.clone(),
        world_generation: handoff.world_generation,
        receiving_gateway_identity_hash: handoff
            .receiving_gateway_ref
            .gateway_identity_hash
            .clone(),
        handoff_state_revision: handoff.state_revision,
        handoff_hash: revision.revision_hash.clone(),
    })
}

fn validate_rfc3339_micros_utc_timestamp_v1(
    timestamp: &Timestamp,
) -> Result<(), ConfigProjectionFailureV1> {
    let parsed = chrono::DateTime::parse_from_rfc3339(&timestamp.0)
        .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
    if !timestamp.0.ends_with('Z')
        || timestamp.0 != parsed.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
    {
        return Err(ConfigProjectionFailureV1::Malformed);
    }
    Ok(())
}

fn validate_prepared_retained_launch_publication_v1(
    publication: &E3PreparedRetainedLaunchPublicationV1,
) -> Result<(), ConfigProjectionFailureV1> {
    use ConfigProjectionFailureV1::{HashInvalid, Malformed, PartialPublication, WrongBinding};
    let record = publication.record.as_ref().ok_or(PartialPublication)?;
    let groups = publication
        .ordered_child_cgroups
        .as_ref()
        .ok_or(PartialPublication)?;
    let boundary = publication
        .dormant_boundary
        .as_ref()
        .ok_or(PartialPublication)?;
    let intent = publication
        .activation_intent
        .as_ref()
        .ok_or(PartialPublication)?;
    let input = publication
        .launch_input
        .as_ref()
        .ok_or(PartialPublication)?;
    let crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } =
        &record.activation.publication_fence
    else {
        return Err(WrongBinding);
    };
    let reference = crate::ConfigProjectionRefV1 {
        authority_store_id: record.identity.authority_store_id.clone(),
        series_id: record.identity.series_id.clone(),
        record_id: record.record_id.clone(),
        revision: record.revision,
        record_hash: record.record_hash.clone(),
    };
    reference.validate().map_err(|_| Malformed)?;
    let gateway = &publication.gateway;
    let handoff = &publication.prepared_handoff.handoff;
    let gateway_ref = &handoff.receiving_gateway_ref;
    let boundary_ref = crate::GatewayAccessBoundaryRefV1 {
        authority_store_id: boundary.authority_store_id.clone(),
        access_boundary_id: boundary.access_boundary_id.clone(),
        revision: boundary.revision,
        boundary_hash: boundary.boundary_hash.clone(),
    };
    let intent_ref = crate::ManagedGatewayActivationIntentRefV1 {
        authority_store_id: intent.authority_store_id.clone(),
        activation_intent_id: intent.activation_intent_id.clone(),
        intent_hash: intent.intent_hash.clone(),
    };
    if record.identity != publication.identity
        || record.schema_version != 1
        || record.created_at != handoff.created_at
        || record.managed_gateway.posture != crate::ManagedGatewayProjectionPostureV1::Dormant
        || record.nonsecret_handoff.secret_handoff_ref != publication.prepared_handoff_ref
        || record.nonsecret_handoff.credential_source_ref != handoff.credential_source_ref
        || record.nonsecret_handoff.delivery != handoff.delivery
        || record.nonsecret_handoff.receiving_gateway_ref != *gateway_ref
        || record.nonsecret_handoff.observed_state != crate::SecretHandoffStateV1::Prepared
        || record.nonsecret_handoff.activation_ack_ref.is_some()
        || record.managed_gateway.expected_gateway_ref != *gateway_ref
        || record.managed_gateway.access_boundary_ref != boundary_ref
        || record.managed_gateway.activation_intent_ref != intent_ref
        || record.activation.activation_intent_ref != intent_ref
        || record.activation.gateway_activation_ack_ref.is_some()
        || record.activation.released_at.is_some()
        || boundary.authority_store_id != record.identity.authority_store_id
        || boundary.gateway_instance_id != gateway.gateway_instance_id
        || boundary.config_projection_identity_hash != record.identity.identity_hash
        || boundary.access_boundary_id != gateway.access_boundary_id
        || boundary.orchestration_session_id != record.identity.orchestration_session_id
        || boundary.retained_participant_id != record.identity.retained_participant_id
        || boundary.backend_id != record.identity.backend_id
        || boundary.world_id != record.identity.world_id
        || boundary.world_generation != record.identity.world_generation
        || boundary.posture != crate::GatewayAccessPostureV1::DenyAllDormant
        || boundary.revision != 1
        || boundary.predecessor_ref.is_some()
        || intent.preparation_id != handoff.credential_source_ref.preparation_id
        || intent.authority_store_id != record.identity.authority_store_id
        || intent.config_projection_identity_hash != record.identity.identity_hash
        || intent.dormant_record_id != record.record_id
        || intent.dormant_revision != record.revision
        || intent.expected_gateway_ref != *gateway_ref
        || intent.expected_gateway_artifact != record.identity.runtime_artifacts.managed_gateway
        || intent.expected_access_boundary_ref != boundary_ref
        || intent.secret_handoff_ref != publication.prepared_handoff_ref
        || intent.fence_id != *fence_id
        || intent.created_at != record.created_at
        || input.authority_store_id != record.identity.authority_store_id
        || input.dormant_projection_ref != reference
        || input.activation_intent_ref != intent_ref
        || input.gateway_ref != *gateway_ref
        || input.config_projection_identity_hash != record.identity.identity_hash
        || input.orchestration_session_id != record.identity.orchestration_session_id
        || input.retained_participant_id != record.identity.retained_participant_id
        || input.backend_id != record.identity.backend_id
        || input.world_id != record.identity.world_id
        || input.world_generation != record.identity.world_generation
        || input.listener_identity != boundary.gateway_listener
        || input.access_boundary_ref != boundary_ref
        || input.secret_handoff_prepared_ref != publication.prepared_handoff_ref
        || input.readiness_nonce != intent.readiness_nonce
    {
        return Err(WrongBinding);
    }
    for (group, role) in groups.iter().zip([
        crate::E3TerminalProcessRoleV1::ManagedGateway,
        crate::E3TerminalProcessRoleV1::ReadinessProbe,
        crate::E3TerminalProcessRoleV1::Codex,
    ]) {
        if group.role != role
            || group.series_id != record.identity.series_id
            || group.authority_store_id != record.identity.authority_store_id
            || group.fence_id != *fence_id
            || group.turn_id.is_some()
            || group.kernel_boot_id != groups[0].kernel_boot_id
        {
            return Err(WrongBinding);
        }
    }
    if groups[1].cgroup != boundary.readiness_probe_cgroup
        || groups[2].cgroup != boundary.allowed_member_cgroup
    {
        return Err(WrongBinding);
    }
    for (domain, key, field, object) in [
        (
            "substrate.e3.agent-config-projection-record.v1",
            "record",
            "record_hash",
            serde_json::to_value(record),
        ),
        (
            "substrate.e3.logical-config-projection.v1",
            "projection",
            "projection_hash",
            serde_json::to_value(&record.logical),
        ),
        (
            "substrate.e3.effective-config-projection.v1",
            "projection",
            "projection_hash",
            serde_json::to_value(&record.effective),
        ),
        (
            "substrate.e3.native-config-projection.v1",
            "projection",
            "projection_hash",
            serde_json::to_value(&record.native),
        ),
        (
            "substrate.e3.managed-gateway-projection.v1",
            "projection",
            "projection_hash",
            serde_json::to_value(&record.managed_gateway),
        ),
        (
            "substrate.e3.nonsecret-handoff-projection.v1",
            "projection",
            "projection_hash",
            serde_json::to_value(&record.nonsecret_handoff),
        ),
        (
            "substrate.e3.gateway-access-boundary.v1",
            "boundary",
            "boundary_hash",
            serde_json::to_value(boundary),
        ),
        (
            "substrate.e3.managed-gateway-activation-intent.v1",
            "intent",
            "intent_hash",
            serde_json::to_value(intent),
        ),
        (
            "substrate.e3.managed-gateway-launch-input.v1",
            "launch_input",
            "launch_input_hash",
            serde_json::to_value(input),
        ),
    ]
    .into_iter()
    .chain(groups.iter().map(|group| {
        (
            "substrate.e3.child-cgroup-registration.v1",
            "cgroup_registration",
            "cgroup_registration_hash",
            serde_json::to_value(group),
        )
    })) {
        let mut object = object.map_err(|_| Malformed)?;
        let hash = object
            .as_object_mut()
            .ok_or(Malformed)?
            .remove(field)
            .ok_or(Malformed)?;
        if hash.as_str()
            != Some(
                crate::ConfigProjectionCodecV1::domain_sha256(
                    domain,
                    &serde_json::json!({key:object}),
                )?
                .as_str(),
            )
        {
            return Err(HashInvalid);
        }
    }
    Ok(())
}

fn build_prepared_response_v1(
    publication: &E3PreparedRetainedLaunchPublicationV1,
) -> Result<transport_api_types::E3ConfigProjectionPrepareResponseV1, ConfigProjectionFailureV1> {
    use ConfigProjectionFailureV1::PartialPublication;
    let head = publication
        .published_head
        .as_ref()
        .ok_or(PartialPublication)?;
    let lease = publication
        .held_consumer_lease
        .as_ref()
        .ok_or(PartialPublication)?;
    let intent = publication
        .activation_intent
        .as_ref()
        .ok_or(PartialPublication)?;
    let handoff = &publication.prepared_handoff.handoff;
    let mut response = transport_api_types::E3ConfigProjectionPrepareResponseV1 {
        schema_version: 1,
        preparation_id: handoff.credential_source_ref.preparation_id.clone(),
        preparation_idempotency_key: publication.preparation_idempotency_key.clone(),
        prepared_at: handoff.created_at.0.clone(),
        expires_at: handoff.expires_at.0.clone(),
        response_hash: String::new(),
        config_projection: transport_api_types::ConfigProjectionActivationCarrierV1 {
            authority_store_id: head.authority_store_id.clone(),
            series_id: head.series_id.clone(),
            dormant_projection_ref: head.clone(),
            activation_intent_ref: crate::ManagedGatewayActivationIntentRefV1 {
                authority_store_id: intent.authority_store_id.clone(),
                activation_intent_id: intent.activation_intent_id.clone(),
                intent_hash: intent.intent_hash.clone(),
            },
            expected_gateway_ref: handoff.receiving_gateway_ref.clone(),
            fence_id: intent.fence_id.clone(),
            consumer_id: lease.consumer_id.clone(),
            consumer_lease_revision: lease.revision,
            consumer_lease_hash: lease.lease_hash.clone(),
        },
    };
    let mut value =
        serde_json::to_value(&response).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
    value
        .as_object_mut()
        .ok_or(ConfigProjectionFailureV1::Malformed)?
        .remove("response_hash");
    response.response_hash = crate::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.config-projection-prepare-response.v1",
        &serde_json::json!({"response":value}),
    )?;
    response
        .validate()
        .map_err(|_| ConfigProjectionFailureV1::WrongBinding)?;
    Ok(response)
}
