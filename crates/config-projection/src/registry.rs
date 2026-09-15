#[cfg(target_os = "linux")]
mod linux {
    use std::collections::{BTreeMap, BTreeSet};
    use std::ffi::{CStr, CString};
    use std::fs::File;
    use std::io::{Read, Write};
    use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, RawFd};
    use std::path::{Component, Path};
    use std::sync::{Arc, Mutex, OnceLock};

    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use serde::de::{DeserializeOwned, IgnoredAny, MapAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serialize};
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    use crate::{
        AgentConfigProjectionRecordV1, AgentInventorySourceMaterialV1, CanonicalDirectoryV1,
        Codex0125NativeSourceObservationV1, Codex0125ProjectionPlanV1, Codex0125ProjectionV1,
        ConfigProjectionAuthoringInputRefV1, ConfigProjectionCodecV1,
        ConfigProjectionConsumerKindV1, ConfigProjectionConsumerLeasePostureV1,
        ConfigProjectionConsumerLeaseV1, ConfigProjectionFailureV1, ConfigProjectionHeadV1,
        ConfigProjectionIdentityV1, ConfigProjectionRefV1, ConfigProjectionRetirementV1,
        ConfigProjectionStoreV1, ConfigProjectionSubjectBindingV1, DirectoryPhysicalIdentityV1,
        E3ChildCgroupRegistrationV1, E3ChildProcessRegistrationV1, E3ConfigExplainOriginKindV1,
        E3KernelEffectIntentRefV1, E3KernelEffectIntentV1, E3KernelEffectKindV1,
        E3KernelEffectResolutionDispositionV1, E3KernelEffectResolutionV1,
        E3TerminalChildQuiescenceEvidenceV1, EffectiveSubstrateConfigSourceV1,
        GatewayAccessBoundaryRefV1, GatewayAccessBoundaryV1, GatewayAccessPostureV1,
        InstalledAcceptedHomeBootstrapHeadV1, InstalledAcceptedHomeBootstrapRecordV1,
        ManagedGatewayProjectionPostureV1, NativeAgentConfigProjectionV1,
        NativeProjectedFileRoleV1, NativeProjectionSourceManifestV1, NftablesRuleRoleV1,
        RuntimeArtifactAuthorityRoleV1, RuntimeArtifactProvenanceV1, SecretDeliveryMechanismV1,
        SecretHandoffStateV1, Timestamp, TrustedRuntimeArtifactManifestV1,
    };

    const CHILD_NAME: &str = "agent-config-projection-v1";
    const MAX_AUTHORITY_OBJECT_BYTES: u64 = 16 * 1024 * 1024;
    const OPENAT2_RESOLVE: u64 = libc::RESOLVE_BENEATH
        | libc::RESOLVE_NO_MAGICLINKS
        | libc::RESOLVE_NO_SYMLINKS
        | libc::RESOLVE_NO_XDEV;
    static LIVE_CAPABILITIES: OnceLock<Mutex<BTreeMap<(String, String), usize>>> = OnceLock::new();

    type KernelEffectApplicationV1<'a> = dyn FnMut(
            &E3KernelEffectIntentRefV1,
        ) -> Result<Option<E3ChildCgroupRegistrationV1>, ConfigProjectionFailureV1>
        + 'a;

    /// Object-safe bridge to shell's sealed HSA parent transaction.
    pub trait ConfigProjectionHsaAuthorityV1: Send + Sync {
        fn with_locked_parent(
            &self,
            operation: &mut dyn for<'fd> FnMut(
                BorrowedFd<'fd>,
            )
                -> Result<(), ConfigProjectionFailureV1>,
        ) -> Result<(), ConfigProjectionFailureV1>;
    }

    /// Sealed installed accepted-home capability.
    #[derive(Debug)]
    pub struct ConfiguredAcceptedHomeAuthorityV1 {
        installed_record: InstalledAcceptedHomeBootstrapRecordV1,
        accepted_home_descriptor: File,
    }

    impl ConfiguredAcceptedHomeAuthorityV1 {
        pub fn from_installed_bootstrap_authority() -> Result<Self, ConfigProjectionFailureV1> {
            let root = open_absolute_directory(Path::new(
                "/var/lib/substrate/install-bootstrap-authority-v1",
            ))?;
            let substrate_gid = group_gid("substrate")?;
            verify_directory(&root, 0, 0o750)?;
            verify_group(&root, substrate_gid)?;
            let lock = open_file_at(root.as_fd(), "lock", libc::O_RDONLY, 0)?;
            verify_regular_file(&lock, 0, 0o600)?;
            flock(&lock, libc::LOCK_SH)?;
            let result = (|| {
                let mut active_file = open_file_at(root.as_fd(), "active.json", libc::O_RDONLY, 0)?;
                verify_regular_file(&active_file, 0, 0o640)?;
                verify_group(&active_file, substrate_gid)?;
                let active: InstalledAcceptedHomeBootstrapHeadV1 =
                    read_canonical_file(&mut active_file)?;
                validate_installed_head(&active)?;
                let records = open_directory_raw_at(root.as_fd(), "records")?;
                verify_directory(&records, 0, 0o750)?;
                verify_group(&records, substrate_gid)?;
                let record_name = format!("{}.json", active.head_record_hash);
                let mut record_file =
                    open_file_at(records.as_fd(), &record_name, libc::O_RDONLY, 0)?;
                verify_regular_file(&record_file, 0, 0o640)?;
                verify_group(&record_file, substrate_gid)?;
                let record: InstalledAcceptedHomeBootstrapRecordV1 =
                    read_canonical_file(&mut record_file)?;
                validate_installed_record(&record)?;
                if record.record_hash != active.head_record_hash {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Self::from_validated_record(record)
            })();
            let _ = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_UN) };
            result
        }

        #[cfg(any(test, feature = "test-support"))]
        pub fn from_record_for_test(
            record: InstalledAcceptedHomeBootstrapRecordV1,
        ) -> Result<Self, ConfigProjectionFailureV1> {
            validate_installed_record(&record)?;
            Self::from_validated_record(record)
        }

        fn from_validated_record(
            record: InstalledAcceptedHomeBootstrapRecordV1,
        ) -> Result<Self, ConfigProjectionFailureV1> {
            validate_installed_carrier_binding(&record)?;
            let accepted_home_descriptor =
                open_absolute_directory(Path::new(&record.accepted_home.physical_path))?;
            verify_directory(
                &accepted_home_descriptor,
                checked_uid(record.intended_uid)?,
                0o700,
            )?;
            record
                .accepted_home
                .revalidate_linux_from_fd(accepted_home_descriptor.as_fd())?;
            Ok(Self {
                installed_record: record,
                accepted_home_descriptor,
            })
        }

        pub fn revalidate(&self) -> Result<(), ConfigProjectionFailureV1> {
            validate_installed_record(&self.installed_record)?;
            validate_installed_carrier_binding(&self.installed_record)?;
            verify_directory(
                &self.accepted_home_descriptor,
                checked_uid(self.installed_record.intended_uid)?,
                0o700,
            )?;
            self.installed_record
                .accepted_home
                .revalidate_linux_from_fd(self.accepted_home_descriptor.as_fd())
        }

        pub fn accepted_home(&self) -> &CanonicalDirectoryV1 {
            &self.installed_record.accepted_home
        }

        pub fn intended_uid(&self) -> u64 {
            self.installed_record.intended_uid
        }
    }

    impl CanonicalDirectoryV1 {
        pub fn capture_linux_from_fd(
            descriptor: BorrowedFd<'_>,
        ) -> Result<Self, ConfigProjectionFailureV1> {
            let metadata = fstat(descriptor.as_raw_fd())?;
            if metadata.st_mode & libc::S_IFMT != libc::S_IFDIR
                || metadata.st_dev == 0
                || metadata.st_ino == 0
            {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            let link = std::fs::read_link(format!("/proc/self/fd/{}", descriptor.as_raw_fd()))
                .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
            let physical_path = link
                .to_str()
                .filter(|path| path.starts_with('/') && !path.ends_with(" (deleted)"))
                .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?
                .to_string();
            let reopened = open_absolute_directory(Path::new(&physical_path))?;
            let reopened_metadata = fstat(reopened.as_raw_fd())?;
            if reopened_metadata.st_dev != metadata.st_dev
                || reopened_metadata.st_ino != metadata.st_ino
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            Ok(Self {
                physical_path,
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: metadata.st_dev,
                    inode: metadata.st_ino,
                },
            })
        }

        pub fn revalidate_linux_from_fd(
            &self,
            descriptor: BorrowedFd<'_>,
        ) -> Result<(), ConfigProjectionFailureV1> {
            let current = Self::capture_linux_from_fd(descriptor)?;
            if &current == self {
                Ok(())
            } else {
                Err(ConfigProjectionFailureV1::WrongBinding)
            }
        }
    }

    /// Nonsecret recovery observations; these do not confer launch or lease authority.
    pub struct ConfigProjectionRecoveryReadbackV1 {
        pub store: ConfigProjectionStoreV1,
        pub subject: Option<ConfigProjectionSubjectReadbackV1>,
        pub kernel_effects: Vec<E3KernelEffectRecoveryV1>,
    }

    #[allow(
        clippy::large_enum_variant,
        reason = "the contract requires an owned, unboxed metadata result"
    )]
    pub enum ConfigProjectionSubjectReadbackV1 {
        Unbound,
        Bound(ConfigProjectionPreparationMetadataV1),
    }

    pub struct ConfigProjectionPreparationMetadataV1 {
        pub(crate) record: AgentConfigProjectionRecordV1,
        pub(crate) projection_ref: ConfigProjectionRefV1,
        pub(crate) prepared_handoff: crate::ConfigProjectionSecretHandoffRevisionV1,
        pub(crate) current_handoff: crate::ConfigProjectionSecretHandoffRevisionV1,
        pub(crate) consumer_lease: Option<ConfigProjectionConsumerLeaseV1>,
    }

    impl ConfigProjectionPreparationMetadataV1 {
        pub fn record(&self) -> &AgentConfigProjectionRecordV1 {
            &self.record
        }
        pub fn projection_ref(&self) -> &ConfigProjectionRefV1 {
            &self.projection_ref
        }
    }

    pub struct E3KernelEffectRecoveryV1 {
        pub intent: E3KernelEffectIntentV1,
        pub resolution: Option<E3KernelEffectResolutionV1>,
        pub child_cgroup: Option<E3ChildCgroupRegistrationV1>,
        pub child_processes: Vec<E3ChildProcessRegistrationV1>,
        pub boundary: Option<GatewayAccessBoundaryV1>,
        pub projection: Option<AgentConfigProjectionRecordV1>,
        pub terminal_child_evidence: Vec<E3TerminalChildQuiescenceEvidenceV1>,
        pub gateway_config: Option<crate::GatewayRuntimeConfigIdentityV1>,
    }

    /// Durable E3-B registry. HSA access exists only behind `parent`.
    pub struct ConfigProjectionRegistryV1 {
        parent: Arc<dyn ConfigProjectionHsaAuthorityV1>,
    }

    impl ConfigProjectionRegistryV1 {
        pub fn open(
            parent: Arc<dyn ConfigProjectionHsaAuthorityV1>,
        ) -> Result<Self, ConfigProjectionFailureV1> {
            Ok(Self { parent })
        }

        pub fn recover(
            &self,
            subject: Option<&ConfigProjectionIdentityV1>,
        ) -> Result<ConfigProjectionRecoveryReadbackV1, ConfigProjectionFailureV1> {
            self.with_transaction(subject.is_some(), |transaction| {
                let store = if subject.is_some() {
                    require_store(transaction)?
                } else {
                    ensure_store(transaction, None)?
                };
                let mut kernel_effects = validate_registry_tree(transaction, true)?;
                let subject = subject
                    .map(|identity| {
                        read_preparation_subject_in_transaction_v1(
                            transaction,
                            identity,
                            &kernel_effects,
                        )
                    })
                    .transpose()?;
                if let Some(readback) = &subject {
                    kernel_effects.retain(|effect| match readback {
                        ConfigProjectionSubjectReadbackV1::Unbound => false,
                        ConfigProjectionSubjectReadbackV1::Bound(metadata) => {
                            effect.intent.series_id == metadata.record.identity.series_id
                                && effect.intent.preparation_id
                                    == metadata
                                        .prepared_handoff
                                        .handoff
                                        .credential_source_ref
                                        .preparation_id
                        }
                    });
                }
                Ok(ConfigProjectionRecoveryReadbackV1 {
                    store,
                    subject,
                    kernel_effects,
                })
            })
        }

        pub fn resolve(
            &self,
            projection: Option<(
                &ConfigProjectionIdentityV1,
                &ConfigProjectionConsumerLeaseV1,
            )>,
            authoring_input: Option<&ConfigProjectionAuthoringInputRefV1>,
        ) -> Result<ProjectionAndAuthoringReadbackV1, ConfigProjectionFailureV1> {
            if projection.is_none() && authoring_input.is_none() {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            let parent = Arc::clone(&self.parent);
            self.with_transaction(false, |transaction| {
                let inputs = authoring_input
                    .map(|reference| resolve_authoring_input_in_transaction(transaction, reference))
                    .transpose()?;
                let resolved = projection
                    .map(|(identity, held_lease)| {
                        resolve_projection_in_transaction(transaction, parent, identity, held_lease)
                    })
                    .transpose()?;
                Ok((resolved, inputs))
            })
        }

        pub fn publish_dormant(
            &self,
            record: &AgentConfigProjectionRecordV1,
            gateway: Option<PreparedGatewayChainV1<'_>>,
        ) -> Result<ConfigProjectionRefV1, ConfigProjectionFailureV1> {
            if let Some(chain) = gateway {
                self.with_transaction(false, |transaction| {
                    let store = require_store(transaction)?;
                    validate_record(record, &store)?;
                    validate_prepared_gateway_chain(transaction, record, chain)?;
                    if let Some(expected) = &record.predecessor_ref {
                        let predecessor = read_record(transaction, expected)?;
                        validate_preparation_cleanup_in_transaction_v1(
                            transaction,
                            &predecessor,
                            Some(record),
                        )?;
                        let intents =
                            open_directory_at(transaction.root.as_fd(), "gateway-intents")?;
                        let old: crate::ManagedGatewayActivationIntentV1 = read_canonical_at(
                            intents.as_fd(),
                            &format!(
                                "{}.json",
                                predecessor
                                    .managed_gateway
                                    .activation_intent_ref
                                    .activation_intent_id
                            ),
                        )?;
                        validate_gateway_intent(&old, &store)?;
                        if old.intent_hash
                            != predecessor
                                .managed_gateway
                                .activation_intent_ref
                                .intent_hash
                            || old.readiness_nonce == chain.3.readiness_nonce
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    }
                    publish_prepared_gateway_chain(transaction, chain)?;
                    let crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } =
                        &record.activation.publication_fence
                    else {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    };
                    validate_native_projection_source_input(
                        &record.native,
                        &record.identity.series_id,
                        fence_id,
                    )?;
                    let native_sources =
                        open_directory_at(transaction.root.as_fd(), "native-sources")?;
                    let native_series =
                        open_directory_at(native_sources.as_fd(), &record.identity.series_id)?;
                    let config_bytes = STANDARD
                        .decode(&record.native.files[0].bytes_base64)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
                    let manifest = validate_native_source_directory(
                        &native_series,
                        fence_id,
                        None,
                        Some(&config_bytes),
                        transaction.owner_uid,
                        transaction.native_source_gid,
                    )?;
                    if manifest.authority_store_id != store.authority_store_id
                        || manifest.series_id != record.identity.series_id
                        || manifest.fence_id != *fence_id
                        || manifest.source_root.physical_path
                            != format!(
                                "{}/authority-v1/agent-config-projection-v1/native-sources/{}/{}",
                                store.accepted_home.physical_path,
                                record.identity.series_id,
                                fence_id
                            )
                        || manifest.native_projection_hash != record.native.projection_hash
                        || manifest.ordered_file_hashes != [record.native.files[0].sha256.clone()]
                        || manifest.created_at != record.created_at
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    let source_root = open_directory_at(native_series.as_fd(), fence_id)?;
                    let codex_home = open_directory_at(source_root.as_fd(), "codex-home")?;
                    let system_empty = open_directory_at(source_root.as_fd(), "system-empty")?;
                    verify_native_source_directories(
                        &native_series,
                        fence_id,
                        &source_root,
                        &codex_home,
                        &system_empty,
                        transaction.owner_uid,
                        transaction.native_source_gid,
                    )?;
                    let codex_home_metadata = fstat(codex_home.as_raw_fd())?;
                    let system_empty_metadata = fstat(system_empty.as_raw_fd())?;
                    let (observed_bytes, first_config) = read_native_source_file(
                        codex_home.as_fd(),
                        "config.toml",
                        transaction.owner_uid,
                        transaction.native_source_gid,
                    )?;
                    if first_config.st_uid as u64 != record.native.root.owner_uid
                        || first_config.st_gid as u64 != record.native.root.owner_gid
                        || first_config.st_mode & 0o7777 != record.native.files[0].mode
                        || first_config.st_size as u64 != record.native.files[0].byte_length
                        || observed_bytes != config_bytes
                    {
                        return Err(ConfigProjectionFailureV1::Conflict);
                    }
                    verify_native_source_directories(
                        &native_series,
                        fence_id,
                        &source_root,
                        &codex_home,
                        &system_empty,
                        transaction.owner_uid,
                        transaction.native_source_gid,
                    )?;
                    let source_system = CanonicalDirectoryV1 {
                        physical_path: format!(
                            "{}/system-empty",
                            manifest.source_root.physical_path
                        ),
                        physical_identity: DirectoryPhysicalIdentityV1::Linux {
                            device_id: system_empty_metadata.st_dev,
                            inode: system_empty_metadata.st_ino,
                        },
                    };
                    let source_user = CanonicalDirectoryV1 {
                        physical_path: format!("{}/codex-home", manifest.source_root.physical_path),
                        physical_identity: DirectoryPhysicalIdentityV1::Linux {
                            device_id: codex_home_metadata.st_dev,
                            inode: codex_home_metadata.st_ino,
                        },
                    };
                    for input in &record.native.ambient_closure.inputs {
                        match input.layer.as_str() {
                            "System" if input.directory == source_system => {}
                            "User" if input.directory == source_user => {
                                if input.relative_path == "config.toml"
                                    && (input.device_id != Some(first_config.st_dev)
                                        || input.inode != Some(first_config.st_ino)
                                        || input.byte_length != Some(first_config.st_size as u64)
                                        || input.sha256.as_deref()
                                            != Some(record.native.files[0].sha256.as_str()))
                                {
                                    return Err(ConfigProjectionFailureV1::WrongBinding);
                                }
                            }
                            "Project" => {}
                            "McpCredentials" | "Auth" | "CloudRequirements"
                                if input.directory == source_user => {}
                            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                        }
                    }
                    let launch_inputs =
                        open_directory_at(transaction.root.as_fd(), "gateway-launch-inputs")?;
                    write_immutable(
                        launch_inputs.as_fd(),
                        &format!("{}.json", chain.4.launch_input_id),
                        &ConfigProjectionCodecV1::encode_canonical_json(chain.4)?,
                        "gateway-launch-input",
                        transaction.owner_uid,
                    )?;
                    // The active head is the publication boundary: every immutable dependency
                    // must already be durable, including the exact gateway launch input. A
                    // conflicting input cannot leave a lease-acquirable Dormant head behind.
                    publish_record_in_transaction(
                        transaction,
                        record,
                        record.predecessor_ref.as_ref(),
                        ManagedGatewayProjectionPostureV1::Dormant,
                    )
                })
            } else {
                self.publish(
                    record,
                    record.predecessor_ref.as_ref(),
                    ManagedGatewayProjectionPostureV1::Dormant,
                )
            }
        }

        pub fn publish_ready_closed(
            &self,
            expected_head: &ConfigProjectionRefV1,
            record: &AgentConfigProjectionRecordV1,
            ack: &crate::ManagedGatewayActivationAckV1,
            held_lease: &ConfigProjectionConsumerLeaseV1,
        ) -> Result<ConfigProjectionRefV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                validate_gateway_activation_authority_v1(
                    transaction,
                    expected_head,
                    Some(record),
                    held_lease,
                )?;
                publish_gateway_ack_in_transaction_v1(transaction, record, ack)?;
                let reference = publish_record_in_transaction(
                    transaction,
                    record,
                    Some(expected_head),
                    ManagedGatewayProjectionPostureV1::ReadyClosed,
                )?;
                if read_series_head(transaction, &reference.series_id)?.head_ref != reference
                    || read_record(transaction, &reference)? != *record
                {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                validate_gateway_activation_authority_v1(
                    transaction,
                    expected_head,
                    Some(record),
                    held_lease,
                )?;
                Ok(reference)
            })
        }

        pub fn publish_active(
            &self,
            expected_head: &ConfigProjectionRefV1,
            record: &AgentConfigProjectionRecordV1,
        ) -> Result<ConfigProjectionRefV1, ConfigProjectionFailureV1> {
            self.publish(
                record,
                Some(expected_head),
                ManagedGatewayProjectionPostureV1::Active,
            )
        }

        pub(crate) fn publish_handoff_transition_v1(
            &self,
            identity: &ConfigProjectionIdentityV1,
            fence_id: &str,
            successor: &crate::ConfigProjectionSecretHandoffRevisionV1,
            expected_projection_ref: Option<&ConfigProjectionRefV1>,
            activation_consumer_lease: Option<&ConfigProjectionConsumerLeaseV1>,
        ) -> Result<crate::SecretHandoffRefV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let activation = matches!(
                    successor.handoff.state,
                    SecretHandoffStateV1::Delivered | SecretHandoffStateV1::Consumed
                );
                if activation {
                    let expected =
                        expected_projection_ref.ok_or(ConfigProjectionFailureV1::WrongBinding)?;
                    let lease =
                        activation_consumer_lease.ok_or(ConfigProjectionFailureV1::WrongBinding)?;
                    validate_gateway_activation_authority_v1(transaction, expected, None, lease)?;
                    let dormant = read_record(transaction, expected)?;
                    if dormant.identity != *identity
                        || dormant.activation.publication_fence
                            != (crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                                fence_id: fence_id.into(),
                            })
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    return publish_handoff_transition_in_transaction(
                        transaction,
                        identity,
                        fence_id,
                        successor,
                    );
                }
                if activation_consumer_lease.is_some() {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                if let Some(expected) = expected_projection_ref {
                    let head = read_series_head(transaction, &identity.series_id)?;
                    if head.head_ref != *expected {
                        return Err(ConfigProjectionFailureV1::StaleRevision);
                    }
                    let record = read_record(transaction, expected)?;
                    if record.identity != *identity
                        || record_ref(&record) != *expected
                        || record.managed_gateway.posture
                            == ManagedGatewayProjectionPostureV1::Active
                        || record.activation.publication_fence
                            != (crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                                fence_id: fence_id.into(),
                            })
                        || record.nonsecret_handoff.credential_source_ref
                            != successor.handoff.credential_source_ref
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    validate_preparation_cleanup_in_transaction_v1(transaction, &record, None)?;
                }
                publish_handoff_transition_in_transaction(
                    transaction,
                    identity,
                    fence_id,
                    successor,
                )
            })
        }

        pub fn import_runtime_artifacts(
            &self,
            effective_config: &EffectiveSubstrateConfigSourceV1,
            agent_inventory: &AgentInventorySourceMaterialV1,
            created_at: Timestamp,
        ) -> Result<ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                validate_effective_config_source(effective_config, &store)?;
                validate_agent_inventory_source(agent_inventory, effective_config, &store)?;
                crate::LinuxArtifactSourceV1::import_manifest(
                    Path::new("/var/lib/substrate/runtime-artifacts-v1"),
                    Path::new("/var/lib/substrate/world-deps/runtime-artifacts-v1"),
                    &store.authority_store_id,
                    created_at,
                    |artifact_manifest| {
                        validate_runtime_artifact_manifest(artifact_manifest, &store)?;
                        publish_authoring_inputs(
                            transaction,
                            &store,
                            effective_config,
                            agent_inventory,
                            artifact_manifest,
                        )
                    },
                )
            })
        }

        pub fn publish_native_source(
            &self,
            series_id: &str,
            fence_id: &str,
            plan: &Codex0125ProjectionPlanV1,
            created_at: Timestamp,
        ) -> Result<
            (
                NativeAgentConfigProjectionV1,
                NativeProjectionSourceManifestV1,
            ),
            ConfigProjectionFailureV1,
        > {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                validate_timestamp(&created_at)?;
                publish_native_source_directory(
                    transaction,
                    &store,
                    series_id,
                    fence_id,
                    plan,
                    created_at,
                )
            })
        }

        pub fn acquire_consumer_lease(
            &self,
            acquired_projection_ref: &ConfigProjectionRefV1,
            consumer_kind: ConfigProjectionConsumerKindV1,
            acquired_at: Timestamp,
            prepared_consumer_id: Option<&str>,
        ) -> Result<ConfigProjectionConsumerLeaseV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                let head = read_series_head(transaction, &acquired_projection_ref.series_id)?;
                if read_retirement(transaction, &acquired_projection_ref.series_id)?.is_some() {
                    return Err(ConfigProjectionFailureV1::RetiredSeries);
                }
                if head.head_ref != *acquired_projection_ref {
                    return Err(ConfigProjectionFailureV1::StaleRevision);
                }
                validate_timestamp(&acquired_at)?;
                if let Some(consumer_id) = prepared_consumer_id {
                    return acquire_or_resolve_prepared_consumer_lease_v1(
                        transaction,
                        acquired_projection_ref,
                        consumer_kind,
                        acquired_at,
                        consumer_id,
                    );
                }
                let consumer_id = prefixed_uuid("cpc_");
                let lease_dir = open_lease(
                    transaction,
                    &acquired_projection_ref.series_id,
                    &consumer_id,
                    true,
                )?;
                let revisions = open_or_create_directory(
                    lease_dir.as_fd(),
                    "revisions",
                    transaction.owner_uid,
                )?;
                let mut lease = ConfigProjectionConsumerLeaseV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id,
                    series_id: acquired_projection_ref.series_id.clone(),
                    consumer_id,
                    consumer_kind,
                    revision: 1,
                    predecessor_lease_hash: None,
                    acquired_projection_ref: acquired_projection_ref.clone(),
                    posture: ConfigProjectionConsumerLeasePostureV1::Held,
                    acquired_at,
                    released_at: None,
                    lease_hash: String::new(),
                };
                lease.lease_hash = hash_omitting(
                    "substrate.e3.config-projection-consumer-lease.v1",
                    "lease",
                    &lease,
                    "lease_hash",
                )?;
                validate_lease(&lease, &head.head_ref)?;
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(&lease)?;
                write_immutable(
                    revisions.as_fd(),
                    "00000000000000000001.json",
                    &bytes,
                    "lease",
                    transaction.owner_uid,
                )?;
                cas_head(
                    lease_dir.as_fd(),
                    None,
                    &bytes,
                    "head",
                    transaction.owner_uid,
                )?;
                Ok(lease)
            })
        }

        pub fn release_consumer_lease(
            &self,
            held: &ConfigProjectionConsumerLeaseV1,
            released_at: Timestamp,
        ) -> Result<ConfigProjectionConsumerLeaseV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                if held.authority_store_id != store.authority_store_id
                    || held.posture != ConfigProjectionConsumerLeasePostureV1::Held
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                validate_timestamp(&released_at)?;
                let mut released = held.clone();
                released.revision = held
                    .revision
                    .checked_add(1)
                    .ok_or(ConfigProjectionFailureV1::Malformed)?;
                released.predecessor_lease_hash = Some(held.lease_hash.clone());
                released.posture = ConfigProjectionConsumerLeasePostureV1::Released;
                released.released_at = Some(released_at);
                released.lease_hash.clear();
                released.lease_hash = hash_omitting(
                    "substrate.e3.config-projection-consumer-lease.v1",
                    "lease",
                    &released,
                    "lease_hash",
                )?;
                validate_lease(&released, &held.acquired_projection_ref)?;
                let lease_dir = open_lease(transaction, &held.series_id, &held.consumer_id, false)?;
                let current_bytes = read_file_at(lease_dir.as_fd(), "head.json")?;
                let current: ConfigProjectionConsumerLeaseV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(&current_bytes)?;
                if current == released {
                    return Ok(current);
                }
                if &current != held {
                    return Err(ConfigProjectionFailureV1::StaleRevision);
                }
                let revisions = open_directory_at(lease_dir.as_fd(), "revisions")?;
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(&released)?;
                let name = format!("{:020}.json", released.revision);
                write_immutable(
                    revisions.as_fd(),
                    &name,
                    &bytes,
                    "lease",
                    transaction.owner_uid,
                )?;
                cas_head(
                    lease_dir.as_fd(),
                    Some(&current_bytes),
                    &bytes,
                    "head",
                    transaction.owner_uid,
                )?;
                Ok(released)
            })
        }

        /// Publish and read back the intent before applying its bounded kernel effect under the
        /// same parent and child locks. `None` performs immutable intent publication only.
        /// An effect callback is accepted only for a fresh intent, is called at most once, and
        /// must not enter another registry operation. Failure retains the immutable intent for
        /// explicit recovery/resolution; an old intent never authorizes replaying an effect.
        /// A cgroup effect returns its exact registration, which is validated, persisted and read
        /// back before either lock is released. Boundary effects return no cgroup registration.
        pub fn publish_kernel_effect_intent(
            &self,
            intent: &E3KernelEffectIntentV1,
            apply_effect: Option<&mut KernelEffectApplicationV1<'_>>,
            gateway_identity: Option<&crate::InWorldGatewayIdentityV1>,
        ) -> Result<E3KernelEffectIntentRefV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                validate_kernel_effect_intent(intent, &store)?;
                if apply_effect.is_some()
                    && read_retirement(transaction, &intent.series_id)?.is_some()
                {
                    return Err(ConfigProjectionFailureV1::RetiredSeries);
                }
                let root = open_directory_at(transaction.root.as_fd(), "kernel-effects")?;
                let intents = open_directory_at(root.as_fd(), "intents")?;
                let name = format!("{}.json", intent.effect_intent_id);
                if apply_effect.is_some()
                    && read_optional_canonical_at::<E3KernelEffectIntentV1>(intents.as_fd(), &name)?
                        .is_some()
                {
                    // The earlier process may have died immediately before or after mutation.
                    // Recovery must observe/resolve that intent, never retry its kernel action.
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                // The socket-bound gateway identity is a dependency of the boundary intent,
                // not something first made durable after the kernel installation. Intent-only
                // historical publications remain available; an actual boundary callback must
                // supply its exact immutable identity before either intent or effect is exposed.
                match (&intent.effect, gateway_identity) {
                    (
                        E3KernelEffectKindV1::InstallGatewayBoundary {
                            access_boundary_id, ..
                        },
                        Some(gateway),
                    ) => {
                        validate_gateway_identity(gateway, &store)?;
                        if gateway.access_boundary_id != *access_boundary_id {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                        let gateways = open_directory_at(transaction.root.as_fd(), "gateways")?;
                        let name = format!("{}.json", gateway.gateway_instance_id);
                        write_immutable(
                            gateways.as_fd(),
                            &name,
                            &ConfigProjectionCodecV1::encode_canonical_json(gateway)?,
                            "gateway",
                            transaction.owner_uid,
                        )?;
                        let readback: crate::InWorldGatewayIdentityV1 =
                            read_canonical_at(gateways.as_fd(), &name)?;
                        if readback != *gateway {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        }
                    }
                    (E3KernelEffectKindV1::InstallGatewayBoundary { .. }, None)
                        if apply_effect.is_some() =>
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    (_, Some(_)) => return Err(ConfigProjectionFailureV1::WrongBinding),
                    (_, None) => {}
                }
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(intent)?;
                write_immutable(
                    intents.as_fd(),
                    &format!("{}.json", intent.effect_intent_id),
                    &bytes,
                    "kernel-effect-intent",
                    transaction.owner_uid,
                )?;
                let readback: E3KernelEffectIntentV1 = read_canonical_at(
                    intents.as_fd(),
                    &format!("{}.json", intent.effect_intent_id),
                )?;
                if readback != *intent {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                let reference = kernel_effect_intent_ref(intent);
                transaction.verify_scope()?;
                if let Some(apply_effect) = apply_effect {
                    let registration = apply_effect(&reference)?;
                    transaction.verify_scope()?;
                    match (&intent.effect, registration) {
                        (E3KernelEffectKindV1::CreateChildCgroup { .. }, Some(registration)) => {
                            validate_child_cgroup_registration(&registration, &store, intent)?;
                            write_child_cgroup_registration(transaction, &registration)?;
                        }
                        (E3KernelEffectKindV1::InstallGatewayBoundary { .. }, None) => {}
                        _ => return Err(ConfigProjectionFailureV1::WrongBinding),
                    }
                }
                Ok(reference)
            })
        }

        pub fn publish_kernel_effect_resolution(
            &self,
            resolution: &E3KernelEffectResolutionV1,
            resolve_effect: Option<
                &mut dyn FnMut() -> Result<
                    Option<GatewayAccessBoundaryV1>,
                    ConfigProjectionFailureV1,
                >,
            >,
            expected_projection_ref: Option<&ConfigProjectionRefV1>,
        ) -> Result<E3KernelEffectResolutionV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                let intent = resolve_kernel_effect_intent(
                    transaction,
                    &store,
                    &resolution.effect_intent_ref,
                )?;
                validate_kernel_effect_resolution(resolution, &store, &intent)?;
                let effects = validate_registry_tree(transaction, true)?;
                let effect = effects
                    .iter()
                    .find(|effect| effect.intent == intent)
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                match (expected_projection_ref, effect.projection.as_ref()) {
                    (Some(expected), Some(record)) if record_ref(record) == *expected => {
                        if read_series_head(transaction, &intent.series_id)?.head_ref != *expected {
                            return Err(ConfigProjectionFailureV1::StaleRevision);
                        }
                    }
                    (None, None) => {}
                    _ => return Err(ConfigProjectionFailureV1::StaleRevision),
                }

                ensure_intent_has_no_other_resolution(transaction, resolution)?;
                let root = open_directory_at(transaction.root.as_fd(), "kernel-effects")?;
                let resolutions = open_directory_at(root.as_fd(), "resolutions")?;
                if let Some(existing) = read_optional_canonical_at::<E3KernelEffectResolutionV1>(
                    resolutions.as_fd(),
                    &format!("{}.json", resolution.resolution_id),
                )? {
                    if effect
                        .boundary
                        .as_ref()
                        .is_some_and(|boundary| boundary.posture != GatewayAccessPostureV1::Revoked)
                    {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                    return if existing == *resolution {
                        Ok(existing)
                    } else {
                        Err(ConfigProjectionFailureV1::Conflict)
                    };
                }
                if resolve_effect.is_none()
                    && effect
                        .boundary
                        .as_ref()
                        .is_some_and(|boundary| boundary.posture != GatewayAccessPostureV1::Revoked)
                {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                if let Some(resolve_effect) = resolve_effect {
                    transaction.verify_scope()?;
                    let revoked = resolve_effect()?;
                    transaction.verify_scope()?;
                    if let Some(revoked) = revoked {
                        let prior = effect
                            .boundary
                            .as_ref()
                            .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
                        validate_boundary_object(&revoked, &store)?;
                        if revoked.posture != GatewayAccessPostureV1::Revoked
                            || revoked.kernel_effect_intent_ref != resolution.effect_intent_ref
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                        if &revoked != prior {
                            validate_boundary_transition(prior, &revoked)?;
                        }
                        let boundaries =
                            open_directory_at(transaction.root.as_fd(), "gateway-boundaries")?;
                        let root =
                            open_directory_at(boundaries.as_fd(), &revoked.access_boundary_id)?;
                        write_immutable(
                            root.as_fd(),
                            &format!("{:020}.json", revoked.revision),
                            &ConfigProjectionCodecV1::encode_canonical_json(&revoked)?,
                            "boundary",
                            transaction.owner_uid,
                        )?;
                        if read_boundary(
                            transaction,
                            &GatewayAccessBoundaryRefV1 {
                                authority_store_id: revoked.authority_store_id.clone(),
                                access_boundary_id: revoked.access_boundary_id.clone(),
                                revision: revoked.revision,
                                boundary_hash: revoked.boundary_hash.clone(),
                            },
                        )? != revoked
                        {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        }
                    } else if effect
                        .boundary
                        .as_ref()
                        .is_some_and(|boundary| boundary.posture != GatewayAccessPostureV1::Revoked)
                    {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                }
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(resolution)?;
                write_immutable(
                    resolutions.as_fd(),
                    &format!("{}.json", resolution.resolution_id),
                    &bytes,
                    "kernel-effect-resolution",
                    transaction.owner_uid,
                )?;
                let readback: E3KernelEffectResolutionV1 = read_canonical_at(
                    resolutions.as_fd(),
                    &format!("{}.json", resolution.resolution_id),
                )?;
                if readback != *resolution {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                Ok(readback)
            })
        }

        pub fn publish_child_cgroup_registration(
            &self,
            registration: &E3ChildCgroupRegistrationV1,
        ) -> Result<E3ChildCgroupRegistrationV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                let intent = resolve_kernel_effect_intent(
                    transaction,
                    &store,
                    &registration.kernel_effect_intent_ref,
                )?;
                validate_child_cgroup_registration(registration, &store, &intent)?;
                write_child_cgroup_registration(transaction, registration)
            })
        }

        pub fn publish_child_process_registration(
            &self,
            registration: &E3ChildProcessRegistrationV1,
        ) -> Result<E3ChildProcessRegistrationV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                let cgroup = resolve_child_cgroup_registration(
                    transaction,
                    &store,
                    &registration.series_id,
                    &registration.cgroup_registration_id,
                )?;
                validate_child_process_registration(registration, &store, &cgroup)?;
                let root = open_directory_at(transaction.root.as_fd(), "child-processes")?;
                let series = open_or_create_directory(
                    root.as_fd(),
                    &registration.series_id,
                    transaction.owner_uid,
                )?;
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(registration)?;
                write_immutable(
                    series.as_fd(),
                    &format!("{}.json", registration.registration_id),
                    &bytes,
                    "child-process",
                    transaction.owner_uid,
                )?;
                let readback: E3ChildProcessRegistrationV1 = read_canonical_at(
                    series.as_fd(),
                    &format!("{}.json", registration.registration_id),
                )?;
                if readback != *registration {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                Ok(readback)
            })
        }

        pub fn publish_terminal_child_evidence(
            &self,
            evidence: &E3TerminalChildQuiescenceEvidenceV1,
        ) -> Result<crate::E3TerminalChildQuiescenceEvidenceRefV1, ConfigProjectionFailureV1>
        {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                validate_terminal_evidence_object(transaction, &store, evidence)?;
                let root = open_directory_at(transaction.root.as_fd(), "terminal-child-evidence")?;
                let series = open_or_create_directory(
                    root.as_fd(),
                    &evidence.series_id,
                    transaction.owner_uid,
                )?;
                if let Some(existing) = read_optional_canonical_at::<
                    E3TerminalChildQuiescenceEvidenceV1,
                >(
                    series.as_fd(), &format!("{}.json", evidence.evidence_id)
                )? {
                    if existing != *evidence {
                        return Err(ConfigProjectionFailureV1::Conflict);
                    }
                    let record = read_record(transaction, &evidence.final_projection_ref)?;
                    if record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Active {
                        validate_preparation_cleanup_in_transaction_v1(transaction, &record, None)?;
                    }
                }
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(evidence)?;
                write_immutable(
                    series.as_fd(),
                    &format!("{}.json", evidence.evidence_id),
                    &bytes,
                    "terminal-child",
                    transaction.owner_uid,
                )?;
                let readback: E3TerminalChildQuiescenceEvidenceV1 =
                    read_canonical_at(series.as_fd(), &format!("{}.json", evidence.evidence_id))?;
                if readback != *evidence {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                Ok(crate::E3TerminalChildQuiescenceEvidenceRefV1 {
                    authority_store_id: evidence.authority_store_id.clone(),
                    series_id: evidence.series_id.clone(),
                    evidence_id: evidence.evidence_id.clone(),
                    evidence_hash: evidence.evidence_hash.clone(),
                })
            })
        }

        pub fn retire(
            &self,
            retirement: &ConfigProjectionRetirementV1,
        ) -> Result<ConfigProjectionRetirementV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                let store = require_store(transaction)?;
                let head = read_series_head(transaction, &retirement.series_id)?;
                validate_retirement(retirement, &store, &head.head_ref)?;
                if live_capabilities()
                    .lock()
                    .map_err(|_| ConfigProjectionFailureV1::Conflict)?
                    .get(&(
                        retirement.authority_store_id.clone(),
                        retirement.series_id.clone(),
                    ))
                    .copied()
                    .unwrap_or(0)
                    != 0
                {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                ensure_all_leases_released(transaction, &retirement.series_id, None)?;
                validate_terminal_evidence(transaction, retirement)?;
                validate_revoked_boundary(transaction, retirement)?;
                let retirement_dir = open_directory_at(transaction.root.as_fd(), "retirement")?;
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(retirement)?;
                let name = format!("{}.json", retirement.series_id);
                write_immutable(
                    retirement_dir.as_fd(),
                    &name,
                    &bytes,
                    "retirement",
                    transaction.owner_uid,
                )?;
                Ok(retirement.clone())
            })
        }

        fn publish(
            &self,
            record: &AgentConfigProjectionRecordV1,
            expected_head: Option<&ConfigProjectionRefV1>,
            expected_posture: ManagedGatewayProjectionPostureV1,
        ) -> Result<ConfigProjectionRefV1, ConfigProjectionFailureV1> {
            self.with_transaction(false, |transaction| {
                publish_record_in_transaction(transaction, record, expected_head, expected_posture)
            })
        }

        fn with_transaction<T>(
            &self,
            require_existing: bool,
            operation: impl FnOnce(
                &mut ConfigProjectionChildTransactionV1,
            ) -> Result<T, ConfigProjectionFailureV1>,
        ) -> Result<T, ConfigProjectionFailureV1> {
            let mut operation = Some(operation);
            let mut captured = None;
            let mut callback = |authority_fd: BorrowedFd<'_>| {
                let result =
                    ConfigProjectionChildTransactionV1::begin(authority_fd, require_existing)
                        .and_then(|mut transaction| {
                            if require_existing {
                                require_store(&transaction)?;
                            }
                            recover_owned_temps(&transaction)?;
                            recover_native_source_temps(&transaction)?;
                            let operation_result = operation
                                .take()
                                .ok_or(ConfigProjectionFailureV1::Conflict)?(
                                &mut transaction
                            );
                            let finish_result = transaction.finish();
                            match finish_result {
                                Ok(()) => operation_result,
                                Err(error) => Err(error),
                            }
                        });
                captured = Some(result);
                Ok(())
            };
            self.parent.with_locked_parent(&mut callback)?;
            captured.ok_or(ConfigProjectionFailureV1::Conflict)?
        }
    }

    // Shared by the admitted publication operations; the caller owns both authority locks.
    type AuthoringReadbackV1 = (
        EffectiveSubstrateConfigSourceV1,
        AgentInventorySourceMaterialV1,
        TrustedRuntimeArtifactManifestV1,
    );
    type ProjectionAndAuthoringReadbackV1 = (
        Option<ConfigProjectionResolutionV1>,
        Option<AuthoringReadbackV1>,
    );

    fn resolve_authoring_input_in_transaction(
        transaction: &ConfigProjectionChildTransactionV1,
        reference: &ConfigProjectionAuthoringInputRefV1,
    ) -> Result<AuthoringReadbackV1, ConfigProjectionFailureV1> {
        reference
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let store = require_store(transaction)?;
        if reference.authority_store_id != store.authority_store_id {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let inputs = open_directory_at(transaction.root.as_fd(), "inputs")?;
        let effective_root = open_directory_at(inputs.as_fd(), "effective-config")?;
        let inventory_root = open_directory_at(inputs.as_fd(), "agent-inventory")?;
        let artifact_root = open_directory_at(transaction.root.as_fd(), "runtime-artifacts")?;
        let manifest_root = open_directory_at(
            artifact_root.as_fd(),
            &reference.runtime_artifact_manifest_id,
        )?;
        let effective: EffectiveSubstrateConfigSourceV1 = read_canonical_at(
            effective_root.as_fd(),
            &format!("{}.json", reference.effective_config_source_hash),
        )?;
        let inventory: AgentInventorySourceMaterialV1 = read_canonical_at(
            inventory_root.as_fd(),
            &format!("{}.json", reference.agent_inventory_source_hash),
        )?;
        let manifest: TrustedRuntimeArtifactManifestV1 = read_canonical_at(
            manifest_root.as_fd(),
            &format!("{:020}.json", reference.runtime_artifact_manifest_revision),
        )?;
        validate_effective_config_source(&effective, &store)?;
        validate_agent_inventory_source(&inventory, &effective, &store)?;
        validate_runtime_artifact_manifest(&manifest, &store)?;
        if effective.source_hash != reference.effective_config_source_hash
            || inventory.source_hash != reference.agent_inventory_source_hash
            || manifest.manifest_id != reference.runtime_artifact_manifest_id
            || manifest.revision != reference.runtime_artifact_manifest_revision
            || manifest.manifest_hash != reference.runtime_artifact_manifest_hash
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok((effective, inventory, manifest))
    }

    fn resolve_projection_in_transaction(
        transaction: &mut ConfigProjectionChildTransactionV1,
        parent: Arc<dyn ConfigProjectionHsaAuthorityV1>,
        identity: &ConfigProjectionIdentityV1,
        held_lease: &ConfigProjectionConsumerLeaseV1,
    ) -> Result<ConfigProjectionResolutionV1, ConfigProjectionFailureV1> {
        let Some(store) = transaction
            .read_resolution_object::<ConfigProjectionStoreV1>(&["store.json".to_string()])?
        else {
            return Ok(ConfigProjectionResolutionV1::Missing);
        };
        let store = match store {
            ResolutionObjectV1::V1(store) => store,
            ResolutionObjectV1::Newer(observed_schema_version) => {
                return Ok(ConfigProjectionResolutionV1::UnsupportedNewerSchema {
                    observed_schema_version,
                });
            }
        };
        validate_store(&store, transaction)?;
        if identity.authority_store_id != store.authority_store_id {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let subject_hash = subject_hash(identity)?;
        let binding_name = format!("{subject_hash}.json");
        let Some(binding) = transaction
            .read_resolution_object::<ConfigProjectionSubjectBindingV1>(&[
                "subjects".to_string(),
                binding_name,
            ])?
        else {
            return Ok(ConfigProjectionResolutionV1::Missing);
        };
        let binding = match binding {
            ResolutionObjectV1::V1(binding) => binding,
            ResolutionObjectV1::Newer(observed_schema_version) => {
                return Ok(ConfigProjectionResolutionV1::UnsupportedNewerSchema {
                    observed_schema_version,
                });
            }
        };
        validate_binding(&binding, &store)?;
        if binding.subject_hash != subject_hash
            || identity_without_series_hash(&binding.identity)?
                != identity_without_series_hash(identity)?
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let head = transaction
            .read_resolution_object::<ConfigProjectionHeadV1>(&[
                "series".to_string(),
                binding.series_id.clone(),
                "head.json".to_string(),
            ])?
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let head = match head {
            ResolutionObjectV1::V1(head) => head,
            ResolutionObjectV1::Newer(observed_schema_version) => {
                return Ok(ConfigProjectionResolutionV1::UnsupportedNewerSchema {
                    observed_schema_version,
                });
            }
        };
        validate_head(&head, &store, &binding.series_id)?;
        if let Some(retirement) = transaction
            .read_resolution_object::<ConfigProjectionRetirementV1>(&[
                "retirement".to_string(),
                format!("{}.json", binding.series_id),
            ])?
        {
            let retirement = match retirement {
                ResolutionObjectV1::V1(retirement) => retirement,
                ResolutionObjectV1::Newer(observed_schema_version) => {
                    return Ok(ConfigProjectionResolutionV1::UnsupportedNewerSchema {
                        observed_schema_version,
                    });
                }
            };
            validate_retirement(&retirement, &store, &head.head_ref)?;
            return Ok(ConfigProjectionResolutionV1::Retired {
                final_projection_ref: retirement.final_head_ref.clone(),
                retirement,
            });
        }
        let record = transaction
            .read_resolution_object::<AgentConfigProjectionRecordV1>(&[
                "series".to_string(),
                head.head_ref.series_id.clone(),
                "records".to_string(),
                format!(
                    "{:020}-{}.json",
                    head.head_ref.revision, head.head_ref.record_id
                ),
            ])?
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let record = match record {
            ResolutionObjectV1::V1(record) => record,
            ResolutionObjectV1::Newer(observed_schema_version) => {
                return Ok(ConfigProjectionResolutionV1::UnsupportedNewerSchema {
                    observed_schema_version,
                });
            }
        };
        validate_record(&record, &store)?;
        validate_gateway_ack_chain_v1(transaction, Some(&record), None)?;
        if record_ref(&record) != head.head_ref
            || record.identity != binding.identity
            || record.identity != *identity
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let held_lease = resolve_held_lease(transaction, held_lease, &head.head_ref)?;
        let capability = PublishedConfigProjectionCapabilityV1::new(
            store.authority_store_id,
            record.identity.clone(),
            head.head_ref.clone(),
            held_lease,
            parent,
        )?;
        Ok(ConfigProjectionResolutionV1::Current {
            identity: record.identity.clone(),
            projection_ref: head.head_ref,
            record,
            capability,
        })
    }

    fn read_preparation_subject_in_transaction_v1(
        transaction: &ConfigProjectionChildTransactionV1,
        candidate: &ConfigProjectionIdentityV1,
        effects: &[E3KernelEffectRecoveryV1],
    ) -> Result<ConfigProjectionSubjectReadbackV1, ConfigProjectionFailureV1> {
        let store = require_store(transaction)?;
        if candidate.authority_store_id != store.authority_store_id
            || candidate.accepted_home != store.accepted_home
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_hash_field(
            &candidate.identity_hash,
            hash_omitting(
                "substrate.e3.config-projection-identity.v1",
                "identity",
                candidate,
                "identity_hash",
            )?,
        )?;
        let subject = subject_hash(candidate)?;
        let subjects = open_directory_at(transaction.root.as_fd(), "subjects")?;
        let Some(binding) = read_optional_canonical_at::<ConfigProjectionSubjectBindingV1>(
            subjects.as_fd(),
            &format!("{subject}.json"),
        )?
        else {
            if effects.iter().any(|effect| {
                effect.projection.as_ref().is_none_or(|record| {
                    identity_without_series_hash(&record.identity).ok()
                        == identity_without_series_hash(candidate).ok()
                })
            }) {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            return Ok(ConfigProjectionSubjectReadbackV1::Unbound);
        };
        validate_binding(&binding, &store)?;
        if binding.subject_hash != subject
            || identity_without_series_hash(&binding.identity)?
                != identity_without_series_hash(candidate)?
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let head = read_series_head(transaction, &binding.series_id)?;
        if let Some(retirement) = read_retirement(transaction, &binding.series_id)? {
            validate_retirement(&retirement, &store, &head.head_ref)?;
            validate_revoked_boundary(transaction, &retirement)?;
            return Err(ConfigProjectionFailureV1::RetiredSeries);
        }
        let record = read_record(transaction, &head.head_ref)?;
        validate_record(&record, &store)?;
        validate_gateway_ack_chain_v1(transaction, Some(&record), None)?;
        if record.identity != binding.identity || record_ref(&record) != head.head_ref {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let fence = match &record.activation.publication_fence {
            crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id }
            | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => fence_id,
        };
        // The subject filter cannot erase a pending historical or headless obligation.
        // A validated resolved headless effect intentionally has no projection; retain it
        // in the readback for the cleanup consumer's required current kernel observations.
        if effects.iter().any(|effect| {
            effect.intent.series_id == binding.series_id
                && effect.intent.authority_store_id == store.authority_store_id
                && effect.intent.fence_id != *fence
                && effect.resolution.is_none()
        }) {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let handoffs = open_directory_at(transaction.root.as_fd(), "handoffs")?;
        let handoff = open_directory_at(
            handoffs.as_fd(),
            &record.nonsecret_handoff.secret_handoff_ref.handoff_id,
        )?;
        let current_ref: crate::SecretHandoffRefV1 =
            read_canonical_at(handoff.as_fd(), "head.json")?;
        let current_handoff = read_handoff_revision(transaction, &current_ref)?;
        let mut prepared_handoff = current_handoff.clone();
        while let Some(predecessor_ref) = &prepared_handoff.predecessor_ref {
            let predecessor = read_handoff_revision(transaction, predecessor_ref)?;
            validate_handoff_transition(&predecessor, &prepared_handoff)?;
            prepared_handoff = predecessor;
        }
        if prepared_handoff.handoff.state != SecretHandoffStateV1::Prepared
            || prepared_handoff.handoff.state_revision != 1
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_handoff_transition_attempt(
            transaction,
            &store,
            &binding.identity,
            fence,
            &prepared_handoff,
            None,
        )?;
        let consumer_id = prepared_member_dispatch_consumer_id_v1(
            &prepared_handoff
                .handoff
                .credential_source_ref
                .preparation_id,
        )?;
        let leases = open_directory_at(transaction.root.as_fd(), "leases")?;
        let consumer = open_optional_directory_at(leases.as_fd(), &binding.series_id)?
            .map(|series| open_optional_directory_at(series.as_fd(), &consumer_id))
            .transpose()?
            .flatten();
        let consumer_lease = consumer
            .map(|consumer| {
                let lease: ConfigProjectionConsumerLeaseV1 =
                    read_canonical_at(consumer.as_fd(), "head.json")?;
                validate_lease(&lease, &lease.acquired_projection_ref)?;
                if lease.consumer_id != consumer_id
                    || lease.series_id != binding.series_id
                    || lease.acquired_projection_ref.authority_store_id != store.authority_store_id
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                let acquired = read_record(transaction, &lease.acquired_projection_ref)?;
                validate_record(&acquired, &store)?;
                validate_gateway_ack_chain_v1(transaction, Some(&acquired), None)?;
                if acquired.identity != record.identity
                    || acquired.activation.publication_fence
                        != (crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                            fence_id: fence.clone(),
                        })
                    || acquired.managed_gateway.posture
                        != ManagedGatewayProjectionPostureV1::Dormant
                    || acquired.nonsecret_handoff.secret_handoff_ref
                        != handoff_reference(&prepared_handoff)?
                    || lease.acquired_at != acquired.created_at
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                let revisions = open_directory_at(consumer.as_fd(), "revisions")?;
                let exact: ConfigProjectionConsumerLeaseV1 =
                    read_canonical_at(revisions.as_fd(), &format!("{:020}.json", lease.revision))?;
                if exact != lease {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                Ok(lease)
            })
            .transpose()?;
        Ok(ConfigProjectionSubjectReadbackV1::Bound(
            ConfigProjectionPreparationMetadataV1 {
                record,
                projection_ref: head.head_ref,
                prepared_handoff,
                current_handoff,
                consumer_lease,
            },
        ))
    }

    type PreparedGatewayChainV1<'a> = (
        &'a crate::ConfigProjectionSecretHandoffRevisionV1,
        &'a crate::InWorldGatewayIdentityV1,
        &'a GatewayAccessBoundaryV1,
        &'a crate::ManagedGatewayActivationIntentV1,
        &'a crate::ManagedGatewayLaunchInputV1,
        &'a [E3ChildCgroupRegistrationV1; 3],
    );

    fn handoff_reference(
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

    fn validate_handoff_revision(
        revision: &crate::ConfigProjectionSecretHandoffRevisionV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let handoff = &revision.handoff;
        validate_prefixed_uuid(&handoff.handoff_id, "gsh_")?;
        validate_credential_source_ref(&handoff.credential_source_ref)?;
        handoff
            .receiving_gateway_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        validate_timestamp(&handoff.created_at)?;
        validate_timestamp(&handoff.expires_at)?;
        if revision.schema_version != 1
            || handoff.schema_version != 1
            || revision.authority_store_id != store.authority_store_id
            || handoff.receiving_gateway_ref.authority_store_id != store.authority_store_id
            || handoff.orchestration_session_id.is_empty()
            || handoff
                .retained_participant_id
                .as_ref()
                .is_none_or(String::is_empty)
            || handoff.runtime_family != "codex"
            || handoff.world_id.is_empty()
            || handoff.world_generation == 0
            || handoff.created_at != handoff.credential_source_ref.issued_at
            || handoff.expires_at != handoff.credential_source_ref.expires_at
            || !matches!(&handoff.delivery, SecretDeliveryMechanismV1::SecureFd {
                fd_name, one_time: true, gateway_receiver_only: true,
                deny_child_inheritance: true, close_after_consume: true,
            } if fd_name == "SUBSTRATE_LLM_AUTH_BUNDLE_FD")
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        match (
            handoff.state,
            handoff.state_revision,
            &revision.predecessor_ref,
        ) {
            (SecretHandoffStateV1::Prepared, 1, None)
                if handoff.delivered_at.is_none()
                    && handoff.consumed_at.is_none()
                    && handoff.failure_diagnostic_ref.is_none() => {}
            (SecretHandoffStateV1::Delivered, 2, Some(previous))
                if previous.handoff_state_revision == 1
                    && handoff.delivered_at.is_some()
                    && handoff.consumed_at.is_none()
                    && handoff.failure_diagnostic_ref.is_none() => {}
            (SecretHandoffStateV1::Consumed, 3, Some(previous))
                if previous.handoff_state_revision == 2
                    && handoff.delivered_at.is_some()
                    && handoff.consumed_at.is_some()
                    && handoff.failure_diagnostic_ref.is_none() => {}
            (
                SecretHandoffStateV1::Failed | SecretHandoffStateV1::Expired,
                2 | 3,
                Some(previous),
            ) if previous.handoff_state_revision + 1 == handoff.state_revision
                && handoff.consumed_at.is_none() => {}
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        }
        for time in [handoff.delivered_at.as_ref(), handoff.consumed_at.as_ref()]
            .into_iter()
            .flatten()
        {
            validate_timestamp(time)?;
        }
        if let Some(previous) = &revision.predecessor_ref {
            let mut expected = handoff_reference(revision)?;
            expected.handoff_state_revision = previous.handoff_state_revision;
            expected.handoff_hash = previous.handoff_hash.clone();
            validate_sha256(&previous.handoff_hash)?;
            if previous != &expected {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        validate_hash_field(
            &revision.revision_hash,
            hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                revision,
                "revision_hash",
            )?,
        )
    }

    fn validate_gateway_identity(
        gateway: &crate::InWorldGatewayIdentityV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_prefixed_uuid(&gateway.gateway_instance_id, "cgi_")?;
        validate_prefixed_uuid(&gateway.access_boundary_id, "gab_")?;
        validate_sha256(&gateway.config_projection_identity_hash)?;
        validate_sha256(&gateway.gateway_artifact_sha256)?;
        if gateway.schema_version != 1
            || gateway.authority_store_id != store.authority_store_id
            || gateway.backend_id != "cli:codex-world"
            || gateway.world_generation == 0
            || gateway.world_id.is_empty()
            || gateway.orchestration_session_id.is_empty()
            || gateway.retained_participant_id.is_empty()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_hash_field(
            &gateway.gateway_identity_hash,
            hash_omitting(
                "substrate.e3.in-world-gateway-identity.v1",
                "gateway",
                gateway,
                "gateway_identity_hash",
            )?,
        )
    }

    fn validate_gateway_intent(
        intent: &crate::ManagedGatewayActivationIntentV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_prefixed_uuid(&intent.preparation_id, "e3p_")?;
        validate_prefixed_uuid(&intent.activation_intent_id, "gai_")?;
        validate_prefixed_uuid(&intent.dormant_record_id, "cpr_")?;
        validate_prefixed_uuid(&intent.fence_id, "cpf_")?;
        validate_sha256(&intent.config_projection_identity_hash)?;
        validate_prefixed_uuid(&intent.readiness_nonce, "")?;
        validate_timestamp(&intent.created_at)?;
        intent
            .expected_gateway_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        validate_boundary_ref(&intent.expected_access_boundary_ref)?;
        if intent.schema_version != 1
            || intent.authority_store_id != store.authority_store_id
            || intent.dormant_revision == 0
            || intent.expected_gateway_ref.authority_store_id != store.authority_store_id
            || intent.expected_access_boundary_ref.authority_store_id != store.authority_store_id
            || intent.secret_handoff_ref.authority_store_id != store.authority_store_id
            || intent.secret_handoff_ref.handoff_state_revision != 1
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_hash_field(
            &intent.intent_hash,
            hash_omitting(
                "substrate.e3.managed-gateway-activation-intent.v1",
                "intent",
                intent,
                "intent_hash",
            )?,
        )
    }

    fn validate_gateway_launch_input(
        input: &crate::ManagedGatewayLaunchInputV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_prefixed_uuid(&input.launch_input_id, "gal_")?;
        validate_sha256(&input.config_projection_identity_hash)?;
        validate_prefixed_uuid(&input.readiness_nonce, "")?;
        input
            .activation_intent_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        input
            .dormant_projection_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        input
            .gateway_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        validate_boundary_ref(&input.access_boundary_ref)?;
        let surface = &input.http_surface;
        let listener = &input.listener_identity;
        if input.schema_version != 1
            || input.authority_store_id != store.authority_store_id
            || input.activation_intent_ref.authority_store_id != store.authority_store_id
            || input.dormant_projection_ref.authority_store_id != store.authority_store_id
            || input.gateway_ref.authority_store_id != store.authority_store_id
            || input.access_boundary_ref.authority_store_id != store.authority_store_id
            || input.secret_handoff_prepared_ref.authority_store_id != store.authority_store_id
            || input.secret_handoff_prepared_ref.handoff_state_revision != 1
            || input.backend_id != "cli:codex-world"
            || input.world_generation == 0
            || input.world_id.is_empty()
            || input.orchestration_session_id.is_empty()
            || input.retained_participant_id.is_empty()
            || !surface.inherited_listener_only
            || surface.auxiliary_listener_count != 0
            || surface.readiness_method != "GET"
            || surface.readiness_path != "/health"
            || surface.member_method != "POST"
            || surface.member_path != "/v1/responses"
            || listener.transport != "tcp"
            || listener.address != "127.0.0.1"
            || listener.port == 0
            || listener.network_namespace_inode == 0
            || listener.socket_inode == 0
            || listener.listen_backlog != 16
            || !listener.deny_boundary_effective_before_listen
            || listener.responses_base_path != "/v1"
            || input.gateway_config.relative_path != "config.toml"
            || input.gateway_config.mode != 0o600
            || input.gateway_config.byte_length == 0
            || input.gateway_config.byte_length > 65536
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_sha256(&input.gateway_config.sha256)?;
        validate_hash_field(
            &input.launch_input_hash,
            hash_omitting(
                "substrate.e3.managed-gateway-launch-input.v1",
                "launch_input",
                input,
                "launch_input_hash",
            )?,
        )
    }

    fn validate_prepared_gateway_chain(
        transaction: &ConfigProjectionChildTransactionV1,
        record: &AgentConfigProjectionRecordV1,
        (handoff, gateway, boundary, intent, input, registrations): PreparedGatewayChainV1<'_>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let store = require_store(transaction)?;
        validate_handoff_revision(handoff, &store)?;
        validate_gateway_identity(gateway, &store)?;
        validate_boundary_object(boundary, &store)?;
        validate_gateway_intent(intent, &store)?;
        validate_gateway_launch_input(input, &store)?;
        let identity = &record.identity;
        let gateway_ref = crate::InWorldGatewayRefV1 {
            authority_store_id: gateway.authority_store_id.clone(),
            gateway_instance_id: gateway.gateway_instance_id.clone(),
            gateway_identity_hash: gateway.gateway_identity_hash.clone(),
        };
        let boundary_ref = GatewayAccessBoundaryRefV1 {
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
        let crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } =
            &record.activation.publication_fence
        else {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        };
        validate_native_projection_source_input(&record.native, &identity.series_id, fence_id)?;
        if record.managed_gateway.posture != ManagedGatewayProjectionPostureV1::Dormant
            || handoff.handoff.state != SecretHandoffStateV1::Prepared
            || handoff_reference(handoff)? != record.nonsecret_handoff.secret_handoff_ref
            || handoff.handoff.credential_source_ref
                != record.nonsecret_handoff.credential_source_ref
            || handoff.handoff.delivery != record.nonsecret_handoff.delivery
            || handoff.handoff.receiving_gateway_ref != gateway_ref
            || handoff.handoff.orchestration_session_id != identity.orchestration_session_id
            || handoff.handoff.retained_participant_id.as_ref()
                != Some(&identity.retained_participant_id)
            || handoff.handoff.world_id != identity.world_id
            || handoff.handoff.world_generation != identity.world_generation
            || handoff.handoff.runtime_family != identity.runtime_family
            || record.managed_gateway.expected_gateway_ref != gateway_ref
            || record.managed_gateway.activation_intent_ref != intent_ref
            || record.managed_gateway.access_boundary_ref != boundary_ref
            || gateway.config_projection_identity_hash != identity.identity_hash
            || gateway.orchestration_session_id != identity.orchestration_session_id
            || gateway.retained_participant_id != identity.retained_participant_id
            || gateway.world_id != identity.world_id
            || gateway.world_generation != identity.world_generation
            || gateway.backend_id != identity.backend_id
            || gateway.access_boundary_id != boundary.access_boundary_id
            || gateway.gateway_artifact_sha256 != identity.runtime_artifacts.managed_gateway.sha256
            || boundary.gateway_instance_id != gateway.gateway_instance_id
            || boundary.config_projection_identity_hash != identity.identity_hash
            || boundary.orchestration_session_id != identity.orchestration_session_id
            || boundary.retained_participant_id != identity.retained_participant_id
            || boundary.backend_id != identity.backend_id
            || boundary.world_id != identity.world_id
            || boundary.world_generation != identity.world_generation
            || boundary.posture != GatewayAccessPostureV1::DenyAllDormant
            || intent.preparation_id != handoff.handoff.credential_source_ref.preparation_id
            || intent.config_projection_identity_hash != identity.identity_hash
            || intent.dormant_record_id != record.record_id
            || intent.dormant_revision != record.revision
            || intent.expected_gateway_artifact != identity.runtime_artifacts.managed_gateway
            || intent.expected_gateway_ref != gateway_ref
            || intent.expected_access_boundary_ref != boundary_ref
            || intent.secret_handoff_ref != handoff_reference(handoff)?
            || &intent.fence_id != fence_id
            || input.activation_intent_ref != intent_ref
            || input.dormant_projection_ref != record_ref(record)
            || input.gateway_ref != gateway_ref
            || input.config_projection_identity_hash != identity.identity_hash
            || input.orchestration_session_id != identity.orchestration_session_id
            || input.retained_participant_id != identity.retained_participant_id
            || input.backend_id != identity.backend_id
            || input.world_id != identity.world_id
            || input.world_generation != identity.world_generation
            || input.listener_identity != boundary.gateway_listener
            || input.access_boundary_ref != boundary_ref
            || input.secret_handoff_prepared_ref != handoff_reference(handoff)?
            || input.readiness_nonce != intent.readiness_nonce
            || input.gateway_config.root.physical_path
                != format!(
                    "/run/substrate/e3-gateway/{}/{fence_id}",
                    identity.series_id
                )
            || record.managed_gateway.codex_base_url
                != format!("http://127.0.0.1:{}/v1", boundary.gateway_listener.port)
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let gateways = open_directory_at(transaction.root.as_fd(), "gateways")?;
        let durable_gateway: crate::InWorldGatewayIdentityV1 = read_canonical_at(
            gateways.as_fd(),
            &format!("{}.json", gateway.gateway_instance_id),
        )?;
        if durable_gateway != *gateway {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let roles = [
            crate::E3TerminalProcessRoleV1::ManagedGateway,
            crate::E3TerminalProcessRoleV1::ReadinessProbe,
            crate::E3TerminalProcessRoleV1::Codex,
        ];
        for (registration, role) in registrations.iter().zip(roles) {
            let durable = resolve_child_cgroup_registration(
                transaction,
                &store,
                &identity.series_id,
                &registration.cgroup_registration_id,
            )?;
            let effect = resolve_kernel_effect_intent(
                transaction,
                &store,
                &durable.kernel_effect_intent_ref,
            )?;
            if durable != *registration
                || registration.role != role
                || registration.turn_id.is_some()
                || registration.fence_id != *fence_id
                || effect.preparation_id != intent.preparation_id
                || !matches!(&effect.effect, E3KernelEffectKindV1::CreateChildCgroup { parent_cgroup, .. } if parent_cgroup.cgroup_relative_path == format!("substrate/{}", identity.world_id))
                || registration.kernel_boot_id != registrations[0].kernel_boot_id
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        if registrations[1].cgroup != boundary.readiness_probe_cgroup
            || registrations[2].cgroup != boundary.allowed_member_cgroup
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let effect =
            resolve_kernel_effect_intent(transaction, &store, &boundary.kernel_effect_intent_ref)?;
        if effect.series_id != identity.series_id
            || effect.preparation_id != intent.preparation_id
            || effect.fence_id != *fence_id
            || !matches!(effect.effect, E3KernelEffectKindV1::InstallGatewayBoundary {
                access_boundary_id, network_namespace_inode, table_name, chain_name,
            } if access_boundary_id == boundary.access_boundary_id
                && network_namespace_inode == boundary.gateway_listener.network_namespace_inode
                && table_name == boundary.nftables_chain.table && chain_name == boundary.nftables_chain.chain)
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(())
    }

    fn publish_prepared_gateway_chain(
        transaction: &ConfigProjectionChildTransactionV1,
        (handoff, gateway, boundary, intent, _input, _registrations): PreparedGatewayChainV1<'_>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let boundaries = open_directory_at(transaction.root.as_fd(), "gateway-boundaries")?;
        let boundary_root = open_or_create_directory(
            boundaries.as_fd(),
            &boundary.access_boundary_id,
            transaction.owner_uid,
        )?;
        write_immutable(
            boundary_root.as_fd(),
            "00000000000000000001.json",
            &ConfigProjectionCodecV1::encode_canonical_json(boundary)?,
            "boundary",
            transaction.owner_uid,
        )?;
        let handoffs = open_directory_at(transaction.root.as_fd(), "handoffs")?;
        let root = open_or_create_directory(
            handoffs.as_fd(),
            &handoff.handoff.handoff_id,
            transaction.owner_uid,
        )?;
        let revisions = open_or_create_directory(root.as_fd(), "revisions", transaction.owner_uid)?;
        write_immutable(
            revisions.as_fd(),
            "00000000000000000001.json",
            &ConfigProjectionCodecV1::encode_canonical_json(handoff)?,
            "handoff-revision",
            transaction.owner_uid,
        )?;
        write_immutable(
            root.as_fd(),
            "head.json",
            &ConfigProjectionCodecV1::encode_canonical_json(&handoff_reference(handoff)?)?,
            "handoff-head",
            transaction.owner_uid,
        )?;
        for (directory, name, kind, bytes) in [
            (
                "gateways",
                format!("{}.json", gateway.gateway_instance_id),
                "gateway",
                ConfigProjectionCodecV1::encode_canonical_json(gateway)?,
            ),
            (
                "gateway-intents",
                format!("{}.json", intent.activation_intent_id),
                "gateway-intent",
                ConfigProjectionCodecV1::encode_canonical_json(intent)?,
            ),
        ] {
            let root = open_directory_at(transaction.root.as_fd(), directory)?;
            write_immutable(root.as_fd(), &name, &bytes, kind, transaction.owner_uid)?;
        }
        Ok(())
    }

    fn validate_gateway_recovery_candidate(
        transaction: &ConfigProjectionChildTransactionV1,
        path: &[String],
        kind: &str,
        bytes: &[u8],
    ) -> Result<(String, bool), ConfigProjectionFailureV1> {
        let store = require_store(transaction)?;
        let name = match kind {
            "gateway-ack" if path == ["gateway-acks"] => {
                let ack: crate::ManagedGatewayActivationAckV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_gateway_ack_chain_v1(transaction, None, Some(&ack))?;
                format!("{}.json", ack.activation_ack_id)
            }
            "gateway" if path == ["gateways"] => {
                let value: crate::InWorldGatewayIdentityV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_gateway_identity(&value, &store)?;
                format!("{}.json", value.gateway_instance_id)
            }
            "gateway-intent" if path == ["gateway-intents"] => {
                let value: crate::ManagedGatewayActivationIntentV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_gateway_intent(&value, &store)?;
                format!("{}.json", value.activation_intent_id)
            }
            "gateway-launch-input" if path == ["gateway-launch-inputs"] => {
                let value: crate::ManagedGatewayLaunchInputV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_gateway_launch_input(&value, &store)?;
                format!("{}.json", value.launch_input_id)
            }
            "handoff-revision"
                if path.len() == 3 && path[0] == "handoffs" && path[2] == "revisions" =>
            {
                let value: crate::ConfigProjectionSecretHandoffRevisionV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_handoff_revision(&value, &store)?;
                if value.handoff.handoff_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                format!("{:020}.json", value.handoff.state_revision)
            }
            "handoff-head" if path.len() == 2 && path[0] == "handoffs" => {
                let reference: crate::SecretHandoffRefV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                if reference.handoff_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                read_handoff_revision(transaction, &reference)?;
                // Prepared heads are immutable first-writer publications. Later transitions
                // use expected-head CAS and cannot be recovered as an initial publication.
                if reference.handoff_state_revision != 1 {
                    return Err(ConfigProjectionFailureV1::StaleRevision);
                }
                "head.json".to_owned()
            }
            _ => return Err(ConfigProjectionFailureV1::PartialPublication),
        };
        Ok((name, false))
    }

    fn read_handoff_revision(
        transaction: &ConfigProjectionChildTransactionV1,
        reference: &crate::SecretHandoffRefV1,
    ) -> Result<crate::ConfigProjectionSecretHandoffRevisionV1, ConfigProjectionFailureV1> {
        validate_prefixed_uuid(&reference.handoff_id, "gsh_")?;
        let root = open_directory_at(transaction.root.as_fd(), "handoffs")?;
        let handoff = open_directory_at(root.as_fd(), &reference.handoff_id)?;
        let revisions = open_directory_at(handoff.as_fd(), "revisions")?;
        let value = read_canonical_at(
            revisions.as_fd(),
            &format!("{:020}.json", reference.handoff_state_revision),
        )?;
        validate_handoff_revision(&value, &require_store(transaction)?)?;
        if handoff_reference(&value)? != *reference {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(value)
    }

    fn validate_preparation_cleanup_in_transaction_v1(
        transaction: &ConfigProjectionChildTransactionV1,
        record: &AgentConfigProjectionRecordV1,
        pending_successor: Option<&AgentConfigProjectionRecordV1>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let store = require_store(transaction)?;
        validate_record(record, &store)?;
        // A cleanup receipt is historical authority only for this exact immutable record.
        // In particular, ReadyClosed cannot be treated as a reference-only ACK fixture.
        if read_record(transaction, &record_ref(record))? != *record {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_gateway_ack_chain_v1(transaction, Some(record), None)?;
        if let Some(successor) = pending_successor {
            validate_record(successor, &store)?;
            validate_record_transition(record, successor)?;
            let current = read_series_head(transaction, &record.identity.series_id)?;
            if current.head_ref != record_ref(record) && current.head_ref != record_ref(successor) {
                return Err(ConfigProjectionFailureV1::StaleRevision);
            }
        }
        if pending_successor.is_some()
            || record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Active
        {
            let subjects = open_directory_at(transaction.root.as_fd(), "subjects")?;
            let subject = subject_hash(&record.identity)?;
            let binding: ConfigProjectionSubjectBindingV1 =
                read_canonical_at(subjects.as_fd(), &format!("{subject}.json"))?;
            validate_binding(&binding, &store)?;
            if binding.identity != record.identity || binding.subject_hash != subject {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let handoffs = open_directory_at(transaction.root.as_fd(), "handoffs")?;
            let handoff = open_directory_at(
                handoffs.as_fd(),
                &record.nonsecret_handoff.secret_handoff_ref.handoff_id,
            )?;
            let current_ref: crate::SecretHandoffRefV1 =
                read_canonical_at(handoff.as_fd(), "head.json")?;
            let terminal = read_handoff_revision(transaction, &current_ref)?;
            if matches!(
                record.managed_gateway.posture,
                ManagedGatewayProjectionPostureV1::ReadyClosed
                    | ManagedGatewayProjectionPostureV1::Active
            ) && current_ref != record.nonsecret_handoff.secret_handoff_ref
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let mut prepared = terminal.clone();
            while let Some(reference) = &prepared.predecessor_ref {
                let predecessor = read_handoff_revision(transaction, reference)?;
                validate_handoff_transition(&predecessor, &prepared)?;
                prepared = predecessor;
            }
            if prepared.handoff.state != SecretHandoffStateV1::Prepared
                || (record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Dormant
                    && handoff_reference(&prepared)? != record.nonsecret_handoff.secret_handoff_ref)
                || (terminal.handoff.state == SecretHandoffStateV1::Consumed
                    && (terminal.handoff.state_revision != 3
                        || prepared.handoff.state_revision != 1))
                || prepared.handoff.credential_source_ref
                    != record.nonsecret_handoff.credential_source_ref
                || terminal.handoff.receiving_gateway_ref
                    != record.managed_gateway.expected_gateway_ref
                || !matches!(
                    (record.managed_gateway.posture, terminal.handoff.state),
                    (
                        ManagedGatewayProjectionPostureV1::Dormant
                            | ManagedGatewayProjectionPostureV1::ReadyClosed,
                        SecretHandoffStateV1::Failed
                            | SecretHandoffStateV1::Expired
                            | SecretHandoffStateV1::Consumed
                    ) | (
                        ManagedGatewayProjectionPostureV1::Active,
                        SecretHandoffStateV1::Consumed
                    )
                )
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let fence = match &record.activation.publication_fence {
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id }
                | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => fence_id,
            };
            let consumer_id = prepared_member_dispatch_consumer_id_v1(
                &prepared.handoff.credential_source_ref.preparation_id,
            )?;
            let leases = open_directory_at(transaction.root.as_fd(), "leases")?;
            let series = open_directory_at(leases.as_fd(), &record.identity.series_id)?;
            let consumer = open_directory_at(series.as_fd(), &consumer_id)?;
            let lease: ConfigProjectionConsumerLeaseV1 =
                read_canonical_at(consumer.as_fd(), "head.json")?;
            validate_lease(&lease, &lease.acquired_projection_ref)?;
            let acquired = read_record(transaction, &lease.acquired_projection_ref)?;
            validate_record(&acquired, &store)?;
            if lease.consumer_id != consumer_id
                || lease.posture != ConfigProjectionConsumerLeasePostureV1::Released
                || lease.consumer_kind != ConfigProjectionConsumerKindV1::MemberDispatchV2
                || record_ref(&acquired) != lease.acquired_projection_ref
                || acquired.identity != record.identity
                || acquired.managed_gateway.activation_intent_ref
                    != record.managed_gateway.activation_intent_ref
                || acquired.managed_gateway.expected_gateway_ref
                    != record.managed_gateway.expected_gateway_ref
                || acquired.activation.publication_fence
                    != (crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                        fence_id: fence.clone(),
                    })
                || acquired.nonsecret_handoff.secret_handoff_ref != handoff_reference(&prepared)?
                || acquired.managed_gateway.posture != ManagedGatewayProjectionPostureV1::Dormant
                || acquired.created_at != lease.acquired_at
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            ensure_all_leases_released(transaction, &record.identity.series_id, Some(fence))?;
        }
        let effects = validate_registry_tree(transaction, true)?;
        if effects.iter().any(|effect| {
            effect.intent.series_id == record.identity.series_id
                && effect.intent.authority_store_id == record.identity.authority_store_id
                && effect.projection.is_none()
                && effect.resolution.is_none()
                && !pending_successor.is_some_and(|successor| {
                    successor
                        .nonsecret_handoff
                        .credential_source_ref
                        .preparation_id
                        == effect.intent.preparation_id
                        && successor.activation.publication_fence
                            == (crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                                fence_id: effect.intent.fence_id.clone(),
                            })
                })
        }) {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let fence = match &record.activation.publication_fence {
            crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id }
            | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => fence_id,
        };
        let attempt: Vec<_> = effects
            .iter()
            .filter(|effect| {
                effect.intent.series_id == record.identity.series_id
                    && effect.intent.authority_store_id == record.identity.authority_store_id
                    && effect.intent.fence_id == *fence
                    && effect.intent.preparation_id
                        == record
                            .nonsecret_handoff
                            .credential_source_ref
                            .preparation_id
            })
            .collect();
        if attempt.len() != 4
            || attempt.iter().any(|effect| {
                effect.resolution.is_none()
                    || effect.gateway_config.is_none()
                    || effect.projection.as_ref() != Some(record)
            })
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        for effect in &attempt {
            if let Some(registration) = &effect.child_cgroup {
                let resolution = effect
                    .resolution
                    .as_ref()
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                if resolution.disposition
                    != E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent
                    || resolution.observed_cgroup.as_ref() != Some(&registration.cgroup)
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
        }
        let boundaries: Vec<_> = attempt
            .iter()
            .filter_map(|effect| effect.boundary.as_ref())
            .collect();
        if boundaries.len() != 1
            || boundaries[0].posture != GatewayAccessPostureV1::Revoked
            || boundaries[0].access_boundary_id
                != record
                    .managed_gateway
                    .access_boundary_ref
                    .access_boundary_id
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        if record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Active {
            let allow = read_boundary(transaction, &record.managed_gateway.access_boundary_ref)?;
            if allow.posture != GatewayAccessPostureV1::AllowExactMember {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            validate_boundary_transition(&allow, boundaries[0])?;
        }
        let expected = record_ref(record);
        let mut proof = None;
        for effect in attempt {
            let matching: Vec<_> = effect
                .terminal_child_evidence
                .iter()
                .filter(|evidence| evidence.final_projection_ref == expected)
                .collect();
            if matching.len() != 1 {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            if proof.is_some_and(|old| old != matching[0]) {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            proof = Some(matching[0]);
        }
        Ok(())
    }

    fn publish_handoff_transition_in_transaction(
        transaction: &mut ConfigProjectionChildTransactionV1,
        identity: &ConfigProjectionIdentityV1,
        fence_id: &str,
        successor: &crate::ConfigProjectionSecretHandoffRevisionV1,
    ) -> Result<crate::SecretHandoffRefV1, ConfigProjectionFailureV1> {
        let store = require_store(transaction)?;
        validate_prefixed_uuid(fence_id, "cpf_")?;
        validate_handoff_revision(successor, &store)?;
        let predecessor_ref = successor
            .predecessor_ref
            .as_ref()
            .ok_or(ConfigProjectionFailureV1::StaleRevision)?;
        let successor_ref = handoff_reference(successor)?;
        let handoffs = open_directory_at(transaction.root.as_fd(), "handoffs")?;
        let handoff = open_directory_at(handoffs.as_fd(), &successor_ref.handoff_id)?;
        let current_bytes = read_file_at(handoff.as_fd(), "head.json")?;
        let current_ref: crate::SecretHandoffRefV1 =
            ConfigProjectionCodecV1::decode_canonical_json(&current_bytes)?;
        let expected_predecessor_bytes =
            ConfigProjectionCodecV1::encode_canonical_json(predecessor_ref)?;
        let successor_head_bytes = ConfigProjectionCodecV1::encode_canonical_json(&successor_ref)?;

        if current_ref == successor_ref {
            if current_bytes != successor_head_bytes {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            let installed = read_handoff_revision(transaction, &successor_ref)?;
            if installed != *successor {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            let predecessor = read_handoff_revision(transaction, predecessor_ref)?;
            validate_handoff_transition_attempt(
                transaction,
                &store,
                identity,
                fence_id,
                &predecessor,
                Some(successor),
            )?;
            return Ok(successor_ref);
        }
        if current_ref != *predecessor_ref || current_bytes != expected_predecessor_bytes {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }

        let predecessor = read_handoff_revision(transaction, predecessor_ref)?;
        validate_handoff_transition_attempt(
            transaction,
            &store,
            identity,
            fence_id,
            &predecessor,
            Some(successor),
        )?;

        let revisions = open_directory_at(handoff.as_fd(), "revisions")?;
        let successor_bytes = ConfigProjectionCodecV1::encode_canonical_json(successor)?;
        let successor_name = format!("{:020}.json", successor_ref.handoff_state_revision);
        write_immutable(
            revisions.as_fd(),
            &successor_name,
            &successor_bytes,
            "handoff-revision",
            transaction.owner_uid,
        )?;
        let installed: crate::ConfigProjectionSecretHandoffRevisionV1 =
            read_canonical_at(revisions.as_fd(), &successor_name)?;
        if installed != *successor {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }

        cas_head(
            handoff.as_fd(),
            Some(&expected_predecessor_bytes),
            &successor_head_bytes,
            "handoff-head",
            transaction.owner_uid,
        )?;
        if read_file_at(handoff.as_fd(), "head.json")? != successor_head_bytes
            || read_handoff_revision(transaction, &successor_ref)? != *successor
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(successor_ref)
    }

    fn validate_handoff_transition_attempt(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        identity: &ConfigProjectionIdentityV1,
        fence_id: &str,
        predecessor: &crate::ConfigProjectionSecretHandoffRevisionV1,
        successor: Option<&crate::ConfigProjectionSecretHandoffRevisionV1>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if let Some(successor) = successor {
            validate_handoff_transition(predecessor, successor)?;
        }
        let prepared = match predecessor.handoff.state {
            SecretHandoffStateV1::Prepared => predecessor.clone(),
            SecretHandoffStateV1::Delivered => read_handoff_revision(
                transaction,
                predecessor
                    .predecessor_ref
                    .as_ref()
                    .ok_or(ConfigProjectionFailureV1::WrongBinding)?,
            )?,
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        };
        let prepared = match prepared {
            value if value.handoff.state == SecretHandoffStateV1::Prepared => value,
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        };

        if read_retirement(transaction, &identity.series_id)?.is_some() {
            return Err(ConfigProjectionFailureV1::RetiredSeries);
        }
        let subject = subject_hash(identity)?;
        let subjects = open_directory_at(transaction.root.as_fd(), "subjects")?;
        let binding: ConfigProjectionSubjectBindingV1 =
            read_canonical_at(subjects.as_fd(), &format!("{subject}.json"))?;
        validate_binding(&binding, store)?;
        if binding.identity != *identity || binding.series_id != identity.series_id {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let projection_head = read_series_head(transaction, &identity.series_id)?;
        let current = read_record(transaction, &projection_head.head_ref)?;
        validate_record(&current, store)?;
        if record_ref(&current) != projection_head.head_ref || current.identity != *identity {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let current_fence = match &current.activation.publication_fence {
            crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id }
            | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => fence_id,
        };
        if current_fence != fence_id {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }

        let intents = open_directory_at(transaction.root.as_fd(), "gateway-intents")?;
        let intent: crate::ManagedGatewayActivationIntentV1 = read_canonical_at(
            intents.as_fd(),
            &format!(
                "{}.json",
                current
                    .managed_gateway
                    .activation_intent_ref
                    .activation_intent_id
            ),
        )?;
        validate_gateway_intent(&intent, store)?;
        if current.managed_gateway.activation_intent_ref
            != (crate::ManagedGatewayActivationIntentRefV1 {
                authority_store_id: intent.authority_store_id.clone(),
                activation_intent_id: intent.activation_intent_id.clone(),
                intent_hash: intent.intent_hash.clone(),
            })
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let series = open_series(transaction, &identity.series_id, false)?;
        let records = open_directory_at(series.as_fd(), "records")?;
        let dormant: AgentConfigProjectionRecordV1 = read_canonical_at(
            records.as_fd(),
            &format!(
                "{:020}-{}.json",
                intent.dormant_revision, intent.dormant_record_id
            ),
        )?;
        validate_record(&dormant, store)?;
        if dormant.identity != *identity
            || dormant.record_id != intent.dormant_record_id
            || dormant.revision != intent.dormant_revision
            || dormant.managed_gateway.activation_intent_ref
                != current.managed_gateway.activation_intent_ref
            || dormant.managed_gateway.expected_gateway_ref
                != current.managed_gateway.expected_gateway_ref
            || dormant.activation.activation_intent_ref != current.activation.activation_intent_ref
            || dormant.nonsecret_handoff.secret_handoff_ref != handoff_reference(&prepared)?
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        if successor.is_some_and(|successor| {
            matches!(
                successor.handoff.state,
                SecretHandoffStateV1::Failed | SecretHandoffStateV1::Expired
            )
        }) && current.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Active
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }

        let gateways = open_directory_at(transaction.root.as_fd(), "gateways")?;
        let gateway: crate::InWorldGatewayIdentityV1 = read_canonical_at(
            gateways.as_fd(),
            &format!(
                "{}.json",
                dormant
                    .managed_gateway
                    .expected_gateway_ref
                    .gateway_instance_id
            ),
        )?;
        let boundary = read_boundary(transaction, &dormant.managed_gateway.access_boundary_ref)?;
        let inputs = open_directory_at(transaction.root.as_fd(), "gateway-launch-inputs")?;
        let mut launch_input = None;
        for name in list_names(&inputs)? {
            let candidate: crate::ManagedGatewayLaunchInputV1 =
                read_canonical_at(inputs.as_fd(), &name)?;
            validate_gateway_launch_input(&candidate, store)?;
            if candidate.activation_intent_ref == dormant.managed_gateway.activation_intent_ref
                && launch_input.replace(candidate).is_some()
            {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
        }
        let launch_input = launch_input.ok_or(ConfigProjectionFailureV1::MissingPreparation)?;

        let cgroups = open_directory_at(transaction.root.as_fd(), "child-cgroups")?;
        let cgroup_series = open_directory_at(cgroups.as_fd(), &identity.series_id)?;
        let mut registrations: [Option<E3ChildCgroupRegistrationV1>; 3] = [None, None, None];
        for name in list_names(&cgroup_series)? {
            let candidate: E3ChildCgroupRegistrationV1 =
                read_canonical_at(cgroup_series.as_fd(), &name)?;
            let effect = resolve_kernel_effect_intent(
                transaction,
                store,
                &candidate.kernel_effect_intent_ref,
            )?;
            validate_child_cgroup_registration(&candidate, store, &effect)?;
            if candidate.fence_id != fence_id || effect.preparation_id != intent.preparation_id {
                continue;
            }
            let slot = match candidate.role {
                crate::E3TerminalProcessRoleV1::ManagedGateway => 0,
                crate::E3TerminalProcessRoleV1::ReadinessProbe => 1,
                crate::E3TerminalProcessRoleV1::Codex => 2,
            };
            if registrations[slot].replace(candidate).is_some() {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
        }
        let registrations = [
            registrations[0]
                .take()
                .ok_or(ConfigProjectionFailureV1::MissingPreparation)?,
            registrations[1]
                .take()
                .ok_or(ConfigProjectionFailureV1::MissingPreparation)?,
            registrations[2]
                .take()
                .ok_or(ConfigProjectionFailureV1::MissingPreparation)?,
        ];
        validate_prepared_gateway_chain(
            transaction,
            &dormant,
            (
                &prepared,
                &gateway,
                &boundary,
                &intent,
                &launch_input,
                &registrations,
            ),
        )
    }

    fn validate_handoff_transition(
        predecessor: &crate::ConfigProjectionSecretHandoffRevisionV1,
        successor: &crate::ConfigProjectionSecretHandoffRevisionV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let predecessor_ref = handoff_reference(predecessor)?;
        if successor.predecessor_ref.as_ref() != Some(&predecessor_ref)
            || successor.handoff.state_revision != predecessor.handoff.state_revision + 1
            || successor.authority_store_id != predecessor.authority_store_id
            || successor.schema_version != predecessor.schema_version
        {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        let before = &predecessor.handoff;
        let after = &successor.handoff;
        if before.handoff_id != after.handoff_id
            || before.orchestration_session_id != after.orchestration_session_id
            || before.world_id != after.world_id
            || before.world_generation != after.world_generation
            || before.retained_participant_id != after.retained_participant_id
            || before.runtime_family != after.runtime_family
            || before.credential_source_ref != after.credential_source_ref
            || before.receiving_gateway_ref != after.receiving_gateway_ref
            || before.delivery != after.delivery
            || before.created_at != after.created_at
            || before.expires_at != after.expires_at
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        match (before.state, after.state) {
            (SecretHandoffStateV1::Prepared, SecretHandoffStateV1::Delivered)
                if after.delivered_at.is_some()
                    && after.consumed_at.is_none()
                    && after.failure_diagnostic_ref.is_none() => {}
            (SecretHandoffStateV1::Delivered, SecretHandoffStateV1::Consumed)
                if after.delivered_at == before.delivered_at
                    && after.consumed_at.is_some()
                    && after.failure_diagnostic_ref.is_none() => {}
            (
                SecretHandoffStateV1::Prepared | SecretHandoffStateV1::Delivered,
                SecretHandoffStateV1::Failed | SecretHandoffStateV1::Expired,
            ) if after.delivered_at == before.delivered_at && after.consumed_at.is_none() => {
                if let Some(diagnostic) = &after.failure_diagnostic_ref {
                    if diagnostic.schema_version != 1
                        || diagnostic.authority_store_id != successor.authority_store_id
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    validate_prefixed_uuid(&diagnostic.diagnostic_id, "rdg_")?;
                    validate_sha256(&diagnostic.diagnostic_hash)?;
                }
            }
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        }
        Ok(())
    }

    fn validate_gateway_preparation_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
        effects: &mut [E3KernelEffectRecoveryV1],
    ) -> Result<(), ConfigProjectionFailureV1> {
        for (directory, kind) in [
            ("gateways", "gateway"),
            ("gateway-acks", "gateway-ack"),
            ("gateway-intents", "gateway-intent"),
            ("gateway-launch-inputs", "gateway-launch-input"),
        ] {
            let Some(root) = open_optional_directory_at(transaction.root.as_fd(), directory)?
            else {
                continue;
            };
            for name in list_names(&root)? {
                let bytes = read_file_at(root.as_fd(), &name)?;
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != kind {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                } else if verify_objects {
                    let (expected, _) = validate_gateway_recovery_candidate(
                        transaction,
                        &[directory.to_owned()],
                        kind,
                        &bytes,
                    )?;
                    if name != expected {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    if kind == "gateway-launch-input" {
                        let input: crate::ManagedGatewayLaunchInputV1 =
                            ConfigProjectionCodecV1::decode_canonical_json(&bytes)?;
                        let record = read_record(transaction, &input.dormant_projection_ref)?;
                        validate_record(&record, &require_store(transaction)?)?;
                        let prepared =
                            read_handoff_revision(transaction, &input.secret_handoff_prepared_ref)?;
                        let boundary = read_boundary(transaction, &input.access_boundary_ref)?;
                        let gateways = open_directory_at(transaction.root.as_fd(), "gateways")?;
                        let gateway = read_canonical_at(
                            gateways.as_fd(),
                            &format!("{}.json", input.gateway_ref.gateway_instance_id),
                        )?;
                        let intents =
                            open_directory_at(transaction.root.as_fd(), "gateway-intents")?;
                        let intent: crate::ManagedGatewayActivationIntentV1 = read_canonical_at(
                            intents.as_fd(),
                            &format!("{}.json", input.activation_intent_ref.activation_intent_id),
                        )?;
                        let mut registrations = [None, None, None];
                        for effect in effects.iter().filter(|effect| {
                            effect.intent.authority_store_id == input.authority_store_id
                                && effect.intent.series_id == record.identity.series_id
                                && effect.intent.fence_id == intent.fence_id
                                && effect.intent.preparation_id == intent.preparation_id
                        }) {
                            if let Some(registration) = &effect.child_cgroup {
                                let slot = match registration.role {
                                    crate::E3TerminalProcessRoleV1::ManagedGateway => 0,
                                    crate::E3TerminalProcessRoleV1::ReadinessProbe => 1,
                                    crate::E3TerminalProcessRoleV1::Codex => 2,
                                };
                                if registrations[slot].replace(registration.clone()).is_some() {
                                    return Err(ConfigProjectionFailureV1::Conflict);
                                }
                            }
                        }
                        let registrations = [
                            registrations[0]
                                .take()
                                .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                            registrations[1]
                                .take()
                                .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                            registrations[2]
                                .take()
                                .ok_or(ConfigProjectionFailureV1::PartialPublication)?,
                        ];
                        validate_prepared_gateway_chain(
                            transaction,
                            &record,
                            (
                                &prepared,
                                &gateway,
                                &boundary,
                                &intent,
                                &input,
                                &registrations,
                            ),
                        )?;
                        for effect in effects.iter_mut().filter(|effect| {
                            effect.intent.authority_store_id == input.authority_store_id
                                && effect.intent.series_id == record.identity.series_id
                                && effect.intent.fence_id == intent.fence_id
                                && effect.intent.preparation_id == intent.preparation_id
                        }) {
                            let retained = effect
                                .projection
                                .as_ref()
                                .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                            if retained.identity != record.identity
                                || retained.managed_gateway.expected_gateway_ref
                                    != record.managed_gateway.expected_gateway_ref
                                || retained.managed_gateway.activation_intent_ref
                                    != record.managed_gateway.activation_intent_ref
                                || retained.nonsecret_handoff.credential_source_ref
                                    != prepared.handoff.credential_source_ref
                            {
                                return Err(ConfigProjectionFailureV1::WrongBinding);
                            }
                            // Even equal config identities from distinct launch inputs are ambiguous.
                            if effect
                                .gateway_config
                                .replace(input.gateway_config.clone())
                                .is_some()
                            {
                                return Err(ConfigProjectionFailureV1::Conflict);
                            }
                        }
                    }
                }
            }
        }
        if verify_objects {
            for effect in effects.iter() {
                // A recorded preparation requires its launch input even when another
                // dependency has also disappeared. Absence is only valid before publication.
                if let Some(record) = &effect.projection {
                    if effect.gateway_config.is_none() {
                        let intents =
                            open_directory_at(transaction.root.as_fd(), "gateway-intents")?;
                        let handoffs = open_directory_at(transaction.root.as_fd(), "handoffs")?;
                        let has_preparation = read_optional_canonical_at::<
                            crate::ManagedGatewayActivationIntentV1,
                        >(
                            intents.as_fd(),
                            &format!(
                                "{}.json",
                                record
                                    .managed_gateway
                                    .activation_intent_ref
                                    .activation_intent_id
                            ),
                        )?
                        .is_some()
                            || open_optional_directory_at(
                                handoffs.as_fd(),
                                &record.nonsecret_handoff.secret_handoff_ref.handoff_id,
                            )?
                            .is_some()
                            || effects.iter().any(|other| {
                                other.intent.authority_store_id == effect.intent.authority_store_id
                                    && other.intent.series_id == effect.intent.series_id
                                    && other.intent.fence_id == effect.intent.fence_id
                                    && other.intent.preparation_id == effect.intent.preparation_id
                                    && other.child_cgroup.is_some()
                            });
                        if has_preparation {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        }
                    }
                }
            }
        }
        let Some(root) = open_optional_directory_at(transaction.root.as_fd(), "handoffs")? else {
            return Ok(());
        };
        for id in list_names(&root)? {
            validate_prefixed_uuid(&id, "gsh_")?;
            let handoff = open_directory_at(root.as_fd(), &id)?;
            for name in list_names(&handoff)? {
                match name.as_str() {
                    "revisions" => {
                        let revisions = open_directory_at(handoff.as_fd(), "revisions")?;
                        for name in list_names(&revisions)? {
                            let bytes = read_file_at(revisions.as_fd(), &name)?;
                            if name.starts_with(".e3-tmp.") {
                                if parse_temp_name(&name)? != "handoff-revision" {
                                    return Err(ConfigProjectionFailureV1::PartialPublication);
                                }
                            } else if verify_objects {
                                let (expected, _) = validate_gateway_recovery_candidate(
                                    transaction,
                                    &["handoffs".to_owned(), id.clone(), "revisions".to_owned()],
                                    "handoff-revision",
                                    &bytes,
                                )?;
                                if name != expected {
                                    return Err(ConfigProjectionFailureV1::WrongBinding);
                                }
                            }
                        }
                    }
                    "head.json" => {
                        let reference: crate::SecretHandoffRefV1 =
                            read_canonical_at(handoff.as_fd(), &name)?;
                        if reference.handoff_id != id {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                        if verify_objects {
                            read_handoff_revision(transaction, &reference)?;
                        }
                    }
                    _ if name.starts_with(".e3-tmp.")
                        && parse_temp_name(&name)? == "handoff-head" =>
                    {
                        read_file_at(handoff.as_fd(), &name)?;
                    }
                    _ => return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture),
                }
            }
        }
        Ok(())
    }

    // Publication authority is deliberately separate from historical ACK validation. The
    // original lease was acquired against Dormant and is never relabeled as ReadyClosed.
    fn validate_gateway_activation_authority_v1(
        transaction: &ConfigProjectionChildTransactionV1,
        expected: &ConfigProjectionRefV1,
        ready: Option<&AgentConfigProjectionRecordV1>,
        lease: &ConfigProjectionConsumerLeaseV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        use ConfigProjectionFailureV1::{RetiredSeries, StaleRevision, WrongBinding};
        let store = require_store(transaction)?;
        let dormant = read_record(transaction, expected)?;
        validate_record(&dormant, &store)?;
        if record_ref(&dormant) != *expected
            || dormant.managed_gateway.posture != ManagedGatewayProjectionPostureV1::Dormant
            || !matches!(
                dormant.activation.publication_fence,
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { .. }
            )
            || lease.revision != 1
            || lease.predecessor_lease_hash.is_some()
            || lease.consumer_kind != ConfigProjectionConsumerKindV1::MemberDispatchV2
            || lease.consumer_id
                != prepared_member_dispatch_consumer_id_v1(
                    &dormant
                        .nonsecret_handoff
                        .credential_source_ref
                        .preparation_id,
                )?
        {
            return Err(WrongBinding);
        }
        if read_retirement(transaction, &expected.series_id)?.is_some() {
            return Err(RetiredSeries);
        }
        let subjects = open_directory_at(transaction.root.as_fd(), "subjects")?;
        let subject = subject_hash(&dormant.identity)?;
        let binding: ConfigProjectionSubjectBindingV1 =
            read_canonical_at(subjects.as_fd(), &format!("{subject}.json"))?;
        validate_binding(&binding, &store)?;
        if binding.identity != dormant.identity || binding.subject_hash != subject {
            return Err(WrongBinding);
        }
        resolve_held_lease(transaction, lease, expected)?;
        let head = read_series_head(transaction, &expected.series_id)?;
        if let Some(ready) = ready {
            validate_record(ready, &store)?;
            validate_record_transition(&dormant, ready)?;
            if ready.managed_gateway.posture != ManagedGatewayProjectionPostureV1::ReadyClosed
                || ready.predecessor_ref.as_ref() != Some(expected)
                || ready.activation.publication_fence != dormant.activation.publication_fence
                || ready.native != dormant.native
            {
                return Err(WrongBinding);
            }
            if head.head_ref == record_ref(ready) {
                if read_record(transaction, &head.head_ref)? != *ready {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                return Ok(());
            }
        }
        if head.head_ref != *expected {
            return Err(StaleRevision);
        }
        Ok(())
    }

    fn resolve_gateway_ack_in_transaction_v1(
        transaction: &ConfigProjectionChildTransactionV1,
        reference: &crate::ManagedGatewayActivationAckRefV1,
    ) -> Result<crate::ManagedGatewayActivationAckV1, ConfigProjectionFailureV1> {
        validate_ack_ref(reference)?;
        let root = open_directory_at(transaction.root.as_fd(), "gateway-acks")?;
        let ack: crate::ManagedGatewayActivationAckV1 = read_canonical_at(
            root.as_fd(),
            &format!("{}.json", reference.activation_ack_id),
        )?;
        if ack.authority_store_id != reference.authority_store_id
            || ack.activation_ack_id != reference.activation_ack_id
            || ack.ack_hash != reference.ack_hash
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(ack)
    }

    fn publish_gateway_ack_in_transaction_v1(
        transaction: &ConfigProjectionChildTransactionV1,
        record: &AgentConfigProjectionRecordV1,
        ack: &crate::ManagedGatewayActivationAckV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_gateway_ack_chain_v1(transaction, Some(record), Some(ack))?;
        let root = open_or_create_directory(
            transaction.root.as_fd(),
            "gateway-acks",
            transaction.owner_uid,
        )?;
        write_immutable(
            root.as_fd(),
            &format!("{}.json", ack.activation_ack_id),
            &ConfigProjectionCodecV1::encode_canonical_json(ack)?,
            "gateway-ack",
            transaction.owner_uid,
        )?;
        let reference = crate::ManagedGatewayActivationAckRefV1 {
            authority_store_id: ack.authority_store_id.clone(),
            activation_ack_id: ack.activation_ack_id.clone(),
            ack_hash: ack.ack_hash.clone(),
        };
        if resolve_gateway_ack_in_transaction_v1(transaction, &reference)? != *ack {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        validate_gateway_ack_chain_v1(transaction, Some(record), None)
    }

    // Historical evidence never requires a live PID or a still-Held lease. Every dependency
    // is resolved by immutable identity, including process registration, not by PID alone.
    fn validate_gateway_ack_chain_v1(
        transaction: &ConfigProjectionChildTransactionV1,
        record: Option<&AgentConfigProjectionRecordV1>,
        supplied_ack: Option<&crate::ManagedGatewayActivationAckV1>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        use ConfigProjectionFailureV1::{Conflict, PartialPublication, WrongBinding};
        if record.is_some_and(|r| {
            r.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Dormant
        }) {
            return if supplied_ack.is_none() {
                Ok(())
            } else {
                Err(WrongBinding)
            };
        }
        let resolved;
        let ack = if let Some(ack) = supplied_ack {
            ack
        } else {
            let reference = record
                .and_then(|r| r.managed_gateway.activation_ack_ref.as_ref())
                .ok_or(PartialPublication)?;
            resolved = resolve_gateway_ack_in_transaction_v1(transaction, reference)?;
            &resolved
        };
        let store = require_store(transaction)?;
        validate_prefixed_uuid(&ack.activation_ack_id, "gaa_")?;
        validate_timestamp(&ack.observed_at)?;
        validate_hash_field(
            &ack.ack_hash,
            hash_omitting(
                "substrate.e3.managed-gateway-activation-ack.v1",
                "ack",
                ack,
                "ack_hash",
            )?,
        )?;
        let dormant = read_record(transaction, &ack.dormant_projection_ref)?;
        validate_record(&dormant, &store)?;
        if record_ref(&dormant) != ack.dormant_projection_ref
            || dormant.managed_gateway.posture != ManagedGatewayProjectionPostureV1::Dormant
        {
            return Err(WrongBinding);
        }
        let identity = &dormant.identity;
        let crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } =
            &dormant.activation.publication_fence
        else {
            return Err(WrongBinding);
        };
        let intents = open_directory_at(transaction.root.as_fd(), "gateway-intents")?;
        let intent: crate::ManagedGatewayActivationIntentV1 = read_canonical_at(
            intents.as_fd(),
            &format!(
                "{}.json",
                dormant
                    .managed_gateway
                    .activation_intent_ref
                    .activation_intent_id
            ),
        )?;
        let inputs = open_directory_at(transaction.root.as_fd(), "gateway-launch-inputs")?;
        validate_prefixed_uuid(&ack.launch_input_ref.launch_input_id, "gal_")?;
        let input: crate::ManagedGatewayLaunchInputV1 = read_canonical_at(
            inputs.as_fd(),
            &format!("{}.json", ack.launch_input_ref.launch_input_id),
        )?;
        let gateways = open_directory_at(transaction.root.as_fd(), "gateways")?;
        let gateway: crate::InWorldGatewayIdentityV1 = read_canonical_at(
            gateways.as_fd(),
            &format!(
                "{}.json",
                dormant
                    .managed_gateway
                    .expected_gateway_ref
                    .gateway_instance_id
            ),
        )?;
        let boundary = read_boundary(transaction, &dormant.managed_gateway.access_boundary_ref)?;
        let prepared =
            read_handoff_revision(transaction, &dormant.nonsecret_handoff.secret_handoff_ref)?;
        let cgroups = open_directory_at(transaction.root.as_fd(), "child-cgroups")?;
        let cgroups = open_directory_at(cgroups.as_fd(), &identity.series_id)?;
        let mut registrations = [None, None, None];
        for name in list_names(&cgroups)? {
            if name.starts_with(".e3-tmp.") {
                continue;
            }
            let group: E3ChildCgroupRegistrationV1 = read_canonical_at(cgroups.as_fd(), &name)?;
            if group.fence_id != *fence_id {
                continue;
            }
            let slot = match group.role {
                crate::E3TerminalProcessRoleV1::ManagedGateway => 0,
                crate::E3TerminalProcessRoleV1::ReadinessProbe => 1,
                crate::E3TerminalProcessRoleV1::Codex => 2,
            };
            if registrations[slot].replace(group).is_some() {
                return Err(Conflict);
            }
        }
        let registrations = [
            registrations[0].take().ok_or(PartialPublication)?,
            registrations[1].take().ok_or(PartialPublication)?,
            registrations[2].take().ok_or(PartialPublication)?,
        ];
        validate_prepared_gateway_chain(
            transaction,
            &dormant,
            (
                &prepared,
                &gateway,
                &boundary,
                &intent,
                &input,
                &registrations,
            ),
        )?;
        let consumed = read_handoff_revision(transaction, &ack.secret_handoff_ref)?;
        let handoffs = open_directory_at(transaction.root.as_fd(), "handoffs")?;
        let handoff = open_directory_at(handoffs.as_fd(), &ack.secret_handoff_ref.handoff_id)?;
        let handoff_head: crate::SecretHandoffRefV1 =
            read_canonical_at(handoff.as_fd(), "head.json")?;
        if handoff_head != ack.secret_handoff_ref {
            return Err(WrongBinding);
        }
        let delivered = read_handoff_revision(
            transaction,
            consumed.predecessor_ref.as_ref().ok_or(WrongBinding)?,
        )?;
        validate_handoff_transition(&prepared, &delivered)?;
        validate_handoff_transition(&delivered, &consumed)?;
        let process = &ack.gateway_process_identity;
        let security = &ack.child_security_attestation;
        let artifact = &identity.runtime_artifacts.managed_gateway;
        validate_hash_field(
            &security.attestation_hash,
            hash_omitting(
                "substrate.e3.child-security-attestation.v1",
                "attestation",
                security,
                "attestation_hash",
            )?,
        )?;
        for hash in [
            &security.e2_enforcement_plan_hash,
            &security.derived_support_ruleset_hash,
            &security.role_narrowing_ruleset_hash,
            &security.effective_landlock_hash,
            &security.enforcement_input_hash,
            &security.denied_control_probe_hash,
        ] {
            validate_sha256(hash)?;
        }
        let namespace = &security.user_namespace;
        if namespace.namespace_device_id == 0
            || namespace.namespace_inode == 0
            || namespace.parent_namespace_device_id == 0
            || namespace.parent_namespace_inode == 0
            || (namespace.namespace_device_id, namespace.namespace_inode)
                == (
                    namespace.parent_namespace_device_id,
                    namespace.parent_namespace_inode,
                )
            || namespace.uid_map.inside_id != security.real_uid
            || namespace.uid_map.outside_id != security.real_uid
            || namespace.uid_map.length != 1
            || namespace.gid_map.inside_id != security.real_gid
            || namespace.gid_map.outside_id != security.real_gid
            || namespace.gid_map.length != 1
        {
            return Err(WrongBinding);
        }
        validate_sha256(&process.secret_ready_attestation_hash)?;
        if ack.schema_version != 1
            || ack.authority_store_id != store.authority_store_id
            || ack.config_projection_identity_hash != identity.identity_hash
            || ack.activation_intent_ref != dormant.managed_gateway.activation_intent_ref
            || ack.gateway_ref != dormant.managed_gateway.expected_gateway_ref
            || ack.access_boundary_ref != dormant.managed_gateway.access_boundary_ref
            || ack.listener_identity != boundary.gateway_listener
            || ack.readiness_nonce != intent.readiness_nonce
            || ack.launch_input_ref.authority_store_id != input.authority_store_id
            || ack.launch_input_ref.launch_input_id != input.launch_input_id
            || ack.launch_input_ref.launch_input_hash != input.launch_input_hash
            || ack.gateway_ready_revision != 1
            || ack.secret_handoff_terminal_state != SecretHandoffStateV1::Consumed
            || consumed.handoff.state != SecretHandoffStateV1::Consumed
            || delivered.handoff.state != SecretHandoffStateV1::Delivered
            || consumed.handoff.consumed_at.as_ref() != Some(&ack.observed_at)
            || process.pid == 0
            || process.pid_start_time_ticks == 0
            || process.pidfd_inode == 0
            || process.executable_device_id != artifact.device_id
            || process.executable_inode != artifact.inode
            || process.executable_sha256 != artifact.sha256
            || process.process_cgroup != registrations[0].cgroup
            || process.child_security_attestation_hash != security.attestation_hash
            || security.schema_version != 1
            || security.child_role != crate::E3IsolatedChildRoleV1::ManagedGateway
            || security.projection_identity_hash != identity.identity_hash
            || security.pid != process.pid
            || security.pid_start_time_ticks != process.pid_start_time_ticks
            || security.kernel_boot_id != registrations[0].kernel_boot_id
            || security.policy_snapshot_ref != identity.immutable_launch_cap.policy_snapshot_ref
            || security.policy_snapshot_hash != identity.immutable_launch_cap.policy_snapshot_hash
            || security.policy_snapshot_revision
                != identity.immutable_launch_cap.policy_snapshot_revision
            || security.real_uid == 0
            || security.real_gid == 0
            || [
                security.effective_uid,
                security.saved_uid,
                security.filesystem_uid,
            ]
            .iter()
            .any(|id| *id != security.real_uid)
            || [
                security.effective_gid,
                security.saved_gid,
                security.filesystem_gid,
            ]
            .iter()
            .any(|id| *id != security.real_gid)
            || security.supplementary_group_count != 0
            || !security.no_new_privs
            || security.dumpable != 0
            || security.tracer_pid != 0
            || security.seccomp_mode != 2
            || security.landlock_abi == 0
            || security.cap_last_cap > 63
            || [
                &security.cap_inheritable,
                &security.cap_permitted,
                &security.cap_effective,
                &security.cap_bounding,
                &security.cap_ambient,
            ]
            .iter()
            .any(|v| v.as_str() != "0000000000000000")
        {
            return Err(WrongBinding);
        }
        let processes = open_directory_at(transaction.root.as_fd(), "child-processes")?;
        let processes = open_directory_at(processes.as_fd(), &identity.series_id)?;
        let mut registered = false;
        for name in list_names(&processes)? {
            if name.starts_with(".e3-tmp.") {
                continue;
            }
            let registration: E3ChildProcessRegistrationV1 =
                read_canonical_at(processes.as_fd(), &name)?;
            if registration.fence_id != *fence_id
                || registration.role != crate::E3TerminalProcessRoleV1::ManagedGateway
            {
                continue;
            }
            let exact = resolve_child_process_registration(
                transaction,
                &store,
                &identity.series_id,
                &registration.registration_id,
            )?;
            if exact != registration
                || name != format!("{}.json", registration.registration_id)
                || registered
                || registration.pid != process.pid
                || registration.pid_start_time_ticks != process.pid_start_time_ticks
                || registration.process_cgroup != process.process_cgroup
                || registration.kernel_boot_id != security.kernel_boot_id
                || registration.registered_at.0 > ack.observed_at.0
            {
                return Err(WrongBinding);
            }
            registered = true;
        }
        if !registered {
            return Err(PartialPublication);
        }
        if let Some(record) = record {
            let reference = crate::ManagedGatewayActivationAckRefV1 {
                authority_store_id: ack.authority_store_id.clone(),
                activation_ack_id: ack.activation_ack_id.clone(),
                ack_hash: ack.ack_hash.clone(),
            };
            if record.identity != *identity
                || record.managed_gateway.activation_ack_ref.as_ref() != Some(&reference)
                || record.nonsecret_handoff.activation_ack_ref.as_ref() != Some(&reference)
                || record.activation.gateway_activation_ack_ref.as_ref() != Some(&reference)
                || record.managed_gateway.activation_intent_ref != ack.activation_intent_ref
                || record.managed_gateway.expected_gateway_ref != ack.gateway_ref
                || record.nonsecret_handoff.secret_handoff_ref != ack.secret_handoff_ref
                || record.nonsecret_handoff.credential_source_ref
                    != consumed.handoff.credential_source_ref
                || record.nonsecret_handoff.delivery != consumed.handoff.delivery
                || record.native != dormant.native
            {
                return Err(WrongBinding);
            }
            match record.managed_gateway.posture {
                ManagedGatewayProjectionPostureV1::ReadyClosed => {
                    if record.predecessor_ref.as_ref() != Some(&ack.dormant_projection_ref)
                        || record.activation.publication_fence
                            != dormant.activation.publication_fence
                        || record.managed_gateway.access_boundary_ref != ack.access_boundary_ref
                        || record.created_at != ack.observed_at
                    {
                        return Err(WrongBinding);
                    }
                }
                ManagedGatewayProjectionPostureV1::Active => {
                    let crate::ConfigProjectionPublicationFenceV1::Released {
                        closed_record_ref,
                        activation_ack_ref,
                        ..
                    } = &record.activation.publication_fence
                    else {
                        return Err(WrongBinding);
                    };
                    let closed = read_record(transaction, closed_record_ref)?;
                    validate_record(&closed, &store)?;
                    if record_ref(&closed) != *closed_record_ref
                        || activation_ack_ref != &reference
                        || closed.managed_gateway.posture
                            != ManagedGatewayProjectionPostureV1::ReadyClosed
                    {
                        return Err(WrongBinding);
                    }
                    validate_gateway_ack_chain_v1(transaction, Some(&closed), Some(ack))?;
                }
                _ => return Err(WrongBinding),
            }
        }
        Ok(())
    }

    fn publish_record_in_transaction(
        transaction: &mut ConfigProjectionChildTransactionV1,
        record: &AgentConfigProjectionRecordV1,
        expected_head: Option<&ConfigProjectionRefV1>,
        expected_posture: ManagedGatewayProjectionPostureV1,
    ) -> Result<ConfigProjectionRefV1, ConfigProjectionFailureV1> {
        let store = ensure_store(
            transaction,
            Some(record.identity.authority_store_id.as_str()),
        )?;
        validate_record(record, &store)?;
        validate_gateway_ack_chain_v1(transaction, Some(record), None)?;
        if record.managed_gateway.posture != expected_posture {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        if read_retirement(transaction, &record.identity.series_id)?.is_some() {
            return Err(ConfigProjectionFailureV1::RetiredSeries);
        }
        let reference = record_ref(record);
        match (
            expected_head,
            record.revision,
            record.predecessor_ref.as_ref(),
        ) {
            (None, 1, None) => publish_first(transaction, &store, record, &reference)?,
            (Some(expected), revision, Some(predecessor))
                if revision == expected.revision.saturating_add(1) && predecessor == expected =>
            {
                publish_successor(transaction, &store, record, expected, &reference)?
            }
            _ => return Err(ConfigProjectionFailureV1::StaleRevision),
        }
        Ok(reference)
    }

    #[allow(
        clippy::large_enum_variant,
        reason = "the contract returns owned, unboxed projection records"
    )]
    pub enum ConfigProjectionResolutionV1 {
        Current {
            identity: ConfigProjectionIdentityV1,
            projection_ref: ConfigProjectionRefV1,
            record: AgentConfigProjectionRecordV1,
            capability: PublishedConfigProjectionCapabilityV1,
        },
        Retired {
            final_projection_ref: ConfigProjectionRefV1,
            retirement: ConfigProjectionRetirementV1,
        },
        Missing,
        UnsupportedLegacyState {
            diagnostic_ref: RedactedDiagnosticRefV1,
        },
        UnsupportedNewerSchema {
            observed_schema_version: u32,
        },
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct RedactedDiagnosticRefV1 {
        pub schema_version: u32,
        pub authority_store_id: String,
        pub diagnostic_id: String,
        pub diagnostic_hash: String,
    }

    pub struct PublishedConfigProjectionCapabilityV1 {
        key: (String, String),
        _identity: ConfigProjectionIdentityV1,
        _projection_ref: ConfigProjectionRefV1,
        _held_lease: ConfigProjectionConsumerLeaseV1,
        _parent: Arc<dyn ConfigProjectionHsaAuthorityV1>,
    }

    impl PublishedConfigProjectionCapabilityV1 {
        fn new(
            authority_store_id: String,
            identity: ConfigProjectionIdentityV1,
            projection_ref: ConfigProjectionRefV1,
            held_lease: ConfigProjectionConsumerLeaseV1,
            parent: Arc<dyn ConfigProjectionHsaAuthorityV1>,
        ) -> Result<Self, ConfigProjectionFailureV1> {
            let key = (authority_store_id, identity.series_id.clone());
            let mut counts = live_capabilities()
                .lock()
                .map_err(|_| ConfigProjectionFailureV1::Conflict)?;
            *counts.entry(key.clone()).or_default() += 1;
            drop(counts);
            Ok(Self {
                key,
                _identity: identity,
                _projection_ref: projection_ref,
                _held_lease: held_lease,
                _parent: parent,
            })
        }
    }

    impl Drop for PublishedConfigProjectionCapabilityV1 {
        fn drop(&mut self) {
            if let Ok(mut counts) = live_capabilities().lock() {
                if let Some(count) = counts.get_mut(&self.key) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        counts.remove(&self.key);
                    }
                }
            }
        }
    }

    fn live_capabilities() -> &'static Mutex<BTreeMap<(String, String), usize>> {
        LIVE_CAPABILITIES.get_or_init(|| Mutex::new(BTreeMap::new()))
    }

    struct ConfigProjectionChildTransactionV1 {
        root: File,
        root_device: u64,
        root_inode: u64,
        lock: Option<File>,
        lock_device: u64,
        lock_inode: u64,
        owner_uid: libc::uid_t,
        native_source_gid: libc::gid_t,
        observed_resolution_files: Vec<(Vec<String>, Vec<u8>)>,
        observed_newer_schema: bool,
    }

    impl ConfigProjectionChildTransactionV1 {
        fn begin(
            authority_fd: BorrowedFd<'_>,
            require_existing: bool,
        ) -> Result<Self, ConfigProjectionFailureV1> {
            let authority_metadata = fstat(authority_fd.as_raw_fd())?;
            if authority_metadata.st_mode & libc::S_IFMT != libc::S_IFDIR {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            let root = if require_existing {
                open_directory_at(authority_fd, CHILD_NAME)?
            } else {
                open_or_create_directory(authority_fd, CHILD_NAME, authority_metadata.st_uid)?
            };
            verify_directory(&root, authority_metadata.st_uid, 0o700)?;
            let root_metadata = fstat(root.as_raw_fd())?;
            let lock = if require_existing {
                open_file_at(root.as_fd(), "lock", libc::O_RDWR, 0)?
            } else {
                open_or_create_regular_file(root.as_fd(), "lock", authority_metadata.st_uid)?
            };
            verify_regular_file(&lock, authority_metadata.st_uid, 0o600)?;
            flock(&lock, libc::LOCK_EX)?;
            let lock_metadata = fstat(lock.as_raw_fd())?;
            let transaction = Self {
                root,
                root_device: root_metadata.st_dev,
                root_inode: root_metadata.st_ino,
                lock: Some(lock),
                lock_device: lock_metadata.st_dev,
                lock_inode: lock_metadata.st_ino,
                owner_uid: authority_metadata.st_uid,
                native_source_gid: authority_metadata.st_gid,
                observed_resolution_files: Vec::new(),
                observed_newer_schema: false,
            };
            validate_registry_tree(&transaction, false)?;
            transaction.verify_scope()?;
            Ok(transaction)
        }

        fn finish(mut self) -> Result<(), ConfigProjectionFailureV1> {
            self.verify_scope()?;
            validate_registry_tree(&self, !self.observed_newer_schema)?;
            self.revalidate_observed_resolution_files()?;
            fsync_directory_tree(&self.root)?;
            self.verify_scope()?;
            if let Some(lock) = self.lock.take() {
                flock(&lock, libc::LOCK_UN)?;
            }
            Ok(())
        }

        fn read_resolution_object<T>(
            &mut self,
            path: &[String],
        ) -> Result<Option<ResolutionObjectV1<T>>, ConfigProjectionFailureV1>
        where
            T: DeserializeOwned + Serialize,
        {
            let Some((file_name, directories)) = path.split_last() else {
                return Err(ConfigProjectionFailureV1::Malformed);
            };
            let mut parent = reopen_same(&self.root)?;
            for directory in directories {
                parent = open_directory_at(parent.as_fd(), directory)?;
            }
            let bytes = match read_file_at(parent.as_fd(), file_name) {
                Ok(bytes) => bytes,
                Err(ConfigProjectionFailureV1::MissingPreparation) => return Ok(None),
                Err(error) => return Err(error),
            };
            self.observed_resolution_files
                .push((path.to_vec(), bytes.clone()));
            if let Some(observed_schema_version) = newer_schema_version(&bytes) {
                self.observed_newer_schema = true;
                return Ok(Some(ResolutionObjectV1::Newer(observed_schema_version)));
            }
            Ok(Some(ResolutionObjectV1::V1(
                ConfigProjectionCodecV1::decode_canonical_json(&bytes)?,
            )))
        }

        fn revalidate_observed_resolution_files(&self) -> Result<(), ConfigProjectionFailureV1> {
            for (path, expected) in &self.observed_resolution_files {
                let (file_name, directories) = path
                    .split_last()
                    .ok_or(ConfigProjectionFailureV1::Malformed)?;
                let mut parent = reopen_same(&self.root)?;
                for directory in directories {
                    parent = open_directory_at(parent.as_fd(), directory)?;
                }
                if read_file_at(parent.as_fd(), file_name)? != *expected {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
            }
            Ok(())
        }

        fn verify_scope(&self) -> Result<(), ConfigProjectionFailureV1> {
            let root = fstat(self.root.as_raw_fd())?;
            let lock = self
                .lock
                .as_ref()
                .ok_or(ConfigProjectionFailureV1::Conflict)?;
            let lock_metadata = fstat(lock.as_raw_fd())?;
            if root.st_dev != self.root_device
                || root.st_ino != self.root_inode
                || lock_metadata.st_dev != self.lock_device
                || lock_metadata.st_ino != self.lock_inode
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            verify_directory(&self.root, self.owner_uid, 0o700)?;
            verify_regular_file(lock, self.owner_uid, 0o600)
        }
    }

    enum ResolutionObjectV1<T> {
        V1(T),
        Newer(u32),
    }

    fn newer_schema_version(bytes: &[u8]) -> Option<u32> {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        let discriminator = SchemaVersionDiscriminatorV1::deserialize(&mut deserializer).ok()?;
        deserializer.end().ok()?;
        discriminator.0.filter(|version| *version > 1)
    }

    struct SchemaVersionDiscriminatorV1(Option<u32>);

    impl<'de> Deserialize<'de> for SchemaVersionDiscriminatorV1 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(SchemaVersionDiscriminatorVisitorV1)
        }
    }

    struct SchemaVersionDiscriminatorVisitorV1;

    impl<'de> Visitor<'de> for SchemaVersionDiscriminatorVisitorV1 {
        type Value = SchemaVersionDiscriminatorV1;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a complete object with at most one unsigned schema_version")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut schema_version = None;
            while let Some(key) = map.next_key::<String>()? {
                if key == "schema_version" {
                    if schema_version.is_some() {
                        return Err(serde::de::Error::duplicate_field("schema_version"));
                    }
                    schema_version = Some(map.next_value::<u32>()?);
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(SchemaVersionDiscriminatorV1(schema_version))
        }
    }

    fn ensure_store(
        transaction: &ConfigProjectionChildTransactionV1,
        authority_store_hint: Option<&str>,
    ) -> Result<ConfigProjectionStoreV1, ConfigProjectionFailureV1> {
        if let Some(store) = read_store(transaction)? {
            validate_store(&store, transaction)?;
            if authority_store_hint.is_some_and(|hint| hint != store.authority_store_id) {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            ensure_base_directories(transaction)?;
            return Ok(store);
        }
        let authority_store_id = authority_store_hint
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| prefixed_uuid("cpa_"));
        validate_prefixed_uuid(&authority_store_id, "cpa_")?;
        let accepted_home = accepted_home_from_authority_child(&transaction.root)?;
        let mut store = ConfigProjectionStoreV1 {
            schema_version: 1,
            authority_store_id,
            accepted_home,
            created_at: now_timestamp(),
            store_hash: String::new(),
        };
        store.store_hash = hash_omitting(
            "substrate.e3.config-projection-store.v1",
            "store",
            &store,
            "store_hash",
        )?;
        let bytes = ConfigProjectionCodecV1::encode_canonical_json(&store)?;
        write_immutable(
            transaction.root.as_fd(),
            "store.json",
            &bytes,
            "store",
            transaction.owner_uid,
        )?;
        ensure_base_directories(transaction)?;
        validate_store(&store, transaction)?;
        Ok(store)
    }

    fn ensure_base_directories(
        transaction: &ConfigProjectionChildTransactionV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        for name in [
            "inputs",
            "runtime-artifacts",
            "native-sources",
            "subjects",
            "series",
            "leases",
            "gateway-boundaries",
            "gateways",
            "gateway-intents",
            "gateway-launch-inputs",
            "handoffs",
            "kernel-effects",
            "child-cgroups",
            "child-processes",
            "terminal-child-evidence",
            "retirement",
        ] {
            open_or_create_directory(transaction.root.as_fd(), name, transaction.owner_uid)?;
        }
        let inputs = open_directory_at(transaction.root.as_fd(), "inputs")?;
        for name in ["effective-config", "agent-inventory"] {
            open_or_create_directory(inputs.as_fd(), name, transaction.owner_uid)?;
        }
        let kernel_effects = open_directory_at(transaction.root.as_fd(), "kernel-effects")?;
        for name in ["intents", "resolutions"] {
            open_or_create_directory(kernel_effects.as_fd(), name, transaction.owner_uid)?;
        }
        fsync(&kernel_effects)?;
        fsync(&inputs)?;
        fsync(&transaction.root)
    }

    fn read_store(
        transaction: &ConfigProjectionChildTransactionV1,
    ) -> Result<Option<ConfigProjectionStoreV1>, ConfigProjectionFailureV1> {
        read_optional_canonical_at(transaction.root.as_fd(), "store.json")
    }

    fn require_store(
        transaction: &ConfigProjectionChildTransactionV1,
    ) -> Result<ConfigProjectionStoreV1, ConfigProjectionFailureV1> {
        let store =
            read_store(transaction)?.ok_or(ConfigProjectionFailureV1::MissingPreparation)?;
        validate_store(&store, transaction)?;
        Ok(store)
    }

    fn publish_first(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        record: &AgentConfigProjectionRecordV1,
        reference: &ConfigProjectionRefV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let subject_hash = subject_hash(&record.identity)?;
        let subjects = open_directory_at(transaction.root.as_fd(), "subjects")?;
        let mut binding = ConfigProjectionSubjectBindingV1 {
            schema_version: 1,
            subject_hash: subject_hash.clone(),
            identity: record.identity.clone(),
            series_id: record.identity.series_id.clone(),
            created_at: record.created_at.clone(),
            binding_hash: String::new(),
        };
        binding.binding_hash = hash_omitting(
            "substrate.e3.config-projection-subject-binding.v1",
            "binding",
            &binding,
            "binding_hash",
        )?;
        validate_binding(&binding, store)?;
        let binding_bytes = ConfigProjectionCodecV1::encode_canonical_json(&binding)?;
        write_immutable(
            subjects.as_fd(),
            &format!("{subject_hash}.json"),
            &binding_bytes,
            "subject",
            transaction.owner_uid,
        )?;
        let series = open_series(transaction, &record.identity.series_id, true)?;
        let records = open_or_create_directory(series.as_fd(), "records", transaction.owner_uid)?;
        let record_bytes = ConfigProjectionCodecV1::encode_canonical_json(record)?;
        write_immutable(
            records.as_fd(),
            &record_name(record),
            &record_bytes,
            "record",
            transaction.owner_uid,
        )?;
        let mut head = ConfigProjectionHeadV1 {
            schema_version: 1,
            authority_store_id: store.authority_store_id.clone(),
            series_id: record.identity.series_id.clone(),
            head_ref: reference.clone(),
            head_revision: 1,
            predecessor_head_hash: None,
            updated_at: record.created_at.clone(),
            head_hash: String::new(),
        };
        head.head_hash = hash_omitting(
            "substrate.e3.config-projection-head.v1",
            "head",
            &head,
            "head_hash",
        )?;
        let head_bytes = ConfigProjectionCodecV1::encode_canonical_json(&head)?;
        cas_head(
            series.as_fd(),
            None,
            &head_bytes,
            "head",
            transaction.owner_uid,
        )
    }

    fn publish_successor(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        record: &AgentConfigProjectionRecordV1,
        expected: &ConfigProjectionRefV1,
        reference: &ConfigProjectionRefV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let series = open_series(transaction, &record.identity.series_id, false)?;
        let current_bytes = read_file_at(series.as_fd(), "head.json")?;
        let current: ConfigProjectionHeadV1 =
            ConfigProjectionCodecV1::decode_canonical_json(&current_bytes)?;
        validate_head(&current, store, &record.identity.series_id)?;
        let exact_retry = current.head_ref == *reference;
        if !exact_retry && current.head_ref != *expected {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        let predecessor = read_record(transaction, expected)?;
        validate_record(&predecessor, store)?;
        validate_gateway_ack_chain_v1(transaction, Some(&predecessor), None)?;
        if record_ref(&predecessor) != *expected || predecessor.identity != record.identity {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_record_transition(&predecessor, record)?;
        if record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Dormant {
            let intents = open_directory_at(transaction.root.as_fd(), "gateway-intents")?;
            let old: crate::ManagedGatewayActivationIntentV1 = read_canonical_at(
                intents.as_fd(),
                &format!(
                    "{}.json",
                    predecessor
                        .managed_gateway
                        .activation_intent_ref
                        .activation_intent_id
                ),
            )?;
            let fresh: crate::ManagedGatewayActivationIntentV1 = read_canonical_at(
                intents.as_fd(),
                &format!(
                    "{}.json",
                    record
                        .managed_gateway
                        .activation_intent_ref
                        .activation_intent_id
                ),
            )?;
            validate_gateway_intent(&old, store)?;
            validate_gateway_intent(&fresh, store)?;
            if old.activation_intent_id
                != predecessor
                    .managed_gateway
                    .activation_intent_ref
                    .activation_intent_id
                || old.intent_hash
                    != predecessor
                        .managed_gateway
                        .activation_intent_ref
                        .intent_hash
                || fresh.activation_intent_id
                    != record
                        .managed_gateway
                        .activation_intent_ref
                        .activation_intent_id
                || fresh.intent_hash != record.managed_gateway.activation_intent_ref.intent_hash
                || old.readiness_nonce == fresh.readiness_nonce
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        if exact_retry {
            if record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Dormant {
                validate_preparation_cleanup_in_transaction_v1(
                    transaction,
                    &predecessor,
                    Some(record),
                )?;
            }
            let existing = read_record(transaction, reference)?;
            validate_record(&existing, store)?;
            validate_gateway_ack_chain_v1(transaction, Some(&existing), None)?;
            if existing != *record {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            // Retry success requires the same immutable dependencies as the installed successor.
            validate_registry_tree(transaction, true)?;
            return Ok(());
        }
        let records = open_directory_at(series.as_fd(), "records")?;
        let record_bytes = ConfigProjectionCodecV1::encode_canonical_json(record)?;
        write_immutable(
            records.as_fd(),
            &record_name(record),
            &record_bytes,
            "record",
            transaction.owner_uid,
        )?;
        if record.managed_gateway.posture == ManagedGatewayProjectionPostureV1::Dormant {
            validate_preparation_cleanup_in_transaction_v1(
                transaction,
                &predecessor,
                Some(record),
            )?;
        }
        let mut head = ConfigProjectionHeadV1 {
            schema_version: 1,
            authority_store_id: store.authority_store_id.clone(),
            series_id: record.identity.series_id.clone(),
            head_ref: reference.clone(),
            head_revision: current
                .head_revision
                .checked_add(1)
                .ok_or(ConfigProjectionFailureV1::Conflict)?,
            predecessor_head_hash: Some(current.head_hash),
            updated_at: record.created_at.clone(),
            head_hash: String::new(),
        };
        head.head_hash = hash_omitting(
            "substrate.e3.config-projection-head.v1",
            "head",
            &head,
            "head_hash",
        )?;
        let head_bytes = ConfigProjectionCodecV1::encode_canonical_json(&head)?;
        cas_head(
            series.as_fd(),
            Some(&current_bytes),
            &head_bytes,
            "head",
            transaction.owner_uid,
        )
    }

    fn validate_record(
        record: &AgentConfigProjectionRecordV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if record.schema_version != 1
            || record.identity.schema_version != 1
            || record.identity.authority_store_id != store.authority_store_id
            || record.identity.accepted_home != store.accepted_home
            || record.identity.runtime_family != "codex"
            || record.identity.series_id.is_empty()
            || record.revision == 0
            || record.logical.requested_mcp_servers.len() + record.logical.requested_features.len()
                != 0
            || !record.effective.mcp_servers.is_empty()
            || !record.effective.features.is_empty()
            || record.effective.accepted_policy != record.identity.immutable_launch_cap
            || record.effective.capabilities != record.logical.capabilities
            || record.effective.logical_projection_hash != record.logical.projection_hash
            || record.effective.model != record.logical.requested_model
            || record.effective.model != "codex"
            || record.effective.provider.provider_id != "substrate-managed-gateway"
            || record.effective.provider.wire_api != "responses"
            || record.effective.provider.requires_openai_auth
            || record.effective.provider.supports_websockets
            || record.native.environment != record.effective.environment
            || record.logical.backend_id != record.identity.backend_id
            || record.logical.runtime_family != record.identity.runtime_family
            || record.logical.execution_scope != "world"
            || record.logical.cli_mode != "persistent"
            || record.logical.placement != "world"
            || (record.revision == 1) != record.predecessor_ref.is_none()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&record.identity.authority_store_id, "cpa_")?;
        validate_prefixed_uuid(&record.identity.series_id, "cps_")?;
        validate_prefixed_uuid(&record.record_id, "cpr_")?;
        validate_timestamp(&record.created_at)?;
        if let Some(predecessor) = &record.predecessor_ref {
            predecessor
                .validate()
                .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            if predecessor.authority_store_id != record.identity.authority_store_id
                || predecessor.series_id != record.identity.series_id
                || predecessor.revision.saturating_add(1) != record.revision
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        record_ref(record)
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        validate_hash_field(
            &record.identity.identity_hash,
            hash_omitting(
                "substrate.e3.config-projection-identity.v1",
                "identity",
                &record.identity,
                "identity_hash",
            )?,
        )?;
        validate_projection_hash(
            "substrate.e3.logical-config-projection.v1",
            &record.logical,
            &record.logical.projection_hash,
        )?;
        validate_projection_hash(
            "substrate.e3.effective-config-projection.v1",
            &record.effective,
            &record.effective.projection_hash,
        )?;
        validate_projection_hash(
            "substrate.e3.native-config-projection.v1",
            &record.native,
            &record.native.projection_hash,
        )?;
        validate_projection_hash(
            "substrate.e3.managed-gateway-projection.v1",
            &record.managed_gateway,
            &record.managed_gateway.projection_hash,
        )?;
        validate_projection_hash(
            "substrate.e3.nonsecret-handoff-projection.v1",
            &record.nonsecret_handoff,
            &record.nonsecret_handoff.projection_hash,
        )?;
        validate_hash_field(
            &record.record_hash,
            hash_omitting(
                "substrate.e3.agent-config-projection-record.v1",
                "record",
                record,
                "record_hash",
            )?,
        )?;
        validate_reference_bindings(record)?;
        validate_artifact(
            &record.identity.runtime_artifacts.codex,
            crate::ConfigProjectionArtifactRoleV1::Codex0125,
            &record.identity.authority_store_id,
        )?;
        validate_artifact(
            &record.identity.runtime_artifacts.world_entry_wrapper,
            crate::ConfigProjectionArtifactRoleV1::WorldEntryWrapper,
            &record.identity.authority_store_id,
        )?;
        validate_artifact(
            &record.identity.runtime_artifacts.managed_gateway,
            crate::ConfigProjectionArtifactRoleV1::ManagedGateway,
            &record.identity.authority_store_id,
        )?;
        validate_credential_source_ref(&record.nonsecret_handoff.credential_source_ref)?;
        Ok(())
    }

    fn validate_reference_bindings(
        record: &AgentConfigProjectionRecordV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        for authority_store_id in [
            &record
                .managed_gateway
                .activation_intent_ref
                .authority_store_id,
            &record
                .managed_gateway
                .expected_gateway_ref
                .authority_store_id,
            &record
                .managed_gateway
                .access_boundary_ref
                .authority_store_id,
            &record
                .nonsecret_handoff
                .secret_handoff_ref
                .authority_store_id,
            &record
                .nonsecret_handoff
                .receiving_gateway_ref
                .authority_store_id,
            &record.activation.activation_intent_ref.authority_store_id,
        ] {
            if authority_store_id != &record.identity.authority_store_id {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        record
            .managed_gateway
            .activation_intent_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        record
            .managed_gateway
            .expected_gateway_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        validate_boundary_ref(&record.managed_gateway.access_boundary_ref)?;
        validate_prefixed_uuid(
            &record.nonsecret_handoff.secret_handoff_ref.handoff_id,
            "gsh_",
        )?;
        validate_sha256(&record.nonsecret_handoff.secret_handoff_ref.handoff_hash)?;
        if record.managed_gateway.activation_intent_ref != record.activation.activation_intent_ref
            || record.managed_gateway.activation_intent_ref
                != record.effective.provider.gateway_intent_ref
            || record.managed_gateway.expected_gateway_ref
                != record.nonsecret_handoff.receiving_gateway_ref
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        match &record.nonsecret_handoff.delivery {
            SecretDeliveryMechanismV1::SecureFd {
                fd_name,
                one_time: true,
                gateway_receiver_only: true,
                deny_child_inheritance: true,
                close_after_consume: true,
            } if fd_name == "SUBSTRATE_LLM_AUTH_BUNDLE_FD" => {}
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        }
        match (
            &record.managed_gateway.posture,
            &record.activation.publication_fence,
        ) {
            (
                ManagedGatewayProjectionPostureV1::Dormant,
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id },
            ) if record.nonsecret_handoff.observed_state == SecretHandoffStateV1::Prepared
                && record.managed_gateway.activation_ack_ref.is_none()
                && record.nonsecret_handoff.activation_ack_ref.is_none()
                && record.activation.gateway_activation_ack_ref.is_none()
                && record.activation.released_at.is_none() =>
            {
                validate_prefixed_uuid(fence_id, "cpf_")?;
            }
            (
                ManagedGatewayProjectionPostureV1::ReadyClosed,
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id },
            ) if record.nonsecret_handoff.observed_state == SecretHandoffStateV1::Consumed
                && record.managed_gateway.activation_ack_ref.is_some()
                && record.managed_gateway.activation_ack_ref
                    == record.nonsecret_handoff.activation_ack_ref
                && record.managed_gateway.activation_ack_ref
                    == record.activation.gateway_activation_ack_ref
                && record.activation.released_at.is_none() =>
            {
                validate_prefixed_uuid(fence_id, "cpf_")?;
                validate_ack_ref(record.managed_gateway.activation_ack_ref.as_ref().unwrap())?;
            }
            (
                ManagedGatewayProjectionPostureV1::Active,
                crate::ConfigProjectionPublicationFenceV1::Released {
                    fence_id,
                    closed_record_ref,
                    activation_ack_ref,
                    release_hash,
                },
            ) if record.nonsecret_handoff.observed_state == SecretHandoffStateV1::Consumed
                && record.managed_gateway.activation_ack_ref.is_some()
                && record.managed_gateway.activation_ack_ref
                    == record.nonsecret_handoff.activation_ack_ref
                && record.managed_gateway.activation_ack_ref
                    == record.activation.gateway_activation_ack_ref
                && record.activation.released_at.is_some()
                && record.managed_gateway.activation_ack_ref.as_ref()
                    == Some(activation_ack_ref) =>
            {
                validate_prefixed_uuid(fence_id, "cpf_")?;
                validate_ack_ref(activation_ack_ref)?;
                validate_sha256(release_hash)?;
                validate_timestamp(record.activation.released_at.as_ref().unwrap())?;
                let expected_release_hash = ConfigProjectionCodecV1::domain_sha256(
                    "substrate.e3.config-projection-release.v1",
                    &serde_json::json!({
                        "activation_ack_ref": activation_ack_ref,
                        "closed_record_ref": closed_record_ref,
                        "fence_id": fence_id
                    }),
                )?;
                if release_hash != &expected_release_hash {
                    return Err(ConfigProjectionFailureV1::HashInvalid);
                }
            }
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        }
        Ok(())
    }

    fn validate_ack_ref(
        reference: &crate::ManagedGatewayActivationAckRefV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_prefixed_uuid(&reference.authority_store_id, "cpa_")?;
        validate_prefixed_uuid(&reference.activation_ack_id, "gaa_")?;
        validate_sha256(&reference.ack_hash)
    }

    fn validate_record_transition(
        predecessor: &AgentConfigProjectionRecordV1,
        successor: &AgentConfigProjectionRecordV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if predecessor.revision.checked_add(1) != Some(successor.revision)
            || successor.identity != predecessor.identity
            || successor.predecessor_ref.as_ref() != Some(&record_ref(predecessor))
        {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        match (
            predecessor.managed_gateway.posture,
            successor.managed_gateway.posture,
            &predecessor.activation.publication_fence,
            &successor.activation.publication_fence,
        ) {
            (
                ManagedGatewayProjectionPostureV1::Dormant,
                ManagedGatewayProjectionPostureV1::ReadyClosed,
                predecessor_fence @ crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                    ..
                },
                successor_fence @ crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                    ..
                },
            ) if predecessor_fence == successor_fence
                && predecessor.managed_gateway.activation_intent_ref
                    == successor.managed_gateway.activation_intent_ref
                && predecessor.managed_gateway.expected_gateway_ref
                    == successor.managed_gateway.expected_gateway_ref
                && predecessor.managed_gateway.access_boundary_ref
                    == successor.managed_gateway.access_boundary_ref =>
            {
                Ok(())
            }
            (
                ManagedGatewayProjectionPostureV1::ReadyClosed,
                ManagedGatewayProjectionPostureV1::Active,
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                    fence_id: closed_fence,
                },
                crate::ConfigProjectionPublicationFenceV1::Released {
                    fence_id,
                    closed_record_ref,
                    activation_ack_ref,
                    ..
                },
            ) if fence_id == closed_fence
                && closed_record_ref == &record_ref(predecessor)
                && predecessor.managed_gateway.activation_ack_ref.as_ref()
                    == Some(activation_ack_ref)
                && successor.managed_gateway.activation_ack_ref.as_ref()
                    == Some(activation_ack_ref)
                && predecessor.managed_gateway.expected_gateway_ref
                    == successor.managed_gateway.expected_gateway_ref
                && predecessor.managed_gateway.activation_intent_ref
                    == successor.managed_gateway.activation_intent_ref
                && successor
                    .managed_gateway
                    .access_boundary_ref
                    .access_boundary_id
                    == predecessor
                        .managed_gateway
                        .access_boundary_ref
                        .access_boundary_id
                && successor.managed_gateway.access_boundary_ref.revision
                    == predecessor
                        .managed_gateway
                        .access_boundary_ref
                        .revision
                        .saturating_add(1) =>
            {
                Ok(())
            }
            (
                ManagedGatewayProjectionPostureV1::Dormant
                | ManagedGatewayProjectionPostureV1::ReadyClosed
                | ManagedGatewayProjectionPostureV1::Active,
                ManagedGatewayProjectionPostureV1::Dormant,
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                    fence_id: old_fence,
                }
                | crate::ConfigProjectionPublicationFenceV1::Released {
                    fence_id: old_fence,
                    ..
                },
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id },
            ) if old_fence != fence_id
                && predecessor.native.root.root_id != successor.native.root.root_id
                && predecessor.native.root.authority_relative_path
                    != successor.native.root.authority_relative_path
                && predecessor.native.root.guest_absolute_path
                    != successor.native.root.guest_absolute_path
                && predecessor
                    .managed_gateway
                    .expected_gateway_ref
                    .gateway_instance_id
                    != successor
                        .managed_gateway
                        .expected_gateway_ref
                        .gateway_instance_id
                && predecessor
                    .managed_gateway
                    .access_boundary_ref
                    .access_boundary_id
                    != successor
                        .managed_gateway
                        .access_boundary_ref
                        .access_boundary_id
                && predecessor
                    .managed_gateway
                    .activation_intent_ref
                    .activation_intent_id
                    != successor
                        .managed_gateway
                        .activation_intent_ref
                        .activation_intent_id
                && predecessor.nonsecret_handoff.secret_handoff_ref.handoff_id
                    != successor.nonsecret_handoff.secret_handoff_ref.handoff_id
                && predecessor
                    .nonsecret_handoff
                    .credential_source_ref
                    .preparation_id
                    != successor
                        .nonsecret_handoff
                        .credential_source_ref
                        .preparation_id
                && predecessor
                    .nonsecret_handoff
                    .credential_source_ref
                    .credential_source_id
                    != successor
                        .nonsecret_handoff
                        .credential_source_ref
                        .credential_source_id =>
            {
                Ok(())
            }
            _ => Err(ConfigProjectionFailureV1::WrongBinding),
        }
    }

    fn publish_authoring_inputs(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        effective_config: &EffectiveSubstrateConfigSourceV1,
        agent_inventory: &AgentInventorySourceMaterialV1,
        artifact_manifest: &TrustedRuntimeArtifactManifestV1,
    ) -> Result<ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1> {
        let inputs = open_directory_at(transaction.root.as_fd(), "inputs")?;
        let effective_root = open_directory_at(inputs.as_fd(), "effective-config")?;
        let inventory_root = open_directory_at(inputs.as_fd(), "agent-inventory")?;
        let effective_bytes = ConfigProjectionCodecV1::encode_canonical_json(effective_config)?;
        write_immutable(
            effective_root.as_fd(),
            &format!("{}.json", effective_config.source_hash),
            &effective_bytes,
            "input",
            transaction.owner_uid,
        )?;
        let inventory_bytes = ConfigProjectionCodecV1::encode_canonical_json(agent_inventory)?;
        write_immutable(
            inventory_root.as_fd(),
            &format!("{}.json", agent_inventory.source_hash),
            &inventory_bytes,
            "input",
            transaction.owner_uid,
        )?;

        let artifacts = open_directory_at(transaction.root.as_fd(), "runtime-artifacts")?;
        let manifest_root = open_or_create_directory(
            artifacts.as_fd(),
            &artifact_manifest.manifest_id,
            transaction.owner_uid,
        )?;
        let manifest_bytes = ConfigProjectionCodecV1::encode_canonical_json(artifact_manifest)?;
        write_immutable(
            manifest_root.as_fd(),
            &format!("{:020}.json", artifact_manifest.revision),
            &manifest_bytes,
            "artifact-manifest",
            transaction.owner_uid,
        )?;

        let mut reference = ConfigProjectionAuthoringInputRefV1 {
            authority_store_id: store.authority_store_id.clone(),
            effective_config_source_hash: effective_config.source_hash.clone(),
            agent_inventory_source_hash: agent_inventory.source_hash.clone(),
            runtime_artifact_manifest_id: artifact_manifest.manifest_id.clone(),
            runtime_artifact_manifest_revision: artifact_manifest.revision,
            runtime_artifact_manifest_hash: artifact_manifest.manifest_hash.clone(),
            input_ref_hash: String::new(),
        };
        reference.input_ref_hash = reference
            .canonical_hash()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        reference
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;

        let effective_readback: EffectiveSubstrateConfigSourceV1 = read_canonical_at(
            effective_root.as_fd(),
            &format!("{}.json", effective_config.source_hash),
        )?;
        let inventory_readback: AgentInventorySourceMaterialV1 = read_canonical_at(
            inventory_root.as_fd(),
            &format!("{}.json", agent_inventory.source_hash),
        )?;
        let manifest_readback: TrustedRuntimeArtifactManifestV1 = read_canonical_at(
            manifest_root.as_fd(),
            &format!("{:020}.json", artifact_manifest.revision),
        )?;
        if effective_readback != *effective_config
            || inventory_readback != *agent_inventory
            || manifest_readback != *artifact_manifest
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(reference)
    }

    fn validate_effective_config_source(
        source: &EffectiveSubstrateConfigSourceV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if source.schema_version != 1
            || source.authority_store_id != store.authority_store_id
            || source.accepted_home != store.accepted_home
            || source.workspace_root.physical_path.is_empty()
            || source.values.managed_gateway_mode != "in_world"
            || source.values.default_execution_scope != "world"
            || source.values.default_cli_mode != "persistent"
            || source.values.default_backend_id != "cli:codex-world"
            || !source.values.llm_enabled
            || !source.values.agents_enabled
            || !source.values.world_enabled
            || !source.values.managed_gateway_enabled
        {
            return Err(ConfigProjectionFailureV1::UnsupportedConfiguration);
        }
        let expected_keys = [
            "llm.enabled",
            "llm.gateway.enabled",
            "llm.gateway.mode",
            "llm.routing.default_backend",
            "agents.enabled",
            "agents.defaults.execution.scope",
            "agents.defaults.cli.mode",
            "world.enabled",
        ];
        if source
            .ordered_explain_origins
            .iter()
            .map(|origin| origin.key.as_str())
            .ne(expected_keys)
        {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        for origin in &source.ordered_explain_origins {
            match origin.source_kind {
                E3ConfigExplainOriginKindV1::GlobalPatch => {
                    let location = origin
                        .source_location
                        .as_ref()
                        .ok_or(ConfigProjectionFailureV1::Malformed)?;
                    if location.source_root != source.accepted_home
                        || location.source_relative_path != "config.yaml"
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    validate_sha256(&location.source_bytes_sha256)?;
                }
                E3ConfigExplainOriginKindV1::WorkspacePatch => {
                    let location = origin
                        .source_location
                        .as_ref()
                        .ok_or(ConfigProjectionFailureV1::Malformed)?;
                    if location.source_root != source.workspace_root
                        || location.source_relative_path != ".substrate/workspace.yaml"
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                    validate_sha256(&location.source_bytes_sha256)?;
                }
                E3ConfigExplainOriginKindV1::CliFlag => {
                    return Err(ConfigProjectionFailureV1::Malformed)
                }
                E3ConfigExplainOriginKindV1::Default | E3ConfigExplainOriginKindV1::OverrideEnv => {
                    if origin.source_location.is_some() {
                        return Err(ConfigProjectionFailureV1::Malformed);
                    }
                }
            }
        }
        let mut revision_value =
            serde_json::to_value(source).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let revision_object = revision_value
            .as_object_mut()
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        revision_object.remove("source_revision");
        revision_object.remove("source_hash");
        let revision = ordinary_sha256(&ConfigProjectionCodecV1::encode_canonical_json(
            &revision_value,
        )?);
        if source.source_revision != format!("ecsr1_{revision}") {
            return Err(ConfigProjectionFailureV1::HashInvalid);
        }
        validate_hash_field(
            &source.source_hash,
            hash_omitting(
                "substrate.e3.effective-substrate-config-source.v1",
                "source",
                source,
                "source_hash",
            )?,
        )
    }

    fn validate_agent_inventory_source(
        source: &AgentInventorySourceMaterialV1,
        effective: &EffectiveSubstrateConfigSourceV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let expected_root = match source.inventory_scope.as_str() {
            "global" => &store.accepted_home,
            "workspace" => &effective.workspace_root,
            _ => return Err(ConfigProjectionFailureV1::Malformed),
        };
        if &source.accepted_root != expected_root
            || source.relative_path.is_empty()
            || source.relative_path.starts_with('/')
            || source
                .relative_path
                .split('/')
                .any(|part| part == ".." || part.is_empty())
            || !source.relative_path.ends_with(".yaml")
            || source.file_device_id == 0
            || source.file_inode == 0
            || source.byte_length == 0
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_sha256(&source.raw_bytes_sha256)?;
        if source.source_revision != format!("aisr1_{}", source.raw_bytes_sha256) {
            return Err(ConfigProjectionFailureV1::HashInvalid);
        }
        validate_hash_field(
            &source.source_hash,
            hash_omitting(
                "substrate.e3.agent-inventory-source.v1",
                "source",
                source,
                "source_hash",
            )?,
        )
    }

    fn validate_runtime_artifact_manifest(
        manifest: &TrustedRuntimeArtifactManifestV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if manifest.schema_version != 1
            || manifest.authority_store_id != store.authority_store_id
            || manifest.revision != 1
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&manifest.manifest_id, "ram_")?;
        validate_timestamp(&manifest.created_at)?;
        let expected_roles = [
            RuntimeArtifactAuthorityRoleV1::Codex0125,
            RuntimeArtifactAuthorityRoleV1::ManagedGateway,
            RuntimeArtifactAuthorityRoleV1::WorldEntryWrapper,
        ];
        if manifest
            .entries
            .iter()
            .map(|entry| entry.authority_role)
            .ne(expected_roles)
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        for (entry, expected_path) in manifest.entries.iter().zip([
            "/var/lib/substrate/world-deps/codex-runtime/bin/codex",
            "/usr/local/lib/substrate/e3/substrate-gateway",
            "/usr/local/lib/substrate/e3/substrate-world-entry",
        ]) {
            validate_prefixed_uuid(&entry.manifest_entry_id, "rae_")?;
            validate_prefixed_uuid(&entry.installer_source_ref.source_store_id, "ias_")?;
            validate_prefixed_uuid(&entry.installer_source_ref.source_record_id, "iar_")?;
            if entry.configured_absolute_path != expected_path
                || entry.device_id == 0
                || entry.inode == 0
                || entry.mode != 0o755
                || entry.owner_uid != 0
                || entry.byte_length == 0
                || entry.installer_source_ref.revision == 0
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            validate_sha256(&entry.sha256)?;
            validate_sha256(&entry.installer_source_ref.record_hash)?;
            validate_runtime_support(&entry.runtime_support)?;
            match (&entry.authority_role, &entry.provenance) {
                (
                    RuntimeArtifactAuthorityRoleV1::Codex0125,
                    RuntimeArtifactProvenanceV1::OfficialCodexRelease {
                        version,
                        target_triple,
                        archive_name,
                        archive_url,
                        archive_sha256,
                        archive_entry_path,
                        extracted_executable_sha256,
                    },
                ) if version == "0.125.0"
                    && target_triple == "x86_64-unknown-linux-musl"
                    && archive_name == "codex-x86_64-unknown-linux-musl.tar.gz"
                    && archive_url == "https://github.com/openai/codex/releases/download/rust-v0.125.0/codex-x86_64-unknown-linux-musl.tar.gz"
                    && archive_sha256 == "4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001"
                    && archive_entry_path == "codex-x86_64-unknown-linux-musl"
                    && extracted_executable_sha256 == "86dc42ac5823f25233d6dc4ec5ff34693afd8c32ff2d17b54c5eb0d15bc7d902"
                    && entry.sha256 == *extracted_executable_sha256 => {}
                (
                    RuntimeArtifactAuthorityRoleV1::ManagedGateway,
                    RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                        component,
                        source_commit,
                        source_tree,
                        cargo_lock_sha256,
                        target_triple,
                        profile,
                        executable_sha256,
                    },
                ) if component == "substrate-gateway"
                    && validate_git_object_id(source_commit).is_ok()
                    && validate_git_object_id(source_tree).is_ok()
                    && validate_sha256(cargo_lock_sha256).is_ok()
                    && target_triple == "x86_64-unknown-linux-musl"
                    && profile == "release"
                    && executable_sha256 == &entry.sha256 => {}
                (
                    RuntimeArtifactAuthorityRoleV1::WorldEntryWrapper,
                    RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                        component,
                        source_commit,
                        source_tree,
                        cargo_lock_sha256,
                        target_triple,
                        profile,
                        executable_sha256,
                    },
                ) if component == "substrate-world-entry"
                    && validate_git_object_id(source_commit).is_ok()
                    && validate_git_object_id(source_tree).is_ok()
                    && validate_sha256(cargo_lock_sha256).is_ok()
                    && target_triple == "x86_64-unknown-linux-musl"
                    && profile == "release"
                    && executable_sha256 == &entry.sha256 => {}
                _ => return Err(ConfigProjectionFailureV1::WrongBinding),
            }
            validate_hash_field(
                &entry.entry_hash,
                hash_omitting(
                    "substrate.e3.runtime-artifact-entry.v1",
                    "entry",
                    entry,
                    "entry_hash",
                )?,
            )?;
        }
        validate_hash_field(
            &manifest.manifest_hash,
            hash_omitting(
                "substrate.e3.runtime-artifact-manifest.v1",
                "manifest",
                manifest,
                "manifest_hash",
            )?,
        )
    }

    fn validate_native_projection_source_input(
        native: &NativeAgentConfigProjectionV1,
        series_id: &str,
        fence_id: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_prefixed_uuid(series_id, "cps_")?;
        validate_prefixed_uuid(fence_id, "cpf_")?;
        validate_prefixed_uuid(&native.root.root_id, "cnr_")?;
        if native.root.authority_relative_path
            != format!(
                "authority-v1/agent-config-projection-v1/native-sources/{series_id}/{fence_id}"
            )
            || native.root.guest_absolute_path
                != format!("/run/substrate/member-config/{series_id}/{fence_id}")
            || native.root.directory_mode != 0o700
            || native.directories
                != [
                    ".",
                    "system-empty",
                    "home",
                    "codex-home",
                    "state",
                    "state/sqlite",
                    "state/log",
                    "tmp",
                    "tmp/output-last-message",
                ]
                .into_iter()
                .map(|relative_path| crate::NativeProjectedDirectoryV1 {
                    relative_path: relative_path.to_string(),
                    mode: 0o700,
                })
                .collect::<Vec<_>>()
            || native.files.len() != 1
            || native.files[0].role != NativeProjectedFileRoleV1::CodexConfigToml
            || native.files[0].relative_path != "codex-home/config.toml"
            || native.files[0].mode != 0o600
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_sha256(&native.projection_hash)?;
        validate_sha256(&native.files[0].sha256)?;
        let bytes = STANDARD
            .decode(&native.files[0].bytes_base64)
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        if bytes.len() as u64 != native.files[0].byte_length
            || ordinary_sha256(&bytes) != native.files[0].sha256
        {
            return Err(ConfigProjectionFailureV1::HashInvalid);
        }
        let closure = &native.ambient_closure;
        let pinned_loader = &closure.loader_source;
        if pinned_loader.codex_version != "0.125.0"
            || pinned_loader.upstream_tag != "rust-v0.125.0"
            || pinned_loader.config_loader_source_sha256
                != "f8e2eff1db4d4cc004dff849682e81e224acca742b5d38d94e2db4a713b560bc"
            || pinned_loader.layer_io_source_sha256
                != "5b8aa76b0776370cc6db86b679efbb6bab361d2d474dc59768724f8a4c80a52a"
            || pinned_loader.loader_model_source_sha256
                != "8c6573bc83f53396a31bae92371790997c30521e46d29c0662c44d58d570ea81"
            || pinned_loader.exec_source_sha256
                != "b113fd23d8a0d264556b234fb067a4233ab8c3d5491dd393c0bc39b53c3170bd"
            || pinned_loader.cloud_requirements_source_sha256
                != "f43176f2889c54d19b65b0a669e98e22effc52712ca38710b9265ee22c804c77"
            || pinned_loader.auth_storage_source_sha256
                != "31c05505d2ed91f852a225e154929db3083ae61ef8a662fb6aad09442c33650c"
            || pinned_loader.config_types_source_sha256
                != "927c2a72b29136a5d8f627450a1f85a91173bc76e863351fd7233664b5ee32e4"
            || pinned_loader.validator_schema_version != 1
            || closure.allowed_enabled_layers != ["System", "User"]
            || closure.forbidden_cli_overrides
                != [
                    "--config",
                    "--ignore-rules",
                    "--ignore-user-config",
                    "--model",
                    "--oss",
                    "--profile",
                    "-c",
                ]
            || closure.inputs.len() != 16
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let input = |index: usize| -> Result<
            &crate::CodexLoaderInputAttestationV1,
            ConfigProjectionFailureV1,
        > {
            closure
                .inputs
                .get(index)
                .ok_or(ConfigProjectionFailureV1::WrongBinding)
        };
        let locator_matches = |input: &crate::CodexLoaderInputAttestationV1| {
            input.locator
                == format!(
                    "{}/{}",
                    input.directory.physical_path.trim_end_matches('/'),
                    input.relative_path
                )
        };
        let source_directory = |directory: &CanonicalDirectoryV1| {
            matches!(
                directory.physical_identity,
                DirectoryPhysicalIdentityV1::Linux {
                    device_id: 1..,
                    inode: 1..
                }
            ) && !directory.physical_path.is_empty()
        };
        let system_directory = input(0)?.directory.clone();
        let user_directory = input(5)?.directory.clone();
        for (index, relative_path) in [
            "config.toml",
            "managed_config.toml",
            "requirements.toml",
            "rules",
            "skills",
        ]
        .into_iter()
        .enumerate()
        {
            let entry = input(index)?;
            if entry.layer != "System"
                || entry.relative_path != relative_path
                || entry.disposition != crate::CodexLoaderInputDispositionV1::ProvenAbsent
                || entry.directory != system_directory
                || !locator_matches(entry)
                || entry.device_id.is_some()
                || entry.inode.is_some()
                || entry.byte_length.is_some()
                || entry.sha256.is_some()
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        if !source_directory(&system_directory) || !source_directory(&user_directory) {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        for (offset, relative_path) in [
            "config.toml",
            "managed_config.toml",
            "requirements.toml",
            "rules",
            "skills",
        ]
        .into_iter()
        .enumerate()
        {
            let entry = input(5 + offset)?;
            if entry.layer != "User"
                || entry.relative_path != relative_path
                || entry.directory != user_directory
                || !locator_matches(entry)
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            if offset == 0 {
                if entry.disposition != crate::CodexLoaderInputDispositionV1::Projected
                    || entry.device_id.is_none()
                    || entry.inode.is_none()
                    || entry.byte_length != Some(native.files[0].byte_length)
                    || entry.sha256.as_deref() != Some(native.files[0].sha256.as_str())
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            } else if entry.disposition != crate::CodexLoaderInputDispositionV1::ProvenAbsent
                || entry.device_id.is_some()
                || entry.inode.is_some()
                || entry.byte_length.is_some()
                || entry.sha256.is_some()
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        for (offset, relative_path) in [".codex/config.toml", ".codex/rules", ".codex/skills"]
            .into_iter()
            .enumerate()
        {
            let entry = input(10 + offset)?;
            if entry.layer != "Project"
                || entry.relative_path != relative_path
                || entry.disposition != crate::CodexLoaderInputDispositionV1::DisabledByTrust
                || entry.directory != native.cwd
                || !locator_matches(entry)
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        for (offset, (layer, relative_path, disposition)) in [
            ("McpCredentials", ".credentials.json", crate::CodexLoaderInputDispositionV1::DisabledByEmptyMcpSetAndProvenAbsent),
            ("Auth", "auth.json", crate::CodexLoaderInputDispositionV1::DisabledByEphemeralCredentialStoreAndProvenAbsent),
            ("CloudRequirements", "cloud-requirements-cache.json", crate::CodexLoaderInputDispositionV1::DisabledByNoEphemeralAuthAndProvenAbsent),
        ].into_iter().enumerate() {
            let entry = input(13 + offset)?;
            if entry.layer != layer || entry.relative_path != relative_path || entry.disposition != disposition
                || entry.directory != user_directory || !locator_matches(entry)
                || entry.device_id.is_some() || entry.inode.is_some() || entry.byte_length.is_some() || entry.sha256.is_some() {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
        }
        if ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.codex-0.125-loader-inputs.v1",
            &serde_json::json!({"inputs": &closure.inputs, "loader_source": &closure.loader_source}),
        )? != closure.validated_loader_input_fingerprint
        {
            return Err(ConfigProjectionFailureV1::HashInvalid);
        }
        let mut value =
            serde_json::to_value(native).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        value
            .as_object_mut()
            .ok_or(ConfigProjectionFailureV1::Malformed)?
            .remove("projection_hash");
        validate_hash_field(
            &native.projection_hash,
            ConfigProjectionCodecV1::domain_sha256(
                "",
                &serde_json::json!({
                    "domain": "substrate.e3.native-config-projection.v1",
                    "projection": value,
                }),
            )?,
        )
    }

    fn publish_native_source_directory(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        series_id: &str,
        fence_id: &str,
        plan: &Codex0125ProjectionPlanV1,
        created_at: Timestamp,
    ) -> Result<
        (
            NativeAgentConfigProjectionV1,
            NativeProjectionSourceManifestV1,
        ),
        ConfigProjectionFailureV1,
    > {
        let native_sources = open_directory_at(transaction.root.as_fd(), "native-sources")?;
        let series =
            open_or_create_directory(native_sources.as_fd(), series_id, transaction.owner_uid)?;
        let config_bytes = plan.config_bytes_v1();
        let expected_physical_path = format!(
            "{}/authority-v1/agent-config-projection-v1/native-sources/{series_id}/{fence_id}",
            store.accepted_home.physical_path
        );
        match open_directory_at(series.as_fd(), fence_id) {
            Ok(_) => {
                let existing = validate_native_source_directory(
                    &series,
                    fence_id,
                    None,
                    Some(config_bytes),
                    transaction.owner_uid,
                    transaction.native_source_gid,
                )?;
                let observation = capture_codex_native_source_observation_v1(
                    &series,
                    fence_id,
                    &expected_physical_path,
                    transaction.owner_uid,
                    transaction.native_source_gid,
                    config_bytes,
                )?;
                let native =
                    Codex0125ProjectionV1::finalize_from_native_source_v1(plan, &observation)?;
                validate_native_projection_source_input(&native, series_id, fence_id)?;
                if existing.authority_store_id == store.authority_store_id
                    && existing.series_id == series_id
                    && existing.fence_id == fence_id
                    && existing.source_root.physical_path == expected_physical_path
                    && existing.native_projection_hash == native.projection_hash
                    && existing.ordered_file_hashes == [native.files[0].sha256.clone()]
                    && existing.created_at == created_at
                {
                    return Ok((native, existing));
                }
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {}
            Err(error) => return Err(error),
        }
        let temp_name = format!(".e3-native-source-tmp.{}", Uuid::now_v7());
        let temp = open_or_create_directory_with_gid(
            series.as_fd(),
            &temp_name,
            transaction.owner_uid,
            Some(transaction.native_source_gid),
        )?;
        let codex_home = open_or_create_directory_with_gid(
            temp.as_fd(),
            "codex-home",
            transaction.owner_uid,
            Some(transaction.native_source_gid),
        )?;
        let system_empty = open_or_create_directory_with_gid(
            temp.as_fd(),
            "system-empty",
            transaction.owner_uid,
            Some(transaction.native_source_gid),
        )?;
        if !list_names(&system_empty)?.is_empty() {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        write_exclusive_file_with_gid(
            codex_home.as_fd(),
            "config.toml",
            config_bytes,
            transaction.owner_uid,
            Some(transaction.native_source_gid),
        )?;

        let observation = capture_codex_native_source_observation_v1(
            &series,
            &temp_name,
            &expected_physical_path,
            transaction.owner_uid,
            transaction.native_source_gid,
            config_bytes,
        )?;
        let native = Codex0125ProjectionV1::finalize_from_native_source_v1(plan, &observation)?;
        validate_native_projection_source_input(&native, series_id, fence_id)?;

        let temp_metadata = fstat(temp.as_raw_fd())?;
        if native.root.owner_uid != transaction.owner_uid as u64
            || native.root.owner_gid != temp_metadata.st_gid as u64
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let source_root = CanonicalDirectoryV1 {
            physical_path: expected_physical_path.clone(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: temp_metadata.st_dev,
                inode: temp_metadata.st_ino,
            },
        };
        let mut manifest = NativeProjectionSourceManifestV1 {
            schema_version: 1,
            authority_store_id: store.authority_store_id.clone(),
            series_id: series_id.to_string(),
            fence_id: fence_id.to_string(),
            source_root,
            native_projection_hash: native.projection_hash.clone(),
            ordered_file_hashes: vec![native.files[0].sha256.clone()],
            created_at,
            manifest_hash: String::new(),
        };
        manifest.manifest_hash = hash_omitting(
            "substrate.e3.native-projection-source-manifest.v1",
            "manifest",
            &manifest,
            "manifest_hash",
        )?;
        let manifest_bytes = ConfigProjectionCodecV1::encode_canonical_json(&manifest)?;
        write_exclusive_file_with_gid(
            temp.as_fd(),
            "source-manifest.json",
            &manifest_bytes,
            transaction.owner_uid,
            Some(transaction.native_source_gid),
        )?;
        fsync(&codex_home)?;
        fsync(&system_empty)?;
        fsync(&temp)?;
        match rename_noreplace(series.as_fd(), &temp_name, fence_id) {
            Ok(()) => {}
            Err(ConfigProjectionFailureV1::Conflict) => {
                let existing = validate_native_source_directory(
                    &series,
                    fence_id,
                    Some(&manifest),
                    Some(config_bytes),
                    transaction.owner_uid,
                    transaction.native_source_gid,
                )?;
                if existing != manifest {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                remove_native_source_temp(&series, &temp_name)?;
            }
            Err(error) => return Err(error),
        }
        fsync(&series)?;
        let readback = validate_native_source_directory(
            &series,
            fence_id,
            Some(&manifest),
            Some(config_bytes),
            transaction.owner_uid,
            transaction.native_source_gid,
        )?;
        if readback == manifest {
            let readback_observation = capture_codex_native_source_observation_v1(
                &series,
                fence_id,
                &expected_physical_path,
                transaction.owner_uid,
                transaction.native_source_gid,
                config_bytes,
            )?;
            let readback_native =
                Codex0125ProjectionV1::finalize_from_native_source_v1(plan, &readback_observation)?;
            if readback_native != native {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            Ok((native, manifest))
        } else {
            Err(ConfigProjectionFailureV1::PartialPublication)
        }
    }

    #[cfg(test)]
    fn write_exclusive_file(
        parent: BorrowedFd<'_>,
        name: &str,
        bytes: &[u8],
        owner_uid: libc::uid_t,
    ) -> Result<(), ConfigProjectionFailureV1> {
        write_exclusive_file_with_gid(parent, name, bytes, owner_uid, None)
    }

    fn write_exclusive_file_with_gid(
        parent: BorrowedFd<'_>,
        name: &str,
        bytes: &[u8],
        owner_uid: libc::uid_t,
        owner_gid: Option<libc::gid_t>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let mut file = open_file_at(
            parent,
            name,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        establish_created_owner_with_gid(parent, name, &file, owner_uid, false, owner_gid)?;
        #[cfg(test)]
        ownership_test_boundary("before_write", parent, name)?;
        verify_owned_entry_with_gid(parent, name, &file, owner_uid, false, owner_gid)?;
        file.write_all(bytes)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        #[cfg(test)]
        ownership_test_boundary("written", parent, name)?;
        fsync(&file)?;
        fsync_fd(parent.as_raw_fd())?;
        let written =
            verify_owned_entry_with_gid(parent, name, &file, owner_uid, false, owner_gid)?;
        if read_file_at(parent, name)? != bytes
            || !same_owned_metadata(
                &written,
                &verify_owned_entry_with_gid(parent, name, &file, owner_uid, false, owner_gid)?,
            )
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(())
    }

    fn verify_native_source_directories(
        series: &File,
        name: &str,
        root: &File,
        codex_home: &File,
        system_empty: &File,
        owner_uid: libc::uid_t,
        owner_gid: libc::gid_t,
    ) -> Result<(), ConfigProjectionFailureV1> {
        for (parent, name, held) in [
            (series.as_fd(), name, root),
            (root.as_fd(), "codex-home", codex_home),
            (root.as_fd(), "system-empty", system_empty),
        ] {
            verify_owned_entry_with_gid(parent, name, held, owner_uid, true, Some(owner_gid))?;
        }
        Ok(())
    }

    fn read_native_source_file(
        parent: BorrowedFd<'_>,
        name: &str,
        owner_uid: libc::uid_t,
        owner_gid: libc::gid_t,
    ) -> Result<(Vec<u8>, libc::stat), ConfigProjectionFailureV1> {
        let mut file = open_file_at(parent, name, libc::O_RDONLY, 0)?;
        let before =
            verify_owned_entry_with_gid(parent, name, &file, owner_uid, false, Some(owner_gid))?;
        if before.st_size < 0 || before.st_size as u64 > MAX_AUTHORITY_OBJECT_BYTES {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut bytes = Vec::with_capacity(before.st_size as usize);
        file.read_to_end(&mut bytes)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        let after =
            verify_owned_entry_with_gid(parent, name, &file, owner_uid, false, Some(owner_gid))?;
        if !same_owned_metadata(&before, &after) || bytes.len() as u64 != before.st_size as u64 {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok((bytes, after))
    }

    fn capture_codex_native_source_observation_v1(
        series: &File,
        name: &str,
        final_physical_path: &str,
        owner_uid: libc::uid_t,
        owner_gid: libc::gid_t,
        expected_config_bytes: &[u8],
    ) -> Result<Codex0125NativeSourceObservationV1, ConfigProjectionFailureV1> {
        let source_root = open_directory_at(series.as_fd(), name)?;
        let codex_home = open_directory_at(source_root.as_fd(), "codex-home")?;
        let system_empty = open_directory_at(source_root.as_fd(), "system-empty")?;
        verify_native_source_directories(
            series,
            name,
            &source_root,
            &codex_home,
            &system_empty,
            owner_uid,
            owner_gid,
        )?;
        if !list_names(&system_empty)?.is_empty() {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        let root_metadata = fstat(source_root.as_raw_fd())?;
        let codex_home_metadata = fstat(codex_home.as_raw_fd())?;
        let system_empty_metadata = fstat(system_empty.as_raw_fd())?;
        let (bytes, first) =
            read_native_source_file(codex_home.as_fd(), "config.toml", owner_uid, owner_gid)?;
        if bytes != expected_config_bytes {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        verify_native_source_directories(
            series,
            name,
            &source_root,
            &codex_home,
            &system_empty,
            owner_uid,
            owner_gid,
        )?;
        Ok(Codex0125NativeSourceObservationV1::new(
            CanonicalDirectoryV1 {
                physical_path: final_physical_path.to_string(),
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: root_metadata.st_dev,
                    inode: root_metadata.st_ino,
                },
            },
            CanonicalDirectoryV1 {
                physical_path: format!("{final_physical_path}/codex-home"),
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: codex_home_metadata.st_dev,
                    inode: codex_home_metadata.st_ino,
                },
            },
            CanonicalDirectoryV1 {
                physical_path: format!("{final_physical_path}/system-empty"),
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: system_empty_metadata.st_dev,
                    inode: system_empty_metadata.st_ino,
                },
            },
            first.st_dev,
            first.st_ino,
            first.st_mode & 0o7777,
            first.st_uid as u64,
            first.st_gid as u64,
            first.st_nlink,
            first.st_size as u64,
            ordinary_sha256(&bytes),
        ))
    }

    fn validate_native_source_directory(
        series: &File,
        name: &str,
        expected_manifest: Option<&NativeProjectionSourceManifestV1>,
        expected_config: Option<&[u8]>,
        owner_uid: libc::uid_t,
        owner_gid: libc::gid_t,
    ) -> Result<NativeProjectionSourceManifestV1, ConfigProjectionFailureV1> {
        let root = open_directory_at(series.as_fd(), name)?;
        if list_names(&root)? != ["codex-home", "source-manifest.json", "system-empty"] {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let codex_home = open_directory_at(root.as_fd(), "codex-home")?;
        let system_empty = open_directory_at(root.as_fd(), "system-empty")?;
        verify_native_source_directories(
            series,
            name,
            &root,
            &codex_home,
            &system_empty,
            owner_uid,
            owner_gid,
        )?;
        if list_names(&codex_home)? != ["config.toml"] || !list_names(&system_empty)?.is_empty() {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let (config, _) =
            read_native_source_file(codex_home.as_fd(), "config.toml", owner_uid, owner_gid)?;
        let (manifest_bytes, _) =
            read_native_source_file(root.as_fd(), "source-manifest.json", owner_uid, owner_gid)?;
        let manifest: NativeProjectionSourceManifestV1 =
            ConfigProjectionCodecV1::decode_canonical_json(&manifest_bytes)?;
        validate_prefixed_uuid(&manifest.series_id, "cps_")?;
        validate_prefixed_uuid(&manifest.fence_id, "cpf_")?;
        validate_timestamp(&manifest.created_at)?;
        validate_sha256(&manifest.native_projection_hash)?;
        if manifest.series_id != fstat_series_component(series, &manifest.series_id)?
            || (!name.starts_with(".e3-native-source-tmp.") && manifest.fence_id != name)
            || manifest.ordered_file_hashes != [ordinary_sha256(&config)]
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let metadata = fstat(root.as_raw_fd())?;
        let DirectoryPhysicalIdentityV1::Linux { device_id, inode } =
            &manifest.source_root.physical_identity;
        if *device_id != metadata.st_dev || *inode != metadata.st_ino {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_hash_field(
            &manifest.manifest_hash,
            hash_omitting(
                "substrate.e3.native-projection-source-manifest.v1",
                "manifest",
                &manifest,
                "manifest_hash",
            )?,
        )?;
        if expected_manifest.is_some_and(|expected| expected != &manifest)
            || expected_config.is_some_and(|expected| expected != config)
        {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        verify_native_source_directories(
            series,
            name,
            &root,
            &codex_home,
            &system_empty,
            owner_uid,
            owner_gid,
        )?;
        Ok(manifest)
    }

    fn fstat_series_component(
        series: &File,
        expected: &str,
    ) -> Result<String, ConfigProjectionFailureV1> {
        let link = std::fs::read_link(format!("/proc/self/fd/{}", series.as_raw_fd()))
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let component = link
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
        if component == expected {
            Ok(component.to_string())
        } else {
            Err(ConfigProjectionFailureV1::WrongBinding)
        }
    }

    fn remove_native_source_temp(
        series: &File,
        temp_name: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let temp = open_directory_at(series.as_fd(), temp_name)?;
        let codex_home = open_directory_at(temp.as_fd(), "codex-home")?;
        unlink_at(codex_home.as_fd(), "config.toml")?;
        unlink_directory_at(temp.as_fd(), "codex-home")?;
        unlink_directory_at(temp.as_fd(), "system-empty")?;
        unlink_at(temp.as_fd(), "source-manifest.json")?;
        unlink_directory_at(series.as_fd(), temp_name)?;
        fsync(series)
    }

    fn unlink_directory_at(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        if unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(())
    }

    fn recover_native_source_temps(
        transaction: &ConfigProjectionChildTransactionV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(native_sources) =
            open_optional_directory_at(transaction.root.as_fd(), "native-sources")?
        else {
            return Ok(());
        };
        for series_id in list_names(&native_sources)? {
            validate_prefixed_uuid(&series_id, "cps_")?;
            let series = open_directory_at(native_sources.as_fd(), &series_id)?;
            let temps = list_names(&series)?
                .into_iter()
                .filter(|name| name.starts_with(".e3-native-source-tmp."))
                .collect::<Vec<_>>();
            if temps.len() > 1 {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            let Some(temp_name) = temps.first() else {
                continue;
            };
            validate_native_temp_name(temp_name)?;
            let candidate = validate_native_source_directory(
                &series,
                temp_name,
                None,
                None,
                transaction.owner_uid,
                transaction.native_source_gid,
            )?;
            match open_directory_at(series.as_fd(), &candidate.fence_id) {
                Ok(_) => {
                    let final_manifest = validate_native_source_directory(
                        &series,
                        &candidate.fence_id,
                        Some(&candidate),
                        None,
                        transaction.owner_uid,
                        transaction.native_source_gid,
                    )?;
                    if final_manifest != candidate {
                        return Err(ConfigProjectionFailureV1::Conflict);
                    }
                    remove_native_source_temp(&series, temp_name)?;
                }
                Err(ConfigProjectionFailureV1::MissingPreparation) => {
                    rename_noreplace(series.as_fd(), temp_name, &candidate.fence_id)?;
                    fsync(&series)?;
                    validate_native_source_directory(
                        &series,
                        &candidate.fence_id,
                        Some(&candidate),
                        None,
                        transaction.owner_uid,
                        transaction.native_source_gid,
                    )?;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn validate_native_temp_name(name: &str) -> Result<(), ConfigProjectionFailureV1> {
        let raw = name
            .strip_prefix(".e3-native-source-tmp.")
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let parsed =
            Uuid::parse_str(raw).map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        if parsed.get_version_num() == 7 && parsed.to_string() == raw {
            Ok(())
        } else {
            Err(ConfigProjectionFailureV1::PartialPublication)
        }
    }

    fn validate_artifact(
        artifact: &crate::DescriptorPinnedArtifactV1,
        expected_role: crate::ConfigProjectionArtifactRoleV1,
        authority_store_id: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if artifact.role != expected_role
            || artifact.file_type != "regular"
            || artifact.configured_absolute_path.is_empty()
            || !Path::new(&artifact.configured_absolute_path).is_absolute()
            || artifact.device_id == 0
            || artifact.inode == 0
            || artifact.mode & 0o111 == 0
            || artifact.mode & 0o7022 != 0
            || artifact.byte_length == 0
            || artifact.authority_ref.authority_store_id != authority_store_id
            || artifact.authority_ref.manifest_revision == 0
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&artifact.authority_ref.manifest_id, "ram_")?;
        validate_prefixed_uuid(&artifact.authority_ref.manifest_entry_id, "rae_")?;
        validate_sha256(&artifact.sha256)?;
        validate_sha256(&artifact.authority_ref.manifest_hash)?;
        validate_sha256(&artifact.authority_ref.entry_hash)?;
        validate_runtime_support(&artifact.runtime_support)
    }

    fn validate_runtime_support(
        support: &crate::E3RuntimeSupportManifestV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if support.schema_version != 1
            || support.support_policy_version != 1
            || support.elf_interpreter.is_some()
            || support.dynamic_loader_cache.is_some()
            || !support.ordered_elf_dependencies.is_empty()
            || support
                .ordered_present_common_files
                .iter()
                .map(|file| file.absolute_path.as_str())
                .ne([
                    "/etc/hosts",
                    "/etc/nsswitch.conf",
                    "/etc/passwd",
                    "/etc/group",
                    "/etc/resolv.conf",
                    "/etc/ssl/certs/ca-certificates.crt",
                ])
            || support.ordered_present_common_files.iter().any(|file| {
                file.device_id == 0
                    || file.inode == 0
                    || file.mode & 0o022 != 0
                    || file.byte_length == 0
                    || validate_sha256(&file.sha256).is_err()
            })
            || support.system_config_mount_target.absolute_path != "/etc/codex"
            || support.system_config_mount_target.device_id == 0
            || support.system_config_mount_target.inode == 0
            || support.system_config_mount_target.mode != 0o755
            || support.system_config_mount_target.owner_uid != 0
            || support.system_config_mount_target.owner_gid != 0
            || !support
                .system_config_mount_target
                .ordered_entry_names
                .is_empty()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_hash_field(
            &support.manifest_hash,
            hash_omitting(
                "substrate.e3.runtime-support-manifest.v1",
                "manifest",
                support,
                "manifest_hash",
            )?,
        )
    }

    fn validate_credential_source_ref(
        reference: &crate::CredentialSourceRefV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let expected_fields = if reference.optional_account_id_present {
            vec![
                "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN".to_string(),
                "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID".to_string(),
            ]
        } else {
            vec!["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN".to_string()]
        };
        if reference.schema_version != 1
            || reference.credential_source_id.is_empty()
            || reference.preparation_id.is_empty()
            || reference.selected_backend_id != "cli:codex-world"
            || reference.bundle_backend_id != "cli:codex"
            || reference.ordered_field_names != expected_fields
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_timestamp(&reference.issued_at)?;
        validate_timestamp(&reference.expires_at)?;
        if reference.expires_at <= reference.issued_at {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        validate_hash_field(
            &reference.ref_hash,
            hash_omitting(
                "substrate.e3.credential-source-ref.v1",
                "credential_source",
                reference,
                "ref_hash",
            )?,
        )
    }

    fn validate_projection_hash<T: Serialize>(
        domain: &str,
        projection: &T,
        expected: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_hash_field(
            expected,
            hash_omitting(domain, "projection", projection, "projection_hash")?,
        )
    }

    fn validate_store(
        store: &ConfigProjectionStoreV1,
        transaction: &ConfigProjectionChildTransactionV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if store.schema_version != 1 {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        validate_prefixed_uuid(&store.authority_store_id, "cpa_")?;
        validate_timestamp(&store.created_at)?;
        let actual_home = accepted_home_from_authority_child(&transaction.root)?;
        if actual_home != store.accepted_home {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_hash_field(
            &store.store_hash,
            hash_omitting(
                "substrate.e3.config-projection-store.v1",
                "store",
                store,
                "store_hash",
            )?,
        )
    }

    fn validate_binding(
        binding: &ConfigProjectionSubjectBindingV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if binding.schema_version != 1
            || binding.identity.authority_store_id != store.authority_store_id
            || binding.identity.accepted_home != store.accepted_home
            || binding.series_id != binding.identity.series_id
            || binding.subject_hash != subject_hash(&binding.identity)?
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_timestamp(&binding.created_at)?;
        validate_hash_field(
            &binding.binding_hash,
            hash_omitting(
                "substrate.e3.config-projection-subject-binding.v1",
                "binding",
                binding,
                "binding_hash",
            )?,
        )
    }

    fn validate_head(
        head: &ConfigProjectionHeadV1,
        store: &ConfigProjectionStoreV1,
        series_id: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if head.schema_version != 1
            || head.authority_store_id != store.authority_store_id
            || head.series_id != series_id
            || head.head_ref.authority_store_id != store.authority_store_id
            || head.head_ref.series_id != series_id
            || head.head_revision == 0
            || head.head_revision != head.head_ref.revision
            || (head.head_revision == 1) != head.predecessor_head_hash.is_none()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        head.head_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        validate_timestamp(&head.updated_at)?;
        validate_hash_field(
            &head.head_hash,
            hash_omitting(
                "substrate.e3.config-projection-head.v1",
                "head",
                head,
                "head_hash",
            )?,
        )
    }

    fn validate_lease(
        lease: &ConfigProjectionConsumerLeaseV1,
        projection_ref: &ConfigProjectionRefV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if lease.schema_version != 1
            || lease.authority_store_id != projection_ref.authority_store_id
            || lease.series_id != projection_ref.series_id
            || lease.acquired_projection_ref != *projection_ref
            || lease.revision == 0
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&lease.consumer_id, "cpc_")?;
        validate_timestamp(&lease.acquired_at)?;
        match (lease.posture, &lease.released_at) {
            (ConfigProjectionConsumerLeasePostureV1::Held, None) => {}
            (ConfigProjectionConsumerLeasePostureV1::Released, Some(timestamp)) => {
                validate_timestamp(timestamp)?;
            }
            _ => return Err(ConfigProjectionFailureV1::Malformed),
        }
        validate_hash_field(
            &lease.lease_hash,
            hash_omitting(
                "substrate.e3.config-projection-consumer-lease.v1",
                "lease",
                lease,
                "lease_hash",
            )?,
        )
    }

    fn validate_retirement(
        retirement: &ConfigProjectionRetirementV1,
        store: &ConfigProjectionStoreV1,
        final_ref: &ConfigProjectionRefV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if retirement.schema_version != 1
            || retirement.authority_store_id != store.authority_store_id
            || retirement.series_id != final_ref.series_id
            || retirement.final_head_ref != *final_ref
            || retirement.consumer_lease_count != 0
            || retirement.terminal_child_evidence_ref.authority_store_id != store.authority_store_id
            || retirement.terminal_child_evidence_ref.series_id != retirement.series_id
            || retirement.revoked_boundary_ref.authority_store_id != store.authority_store_id
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_timestamp(&retirement.retired_at)?;
        validate_hash_field(
            &retirement.retirement_hash,
            hash_omitting(
                "substrate.e3.config-projection-retirement.v1",
                "retirement",
                retirement,
                "retirement_hash",
            )?,
        )
    }

    fn validate_boundary_ref(
        reference: &GatewayAccessBoundaryRefV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_prefixed_uuid(&reference.authority_store_id, "cpa_")?;
        validate_prefixed_uuid(&reference.access_boundary_id, "gab_")?;
        if reference.revision == 0 {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        validate_sha256(&reference.boundary_hash)
    }

    fn validate_cgroup_identity(
        cgroup: &crate::CanonicalCgroupIdentityV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if cgroup.cgroup_v2_mount_device_id == 0
            || cgroup.cgroup_v2_mount_inode == 0
            || cgroup.cgroup_directory_inode == 0
            || cgroup.cgroup_relative_path.is_empty()
            || Path::new(&cgroup.cgroup_relative_path).is_absolute()
            || Path::new(&cgroup.cgroup_relative_path)
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        Ok(())
    }

    fn kernel_effect_intent_ref(intent: &E3KernelEffectIntentV1) -> E3KernelEffectIntentRefV1 {
        E3KernelEffectIntentRefV1 {
            authority_store_id: intent.authority_store_id.clone(),
            effect_intent_id: intent.effect_intent_id.clone(),
            intent_hash: intent.intent_hash.clone(),
        }
    }

    fn validate_kernel_effect_intent(
        intent: &E3KernelEffectIntentV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if intent.schema_version != 1 || intent.authority_store_id != store.authority_store_id {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&intent.authority_store_id, "cpa_")?;
        validate_prefixed_uuid(&intent.series_id, "cps_")?;
        validate_prefixed_uuid(&intent.effect_intent_id, "eki_")?;
        validate_prefixed_uuid(&intent.preparation_id, "e3p_")?;
        validate_prefixed_uuid(&intent.fence_id, "cpf_")?;
        validate_timestamp(&intent.created_at)?;
        match &intent.effect {
            E3KernelEffectKindV1::CreateChildCgroup {
                cgroup_registration_id,
                parent_cgroup,
                child_component,
                expected_relative_path,
                ..
            } => {
                validate_prefixed_uuid(cgroup_registration_id, "ecg_")?;
                validate_cgroup_identity(parent_cgroup)?;
                let expected_component = format!(
                    "substrate-e3-{}",
                    &ordinary_sha256(cgroup_registration_id.as_bytes())[..24]
                );
                let expected_path = format!(
                    "{}/{}",
                    parent_cgroup.cgroup_relative_path, expected_component
                );
                if child_component != &expected_component
                    || expected_relative_path != &expected_path
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
            E3KernelEffectKindV1::InstallGatewayBoundary {
                access_boundary_id,
                network_namespace_inode,
                table_name,
                chain_name,
            } => {
                validate_prefixed_uuid(access_boundary_id, "gab_")?;
                let expected_table = format!(
                    "substrate_e3_{}",
                    &ordinary_sha256(access_boundary_id.as_bytes())[..24]
                );
                if *network_namespace_inode == 0
                    || table_name != &expected_table
                    || chain_name != "gateway_output"
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
        }
        validate_hash_field(
            &intent.intent_hash,
            hash_omitting(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                intent,
                "intent_hash",
            )?,
        )
    }

    fn resolve_kernel_effect_intent(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        reference: &E3KernelEffectIntentRefV1,
    ) -> Result<E3KernelEffectIntentV1, ConfigProjectionFailureV1> {
        if reference.authority_store_id != store.authority_store_id {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&reference.effect_intent_id, "eki_")?;
        validate_sha256(&reference.intent_hash)?;
        let kernel_effects = open_directory_at(transaction.root.as_fd(), "kernel-effects")?;
        let intents = open_directory_at(kernel_effects.as_fd(), "intents")?;
        let intent: E3KernelEffectIntentV1 = read_canonical_at(
            intents.as_fd(),
            &format!("{}.json", reference.effect_intent_id),
        )?;
        validate_kernel_effect_intent(&intent, store)?;
        if kernel_effect_intent_ref(&intent) != *reference {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(intent)
    }

    fn validate_kernel_effect_resolution(
        resolution: &E3KernelEffectResolutionV1,
        store: &ConfigProjectionStoreV1,
        intent: &E3KernelEffectIntentV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if resolution.schema_version != 1
            || resolution.authority_store_id != store.authority_store_id
            || resolution.effect_intent_ref != kernel_effect_intent_ref(intent)
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&resolution.resolution_id, "ekr_")?;
        validate_timestamp(&resolution.resolved_at)?;
        match (
            resolution.disposition,
            &intent.effect,
            &resolution.observed_cgroup,
            resolution.observed_nftables_table_handle,
        ) {
            (E3KernelEffectResolutionDispositionV1::NoEffectObserved, _, None, None) => {}
            (
                E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                E3KernelEffectKindV1::CreateChildCgroup {
                    parent_cgroup,
                    expected_relative_path,
                    ..
                },
                Some(cgroup),
                None,
            ) => {
                validate_cgroup_identity(cgroup)?;
                if cgroup.cgroup_v2_mount_device_id != parent_cgroup.cgroup_v2_mount_device_id
                    || cgroup.cgroup_v2_mount_inode != parent_cgroup.cgroup_v2_mount_inode
                    || cgroup.cgroup_relative_path != *expected_relative_path
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
            (
                E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                E3KernelEffectKindV1::InstallGatewayBoundary { .. },
                None,
                Some(handle),
            ) if handle != 0 => {}
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        }
        validate_hash_field(
            &resolution.resolution_hash,
            hash_omitting(
                "substrate.e3.kernel-effect-resolution.v1",
                "resolution",
                resolution,
                "resolution_hash",
            )?,
        )
    }

    fn ensure_intent_has_no_other_resolution(
        transaction: &ConfigProjectionChildTransactionV1,
        candidate: &E3KernelEffectResolutionV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let kernel_effects = open_directory_at(transaction.root.as_fd(), "kernel-effects")?;
        let resolutions = open_directory_at(kernel_effects.as_fd(), "resolutions")?;
        for name in list_names(&resolutions)? {
            if name.starts_with(".e3-tmp.") {
                continue;
            }
            let existing: E3KernelEffectResolutionV1 =
                read_canonical_at(resolutions.as_fd(), &name)?;
            if existing.effect_intent_ref == candidate.effect_intent_ref
                && existing.resolution_id != candidate.resolution_id
            {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
        }
        Ok(())
    }

    fn write_child_cgroup_registration(
        transaction: &ConfigProjectionChildTransactionV1,
        registration: &E3ChildCgroupRegistrationV1,
    ) -> Result<E3ChildCgroupRegistrationV1, ConfigProjectionFailureV1> {
        let root = open_directory_at(transaction.root.as_fd(), "child-cgroups")?;
        let series =
            open_or_create_directory(root.as_fd(), &registration.series_id, transaction.owner_uid)?;
        let bytes = ConfigProjectionCodecV1::encode_canonical_json(registration)?;
        let name = format!("{}.json", registration.cgroup_registration_id);
        write_immutable(
            series.as_fd(),
            &name,
            &bytes,
            "child-cgroup",
            transaction.owner_uid,
        )?;
        let readback: E3ChildCgroupRegistrationV1 = read_canonical_at(series.as_fd(), &name)?;
        if readback != *registration {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(readback)
    }

    fn validate_child_cgroup_registration(
        registration: &E3ChildCgroupRegistrationV1,
        store: &ConfigProjectionStoreV1,
        intent: &E3KernelEffectIntentV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let E3KernelEffectKindV1::CreateChildCgroup {
            cgroup_registration_id,
            role,
            parent_cgroup,
            expected_relative_path,
            ..
        } = &intent.effect
        else {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        };
        if registration.schema_version != 1
            || registration.authority_store_id != store.authority_store_id
            || registration.series_id != intent.series_id
            || registration.cgroup_registration_id != *cgroup_registration_id
            || registration.kernel_effect_intent_ref != kernel_effect_intent_ref(intent)
            || registration.fence_id != intent.fence_id
            || registration.role != *role
            || registration.cgroup.cgroup_relative_path != *expected_relative_path
            || registration.cgroup.cgroup_v2_mount_device_id
                != parent_cgroup.cgroup_v2_mount_device_id
            || registration.cgroup.cgroup_v2_mount_inode != parent_cgroup.cgroup_v2_mount_inode
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        if registration
            .turn_id
            .as_ref()
            .is_some_and(|value| value.is_empty())
        {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        validate_cgroup_identity(&registration.cgroup)?;
        validate_kernel_boot_id(&registration.kernel_boot_id)?;
        validate_timestamp(&registration.registered_at)?;
        validate_hash_field(
            &registration.cgroup_registration_hash,
            hash_omitting(
                "substrate.e3.child-cgroup-registration.v1",
                "cgroup_registration",
                registration,
                "cgroup_registration_hash",
            )?,
        )
    }

    fn resolve_child_cgroup_registration(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        series_id: &str,
        cgroup_registration_id: &str,
    ) -> Result<E3ChildCgroupRegistrationV1, ConfigProjectionFailureV1> {
        validate_prefixed_uuid(series_id, "cps_")?;
        validate_prefixed_uuid(cgroup_registration_id, "ecg_")?;
        let root = open_directory_at(transaction.root.as_fd(), "child-cgroups")?;
        let series = open_directory_at(root.as_fd(), series_id)?;
        let registration: E3ChildCgroupRegistrationV1 =
            read_canonical_at(series.as_fd(), &format!("{cgroup_registration_id}.json"))?;
        let intent = resolve_kernel_effect_intent(
            transaction,
            store,
            &registration.kernel_effect_intent_ref,
        )?;
        validate_child_cgroup_registration(&registration, store, &intent)?;
        if registration.series_id != series_id
            || registration.cgroup_registration_id != cgroup_registration_id
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(registration)
    }

    fn validate_child_process_registration(
        registration: &E3ChildProcessRegistrationV1,
        store: &ConfigProjectionStoreV1,
        cgroup: &E3ChildCgroupRegistrationV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if registration.schema_version != 1
            || registration.authority_store_id != store.authority_store_id
            || registration.series_id != cgroup.series_id
            || registration.cgroup_registration_id != cgroup.cgroup_registration_id
            || registration.cgroup_registration_hash != cgroup.cgroup_registration_hash
            || registration.fence_id != cgroup.fence_id
            || registration.role != cgroup.role
            || registration.process_cgroup != cgroup.cgroup
            || registration.kernel_boot_id != cgroup.kernel_boot_id
            || registration.pid == 0
            || registration.pid_start_time_ticks == 0
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&registration.registration_id, "ecp_")?;
        validate_prefixed_uuid(&registration.parent_service_instance_id, "wsi_")?;
        validate_timestamp(&registration.registered_at)?;
        validate_hash_field(
            &registration.registration_hash,
            hash_omitting(
                "substrate.e3.child-process-registration.v1",
                "registration",
                registration,
                "registration_hash",
            )?,
        )
    }

    fn resolve_child_process_registration(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        series_id: &str,
        registration_id: &str,
    ) -> Result<E3ChildProcessRegistrationV1, ConfigProjectionFailureV1> {
        validate_prefixed_uuid(series_id, "cps_")?;
        validate_prefixed_uuid(registration_id, "ecp_")?;
        let root = open_directory_at(transaction.root.as_fd(), "child-processes")?;
        let series = open_directory_at(root.as_fd(), series_id)?;
        let registration: E3ChildProcessRegistrationV1 =
            read_canonical_at(series.as_fd(), &format!("{registration_id}.json"))?;
        let cgroup = resolve_child_cgroup_registration(
            transaction,
            store,
            series_id,
            &registration.cgroup_registration_id,
        )?;
        validate_child_process_registration(&registration, store, &cgroup)?;
        if registration.series_id != series_id || registration.registration_id != registration_id {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(registration)
    }

    fn validate_kernel_boot_id(value: &str) -> Result<(), ConfigProjectionFailureV1> {
        let parsed = Uuid::parse_str(value).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        if parsed.to_string() != value {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        Ok(())
    }

    fn validate_boundary_object(
        boundary: &GatewayAccessBoundaryV1,
        store: &ConfigProjectionStoreV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if boundary.schema_version != 1
            || boundary.authority_store_id != store.authority_store_id
            || boundary.kernel_effect_intent_ref.authority_store_id != store.authority_store_id
            || boundary.revision == 0
            || boundary.gateway_listener.transport != "tcp"
            || boundary.gateway_listener.address != "127.0.0.1"
            || boundary.gateway_listener.port == 0
            || boundary.gateway_listener.socket_inode == 0
            || boundary.gateway_listener.listen_backlog != 16
            || !boundary
                .gateway_listener
                .deny_boundary_effective_before_listen
            || boundary.gateway_listener.responses_base_path != "/v1"
            || boundary.nftables_chain.family != "inet"
            || boundary.nftables_chain.chain != "gateway_output"
            || boundary.nftables_chain.chain_type != "filter"
            || boundary.nftables_chain.hook != "output"
            || boundary.nftables_chain.priority != -100
            || boundary.nftables_chain.policy != "accept"
            || boundary.nftables_chain.chain_handle == 0
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&boundary.access_boundary_id, "gab_")?;
        validate_prefixed_uuid(&boundary.gateway_instance_id, "cgi_")?;
        validate_prefixed_uuid(&boundary.kernel_effect_intent_ref.effect_intent_id, "eki_")?;
        validate_sha256(&boundary.kernel_effect_intent_ref.intent_hash)?;
        validate_sha256(&boundary.config_projection_identity_hash)?;
        validate_cgroup_identity(&boundary.readiness_probe_cgroup)?;
        validate_cgroup_identity(&boundary.allowed_member_cgroup)?;
        let expected_table = format!(
            "substrate_e3_{}",
            &ordinary_sha256(boundary.access_boundary_id.as_bytes())[..24]
        );
        if boundary.nftables_chain.table != expected_table {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let expected_roles: &[NftablesRuleRoleV1] = match boundary.posture {
            GatewayAccessPostureV1::DenyAllDormant => &[
                NftablesRuleRoleV1::ReadinessProbeAccept,
                NftablesRuleRoleV1::RejectRemainder,
            ],
            GatewayAccessPostureV1::AllowExactMember => &[
                NftablesRuleRoleV1::ReadinessProbeAccept,
                NftablesRuleRoleV1::ExactMemberAccept,
                NftablesRuleRoleV1::RejectRemainder,
            ],
            GatewayAccessPostureV1::Revoked => &[NftablesRuleRoleV1::RejectRemainder],
        };
        if boundary.nftables_rules.len() != expected_roles.len() {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        for (rule, expected_role) in boundary.nftables_rules.iter().zip(expected_roles) {
            if rule.role != *expected_role
                || rule.family != "inet"
                || rule.table != boundary.nftables_chain.table
                || rule.chain != boundary.nftables_chain.chain
                || rule.rule_handle == 0
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            ConfigProjectionCodecV1::decode_canonical_json::<serde_json::Value>(
                rule.canonical_expression.as_bytes(),
            )?;
        }
        match (
            &boundary.posture,
            boundary.revision,
            &boundary.predecessor_ref,
        ) {
            (GatewayAccessPostureV1::DenyAllDormant, 1, None) => {}
            (
                GatewayAccessPostureV1::AllowExactMember | GatewayAccessPostureV1::Revoked,
                n,
                Some(predecessor),
            ) if n > 1 && predecessor.revision + 1 == n => {
                validate_boundary_ref(predecessor)?;
                if predecessor.authority_store_id != boundary.authority_store_id
                    || predecessor.access_boundary_id != boundary.access_boundary_id
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
            _ => return Err(ConfigProjectionFailureV1::WrongBinding),
        }
        validate_hash_field(
            &boundary.boundary_hash,
            hash_omitting(
                "substrate.e3.gateway-access-boundary.v1",
                "boundary",
                boundary,
                "boundary_hash",
            )?,
        )
    }

    fn validate_boundary_transition(
        predecessor: &GatewayAccessBoundaryV1,
        successor: &GatewayAccessBoundaryV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if successor.predecessor_ref.as_ref()
            != Some(&GatewayAccessBoundaryRefV1 {
                authority_store_id: predecessor.authority_store_id.clone(),
                access_boundary_id: predecessor.access_boundary_id.clone(),
                revision: predecessor.revision,
                boundary_hash: predecessor.boundary_hash.clone(),
            })
            || predecessor.authority_store_id != successor.authority_store_id
            || predecessor.kernel_effect_intent_ref != successor.kernel_effect_intent_ref
            || predecessor.access_boundary_id != successor.access_boundary_id
            || predecessor.gateway_instance_id != successor.gateway_instance_id
            || predecessor.config_projection_identity_hash
                != successor.config_projection_identity_hash
            || predecessor.orchestration_session_id != successor.orchestration_session_id
            || predecessor.retained_participant_id != successor.retained_participant_id
            || predecessor.backend_id != successor.backend_id
            || predecessor.world_id != successor.world_id
            || predecessor.world_generation != successor.world_generation
            || predecessor.gateway_listener != successor.gateway_listener
            || predecessor.readiness_probe_cgroup != successor.readiness_probe_cgroup
            || predecessor.allowed_member_cgroup != successor.allowed_member_cgroup
            || predecessor.nftables_chain != successor.nftables_chain
            || !matches!(
                (predecessor.posture, successor.posture),
                (
                    GatewayAccessPostureV1::DenyAllDormant,
                    GatewayAccessPostureV1::AllowExactMember | GatewayAccessPostureV1::Revoked
                ) | (
                    GatewayAccessPostureV1::AllowExactMember,
                    GatewayAccessPostureV1::Revoked
                )
            )
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(())
    }

    fn read_boundary(
        transaction: &ConfigProjectionChildTransactionV1,
        reference: &GatewayAccessBoundaryRefV1,
    ) -> Result<GatewayAccessBoundaryV1, ConfigProjectionFailureV1> {
        validate_boundary_ref(reference)?;
        let store = require_store(transaction)?;
        let root = open_directory_at(transaction.root.as_fd(), "gateway-boundaries")?;
        let series = open_directory_at(root.as_fd(), &reference.access_boundary_id)?;
        let boundary: GatewayAccessBoundaryV1 =
            read_canonical_at(series.as_fd(), &format!("{:020}.json", reference.revision))?;
        validate_boundary_object(&boundary, &store)?;
        if boundary.authority_store_id != reference.authority_store_id
            || boundary.access_boundary_id != reference.access_boundary_id
            || boundary.revision != reference.revision
            || boundary.boundary_hash != reference.boundary_hash
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(boundary)
    }

    fn validate_revoked_boundary(
        transaction: &ConfigProjectionChildTransactionV1,
        retirement: &ConfigProjectionRetirementV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let boundary = read_boundary(transaction, &retirement.revoked_boundary_ref)?;
        if boundary.posture != GatewayAccessPostureV1::Revoked {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let predecessor_ref = boundary
            .predecessor_ref
            .as_ref()
            .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
        let predecessor = read_boundary(transaction, predecessor_ref)?;
        validate_boundary_transition(&predecessor, &boundary)?;
        let store = require_store(transaction)?;
        let record = read_record(transaction, &retirement.final_head_ref)?;
        validate_record(&record, &store)?;
        if predecessor_ref != &record.managed_gateway.access_boundary_ref
            || boundary.gateway_instance_id
                != record
                    .managed_gateway
                    .expected_gateway_ref
                    .gateway_instance_id
            || boundary.config_projection_identity_hash != record.identity.identity_hash
            || boundary.orchestration_session_id != record.identity.orchestration_session_id
            || boundary.retained_participant_id != record.identity.retained_participant_id
            || boundary.backend_id != record.identity.backend_id
            || boundary.world_id != record.identity.world_id
            || boundary.world_generation != record.identity.world_generation
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(())
    }

    fn validate_terminal_evidence_object(
        transaction: &ConfigProjectionChildTransactionV1,
        store: &ConfigProjectionStoreV1,
        evidence: &E3TerminalChildQuiescenceEvidenceV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if evidence.schema_version != 1
            || evidence.authority_store_id != store.authority_store_id
            || evidence.series_id != evidence.final_projection_ref.series_id
            || evidence.final_projection_ref.authority_store_id != store.authority_store_id
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_prefixed_uuid(&evidence.evidence_id, "tce_")?;
        evidence
            .final_projection_ref
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        validate_timestamp(&evidence.observed_at)?;
        let record = read_record(transaction, &evidence.final_projection_ref)?;
        validate_record(&record, store)?;
        if record_ref(&record) != evidence.final_projection_ref
            || record.identity.world_id != evidence.world_id
            || record.identity.world_generation != evidence.world_generation
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        // Attribute registrations through each retained record's immutable Prepared attempt.
        // The chain bounds revision order; it is not itself a registration or preparation join.
        let intents = open_directory_at(transaction.root.as_fd(), "gateway-intents")?;
        let series = open_series(transaction, &evidence.series_id, false)?;
        let records = open_directory_at(series.as_fd(), "records")?;
        let mut attempts = BTreeMap::new();
        let mut reference = read_series_head(transaction, &evidence.series_id)?.head_ref;
        let mut found_final = false;
        let mut complete_preparations = true;
        loop {
            let retained = read_record(transaction, &reference)?;
            validate_record(&retained, store)?;
            if record_ref(&retained) != reference || retained.identity != record.identity {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            found_final |= reference == evidence.final_projection_ref;
            let intent: Option<crate::ManagedGatewayActivationIntentV1> =
                read_optional_canonical_at(
                    intents.as_fd(),
                    &format!(
                        "{}.json",
                        retained
                            .managed_gateway
                            .activation_intent_ref
                            .activation_intent_id
                    ),
                )?;
            if let Some(intent) = intent {
                validate_gateway_intent(&intent, store)?;
                let prepared = read_handoff_revision(transaction, &intent.secret_handoff_ref)?;
                let dormant: AgentConfigProjectionRecordV1 = read_canonical_at(
                    records.as_fd(),
                    &format!(
                        "{:020}-{}.json",
                        intent.dormant_revision, intent.dormant_record_id
                    ),
                )?;
                validate_record(&dormant, store)?;
                let fence = match &retained.activation.publication_fence {
                    crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id }
                    | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => {
                        fence_id
                    }
                };
                if intent.intent_hash != retained.managed_gateway.activation_intent_ref.intent_hash
                    || intent.config_projection_identity_hash != retained.identity.identity_hash
                    || intent.fence_id != *fence
                    || intent.dormant_revision > retained.revision
                    || dormant.identity != retained.identity
                    || dormant.activation.publication_fence
                        != (crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                            fence_id: fence.clone(),
                        })
                    || dormant.managed_gateway.posture != ManagedGatewayProjectionPostureV1::Dormant
                    || dormant.nonsecret_handoff.secret_handoff_ref != handoff_reference(&prepared)?
                    || prepared.handoff.state != SecretHandoffStateV1::Prepared
                    || prepared.handoff.state_revision != 1
                    || prepared.predecessor_ref.is_some()
                    || prepared.handoff.credential_source_ref
                        != retained.nonsecret_handoff.credential_source_ref
                    || prepared.handoff.credential_source_ref.preparation_id
                        != intent.preparation_id
                    || prepared.handoff.receiving_gateway_ref != intent.expected_gateway_ref
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                let key = (fence.clone(), intent.preparation_id.clone());
                if attempts
                    .insert(key, intent.dormant_revision)
                    .is_some_and(|revision| revision != intent.dormant_revision)
                {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
            } else {
                // Earlier registry-only records retain the complete-series requirement. Production
                // preparation dependencies are independently mandatory in the preparation traversal.
                complete_preparations = false;
            }
            match &retained.predecessor_ref {
                Some(prior) if prior.revision.checked_add(1) == Some(retained.revision) => {
                    reference = prior.clone()
                }
                None if retained.revision == 1 => break,
                _ => return Err(ConfigProjectionFailureV1::WrongBinding),
            }
        }
        if !found_final {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let applies = |registration: &E3ChildCgroupRegistrationV1| -> Result<bool, ConfigProjectionFailureV1> {
            let intent = resolve_kernel_effect_intent(transaction, store, &registration.kernel_effect_intent_ref)?;
            validate_child_cgroup_registration(registration, store, &intent)?;
            Ok(!complete_preparations || attempts.get(&(intent.fence_id, intent.preparation_id))
                .is_some_and(|revision| *revision <= evidence.final_projection_ref.revision))
        };
        let mut prior_process_key = None;
        let mut observed_process_registrations = BTreeSet::new();
        for observation in &evidence.ordered_terminal_processes {
            validate_prefixed_uuid(&observation.registration_id, "ecp_")?;
            validate_sha256(&observation.registration_hash)?;
            if observation.pid == 0 || observation.pid_start_time_ticks == 0 {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            validate_terminal_observation_shape(observation)?;
            let registration = resolve_child_process_registration(
                transaction,
                store,
                &evidence.series_id,
                &observation.registration_id,
            )?;
            if observation.registration_hash != registration.registration_hash
                || observation.role != registration.role
                || observation.pid != registration.pid
                || observation.pid_start_time_ticks != registration.pid_start_time_ticks
                || !observed_process_registrations.insert(observation.registration_id.as_str())
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            if let crate::E3TerminalProcessObservationKindV1::RecoveryObservedTerminal {
                original_service_instance_id,
                recovery_service_instance_id,
                ..
            } = &observation.observation
            {
                if original_service_instance_id != &registration.parent_service_instance_id
                    || recovery_service_instance_id == original_service_instance_id
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
            let key = (
                terminal_role_rank(observation.role),
                observation.pid,
                observation.pid_start_time_ticks,
                observation.registration_id.as_str(),
            );
            if prior_process_key
                .as_ref()
                .is_some_and(|prior| prior >= &key)
            {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            prior_process_key = Some(key);
        }
        let process_root = open_directory_at(transaction.root.as_fd(), "child-processes")?;
        let mut durable_process_registrations = BTreeSet::new();
        for id in list_registration_ids_for_series(&process_root, &evidence.series_id)? {
            let process =
                resolve_child_process_registration(transaction, store, &evidence.series_id, &id)?;
            let cgroup = resolve_child_cgroup_registration(
                transaction,
                store,
                &evidence.series_id,
                &process.cgroup_registration_id,
            )?;
            if applies(&cgroup)? {
                durable_process_registrations.insert(id);
            }
        }
        if durable_process_registrations
            != observed_process_registrations
                .into_iter()
                .map(str::to_owned)
                .collect()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let mut prior_cgroup_key = None;
        let mut observed_cgroup_registrations = BTreeSet::new();
        for entry in &evidence.ordered_empty_cgroups {
            validate_prefixed_uuid(&entry.cgroup_registration_id, "ecg_")?;
            validate_sha256(&entry.cgroup_registration_hash)?;
            validate_sha256(&entry.cgroup_events_sha256)?;
            validate_sha256(&entry.cgroup_procs_sha256)?;
            validate_cgroup_identity(&entry.cgroup)?;
            if entry.populated || !entry.ordered_live_pids.is_empty() {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let registration = resolve_child_cgroup_registration(
                transaction,
                store,
                &evidence.series_id,
                &entry.cgroup_registration_id,
            )?;
            if entry.cgroup_registration_hash != registration.cgroup_registration_hash
                || entry.role != registration.role
                || entry.cgroup != registration.cgroup
                || !observed_cgroup_registrations.insert(entry.cgroup_registration_id.as_str())
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            let key = (
                terminal_role_rank(entry.role),
                entry.cgroup.cgroup_v2_mount_device_id,
                entry.cgroup.cgroup_v2_mount_inode,
                entry.cgroup.cgroup_directory_inode,
                entry.cgroup.cgroup_relative_path.as_str(),
                entry.cgroup_registration_id.as_str(),
            );
            if prior_cgroup_key.as_ref().is_some_and(|prior| prior >= &key) {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            prior_cgroup_key = Some(key);
        }
        let cgroup_root = open_directory_at(transaction.root.as_fd(), "child-cgroups")?;
        let mut durable_cgroup_registrations = BTreeSet::new();
        for id in list_registration_ids_for_series(&cgroup_root, &evidence.series_id)? {
            let cgroup =
                resolve_child_cgroup_registration(transaction, store, &evidence.series_id, &id)?;
            if applies(&cgroup)? {
                durable_cgroup_registrations.insert(id);
            }
        }
        if durable_cgroup_registrations
            != observed_cgroup_registrations
                .into_iter()
                .map(str::to_owned)
                .collect()
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_hash_field(
            &evidence.evidence_hash,
            hash_omitting(
                "substrate.e3.terminal-child-quiescence.v1",
                "evidence",
                evidence,
                "evidence_hash",
            )?,
        )
    }

    fn list_registration_ids_for_series(
        root: &File,
        series_id: &str,
    ) -> Result<BTreeSet<String>, ConfigProjectionFailureV1> {
        let series = match open_directory_at(root.as_fd(), series_id) {
            Ok(series) => series,
            Err(ConfigProjectionFailureV1::MissingPreparation) => return Ok(BTreeSet::new()),
            Err(error) => return Err(error),
        };
        list_names(&series)?
            .into_iter()
            .filter(|name| !name.starts_with(".e3-tmp."))
            .map(|name| {
                name.strip_suffix(".json")
                    .map(str::to_owned)
                    .ok_or(ConfigProjectionFailureV1::Malformed)
            })
            .collect()
    }

    fn terminal_role_rank(role: crate::E3TerminalProcessRoleV1) -> u8 {
        match role {
            crate::E3TerminalProcessRoleV1::ManagedGateway => 0,
            crate::E3TerminalProcessRoleV1::Codex => 1,
            crate::E3TerminalProcessRoleV1::ReadinessProbe => 2,
        }
    }

    fn validate_terminal_observation_shape(
        observation: &crate::E3TerminalProcessObservationV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        use crate::{E3RecoveryProcIdentityV1, E3TerminalProcessObservationKindV1};

        match &observation.observation {
            E3TerminalProcessObservationKindV1::ParentWaitid { .. } => Ok(()),
            E3TerminalProcessObservationKindV1::RecoveryObservedTerminal {
                original_service_instance_id,
                recovery_service_instance_id,
                pidfd_open_errno,
                pidfd_kill_errno,
                pidfd_became_readable,
                waitid_errno,
                proc_identity,
            } => {
                validate_prefixed_uuid(original_service_instance_id, "wsi_")?;
                validate_prefixed_uuid(recovery_service_instance_id, "wsi_")?;
                let absent_or_reused = match proc_identity {
                    E3RecoveryProcIdentityV1::Absent => true,
                    E3RecoveryProcIdentityV1::PidReused {
                        observed_pid_start_time_ticks,
                    } => {
                        *observed_pid_start_time_ticks != 0
                            && *observed_pid_start_time_ticks != observation.pid_start_time_ticks
                    }
                };
                let shape = match pidfd_open_errno {
                    Some(errno) => {
                        *errno == libc::ESRCH
                            && pidfd_kill_errno.is_none()
                            && pidfd_became_readable.is_none()
                            && waitid_errno.is_none()
                    }
                    None if pidfd_became_readable == &Some(true) => {
                        pidfd_kill_errno.is_none_or(|errno| errno == libc::ESRCH)
                            && *waitid_errno == Some(libc::ECHILD)
                    }
                    None => {
                        pidfd_kill_errno.is_none()
                            && pidfd_became_readable.is_none()
                            && waitid_errno.is_none()
                            && matches!(proc_identity, E3RecoveryProcIdentityV1::PidReused { .. })
                    }
                };
                if absent_or_reused && shape {
                    Ok(())
                } else {
                    Err(ConfigProjectionFailureV1::Malformed)
                }
            }
        }
    }

    fn validate_terminal_evidence(
        transaction: &ConfigProjectionChildTransactionV1,
        retirement: &ConfigProjectionRetirementV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let root = open_directory_at(transaction.root.as_fd(), "terminal-child-evidence")?;
        let series = open_directory_at(root.as_fd(), &retirement.series_id)?;
        let name = format!(
            "{}.json",
            retirement.terminal_child_evidence_ref.evidence_id
        );
        let evidence: E3TerminalChildQuiescenceEvidenceV1 =
            read_canonical_at(series.as_fd(), &name)?;
        if evidence.authority_store_id != retirement.authority_store_id
            || evidence.series_id != retirement.series_id
            || evidence.evidence_id != retirement.terminal_child_evidence_ref.evidence_id
            || evidence.evidence_hash != retirement.terminal_child_evidence_ref.evidence_hash
            || evidence.final_projection_ref != retirement.final_head_ref
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let store = require_store(transaction)?;
        validate_terminal_evidence_object(transaction, &store, &evidence)
    }

    fn ensure_all_leases_released(
        transaction: &ConfigProjectionChildTransactionV1,
        series_id: &str,
        fence_id: Option<&str>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let leases = open_directory_at(transaction.root.as_fd(), "leases")?;
        let Some(series) = open_optional_directory_at(leases.as_fd(), series_id)? else {
            return Ok(());
        };
        for consumer_id in list_names(&series)? {
            validate_prefixed_uuid(&consumer_id, "cpc_")?;
            let consumer = open_directory_at(series.as_fd(), &consumer_id)?;
            let lease: ConfigProjectionConsumerLeaseV1 =
                read_canonical_at(consumer.as_fd(), "head.json")?;
            validate_lease(&lease, &lease.acquired_projection_ref)?;
            let acquired = read_record(transaction, &lease.acquired_projection_ref)?;
            validate_record(&acquired, &require_store(transaction)?)?;
            let acquired_fence = match &acquired.activation.publication_fence {
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id }
                | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => fence_id,
            };
            if record_ref(&acquired) != lease.acquired_projection_ref
                || lease.consumer_id != consumer_id
                || lease.series_id != series_id
            {
                return Err(ConfigProjectionFailureV1::WrongBinding);
            }
            if fence_id.is_none_or(|fence| fence == acquired_fence)
                && lease.posture != ConfigProjectionConsumerLeasePostureV1::Released
            {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
        }
        Ok(())
    }

    fn read_retirement(
        transaction: &ConfigProjectionChildTransactionV1,
        series_id: &str,
    ) -> Result<Option<ConfigProjectionRetirementV1>, ConfigProjectionFailureV1> {
        let retirement = open_directory_at(transaction.root.as_fd(), "retirement")?;
        read_optional_canonical_at(retirement.as_fd(), &format!("{series_id}.json"))
    }

    fn read_series_head(
        transaction: &ConfigProjectionChildTransactionV1,
        series_id: &str,
    ) -> Result<ConfigProjectionHeadV1, ConfigProjectionFailureV1> {
        let store = require_store(transaction)?;
        let series = open_series(transaction, series_id, false)?;
        let head: ConfigProjectionHeadV1 = read_canonical_at(series.as_fd(), "head.json")?;
        validate_head(&head, &store, series_id)?;
        Ok(head)
    }

    fn read_record(
        transaction: &ConfigProjectionChildTransactionV1,
        reference: &ConfigProjectionRefV1,
    ) -> Result<AgentConfigProjectionRecordV1, ConfigProjectionFailureV1> {
        reference
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let series = open_series(transaction, &reference.series_id, false)?;
        let records = open_directory_at(series.as_fd(), "records")?;
        read_canonical_at(
            records.as_fd(),
            &format!("{:020}-{}.json", reference.revision, reference.record_id),
        )
    }

    fn open_series(
        transaction: &ConfigProjectionChildTransactionV1,
        series_id: &str,
        create: bool,
    ) -> Result<File, ConfigProjectionFailureV1> {
        validate_prefixed_uuid(series_id, "cps_")?;
        let series = open_directory_at(transaction.root.as_fd(), "series")?;
        if create {
            open_or_create_directory(series.as_fd(), series_id, transaction.owner_uid)
        } else {
            open_directory_at(series.as_fd(), series_id)
        }
    }

    pub(crate) fn prepared_member_dispatch_consumer_id_v1(
        preparation_id: &str,
    ) -> Result<String, ConfigProjectionFailureV1> {
        validate_prefixed_uuid(preparation_id, "e3p_")?;
        Ok(format!("cpc_{}", &preparation_id[4..]))
    }

    fn acquire_or_resolve_prepared_consumer_lease_v1(
        transaction: &ConfigProjectionChildTransactionV1,
        projection_ref: &ConfigProjectionRefV1,
        kind: ConfigProjectionConsumerKindV1,
        acquired_at: Timestamp,
        consumer_id: &str,
    ) -> Result<ConfigProjectionConsumerLeaseV1, ConfigProjectionFailureV1> {
        if kind != ConfigProjectionConsumerKindV1::MemberDispatchV2 {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let store = require_store(transaction)?;
        let record = read_record(transaction, projection_ref)?;
        validate_record(&record, &store)?;
        let handoff =
            read_handoff_revision(transaction, &record.nonsecret_handoff.secret_handoff_ref)?;
        if record.managed_gateway.posture != ManagedGatewayProjectionPostureV1::Dormant
            || handoff.handoff.state != SecretHandoffStateV1::Prepared
            || handoff.handoff.state_revision != 1
            || handoff.handoff.credential_source_ref
                != record.nonsecret_handoff.credential_source_ref
            || prepared_member_dispatch_consumer_id_v1(
                &handoff.handoff.credential_source_ref.preparation_id,
            )? != consumer_id
            || acquired_at != record.created_at
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let mut expected = ConfigProjectionConsumerLeaseV1 {
            schema_version: 1,
            authority_store_id: store.authority_store_id,
            series_id: projection_ref.series_id.clone(),
            consumer_id: consumer_id.to_owned(),
            consumer_kind: kind,
            revision: 1,
            predecessor_lease_hash: None,
            acquired_projection_ref: projection_ref.clone(),
            posture: ConfigProjectionConsumerLeasePostureV1::Held,
            acquired_at,
            released_at: None,
            lease_hash: String::new(),
        };
        expected.lease_hash = hash_omitting(
            "substrate.e3.config-projection-consumer-lease.v1",
            "lease",
            &expected,
            "lease_hash",
        )?;
        validate_lease(&expected, projection_ref)?;
        let leases = open_directory_at(transaction.root.as_fd(), "leases")?;
        let series = open_or_create_directory(
            leases.as_fd(),
            &projection_ref.series_id,
            transaction.owner_uid,
        )?;
        if open_optional_directory_at(series.as_fd(), consumer_id)?.is_some() {
            // An existing occupant is never replaced, even when Released or incomplete.
            return resolve_held_lease(transaction, &expected, projection_ref);
        }
        let consumer =
            open_or_create_directory(series.as_fd(), consumer_id, transaction.owner_uid)?;
        let revisions =
            open_or_create_directory(consumer.as_fd(), "revisions", transaction.owner_uid)?;
        let bytes = ConfigProjectionCodecV1::encode_canonical_json(&expected)?;
        write_immutable(
            revisions.as_fd(),
            "00000000000000000001.json",
            &bytes,
            "lease",
            transaction.owner_uid,
        )?;
        cas_head(
            consumer.as_fd(),
            None,
            &bytes,
            "head",
            transaction.owner_uid,
        )?;
        resolve_held_lease(transaction, &expected, projection_ref)
    }

    fn open_lease(
        transaction: &ConfigProjectionChildTransactionV1,
        series_id: &str,
        consumer_id: &str,
        create: bool,
    ) -> Result<File, ConfigProjectionFailureV1> {
        validate_prefixed_uuid(series_id, "cps_")?;
        validate_prefixed_uuid(consumer_id, "cpc_")?;
        let leases = open_directory_at(transaction.root.as_fd(), "leases")?;
        let series = if create {
            open_or_create_directory(leases.as_fd(), series_id, transaction.owner_uid)?
        } else {
            open_directory_at(leases.as_fd(), series_id)?
        };
        if create {
            open_or_create_directory(series.as_fd(), consumer_id, transaction.owner_uid)
        } else {
            open_directory_at(series.as_fd(), consumer_id)
        }
    }

    fn resolve_held_lease(
        transaction: &ConfigProjectionChildTransactionV1,
        held: &ConfigProjectionConsumerLeaseV1,
        projection_ref: &ConfigProjectionRefV1,
    ) -> Result<ConfigProjectionConsumerLeaseV1, ConfigProjectionFailureV1> {
        validate_lease(held, projection_ref)?;
        if held.revision != 1
            || held.predecessor_lease_hash.is_some()
            || held.posture != ConfigProjectionConsumerLeasePostureV1::Held
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let consumer = open_lease(transaction, &held.series_id, &held.consumer_id, false)?;
        let current: ConfigProjectionConsumerLeaseV1 =
            read_canonical_at(consumer.as_fd(), "head.json")?;
        validate_lease(&current, projection_ref)?;
        if current != *held {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        let revisions = open_directory_at(consumer.as_fd(), "revisions")?;
        let immutable: ConfigProjectionConsumerLeaseV1 =
            read_canonical_at(revisions.as_fd(), &format!("{:020}.json", held.revision))?;
        if immutable != *held {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        Ok(immutable)
    }

    fn record_ref(record: &AgentConfigProjectionRecordV1) -> ConfigProjectionRefV1 {
        ConfigProjectionRefV1 {
            authority_store_id: record.identity.authority_store_id.clone(),
            series_id: record.identity.series_id.clone(),
            record_id: record.record_id.clone(),
            revision: record.revision,
            record_hash: record.record_hash.clone(),
        }
    }

    fn record_name(record: &AgentConfigProjectionRecordV1) -> String {
        format!("{:020}-{}.json", record.revision, record.record_id)
    }

    fn subject_hash(
        identity: &ConfigProjectionIdentityV1,
    ) -> Result<String, ConfigProjectionFailureV1> {
        let identity = identity_without_series_hash(identity)?;
        let mut payload = BTreeMap::new();
        payload.insert("identity", identity);
        ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.config-projection-subject.v1",
            &payload,
        )
    }

    fn identity_without_series_hash(
        identity: &ConfigProjectionIdentityV1,
    ) -> Result<serde_json::Value, ConfigProjectionFailureV1> {
        let mut value =
            serde_json::to_value(identity).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let object = value
            .as_object_mut()
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        object.remove("series_id");
        object.remove("identity_hash");
        Ok(value)
    }

    fn hash_omitting<T: Serialize>(
        domain: &str,
        member: &str,
        value: &T,
        omitted: &str,
    ) -> Result<String, ConfigProjectionFailureV1> {
        let mut value =
            serde_json::to_value(value).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        value
            .as_object_mut()
            .ok_or(ConfigProjectionFailureV1::Malformed)?
            .remove(omitted)
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        let mut payload = BTreeMap::new();
        payload.insert(member.to_string(), value);
        ConfigProjectionCodecV1::domain_sha256(domain, &payload)
    }

    fn validate_hash_field(
        stored: &str,
        computed: String,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_sha256(stored)?;
        if stored == computed {
            Ok(())
        } else {
            Err(ConfigProjectionFailureV1::HashInvalid)
        }
    }

    fn validate_sha256(value: &str) -> Result<(), ConfigProjectionFailureV1> {
        if value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(())
        } else {
            Err(ConfigProjectionFailureV1::Malformed)
        }
    }

    fn validate_git_object_id(value: &str) -> Result<(), ConfigProjectionFailureV1> {
        if matches!(value.len(), 40 | 64)
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(())
        } else {
            Err(ConfigProjectionFailureV1::Malformed)
        }
    }

    fn ordinary_sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn validate_prefixed_uuid(value: &str, prefix: &str) -> Result<(), ConfigProjectionFailureV1> {
        let raw = value
            .strip_prefix(prefix)
            .ok_or(ConfigProjectionFailureV1::Malformed)?;
        let parsed = Uuid::parse_str(raw).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        if parsed.get_version_num() == 7 && parsed.to_string() == raw {
            Ok(())
        } else {
            Err(ConfigProjectionFailureV1::Malformed)
        }
    }

    fn prefixed_uuid(prefix: &str) -> String {
        format!("{prefix}{}", Uuid::now_v7())
    }

    fn validate_timestamp(timestamp: &Timestamp) -> Result<(), ConfigProjectionFailureV1> {
        let bytes = timestamp.0.as_bytes();
        if bytes.len() != 27
            || bytes.get(4) != Some(&b'-')
            || bytes.get(7) != Some(&b'-')
            || bytes.get(10) != Some(&b'T')
            || bytes.get(13) != Some(&b':')
            || bytes.get(16) != Some(&b':')
            || bytes.get(19) != Some(&b'.')
            || bytes.get(26) != Some(&b'Z')
            || !bytes[20..26].iter().all(u8::is_ascii_digit)
            || chrono::DateTime::parse_from_rfc3339(&timestamp.0).is_err()
        {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        Ok(())
    }

    fn now_timestamp() -> Timestamp {
        Timestamp(
            chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.6fZ")
                .to_string(),
        )
    }

    fn validate_installed_record(
        record: &InstalledAcceptedHomeBootstrapRecordV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if record.schema_version != 1 || record.intended_uid > u64::from(u32::MAX) {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        validate_timestamp(&record.installed_at)?;
        validate_hash_field(
            &record.record_hash,
            hash_omitting(
                "substrate.e3.installed-accepted-home-bootstrap.v1",
                "record",
                record,
                "record_hash",
            )?,
        )
    }

    fn validate_installed_head(
        head: &InstalledAcceptedHomeBootstrapHeadV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if head.schema_version != 1 {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        validate_sha256(&head.head_record_hash)?;
        if let Some(predecessor) = &head.predecessor_head_hash {
            validate_sha256(predecessor)?;
        }
        validate_timestamp(&head.updated_at)?;
        validate_hash_field(
            &head.head_hash,
            hash_omitting(
                "substrate.e3.installed-accepted-home-bootstrap-head.v1",
                "head",
                head,
                "head_hash",
            )?,
        )
    }

    fn validate_installed_carrier_binding(
        record: &InstalledAcceptedHomeBootstrapRecordV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformPrincipalV1};

        let carrier = InstallBootstrapContextCarrierV1::decode(&record.install_bootstrap_carrier)
            .map_err(|_| ConfigProjectionFailureV1::WrongBinding)?;
        carrier
            .validate()
            .map_err(|_| ConfigProjectionFailureV1::WrongBinding)?;
        let PlatformPrincipalV1::Unix { account, uid } = &carrier.context.intended_host_principal
        else {
            return Err(ConfigProjectionFailureV1::UnsupportedPlatform);
        };
        if carrier.context.host_substrate_home != record.accepted_home.physical_path
            || carrier.host_context_commitment != record.host_context_commitment
            || account != &record.intended_account
            || u64::from(*uid) != record.intended_uid
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let (current_uid, current_gid) = passwd_identity(account)?;
        if u64::from(current_uid) != record.intended_uid
            || u64::from(current_gid) != record.intended_gid
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(())
    }

    fn passwd_identity(account: &str) -> Result<(u32, u32), ConfigProjectionFailureV1> {
        let account = CString::new(account).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; 16 * 1024];
        let status = unsafe {
            libc::getpwnam_r(
                account.as_ptr(),
                pwd.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if status != 0 || result.is_null() {
            return Err(ConfigProjectionFailureV1::UnsupportedConfiguration);
        }
        let pwd = unsafe { pwd.assume_init() };
        Ok((pwd.pw_uid, pwd.pw_gid))
    }

    fn group_gid(name: &str) -> Result<libc::gid_t, ConfigProjectionFailureV1> {
        let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let mut group = std::mem::MaybeUninit::<libc::group>::uninit();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; 16 * 1024];
        let status = unsafe {
            libc::getgrnam_r(
                name.as_ptr(),
                group.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if status != 0 || result.is_null() {
            return Err(ConfigProjectionFailureV1::UnsupportedConfiguration);
        }
        Ok(unsafe { group.assume_init() }.gr_gid)
    }

    fn checked_uid(value: u64) -> Result<libc::uid_t, ConfigProjectionFailureV1> {
        libc::uid_t::try_from(value).map_err(|_| ConfigProjectionFailureV1::Malformed)
    }

    fn accepted_home_from_authority_child(
        child: &File,
    ) -> Result<CanonicalDirectoryV1, ConfigProjectionFailureV1> {
        let link = std::fs::read_link(format!("/proc/self/fd/{}", child.as_raw_fd()))
            .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        let authority = link
            .parent()
            .filter(|path| path.file_name().is_some_and(|name| name == "authority-v1"))
            .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
        let accepted_home = authority
            .parent()
            .ok_or(ConfigProjectionFailureV1::WrongBinding)?;
        let descriptor = open_absolute_directory(accepted_home)?;
        CanonicalDirectoryV1::capture_linux_from_fd(descriptor.as_fd())
    }

    #[repr(C)]
    struct OpenHow {
        flags: u64,
        mode: u64,
        resolve: u64,
    }

    fn openat2(
        parent: BorrowedFd<'_>,
        path: &str,
        flags: i32,
        mode: u32,
    ) -> Result<File, ConfigProjectionFailureV1> {
        let path = CString::new(path).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let how = OpenHow {
            flags: flags as u64,
            mode: u64::from(mode),
            resolve: OPENAT2_RESOLVE,
        };
        let fd = unsafe {
            libc::syscall(
                libc::SYS_openat2,
                parent.as_raw_fd(),
                path.as_ptr(),
                &how,
                std::mem::size_of::<OpenHow>(),
            )
        } as RawFd;
        if fd < 0 {
            return Err(map_io_error());
        }
        Ok(unsafe { File::from_raw_fd(fd) })
    }

    fn open_absolute_directory(path: &Path) -> Result<File, ConfigProjectionFailureV1> {
        if !path.is_absolute() {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        let root_fd = unsafe {
            libc::open(
                c"/".as_ptr(),
                libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if root_fd < 0 {
            return Err(map_io_error());
        }
        let mut current = unsafe { File::from_raw_fd(root_fd) };
        for component in path.components() {
            match component {
                Component::RootDir => {}
                Component::Normal(component) => {
                    let component = component
                        .to_str()
                        .ok_or(ConfigProjectionFailureV1::Malformed)?;
                    let component = CString::new(component)
                        .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
                    let fd = unsafe {
                        libc::openat(
                            current.as_raw_fd(),
                            component.as_ptr(),
                            libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                        )
                    };
                    if fd < 0 {
                        return Err(map_io_error());
                    }
                    current = unsafe { File::from_raw_fd(fd) };
                }
                _ => return Err(ConfigProjectionFailureV1::Malformed),
            }
        }
        Ok(current)
    }

    fn open_directory_at(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<File, ConfigProjectionFailureV1> {
        let directory = open_directory_raw_at(parent, name)?;
        let owner_uid = fstat(parent.as_raw_fd())?.st_uid;
        verify_directory(&directory, owner_uid, 0o700)?;
        Ok(directory)
    }

    fn open_directory_raw_at(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<File, ConfigProjectionFailureV1> {
        validate_component(name)?;
        openat2(
            parent,
            name,
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        )
    }

    fn open_optional_directory_at(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<Option<File>, ConfigProjectionFailureV1> {
        match open_directory_at(parent, name) {
            Ok(file) => Ok(Some(file)),
            Err(ConfigProjectionFailureV1::MissingPreparation) => Ok(None),
            Err(error) => Err(error),
        }
    }

    #[cfg(test)]
    type OwnershipTestHook =
        Box<dyn FnMut(&str, BorrowedFd<'_>, &str) -> Result<(), ConfigProjectionFailureV1>>;

    #[cfg(test)]
    thread_local! {
        static OWNERSHIP_TEST_HOOK: std::cell::RefCell<Option<OwnershipTestHook>> = const { std::cell::RefCell::new(None) };
    }

    #[cfg(test)]
    fn ownership_test_boundary(
        stage: &str,
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        OWNERSHIP_TEST_HOOK.with(|hook| {
            if let Some(hook) = hook.borrow_mut().as_mut() {
                hook(stage, parent, name)?;
            }
            Ok(())
        })
    }

    // Only exclusive creation paths may call this. Existing entries and EEXIST
    // races must never enter ownership establishment.
    fn establish_created_owner(
        parent: BorrowedFd<'_>,
        name: &str,
        file: &File,
        owner_uid: libc::uid_t,
        directory: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        establish_created_owner_with_gid(parent, name, file, owner_uid, directory, None)
    }

    fn establish_created_owner_with_gid(
        parent: BorrowedFd<'_>,
        name: &str,
        file: &File,
        owner_uid: libc::uid_t,
        directory: bool,
        owner_gid: Option<libc::gid_t>,
    ) -> Result<(), ConfigProjectionFailureV1> {
        #[cfg(test)]
        ownership_test_boundary("created", parent, name)?;
        let creator_uid = unsafe { libc::geteuid() };
        let before = verify_owned_entry(parent, name, file, creator_uid, directory)?;
        // Native-source parents are private 0700 directories, with no set-GID inheritance.
        if owner_gid.is_some() && before.st_gid != unsafe { libc::getegid() } {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        if (directory && !list_names(file)?.is_empty()) || (!directory && before.st_size != 0) {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        if (creator_uid != owner_uid || owner_gid.is_some_and(|gid| gid != before.st_gid))
            && unsafe { libc::fchown(file.as_raw_fd(), owner_uid, owner_gid.unwrap_or(!0)) } != 0
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        #[cfg(test)]
        ownership_test_boundary("owned", parent, name)?;
        let after =
            verify_owned_entry_with_gid(parent, name, file, owner_uid, directory, owner_gid)?;
        if before.st_dev != after.st_dev
            || before.st_ino != after.st_ino
            || before.st_mode != after.st_mode
            || owner_gid.unwrap_or(before.st_gid) != after.st_gid
            || before.st_nlink != after.st_nlink
            || before.st_size != after.st_size
            || before.st_mtime != after.st_mtime
            || before.st_mtime_nsec != after.st_mtime_nsec
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(())
    }

    fn verify_owned_entry(
        parent: BorrowedFd<'_>,
        name: &str,
        file: &File,
        owner_uid: libc::uid_t,
        directory: bool,
    ) -> Result<libc::stat, ConfigProjectionFailureV1> {
        let named = if directory {
            verify_directory(file, owner_uid, 0o700)?;
            open_directory_raw_at(parent, name)?
        } else {
            verify_regular_file(file, owner_uid, 0o600)?;
            open_file_at(parent, name, libc::O_RDONLY, 0)?
        };
        let before = fstat(file.as_raw_fd())?;
        let named_metadata = fstat(named.as_raw_fd())?;
        let after = fstat(file.as_raw_fd())?;
        if before.st_dev != fstat(parent.as_raw_fd())?.st_dev
            || !same_owned_metadata(&before, &named_metadata)
            || !same_owned_metadata(&before, &after)
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        if directory {
            verify_directory(&named, owner_uid, 0o700)?;
        } else {
            verify_regular_file(&named, owner_uid, 0o600)?;
        }
        if !same_owned_metadata(&after, &fstat(named.as_raw_fd())?) {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(after)
    }

    fn verify_owned_entry_with_gid(
        parent: BorrowedFd<'_>,
        name: &str,
        file: &File,
        owner_uid: libc::uid_t,
        directory: bool,
        owner_gid: Option<libc::gid_t>,
    ) -> Result<libc::stat, ConfigProjectionFailureV1> {
        let metadata = verify_owned_entry(parent, name, file, owner_uid, directory)?;
        if owner_gid.is_some_and(|gid| gid != metadata.st_gid) {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(metadata)
    }

    fn same_owned_metadata(a: &libc::stat, b: &libc::stat) -> bool {
        a.st_dev == b.st_dev
            && a.st_ino == b.st_ino
            && a.st_uid == b.st_uid
            && a.st_gid == b.st_gid
            && a.st_mode == b.st_mode
            && a.st_nlink == b.st_nlink
            && a.st_size == b.st_size
            && a.st_mtime == b.st_mtime
            && a.st_mtime_nsec == b.st_mtime_nsec
            && a.st_ctime == b.st_ctime
            && a.st_ctime_nsec == b.st_ctime_nsec
    }

    fn open_or_create_directory(
        parent: BorrowedFd<'_>,
        name: &str,
        owner_uid: libc::uid_t,
    ) -> Result<File, ConfigProjectionFailureV1> {
        open_or_create_directory_with_gid(parent, name, owner_uid, None)
    }

    fn open_or_create_directory_with_gid(
        parent: BorrowedFd<'_>,
        name: &str,
        owner_uid: libc::uid_t,
        owner_gid: Option<libc::gid_t>,
    ) -> Result<File, ConfigProjectionFailureV1> {
        validate_component(name)?;
        match open_directory_at(parent, name) {
            Ok(directory) => {
                verify_owned_entry_with_gid(parent, name, &directory, owner_uid, true, owner_gid)?;
                Ok(directory)
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {
                let name_c =
                    CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
                #[cfg(test)]
                ownership_test_boundary("before_create", parent, name)?;
                if unsafe { libc::mkdirat(parent.as_raw_fd(), name_c.as_ptr(), 0o700) } != 0 {
                    return Err(map_io_error());
                }
                // mkdir succeeded exclusively. Open without the final owner check,
                // then bind the still creator-owned, empty inode before fchown.
                let directory = open_directory_raw_at(parent, name)?;
                establish_created_owner_with_gid(
                    parent, name, &directory, owner_uid, true, owner_gid,
                )?;
                fsync(&directory)?;
                fsync_fd(parent.as_raw_fd())?;
                verify_owned_entry_with_gid(parent, name, &directory, owner_uid, true, owner_gid)?;
                Ok(directory)
            }
            Err(error) => Err(error),
        }
    }

    fn open_file_at(
        parent: BorrowedFd<'_>,
        name: &str,
        access: i32,
        mode: u32,
    ) -> Result<File, ConfigProjectionFailureV1> {
        validate_component(name)?;
        openat2(
            parent,
            name,
            access | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            mode,
        )
    }

    fn open_or_create_regular_file(
        parent: BorrowedFd<'_>,
        name: &str,
        owner_uid: libc::uid_t,
    ) -> Result<File, ConfigProjectionFailureV1> {
        match open_file_at(parent, name, libc::O_RDWR, 0) {
            Ok(file) => {
                verify_owned_entry(parent, name, &file, owner_uid, false)?;
                Ok(file)
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {
                #[cfg(test)]
                ownership_test_boundary("before_create", parent, name)?;
                let file = open_file_at(
                    parent,
                    name,
                    libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
                    0o600,
                )?;
                establish_created_owner(parent, name, &file, owner_uid, false)?;
                fsync(&file)?;
                fsync_fd(parent.as_raw_fd())?;
                verify_owned_entry(parent, name, &file, owner_uid, false)?;
                Ok(file)
            }
            Err(error) => Err(error),
        }
    }

    fn read_file_at(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<Vec<u8>, ConfigProjectionFailureV1> {
        let mut file = open_file_at(parent, name, libc::O_RDONLY, 0)?;
        let metadata = fstat(file.as_raw_fd())?;
        let parent_metadata = fstat(parent.as_raw_fd())?;
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG
            || metadata.st_uid != parent_metadata.st_uid
            || metadata.st_mode & 0o7777
                != if parent_metadata.st_mode & 0o7777 == 0o750 {
                    0o640
                } else {
                    0o600
                }
            || metadata.st_nlink != 1
            || metadata.st_size < 0
            || metadata.st_size as u64 > MAX_AUTHORITY_OBJECT_BYTES
            || has_xattrs(&file)?
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut bytes = Vec::with_capacity(metadata.st_size as usize);
        file.read_to_end(&mut bytes)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        if bytes.len() as u64 != metadata.st_size as u64 {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(bytes)
    }

    fn read_canonical_file<T>(file: &mut File) -> Result<T, ConfigProjectionFailureV1>
    where
        T: DeserializeOwned + Serialize,
    {
        let metadata = fstat(file.as_raw_fd())?;
        if metadata.st_size < 0 || metadata.st_size as u64 > MAX_AUTHORITY_OBJECT_BYTES {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut bytes = Vec::with_capacity(metadata.st_size as usize);
        file.read_to_end(&mut bytes)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        if bytes.len() as u64 != metadata.st_size as u64 {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        ConfigProjectionCodecV1::decode_canonical_json(&bytes)
    }

    fn read_canonical_at<T>(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<T, ConfigProjectionFailureV1>
    where
        T: DeserializeOwned + Serialize,
    {
        ConfigProjectionCodecV1::decode_canonical_json(&read_file_at(parent, name)?)
    }

    fn read_optional_canonical_at<T>(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<Option<T>, ConfigProjectionFailureV1>
    where
        T: DeserializeOwned + Serialize,
    {
        match read_file_at(parent, name) {
            Ok(bytes) => ConfigProjectionCodecV1::decode_canonical_json(&bytes).map(Some),
            Err(ConfigProjectionFailureV1::MissingPreparation) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn write_immutable(
        parent: BorrowedFd<'_>,
        final_name: &str,
        bytes: &[u8],
        kind: &str,
        owner_uid: libc::uid_t,
    ) -> Result<(), ConfigProjectionFailureV1> {
        validate_component(final_name)?;
        match read_file_at(parent, final_name) {
            Ok(existing) => {
                return if existing == bytes {
                    Ok(())
                } else {
                    Err(ConfigProjectionFailureV1::Conflict)
                };
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {}
            Err(error) => return Err(error),
        }
        let temp_name = temp_name(kind);
        let mut temp = open_file_at(
            parent,
            &temp_name,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        establish_created_owner(parent, &temp_name, &temp, owner_uid, false)?;
        #[cfg(test)]
        ownership_test_boundary("before_write", parent, &temp_name)?;
        verify_owned_entry(parent, &temp_name, &temp, owner_uid, false)?;
        temp.write_all(bytes)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        #[cfg(test)]
        ownership_test_boundary("written", parent, &temp_name)?;
        fsync(&temp)?;
        #[cfg(test)]
        ownership_test_boundary("before_publish", parent, &temp_name)?;
        verify_owned_entry(parent, &temp_name, &temp, owner_uid, false)?;
        rename_noreplace(parent, &temp_name, final_name)?;
        fsync_fd(parent.as_raw_fd())?;
        #[cfg(test)]
        ownership_test_boundary("published", parent, final_name)?;
        let published = verify_owned_entry(parent, final_name, &temp, owner_uid, false)?;
        if read_file_at(parent, final_name)? != bytes
            || !same_owned_metadata(
                &published,
                &verify_owned_entry(parent, final_name, &temp, owner_uid, false)?,
            )
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(())
    }

    fn cas_head(
        parent: BorrowedFd<'_>,
        expected: Option<&[u8]>,
        replacement: &[u8],
        kind: &str,
        owner_uid: libc::uid_t,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let current = match read_file_at(parent, "head.json") {
            Ok(bytes) => Some(bytes),
            Err(ConfigProjectionFailureV1::MissingPreparation) => None,
            Err(error) => return Err(error),
        };
        if current.as_deref() != expected {
            if current.as_deref() == Some(replacement) {
                return Ok(());
            }
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        let temp_name = temp_name(kind);
        let mut temp = open_file_at(
            parent,
            &temp_name,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        establish_created_owner(parent, &temp_name, &temp, owner_uid, false)?;
        #[cfg(test)]
        ownership_test_boundary("before_write", parent, &temp_name)?;
        verify_owned_entry(parent, &temp_name, &temp, owner_uid, false)?;
        temp.write_all(replacement)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        #[cfg(test)]
        ownership_test_boundary("written", parent, &temp_name)?;
        fsync(&temp)?;
        let rechecked = match read_file_at(parent, "head.json") {
            Ok(bytes) => Some(bytes),
            Err(ConfigProjectionFailureV1::MissingPreparation) => None,
            Err(error) => return Err(error),
        };
        if rechecked.as_deref() != expected {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        #[cfg(test)]
        ownership_test_boundary("before_publish", parent, &temp_name)?;
        verify_owned_entry(parent, &temp_name, &temp, owner_uid, false)?;
        if expected.is_none() {
            rename_noreplace(parent, &temp_name, "head.json")?;
        } else {
            rename_replace(parent, &temp_name, "head.json")?;
        }
        fsync_fd(parent.as_raw_fd())?;
        #[cfg(test)]
        ownership_test_boundary("published", parent, "head.json")?;
        let published = verify_owned_entry(parent, "head.json", &temp, owner_uid, false)?;
        if read_file_at(parent, "head.json")? != replacement
            || !same_owned_metadata(
                &published,
                &verify_owned_entry(parent, "head.json", &temp, owner_uid, false)?,
            )
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(())
    }

    fn recover_owned_temps(
        transaction: &ConfigProjectionChildTransactionV1,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let mut directories = vec![(Vec::<String>::new(), reopen_same(&transaction.root)?)];
        while let Some((path, directory)) = directories.pop() {
            let names = list_names(&directory)?;
            let temps: Vec<_> = names
                .iter()
                .filter(|name| name.starts_with(".e3-tmp."))
                .cloned()
                .collect();
            if temps.len() > 1 {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
            if let Some(temp) = temps.first() {
                recover_temp(transaction, directory.as_fd(), &path, temp)?;
            }
            for name in names {
                if name.starts_with(".e3-tmp.") || name == "lock" {
                    continue;
                }
                if entry_is_directory(directory.as_fd(), &name)? {
                    let child = open_directory_at(directory.as_fd(), &name)?;
                    let mut child_path = path.clone();
                    child_path.push(name);
                    directories.push((child_path, child));
                }
            }
        }
        Ok(())
    }

    fn recover_temp(
        transaction: &ConfigProjectionChildTransactionV1,
        parent: BorrowedFd<'_>,
        path: &[String],
        temp_name: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let kind = parse_temp_name(temp_name)?;
        let bytes = read_file_at(parent, temp_name)?;
        let (final_name, is_head) = validate_recovery_candidate(transaction, path, kind, &bytes)?;
        match read_file_at(parent, &final_name) {
            Ok(existing) if existing == bytes => {
                unlink_at(parent, temp_name)?;
                fsync_fd(parent.as_raw_fd())
            }
            Ok(existing) if is_head => {
                if recoverable_head_predecessor(transaction, path, &existing, &bytes)? {
                    rename_replace(parent, temp_name, &final_name)?;
                    fsync_fd(parent.as_raw_fd())
                } else {
                    Err(ConfigProjectionFailureV1::Conflict)
                }
            }
            Ok(_) => Err(ConfigProjectionFailureV1::Conflict),
            Err(ConfigProjectionFailureV1::MissingPreparation) if is_head => {
                if recoverable_initial_head(transaction, path, &bytes)? {
                    rename_noreplace(parent, temp_name, &final_name)?;
                    fsync_fd(parent.as_raw_fd())
                } else {
                    Err(ConfigProjectionFailureV1::Conflict)
                }
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {
                rename_noreplace(parent, temp_name, &final_name)?;
                fsync_fd(parent.as_raw_fd())
            }
            Err(error) => Err(error),
        }
    }

    fn validate_recovery_candidate(
        transaction: &ConfigProjectionChildTransactionV1,
        path: &[String],
        kind: &str,
        bytes: &[u8],
    ) -> Result<(String, bool), ConfigProjectionFailureV1> {
        match kind {
            "gateway"
            | "gateway-ack"
            | "gateway-intent"
            | "gateway-launch-input"
            | "handoff-revision"
            | "handoff-head" => validate_gateway_recovery_candidate(transaction, path, kind, bytes),
            "store" if path.is_empty() => {
                let store: ConfigProjectionStoreV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_store(&store, transaction)?;
                Ok(("store.json".to_string(), false))
            }
            "input" if path == ["inputs", "effective-config"] => {
                let store = require_store(transaction)?;
                let source: EffectiveSubstrateConfigSourceV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_effective_config_source(&source, &store)?;
                Ok((format!("{}.json", source.source_hash), false))
            }
            "input" if path == ["inputs", "agent-inventory"] => {
                let source: AgentInventorySourceMaterialV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_sha256(&source.raw_bytes_sha256)?;
                if source.source_revision != format!("aisr1_{}", source.raw_bytes_sha256) {
                    return Err(ConfigProjectionFailureV1::HashInvalid);
                }
                validate_hash_field(
                    &source.source_hash,
                    hash_omitting(
                        "substrate.e3.agent-inventory-source.v1",
                        "source",
                        &source,
                        "source_hash",
                    )?,
                )?;
                Ok((format!("{}.json", source.source_hash), false))
            }
            "artifact-manifest" if path.len() == 2 && path[0] == "runtime-artifacts" => {
                let store = require_store(transaction)?;
                let manifest: TrustedRuntimeArtifactManifestV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_runtime_artifact_manifest(&manifest, &store)?;
                if manifest.manifest_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok((format!("{:020}.json", manifest.revision), false))
            }
            "subject" if path == ["subjects"] => {
                let store = require_store(transaction)?;
                let binding: ConfigProjectionSubjectBindingV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_binding(&binding, &store)?;
                Ok((format!("{}.json", binding.subject_hash), false))
            }
            "record" if path.len() == 3 && path[0] == "series" && path[2] == "records" => {
                let store = require_store(transaction)?;
                let record: AgentConfigProjectionRecordV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_record(&record, &store)?;
                validate_gateway_ack_chain_v1(transaction, Some(&record), None)?;
                if record.identity.series_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok((record_name(&record), false))
            }
            "lease" if path.len() == 4 && path[0] == "leases" && path[3] == "revisions" => {
                let store = require_store(transaction)?;
                let lease: ConfigProjectionConsumerLeaseV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_lease(&lease, &lease.acquired_projection_ref)?;
                if lease.authority_store_id != store.authority_store_id
                    || lease.series_id != path[1]
                    || lease.consumer_id != path[2]
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok((format!("{:020}.json", lease.revision), false))
            }
            "kernel-effect-intent" if path == ["kernel-effects", "intents"] => {
                let store = require_store(transaction)?;
                let intent: E3KernelEffectIntentV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_kernel_effect_intent(&intent, &store)?;
                Ok((format!("{}.json", intent.effect_intent_id), false))
            }
            "kernel-effect-resolution" if path == ["kernel-effects", "resolutions"] => {
                let store = require_store(transaction)?;
                let resolution: E3KernelEffectResolutionV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                let intent = resolve_kernel_effect_intent(
                    transaction,
                    &store,
                    &resolution.effect_intent_ref,
                )?;
                validate_kernel_effect_resolution(&resolution, &store, &intent)?;
                ensure_intent_has_no_other_resolution(transaction, &resolution)?;
                Ok((format!("{}.json", resolution.resolution_id), false))
            }
            "child-cgroup" if path.len() == 2 && path[0] == "child-cgroups" => {
                let store = require_store(transaction)?;
                let registration: E3ChildCgroupRegistrationV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                let intent = resolve_kernel_effect_intent(
                    transaction,
                    &store,
                    &registration.kernel_effect_intent_ref,
                )?;
                validate_child_cgroup_registration(&registration, &store, &intent)?;
                if registration.series_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok((
                    format!("{}.json", registration.cgroup_registration_id),
                    false,
                ))
            }
            "child-process" if path.len() == 2 && path[0] == "child-processes" => {
                let store = require_store(transaction)?;
                let registration: E3ChildProcessRegistrationV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                let cgroup = resolve_child_cgroup_registration(
                    transaction,
                    &store,
                    &registration.series_id,
                    &registration.cgroup_registration_id,
                )?;
                validate_child_process_registration(&registration, &store, &cgroup)?;
                if registration.series_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok((format!("{}.json", registration.registration_id), false))
            }
            "terminal-child" if path.len() == 2 && path[0] == "terminal-child-evidence" => {
                let store = require_store(transaction)?;
                let evidence: E3TerminalChildQuiescenceEvidenceV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_terminal_evidence_object(transaction, &store, &evidence)?;
                if evidence.series_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok((format!("{}.json", evidence.evidence_id), false))
            }
            "boundary" if path.len() == 2 && path[0] == "gateway-boundaries" => {
                let store = require_store(transaction)?;
                let boundary: GatewayAccessBoundaryV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_boundary_object(&boundary, &store)?;
                if boundary.access_boundary_id != path[1] {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok((format!("{:020}.json", boundary.revision), false))
            }
            "retirement" if path == ["retirement"] => {
                let store = require_store(transaction)?;
                let retirement: ConfigProjectionRetirementV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                let head = read_series_head(transaction, &retirement.series_id)?;
                validate_retirement(&retirement, &store, &head.head_ref)?;
                if live_capabilities()
                    .lock()
                    .map_err(|_| ConfigProjectionFailureV1::Conflict)?
                    .get(&(
                        retirement.authority_store_id.clone(),
                        retirement.series_id.clone(),
                    ))
                    .copied()
                    .unwrap_or(0)
                    != 0
                {
                    return Err(ConfigProjectionFailureV1::Conflict);
                }
                ensure_all_leases_released(transaction, &retirement.series_id, None)?;
                validate_terminal_evidence(transaction, &retirement)?;
                validate_revoked_boundary(transaction, &retirement)?;
                Ok((format!("{}.json", retirement.series_id), false))
            }
            "head" if path.len() == 2 && path[0] == "series" => {
                let store = require_store(transaction)?;
                let head: ConfigProjectionHeadV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_head(&head, &store, &path[1])?;
                let record = read_record(transaction, &head.head_ref)?;
                validate_record(&record, &store)?;
                validate_gateway_ack_chain_v1(transaction, Some(&record), None)?;
                if record_ref(&record) != head.head_ref {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok(("head.json".to_string(), true))
            }
            "head" if path.len() == 3 && path[0] == "leases" => {
                let store = require_store(transaction)?;
                let lease: ConfigProjectionConsumerLeaseV1 =
                    ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
                validate_lease(&lease, &lease.acquired_projection_ref)?;
                if lease.authority_store_id != store.authority_store_id
                    || lease.series_id != path[1]
                    || lease.consumer_id != path[2]
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                let leases = open_directory_at(transaction.root.as_fd(), "leases")?;
                let series = open_directory_at(leases.as_fd(), &path[1])?;
                let consumer = open_directory_at(series.as_fd(), &path[2])?;
                let revisions = open_directory_at(consumer.as_fd(), "revisions")?;
                let revision_bytes =
                    read_file_at(revisions.as_fd(), &format!("{:020}.json", lease.revision))?;
                if revision_bytes != bytes {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
                Ok(("head.json".to_string(), true))
            }
            _ => Err(ConfigProjectionFailureV1::PartialPublication),
        }
    }

    fn lease_successor_binding_equal(
        current: &ConfigProjectionConsumerLeaseV1,
        next: &ConfigProjectionConsumerLeaseV1,
    ) -> bool {
        current.authority_store_id == next.authority_store_id
            && current.series_id == next.series_id
            && current.consumer_id == next.consumer_id
            && current.consumer_kind == next.consumer_kind
            && current.acquired_projection_ref == next.acquired_projection_ref
            && current.acquired_at == next.acquired_at
            && current.posture == ConfigProjectionConsumerLeasePostureV1::Held
            && next.posture == ConfigProjectionConsumerLeasePostureV1::Released
    }

    fn recoverable_head_predecessor(
        transaction: &ConfigProjectionChildTransactionV1,
        path: &[String],
        existing: &[u8],
        candidate: &[u8],
    ) -> Result<bool, ConfigProjectionFailureV1> {
        if path.len() == 2 && path[0] == "series" {
            let store = require_store(transaction)?;
            let current: ConfigProjectionHeadV1 =
                ConfigProjectionCodecV1::decode_canonical_json(existing)?;
            let next: ConfigProjectionHeadV1 =
                ConfigProjectionCodecV1::decode_canonical_json(candidate)?;
            validate_head(&current, &store, &path[1])?;
            validate_head(&next, &store, &path[1])?;
            let record = read_record(transaction, &next.head_ref)?;
            validate_record(&record, &store)?;
            validate_gateway_ack_chain_v1(transaction, Some(&record), None)?;
            return Ok(
                next.predecessor_head_hash.as_deref() == Some(&current.head_hash)
                    && next.head_revision == current.head_revision.saturating_add(1)
                    && record.predecessor_ref.as_ref() == Some(&current.head_ref),
            );
        }
        let store = require_store(transaction)?;
        let current: ConfigProjectionConsumerLeaseV1 =
            ConfigProjectionCodecV1::decode_canonical_json(existing)?;
        let next: ConfigProjectionConsumerLeaseV1 =
            ConfigProjectionCodecV1::decode_canonical_json(candidate)?;
        validate_lease(&current, &current.acquired_projection_ref)?;
        validate_lease(&next, &next.acquired_projection_ref)?;
        Ok(current.authority_store_id == store.authority_store_id
            && next.predecessor_lease_hash.as_deref() == Some(&current.lease_hash)
            && next.revision == current.revision.saturating_add(1)
            && lease_successor_binding_equal(&current, &next))
    }

    fn recoverable_initial_head(
        transaction: &ConfigProjectionChildTransactionV1,
        path: &[String],
        bytes: &[u8],
    ) -> Result<bool, ConfigProjectionFailureV1> {
        if path.len() == 2 && path[0] == "series" {
            let head: ConfigProjectionHeadV1 =
                ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
            let record = read_record(transaction, &head.head_ref)?;
            return Ok(head.predecessor_head_hash.is_none()
                && head.head_revision == 1
                && record.predecessor_ref.is_none());
        }
        let lease: ConfigProjectionConsumerLeaseV1 =
            ConfigProjectionCodecV1::decode_canonical_json(bytes)?;
        Ok(lease.predecessor_lease_hash.is_none() && lease.revision == 1)
    }

    fn validate_registry_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<Vec<E3KernelEffectRecoveryV1>, ConfigProjectionFailureV1> {
        let root_names = list_names(&transaction.root)?;
        let allowed: BTreeSet<&str> = [
            "lock",
            "store.json",
            "inputs",
            "runtime-artifacts",
            "native-sources",
            "subjects",
            "series",
            "leases",
            "gateway-boundaries",
            "gateways",
            "gateway-intents",
            "gateway-acks",
            "gateway-launch-inputs",
            "handoffs",
            "kernel-effects",
            "child-cgroups",
            "child-processes",
            "terminal-child-evidence",
            "retirement",
        ]
        .into_iter()
        .collect();
        for name in root_names {
            if name.starts_with(".e3-tmp.") {
                if parse_temp_name(&name)? != "store" {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
                let _ = read_file_at(transaction.root.as_fd(), &name)?;
            } else if !allowed.contains(name.as_str()) {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
        }
        if verify_objects
            && list_names(&transaction.root)?
                .iter()
                .any(|name| name == "store.json")
        {
            let store: ConfigProjectionStoreV1 =
                read_canonical_at(transaction.root.as_fd(), "store.json")?;
            validate_store(&store, transaction)?;
        }
        validate_subject_tree(transaction, verify_objects)?;
        validate_input_tree(transaction, verify_objects)?;
        validate_runtime_artifact_tree(transaction, verify_objects)?;
        validate_native_source_tree(transaction, verify_objects)?;
        let mut effects = validate_kernel_effect_tree(transaction, verify_objects)?;
        validate_series_tree(transaction, verify_objects, &mut effects)?;
        validate_lease_tree(transaction, verify_objects)?;
        validate_gateway_boundary_tree(transaction, verify_objects, &mut effects)?;
        validate_child_cgroup_tree(transaction, verify_objects, &mut effects)?;
        validate_child_process_tree(transaction, verify_objects, &mut effects)?;
        validate_gateway_preparation_tree(transaction, verify_objects, &mut effects)?;
        validate_terminal_evidence_tree(transaction, verify_objects, &mut effects)?;
        validate_retirement_tree(transaction, verify_objects)?;
        Ok(effects)
    }

    fn validate_input_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(inputs) = open_optional_directory_at(transaction.root.as_fd(), "inputs")? else {
            return Ok(());
        };
        let names = list_names(&inputs)?;
        if names != ["agent-inventory", "effective-config"] {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let store = verify_objects
            .then(|| require_store(transaction))
            .transpose()?;
        for kind in ["effective-config", "agent-inventory"] {
            let root = open_directory_at(inputs.as_fd(), kind)?;
            for name in list_names(&root)? {
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != "input" {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                    let _ = read_file_at(root.as_fd(), &name)?;
                    continue;
                }
                let hash = name
                    .strip_suffix(".json")
                    .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                validate_sha256(hash)?;
                if verify_objects {
                    if kind == "effective-config" {
                        let source: EffectiveSubstrateConfigSourceV1 =
                            read_canonical_at(root.as_fd(), &name)?;
                        validate_effective_config_source(
                            &source,
                            store
                                .as_ref()
                                .ok_or(ConfigProjectionFailureV1::WrongBinding)?,
                        )?;
                        if source.source_hash != hash {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    } else {
                        let source: AgentInventorySourceMaterialV1 =
                            read_canonical_at(root.as_fd(), &name)?;
                        if source.source_hash != hash
                            || !matches!(source.inventory_scope.as_str(), "global" | "workspace")
                            || source.source_revision
                                != format!("aisr1_{}", source.raw_bytes_sha256)
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                        validate_sha256(&source.raw_bytes_sha256)?;
                        validate_hash_field(
                            &source.source_hash,
                            hash_omitting(
                                "substrate.e3.agent-inventory-source.v1",
                                "source",
                                &source,
                                "source_hash",
                            )?,
                        )?;
                    }
                }
                let _ = read_file_at(root.as_fd(), &name)?;
            }
        }
        Ok(())
    }

    fn validate_runtime_artifact_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(root) = open_optional_directory_at(transaction.root.as_fd(), "runtime-artifacts")?
        else {
            return Ok(());
        };
        let store = verify_objects
            .then(|| require_store(transaction))
            .transpose()?;
        for manifest_id in list_names(&root)? {
            validate_prefixed_uuid(&manifest_id, "ram_")?;
            let manifest_root = open_directory_at(root.as_fd(), &manifest_id)?;
            for name in list_names(&manifest_root)? {
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != "artifact-manifest" {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                } else {
                    validate_revision_filename(&name)?;
                    if verify_objects {
                        let manifest: TrustedRuntimeArtifactManifestV1 =
                            read_canonical_at(manifest_root.as_fd(), &name)?;
                        validate_runtime_artifact_manifest(
                            &manifest,
                            store
                                .as_ref()
                                .ok_or(ConfigProjectionFailureV1::WrongBinding)?,
                        )?;
                        if manifest.manifest_id != manifest_id
                            || name != format!("{:020}.json", manifest.revision)
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    }
                }
                let _ = read_file_at(manifest_root.as_fd(), &name)?;
            }
        }
        Ok(())
    }

    fn validate_native_source_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(root) = open_optional_directory_at(transaction.root.as_fd(), "native-sources")?
        else {
            return Ok(());
        };
        let store = if verify_objects || !list_names(&root)?.is_empty() {
            Some(require_store(transaction)?)
        } else {
            None
        };
        for series_id in list_names(&root)? {
            validate_prefixed_uuid(&series_id, "cps_")?;
            let series = open_directory_at(root.as_fd(), &series_id)?;
            let mut temp_count = 0usize;
            for name in list_names(&series)? {
                if name.starts_with(".e3-native-source-tmp.") {
                    validate_native_temp_name(&name)?;
                    temp_count += 1;
                } else {
                    validate_prefixed_uuid(&name, "cpf_")?;
                }
                let manifest = validate_native_source_directory(
                    &series,
                    &name,
                    None,
                    None,
                    transaction.owner_uid,
                    transaction.native_source_gid,
                )?;
                let store = store
                    .as_ref()
                    .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                if manifest.authority_store_id != store.authority_store_id
                    || manifest.series_id != series_id
                    || manifest.source_root.physical_path
                        != format!(
                            "{}/authority-v1/agent-config-projection-v1/native-sources/{}/{}",
                            store.accepted_home.physical_path, series_id, manifest.fence_id
                        )
                {
                    return Err(ConfigProjectionFailureV1::WrongBinding);
                }
            }
            if temp_count > 1 {
                return Err(ConfigProjectionFailureV1::PartialPublication);
            }
        }
        Ok(())
    }

    fn validate_subject_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(subjects) = open_optional_directory_at(transaction.root.as_fd(), "subjects")?
        else {
            return Ok(());
        };
        for name in list_names(&subjects)? {
            if name.starts_with(".e3-tmp.") {
                if parse_temp_name(&name)? != "subject" {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
            } else {
                let hash = name
                    .strip_suffix(".json")
                    .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                validate_sha256(hash)?;
                if verify_objects {
                    let store = require_store(transaction)?;
                    let binding: ConfigProjectionSubjectBindingV1 =
                        read_canonical_at(subjects.as_fd(), &name)?;
                    validate_binding(&binding, &store)?;
                    if binding.subject_hash != hash {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                }
            }
            let _ = read_file_at(subjects.as_fd(), &name)?;
        }
        Ok(())
    }

    fn validate_series_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
        effects: &mut [E3KernelEffectRecoveryV1],
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(series_root) = open_optional_directory_at(transaction.root.as_fd(), "series")?
        else {
            return Ok(());
        };
        for series_id in list_names(&series_root)? {
            validate_prefixed_uuid(&series_id, "cps_")?;
            let series = open_directory_at(series_root.as_fd(), &series_id)?;
            for name in list_names(&series)? {
                match name.as_str() {
                    "head.json" => {
                        if verify_objects {
                            let store = require_store(transaction)?;
                            let head: ConfigProjectionHeadV1 =
                                read_canonical_at(series.as_fd(), &name)?;
                            validate_head(&head, &store, &series_id)?;
                            let record = read_record(transaction, &head.head_ref)?;
                            validate_record(&record, &store)?;
                            validate_gateway_ack_chain_v1(transaction, Some(&record), None)?;
                            if record_ref(&record) != head.head_ref {
                                return Err(ConfigProjectionFailureV1::WrongBinding);
                            }
                        }
                    }
                    "records" => {
                        let records = open_directory_at(series.as_fd(), "records")?;
                        for file_name in list_names(&records)? {
                            if file_name.starts_with(".e3-tmp.") {
                                if parse_temp_name(&file_name)? != "record" {
                                    return Err(ConfigProjectionFailureV1::PartialPublication);
                                }
                            } else {
                                validate_record_filename(&file_name)?;
                                if verify_objects {
                                    let store = require_store(transaction)?;
                                    let record: AgentConfigProjectionRecordV1 =
                                        read_canonical_at(records.as_fd(), &file_name)?;
                                    validate_record(&record, &store)?;
                                    validate_gateway_ack_chain_v1(
                                        transaction,
                                        Some(&record),
                                        None,
                                    )?;
                                    for effect in effects
                                        .iter_mut()
                                        .filter(|effect| effect.intent.series_id == series_id)
                                    {
                                        let fence = match &record.activation.publication_fence {
                                            crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => fence_id,
                                        };
                                        if effect.intent.fence_id == *fence {
                                            if effect.intent.preparation_id
                                                != record
                                                    .nonsecret_handoff
                                                    .credential_source_ref
                                                    .preparation_id
                                            {
                                                return Err(
                                                    ConfigProjectionFailureV1::WrongBinding,
                                                );
                                            }
                                            if effect
                                                .projection
                                                .as_ref()
                                                .is_none_or(|old| old.revision < record.revision)
                                            {
                                                effect.projection = Some(record.clone());
                                            } else if effect.projection.as_ref().is_some_and(
                                                |old| {
                                                    old.revision == record.revision
                                                        && old != &record
                                                },
                                            ) {
                                                return Err(ConfigProjectionFailureV1::Conflict);
                                            }
                                        }
                                    }
                                    if record.identity.series_id != series_id
                                        || file_name != record_name(&record)
                                    {
                                        return Err(ConfigProjectionFailureV1::WrongBinding);
                                    }
                                }
                            }
                            let _ = read_file_at(records.as_fd(), &file_name)?;
                        }
                    }
                    name if name.starts_with(".e3-tmp.") => {
                        if parse_temp_name(name)? != "head" {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        }
                        let _ = read_file_at(series.as_fd(), name)?;
                    }
                    _ => return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture),
                }
            }
        }
        Ok(())
    }

    fn validate_lease_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(leases) = open_optional_directory_at(transaction.root.as_fd(), "leases")? else {
            return Ok(());
        };
        for series_id in list_names(&leases)? {
            validate_prefixed_uuid(&series_id, "cps_")?;
            let series = open_directory_at(leases.as_fd(), &series_id)?;
            for consumer_id in list_names(&series)? {
                validate_prefixed_uuid(&consumer_id, "cpc_")?;
                let consumer = open_directory_at(series.as_fd(), &consumer_id)?;
                let mut head = None;
                let mut revisions_by_number = BTreeMap::new();
                for name in list_names(&consumer)? {
                    match name.as_str() {
                        "head.json" => {
                            if verify_objects {
                                let lease: ConfigProjectionConsumerLeaseV1 =
                                    read_canonical_at(consumer.as_fd(), &name)?;
                                head = Some(lease);
                            }
                        }
                        "revisions" => {
                            let revisions = open_directory_at(consumer.as_fd(), "revisions")?;
                            for file_name in list_names(&revisions)? {
                                if file_name.starts_with(".e3-tmp.") {
                                    if parse_temp_name(&file_name)? != "lease" {
                                        return Err(ConfigProjectionFailureV1::PartialPublication);
                                    }
                                } else {
                                    validate_revision_filename(&file_name)?;
                                    if verify_objects {
                                        let lease: ConfigProjectionConsumerLeaseV1 =
                                            read_canonical_at(revisions.as_fd(), &file_name)?;
                                        validate_lease(&lease, &lease.acquired_projection_ref)?;
                                        if lease.series_id != series_id
                                            || lease.consumer_id != consumer_id
                                            || file_name != format!("{:020}.json", lease.revision)
                                            || revisions_by_number
                                                .insert(lease.revision, lease)
                                                .is_some()
                                        {
                                            return Err(ConfigProjectionFailureV1::WrongBinding);
                                        }
                                    }
                                }
                                let _ = read_file_at(revisions.as_fd(), &file_name)?;
                            }
                        }
                        name if name.starts_with(".e3-tmp.") => {
                            if parse_temp_name(name)? != "head" {
                                return Err(ConfigProjectionFailureV1::PartialPublication);
                            }
                            let _ = read_file_at(consumer.as_fd(), name)?;
                        }
                        _ => return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture),
                    }
                }
                let mut predecessor: Option<&ConfigProjectionConsumerLeaseV1> = None;
                for (revision, lease) in &revisions_by_number {
                    if *revision == 1 {
                        if lease.predecessor_lease_hash.is_some()
                            || lease.posture != ConfigProjectionConsumerLeasePostureV1::Held
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    } else {
                        let Some(previous) = predecessor else {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        };
                        if *revision != previous.revision.saturating_add(1)
                            || lease.predecessor_lease_hash.as_deref() != Some(&previous.lease_hash)
                            || !lease_successor_binding_equal(previous, lease)
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    }
                    predecessor = Some(lease);
                }
                if let Some(head) = head {
                    validate_lease(&head, &head.acquired_projection_ref)?;
                    if head.series_id != series_id
                        || head.consumer_id != consumer_id
                        || revisions_by_number.get(&head.revision) != Some(&head)
                    {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_kernel_effect_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<Vec<E3KernelEffectRecoveryV1>, ConfigProjectionFailureV1> {
        let Some(root) = open_optional_directory_at(transaction.root.as_fd(), "kernel-effects")?
        else {
            return Ok(Vec::new());
        };
        if list_names(&root)? != ["intents", "resolutions"] {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let store = verify_objects
            .then(|| require_store(transaction))
            .transpose()?;
        let mut effects: Vec<E3KernelEffectRecoveryV1> = Vec::new();
        for (kind, prefix, temp_kind) in [
            ("intents", "eki_", "kernel-effect-intent"),
            ("resolutions", "ekr_", "kernel-effect-resolution"),
        ] {
            let directory = open_directory_at(root.as_fd(), kind)?;
            let mut resolved_intents = BTreeSet::new();
            for name in list_names(&directory)? {
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != temp_kind {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                } else {
                    let object_id = name
                        .strip_suffix(".json")
                        .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    validate_prefixed_uuid(object_id, prefix)?;
                    if let Some(store) = store.as_ref() {
                        if kind == "intents" {
                            let intent: E3KernelEffectIntentV1 =
                                read_canonical_at(directory.as_fd(), &name)?;
                            validate_kernel_effect_intent(&intent, store)?;
                            effects.push(E3KernelEffectRecoveryV1 {
                                intent: intent.clone(),
                                resolution: None,
                                child_cgroup: None,
                                child_processes: Vec::new(),
                                boundary: None,
                                projection: None,
                                terminal_child_evidence: Vec::new(),
                                gateway_config: None,
                            });
                            if intent.effect_intent_id != object_id {
                                return Err(ConfigProjectionFailureV1::WrongBinding);
                            }
                        } else {
                            let resolution: E3KernelEffectResolutionV1 =
                                read_canonical_at(directory.as_fd(), &name)?;
                            let intent = resolve_kernel_effect_intent(
                                transaction,
                                store,
                                &resolution.effect_intent_ref,
                            )?;
                            validate_kernel_effect_resolution(&resolution, store, &intent)?;
                            let effect = effects
                                .iter_mut()
                                .find(|effect| effect.intent == intent)
                                .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                            if effect.resolution.replace(resolution.clone()).is_some() {
                                return Err(ConfigProjectionFailureV1::Conflict);
                            }
                            if resolution.resolution_id != object_id
                                || !resolved_intents
                                    .insert(resolution.effect_intent_ref.effect_intent_id.clone())
                            {
                                return Err(ConfigProjectionFailureV1::Conflict);
                            }
                        }
                    }
                }
                let _ = read_file_at(directory.as_fd(), &name)?;
            }
        }
        Ok(effects)
    }

    fn validate_child_cgroup_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
        effects: &mut [E3KernelEffectRecoveryV1],
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(root) = open_optional_directory_at(transaction.root.as_fd(), "child-cgroups")?
        else {
            return Ok(());
        };
        let store = verify_objects
            .then(|| require_store(transaction))
            .transpose()?;
        for series_id in list_names(&root)? {
            validate_prefixed_uuid(&series_id, "cps_")?;
            let series = open_directory_at(root.as_fd(), &series_id)?;
            for name in list_names(&series)? {
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != "child-cgroup" {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                } else {
                    let registration_id = name
                        .strip_suffix(".json")
                        .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    validate_prefixed_uuid(registration_id, "ecg_")?;
                    if let Some(store) = store.as_ref() {
                        let registration: E3ChildCgroupRegistrationV1 =
                            read_canonical_at(series.as_fd(), &name)?;
                        let intent = resolve_kernel_effect_intent(
                            transaction,
                            store,
                            &registration.kernel_effect_intent_ref,
                        )?;
                        validate_child_cgroup_registration(&registration, store, &intent)?;
                        let effect = effects
                            .iter_mut()
                            .find(|effect| effect.intent == intent)
                            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                        if effect.child_cgroup.replace(registration.clone()).is_some() {
                            return Err(ConfigProjectionFailureV1::Conflict);
                        }
                        if registration.series_id != series_id
                            || registration.cgroup_registration_id != registration_id
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    }
                }
                let _ = read_file_at(series.as_fd(), &name)?;
            }
        }
        Ok(())
    }

    fn validate_child_process_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
        effects: &mut [E3KernelEffectRecoveryV1],
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(root) = open_optional_directory_at(transaction.root.as_fd(), "child-processes")?
        else {
            return Ok(());
        };
        let store = verify_objects
            .then(|| require_store(transaction))
            .transpose()?;
        for series_id in list_names(&root)? {
            validate_prefixed_uuid(&series_id, "cps_")?;
            let series = open_directory_at(root.as_fd(), &series_id)?;
            for name in list_names(&series)? {
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != "child-process" {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                } else {
                    let registration_id = name
                        .strip_suffix(".json")
                        .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    validate_prefixed_uuid(registration_id, "ecp_")?;
                    if let Some(store) = store.as_ref() {
                        let registration: E3ChildProcessRegistrationV1 =
                            read_canonical_at(series.as_fd(), &name)?;
                        let cgroup = resolve_child_cgroup_registration(
                            transaction,
                            store,
                            &series_id,
                            &registration.cgroup_registration_id,
                        )?;
                        validate_child_process_registration(&registration, store, &cgroup)?;
                        let effect = effects
                            .iter_mut()
                            .find(|effect| effect.child_cgroup.as_ref() == Some(&cgroup))
                            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                        effect.child_processes.push(registration.clone());
                        if registration.series_id != series_id
                            || registration.registration_id != registration_id
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                    }
                }
                let _ = read_file_at(series.as_fd(), &name)?;
            }
        }
        Ok(())
    }

    fn validate_terminal_evidence_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
        effects: &mut [E3KernelEffectRecoveryV1],
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(root) =
            open_optional_directory_at(transaction.root.as_fd(), "terminal-child-evidence")?
        else {
            return Ok(());
        };
        for series_id in list_names(&root)? {
            validate_prefixed_uuid(&series_id, "cps_")?;
            let series = open_directory_at(root.as_fd(), &series_id)?;
            for name in list_names(&series)? {
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != "terminal-child" {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                } else {
                    let evidence_id = name
                        .strip_suffix(".json")
                        .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                    validate_prefixed_uuid(evidence_id, "tce_")?;
                    if verify_objects {
                        let store = require_store(transaction)?;
                        let evidence: E3TerminalChildQuiescenceEvidenceV1 =
                            read_canonical_at(series.as_fd(), &name)?;
                        validate_terminal_evidence_object(transaction, &store, &evidence)?;
                        if evidence.series_id != series_id || evidence.evidence_id != evidence_id {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                        let record = read_record(transaction, &evidence.final_projection_ref)?;
                        let fence_id = match &record.activation.publication_fence {
                            crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed {
                                fence_id,
                            }
                            | crate::ConfigProjectionPublicationFenceV1::Released {
                                fence_id,
                                ..
                            } => fence_id,
                        };
                        for effect in effects.iter_mut().filter(|effect| {
                            effect.intent.authority_store_id == evidence.authority_store_id
                                && effect.intent.series_id == evidence.series_id
                        }) {
                            let final_attempt = effect.intent.fence_id == *fence_id
                                && effect.intent.preparation_id
                                    == record
                                        .nonsecret_handoff
                                        .credential_source_ref
                                        .preparation_id;
                            let covered =
                                effect.child_cgroup.as_ref().is_some_and(|registration| {
                                    evidence.ordered_empty_cgroups.iter().any(|entry| {
                                        entry.cgroup_registration_id
                                            == registration.cgroup_registration_id
                                            && entry.cgroup_registration_hash
                                                == registration.cgroup_registration_hash
                                    })
                                });
                            if final_attempt || covered {
                                if effect
                                    .terminal_child_evidence
                                    .iter()
                                    .any(|prior| prior.evidence_id == evidence.evidence_id)
                                {
                                    return Err(ConfigProjectionFailureV1::Conflict);
                                }
                                effect.terminal_child_evidence.push(evidence.clone());
                            }
                        }
                    }
                }
                let _ = read_file_at(series.as_fd(), &name)?;
            }
        }
        for effect in effects {
            effect.terminal_child_evidence.sort_by(|left, right| {
                (left.final_projection_ref.revision, &left.evidence_id)
                    .cmp(&(right.final_projection_ref.revision, &right.evidence_id))
            });
        }
        Ok(())
    }

    fn validate_gateway_boundary_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
        effects: &mut [E3KernelEffectRecoveryV1],
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(root) =
            open_optional_directory_at(transaction.root.as_fd(), "gateway-boundaries")?
        else {
            return Ok(());
        };
        for boundary_id in list_names(&root)? {
            validate_prefixed_uuid(&boundary_id, "gab_")?;
            let revisions = open_directory_at(root.as_fd(), &boundary_id)?;
            let store = verify_objects
                .then(|| require_store(transaction))
                .transpose()?;
            let mut prior = None;
            for name in list_names(&revisions)? {
                if name.starts_with(".e3-tmp.") {
                    if parse_temp_name(&name)? != "boundary" {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                } else {
                    validate_revision_filename(&name)?;
                    if let Some(store) = store.as_ref() {
                        let boundary: GatewayAccessBoundaryV1 =
                            read_canonical_at(revisions.as_fd(), &name)?;
                        validate_boundary_object(&boundary, store)?;
                        if boundary.access_boundary_id != boundary_id
                            || name != format!("{:020}.json", boundary.revision)
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                        if let Some(predecessor) = prior.as_ref() {
                            validate_boundary_transition(predecessor, &boundary)?;
                        } else if boundary.revision != 1 {
                            return Err(ConfigProjectionFailureV1::PartialPublication);
                        }
                        let effect = effects
                            .iter_mut()
                            .find(|effect| {
                                effect.intent.effect_intent_id
                                    == boundary.kernel_effect_intent_ref.effect_intent_id
                            })
                            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
                        if kernel_effect_intent_ref(&effect.intent)
                            != boundary.kernel_effect_intent_ref
                            || !matches!(&effect.intent.effect, E3KernelEffectKindV1::InstallGatewayBoundary {
                                access_boundary_id, network_namespace_inode, table_name, chain_name,
                            } if access_boundary_id == &boundary.access_boundary_id
                                && *network_namespace_inode == boundary.gateway_listener.network_namespace_inode
                                && table_name == &boundary.nftables_chain.table
                                && chain_name == &boundary.nftables_chain.chain)
                            || effect.boundary.as_ref().is_some_and(|old| {
                                old.access_boundary_id != boundary.access_boundary_id
                            })
                        {
                            return Err(ConfigProjectionFailureV1::WrongBinding);
                        }
                        if let Some(record) = &effect.projection {
                            if record.identity.authority_store_id != boundary.authority_store_id
                                || record.identity.identity_hash
                                    != boundary.config_projection_identity_hash
                                || record.identity.orchestration_session_id
                                    != boundary.orchestration_session_id
                                || record.identity.retained_participant_id
                                    != boundary.retained_participant_id
                                || record.identity.backend_id != boundary.backend_id
                                || record.identity.world_id != boundary.world_id
                                || record.identity.world_generation != boundary.world_generation
                                || record
                                    .managed_gateway
                                    .expected_gateway_ref
                                    .gateway_instance_id
                                    != boundary.gateway_instance_id
                                || record
                                    .managed_gateway
                                    .access_boundary_ref
                                    .access_boundary_id
                                    != boundary.access_boundary_id
                            {
                                return Err(ConfigProjectionFailureV1::WrongBinding);
                            }
                        }
                        effect.boundary = Some(boundary.clone());
                        prior = Some(boundary);
                    }
                }
                let _ = read_file_at(revisions.as_fd(), &name)?;
            }
        }
        Ok(())
    }

    fn validate_retirement_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let Some(root) = open_optional_directory_at(transaction.root.as_fd(), "retirement")? else {
            return Ok(());
        };
        for name in list_names(&root)? {
            if name.starts_with(".e3-tmp.") {
                if parse_temp_name(&name)? != "retirement" {
                    return Err(ConfigProjectionFailureV1::PartialPublication);
                }
            } else {
                let series_id = name
                    .strip_suffix(".json")
                    .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                validate_prefixed_uuid(series_id, "cps_")?;
                if verify_objects {
                    let store = require_store(transaction)?;
                    let retirement: ConfigProjectionRetirementV1 =
                        read_canonical_at(root.as_fd(), &name)?;
                    let head = read_series_head(transaction, series_id)?;
                    validate_retirement(&retirement, &store, &head.head_ref)?;
                    validate_terminal_evidence(transaction, &retirement)?;
                    validate_revoked_boundary(transaction, &retirement)?;
                    ensure_all_leases_released(transaction, series_id, None)?;
                    if retirement.series_id != series_id {
                        return Err(ConfigProjectionFailureV1::WrongBinding);
                    }
                }
            }
            let _ = read_file_at(root.as_fd(), &name)?;
        }
        Ok(())
    }

    fn validate_record_filename(name: &str) -> Result<(), ConfigProjectionFailureV1> {
        let (revision, record_id) = name
            .strip_suffix(".json")
            .and_then(|name| name.split_once('-'))
            .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if revision.len() != 20
            || !revision.bytes().all(|byte| byte.is_ascii_digit())
            || revision
                .parse::<u64>()
                .ok()
                .filter(|value| *value != 0)
                .is_none()
        {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        validate_prefixed_uuid(record_id, "cpr_")
    }

    fn validate_revision_filename(name: &str) -> Result<(), ConfigProjectionFailureV1> {
        let revision = name
            .strip_suffix(".json")
            .ok_or(ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
        if revision.len() == 20
            && revision.bytes().all(|byte| byte.is_ascii_digit())
            && revision
                .parse::<u64>()
                .ok()
                .filter(|value| *value != 0)
                .is_some()
        {
            Ok(())
        } else {
            Err(ConfigProjectionFailureV1::Malformed)
        }
    }

    fn temp_name(kind: &str) -> String {
        format!(".e3-tmp.{}.{kind}", Uuid::now_v7())
    }

    fn parse_temp_name(name: &str) -> Result<&str, ConfigProjectionFailureV1> {
        let rest = name
            .strip_prefix(".e3-tmp.")
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let (uuid, kind) = rest
            .split_once('.')
            .ok_or(ConfigProjectionFailureV1::PartialPublication)?;
        let parsed =
            Uuid::parse_str(uuid).map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        if parsed.get_version_num() != 7
            || parsed.to_string() != uuid
            || !matches!(
                kind,
                "store"
                    | "input"
                    | "artifact-manifest"
                    | "subject"
                    | "record"
                    | "head"
                    | "boundary"
                    | "gateway"
                    | "gateway-ack"
                    | "gateway-intent"
                    | "gateway-launch-input"
                    | "handoff-revision"
                    | "handoff-head"
                    | "lease"
                    | "kernel-effect-intent"
                    | "kernel-effect-resolution"
                    | "child-cgroup"
                    | "child-process"
                    | "terminal-child"
                    | "retirement"
            )
        {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(kind)
    }

    fn rename_noreplace(
        parent: BorrowedFd<'_>,
        source: &str,
        target: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        rename_at(parent, source, target, libc::RENAME_NOREPLACE)
    }

    fn rename_replace(
        parent: BorrowedFd<'_>,
        source: &str,
        target: &str,
    ) -> Result<(), ConfigProjectionFailureV1> {
        rename_at(parent, source, target, 0)
    }

    fn rename_at(
        parent: BorrowedFd<'_>,
        source: &str,
        target: &str,
        flags: u32,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let source = CString::new(source).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let target = CString::new(target).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let status = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                parent.as_raw_fd(),
                source.as_ptr(),
                parent.as_raw_fd(),
                target.as_ptr(),
                flags,
            )
        };
        if status != 0 {
            return Err(match std::io::Error::last_os_error().raw_os_error() {
                Some(libc::EEXIST) => ConfigProjectionFailureV1::Conflict,
                _ => ConfigProjectionFailureV1::PartialPublication,
            });
        }
        Ok(())
    }

    fn unlink_at(parent: BorrowedFd<'_>, name: &str) -> Result<(), ConfigProjectionFailureV1> {
        let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        if unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(())
    }

    fn list_names(directory: &File) -> Result<Vec<String>, ConfigProjectionFailureV1> {
        let duplicate = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                c".".as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if duplicate < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let stream = unsafe { libc::fdopendir(duplicate) };
        if stream.is_null() {
            unsafe { libc::close(duplicate) };
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let mut names = Vec::new();
        loop {
            unsafe { *libc::__errno_location() = 0 };
            let entry = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                let errno = unsafe { *libc::__errno_location() };
                unsafe { libc::closedir(stream) };
                if errno != 0 {
                    return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
                }
                break;
            }
            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }
                .to_str()
                .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            if name != "." && name != ".." {
                names.push(name.to_string());
            }
        }
        names.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        Ok(names)
    }

    fn entry_is_directory(
        parent: BorrowedFd<'_>,
        name: &str,
    ) -> Result<bool, ConfigProjectionFailureV1> {
        let name = CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        if unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(unsafe { metadata.assume_init() }.st_mode & libc::S_IFMT == libc::S_IFDIR)
    }

    fn reopen_same(file: &File) -> Result<File, ConfigProjectionFailureV1> {
        let fd = unsafe {
            libc::openat(
                file.as_raw_fd(),
                c".".as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(unsafe { File::from_raw_fd(fd) })
    }

    fn verify_directory(
        file: &File,
        owner_uid: libc::uid_t,
        expected_mode: u32,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let metadata = fstat(file.as_raw_fd())?;
        if metadata.st_mode & libc::S_IFMT != libc::S_IFDIR
            || metadata.st_uid != owner_uid
            || metadata.st_mode & 0o7777 != expected_mode
            || has_directory_xattrs(file)?
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(())
    }

    fn verify_regular_file(
        file: &File,
        owner_uid: libc::uid_t,
        expected_mode: u32,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let metadata = fstat(file.as_raw_fd())?;
        if metadata.st_mode & libc::S_IFMT != libc::S_IFREG
            || metadata.st_uid != owner_uid
            || metadata.st_mode & 0o7777 != expected_mode
            || metadata.st_nlink != 1
            || has_xattrs(file)?
        {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(())
    }

    fn verify_group(
        file: &File,
        expected_gid: libc::gid_t,
    ) -> Result<(), ConfigProjectionFailureV1> {
        if fstat(file.as_raw_fd())?.st_gid == expected_gid {
            Ok(())
        } else {
            Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
        }
    }

    fn has_xattrs(file: &File) -> Result<bool, ConfigProjectionFailureV1> {
        let size = unsafe { libc::flistxattr(file.as_raw_fd(), std::ptr::null_mut(), 0) };
        if size < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(size != 0)
    }

    fn has_directory_xattrs(file: &File) -> Result<bool, ConfigProjectionFailureV1> {
        let fd = unsafe {
            libc::openat(
                file.as_raw_fd(),
                c".".as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let reopened = unsafe { File::from_raw_fd(fd) };
        let original = fstat(file.as_raw_fd())?;
        let current = fstat(reopened.as_raw_fd())?;
        if original.st_dev != current.st_dev || original.st_ino != current.st_ino {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        has_xattrs(&reopened)
    }

    fn fstat(fd: RawFd) -> Result<libc::stat, ConfigProjectionFailureV1> {
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        if unsafe { libc::fstat(fd, metadata.as_mut_ptr()) } != 0 {
            return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        Ok(unsafe { metadata.assume_init() })
    }

    fn flock(file: &File, operation: i32) -> Result<(), ConfigProjectionFailureV1> {
        if unsafe { libc::flock(file.as_raw_fd(), operation) } != 0 {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        Ok(())
    }

    fn fsync(file: &File) -> Result<(), ConfigProjectionFailureV1> {
        fsync_fd(file.as_raw_fd())
    }

    fn fsync_directory_tree(directory: &File) -> Result<(), ConfigProjectionFailureV1> {
        for name in list_names(directory)? {
            if entry_is_directory(directory.as_fd(), &name)? {
                let child = open_directory_at(directory.as_fd(), &name)?;
                fsync_directory_tree(&child)?;
            }
        }
        fsync(directory)
    }

    fn fsync_fd(fd: RawFd) -> Result<(), ConfigProjectionFailureV1> {
        if unsafe { libc::fsync(fd) } != 0 {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(())
    }

    fn validate_component(name: &str) -> Result<(), ConfigProjectionFailureV1> {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.as_bytes().contains(&0)
        {
            Err(ConfigProjectionFailureV1::Malformed)
        } else {
            Ok(())
        }
    }

    fn map_io_error() -> ConfigProjectionFailureV1 {
        match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ENOENT) => ConfigProjectionFailureV1::MissingPreparation,
            Some(libc::EEXIST) => ConfigProjectionFailureV1::Conflict,
            Some(libc::EXDEV | libc::ELOOP | libc::ENOTDIR) => {
                ConfigProjectionFailureV1::UnsupportedSecurityPosture
            }
            _ => ConfigProjectionFailureV1::PartialPublication,
        }
    }

    #[cfg(test)]
    mod tests {
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;
        use std::sync::atomic::{AtomicUsize, Ordering};

        use serde_json::json;

        use super::*;

        struct TestParent {
            authority: PathBuf,
            parent_lock: PathBuf,
            calls: AtomicUsize,
        }

        impl ConfigProjectionHsaAuthorityV1 for TestParent {
            fn with_locked_parent(
                &self,
                operation: &mut dyn for<'fd> FnMut(
                    BorrowedFd<'fd>,
                )
                    -> Result<(), ConfigProjectionFailureV1>,
            ) -> Result<(), ConfigProjectionFailureV1> {
                let lock = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&self.parent_lock)
                    .map_err(|_| ConfigProjectionFailureV1::Conflict)?;
                flock(&lock, libc::LOCK_EX)?;
                let authority = File::open(&self.authority)
                    .map_err(|_| ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
                self.calls.fetch_add(1, Ordering::SeqCst);
                let result = operation(authority.as_fd());
                let unlock = flock(&lock, libc::LOCK_UN);
                result.and(unlock)
            }
        }

        #[test]
        #[ignore = "explicit root native-source group test; no process identity switching"]
        fn test_e3_native_source_authenticated_gid_creation_and_rejection() {
            assert_eq!(unsafe { libc::geteuid() }, 0);
            let temp = tempfile::tempdir().unwrap();
            let authority = File::open(temp.path()).unwrap();
            assert_eq!(
                unsafe { libc::fchown(authority.as_raw_fd(), 1000, 1000) },
                0
            );
            let transaction =
                ConfigProjectionChildTransactionV1::begin(authority.as_fd(), false).unwrap();
            assert_eq!(transaction.native_source_gid, 1000);
            // Existing registry parents retain their original GID. Only newly created source
            // entries acquire the authenticated group, before any source bytes are written.
            assert_eq!(
                fstat(transaction.root.as_raw_fd()).unwrap().st_gid,
                unsafe { libc::getegid() }
            );
            let source = open_or_create_directory_with_gid(
                transaction.root.as_fd(),
                "native-sources",
                transaction.owner_uid,
                Some(transaction.native_source_gid),
            )
            .unwrap();
            let config = open_or_create_directory_with_gid(
                source.as_fd(),
                "codex-home",
                transaction.owner_uid,
                Some(transaction.native_source_gid),
            )
            .unwrap();
            write_exclusive_file_with_gid(
                config.as_fd(),
                "config.toml",
                b"exact config bytes",
                transaction.owner_uid,
                Some(transaction.native_source_gid),
            )
            .unwrap();
            for file in [&source, &config] {
                let stat = fstat(file.as_raw_fd()).unwrap();
                assert_eq!(
                    (stat.st_uid, stat.st_gid, stat.st_mode & 0o7777),
                    (1000, 1000, 0o700)
                );
            }
            let file = open_file_at(config.as_fd(), "config.toml", libc::O_RDONLY, 0).unwrap();
            let stat = fstat(file.as_raw_fd()).unwrap();
            assert_eq!(
                (stat.st_uid, stat.st_gid, stat.st_mode & 0o7777),
                (1000, 1000, 0o600)
            );
            open_or_create_directory_with_gid(source.as_fd(), "codex-home", 1000, Some(1000))
                .unwrap();
            assert_eq!(unsafe { libc::fchown(config.as_raw_fd(), !0, 1001) }, 0);
            let before = fstat(config.as_raw_fd()).unwrap();
            assert!(open_or_create_directory_with_gid(
                source.as_fd(),
                "codex-home",
                1000,
                Some(1000)
            )
            .is_err());
            assert!(same_owned_metadata(
                &before,
                &fstat(config.as_raw_fd()).unwrap()
            ));
            assert_eq!(
                read_file_at(config.as_fd(), "config.toml").unwrap(),
                b"exact config bytes"
            );
            assert!(write_exclusive_file_with_gid(
                config.as_fd(),
                "config.toml",
                b"replacement",
                1000,
                Some(1000)
            )
            .is_err());
            assert_eq!(
                read_file_at(config.as_fd(), "config.toml").unwrap(),
                b"exact config bytes"
            );
        }

        #[test]
        #[ignore = "explicit root native-source group fault injection"]
        fn test_e3_native_source_gid_substitutions_reject_without_repair() {
            assert_eq!(unsafe { libc::geteuid() }, 0);
            for stage in [
                "before_create",
                "created",
                "owned",
                "before_write",
                "written",
            ] {
                let temp = tempfile::tempdir().unwrap();
                let parent = File::open(temp.path()).unwrap();
                let reached = Arc::new(AtomicUsize::new(0));
                let count = reached.clone();
                OWNERSHIP_TEST_HOOK.with(|hook| {
                    *hook.borrow_mut() = Some(Box::new(move |actual, parent, name| {
                        if actual == stage {
                            count.fetch_add(1, Ordering::SeqCst);
                            let file = if stage == "before_create" {
                                let name_c = CString::new(name).unwrap();
                                assert_eq!(
                                    unsafe {
                                        libc::mkdirat(parent.as_raw_fd(), name_c.as_ptr(), 0o700)
                                    },
                                    0
                                );
                                open_directory_raw_at(parent, name).unwrap()
                            } else {
                                open_file_at(parent, name, libc::O_RDONLY, 0).unwrap()
                            };
                            assert_eq!(unsafe { libc::fchown(file.as_raw_fd(), !0, 1001) }, 0);
                        }
                        Ok(())
                    }));
                });
                let result = if stage == "before_create" {
                    open_or_create_directory_with_gid(parent.as_fd(), "source", 0, Some(1000))
                        .map(|_| ())
                } else {
                    write_exclusive_file_with_gid(
                        parent.as_fd(),
                        "source",
                        b"planned",
                        0,
                        Some(1000),
                    )
                };
                OWNERSHIP_TEST_HOOK.with(|hook| hook.borrow_mut().take());
                assert_eq!(reached.load(Ordering::SeqCst), 1, "{stage}");
                assert!(result.is_err(), "{stage}");
                let file = open_file_at(parent.as_fd(), "source", libc::O_RDONLY, 0).unwrap();
                assert_eq!(fstat(file.as_raw_fd()).unwrap().st_gid, 1001, "{stage}");
                if stage != "before_create" {
                    let bytes = read_file_at(parent.as_fd(), "source").unwrap();
                    assert_eq!(
                        bytes,
                        if stage == "written" {
                            b"planned".as_slice()
                        } else {
                            b""
                        },
                        "{stage}"
                    );
                }
            }
        }

        #[test]
        #[ignore = "explicit root ownership test; no process identity switching"]
        fn test_e3_ownership_root_creation_and_existing_rejection() {
            assert_eq!(unsafe { libc::geteuid() }, 0);
            let temp = tempfile::tempdir().unwrap();
            let parent = File::open(temp.path()).unwrap();
            assert_eq!(unsafe { libc::fchown(parent.as_raw_fd(), 1000, 1000) }, 0);
            let directory = open_or_create_directory(parent.as_fd(), "child", 1000).unwrap();
            let lock = open_or_create_regular_file(directory.as_fd(), "lock", 1000).unwrap();
            write_exclusive_file(directory.as_fd(), "native", b"native", 1000).unwrap();
            write_immutable(directory.as_fd(), "object", b"immutable", "input", 1000).unwrap();
            cas_head(directory.as_fd(), None, b"first", "head", 1000).unwrap();
            cas_head(directory.as_fd(), Some(b"first"), b"second", "head", 1000).unwrap();
            for name in ["lock", "native", "object", "head.json"] {
                let file = open_file_at(directory.as_fd(), name, libc::O_RDONLY, 0).unwrap();
                verify_regular_file(&file, 1000, 0o600).unwrap();
            }
            assert_eq!(fstat(directory.as_raw_fd()).unwrap().st_uid, 1000);
            assert_eq!(
                read_file_at(directory.as_fd(), "head.json").unwrap(),
                b"second"
            );
            open_or_create_directory(parent.as_fd(), "child", 1000).unwrap();
            open_or_create_regular_file(directory.as_fd(), "lock", 1000).unwrap();
            // An existing wrong-owner object is rejected, never repaired.
            assert_eq!(unsafe { libc::fchown(lock.as_raw_fd(), 0, !0) }, 0);
            let before = fstat(lock.as_raw_fd()).unwrap();
            assert!(open_or_create_regular_file(directory.as_fd(), "lock", 1000).is_err());
            let after = fstat(lock.as_raw_fd()).unwrap();
            assert_eq!(
                (
                    before.st_uid,
                    before.st_ino,
                    before.st_ctime,
                    before.st_ctime_nsec
                ),
                (
                    after.st_uid,
                    after.st_ino,
                    after.st_ctime,
                    after.st_ctime_nsec
                )
            );
            assert_eq!(unsafe { libc::fchown(directory.as_raw_fd(), 0, !0) }, 0);
            assert!(open_or_create_directory(parent.as_fd(), "child", 1000).is_err());
            assert_eq!(fstat(directory.as_raw_fd()).unwrap().st_uid, 0);
        }

        #[test]
        fn test_e3_ownership_exclusive_creation_races_do_not_repair() {
            let uid = unsafe { libc::geteuid() };
            for directory in [false, true] {
                let temp = tempfile::tempdir().unwrap();
                let parent = File::open(temp.path()).unwrap();
                OWNERSHIP_TEST_HOOK.with(|hook| {
                    *hook.borrow_mut() = Some(Box::new(move |stage, parent, name| {
                        if stage == "before_create" {
                            let name_c = CString::new(name).unwrap();
                            if directory {
                                assert_eq!(
                                    unsafe {
                                        libc::mkdirat(parent.as_raw_fd(), name_c.as_ptr(), 0o750)
                                    },
                                    0
                                );
                            } else {
                                let mut file = open_file_at(
                                    parent,
                                    name,
                                    libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
                                    0o640,
                                )
                                .unwrap();
                                file.write_all(b"race winner").unwrap();
                            }
                        }
                        Ok(())
                    }))
                });
                let result = if directory {
                    open_or_create_directory(parent.as_fd(), "race", uid)
                } else {
                    open_or_create_regular_file(parent.as_fd(), "race", uid)
                };
                OWNERSHIP_TEST_HOOK.with(|hook| hook.borrow_mut().take());
                assert!(matches!(result, Err(ConfigProjectionFailureV1::Conflict)));
                assert_eq!(
                    std::fs::metadata(temp.path().join("race"))
                        .unwrap()
                        .permissions()
                        .mode()
                        & 0o7777,
                    if directory { 0o750 } else { 0o640 }
                );
                if !directory {
                    assert_eq!(
                        std::fs::read(temp.path().join("race")).unwrap(),
                        b"race winner"
                    );
                }
            }
        }

        #[test]
        #[ignore = "explicit root ownership interruption tests"]
        fn test_e3_ownership_interruptions_and_substitutions() {
            assert_eq!(unsafe { libc::geteuid() }, 0);
            for operation in ["exclusive", "immutable", "cas"] {
                for stage in [
                    "created",
                    "owned",
                    "before_write",
                    "written",
                    "before_publish",
                    "published",
                ] {
                    if operation == "exclusive" && matches!(stage, "before_publish" | "published") {
                        continue;
                    }
                    let temp = tempfile::tempdir().unwrap();
                    let parent = File::open(temp.path()).unwrap();
                    assert_eq!(unsafe { libc::fchown(parent.as_raw_fd(), 1000, 1000) }, 0);
                    let reached = Arc::new(AtomicUsize::new(0));
                    let count = reached.clone();
                    OWNERSHIP_TEST_HOOK.with(|hook| {
                        *hook.borrow_mut() = Some(Box::new(move |actual, _, _| {
                            if actual == stage {
                                count.fetch_add(1, Ordering::SeqCst);
                                return Err(ConfigProjectionFailureV1::PartialPublication);
                            }
                            Ok(())
                        }))
                    });
                    let result = match operation {
                        "exclusive" => {
                            write_exclusive_file(parent.as_fd(), "native", b"authority", 1000)
                        }
                        "immutable" => {
                            write_immutable(parent.as_fd(), "object", b"authority", "input", 1000)
                        }
                        _ => cas_head(parent.as_fd(), None, b"authority", "head", 1000),
                    };
                    OWNERSHIP_TEST_HOOK.with(|hook| hook.borrow_mut().take());
                    assert!(result.is_err(), "{operation}/{stage}");
                    assert_eq!(reached.load(Ordering::SeqCst), 1, "{operation}/{stage}");
                    let names = list_names(&parent).unwrap();
                    assert_eq!(names.len(), 1);
                    let held = open_file_at(parent.as_fd(), &names[0], libc::O_RDONLY, 0).unwrap();
                    let metadata = fstat(held.as_raw_fd()).unwrap();
                    assert_eq!(metadata.st_uid, if stage == "created" { 0 } else { 1000 });
                    assert_eq!(
                        metadata.st_size,
                        if matches!(stage, "created" | "owned" | "before_write") {
                            0
                        } else {
                            9
                        }
                    );
                    if operation != "exclusive" && stage != "published" {
                        assert!(!temp
                            .path()
                            .join(if operation == "cas" {
                                "head.json"
                            } else {
                                "object"
                            })
                            .exists());
                    }
                    if stage == "created" {
                        assert!(
                            open_or_create_regular_file(parent.as_fd(), &names[0], 1000).is_err()
                        );
                        assert_eq!(fstat(held.as_raw_fd()).unwrap().st_uid, 0);
                    }
                }
            }
            for stage in [
                "created",
                "owned",
                "before_write",
                "written",
                "before_publish",
                "published",
            ] {
                for mutation in ["mode", "owner", "xattr", "link", "substitute"] {
                    let temp = tempfile::tempdir().unwrap();
                    let parent = File::open(temp.path()).unwrap();
                    assert_eq!(unsafe { libc::fchown(parent.as_raw_fd(), 1000, 1000) }, 0);
                    let reached = Arc::new(AtomicUsize::new(0));
                    let count = reached.clone();
                    OWNERSHIP_TEST_HOOK.with(|hook| {
                        *hook.borrow_mut() = Some(Box::new(move |actual, parent, name| {
                            if actual != stage {
                                return Ok(());
                            }
                            count.fetch_add(1, Ordering::SeqCst);
                            let file = open_file_at(parent, name, libc::O_RDONLY, 0).unwrap();
                            let name_c = CString::new(name).unwrap();
                            match mutation {
                                "mode" => {
                                    assert_eq!(unsafe { libc::fchmod(file.as_raw_fd(), 0o640) }, 0)
                                }
                                "owner" => assert_eq!(
                                    unsafe { libc::fchown(file.as_raw_fd(), 1001, !0) },
                                    0
                                ),
                                "xattr" => assert_eq!(
                                    unsafe {
                                        libc::fsetxattr(
                                            file.as_raw_fd(),
                                            c"user.e3-test".as_ptr(),
                                            b"x".as_ptr().cast(),
                                            1,
                                            0,
                                        )
                                    },
                                    0
                                ),
                                "link" => assert_eq!(
                                    unsafe {
                                        libc::linkat(
                                            parent.as_raw_fd(),
                                            name_c.as_ptr(),
                                            parent.as_raw_fd(),
                                            c"extra".as_ptr(),
                                            0,
                                        )
                                    },
                                    0
                                ),
                                _ => {
                                    rename_noreplace(parent, name, "held-original").unwrap();
                                    let mut substitute = open_file_at(
                                        parent,
                                        name,
                                        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
                                        0o600,
                                    )
                                    .unwrap();
                                    assert_eq!(
                                        unsafe { libc::fchown(substitute.as_raw_fd(), 1000, !0) },
                                        0
                                    );
                                    substitute.write_all(b"substitute").unwrap();
                                }
                            }
                            Ok(())
                        }))
                    });
                    let result =
                        write_immutable(parent.as_fd(), "object", b"authority", "input", 1000);
                    OWNERSHIP_TEST_HOOK.with(|hook| hook.borrow_mut().take());
                    assert!(result.is_err(), "{stage}/{mutation}");
                    assert_eq!(reached.load(Ordering::SeqCst), 1);
                    if stage != "published" {
                        assert!(!temp.path().join("object").exists());
                    }
                    // Reopen still rejects the residue; it cannot be repaired or adopted.
                    if mutation == "owner" {
                        let name = list_names(&parent).unwrap().pop().unwrap();
                        let file = open_file_at(parent.as_fd(), &name, libc::O_RDONLY, 0).unwrap();
                        assert_eq!(fstat(file.as_raw_fd()).unwrap().st_uid, 1001);
                        assert!(open_or_create_regular_file(parent.as_fd(), &name, 1000).is_err());
                        assert_eq!(fstat(file.as_raw_fd()).unwrap().st_uid, 1001);
                    }
                }
            }
        }

        fn test_registry() -> (
            tempfile::TempDir,
            Arc<TestParent>,
            ConfigProjectionRegistryV1,
            ConfigProjectionStoreV1,
        ) {
            let temp = tempfile::tempdir().unwrap();
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
            let authority = temp.path().join("authority-v1");
            std::fs::create_dir(&authority).unwrap();
            std::fs::set_permissions(&authority, std::fs::Permissions::from_mode(0o700)).unwrap();
            let parent_lock = temp.path().join("parent.lock");
            let file = std::fs::File::create(&parent_lock).unwrap();
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .unwrap();
            let parent = Arc::new(TestParent {
                authority,
                parent_lock,
                calls: AtomicUsize::new(0),
            });
            let registry = ConfigProjectionRegistryV1::open(parent.clone()).unwrap();
            let store = registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            (temp, parent, registry, store)
        }

        fn digest() -> String {
            "11".repeat(32)
        }

        fn id(prefix: &str) -> String {
            prefixed_uuid(prefix)
        }

        fn test_record(
            store: &ConfigProjectionStoreV1,
            series_id: String,
        ) -> AgentConfigProjectionRecordV1 {
            let gateway_id = id("cgi_");
            let intent_id = id("gai_");
            let fence_id = id("cpf_");
            let dpc_id = id("dpc_");
            let artifact = |role: &str| {
                json!({
                    "role": role,
                    "configured_absolute_path": "/usr/bin/true",
                    "device_id": 1,
                    "inode": 2,
                    "file_type": "regular",
                    "mode": 0o755,
                    "owner_uid": 0,
                    "byte_length": 1,
                    "sha256": digest(),
                    "authority_ref": {
                        "authority_store_id": store.authority_store_id,
                        "manifest_id": id("ram_"),
                        "manifest_revision": 1,
                        "manifest_entry_id": id("rae_"),
                        "manifest_hash": digest(),
                        "entry_hash": digest()
                    },
                    "provenance": {"OfficialCodexRelease": {
                        "version": "0.125.0",
                        "target_triple": "x86_64-unknown-linux-musl",
                        "archive_name": "codex.tar.gz",
                        "archive_url": "https://example.invalid/codex.tar.gz",
                        "archive_sha256": digest(),
                        "archive_entry_path": "codex",
                        "extracted_executable_sha256": digest()
                    }},
                    "runtime_support": {
                        "schema_version": 1,
                        "support_policy_version": 1,
                        "elf_execution_model": "StaticExec",
                        "elf_interpreter": null,
                        "dynamic_loader_cache": null,
                        "ordered_elf_dependencies": [],
                        "ordered_present_common_files": [
                            {"absolute_path": "/etc/hosts", "device_id": 10, "inode": 20, "mode": 0o644, "byte_length": 1, "sha256": digest()},
                            {"absolute_path": "/etc/nsswitch.conf", "device_id": 10, "inode": 21, "mode": 0o644, "byte_length": 1, "sha256": digest()},
                            {"absolute_path": "/etc/passwd", "device_id": 10, "inode": 22, "mode": 0o644, "byte_length": 1, "sha256": digest()},
                            {"absolute_path": "/etc/group", "device_id": 10, "inode": 23, "mode": 0o644, "byte_length": 1, "sha256": digest()},
                            {"absolute_path": "/etc/resolv.conf", "device_id": 10, "inode": 24, "mode": 0o644, "byte_length": 1, "sha256": digest()},
                            {"absolute_path": "/etc/ssl/certs/ca-certificates.crt", "device_id": 10, "inode": 25, "mode": 0o644, "byte_length": 1, "sha256": digest()}
                        ],
                        "system_config_mount_target": {
                            "absolute_path": "/etc/codex",
                            "device_id": 1,
                            "inode": 3,
                            "mode": 0o755,
                            "owner_uid": 0,
                            "owner_gid": 0,
                            "ordered_entry_names": []
                        },
                        "manifest_hash": digest()
                    }
                })
            };
            let dpc_ref = json!({
                "authority_store_id": "hsa_store",
                "commitment_id": dpc_id,
                "exact_linkage_hash": digest()
            });
            let policy_ref = json!({
                "ref_id": format!("ao_{}", "22".repeat(16)),
                "object_kind": "policy",
                "schema_version": 1,
                "commitment": {"kind": "CanonicalSha256", "value": {"digest_hex": digest()}}
            });
            let cap = json!({
                "e2_activation_id": "e2a_test",
                "e2_launch_kind": "fresh_spawn",
                "commitment_ref": dpc_ref,
                "commitment_subject": {"RetainedWorkerLaunch": {
                    "retained_participant_id": "participant",
                    "bootstrap_run_id": "bootstrap"
                }},
                "immutable_worker_cap_ref": dpc_ref,
                "immutable_worker_cap_created_revision": 1,
                "immutable_worker_cap_application_revision": 1,
                "policy_snapshot_ref": policy_ref,
                "policy_snapshot_hash": digest(),
                "policy_snapshot_revision": "1",
                "request_id": "request",
                "idempotency_key": "idempotency",
                "caller_participant_id": "caller",
                "caller_backend_id": "cli:codex-world",
                "target_backend_id": "cli:codex-world",
                "target_world": {"world_id": "world", "world_generation": 1},
                "registry_publication_revision": 1
            });
            let intent_ref = json!({
                "authority_store_id": store.authority_store_id,
                "activation_intent_id": intent_id,
                "intent_hash": digest()
            });
            let gateway_ref = json!({
                "authority_store_id": store.authority_store_id,
                "gateway_instance_id": gateway_id,
                "gateway_identity_hash": digest()
            });
            let directory = serde_json::to_value(&store.accepted_home).unwrap();
            let mut record: AgentConfigProjectionRecordV1 = serde_json::from_value(json!({
                "schema_version": 1,
                "identity": {
                    "schema_version": 1,
                    "authority_store_id": store.authority_store_id,
                    "series_id": series_id,
                    "accepted_home": directory,
                    "workspace_root": directory,
                    "orchestration_session_id": "session",
                    "retained_participant_id": "participant",
                    "bootstrap_run_id": "bootstrap",
                    "backend_id": "cli:codex-world",
                    "runtime_family": "codex",
                    "world_id": "world",
                    "world_generation": 1,
                    "immutable_launch_cap": cap,
                    "runtime_artifacts": {
                        "codex": artifact("Codex0125"),
                        "world_entry_wrapper": artifact("WorldEntryWrapper"),
                        "managed_gateway": artifact("ManagedGateway")
                    },
                    "identity_hash": digest()
                },
                "record_id": id("cpr_"),
                "revision": 1,
                "predecessor_ref": null,
                "logical": {
                    "projection_hash": digest(),
                    "sources": [{"EffectiveSubstrateConfig": {
                        "authority_store_id": store.authority_store_id,
                        "source_revision": "1", "source_hash": digest()
                    }}, {"AgentInventory": {
                        "authority_store_id": store.authority_store_id,
                        "inventory_scope": "global", "source_revision": "1", "source_hash": digest()
                    }}],
                    "logical_agent_id": "agent", "placement": "world",
                    "realized_agent_id": "agent", "backend_id": "cli:codex-world",
                    "kind": "cli", "protocol": "pure-agent", "execution_scope": "world",
                    "cli_mode": "persistent", "runtime_family": "codex",
                    "capabilities": [], "requested_model": "codex",
                    "requested_mcp_servers": [], "requested_features": []
                },
                "effective": {
                    "projection_hash": digest(), "logical_projection_hash": digest(),
                    "accepted_policy": cap, "capabilities": [], "model": "codex",
                    "provider": {"provider_id": "substrate-managed-gateway", "wire_api": "responses",
                        "requires_openai_auth": false, "supports_websockets": false,
                        "gateway_intent_ref": intent_ref},
                    "mcp_servers": [], "features": [],
                    "environment": {"inherited_names": [], "set": [], "remove": []},
                    "workspace_overlay": "Disabled"
                },
                "native": {
                    "projection_hash": digest(),
                    "renderer": {"renderer_id": "substrate.codex.config-renderer",
                        "renderer_schema_version": 1, "codex_version": "0.125.0"},
                    "root": {"root_id": "root", "authority_relative_path": "native/root",
                        "guest_absolute_path": "/run/substrate/root", "owner_uid": 1000,
                        "owner_gid": 1000, "directory_mode": 0o700},
                    "files": [], "directories": [],
                    "environment": {"inherited_names": [], "set": [], "remove": []},
                    "invocation": {"wrapper_argv": [], "initial_codex_argv": [],
                        "resume_codex_argv_prefix": [], "prompt_delivery": "stdin-lf-eof",
                        "output_last_message_directory": "out",
                        "output_last_message_name_domain": "turn", "forbidden_arguments": []},
                    "cwd": directory,
                    "ambient_closure": {
                        "loader_source": {"codex_version": "0.125.0", "upstream_tag": "rust-v0.125.0",
                            "config_loader_source_sha256": digest(), "layer_io_source_sha256": digest(),
                            "loader_model_source_sha256": digest(), "exec_source_sha256": digest(),
                            "cloud_requirements_source_sha256": digest(), "auth_storage_source_sha256": digest(),
                            "config_types_source_sha256": digest(), "validator_schema_version": 1},
                        "allowed_enabled_layers": [], "inputs": [], "forbidden_cli_overrides": [],
                        "validated_loader_input_fingerprint": digest()
                    }
                },
                "managed_gateway": {
                    "projection_hash": digest(), "activation_intent_ref": intent_ref,
                    "expected_gateway_ref": gateway_ref, "codex_base_url": "http://127.0.0.1:1/v1",
                    "access_boundary_ref": {"authority_store_id": store.authority_store_id,
                        "access_boundary_id": id("gab_"), "revision": 1, "boundary_hash": digest()},
                    "activation_ack_ref": null, "posture": "Dormant"
                },
                "nonsecret_handoff": {
                    "projection_hash": digest(),
                    "secret_handoff_ref": {"authority_store_id": store.authority_store_id,
                        "handoff_id": id("gsh_"), "orchestration_session_id": "session",
                        "retained_participant_id": "participant", "runtime_family": "codex",
                        "world_id": "world", "world_generation": 1,
                        "receiving_gateway_identity_hash": digest(), "handoff_state_revision": 1,
                        "handoff_hash": digest()},
                    "credential_source_ref": {"schema_version": 1, "credential_source_id": "credential",
                        "preparation_id": id("e3p_"), "selected_backend_id": "cli:codex-world",
                        "bundle_backend_id": "cli:codex", "ordered_field_names": [],
                        "optional_account_id_present": false, "issued_at": "2026-09-10T00:00:00.000000Z",
                        "expires_at": "2026-09-10T01:00:00.000000Z", "ref_hash": digest()},
                    "receiving_gateway_ref": gateway_ref,
                    "delivery": {"SecureFd": {"fd_name": "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
                        "one_time": true, "gateway_receiver_only": true,
                        "deny_child_inheritance": true, "close_after_consume": true}},
                    "observed_state": "Prepared", "activation_ack_ref": null
                },
                "activation": {"activation_intent_ref": intent_ref,
                    "publication_fence": {"ZeroLiveClosed": {"fence_id": fence_id}},
                    "gateway_activation_ack_ref": null, "released_at": null},
                "created_at": "2026-09-10T00:00:00.000000Z",
                "record_hash": digest()
            }))
            .unwrap();
            for artifact in [
                &mut record.identity.runtime_artifacts.codex,
                &mut record.identity.runtime_artifacts.world_entry_wrapper,
                &mut record.identity.runtime_artifacts.managed_gateway,
            ] {
                artifact.runtime_support.manifest_hash.clear();
                artifact.runtime_support.manifest_hash = hash_omitting(
                    "substrate.e3.runtime-support-manifest.v1",
                    "manifest",
                    &artifact.runtime_support,
                    "manifest_hash",
                )
                .unwrap();
            }
            record
                .nonsecret_handoff
                .credential_source_ref
                .ordered_field_names =
                vec!["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN".to_string()];
            record
                .nonsecret_handoff
                .credential_source_ref
                .ref_hash
                .clear();
            record.nonsecret_handoff.credential_source_ref.ref_hash = hash_omitting(
                "substrate.e3.credential-source-ref.v1",
                "credential_source",
                &record.nonsecret_handoff.credential_source_ref,
                "ref_hash",
            )
            .unwrap();
            seal_record(&mut record);
            let boundary = dormant_boundary(&record);
            record.managed_gateway.access_boundary_ref = boundary_ref(&boundary);
            seal_record(&mut record);
            record
        }

        fn seal_record(record: &mut AgentConfigProjectionRecordV1) {
            record.identity.identity_hash = hash_omitting(
                "substrate.e3.config-projection-identity.v1",
                "identity",
                &record.identity,
                "identity_hash",
            )
            .unwrap();
            record.logical.projection_hash = hash_omitting(
                "substrate.e3.logical-config-projection.v1",
                "projection",
                &record.logical,
                "projection_hash",
            )
            .unwrap();
            record.effective.logical_projection_hash = record.logical.projection_hash.clone();
            record.effective.projection_hash = hash_omitting(
                "substrate.e3.effective-config-projection.v1",
                "projection",
                &record.effective,
                "projection_hash",
            )
            .unwrap();
            record.native.projection_hash = hash_omitting(
                "substrate.e3.native-config-projection.v1",
                "projection",
                &record.native,
                "projection_hash",
            )
            .unwrap();
            record.managed_gateway.projection_hash = hash_omitting(
                "substrate.e3.managed-gateway-projection.v1",
                "projection",
                &record.managed_gateway,
                "projection_hash",
            )
            .unwrap();
            record.nonsecret_handoff.projection_hash = hash_omitting(
                "substrate.e3.nonsecret-handoff-projection.v1",
                "projection",
                &record.nonsecret_handoff,
                "projection_hash",
            )
            .unwrap();
            record.record_hash = hash_omitting(
                "substrate.e3.agent-config-projection-record.v1",
                "record",
                record,
                "record_hash",
            )
            .unwrap();
        }

        fn successor(
            current: &AgentConfigProjectionRecordV1,
            posture: ManagedGatewayProjectionPostureV1,
        ) -> AgentConfigProjectionRecordV1 {
            let mut next = current.clone();
            next.predecessor_ref = Some(record_ref(current));
            next.record_id = id("cpr_");
            next.revision += 1;
            next.managed_gateway.posture = posture;
            let ack = current
                .managed_gateway
                .activation_ack_ref
                .clone()
                .unwrap_or_else(|| crate::ManagedGatewayActivationAckRefV1 {
                    authority_store_id: next.identity.authority_store_id.clone(),
                    activation_ack_id: id("gaa_"),
                    ack_hash: digest(),
                });
            if current.nonsecret_handoff.observed_state == SecretHandoffStateV1::Prepared {
                next.nonsecret_handoff
                    .secret_handoff_ref
                    .handoff_state_revision += 2;
                next.nonsecret_handoff.secret_handoff_ref.handoff_hash = "22".repeat(32);
            }
            next.managed_gateway.activation_ack_ref = Some(ack.clone());
            next.nonsecret_handoff.activation_ack_ref = Some(ack.clone());
            next.nonsecret_handoff.observed_state = SecretHandoffStateV1::Consumed;
            next.activation.gateway_activation_ack_ref = Some(ack.clone());
            if posture == ManagedGatewayProjectionPostureV1::Active {
                next.activation.released_at =
                    Some(Timestamp("2026-09-10T00:02:00.000000Z".to_string()));
                let fence_id = match &current.activation.publication_fence {
                    crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } => {
                        fence_id.clone()
                    }
                    _ => panic!("active predecessor must remain closed"),
                };
                let initial_boundary = dormant_boundary(current);
                let allow_boundary =
                    successor_boundary(&initial_boundary, GatewayAccessPostureV1::AllowExactMember);
                next.managed_gateway.access_boundary_ref = boundary_ref(&allow_boundary);
                let closed_record_ref = record_ref(current);
                let release_hash = ConfigProjectionCodecV1::domain_sha256(
                    "substrate.e3.config-projection-release.v1",
                    &json!({
                        "activation_ack_ref": ack,
                        "closed_record_ref": closed_record_ref,
                        "fence_id": fence_id
                    }),
                )
                .unwrap();
                next.activation.publication_fence =
                    crate::ConfigProjectionPublicationFenceV1::Released {
                        fence_id,
                        closed_record_ref,
                        activation_ack_ref: ack,
                        release_hash,
                    };
            }
            seal_record(&mut next);
            next
        }

        fn boundary_ref(boundary: &GatewayAccessBoundaryV1) -> GatewayAccessBoundaryRefV1 {
            GatewayAccessBoundaryRefV1 {
                authority_store_id: boundary.authority_store_id.clone(),
                access_boundary_id: boundary.access_boundary_id.clone(),
                revision: boundary.revision,
                boundary_hash: boundary.boundary_hash.clone(),
            }
        }

        fn boundary_rules(
            posture: GatewayAccessPostureV1,
            table: &str,
        ) -> Vec<crate::NftablesRuleIdentityV1> {
            let roles: &[NftablesRuleRoleV1] = match posture {
                GatewayAccessPostureV1::DenyAllDormant => &[
                    NftablesRuleRoleV1::ReadinessProbeAccept,
                    NftablesRuleRoleV1::RejectRemainder,
                ],
                GatewayAccessPostureV1::AllowExactMember => &[
                    NftablesRuleRoleV1::ReadinessProbeAccept,
                    NftablesRuleRoleV1::ExactMemberAccept,
                    NftablesRuleRoleV1::RejectRemainder,
                ],
                GatewayAccessPostureV1::Revoked => &[NftablesRuleRoleV1::RejectRemainder],
            };
            roles
                .iter()
                .enumerate()
                .map(|(index, role)| crate::NftablesRuleIdentityV1 {
                    role: *role,
                    family: "inet".to_string(),
                    table: table.to_string(),
                    chain: "gateway_output".to_string(),
                    rule_handle: 10 + index as u64,
                    canonical_expression: "{}".to_string(),
                })
                .collect()
        }

        fn dormant_boundary_intent(
            record: &AgentConfigProjectionRecordV1,
        ) -> E3KernelEffectIntentV1 {
            let boundary_id = &record
                .managed_gateway
                .access_boundary_ref
                .access_boundary_id;
            let fence_id = match &record.activation.publication_fence {
                crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id }
                | crate::ConfigProjectionPublicationFenceV1::Released { fence_id, .. } => fence_id,
            };
            let mut intent = E3KernelEffectIntentV1 {
                schema_version: 1,
                authority_store_id: record.identity.authority_store_id.clone(),
                series_id: record.identity.series_id.clone(),
                effect_intent_id: format!("eki_{}", boundary_id.strip_prefix("gab_").unwrap()),
                preparation_id: record
                    .nonsecret_handoff
                    .credential_source_ref
                    .preparation_id
                    .clone(),
                fence_id: fence_id.clone(),
                created_at: record.created_at.clone(),
                effect: E3KernelEffectKindV1::InstallGatewayBoundary {
                    access_boundary_id: boundary_id.clone(),
                    network_namespace_inode: 7,
                    table_name: format!(
                        "substrate_e3_{}",
                        &ordinary_sha256(boundary_id.as_bytes())[..24]
                    ),
                    chain_name: "gateway_output".into(),
                },
                intent_hash: String::new(),
            };
            intent.intent_hash = hash_omitting(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                &intent,
                "intent_hash",
            )
            .unwrap();
            intent
        }

        fn dormant_boundary(record: &AgentConfigProjectionRecordV1) -> GatewayAccessBoundaryV1 {
            let access_boundary_id = record
                .managed_gateway
                .access_boundary_ref
                .access_boundary_id
                .clone();
            let table = format!(
                "substrate_e3_{}",
                &ordinary_sha256(access_boundary_id.as_bytes())[..24]
            );
            let mut boundary = GatewayAccessBoundaryV1 {
                schema_version: 1,
                authority_store_id: record.identity.authority_store_id.clone(),
                kernel_effect_intent_ref: kernel_effect_intent_ref(&dormant_boundary_intent(
                    record,
                )),
                access_boundary_id,
                gateway_instance_id: record
                    .managed_gateway
                    .expected_gateway_ref
                    .gateway_instance_id
                    .clone(),
                config_projection_identity_hash: record.identity.identity_hash.clone(),
                orchestration_session_id: record.identity.orchestration_session_id.clone(),
                retained_participant_id: record.identity.retained_participant_id.clone(),
                backend_id: record.identity.backend_id.clone(),
                world_id: record.identity.world_id.clone(),
                world_generation: record.identity.world_generation,
                gateway_listener: crate::GatewayListenerIdentityV1 {
                    transport: "tcp".to_string(),
                    network_namespace_inode: 7,
                    address: "127.0.0.1".to_string(),
                    port: 43123,
                    socket_inode: 8,
                    listen_backlog: 16,
                    deny_boundary_effective_before_listen: true,
                    responses_base_path: "/v1".to_string(),
                },
                readiness_probe_cgroup: crate::CanonicalCgroupIdentityV1 {
                    cgroup_v2_mount_device_id: 1,
                    cgroup_v2_mount_inode: 2,
                    cgroup_directory_inode: 3,
                    cgroup_relative_path: "readiness".to_string(),
                },
                allowed_member_cgroup: crate::CanonicalCgroupIdentityV1 {
                    cgroup_v2_mount_device_id: 1,
                    cgroup_v2_mount_inode: 2,
                    cgroup_directory_inode: 4,
                    cgroup_relative_path: "member".to_string(),
                },
                nftables_chain: crate::NftablesChainIdentityV1 {
                    family: "inet".to_string(),
                    table: table.clone(),
                    chain: "gateway_output".to_string(),
                    chain_handle: 9,
                    chain_type: "filter".to_string(),
                    hook: "output".to_string(),
                    priority: -100,
                    policy: "accept".to_string(),
                },
                nftables_rules: boundary_rules(GatewayAccessPostureV1::DenyAllDormant, &table),
                posture: GatewayAccessPostureV1::DenyAllDormant,
                revision: 1,
                predecessor_ref: None,
                boundary_hash: String::new(),
            };
            seal_boundary(&mut boundary);
            boundary
        }

        fn successor_boundary(
            predecessor: &GatewayAccessBoundaryV1,
            posture: GatewayAccessPostureV1,
        ) -> GatewayAccessBoundaryV1 {
            let mut boundary = predecessor.clone();
            boundary.revision += 1;
            boundary.predecessor_ref = Some(boundary_ref(predecessor));
            boundary.posture = posture;
            boundary.nftables_rules = boundary_rules(posture, &boundary.nftables_chain.table);
            seal_boundary(&mut boundary);
            boundary
        }

        fn seal_boundary(boundary: &mut GatewayAccessBoundaryV1) {
            boundary.boundary_hash = hash_omitting(
                "substrate.e3.gateway-access-boundary.v1",
                "boundary",
                boundary,
                "boundary_hash",
            )
            .unwrap();
        }

        struct PreparedGatewayFixture {
            registrations: [E3ChildCgroupRegistrationV1; 3],
            record: AgentConfigProjectionRecordV1,
            handoff: crate::ConfigProjectionSecretHandoffRevisionV1,
            gateway: crate::InWorldGatewayIdentityV1,
            boundary: GatewayAccessBoundaryV1,
            intent: crate::ManagedGatewayActivationIntentV1,
            input: crate::ManagedGatewayLaunchInputV1,
        }

        impl PreparedGatewayFixture {
            fn chain(&self) -> PreparedGatewayChainV1<'_> {
                (
                    &self.handoff,
                    &self.gateway,
                    &self.boundary,
                    &self.intent,
                    &self.input,
                    &self.registrations,
                )
            }

            fn new(
                registry: &ConfigProjectionRegistryV1,
                store: &ConfigProjectionStoreV1,
                predecessor: Option<&AgentConfigProjectionRecordV1>,
            ) -> Self {
                Self::new_at(
                    registry,
                    store,
                    predecessor,
                    Timestamp("2026-09-10T00:00:00.000000Z".into()),
                )
            }

            fn new_at(
                registry: &ConfigProjectionRegistryV1,
                store: &ConfigProjectionStoreV1,
                predecessor: Option<&AgentConfigProjectionRecordV1>,
                created_at: Timestamp,
            ) -> Self {
                let mut record = test_record(store, id("cps_"));
                record.created_at = created_at;
                if let Some(predecessor) = predecessor {
                    record.identity = predecessor.identity.clone();
                    record.effective.accepted_policy = record.identity.immutable_launch_cap.clone();
                    record.revision = predecessor.revision.checked_add(1).unwrap();
                    record.predecessor_ref = Some(record_ref(predecessor));
                    seal_record(&mut record);
                }
                let crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } =
                    &record.activation.publication_fence
                else {
                    unreachable!()
                };
                let fence_id = fence_id.clone();
                let root = crate::NativeProjectionRootV1 {
                    root_id: id("cnr_"),
                    authority_relative_path: format!(
                        "authority-v1/agent-config-projection-v1/native-sources/{}/{}",
                        record.identity.series_id, fence_id
                    ),
                    guest_absolute_path: format!(
                        "/run/substrate/member-config/{}/{}",
                        record.identity.series_id, fence_id
                    ),
                    owner_uid: u64::from(unsafe { libc::geteuid() }),
                    owner_gid: u64::from(unsafe { libc::getegid() }),
                    directory_mode: 0o700,
                };
                record.effective.environment = crate::EffectiveEnvironmentV1 {
                    inherited_names: Vec::new(),
                    set: [
                        ("CODEX_HOME", format!("{}/codex-home", root.guest_absolute_path)),
                        ("CODEX_SQLITE_HOME", format!("{}/state/sqlite", root.guest_absolute_path)),
                        ("HOME", format!("{}/home", root.guest_absolute_path)),
                        ("LANG", "C.UTF-8".to_string()),
                        ("LC_ALL", "C.UTF-8".to_string()),
                        ("PATH", "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()),
                        ("RUST_LOG", "error".to_string()),
                        ("TMPDIR", format!("{}/tmp", root.guest_absolute_path)),
                    ].into_iter().map(|(name, value)| crate::NamedValueV1 { name: name.to_string(), value }).collect(),
                    remove: ["ANTHROPIC_API_KEY", "CODEX_API_KEY", "CODEX_BINARY", "CODEX_OSS_BASE_URL", "CODEX_OSS_PORT", "OPENAI_ACCESS_TOKEN", "OPENAI_API_KEY", "OPENAI_BASE_URL", "OPENAI_ORGANIZATION", "OPENAI_PROJECT", "SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD", "SUBSTRATE_E3_NATIVE_REALIZATION_FD", "SUBSTRATE_E3_NATIVE_SOURCE_FD", "SUBSTRATE_E3_SYSTEM_EMPTY_FD", "SUBSTRATE_E3_WORLD_FS_INPUT_FD", "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME", "SUBSTRATE_LLM_AUTH_BUNDLE_FD", "SUBSTRATE_WORLD_ENTRY_BINARY", "SUBSTRATE_WORLD_ENTRY_BINARY_FD", "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_FD", "SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH", "SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD", "SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH", "SUBSTRATE_WORLD_ENTRY_ROLE", "SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD", "SUBSTRATE_WORLD_ENTRY_WORKING_DIR", "SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD"].into_iter().map(str::to_string).collect(),
                };
                seal_record(&mut record);
                let mut boundary = dormant_boundary(&record);
                let preparation_id = id("e3p_");
                let boot_id = Uuid::now_v7().to_string();
                let registrations = [
                    crate::E3TerminalProcessRoleV1::ManagedGateway,
                    crate::E3TerminalProcessRoleV1::ReadinessProbe,
                    crate::E3TerminalProcessRoleV1::Codex,
                ]
                .map(|role| {
                    let registration_id = id("ecg_");
                    let parent = crate::CanonicalCgroupIdentityV1 {
                        cgroup_v2_mount_device_id: 1,
                        cgroup_v2_mount_inode: 2,
                        cgroup_directory_inode: 11,
                        cgroup_relative_path: format!("substrate/{}", record.identity.world_id),
                    };
                    let component = format!(
                        "substrate-e3-{}",
                        &ordinary_sha256(registration_id.as_bytes())[..24]
                    );
                    let relative = format!("{}/{}", parent.cgroup_relative_path, component);
                    let mut effect = E3KernelEffectIntentV1 {
                        schema_version: 1,
                        authority_store_id: store.authority_store_id.clone(),
                        series_id: record.identity.series_id.clone(),
                        effect_intent_id: id("eki_"),
                        preparation_id: preparation_id.clone(),
                        fence_id: fence_id.clone(),
                        created_at: record.created_at.clone(),
                        intent_hash: String::new(),
                        effect: E3KernelEffectKindV1::CreateChildCgroup {
                            cgroup_registration_id: registration_id.clone(),
                            role,
                            parent_cgroup: parent.clone(),
                            child_component: component,
                            expected_relative_path: relative.clone(),
                        },
                    };
                    effect.intent_hash = hash_omitting(
                        "substrate.e3.kernel-effect-intent.v1",
                        "intent",
                        &effect,
                        "intent_hash",
                    )
                    .unwrap();
                    let reference = registry
                        .publish_kernel_effect_intent(&effect, None, None)
                        .unwrap();
                    let mut registration = E3ChildCgroupRegistrationV1 {
                        schema_version: 1,
                        authority_store_id: store.authority_store_id.clone(),
                        series_id: record.identity.series_id.clone(),
                        cgroup_registration_id: registration_id,
                        kernel_effect_intent_ref: reference,
                        fence_id: fence_id.clone(),
                        turn_id: None,
                        role,
                        cgroup: crate::CanonicalCgroupIdentityV1 {
                            cgroup_directory_inode: match role {
                                crate::E3TerminalProcessRoleV1::ManagedGateway => 12,
                                crate::E3TerminalProcessRoleV1::ReadinessProbe => 13,
                                crate::E3TerminalProcessRoleV1::Codex => 14,
                            },
                            cgroup_relative_path: relative,
                            ..parent
                        },
                        kernel_boot_id: boot_id.clone(),
                        registered_at: record.created_at.clone(),
                        cgroup_registration_hash: String::new(),
                    };
                    registration.cgroup_registration_hash = hash_omitting(
                        "substrate.e3.child-cgroup-registration.v1",
                        "cgroup_registration",
                        &registration,
                        "cgroup_registration_hash",
                    )
                    .unwrap();
                    registry
                        .publish_child_cgroup_registration(&registration)
                        .unwrap();
                    registration
                });
                boundary.readiness_probe_cgroup = registrations[1].cgroup.clone();
                boundary.allowed_member_cgroup = registrations[2].cgroup.clone();
                let mut effect = E3KernelEffectIntentV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    series_id: record.identity.series_id.clone(),
                    effect_intent_id: id("eki_"),
                    preparation_id: preparation_id.clone(),
                    fence_id: fence_id.clone(),
                    created_at: record.created_at.clone(),
                    intent_hash: String::new(),
                    effect: E3KernelEffectKindV1::InstallGatewayBoundary {
                        access_boundary_id: boundary.access_boundary_id.clone(),
                        network_namespace_inode: boundary.gateway_listener.network_namespace_inode,
                        table_name: boundary.nftables_chain.table.clone(),
                        chain_name: boundary.nftables_chain.chain.clone(),
                    },
                };
                effect.intent_hash = hash_omitting(
                    "substrate.e3.kernel-effect-intent.v1",
                    "intent",
                    &effect,
                    "intent_hash",
                )
                .unwrap();
                boundary.kernel_effect_intent_ref = registry
                    .publish_kernel_effect_intent(&effect, None, None)
                    .unwrap();
                seal_boundary(&mut boundary);
                let mut gateway = crate::InWorldGatewayIdentityV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    gateway_instance_id: boundary.gateway_instance_id.clone(),
                    config_projection_identity_hash: record.identity.identity_hash.clone(),
                    orchestration_session_id: record.identity.orchestration_session_id.clone(),
                    retained_participant_id: record.identity.retained_participant_id.clone(),
                    backend_id: record.identity.backend_id.clone(),
                    world_id: record.identity.world_id.clone(),
                    world_generation: record.identity.world_generation,
                    gateway_artifact_sha256: record
                        .identity
                        .runtime_artifacts
                        .managed_gateway
                        .sha256
                        .clone(),
                    access_boundary_id: boundary.access_boundary_id.clone(),
                    gateway_identity_hash: String::new(),
                };
                gateway.gateway_identity_hash = hash_omitting(
                    "substrate.e3.in-world-gateway-identity.v1",
                    "gateway",
                    &gateway,
                    "gateway_identity_hash",
                )
                .unwrap();
                registry
                    .publish_kernel_effect_intent(&effect, None, Some(&gateway))
                    .unwrap();
                let gateway_ref = crate::InWorldGatewayRefV1 {
                    authority_store_id: store.authority_store_id.clone(),
                    gateway_instance_id: gateway.gateway_instance_id.clone(),
                    gateway_identity_hash: gateway.gateway_identity_hash.clone(),
                };
                let credential = &mut record.nonsecret_handoff.credential_source_ref;
                credential.credential_source_id = id("crs_");
                credential.preparation_id = effect.preparation_id.clone();
                credential.issued_at = record.created_at.clone();
                credential.expires_at = Timestamp(
                    (chrono::DateTime::parse_from_rfc3339(&record.created_at.0).unwrap()
                        + chrono::Duration::seconds(120))
                    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                );
                credential.ref_hash = hash_omitting(
                    "substrate.e3.credential-source-ref.v1",
                    "credential_source",
                    credential,
                    "ref_hash",
                )
                .unwrap();
                let mut handoff = crate::ConfigProjectionSecretHandoffRevisionV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    predecessor_ref: None,
                    revision_hash: String::new(),
                    handoff: crate::LaunchTimeSecretHandoffV1 {
                        schema_version: 1,
                        handoff_id: record
                            .nonsecret_handoff
                            .secret_handoff_ref
                            .handoff_id
                            .clone(),
                        orchestration_session_id: record.identity.orchestration_session_id.clone(),
                        world_id: record.identity.world_id.clone(),
                        world_generation: record.identity.world_generation,
                        retained_participant_id: Some(
                            record.identity.retained_participant_id.clone(),
                        ),
                        runtime_family: "codex".to_owned(),
                        credential_source_ref: credential.clone(),
                        receiving_gateway_ref: gateway_ref.clone(),
                        delivery: record.nonsecret_handoff.delivery.clone(),
                        created_at: credential.issued_at.clone(),
                        expires_at: credential.expires_at.clone(),
                        delivered_at: None,
                        consumed_at: None,
                        state_revision: 1,
                        state: SecretHandoffStateV1::Prepared,
                        failure_diagnostic_ref: None,
                    },
                };
                handoff.revision_hash = hash_omitting(
                    "substrate.e3.config-projection-secret-handoff-revision.v1",
                    "revision",
                    &handoff,
                    "revision_hash",
                )
                .unwrap();
                let mut intent = crate::ManagedGatewayActivationIntentV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    preparation_id: effect.preparation_id,
                    activation_intent_id: record
                        .activation
                        .activation_intent_ref
                        .activation_intent_id
                        .clone(),
                    config_projection_identity_hash: record.identity.identity_hash.clone(),
                    dormant_record_id: record.record_id.clone(),
                    dormant_revision: record.revision,
                    expected_gateway_artifact: record
                        .identity
                        .runtime_artifacts
                        .managed_gateway
                        .clone(),
                    expected_gateway_ref: gateway_ref.clone(),
                    expected_access_boundary_ref: boundary_ref(&boundary),
                    secret_handoff_ref: handoff_reference(&handoff).unwrap(),
                    fence_id: fence_id.clone(),
                    readiness_nonce: Uuid::now_v7().to_string(),
                    created_at: record.created_at.clone(),
                    intent_hash: String::new(),
                };
                intent.intent_hash = hash_omitting(
                    "substrate.e3.managed-gateway-activation-intent.v1",
                    "intent",
                    &intent,
                    "intent_hash",
                )
                .unwrap();
                let intent_ref = crate::ManagedGatewayActivationIntentRefV1 {
                    authority_store_id: store.authority_store_id.clone(),
                    activation_intent_id: intent.activation_intent_id.clone(),
                    intent_hash: intent.intent_hash.clone(),
                };
                record.activation.activation_intent_ref = intent_ref.clone();
                record.effective.provider.gateway_intent_ref = intent_ref.clone();
                record.managed_gateway.activation_intent_ref = intent_ref.clone();
                record.managed_gateway.expected_gateway_ref = gateway_ref.clone();
                record.managed_gateway.access_boundary_ref = boundary_ref(&boundary);
                record.managed_gateway.codex_base_url =
                    format!("http://127.0.0.1:{}/v1", boundary.gateway_listener.port);
                record.nonsecret_handoff.secret_handoff_ref = handoff_reference(&handoff).unwrap();
                record.nonsecret_handoff.receiving_gateway_ref = gateway_ref.clone();
                let project_inputs = [".codex/config.toml", ".codex/rules", ".codex/skills"]
                    .into_iter()
                    .map(|relative_path| crate::CodexLoaderInputAttestationV1 {
                        layer: "Project".to_string(),
                        locator: format!(
                            "{}/{}",
                            record.identity.workspace_root.physical_path, relative_path
                        ),
                        disposition: crate::CodexLoaderInputDispositionV1::DisabledByTrust,
                        directory: record.identity.workspace_root.clone(),
                        relative_path: relative_path.to_string(),
                        device_id: None,
                        inode: None,
                        byte_length: None,
                        sha256: None,
                    })
                    .collect();
                let plan = Codex0125ProjectionV1::render(
                    &record.identity,
                    &record.effective,
                    &record.managed_gateway,
                    root,
                    &fence_id,
                    project_inputs,
                )
                .unwrap();
                let (native, _) = registry
                    .publish_native_source(
                        &record.identity.series_id,
                        &fence_id,
                        &plan,
                        record.created_at.clone(),
                    )
                    .unwrap();
                record.native = native;
                seal_record(&mut record);
                let mut input = crate::ManagedGatewayLaunchInputV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    launch_input_id: id("gal_"),
                    activation_intent_ref: intent_ref,
                    dormant_projection_ref: record_ref(&record),
                    gateway_ref,
                    config_projection_identity_hash: record.identity.identity_hash.clone(),
                    orchestration_session_id: record.identity.orchestration_session_id.clone(),
                    retained_participant_id: record.identity.retained_participant_id.clone(),
                    backend_id: record.identity.backend_id.clone(),
                    world_id: record.identity.world_id.clone(),
                    world_generation: record.identity.world_generation,
                    listener_identity: boundary.gateway_listener.clone(),
                    gateway_config: crate::GatewayRuntimeConfigIdentityV1 {
                        root: CanonicalDirectoryV1 {
                            physical_path: format!(
                                "/run/substrate/e3-gateway/{}/{fence_id}",
                                record.identity.series_id
                            ),
                            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                                device_id: 1,
                                inode: 2,
                            },
                        },
                        relative_path: "config.toml".to_owned(),
                        mode: 0o600,
                        byte_length: 1,
                        sha256: digest(),
                    },
                    http_surface: crate::GatewayHttpSurfaceV1 {
                        inherited_listener_only: true,
                        readiness_method: "GET".to_owned(),
                        readiness_path: "/health".to_owned(),
                        member_method: "POST".to_owned(),
                        member_path: "/v1/responses".to_owned(),
                        auxiliary_listener_count: 0,
                    },
                    access_boundary_ref: boundary_ref(&boundary),
                    secret_handoff_prepared_ref: handoff_reference(&handoff).unwrap(),
                    readiness_nonce: intent.readiness_nonce.clone(),
                    launch_input_hash: String::new(),
                };
                input.launch_input_hash = hash_omitting(
                    "substrate.e3.managed-gateway-launch-input.v1",
                    "launch_input",
                    &input,
                    "launch_input_hash",
                )
                .unwrap();
                Self {
                    registrations,
                    record,
                    handoff,
                    gateway,
                    boundary,
                    intent,
                    input,
                }
            }
        }

        fn terminal_handoff_successor(
            prepared: &crate::ConfigProjectionSecretHandoffRevisionV1,
            state: SecretHandoffStateV1,
        ) -> crate::ConfigProjectionSecretHandoffRevisionV1 {
            assert!(matches!(
                state,
                SecretHandoffStateV1::Failed | SecretHandoffStateV1::Expired
            ));
            let mut successor = prepared.clone();
            successor.predecessor_ref = Some(handoff_reference(prepared).unwrap());
            successor.handoff.state_revision = 2;
            successor.handoff.state = state;
            successor.revision_hash = hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                &successor,
                "revision_hash",
            )
            .unwrap();
            successor
        }

        fn delivered_handoff_successor(
            prepared: &crate::ConfigProjectionSecretHandoffRevisionV1,
        ) -> crate::ConfigProjectionSecretHandoffRevisionV1 {
            let mut successor = prepared.clone();
            successor.predecessor_ref = Some(handoff_reference(prepared).unwrap());
            successor.handoff.state_revision = 2;
            successor.handoff.state = SecretHandoffStateV1::Delivered;
            successor.handoff.delivered_at = Some(prepared.handoff.created_at.clone());
            successor.revision_hash = hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                &successor,
                "revision_hash",
            )
            .unwrap();
            successor
        }

        fn consumed_handoff_successor(
            delivered: &crate::ConfigProjectionSecretHandoffRevisionV1,
        ) -> crate::ConfigProjectionSecretHandoffRevisionV1 {
            let mut successor = delivered.clone();
            successor.predecessor_ref = Some(handoff_reference(delivered).unwrap());
            successor.handoff.state_revision = 3;
            successor.handoff.state = SecretHandoffStateV1::Consumed;
            successor.handoff.consumed_at = successor.handoff.delivered_at.clone();
            successor.revision_hash = hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                &successor,
                "revision_hash",
            )
            .unwrap();
            successor
        }

        #[test]
        fn e3_e_launch_input_conflict_cannot_publish_dormant_head() {
            use std::os::unix::fs::OpenOptionsExt;

            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let root = parent.authority.join(CHILD_NAME);
            let inputs = root.join("gateway-launch-inputs");
            // Another immutable object occupying this exact name must stop publication before
            // an independently visible projection head can grant a dispatch-consumer lease.
            let path = inputs.join(format!("{}.json", fixture.input.launch_input_id));
            let mut occupied = fixture.input.clone();
            occupied.readiness_nonce = uuid::Uuid::now_v7().to_string();
            occupied.launch_input_hash = hash_omitting(
                "substrate.e3.managed-gateway-launch-input.v1",
                "launch_input",
                &occupied,
                "launch_input_hash",
            )
            .unwrap();
            let bytes = ConfigProjectionCodecV1::encode_canonical_json(&occupied).unwrap();
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
                .unwrap();
            std::io::Write::write_all(&mut file, &bytes).unwrap();
            file.sync_all().unwrap();
            assert!(registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .is_err());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            assert!(!root
                .join("series")
                .join(&fixture.record.identity.series_id)
                .join("head.json")
                .exists());
        }

        #[test]
        fn e3_e_publication_carrier_binds_once_and_keeps_private_handoff_exact() {
            let (_temp, _parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let make = || {
                crate::E3PreparedRetainedLaunchPublicationV1::new(
                    &fixture.record.identity,
                    &fixture.gateway,
                    &fixture.handoff.handoff.credential_source_ref,
                    fixture.handoff.handoff.handoff_id.clone(),
                    format!("e3pik_{}", "a".repeat(64)),
                    fixture.handoff.handoff.created_at.clone(),
                    fixture.handoff.handoff.expires_at.clone(),
                )
            };
            let mut publication = make().unwrap();
            assert_eq!(
                publication.prepared_handoff_ref(),
                &handoff_reference(&fixture.handoff).unwrap()
            );
            assert!(
                publication.published_head().is_none()
                    && publication.held_consumer_lease().is_none()
            );
            let mut wrong = fixture.boundary.clone();
            wrong.world_generation += 1;
            seal_boundary(&mut wrong);
            assert_eq!(
                publication.bind_prepared_chain(
                    fixture.record.clone(),
                    fixture.registrations.clone(),
                    wrong,
                    fixture.intent.clone(),
                    fixture.input.clone()
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            publication
                .bind_prepared_chain(
                    fixture.record.clone(),
                    fixture.registrations.clone(),
                    fixture.boundary.clone(),
                    fixture.intent.clone(),
                    fixture.input.clone(),
                )
                .unwrap();
            assert_eq!(
                publication.bind_prepared_chain(
                    fixture.record.clone(),
                    fixture.registrations.clone(),
                    fixture.boundary.clone(),
                    fixture.intent.clone(),
                    fixture.input.clone()
                ),
                Err(ConfigProjectionFailureV1::Conflict)
            );
            let mut invalid_hash = fixture.record.clone();
            invalid_hash.logical.projection_hash = digest();
            assert_eq!(
                make().unwrap().bind_prepared_chain(
                    invalid_hash,
                    fixture.registrations.clone(),
                    fixture.boundary.clone(),
                    fixture.intent.clone(),
                    fixture.input.clone()
                ),
                Err(ConfigProjectionFailureV1::HashInvalid)
            );
            let mut invalid_id = fixture.handoff.handoff.credential_source_ref.clone();
            invalid_id.credential_source_id = id("gcs_");
            assert!(matches!(
                crate::E3PreparedRetainedLaunchPublicationV1::new(
                    &fixture.record.identity,
                    &fixture.gateway,
                    &invalid_id,
                    fixture.handoff.handoff.handoff_id.clone(),
                    format!("e3pik_{}", "a".repeat(64)),
                    fixture.handoff.handoff.created_at.clone(),
                    fixture.handoff.handoff.expires_at.clone(),
                ),
                Err(ConfigProjectionFailureV1::Malformed)
            ));
            assert!(matches!(
                crate::E3PreparedRetainedLaunchPublicationV1::new(
                    &fixture.record.identity,
                    &fixture.gateway,
                    &fixture.handoff.handoff.credential_source_ref,
                    fixture.handoff.handoff.handoff_id.clone(),
                    format!("e3pik_{}", "a".repeat(64)),
                    Timestamp("2026-09-10T00:00:00Z".into()),
                    fixture.handoff.handoff.expires_at.clone(),
                ),
                Err(ConfigProjectionFailureV1::Malformed)
            ));
        }

        #[test]
        fn e3_e_same_subject_readback_preserves_bound_series_without_a_lease() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            assert!(matches!(
                registry.recover(Some(&fixture.record.identity)),
                Err(ConfigProjectionFailureV1::PartialPublication)
            ));
            let reference = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let mut candidate = fixture.record.identity.clone();
            candidate.series_id = id("cps_");
            candidate.identity_hash = hash_omitting(
                "substrate.e3.config-projection-identity.v1",
                "identity",
                &candidate,
                "identity_hash",
            )
            .unwrap();
            let readback = registry.recover(Some(&candidate)).unwrap();
            let Some(ConfigProjectionSubjectReadbackV1::Bound(metadata)) = readback.subject else {
                panic!("bound subject missing")
            };
            assert_eq!(metadata.record(), &fixture.record);
            assert_eq!(metadata.projection_ref(), &reference);
            assert!(metadata.consumer_lease.is_none());
            assert_eq!(metadata.current_handoff, fixture.handoff);
            assert_eq!(readback.kernel_effects.len(), 4);
            let consumer =
                prepared_member_dispatch_consumer_id_v1(&fixture.intent.preparation_id).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &reference,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    fixture.record.created_at.clone(),
                    Some(&consumer),
                )
                .unwrap();
            registry
                .release_consumer_lease(&held, Timestamp("2026-09-10T00:03:00.000000Z".into()))
                .unwrap();
            let readback = registry.recover(Some(&candidate)).unwrap();
            let Some(ConfigProjectionSubjectReadbackV1::Bound(metadata)) = readback.subject else {
                panic!("bound subject missing")
            };
            let released = metadata.consumer_lease.unwrap();
            assert_eq!(
                released.posture,
                ConfigProjectionConsumerLeasePostureV1::Released
            );
            assert!(registry
                .resolve(Some((&fixture.record.identity, &released)), None)
                .is_err());
            let head = parent
                .authority
                .join(CHILD_NAME)
                .join("leases")
                .join(&reference.series_id)
                .join(&consumer)
                .join("head.json");
            std::fs::remove_file(&head).unwrap();
            assert!(registry.recover(Some(&candidate)).is_err());
            assert!(!head.exists());
        }

        #[test]
        fn test_e3_abandonment_requires_expected_head_and_complete_cleanup_before_mutation() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let head = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let terminal =
                terminal_handoff_successor(&fixture.handoff, SecretHandoffStateV1::Failed);
            let path = parent
                .authority
                .join(CHILD_NAME)
                .join("handoffs")
                .join(&fixture.handoff.handoff.handoff_id)
                .join("head.json");
            let before = std::fs::read(&path).unwrap();
            let mut stale = head.clone();
            stale.record_id = id("cpr_");
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &terminal,
                    Some(&stale),
                    None
                ),
                Err(ConfigProjectionFailureV1::StaleRevision)
            );
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &terminal,
                    Some(&head),
                    None
                ),
                Err(ConfigProjectionFailureV1::PartialPublication)
            );
            assert_eq!(std::fs::read(path).unwrap(), before);
            let effect = registry.recover(None).unwrap().kernel_effects.remove(0);
            let mut resolution = E3KernelEffectResolutionV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                resolution_id: id("ekr_"),
                effect_intent_ref: kernel_effect_intent_ref(&effect.intent),
                disposition: E3KernelEffectResolutionDispositionV1::NoEffectObserved,
                observed_cgroup: None,
                observed_nftables_table_handle: None,
                resolved_at: now_timestamp(),
                resolution_hash: String::new(),
            };
            resolution.resolution_hash = hash_omitting(
                "substrate.e3.kernel-effect-resolution.v1",
                "resolution",
                &resolution,
                "resolution_hash",
            )
            .unwrap();
            for expected in [None, Some(&stale)] {
                assert_eq!(
                    registry.publish_kernel_effect_resolution(
                        &resolution,
                        Some(&mut || panic!("stale cleanup callback ran")),
                        expected
                    ),
                    Err(ConfigProjectionFailureV1::StaleRevision)
                );
            }
        }

        #[test]
        fn test_e3_recovery_rejects_missing_competing_and_wrong_bound_launch_inputs() {
            for fault in [
                "missing",
                "missing-intent-and-input",
                "competing",
                "wrong-root",
                "wrong-listener",
            ] {
                let (_temp, parent, registry, store) = test_registry();
                let fixture = PreparedGatewayFixture::new(&registry, &store, None);
                registry
                    .publish_dormant(&fixture.record, Some(fixture.chain()))
                    .unwrap();
                let directory = parent
                    .authority
                    .join(CHILD_NAME)
                    .join("gateway-launch-inputs");
                let original = directory.join(format!("{}.json", fixture.input.launch_input_id));
                if matches!(fault, "missing" | "missing-intent-and-input") {
                    std::fs::remove_file(original).unwrap();
                    if fault == "missing-intent-and-input" {
                        std::fs::remove_file(
                            parent
                                .authority
                                .join(CHILD_NAME)
                                .join("gateway-intents")
                                .join(format!("{}.json", fixture.intent.activation_intent_id)),
                        )
                        .unwrap();
                    }
                } else {
                    let mut input = fixture.input.clone();
                    match fault {
                        "competing" => input.launch_input_id = id("gal_"),
                        "wrong-root" => input
                            .gateway_config
                            .root
                            .physical_path
                            .push_str("-substituted"),
                        "wrong-listener" => input.listener_identity.socket_inode += 1,
                        _ => unreachable!(),
                    }
                    input.launch_input_hash = hash_omitting(
                        "substrate.e3.managed-gateway-launch-input.v1",
                        "launch_input",
                        &input,
                        "launch_input_hash",
                    )
                    .unwrap();
                    let path = directory.join(format!("{}.json", input.launch_input_id));
                    std::fs::write(
                        &path,
                        ConfigProjectionCodecV1::encode_canonical_json(&input).unwrap(),
                    )
                    .unwrap();
                    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
                }
                assert!(registry.recover(None).is_err(), "accepted {fault}");
                assert!(
                    registry.recover(Some(&fixture.record.identity)).is_err(),
                    "subject filter hid {fault}"
                );
            }
        }

        // Immutable synthetic history only: this fixture does not execute a gateway or
        // demonstrate installed privilege, listener, readiness or E3-F lifecycle behavior.
        fn ready_gateway_fixture_v1(
            registry: &ConfigProjectionRegistryV1,
            fixture: &PreparedGatewayFixture,
        ) -> (
            AgentConfigProjectionRecordV1,
            crate::ManagedGatewayActivationAckV1,
            ConfigProjectionConsumerLeaseV1,
        ) {
            let dormant = record_ref(&fixture.record);
            let lease = registry
                .acquire_consumer_lease(
                    &dormant,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    fixture.record.created_at.clone(),
                    Some(
                        &prepared_member_dispatch_consumer_id_v1(&fixture.intent.preparation_id)
                            .unwrap(),
                    ),
                )
                .unwrap();
            let delivered = delivered_handoff_successor(&fixture.handoff);
            let consumed = consumed_handoff_successor(&delivered);
            for handoff in [&delivered, &consumed] {
                registry
                    .publish_handoff_transition_v1(
                        &fixture.record.identity,
                        &fixture.intent.fence_id,
                        handoff,
                        Some(&dormant),
                        Some(&lease),
                    )
                    .unwrap();
            }
            let group = &fixture.registrations[0];
            let mut registration = E3ChildProcessRegistrationV1 {
                schema_version: 1,
                authority_store_id: fixture.record.identity.authority_store_id.clone(),
                series_id: fixture.record.identity.series_id.clone(),
                registration_id: id("ecp_"),
                cgroup_registration_id: group.cgroup_registration_id.clone(),
                cgroup_registration_hash: group.cgroup_registration_hash.clone(),
                fence_id: fixture.intent.fence_id.clone(),
                role: crate::E3TerminalProcessRoleV1::ManagedGateway,
                pid: 321,
                pid_start_time_ticks: 123,
                process_cgroup: group.cgroup.clone(),
                kernel_boot_id: group.kernel_boot_id.clone(),
                parent_service_instance_id: id("wsi_"),
                registered_at: fixture.record.created_at.clone(),
                registration_hash: String::new(),
            };
            registration.registration_hash = hash_omitting(
                "substrate.e3.child-process-registration.v1",
                "registration",
                &registration,
                "registration_hash",
            )
            .unwrap();
            registry
                .publish_child_process_registration(&registration)
                .unwrap();
            let identity = &fixture.record.identity;
            let mut security = crate::E3ChildSecurityAttestationV1 {
                schema_version: 1,
                child_role: crate::E3IsolatedChildRoleV1::ManagedGateway,
                projection_identity_hash: identity.identity_hash.clone(),
                pid: registration.pid,
                pid_start_time_ticks: registration.pid_start_time_ticks,
                real_uid: 1000,
                effective_uid: 1000,
                saved_uid: 1000,
                filesystem_uid: 1000,
                real_gid: 1000,
                effective_gid: 1000,
                saved_gid: 1000,
                filesystem_gid: 1000,
                supplementary_group_count: 0,
                cap_inheritable: "0000000000000000".into(),
                cap_permitted: "0000000000000000".into(),
                cap_effective: "0000000000000000".into(),
                cap_bounding: "0000000000000000".into(),
                cap_ambient: "0000000000000000".into(),
                cap_last_cap: 40,
                no_new_privs: true,
                dumpable: 0,
                tracer_pid: 0,
                kernel_boot_id: group.kernel_boot_id.clone(),
                user_namespace: crate::E3UserNamespaceAttestationV1 {
                    namespace_device_id: 4,
                    namespace_inode: 5,
                    owner_uid: 0,
                    parent_namespace_device_id: 4,
                    parent_namespace_inode: 6,
                    uid_map: crate::E3LinuxIdMapExtentV1 {
                        inside_id: 1000,
                        outside_id: 1000,
                        length: 1,
                    },
                    gid_map: crate::E3LinuxIdMapExtentV1 {
                        inside_id: 1000,
                        outside_id: 1000,
                        length: 1,
                    },
                },
                seccomp_mode: 2,
                landlock_abi: 6,
                e2_enforcement_plan_hash: digest(),
                derived_support_ruleset_hash: digest(),
                role_narrowing_ruleset_hash: digest(),
                effective_landlock_hash: digest(),
                policy_snapshot_ref: identity.immutable_launch_cap.policy_snapshot_ref.clone(),
                policy_snapshot_hash: identity.immutable_launch_cap.policy_snapshot_hash.clone(),
                policy_snapshot_revision: identity
                    .immutable_launch_cap
                    .policy_snapshot_revision
                    .clone(),
                enforcement_input_hash: digest(),
                denied_control_probe_hash: digest(),
                attestation_hash: String::new(),
            };
            security.attestation_hash = hash_omitting(
                "substrate.e3.child-security-attestation.v1",
                "attestation",
                &security,
                "attestation_hash",
            )
            .unwrap();
            let input = &fixture.input;
            let artifact = &identity.runtime_artifacts.managed_gateway;
            let mut ack = crate::ManagedGatewayActivationAckV1 {
                schema_version: 1,
                authority_store_id: identity.authority_store_id.clone(),
                activation_ack_id: id("gaa_"),
                activation_intent_ref: input.activation_intent_ref.clone(),
                config_projection_identity_hash: identity.identity_hash.clone(),
                dormant_projection_ref: dormant,
                gateway_ref: input.gateway_ref.clone(),
                gateway_process_identity: crate::GatewayProcessIdentityV1 {
                    pid: registration.pid,
                    pid_start_time_ticks: registration.pid_start_time_ticks,
                    pidfd_inode: 99,
                    executable_device_id: artifact.device_id,
                    executable_inode: artifact.inode,
                    executable_sha256: artifact.sha256.clone(),
                    process_cgroup: group.cgroup.clone(),
                    child_security_attestation_hash: security.attestation_hash.clone(),
                    secret_ready_attestation_hash: digest(),
                },
                child_security_attestation: security,
                listener_identity: input.listener_identity.clone(),
                access_boundary_ref: input.access_boundary_ref.clone(),
                secret_handoff_ref: handoff_reference(&consumed).unwrap(),
                secret_handoff_terminal_state: SecretHandoffStateV1::Consumed,
                gateway_ready_revision: 1,
                readiness_nonce: input.readiness_nonce.clone(),
                launch_input_ref: crate::ManagedGatewayLaunchInputRefV1 {
                    authority_store_id: input.authority_store_id.clone(),
                    launch_input_id: input.launch_input_id.clone(),
                    launch_input_hash: input.launch_input_hash.clone(),
                },
                observed_at: consumed.handoff.consumed_at.clone().unwrap(),
                ack_hash: String::new(),
            };
            ack.ack_hash = hash_omitting(
                "substrate.e3.managed-gateway-activation-ack.v1",
                "ack",
                &ack,
                "ack_hash",
            )
            .unwrap();
            let mut ready = successor(
                &fixture.record,
                ManagedGatewayProjectionPostureV1::ReadyClosed,
            );
            let reference = crate::ManagedGatewayActivationAckRefV1 {
                authority_store_id: ack.authority_store_id.clone(),
                activation_ack_id: ack.activation_ack_id.clone(),
                ack_hash: ack.ack_hash.clone(),
            };
            ready.managed_gateway.activation_ack_ref = Some(reference.clone());
            ready.nonsecret_handoff.activation_ack_ref = Some(reference.clone());
            ready.activation.gateway_activation_ack_ref = Some(reference);
            ready.nonsecret_handoff.secret_handoff_ref = ack.secret_handoff_ref.clone();
            ready.created_at = ack.observed_at.clone();
            seal_record(&mut ready);
            (ready, ack, lease)
        }

        #[test]
        fn test_e3_e_ready_closed_rejects_reference_only_ack() {
            let (_temp, _parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let dormant = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let ready = successor(
                &fixture.record,
                ManagedGatewayProjectionPostureV1::ReadyClosed,
            );
            assert!(registry
                .publish(
                    &ready,
                    Some(&dormant),
                    ManagedGatewayProjectionPostureV1::ReadyClosed
                )
                .is_err());
        }

        #[test]
        fn test_e3_e_service_activation_progress() {
            let (temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new_at(
                &registry,
                &store,
                None,
                Timestamp("2026-09-10T00:00:00.000001Z".into()),
            );
            registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let (_, ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
            // Retain immutable successors but restore the Prepared pointer: the service must
            // reconcile the existing bytes and CAS once at each lost-return boundary.
            let handoff = parent
                .authority
                .join(CHILD_NAME)
                .join("handoffs")
                .join(&fixture.handoff.handoff.handoff_id);
            std::fs::write(
                handoff.join("head.json"),
                ConfigProjectionCodecV1::encode_canonical_json(
                    &handoff_reference(&fixture.handoff).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let uid = unsafe { libc::geteuid() };
            let mut password = std::mem::MaybeUninit::<libc::passwd>::uninit();
            let mut buffer = vec![0u8; 16384];
            let mut result = std::ptr::null_mut();
            assert_eq!(
                unsafe {
                    libc::getpwuid_r(
                        uid,
                        password.as_mut_ptr(),
                        buffer.as_mut_ptr().cast(),
                        buffer.len(),
                        &mut result,
                    )
                },
                0
            );
            assert!(!result.is_null());
            let account = unsafe { CStr::from_ptr((*result).pw_name) }
                .to_str()
                .unwrap()
                .to_owned();
            let carrier = transport_api_types::InstallBootstrapContextCarrierV1::from_context(
                transport_api_types::InstallBootstrapContextV1::new_unix(
                    temp.path().to_str().unwrap(),
                    &account,
                    uid,
                )
                .unwrap(),
            )
            .unwrap();
            let mut bootstrap = InstalledAcceptedHomeBootstrapRecordV1 {
                schema_version: 1,
                install_bootstrap_carrier: carrier.encode().unwrap(),
                host_context_commitment: carrier.host_context_commitment,
                intended_account: account,
                intended_uid: u64::from(uid),
                intended_gid: u64::from(unsafe { libc::getegid() }),
                accepted_home: store.accepted_home.clone(),
                installed_at: fixture.record.created_at.clone(),
                record_hash: String::new(),
            };
            bootstrap.record_hash = hash_omitting(
                "substrate.e3.installed-accepted-home-bootstrap.v1",
                "record",
                &bootstrap,
                "record_hash",
            )
            .unwrap();
            let accepted_home = Arc::new(
                ConfiguredAcceptedHomeAuthorityV1::from_record_for_test(bootstrap).unwrap(),
            );
            let registry = Arc::new(registry);
            let service =
                crate::AgentConfigProjectionServiceV1::new(registry.clone(), accepted_home)
                    .unwrap();
            let mut publication = crate::E3PreparedRetainedLaunchPublicationV1::new(
                &fixture.record.identity,
                &fixture.gateway,
                &fixture.handoff.handoff.credential_source_ref,
                fixture.handoff.handoff.handoff_id.clone(),
                format!("e3pik_{}", "a".repeat(64)),
                fixture.handoff.handoff.created_at.clone(),
                fixture.handoff.handoff.expires_at.clone(),
            )
            .unwrap();
            publication
                .bind_prepared_chain(
                    fixture.record.clone(),
                    fixture.registrations.clone(),
                    fixture.boundary.clone(),
                    fixture.intent.clone(),
                    fixture.input.clone(),
                )
                .unwrap();
            let reference = crate::service::exercise_activation_progress_for_test(
                &service,
                publication,
                lease,
                crate::E3ManagedGatewayActivationObservationV1::Ready {
                    gateway_process_identity: ack.gateway_process_identity,
                    child_security_attestation: ack.child_security_attestation,
                    observed_at: ack.observed_at,
                },
            );
            let ConfigProjectionSubjectReadbackV1::Bound(current) = registry
                .recover(Some(&fixture.record.identity))
                .unwrap()
                .subject
                .unwrap()
            else {
                panic!("missing subject")
            };
            assert_eq!(current.projection_ref, reference);
            assert_eq!(
                current.current_handoff.handoff.state,
                SecretHandoffStateV1::Consumed
            );
        }

        #[test]
        fn test_e3_e_ack_original_lease_and_exact_publication_retry() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let dormant = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let (ready, ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
            for fault in ["consumer", "revision", "acquisition", "released", "kind"] {
                let mut wrong = lease.clone();
                match fault {
                    "consumer" => wrong.consumer_id = id("cpc_"),
                    "revision" => wrong.revision += 1,
                    "acquisition" => wrong.acquired_projection_ref = record_ref(&ready),
                    "released" => wrong.posture = ConfigProjectionConsumerLeasePostureV1::Released,
                    "kind" => {
                        wrong.consumer_kind = ConfigProjectionConsumerKindV1::WorldRuntimeAdapterV3
                    }
                    _ => unreachable!(),
                }
                assert!(
                    registry
                        .publish_ready_closed(&dormant, &ready, &ack, &wrong)
                        .is_err(),
                    "{fault}"
                );
            }
            // ACK-before-head interruption. Its immutable object grants no activation.
            registry
                .with_transaction(false, |transaction| {
                    publish_gateway_ack_in_transaction_v1(transaction, &ready, &ack)
                })
                .unwrap();
            let ConfigProjectionSubjectReadbackV1::Bound(before) = registry
                .recover(Some(&fixture.record.identity))
                .unwrap()
                .subject
                .unwrap()
            else {
                panic!("missing subject")
            };
            assert_eq!(before.projection_ref, dormant);
            let path = parent
                .authority
                .join(CHILD_NAME)
                .join("gateway-acks")
                .join(format!("{}.json", ack.activation_ack_id));
            let bytes = std::fs::read(&path).unwrap();
            let reference = registry
                .publish_ready_closed(&dormant, &ready, &ack, &lease)
                .unwrap();
            // Head-before-reply interruption uses exactly the original lease and frozen objects.
            assert_eq!(
                registry
                    .publish_ready_closed(&dormant, &ready, &ack, &lease)
                    .unwrap(),
                reference
            );
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            assert_eq!(lease.acquired_projection_ref, dormant);
            let mut conflict = ack.clone();
            conflict.gateway_process_identity.pidfd_inode += 1;
            conflict.ack_hash = hash_omitting(
                "substrate.e3.managed-gateway-activation-ack.v1",
                "ack",
                &conflict,
                "ack_hash",
            )
            .unwrap();
            assert!(registry
                .publish_ready_closed(&dormant, &ready, &conflict, &lease)
                .is_err());
            registry
                .release_consumer_lease(&lease, now_timestamp())
                .unwrap();
            assert!(registry
                .publish_ready_closed(&dormant, &ready, &ack, &lease)
                .is_err());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }

        #[test]
        fn test_e3_e_ack_wrong_joins_reject_before_head() {
            for fault in [
                "store",
                "identity",
                "dormant",
                "intent",
                "gateway",
                "listener",
                "boundary",
                "handoff",
                "state",
                "revision",
                "nonce",
                "launch",
                "process",
                "start",
                "artifact",
                "cgroup",
                "security",
                "secret-ready",
                "observed",
            ] {
                let (_temp, _parent, registry, store) = test_registry();
                let fixture = PreparedGatewayFixture::new(&registry, &store, None);
                let dormant = registry
                    .publish_dormant(&fixture.record, Some(fixture.chain()))
                    .unwrap();
                let (mut ready, mut ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
                match fault {
                    "store" => ack.authority_store_id = id("cpa_"),
                    "identity" => ack.config_projection_identity_hash = "ff".repeat(32),
                    "dormant" => ack.dormant_projection_ref.record_hash = "ff".repeat(32),
                    "intent" => ack.activation_intent_ref.intent_hash = "ff".repeat(32),
                    "gateway" => ack.gateway_ref.gateway_identity_hash = "ff".repeat(32),
                    "listener" => ack.listener_identity.socket_inode += 1,
                    "boundary" => ack.access_boundary_ref.boundary_hash = "ff".repeat(32),
                    "handoff" => ack.secret_handoff_ref.handoff_hash = "ff".repeat(32),
                    "state" => ack.secret_handoff_terminal_state = SecretHandoffStateV1::Delivered,
                    "revision" => ack.gateway_ready_revision += 1,
                    "nonce" => ack.readiness_nonce = "ff".repeat(32),
                    "launch" => ack.launch_input_ref.launch_input_hash = "ff".repeat(32),
                    "process" => ack.gateway_process_identity.pid += 1,
                    "start" => ack.gateway_process_identity.pid_start_time_ticks += 1,
                    "artifact" => ack.gateway_process_identity.executable_inode += 1,
                    "cgroup" => {
                        ack.gateway_process_identity
                            .process_cgroup
                            .cgroup_directory_inode += 1
                    }
                    "security" => ack.child_security_attestation.no_new_privs = false,
                    "secret-ready" => ack
                        .gateway_process_identity
                        .secret_ready_attestation_hash
                        .clear(),
                    "observed" => ack.observed_at = Timestamp("2026-09-10T00:00:01.000000Z".into()),
                    _ => unreachable!(),
                }
                ack.ack_hash = hash_omitting(
                    "substrate.e3.managed-gateway-activation-ack.v1",
                    "ack",
                    &ack,
                    "ack_hash",
                )
                .unwrap();
                let reference = crate::ManagedGatewayActivationAckRefV1 {
                    authority_store_id: ack.authority_store_id.clone(),
                    activation_ack_id: ack.activation_ack_id.clone(),
                    ack_hash: ack.ack_hash.clone(),
                };
                ready.managed_gateway.activation_ack_ref = Some(reference.clone());
                ready.nonsecret_handoff.activation_ack_ref = Some(reference.clone());
                ready.activation.gateway_activation_ack_ref = Some(reference);
                seal_record(&mut ready);
                assert!(
                    registry
                        .publish_ready_closed(&dormant, &ready, &ack, &lease)
                        .is_err(),
                    "accepted {fault}"
                );
                let ConfigProjectionSubjectReadbackV1::Bound(current) = registry
                    .recover(Some(&fixture.record.identity))
                    .unwrap()
                    .subject
                    .unwrap()
                else {
                    panic!("missing subject")
                };
                assert_eq!(current.projection_ref, dormant, "advanced for {fault}");
            }
        }

        #[test]
        fn test_e3_e_ack_dependency_corruption_rejects_all_durable_readers() {
            use std::os::unix::fs::OpenOptionsExt;
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let dormant = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let (ready, ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
            registry
                .publish_ready_closed(&dormant, &ready, &ack, &lease)
                .unwrap();
            let path = parent
                .authority
                .join(CHILD_NAME)
                .join("gateway-acks")
                .join(format!("{}.json", ack.activation_ack_id));
            let bytes = std::fs::read(&path).unwrap();
            std::fs::remove_file(&path).unwrap();
            assert!(registry.recover(None).is_err());
            assert!(registry.recover(Some(&fixture.record.identity)).is_err());
            assert!(matches!(
                registry.resolve(Some((&fixture.record.identity, &lease)), None),
                Err(ConfigProjectionFailureV1::MissingPreparation)
            ));
            assert!(registry
                .with_transaction(false, |transaction| validate_recovery_candidate(
                    transaction,
                    &[
                        "series".into(),
                        ready.identity.series_id.clone(),
                        "records".into()
                    ],
                    "record",
                    &ConfigProjectionCodecV1::encode_canonical_json(&ready)?
                ))
                .is_err());
            // Exact immutable ACK temporary recovery is permitted; it never changes the head.
            let temporary = path.parent().unwrap().join(temp_name("gateway-ack"));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)
                .unwrap();
            file.write_all(&bytes).unwrap();
            file.sync_all().unwrap();
            drop(file);
            let ConfigProjectionSubjectReadbackV1::Bound(current) = registry
                .recover(Some(&fixture.record.identity))
                .unwrap()
                .subject
                .unwrap()
            else {
                panic!("missing subject")
            };
            assert_eq!(current.projection_ref, record_ref(&ready));
            assert_eq!(std::fs::read(path).unwrap(), bytes);
            assert!(!temporary.exists());
        }

        #[test]
        fn test_e3_e_activation_handoff_requires_original_guard() {
            let (_temp, _parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let dormant = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let lease = registry
                .acquire_consumer_lease(
                    &dormant,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    fixture.record.created_at.clone(),
                    Some(
                        &prepared_member_dispatch_consumer_id_v1(&fixture.intent.preparation_id)
                            .unwrap(),
                    ),
                )
                .unwrap();
            let delivered = delivered_handoff_successor(&fixture.handoff);
            let consumed = consumed_handoff_successor(&delivered);
            for successor in [&delivered, &consumed] {
                for (head, held) in [(None, None), (Some(&dormant), None), (None, Some(&lease))] {
                    assert_eq!(
                        registry.publish_handoff_transition_v1(
                            &fixture.record.identity,
                            &fixture.intent.fence_id,
                            successor,
                            head,
                            held
                        ),
                        Err(ConfigProjectionFailureV1::WrongBinding)
                    );
                }
                let reference = registry
                    .publish_handoff_transition_v1(
                        &fixture.record.identity,
                        &fixture.intent.fence_id,
                        successor,
                        Some(&dormant),
                        Some(&lease),
                    )
                    .unwrap();
                // Discarding a successful return cannot allocate another revision or timestamp.
                assert_eq!(
                    registry
                        .publish_handoff_transition_v1(
                            &fixture.record.identity,
                            &fixture.intent.fence_id,
                            successor,
                            Some(&dormant),
                            Some(&lease)
                        )
                        .unwrap(),
                    reference
                );
            }
        }

        #[test]
        fn test_e3_e_consumed_cleanup_successors() {
            use std::os::unix::fs::PermissionsExt;
            for publish_ready in [false, true] {
                let (_temp, parent, registry, store) = test_registry();
                let fixture = PreparedGatewayFixture::new(&registry, &store, None);
                let dormant = registry
                    .publish_dormant(&fixture.record, Some(fixture.chain()))
                    .unwrap();
                let (ready, ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
                let old = if publish_ready {
                    registry
                        .publish_ready_closed(&dormant, &ready, &ack, &lease)
                        .unwrap();
                    &ready
                } else {
                    &fixture.record
                };
                let old_ref = record_ref(old);
                let mut evidence = terminal_evidence(&store, old, &old_ref);
                evidence.ordered_terminal_processes = registry
                    .recover(None)
                    .unwrap()
                    .kernel_effects
                    .into_iter()
                    .flat_map(|effect| effect.child_processes)
                    .map(|child| crate::E3TerminalProcessObservationV1 {
                        registration_id: child.registration_id,
                        registration_hash: child.registration_hash,
                        role: child.role,
                        pid: child.pid,
                        pid_start_time_ticks: child.pid_start_time_ticks,
                        observation: crate::E3TerminalProcessObservationKindV1::ParentWaitid {
                            terminal_wait_status: 0,
                        },
                    })
                    .collect();
                evidence.ordered_empty_cgroups = fixture
                    .registrations
                    .iter()
                    .map(|r| crate::E3TerminalCgroupQuiescenceV1 {
                        cgroup_registration_id: r.cgroup_registration_id.clone(),
                        cgroup_registration_hash: r.cgroup_registration_hash.clone(),
                        role: r.role,
                        cgroup: r.cgroup.clone(),
                        cgroup_events_sha256: ordinary_sha256(b"populated 0\n"),
                        cgroup_procs_sha256: ordinary_sha256(b""),
                        populated: false,
                        ordered_live_pids: Vec::new(),
                    })
                    .collect();
                evidence
                    .ordered_empty_cgroups
                    .sort_by_key(|e| terminal_role_rank(e.role));
                evidence.evidence_hash = hash_omitting(
                    "substrate.e3.terminal-child-quiescence.v1",
                    "evidence",
                    &evidence,
                    "evidence_hash",
                )
                .unwrap();
                registry.publish_terminal_child_evidence(&evidence).unwrap();
                let effects = registry.recover(None).unwrap().kernel_effects;
                for effect in effects {
                    let mut resolution = E3KernelEffectResolutionV1 {
                        schema_version: 1,
                        authority_store_id: store.authority_store_id.clone(),
                        resolution_id: id("ekr_"),
                        effect_intent_ref: kernel_effect_intent_ref(&effect.intent),
                        disposition: E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                        observed_cgroup: effect.child_cgroup.as_ref().map(|r| r.cgroup.clone()),
                        observed_nftables_table_handle: effect.boundary.as_ref().map(|_| 1),
                        resolved_at: now_timestamp(),
                        resolution_hash: String::new(),
                    };
                    resolution.resolution_hash = hash_omitting(
                        "substrate.e3.kernel-effect-resolution.v1",
                        "resolution",
                        &resolution,
                        "resolution_hash",
                    )
                    .unwrap();
                    registry
                        .publish_kernel_effect_resolution(
                            &resolution,
                            Some(&mut || {
                                Ok(effect.boundary.as_ref().map(|b| {
                                    successor_boundary(b, GatewayAccessPostureV1::Revoked)
                                }))
                            }),
                            Some(&old_ref),
                        )
                        .unwrap();
                }
                // A terminal credential is never rewritten to a failed/expired state.
                let handoff_path = parent
                    .authority
                    .join(CHILD_NAME)
                    .join("handoffs")
                    .join(&fixture.handoff.handoff.handoff_id);
                let consumed_bytes =
                    std::fs::read(handoff_path.join("revisions/00000000000000000003.json"))
                        .unwrap();
                let head_bytes = std::fs::read(handoff_path.join("head.json")).unwrap();
                let later = PreparedGatewayFixture::new(&registry, &store, Some(old));
                assert!(
                    registry
                        .publish_dormant(&later.record, Some(later.chain()))
                        .is_err(),
                    "Held original lease permitted successor"
                );
                registry
                    .release_consumer_lease(&lease, now_timestamp())
                    .unwrap();
                // Complete immutable/effect readback must be sufficient in both postures.
                registry
                    .with_transaction(false, |t| {
                        validate_preparation_cleanup_in_transaction_v1(t, old, Some(&later.record))
                    })
                    .expect("cleaned Consumed predecessor permits fresh authorization");
                let root = parent.authority.join(CHILD_NAME);
                let mut paths = vec![
                    handoff_path.join("revisions/00000000000000000002.json"),
                    root.join("terminal-child-evidence")
                        .join(&old.identity.series_id)
                        .join(format!("{}.json", evidence.evidence_id)),
                ];
                // Removing an effect resolution must block even with terminal child evidence.
                paths.push(
                    std::fs::read_dir(root.join("kernel-effects/resolutions"))
                        .unwrap()
                        .next()
                        .unwrap()
                        .unwrap()
                        .path(),
                );
                if publish_ready {
                    paths.push(
                        root.join("gateway-acks")
                            .join(format!("{}.json", ack.activation_ack_id)),
                    );
                }
                for path in paths {
                    let bytes = std::fs::read(&path).unwrap();
                    std::fs::remove_file(&path).unwrap();
                    assert!(
                        registry
                            .publish_dormant(&later.record, Some(later.chain()))
                            .is_err(),
                        "missing dependency accepted: {}",
                        path.display()
                    );
                    std::fs::write(&path, &bytes).unwrap();
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                        .unwrap();
                    let mut wrong: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    wrong["authority_store_id"] = serde_json::Value::String(id("cpa_"));
                    std::fs::write(
                        &path,
                        ConfigProjectionCodecV1::encode_canonical_json(&wrong).unwrap(),
                    )
                    .unwrap();
                    assert!(
                        registry
                            .publish_dormant(&later.record, Some(later.chain()))
                            .is_err(),
                        "wrong-bound dependency accepted: {}",
                        path.display()
                    );
                    std::fs::write(&path, &bytes).unwrap();
                }
                let next = registry
                    .publish_dormant(&later.record, Some(later.chain()))
                    .unwrap();
                assert_eq!(next.revision, old_ref.revision + 1);
                assert_eq!(later.record.identity, old.identity);
                assert_ne!(later.intent.preparation_id, fixture.intent.preparation_id);
                assert_eq!(
                    registry
                        .publish_dormant(&later.record, Some(later.chain()))
                        .unwrap(),
                    next
                );
                let mut contender = later.record.clone();
                contender.record_id = id("cpr_");
                seal_record(&mut contender);
                assert_eq!(
                    registry.publish_dormant(&contender, None),
                    Err(ConfigProjectionFailureV1::StaleRevision)
                );
                let failed =
                    terminal_handoff_successor(&fixture.handoff, SecretHandoffStateV1::Failed);
                assert!(registry
                    .publish_handoff_transition_v1(
                        &old.identity,
                        &fixture.intent.fence_id,
                        &failed,
                        Some(&old_ref),
                        None
                    )
                    .is_err());
                assert_eq!(
                    std::fs::read(handoff_path.join("head.json")).unwrap(),
                    head_bytes
                );
                assert_eq!(
                    std::fs::read(handoff_path.join("revisions/00000000000000000003.json"))
                        .unwrap(),
                    consumed_bytes
                );
            }
        }

        #[test]
        fn test_e3_recovery_uses_originating_dormant_chain_after_head_advances() {
            let (_temp, _parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let dormant = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let (ready, ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
            registry
                .publish_ready_closed(&dormant, &ready, &ack, &lease)
                .unwrap();
            let readback = registry.recover(None).unwrap();
            for effect in readback.kernel_effects {
                assert_eq!(effect.projection.as_ref(), Some(&ready));
                assert_eq!(
                    effect.gateway_config.as_ref(),
                    Some(&fixture.input.gateway_config)
                );
            }
        }

        #[test]
        fn test_e3_recovery_retains_complete_cleanup_readbacks() {
            for publication_mode in ["race", "interrupted"] {
                let (_temp, parent, registry, store) = test_registry();
                let fixture = PreparedGatewayFixture::new(&registry, &store, None);
                let reference = registry
                    .publish_dormant(&fixture.record, Some(fixture.chain()))
                    .unwrap();
                let mut evidence = terminal_evidence(&store, &fixture.record, &reference);
                evidence.ordered_empty_cgroups = fixture
                    .registrations
                    .iter()
                    .map(|registration| crate::E3TerminalCgroupQuiescenceV1 {
                        cgroup_registration_id: registration.cgroup_registration_id.clone(),
                        cgroup_registration_hash: registration.cgroup_registration_hash.clone(),
                        role: registration.role,
                        cgroup: registration.cgroup.clone(),
                        cgroup_events_sha256: ordinary_sha256(b"populated 0\n"),
                        cgroup_procs_sha256: ordinary_sha256(b""),
                        populated: false,
                        ordered_live_pids: Vec::new(),
                    })
                    .collect();
                evidence
                    .ordered_empty_cgroups
                    .sort_by_key(|entry| terminal_role_rank(entry.role));
                evidence.evidence_hash = hash_omitting(
                    "substrate.e3.terminal-child-quiescence.v1",
                    "evidence",
                    &evidence,
                    "evidence_hash",
                )
                .unwrap();
                registry.publish_terminal_child_evidence(&evidence).unwrap();
                for subject in [None, Some(&fixture.record.identity)] {
                    let readback = registry.recover(subject).unwrap();
                    assert_eq!(readback.kernel_effects.len(), 4);
                    for effect in readback.kernel_effects {
                        assert_eq!(effect.terminal_child_evidence, [evidence.clone()]);
                        assert_eq!(
                            effect.gateway_config.as_ref(),
                            Some(&fixture.input.gateway_config)
                        );
                    }
                }
                let mut omitted = evidence.clone();
                omitted.evidence_id = id("tce_");
                omitted.ordered_empty_cgroups.pop();
                omitted.evidence_hash = hash_omitting(
                    "substrate.e3.terminal-child-quiescence.v1",
                    "evidence",
                    &omitted,
                    "evidence_hash",
                )
                .unwrap();
                assert_eq!(
                    registry.publish_terminal_child_evidence(&omitted),
                    Err(ConfigProjectionFailureV1::WrongBinding)
                );
                let consumer =
                    prepared_member_dispatch_consumer_id_v1(&fixture.intent.preparation_id)
                        .unwrap();
                let lease = registry
                    .acquire_consumer_lease(
                        &reference,
                        ConfigProjectionConsumerKindV1::MemberDispatchV2,
                        fixture.record.created_at.clone(),
                        Some(&consumer),
                    )
                    .unwrap();
                for effect in registry.recover(None).unwrap().kernel_effects {
                    let mut resolution = E3KernelEffectResolutionV1 {
                        schema_version: 1,
                        authority_store_id: store.authority_store_id.clone(),
                        resolution_id: id("ekr_"),
                        effect_intent_ref: kernel_effect_intent_ref(&effect.intent),
                        disposition: E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                        observed_cgroup: effect
                            .child_cgroup
                            .as_ref()
                            .map(|registration| registration.cgroup.clone()),
                        observed_nftables_table_handle: effect.boundary.as_ref().map(|_| 1),
                        resolved_at: now_timestamp(),
                        resolution_hash: String::new(),
                    };
                    resolution.resolution_hash = hash_omitting(
                        "substrate.e3.kernel-effect-resolution.v1",
                        "resolution",
                        &resolution,
                        "resolution_hash",
                    )
                    .unwrap();
                    registry
                        .publish_kernel_effect_resolution(
                            &resolution,
                            Some(&mut || {
                                Ok(effect.boundary.as_ref().map(|boundary| {
                                    successor_boundary(boundary, GatewayAccessPostureV1::Revoked)
                                }))
                            }),
                            Some(&reference),
                        )
                        .unwrap();
                }
                let terminal =
                    terminal_handoff_successor(&fixture.handoff, SecretHandoffStateV1::Failed);
                registry
                    .publish_handoff_transition_v1(
                        &fixture.record.identity,
                        &fixture.intent.fence_id,
                        &terminal,
                        Some(&reference),
                        None,
                    )
                    .unwrap();
                registry
                    .release_consumer_lease(&lease, now_timestamp())
                    .unwrap();
                let mut later =
                    PreparedGatewayFixture::new(&registry, &store, Some(&fixture.record));
                assert!(matches!(
                    registry.recover(Some(&fixture.record.identity)),
                    Err(ConfigProjectionFailureV1::PartialPublication)
                ));
                let mut wrong_history = evidence.clone();
                wrong_history.evidence_id = id("tce_");
                wrong_history.ordered_empty_cgroups[0].cgroup_registration_id =
                    later.registrations[0].cgroup_registration_id.clone();
                wrong_history.ordered_empty_cgroups[0].cgroup_registration_hash =
                    later.registrations[0].cgroup_registration_hash.clone();
                wrong_history.ordered_empty_cgroups[0].cgroup =
                    later.registrations[0].cgroup.clone();
                wrong_history.evidence_hash = hash_omitting(
                    "substrate.e3.terminal-child-quiescence.v1",
                    "evidence",
                    &wrong_history,
                    "evidence_hash",
                )
                .unwrap();
                assert_eq!(
                    registry.publish_terminal_child_evidence(&wrong_history),
                    Err(ConfigProjectionFailureV1::WrongBinding)
                );
                assert_eq!(later.record.identity, fixture.record.identity);
                assert_ne!(later.intent.fence_id, fixture.intent.fence_id);
                let readback = registry
                    .recover(None)
                    .expect("later preparation must not invalidate earlier terminal evidence");
                assert_eq!(readback.kernel_effects.len(), 8);
                for effect in readback.kernel_effects {
                    if effect.intent.fence_id == fixture.intent.fence_id {
                        assert_eq!(effect.terminal_child_evidence, [evidence.clone()]);
                    } else {
                        assert_eq!(effect.intent.fence_id, later.intent.fence_id);
                        assert!(effect.resolution.is_none());
                        assert!(effect.terminal_child_evidence.is_empty());
                    }
                }
                registry
                    .with_transaction(false, |transaction| {
                        validate_prepared_gateway_chain(transaction, &later.record, later.chain())
                    })
                    .expect("fresh dependencies join");
                validate_record_transition(&fixture.record, &later.record)
                    .expect("fresh structural edge");
                registry
                    .with_transaction(false, |transaction| {
                        validate_preparation_cleanup_in_transaction_v1(
                            transaction,
                            &fixture.record,
                            Some(&later.record),
                        )
                    })
                    .expect("old cleanup permits the fresh attempt");
                let mut reused_nonce_intent = later.intent.clone();
                reused_nonce_intent.readiness_nonce = fixture.intent.readiness_nonce.clone();
                reused_nonce_intent.intent_hash = hash_omitting(
                    "substrate.e3.managed-gateway-activation-intent.v1",
                    "intent",
                    &reused_nonce_intent,
                    "intent_hash",
                )
                .unwrap();
                let mut reused_nonce_record = later.record.clone();
                reused_nonce_record
                    .managed_gateway
                    .activation_intent_ref
                    .intent_hash = reused_nonce_intent.intent_hash.clone();
                reused_nonce_record.activation.activation_intent_ref = reused_nonce_record
                    .managed_gateway
                    .activation_intent_ref
                    .clone();
                reused_nonce_record.effective.provider.gateway_intent_ref = reused_nonce_record
                    .managed_gateway
                    .activation_intent_ref
                    .clone();
                seal_record(&mut reused_nonce_record);
                let mut reused_nonce_input = later.input.clone();
                reused_nonce_input.readiness_nonce = reused_nonce_intent.readiness_nonce.clone();
                reused_nonce_input.activation_intent_ref = reused_nonce_record
                    .managed_gateway
                    .activation_intent_ref
                    .clone();
                reused_nonce_input.dormant_projection_ref = record_ref(&reused_nonce_record);
                reused_nonce_input.launch_input_hash = hash_omitting(
                    "substrate.e3.managed-gateway-launch-input.v1",
                    "launch_input",
                    &reused_nonce_input,
                    "launch_input_hash",
                )
                .unwrap();
                assert_eq!(
                    registry.publish_dormant(
                        &reused_nonce_record,
                        Some((
                            &later.handoff,
                            &later.gateway,
                            &later.boundary,
                            &reused_nonce_intent,
                            &reused_nonce_input,
                            &later.registrations
                        ))
                    ),
                    Err(ConfigProjectionFailureV1::WrongBinding)
                );
                if publication_mode == "race" {
                    use std::io::{Read, Write};
                    let mut competing = later.record.clone();
                    competing.record_id = id("cpr_");
                    seal_record(&mut competing);
                    let mut competing_intent = later.intent.clone();
                    competing_intent.activation_intent_id = id("gai_");
                    competing_intent.dormant_record_id = competing.record_id.clone();
                    competing_intent.intent_hash = hash_omitting(
                        "substrate.e3.managed-gateway-activation-intent.v1",
                        "intent",
                        &competing_intent,
                        "intent_hash",
                    )
                    .unwrap();
                    competing
                        .managed_gateway
                        .activation_intent_ref
                        .activation_intent_id = competing_intent.activation_intent_id.clone();
                    competing.managed_gateway.activation_intent_ref.intent_hash =
                        competing_intent.intent_hash.clone();
                    competing.activation.activation_intent_ref =
                        competing.managed_gateway.activation_intent_ref.clone();
                    competing.effective.provider.gateway_intent_ref =
                        competing.managed_gateway.activation_intent_ref.clone();
                    seal_record(&mut competing);
                    let mut competing_input = later.input.clone();
                    competing_input.activation_intent_ref =
                        competing.managed_gateway.activation_intent_ref.clone();
                    competing_input.launch_input_id = id("gal_");
                    competing_input.dormant_projection_ref = record_ref(&competing);
                    competing_input.launch_input_hash = hash_omitting(
                        "substrate.e3.managed-gateway-launch-input.v1",
                        "launch_input",
                        &competing_input,
                        "launch_input_hash",
                    )
                    .unwrap();
                    registry
                        .with_transaction(false, |transaction| {
                            validate_prepared_gateway_chain(
                                transaction,
                                &competing,
                                (
                                    &later.handoff,
                                    &later.gateway,
                                    &later.boundary,
                                    &competing_intent,
                                    &competing_input,
                                    &later.registrations,
                                ),
                            )
                        })
                        .expect("both proposals have valid immutable inputs before the race");
                    let mut children = Vec::new();
                    for (record, intent, input) in [
                        (&later.record, &later.intent, &later.input),
                        (&competing, &competing_intent, &competing_input),
                    ] {
                        let (mut parent_start, mut child_start) =
                            std::os::unix::net::UnixStream::pair().unwrap();
                        let pid = unsafe { libc::fork() };
                        assert!(pid >= 0);
                        if pid == 0 {
                            drop(parent_start);
                            let mut go = [0];
                            child_start.read_exact(&mut go).unwrap();
                            let child_registry =
                                ConfigProjectionRegistryV1::open(parent.clone()).unwrap();
                            let result = child_registry.publish_dormant(
                                record,
                                Some((
                                    &later.handoff,
                                    &later.gateway,
                                    &later.boundary,
                                    intent,
                                    input,
                                    &later.registrations,
                                )),
                            );
                            unsafe {
                                libc::_exit(match result {
                                    Ok(_) => 0,
                                    Err(ConfigProjectionFailureV1::StaleRevision) => 1,
                                    Err(error) => {
                                        eprintln!("fresh contender failed: {error:?}");
                                        2
                                    }
                                })
                            };
                        }
                        drop(child_start);
                        parent_start.write_all(b"G").unwrap();
                        children.push(pid);
                    }
                    let mut codes = Vec::new();
                    for pid in children {
                        let mut status = 0;
                        assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
                        assert!(libc::WIFEXITED(status));
                        codes.push(libc::WEXITSTATUS(status));
                    }
                    assert!(
                        codes == [0, 1] || codes == [1, 0],
                        "fresh successor race: {codes:?}"
                    );
                    if codes[1] == 0 {
                        later.record = competing;
                        later.intent = competing_intent;
                        later.input = competing_input;
                    }
                    eprintln!("E3_FRESH_SUCCESSOR_TWO_PROCESS_WRITERS_ONE_HEAD_ONE_STALE");
                } else {
                    let expected_record = record_name(&later.record);
                    OWNERSHIP_TEST_HOOK.with(|hook| {
                        *hook.borrow_mut() = Some(Box::new(move |stage, _, name| {
                            if stage == "published" && name == expected_record {
                                return Err(ConfigProjectionFailureV1::PartialPublication);
                            }
                            Ok(())
                        }))
                    });
                    let interrupted = registry.publish_dormant(&later.record, Some(later.chain()));
                    OWNERSHIP_TEST_HOOK.with(|hook| hook.borrow_mut().take());
                    assert!(
                        interrupted.is_err(),
                        "record publication was not interrupted"
                    );
                    let series_path = parent
                        .authority
                        .join(CHILD_NAME)
                        .join("series")
                        .join(&later.record.identity.series_id);
                    let old_head: ConfigProjectionHeadV1 =
                        ConfigProjectionCodecV1::decode_canonical_json(
                            &std::fs::read(series_path.join("head.json")).unwrap(),
                        )
                        .unwrap();
                    assert_eq!(
                        old_head.head_ref, reference,
                        "immutable publication advanced head"
                    );
                    let original =
                        std::fs::read(series_path.join("records").join(record_name(&later.record)))
                            .unwrap();
                    let series_path_hook = series_path.clone();
                    OWNERSHIP_TEST_HOOK.with(|hook| {
                        *hook.borrow_mut() = Some(Box::new(move |stage, directory, name| {
                            if stage == "published"
                                && name == "head.json"
                                && std::fs::read_link(format!(
                                    "/proc/self/fd/{}",
                                    directory.as_raw_fd()
                                ))
                                .unwrap()
                                    == series_path_hook
                            {
                                return Err(ConfigProjectionFailureV1::PartialPublication);
                            }
                            Ok(())
                        }))
                    });
                    let lost_return = registry.publish_dormant(&later.record, Some(later.chain()));
                    OWNERSHIP_TEST_HOOK.with(|hook| hook.borrow_mut().take());
                    assert!(lost_return.is_err(), "head publication return was not lost");
                    let reopened = ConfigProjectionRegistryV1::open(parent.clone()).unwrap();
                    assert_eq!(
                        reopened
                            .publish_dormant(&later.record, Some(later.chain()))
                            .unwrap(),
                        record_ref(&later.record)
                    );
                    assert_eq!(
                        std::fs::read(series_path.join("records").join(record_name(&later.record)))
                            .unwrap(),
                        original
                    );
                    eprintln!(
                        "E3_FRESH_SUCCESSOR_IMMUTABLE_BEFORE_HEAD_AND_LOST_RETURN_EXACT_RETRY"
                    );
                }
                let later_ref = registry
                    .publish_dormant(&later.record, Some(later.chain()))
                    .expect("fresh successor exact retry");
                assert_eq!(later_ref.revision, reference.revision + 1);
                let later_consumer =
                    prepared_member_dispatch_consumer_id_v1(&later.intent.preparation_id).unwrap();
                let later_lease = registry
                    .acquire_consumer_lease(
                        &later_ref,
                        ConfigProjectionConsumerKindV1::MemberDispatchV2,
                        later.record.created_at.clone(),
                        Some(&later_consumer),
                    )
                    .unwrap();
                assert_ne!(later_lease.consumer_id, lease.consumer_id);
                assert_eq!(
                    registry
                        .publish_dormant(&later.record, Some(later.chain()))
                        .unwrap(),
                    later_ref
                );
                let mut contender = later.record.clone();
                contender.record_id = id("cpr_");
                seal_record(&mut contender);
                assert_eq!(
                    registry.publish_dormant(&contender, None),
                    Err(ConfigProjectionFailureV1::StaleRevision)
                );
                let readback = registry.recover(None).unwrap();
                assert_eq!(readback.kernel_effects.len(), 8);
                for effect in readback.kernel_effects {
                    if effect.intent.fence_id == fixture.intent.fence_id {
                        assert_eq!(effect.terminal_child_evidence, [evidence.clone()]);
                    } else {
                        assert_eq!(effect.projection.as_ref(), Some(&later.record));
                        assert!(effect.terminal_child_evidence.is_empty());
                    }
                }
                let mut complete = evidence.clone();
                complete.evidence_id = id("tce_");
                complete.final_projection_ref = later_ref.clone();
                let later_groups: Vec<_> = later
                    .registrations
                    .iter()
                    .map(|registration| crate::E3TerminalCgroupQuiescenceV1 {
                        cgroup_registration_id: registration.cgroup_registration_id.clone(),
                        cgroup_registration_hash: registration.cgroup_registration_hash.clone(),
                        role: registration.role,
                        cgroup: registration.cgroup.clone(),
                        cgroup_events_sha256: ordinary_sha256(b"populated 0\n"),
                        cgroup_procs_sha256: ordinary_sha256(b""),
                        populated: false,
                        ordered_live_pids: Vec::new(),
                    })
                    .collect();
                complete.ordered_empty_cgroups = later_groups;
                complete
                    .ordered_empty_cgroups
                    .sort_by_key(|entry| terminal_role_rank(entry.role));
                complete.evidence_hash = hash_omitting(
                    "substrate.e3.terminal-child-quiescence.v1",
                    "evidence",
                    &complete,
                    "evidence_hash",
                )
                .unwrap();
                assert_eq!(
                    registry.publish_terminal_child_evidence(&complete),
                    Err(ConfigProjectionFailureV1::WrongBinding),
                    "later proof omitted required earlier registrations"
                );
                complete
                    .ordered_empty_cgroups
                    .extend(evidence.ordered_empty_cgroups.clone());
                complete.ordered_empty_cgroups.sort_by_key(|entry| {
                    (
                        terminal_role_rank(entry.role),
                        entry.cgroup.cgroup_v2_mount_device_id,
                        entry.cgroup.cgroup_v2_mount_inode,
                        entry.cgroup.cgroup_directory_inode,
                        entry.cgroup.cgroup_relative_path.clone(),
                        entry.cgroup_registration_id.clone(),
                    )
                });
                complete.evidence_hash = hash_omitting(
                    "substrate.e3.terminal-child-quiescence.v1",
                    "evidence",
                    &complete,
                    "evidence_hash",
                )
                .unwrap();
                registry.publish_terminal_child_evidence(&complete).unwrap();
                for effect in registry.recover(None).unwrap().kernel_effects {
                    if effect.intent.fence_id == fixture.intent.fence_id {
                        if effect.child_cgroup.is_some() {
                            assert_eq!(
                                effect.terminal_child_evidence,
                                [evidence.clone(), complete.clone()]
                            );
                        } else {
                            assert_eq!(effect.terminal_child_evidence, [evidence.clone()]);
                        }
                    } else {
                        assert_eq!(effect.terminal_child_evidence, [complete.clone()]);
                    }
                }
            }
        }

        #[test]
        fn e3_e_same_subject_missing_store_does_not_initialize_authority() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let path = parent.authority.join(CHILD_NAME).join("store.json");
            std::fs::remove_file(&path).unwrap();
            assert!(registry.recover(Some(&fixture.record.identity)).is_err());
            assert!(!path.exists());
        }

        #[test]
        fn e3_e_prepared_lease_resolves_after_transaction_return_is_lost() {
            struct LostReturnParent {
                parent: Arc<TestParent>,
                fail_return: std::sync::atomic::AtomicBool,
            }
            impl ConfigProjectionHsaAuthorityV1 for LostReturnParent {
                fn with_locked_parent(
                    &self,
                    operation: &mut dyn for<'fd> FnMut(
                        BorrowedFd<'fd>,
                    )
                        -> Result<(), ConfigProjectionFailureV1>,
                ) -> Result<(), ConfigProjectionFailureV1> {
                    self.parent.with_locked_parent(operation)?;
                    if self.fail_return.swap(false, Ordering::SeqCst) {
                        Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
                    } else {
                        Ok(())
                    }
                }
            }
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let reference = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let consumer =
                prepared_member_dispatch_consumer_id_v1(&fixture.intent.preparation_id).unwrap();
            let fault = Arc::new(LostReturnParent {
                parent: parent.clone(),
                fail_return: std::sync::atomic::AtomicBool::new(false),
            });
            let interrupted = ConfigProjectionRegistryV1::open(fault.clone()).unwrap();
            fault.fail_return.store(true, Ordering::SeqCst);
            assert_eq!(
                interrupted.acquire_consumer_lease(
                    &reference,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    fixture.record.created_at.clone(),
                    Some(&consumer),
                ),
                Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
            );
            let directory = parent
                .authority
                .join(CHILD_NAME)
                .join("leases")
                .join(&reference.series_id);
            let head_path = directory.join(&consumer).join("head.json");
            let durable_bytes = std::fs::read(&head_path).unwrap();
            let durable: ConfigProjectionConsumerLeaseV1 =
                ConfigProjectionCodecV1::decode_canonical_json(&durable_bytes).unwrap();
            assert_eq!(durable.consumer_id, consumer);
            assert_eq!(
                durable.posture,
                ConfigProjectionConsumerLeasePostureV1::Held
            );
            assert_eq!(durable.revision, 1);
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            let resolved = registry
                .acquire_consumer_lease(
                    &reference,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    fixture.record.created_at.clone(),
                    Some(&consumer),
                )
                .unwrap();
            assert_eq!(resolved, durable);
            assert_eq!(std::fs::read(&head_path).unwrap(), durable_bytes);
            assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
            assert_eq!(
                registry.acquire_consumer_lease(
                    &reference,
                    ConfigProjectionConsumerKindV1::WorldRuntimeAdapterV3,
                    fixture.record.created_at.clone(),
                    Some(&consumer),
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        }

        #[test]
        fn e3_e_prepared_lease_reuses_durable_write_and_rejects_unequal_or_released() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let reference = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let consumer =
                prepared_member_dispatch_consumer_id_v1(&fixture.intent.preparation_id).unwrap();
            let acquire = || {
                registry.acquire_consumer_lease(
                    &reference,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    fixture.record.created_at.clone(),
                    Some(&consumer),
                )
            };
            let first = acquire().unwrap();
            // Discard the first return as if the caller crashed after the durable write.
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            assert_eq!(acquire().unwrap(), first);
            let lease_root = parent
                .authority
                .join(CHILD_NAME)
                .join("leases")
                .join(&reference.series_id);
            assert_eq!(std::fs::read_dir(&lease_root).unwrap().count(), 1);
            assert!(registry
                .acquire_consumer_lease(
                    &reference,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:00:01.000000Z".into()),
                    Some(&consumer)
                )
                .is_err());
            assert!(registry
                .acquire_consumer_lease(
                    &reference,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    fixture.record.created_at.clone(),
                    Some(&id("cpc_"))
                )
                .is_err());
            let released = registry
                .release_consumer_lease(&first, Timestamp("2026-09-10T00:01:00.000000Z".into()))
                .unwrap();
            assert!(acquire().is_err());
            assert_eq!(std::fs::read_dir(&lease_root).unwrap().count(), 1);
            let durable: ConfigProjectionConsumerLeaseV1 = serde_json::from_slice(
                &std::fs::read(lease_root.join(&consumer).join("head.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(durable, released);
        }

        #[test]
        fn e3_e_dormant_rejects_a_rehashed_source_with_a_missing_loader_entry() {
            let (_temp, parent, registry, store) = test_registry();
            let mut fixture = PreparedGatewayFixture::new(&registry, &store, None);
            registry
                .with_transaction(false, |transaction| {
                    validate_prepared_gateway_chain(transaction, &fixture.record, fixture.chain())
                })
                .unwrap();
            let record = &mut fixture.record;
            record.native.ambient_closure.inputs.remove(0);
            record
                .native
                .ambient_closure
                .validated_loader_input_fingerprint = ConfigProjectionCodecV1::domain_sha256(
                "substrate.e3.codex-0.125-loader-inputs.v1",
                &serde_json::json!({
                    "inputs": record.native.ambient_closure.inputs,
                    "loader_source": record.native.ambient_closure.loader_source,
                }),
            )
            .unwrap();
            seal_record(record);
            fixture.input.dormant_projection_ref = record_ref(record);
            fixture.input.launch_input_hash = hash_omitting(
                "substrate.e3.managed-gateway-launch-input.v1",
                "launch_input",
                &fixture.input,
                "launch_input_hash",
            )
            .unwrap();
            let crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } =
                &record.activation.publication_fence
            else {
                unreachable!()
            };
            let fence_id = fence_id.clone();
            let manifest_path = parent
                .authority
                .join(CHILD_NAME)
                .join("native-sources")
                .join(&record.identity.series_id)
                .join(&fence_id)
                .join("source-manifest.json");
            let mut manifest: NativeProjectionSourceManifestV1 =
                ConfigProjectionCodecV1::decode_canonical_json(
                    &std::fs::read(&manifest_path).unwrap(),
                )
                .unwrap();
            manifest.native_projection_hash = record.native.projection_hash.clone();
            manifest.manifest_hash = hash_omitting(
                "substrate.e3.native-projection-source-manifest.v1",
                "manifest",
                &manifest,
                "manifest_hash",
            )
            .unwrap();
            std::fs::write(
                &manifest_path,
                ConfigProjectionCodecV1::encode_canonical_json(&manifest).unwrap(),
            )
            .unwrap();
            // All canonical hashes and cross-object references agree. Only the source
            // closure is incomplete, so the negative cannot pass on an unrelated join.
            validate_record(&fixture.record, &store).unwrap();
            assert_eq!(
                fixture.input.dormant_projection_ref,
                record_ref(&fixture.record)
            );
            assert_eq!(
                validate_native_projection_source_input(
                    &fixture.record.native,
                    &fixture.record.identity.series_id,
                    &fence_id,
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            assert_eq!(
                registry.publish_dormant(&fixture.record, Some(fixture.chain())),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            assert!(!parent
                .authority
                .join(CHILD_NAME)
                .join("series")
                .join(&fixture.record.identity.series_id)
                .join("head.json")
                .exists());
        }

        #[test]
        #[ignore = "explicit root native-source post-publication GID substitution"]
        fn test_e3_native_source_published_gid_substitutions_reject_without_repair() {
            assert_eq!(unsafe { libc::geteuid() }, 0);
            for published in [false, true] {
                for entry in [
                    "",
                    "codex-home",
                    "system-empty",
                    "codex-home/config.toml",
                    "source-manifest.json",
                ] {
                    let (_temp, parent, registry, store) = test_registry();
                    let fixture = PreparedGatewayFixture::new(&registry, &store, None);
                    if published {
                        registry
                            .publish_dormant(&fixture.record, Some(fixture.chain()))
                            .unwrap();
                    }
                    let crate::ConfigProjectionPublicationFenceV1::ZeroLiveClosed { fence_id } =
                        &fixture.record.activation.publication_fence
                    else {
                        unreachable!()
                    };
                    let source = parent
                        .authority
                        .join(CHILD_NAME)
                        .join("native-sources")
                        .join(&fixture.record.identity.series_id)
                        .join(fence_id);
                    let head = parent
                        .authority
                        .join(CHILD_NAME)
                        .join("series")
                        .join(&fixture.record.identity.series_id)
                        .join("head.json");
                    let old_head = std::fs::read(&head).ok();
                    let config_bytes =
                        std::fs::read(source.join("codex-home/config.toml")).unwrap();
                    let manifest_bytes =
                        std::fs::read(source.join("source-manifest.json")).unwrap();
                    let plan = Codex0125ProjectionV1::render(
                        &fixture.record.identity,
                        &fixture.record.effective,
                        &fixture.record.managed_gateway,
                        fixture.record.native.root.clone(),
                        fence_id,
                        fixture
                            .record
                            .native
                            .ambient_closure
                            .inputs
                            .iter()
                            .filter(|input| input.layer == "Project")
                            .cloned()
                            .collect(),
                    )
                    .unwrap();
                    let file = File::open(source.join(entry)).unwrap();
                    assert_eq!(unsafe { libc::fchown(file.as_raw_fd(), !0, 1001) }, 0);
                    let before = fstat(file.as_raw_fd()).unwrap();
                    assert!(
                        registry
                            .publish_dormant(&fixture.record, Some(fixture.chain()))
                            .is_err(),
                        "{entry}, published={published}"
                    );
                    assert!(
                        registry
                            .recover(None)
                            .map(|readback| readback.store)
                            .is_err(),
                        "{entry}"
                    );
                    assert!(
                        registry
                            .publish_native_source(
                                &fixture.record.identity.series_id,
                                fence_id,
                                &plan,
                                fixture.record.created_at.clone()
                            )
                            .is_err(),
                        "{entry}"
                    );
                    assert!(
                        same_owned_metadata(&before, &fstat(file.as_raw_fd()).unwrap()),
                        "{entry}"
                    );
                    assert_eq!(std::fs::read(&head).ok(), old_head);
                    assert_eq!(
                        std::fs::read(source.join("codex-home/config.toml")).unwrap(),
                        config_bytes
                    );
                    assert_eq!(
                        std::fs::read(source.join("source-manifest.json")).unwrap(),
                        manifest_bytes
                    );
                }
            }
        }

        #[test]
        fn e3_e_prepared_gateway_chain_is_exact_immutable_and_recoverable() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let reference = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            assert_eq!(
                registry
                    .publish_dormant(&fixture.record, Some(fixture.chain()))
                    .unwrap(),
                reference
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            let mut wrong = fixture.input.clone();
            wrong.world_generation += 1;
            wrong.launch_input_hash = hash_omitting(
                "substrate.e3.managed-gateway-launch-input.v1",
                "launch_input",
                &wrong,
                "launch_input_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_dormant(
                    &fixture.record,
                    Some((
                        &fixture.handoff,
                        &fixture.gateway,
                        &fixture.boundary,
                        &fixture.intent,
                        &wrong,
                        &fixture.registrations
                    ))
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            let root = parent.authority.join(CHILD_NAME);
            let input_path = root
                .join("gateway-launch-inputs")
                .join(format!("{}.json", fixture.input.launch_input_id));
            let saved = std::fs::read(&input_path).unwrap();
            std::fs::rename(
                &input_path,
                input_path
                    .parent()
                    .unwrap()
                    .join(temp_name("gateway-launch-input")),
            )
            .unwrap();
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            assert_eq!(std::fs::read(&input_path).unwrap(), saved);
            // A renamed but validly hashed object from another launch cannot recover into this name.
            let mut substituted = fixture.input.clone();
            substituted.launch_input_id = id("gal_");
            substituted.launch_input_hash = hash_omitting(
                "substrate.e3.managed-gateway-launch-input.v1",
                "launch_input",
                &substituted,
                "launch_input_hash",
            )
            .unwrap();
            std::fs::write(
                &input_path,
                ConfigProjectionCodecV1::encode_canonical_json(&substituted).unwrap(),
            )
            .unwrap();
            assert!(matches!(
                registry.recover(None).map(|readback| readback.store),
                Err(ConfigProjectionFailureV1::WrongBinding)
            ));
        }

        #[test]
        fn e3_e_handoff_transition_is_exact_retry_only() {
            let (_temp, _parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let successor =
                terminal_handoff_successor(&fixture.handoff, SecretHandoffStateV1::Failed);

            let reference = registry
                .publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &successor,
                    None,
                    None,
                )
                .unwrap();
            assert_eq!(reference, handoff_reference(&successor).unwrap());
            assert_eq!(
                registry
                    .publish_handoff_transition_v1(
                        &fixture.record.identity,
                        &fixture.intent.fence_id,
                        &successor,
                        None,
                        None,
                    )
                    .unwrap(),
                reference
            );

            let conflicting =
                terminal_handoff_successor(&fixture.handoff, SecretHandoffStateV1::Expired);
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &conflicting,
                    None,
                    None,
                ),
                Err(ConfigProjectionFailureV1::StaleRevision)
            );
        }

        #[test]
        fn e3_e_handoff_transition_reconciles_revision_before_head_cas() {
            use std::os::unix::fs::OpenOptionsExt;

            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let successor =
                terminal_handoff_successor(&fixture.handoff, SecretHandoffStateV1::Expired);
            let revision_path = parent
                .authority
                .join(CHILD_NAME)
                .join("handoffs")
                .join(&successor.handoff.handoff_id)
                .join("revisions")
                .join("00000000000000000002.json");
            let bytes = ConfigProjectionCodecV1::encode_canonical_json(&successor).unwrap();
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&revision_path)
                .unwrap();
            std::io::Write::write_all(&mut file, &bytes).unwrap();
            file.sync_all().unwrap();

            let reference = registry
                .publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &successor,
                    None,
                    None,
                )
                .unwrap();
            assert_eq!(reference, handoff_reference(&successor).unwrap());
            assert_eq!(
                std::fs::read(
                    revision_path
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join("head.json")
                )
                .unwrap(),
                ConfigProjectionCodecV1::encode_canonical_json(&reference).unwrap()
            );
        }

        #[test]
        fn e3_e_handoff_transition_rejects_wrong_join_and_illegal_successor() {
            let (_temp, _parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let successor =
                terminal_handoff_successor(&fixture.handoff, SecretHandoffStateV1::Failed);

            let mut wrong_subject = fixture.record.identity.clone();
            wrong_subject.identity_hash = "00".repeat(32);
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &wrong_subject,
                    &fixture.intent.fence_id,
                    &successor,
                    None,
                    None,
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );

            let mut wrong_preparation = successor.clone();
            wrong_preparation
                .handoff
                .credential_source_ref
                .preparation_id = id("e3p_");
            wrong_preparation.handoff.credential_source_ref.ref_hash = hash_omitting(
                "substrate.e3.credential-source-ref.v1",
                "credential_source",
                &wrong_preparation.handoff.credential_source_ref,
                "ref_hash",
            )
            .unwrap();
            wrong_preparation.revision_hash = hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                &wrong_preparation,
                "revision_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &wrong_preparation,
                    None,
                    None,
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );

            let mut wrong_gateway = successor.clone();
            wrong_gateway
                .handoff
                .receiving_gateway_ref
                .gateway_identity_hash = "11".repeat(32);
            wrong_gateway.revision_hash = hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                &wrong_gateway,
                "revision_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &wrong_gateway,
                    None,
                    None,
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &id("cpf_"),
                    &successor,
                    None,
                    None,
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );

            let mut illegal = successor.clone();
            illegal.handoff.state = SecretHandoffStateV1::Consumed;
            illegal.revision_hash = hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                &illegal,
                "revision_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &illegal,
                    None,
                    None,
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
        }

        #[test]
        fn e3_e_handoff_transition_keeps_legal_consumed_retry_after_projection_advances() {
            let (_temp, _parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let dormant_ref = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let (ready, ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
            let consumed =
                consumed_handoff_successor(&delivered_handoff_successor(&fixture.handoff));
            let consumed_ref = handoff_reference(&consumed).unwrap();
            let ready_ref = registry
                .publish_ready_closed(&dormant_ref, &ready, &ack, &lease)
                .unwrap();
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &consumed,
                    Some(&dormant_ref),
                    Some(&lease)
                ),
                Err(ConfigProjectionFailureV1::StaleRevision)
            );

            // Synthetic retained Active history is input to E3-E recovery, not an E3-F lifecycle.
            let allow =
                successor_boundary(&fixture.boundary, GatewayAccessPostureV1::AllowExactMember);
            registry
                .with_transaction(false, |transaction| {
                    let boundaries =
                        open_directory_at(transaction.root.as_fd(), "gateway-boundaries")?;
                    let directory =
                        open_directory_at(boundaries.as_fd(), &allow.access_boundary_id)?;
                    write_immutable(
                        directory.as_fd(),
                        &format!("{:020}.json", allow.revision),
                        &ConfigProjectionCodecV1::encode_canonical_json(&allow)?,
                        "boundary",
                        transaction.owner_uid,
                    )
                })
                .unwrap();
            let mut active = successor(&ready, ManagedGatewayProjectionPostureV1::Active);
            active.managed_gateway.access_boundary_ref = boundary_ref(&allow);
            seal_record(&mut active);
            let active_ref = registry.publish_active(&ready_ref, &active).unwrap();
            registry
                .recover(Some(&active.identity))
                .expect("Active metadata readback remains available before terminal eligibility");
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &consumed,
                    Some(&dormant_ref),
                    Some(&lease)
                ),
                Err(ConfigProjectionFailureV1::StaleRevision)
            );

            let mut terminal_successor = consumed.clone();
            terminal_successor.predecessor_ref = Some(consumed_ref);
            terminal_successor.handoff.state_revision = 4;
            terminal_successor.handoff.state = SecretHandoffStateV1::Failed;
            terminal_successor.handoff.consumed_at = None;
            terminal_successor.revision_hash = hash_omitting(
                "substrate.e3.config-projection-secret-handoff-revision.v1",
                "revision",
                &terminal_successor,
                "revision_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_handoff_transition_v1(
                    &fixture.record.identity,
                    &fixture.intent.fence_id,
                    &terminal_successor,
                    None,
                    None,
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            let mut evidence = terminal_evidence(&store, &active, &active_ref);
            evidence.ordered_terminal_processes = registry
                .recover(None)
                .unwrap()
                .kernel_effects
                .into_iter()
                .flat_map(|effect| effect.child_processes)
                .map(|child| crate::E3TerminalProcessObservationV1 {
                    registration_id: child.registration_id,
                    registration_hash: child.registration_hash,
                    role: child.role,
                    pid: child.pid,
                    pid_start_time_ticks: child.pid_start_time_ticks,
                    observation: crate::E3TerminalProcessObservationKindV1::ParentWaitid {
                        terminal_wait_status: 0,
                    },
                })
                .collect();
            evidence.ordered_empty_cgroups = fixture
                .registrations
                .iter()
                .map(|registration| crate::E3TerminalCgroupQuiescenceV1 {
                    cgroup_registration_id: registration.cgroup_registration_id.clone(),
                    cgroup_registration_hash: registration.cgroup_registration_hash.clone(),
                    role: registration.role,
                    cgroup: registration.cgroup.clone(),
                    cgroup_events_sha256: ordinary_sha256(b"populated 0\n"),
                    cgroup_procs_sha256: ordinary_sha256(b""),
                    populated: false,
                    ordered_live_pids: Vec::new(),
                })
                .collect();
            evidence
                .ordered_empty_cgroups
                .sort_by_key(|entry| terminal_role_rank(entry.role));
            evidence.evidence_hash = hash_omitting(
                "substrate.e3.terminal-child-quiescence.v1",
                "evidence",
                &evidence,
                "evidence_hash",
            )
            .unwrap();
            for effect in registry.recover(None).unwrap().kernel_effects {
                let mut resolution = E3KernelEffectResolutionV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    resolution_id: id("ekr_"),
                    effect_intent_ref: kernel_effect_intent_ref(&effect.intent),
                    disposition: E3KernelEffectResolutionDispositionV1::RevertedAndQuiescent,
                    observed_cgroup: effect
                        .child_cgroup
                        .as_ref()
                        .map(|registration| registration.cgroup.clone()),
                    observed_nftables_table_handle: effect.boundary.as_ref().map(|_| 1),
                    resolved_at: now_timestamp(),
                    resolution_hash: String::new(),
                };
                resolution.resolution_hash = hash_omitting(
                    "substrate.e3.kernel-effect-resolution.v1",
                    "resolution",
                    &resolution,
                    "resolution_hash",
                )
                .unwrap();
                registry
                    .publish_kernel_effect_resolution(
                        &resolution,
                        Some(&mut || {
                            Ok(effect.boundary.as_ref().map(|boundary| {
                                successor_boundary(boundary, GatewayAccessPostureV1::Revoked)
                            }))
                        }),
                        Some(&active_ref),
                    )
                    .unwrap();
            }
            registry
                .recover(None)
                .expect("readback alone grants no Active eligibility");
            let evidence_ref = registry.publish_terminal_child_evidence(&evidence).unwrap();
            assert!(
                registry.publish_terminal_child_evidence(&evidence).is_err(),
                "terminal Active exact retry accepted a Held consumer"
            );
            registry
                .release_consumer_lease(&lease, now_timestamp())
                .unwrap();
            assert_eq!(
                registry.publish_terminal_child_evidence(&evidence).unwrap(),
                evidence_ref
            );
            let first = registry
                .recover(Some(&active.identity))
                .expect("terminal Active history readback");
            let Some(ConfigProjectionSubjectReadbackV1::Bound(metadata)) = first.subject else {
                panic!("Active subject missing")
            };
            assert_eq!(metadata.record(), &active);
            assert_eq!(metadata.current_handoff, consumed);
            assert_eq!(
                metadata.consumer_lease.as_ref().unwrap().posture,
                ConfigProjectionConsumerLeasePostureV1::Released
            );
            for effect in first.kernel_effects {
                assert_eq!(effect.terminal_child_evidence, [evidence.clone()]);
                assert!(effect.resolution.is_some());
            }
            let retry = registry.recover(Some(&active.identity)).unwrap();
            assert!(retry
                .kernel_effects
                .iter()
                .all(|effect| effect.terminal_child_evidence == [evidence.clone()]));
        }

        #[test]
        fn e3_b_registry_first_writer_and_expected_head_cas_are_strict() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let dormant = fixture.record.clone();
            let dormant_ref = registry
                .publish_dormant(&dormant, Some(fixture.chain()))
                .unwrap();
            assert_eq!(
                registry
                    .publish_dormant(&dormant, Some(fixture.chain()))
                    .unwrap(),
                dormant_ref
            );

            let (ready, ack, lease) = ready_gateway_fixture_v1(&registry, &fixture);
            let ready_ref = registry
                .publish_ready_closed(&dormant_ref, &ready, &ack, &lease)
                .unwrap();
            assert_eq!(
                registry
                    .publish_ready_closed(&dormant_ref, &ready, &ack, &lease)
                    .unwrap(),
                ready_ref
            );
            let active = successor(&ready, ManagedGatewayProjectionPostureV1::Active);
            assert_eq!(
                registry.publish_active(&dormant_ref, &active),
                Err(ConfigProjectionFailureV1::StaleRevision)
            );
            let active_ref = registry.publish_active(&ready_ref, &active).unwrap();
            assert_eq!(active_ref.revision, 3);
            assert_eq!(
                registry.publish_active(&ready_ref, &active).unwrap(),
                active_ref
            );
            assert!(parent.calls.load(Ordering::SeqCst) >= 6);
        }

        #[test]
        fn e3c_authoring_preserves_noncloneable_held_lease_resolution() {
            let (_temp, _parent, registry, store) = test_registry();
            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record, None).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                    None,
                )
                .unwrap();

            assert!(matches!(
                registry.resolve(Some((&record.identity, &held)), None).unwrap().0.unwrap(),
                ConfigProjectionResolutionV1::Current {
                    projection_ref: resolved,
                    ..
                } if resolved == projection_ref
            ));
        }

        #[test]
        fn e3c_authoring_objects_recover_only_valid_interrupted_publications() {
            let (temp, _parent, registry, store) = test_registry();
            let keys = [
                "llm.enabled",
                "llm.gateway.enabled",
                "llm.gateway.mode",
                "llm.routing.default_backend",
                "agents.enabled",
                "agents.defaults.execution.scope",
                "agents.defaults.cli.mode",
                "world.enabled",
            ];
            let mut effective = EffectiveSubstrateConfigSourceV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                accepted_home: store.accepted_home.clone(),
                workspace_root: store.accepted_home.clone(),
                values: crate::E3EffectiveConfigInputV1 {
                    llm_enabled: true,
                    agents_enabled: true,
                    world_enabled: true,
                    default_execution_scope: "world".to_string(),
                    default_cli_mode: "persistent".to_string(),
                    managed_gateway_enabled: true,
                    managed_gateway_mode: "in_world".to_string(),
                    default_backend_id: "cli:codex-world".to_string(),
                },
                ordered_explain_origins: keys
                    .into_iter()
                    .map(|key| crate::E3ConfigExplainOriginV1 {
                        key: key.to_string(),
                        source_kind: E3ConfigExplainOriginKindV1::Default,
                        source_location: None,
                    })
                    .collect(),
                source_revision: String::new(),
                source_hash: String::new(),
            };
            let mut revision_value = serde_json::to_value(&effective).unwrap();
            revision_value
                .as_object_mut()
                .unwrap()
                .remove("source_revision");
            revision_value
                .as_object_mut()
                .unwrap()
                .remove("source_hash");
            effective.source_revision = format!(
                "ecsr1_{}",
                ordinary_sha256(
                    &ConfigProjectionCodecV1::encode_canonical_json(&revision_value).unwrap()
                )
            );
            effective.source_hash = hash_omitting(
                "substrate.e3.effective-substrate-config-source.v1",
                "source",
                &effective,
                "source_hash",
            )
            .unwrap();

            let raw_hash = ordinary_sha256(b"version: 3\n");
            let mut inventory = AgentInventorySourceMaterialV1 {
                inventory_scope: "global".to_string(),
                accepted_root: store.accepted_home.clone(),
                relative_path: "agents/codex.yaml".to_string(),
                file_device_id: 1,
                file_inode: 2,
                byte_length: 11,
                raw_bytes_sha256: raw_hash.clone(),
                source_revision: format!("aisr1_{raw_hash}"),
                source_hash: String::new(),
            };
            inventory.source_hash = hash_omitting(
                "substrate.e3.agent-inventory-source.v1",
                "source",
                &inventory,
                "source_hash",
            )
            .unwrap();

            let record = test_record(&store, id("cps_"));
            let roles = [
                (
                    RuntimeArtifactAuthorityRoleV1::Codex0125,
                    "/var/lib/substrate/world-deps/codex-runtime/bin/codex",
                    RuntimeArtifactProvenanceV1::OfficialCodexRelease {
                        version: "0.125.0".to_string(),
                        target_triple: "x86_64-unknown-linux-musl".to_string(),
                        archive_name: "codex-x86_64-unknown-linux-musl.tar.gz".to_string(),
                        archive_url: "https://github.com/openai/codex/releases/download/rust-v0.125.0/codex-x86_64-unknown-linux-musl.tar.gz".to_string(),
                        archive_sha256: "4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001".to_string(),
                        archive_entry_path: "codex-x86_64-unknown-linux-musl".to_string(),
                        extracted_executable_sha256: "86dc42ac5823f25233d6dc4ec5ff34693afd8c32ff2d17b54c5eb0d15bc7d902".to_string(),
                    },
                    "86dc42ac5823f25233d6dc4ec5ff34693afd8c32ff2d17b54c5eb0d15bc7d902".to_string(),
                    record.identity.runtime_artifacts.codex.runtime_support,
                ),
                (
                    RuntimeArtifactAuthorityRoleV1::ManagedGateway,
                    "/usr/local/lib/substrate/e3/substrate-gateway",
                    RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                        component: "substrate-gateway".to_string(),
                        source_commit: "11".repeat(20),
                        source_tree: "22".repeat(20),
                        cargo_lock_sha256: "33".repeat(32),
                        target_triple: "x86_64-unknown-linux-musl".to_string(),
                        profile: "release".to_string(),
                        executable_sha256: "44".repeat(32),
                    },
                    "44".repeat(32),
                    record.identity.runtime_artifacts.managed_gateway.runtime_support,
                ),
                (
                    RuntimeArtifactAuthorityRoleV1::WorldEntryWrapper,
                    "/usr/local/lib/substrate/e3/substrate-world-entry",
                    RuntimeArtifactProvenanceV1::SubstrateSourceBuild {
                        component: "substrate-world-entry".to_string(),
                        source_commit: "11".repeat(20),
                        source_tree: "22".repeat(20),
                        cargo_lock_sha256: "33".repeat(32),
                        target_triple: "x86_64-unknown-linux-musl".to_string(),
                        profile: "release".to_string(),
                        executable_sha256: "55".repeat(32),
                    },
                    "55".repeat(32),
                    record.identity.runtime_artifacts.world_entry_wrapper.runtime_support,
                ),
            ];
            let entries = roles
                .into_iter()
                .enumerate()
                .map(
                    |(index, (authority_role, path, provenance, sha256, runtime_support))| {
                        let mut entry = crate::RuntimeArtifactManifestEntryV1 {
                            manifest_entry_id: id("rae_"),
                            authority_role,
                            configured_absolute_path: path.to_string(),
                            device_id: 10 + index as u64,
                            inode: 20 + index as u64,
                            mode: 0o755,
                            owner_uid: 0,
                            byte_length: 100 + index as u64,
                            sha256,
                            installer_source_ref: crate::InstallerArtifactSourceRefV1 {
                                source_store_id: id("ias_"),
                                source_record_id: id("iar_"),
                                revision: 1,
                                record_hash: "66".repeat(32),
                            },
                            provenance,
                            runtime_support,
                            entry_hash: String::new(),
                        };
                        entry.entry_hash = hash_omitting(
                            "substrate.e3.runtime-artifact-entry.v1",
                            "entry",
                            &entry,
                            "entry_hash",
                        )
                        .unwrap();
                        entry
                    },
                )
                .collect();
            let mut manifest = TrustedRuntimeArtifactManifestV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                manifest_id: id("ram_"),
                revision: 1,
                entries,
                created_at: Timestamp("2026-09-11T12:00:00.000000Z".to_string()),
                manifest_hash: String::new(),
            };
            manifest.manifest_hash = hash_omitting(
                "substrate.e3.runtime-artifact-manifest.v1",
                "manifest",
                &manifest,
                "manifest_hash",
            )
            .unwrap();
            validate_effective_config_source(&effective, &store).unwrap();
            validate_agent_inventory_source(&inventory, &effective, &store).unwrap();
            validate_runtime_artifact_manifest(&manifest, &store).unwrap();
            // Optional test-only export for the real-manager lifecycle fixture. This is
            // synthetic installer evidence, never installed provenance or spawn authority.
            if let Some(path) = std::env::var_os("E3_E_MANAGER_ARTIFACT_FIXTURE") {
                use std::os::unix::fs::OpenOptionsExt;
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(&manifest).unwrap();
                let mut output = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(path)
                    .unwrap();
                output.write_all(&bytes).unwrap();
                output.sync_all().unwrap();
            }
            let reference = registry
                .with_transaction(false, |transaction| {
                    publish_authoring_inputs(transaction, &store, &effective, &inventory, &manifest)
                })
                .unwrap();
            reference.validate().unwrap();
            let (projection, readback) = registry.resolve(None, Some(&reference)).unwrap();
            assert!(projection.is_none());
            assert_eq!(
                readback.unwrap(),
                (effective.clone(), inventory.clone(), manifest.clone())
            );
            for field in [
                "authority_store_id",
                "runtime_artifact_manifest_hash",
                "effective_config_source_hash",
            ] {
                let mut wrong = reference.clone();
                match field {
                    "authority_store_id" => wrong.authority_store_id = id("cpa_"),
                    "runtime_artifact_manifest_hash" => {
                        wrong.runtime_artifact_manifest_hash = "ab".repeat(32)
                    }
                    _ => wrong.effective_config_source_hash = "cd".repeat(32),
                }
                wrong.input_ref_hash = wrong.canonical_hash().unwrap();
                assert!(registry.resolve(None, Some(&wrong)).is_err(), "{field}");
            }
            assert!(matches!(
                registry.resolve(None, None),
                Err(ConfigProjectionFailureV1::Malformed)
            ));

            let child = temp.path().join("authority-v1").join(CHILD_NAME);
            let effective_path = child
                .join("inputs/effective-config")
                .join(format!("{}.json", effective.source_hash));
            let interrupted = effective_path.parent().unwrap().join(temp_name("input"));
            std::fs::rename(&effective_path, &interrupted).unwrap();
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            assert!(effective_path.is_file());

            let malformed = effective_path.parent().unwrap().join(temp_name("input"));
            std::fs::write(&malformed, b"{").unwrap();
            std::fs::set_permissions(&malformed, std::fs::Permissions::from_mode(0o600)).unwrap();
            assert!(registry
                .recover(None)
                .map(|readback| readback.store)
                .is_err());
            assert!(malformed.is_file());
        }

        #[test]
        fn e3_b_registry_resolve_reports_only_complete_unsigned_newer_schema() {
            let (temp, _parent, registry, store) = test_registry();
            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record, None).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                    None,
                )
                .unwrap();
            let record_path = temp
                .path()
                .join("authority-v1")
                .join(CHILD_NAME)
                .join("series")
                .join(&projection_ref.series_id)
                .join("records")
                .join(format!(
                    "{:020}-{}.json",
                    projection_ref.revision, projection_ref.record_id
                ));

            std::fs::write(&record_path, br#"{"ignored":{"z":1},"schema_version":2}"#).unwrap();
            assert!(matches!(
                registry
                    .resolve(Some((&record.identity, &held)), None)
                    .unwrap()
                    .0
                    .unwrap(),
                ConfigProjectionResolutionV1::UnsupportedNewerSchema {
                    observed_schema_version: 2
                }
            ));

            for invalid in [
                br#"{"ignored":true}"#.as_slice(),
                br#"{"schema_version":-1}"#.as_slice(),
                br#"{"schema_version":2,"schema_version":3}"#.as_slice(),
                br#"{"extra":0,"schema_version":1}"#.as_slice(),
                br#"{"schema_version":2.0}"#.as_slice(),
            ] {
                std::fs::write(&record_path, invalid).unwrap();
                let Err(error) = registry
                    .resolve(Some((&record.identity, &held)), None)
                    .map(|value| value.0.unwrap())
                else {
                    panic!("invalid discriminator unexpectedly resolved");
                };
                assert_eq!(error, ConfigProjectionFailureV1::Malformed);
            }
        }

        #[test]
        fn e3_b_registry_resolve_binds_subject_record_and_held_lease() {
            let (temp, _parent, registry, store) = test_registry();
            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record, None).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                    None,
                )
                .unwrap();
            let capability = match registry
                .resolve(Some((&record.identity, &held)), None)
                .unwrap()
                .0
                .unwrap()
            {
                ConfigProjectionResolutionV1::Current { capability, .. } => capability,
                _ => panic!("expected current projection"),
            };
            assert_eq!(capability._identity, record.identity);
            assert_eq!(capability._projection_ref, projection_ref);
            assert_eq!(capability._held_lease, held);
            drop(capability);

            let mut unheld = held.clone();
            unheld.consumer_id = id("cpc_");
            unheld.lease_hash.clear();
            unheld.lease_hash = hash_omitting(
                "substrate.e3.config-projection-consumer-lease.v1",
                "lease",
                &unheld,
                "lease_hash",
            )
            .unwrap();
            assert!(matches!(
                registry
                    .resolve(Some((&record.identity, &unheld)), None)
                    .map(|value| value.0.unwrap()),
                Err(ConfigProjectionFailureV1::MissingPreparation)
            ));

            let mut substituted = record.clone();
            substituted.identity.bootstrap_run_id = "substituted-bootstrap".to_string();
            seal_record(&mut substituted);
            let root = temp.path().join("authority-v1").join(CHILD_NAME);
            let series = root.join("series").join(&record.identity.series_id);
            std::fs::write(
                series.join("records").join(record_name(&record)),
                ConfigProjectionCodecV1::encode_canonical_json(&substituted).unwrap(),
            )
            .unwrap();
            let mut head: ConfigProjectionHeadV1 = ConfigProjectionCodecV1::decode_canonical_json(
                &std::fs::read(series.join("head.json")).unwrap(),
            )
            .unwrap();
            head.head_ref = record_ref(&substituted);
            head.head_hash.clear();
            head.head_hash = hash_omitting(
                "substrate.e3.config-projection-head.v1",
                "head",
                &head,
                "head_hash",
            )
            .unwrap();
            std::fs::write(
                series.join("head.json"),
                ConfigProjectionCodecV1::encode_canonical_json(&head).unwrap(),
            )
            .unwrap();

            assert!(matches!(
                registry
                    .resolve(Some((&record.identity, &held)), None)
                    .map(|value| value.0.unwrap()),
                Err(ConfigProjectionFailureV1::WrongBinding)
            ));
        }

        #[test]
        fn e3_b_registry_two_process_first_writer_has_one_winner() {
            let (_temp, parent, registry, store) = test_registry();
            let first = test_record(&store, id("cps_"));
            let mut second = first.clone();
            second.identity.series_id = id("cps_");
            second.record_id = id("cpr_");
            seal_record(&mut second);
            let mut children = Vec::new();
            for record in [first, second] {
                let child = unsafe { libc::fork() };
                assert!(child >= 0);
                if child == 0 {
                    let child_registry = ConfigProjectionRegistryV1::open(parent.clone()).unwrap();
                    let code = i32::from(child_registry.publish_dormant(&record, None).is_err());
                    unsafe { libc::_exit(code) };
                }
                children.push(child);
            }
            let mut successes = 0;
            let mut conflicts = 0;
            for child in children {
                let mut status = 0;
                assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
                match libc::WEXITSTATUS(status) {
                    0 => successes += 1,
                    1 => conflicts += 1,
                    code => panic!("unexpected child status {code}"),
                }
            }
            assert_eq!((successes, conflicts), (1, 1));
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let cas_base_ref = registry
                .publish_dormant(&fixture.record, Some(fixture.chain()))
                .unwrap();
            let (first_successor, ack, cas_lease) = ready_gateway_fixture_v1(&registry, &fixture);
            let mut second_successor = first_successor.clone();
            second_successor.record_id = id("cpr_");
            seal_record(&mut second_successor);
            let mut cas_children = Vec::new();
            for record in [first_successor, second_successor] {
                let child = unsafe { libc::fork() };
                assert!(child >= 0);
                if child == 0 {
                    let child_registry = ConfigProjectionRegistryV1::open(parent.clone()).unwrap();
                    let code = match child_registry.publish_ready_closed(
                        &cas_base_ref,
                        &record,
                        &ack,
                        &cas_lease,
                    ) {
                        Ok(_) => 0,
                        Err(ConfigProjectionFailureV1::StaleRevision) => 1,
                        Err(_) => 2,
                    };
                    unsafe { libc::_exit(code) };
                }
                cas_children.push(child);
            }
            let mut cas_successes = 0;
            let mut stale_writers = 0;
            for child in cas_children {
                let mut status = 0;
                assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
                match libc::WEXITSTATUS(status) {
                    0 => cas_successes += 1,
                    1 => stale_writers += 1,
                    code => panic!("unexpected CAS child status {code}"),
                }
            }
            assert_eq!((cas_successes, stale_writers), (1, 1));
        }

        #[test]
        fn e3_b_registry_leases_gate_retirement_and_retired_series_is_terminal() {
            let (_temp, parent, registry, store) = test_registry();
            let second_registry = ConfigProjectionRegistryV1::open(parent).unwrap();
            let dormant = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&dormant, None).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                    None,
                )
                .unwrap();
            let resolution = registry
                .resolve(Some((&dormant.identity, &held)), None)
                .unwrap()
                .0
                .unwrap();
            let capability = match resolution {
                ConfigProjectionResolutionV1::Current { capability, .. } => capability,
                _ => panic!("expected current projection"),
            };
            let evidence = terminal_evidence(&store, &dormant, &projection_ref);
            seed_terminal_evidence(&registry, &evidence);
            let retirement = retirement(&dormant, &projection_ref, &evidence);
            assert_eq!(
                second_registry.retire(&retirement),
                Err(ConfigProjectionFailureV1::Conflict)
            );
            drop(capability);
            assert_eq!(
                registry.retire(&retirement),
                Err(ConfigProjectionFailureV1::Conflict)
            );
            let released = registry
                .release_consumer_lease(&held, Timestamp("2026-09-10T00:03:00.000000Z".to_string()))
                .unwrap();
            assert_eq!(
                released.posture,
                ConfigProjectionConsumerLeasePostureV1::Released
            );
            assert_eq!(
                registry
                    .release_consumer_lease(
                        &held,
                        Timestamp("2026-09-10T00:03:00.000000Z".to_string())
                    )
                    .unwrap(),
                released
            );
            assert_eq!(
                registry.retire(&retirement),
                Err(ConfigProjectionFailureV1::MissingPreparation)
            );
            seed_revoked_boundary(&registry, &dormant, &retirement);
            registry.retire(&retirement).unwrap();
            assert!(matches!(
                registry
                    .resolve(Some((&dormant.identity, &held)), None)
                    .unwrap()
                    .0
                    .unwrap(),
                ConfigProjectionResolutionV1::Retired { .. }
            ));
            assert_eq!(
                registry.acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:04:00.000000Z".to_string()),
                    None
                ),
                Err(ConfigProjectionFailureV1::RetiredSeries)
            );
            assert_eq!(
                registry.publish_dormant(&dormant, None),
                Err(ConfigProjectionFailureV1::RetiredSeries)
            );
        }

        fn seed_revoked_boundary(
            registry: &ConfigProjectionRegistryV1,
            record: &AgentConfigProjectionRecordV1,
            retirement: &ConfigProjectionRetirementV1,
        ) {
            let intent = dormant_boundary_intent(record);
            registry
                .publish_kernel_effect_intent(&intent, None, None)
                .unwrap();
            let initial = dormant_boundary(record);
            assert_eq!(
                boundary_ref(&initial),
                record.managed_gateway.access_boundary_ref
            );
            let revoked = successor_boundary(&initial, GatewayAccessPostureV1::Revoked);
            assert_eq!(boundary_ref(&revoked), retirement.revoked_boundary_ref);
            registry
                .with_transaction(false, |transaction| {
                    let root = open_or_create_directory(
                        transaction.root.as_fd(),
                        "gateway-boundaries",
                        transaction.owner_uid,
                    )?;
                    let boundary_dir = open_or_create_directory(
                        root.as_fd(),
                        &initial.access_boundary_id,
                        transaction.owner_uid,
                    )?;
                    for boundary in [&initial, &revoked] {
                        let bytes = ConfigProjectionCodecV1::encode_canonical_json(boundary)?;
                        write_immutable(
                            boundary_dir.as_fd(),
                            &format!("{:020}.json", boundary.revision),
                            &bytes,
                            "boundary",
                            transaction.owner_uid,
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
        }

        fn terminal_evidence(
            store: &ConfigProjectionStoreV1,
            record: &AgentConfigProjectionRecordV1,
            projection_ref: &ConfigProjectionRefV1,
        ) -> E3TerminalChildQuiescenceEvidenceV1 {
            let mut evidence = E3TerminalChildQuiescenceEvidenceV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                series_id: record.identity.series_id.clone(),
                evidence_id: id("tce_"),
                final_projection_ref: projection_ref.clone(),
                world_id: record.identity.world_id.clone(),
                world_generation: record.identity.world_generation,
                ordered_terminal_processes: Vec::new(),
                ordered_empty_cgroups: Vec::new(),
                observed_at: Timestamp("2026-09-10T00:03:00.000000Z".to_string()),
                evidence_hash: String::new(),
            };
            evidence.evidence_hash = hash_omitting(
                "substrate.e3.terminal-child-quiescence.v1",
                "evidence",
                &evidence,
                "evidence_hash",
            )
            .unwrap();
            evidence
        }

        fn seed_terminal_evidence(
            registry: &ConfigProjectionRegistryV1,
            evidence: &E3TerminalChildQuiescenceEvidenceV1,
        ) {
            registry
                .with_transaction(false, |transaction| {
                    let root =
                        open_directory_at(transaction.root.as_fd(), "terminal-child-evidence")?;
                    let series = open_or_create_directory(
                        root.as_fd(),
                        &evidence.series_id,
                        transaction.owner_uid,
                    )?;
                    let bytes = ConfigProjectionCodecV1::encode_canonical_json(evidence)?;
                    write_immutable(
                        series.as_fd(),
                        &format!("{}.json", evidence.evidence_id),
                        &bytes,
                        "terminal-child",
                        transaction.owner_uid,
                    )
                })
                .unwrap();
        }

        fn retirement(
            record: &AgentConfigProjectionRecordV1,
            projection_ref: &ConfigProjectionRefV1,
            evidence: &E3TerminalChildQuiescenceEvidenceV1,
        ) -> ConfigProjectionRetirementV1 {
            let initial = dormant_boundary(record);
            let revoked = successor_boundary(&initial, GatewayAccessPostureV1::Revoked);
            let mut retirement = ConfigProjectionRetirementV1 {
                schema_version: 1,
                authority_store_id: record.identity.authority_store_id.clone(),
                series_id: projection_ref.series_id.clone(),
                final_head_ref: projection_ref.clone(),
                terminal_child_evidence_ref: crate::E3TerminalChildQuiescenceEvidenceRefV1 {
                    authority_store_id: record.identity.authority_store_id.clone(),
                    series_id: projection_ref.series_id.clone(),
                    evidence_id: evidence.evidence_id.clone(),
                    evidence_hash: evidence.evidence_hash.clone(),
                },
                revoked_boundary_ref: boundary_ref(&revoked),
                consumer_lease_count: 0,
                retired_at: Timestamp("2026-09-10T00:04:00.000000Z".to_string()),
                retirement_hash: String::new(),
            };
            retirement.retirement_hash = hash_omitting(
                "substrate.e3.config-projection-retirement.v1",
                "retirement",
                &retirement,
                "retirement_hash",
            )
            .unwrap();
            retirement
        }

        #[test]
        fn e3_b_registry_recovery_is_owned_and_process_death_releases_locks() {
            let (temp, parent, registry, store) = test_registry();
            assert_eq!(
                registry
                    .with_transaction::<()>(false, |_| Err(ConfigProjectionFailureV1::Conflict)),
                Err(ConfigProjectionFailureV1::Conflict)
            );
            let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ =
                    registry.with_transaction::<()>(false, |_| panic!("intentional child unwind"));
            }));
            assert!(unwind.is_err());
            assert_eq!(
                registry
                    .recover(None)
                    .map(|readback| readback.store)
                    .unwrap(),
                store
            );
            let child = unsafe { libc::fork() };
            assert!(child >= 0);
            if child == 0 {
                registry
                    .with_transaction::<()>(false, |_| unsafe { libc::_exit(0) })
                    .unwrap();
                unreachable!();
            }
            let mut status = 0;
            assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
            assert_eq!(libc::WEXITSTATUS(status), 0);
            assert_eq!(
                registry
                    .recover(None)
                    .map(|readback| readback.store)
                    .unwrap(),
                store
            );

            let store_bytes = ConfigProjectionCodecV1::encode_canonical_json(&store).unwrap();
            registry
                .with_transaction(false, |transaction| {
                    let temp_name = temp_name("store");
                    let mut file = open_file_at(
                        transaction.root.as_fd(),
                        &temp_name,
                        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
                        0o600,
                    )?;
                    file.write_all(&store_bytes)
                        .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
                    fsync(&file)
                })
                .unwrap();
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            let child_root = temp.path().join("authority-v1").join(CHILD_NAME);
            assert!(!std::fs::read_dir(&child_root).unwrap().any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".e3-tmp.")));

            let non_e3 = temp
                .path()
                .join("authority-v1")
                .join("preserved-owner-state");
            std::fs::create_dir(&non_e3).unwrap();
            let unknown = child_root.join("unknown-entry");
            std::fs::write(&unknown, b"never-authoritative").unwrap();
            assert_eq!(
                registry.recover(None).map(|readback| readback.store),
                Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
            );
            assert!(non_e3.is_dir());
            assert!(unknown.is_file());
            assert!(parent.calls.load(Ordering::SeqCst) >= 4);
        }

        #[test]
        fn e3_b_registry_recovery_validates_store_temp_before_promotion() {
            let (temp, _parent, registry, store) = test_registry();
            let child_root = temp.path().join("authority-v1").join(CHILD_NAME);
            std::fs::remove_file(child_root.join("store.json")).unwrap();
            let store_bytes = ConfigProjectionCodecV1::encode_canonical_json(&store).unwrap();
            let valid_temp = child_root.join(temp_name("store"));
            std::fs::write(&valid_temp, &store_bytes).unwrap();
            std::fs::set_permissions(&valid_temp, std::fs::Permissions::from_mode(0o600)).unwrap();
            assert_eq!(
                registry
                    .recover(None)
                    .map(|readback| readback.store)
                    .unwrap(),
                store
            );

            std::fs::remove_file(child_root.join("store.json")).unwrap();
            let mut invalid = store.clone();
            invalid.store_hash = digest();
            let invalid_bytes = ConfigProjectionCodecV1::encode_canonical_json(&invalid).unwrap();
            let invalid_temp = child_root.join(temp_name("store"));
            std::fs::write(&invalid_temp, invalid_bytes).unwrap();
            std::fs::set_permissions(&invalid_temp, std::fs::Permissions::from_mode(0o600))
                .unwrap();
            assert_eq!(
                registry.recover(None).map(|readback| readback.store),
                Err(ConfigProjectionFailureV1::HashInvalid)
            );
            assert!(!child_root.join("store.json").exists());
            assert!(invalid_temp.exists());
        }

        #[test]
        fn e3_b_registry_recovers_each_admitted_publication_boundary() {
            let (temp, _parent, registry, store) = test_registry();
            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record, None).unwrap();
            let root = temp.path().join("authority-v1").join(CHILD_NAME);

            let subject = subject_hash(&record.identity).unwrap();
            move_final_to_temp(
                &root.join("subjects"),
                &format!("{subject}.json"),
                "subject",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();

            let records = root
                .join("series")
                .join(&record.identity.series_id)
                .join("records");
            move_final_to_temp(&records, &record_name(&record), "record");
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();

            let series = root.join("series").join(&record.identity.series_id);
            move_final_to_temp(&series, "head.json", "head");
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();

            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                    None,
                )
                .unwrap();
            let consumer = root
                .join("leases")
                .join(&record.identity.series_id)
                .join(&held.consumer_id);
            move_final_to_temp(
                &consumer.join("revisions"),
                "00000000000000000001.json",
                "lease",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            move_final_to_temp(&consumer, "head.json", "head");
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();

            let released = registry
                .release_consumer_lease(&held, Timestamp("2026-09-10T00:02:00.000000Z".to_string()))
                .unwrap();
            let held_bytes =
                std::fs::read(consumer.join("revisions/00000000000000000001.json")).unwrap();
            std::fs::write(consumer.join("head.json"), held_bytes).unwrap();
            let successor_temp = consumer.join(temp_name("head"));
            std::fs::write(
                &successor_temp,
                ConfigProjectionCodecV1::encode_canonical_json(&released).unwrap(),
            )
            .unwrap();
            std::fs::set_permissions(&successor_temp, std::fs::Permissions::from_mode(0o600))
                .unwrap();
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();

            let evidence = terminal_evidence(&store, &record, &projection_ref);
            seed_terminal_evidence(&registry, &evidence);
            let evidence_dir = root
                .join("terminal-child-evidence")
                .join(&record.identity.series_id);
            move_final_to_temp(
                &evidence_dir,
                &format!("{}.json", evidence.evidence_id),
                "terminal-child",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();

            let retirement = retirement(&record, &projection_ref, &evidence);
            seed_revoked_boundary(&registry, &record, &retirement);
            registry.retire(&retirement).unwrap();
            move_final_to_temp(
                &root.join("retirement"),
                &format!("{}.json", record.identity.series_id),
                "retirement",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            assert!(matches!(
                registry
                    .resolve(Some((&record.identity, &held)), None)
                    .unwrap()
                    .0
                    .unwrap(),
                ConfigProjectionResolutionV1::Retired { .. }
            ));
        }

        fn move_final_to_temp(directory: &Path, final_name: &str, kind: &str) {
            let temporary = directory.join(temp_name(kind));
            std::fs::rename(directory.join(final_name), &temporary).unwrap();
            assert!(temporary.is_file());
        }

        #[test]
        fn e3_b_registry_finish_revalidates_object_hashes_and_preserves_owned_temps() {
            let (temp, _parent, registry, store) = test_registry();
            let child_root = temp.path().join("authority-v1").join(CHILD_NAME);
            let result = registry.with_transaction::<()>(false, |transaction| {
                let mut corrupted = store.clone();
                corrupted.store_hash = digest();
                let bytes = ConfigProjectionCodecV1::encode_canonical_json(&corrupted)?;
                let mut file = open_file_at(
                    transaction.root.as_fd(),
                    "store.json",
                    libc::O_WRONLY | libc::O_TRUNC,
                    0,
                )?;
                file.write_all(&bytes)
                    .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
                fsync(&file)?;
                Err(ConfigProjectionFailureV1::Conflict)
            });
            assert_eq!(result, Err(ConfigProjectionFailureV1::HashInvalid));

            let (other_temp, _parent, other_registry, other_store) = test_registry();
            let dormant = test_record(&other_store, id("cps_"));
            other_registry.publish_dormant(&dormant, None).unwrap();
            let subject = subject_hash(&dormant.identity).unwrap();
            let subject_path = other_registry
                .with_transaction(false, |transaction| {
                    let subjects = open_directory_at(transaction.root.as_fd(), "subjects")?;
                    let final_name = format!("{subject}.json");
                    let bytes = read_file_at(subjects.as_fd(), &final_name)?;
                    let invalid_temp_name = temp_name("binding");
                    let mut file = open_file_at(
                        subjects.as_fd(),
                        &invalid_temp_name,
                        libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
                        0o600,
                    )?;
                    file.write_all(&bytes)
                        .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
                    fsync(&file)?;
                    Ok(invalid_temp_name)
                })
                .unwrap_err();
            assert_eq!(subject_path, ConfigProjectionFailureV1::PartialPublication);
            assert!(child_root.is_dir());
            assert!(std::fs::read_dir(
                other_temp
                    .path()
                    .join("authority-v1")
                    .join(CHILD_NAME)
                    .join("subjects")
            )
            .unwrap()
            .any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".binding")));
        }

        #[test]
        fn e3_e_kernel_resolution_retains_failed_cleanup_and_never_replays_completion() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let root = parent.authority.join(CHILD_NAME);
            let mut resolution = E3KernelEffectResolutionV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id,
                resolution_id: id("ekr_"),
                effect_intent_ref: fixture.boundary.kernel_effect_intent_ref.clone(),
                disposition: E3KernelEffectResolutionDispositionV1::NoEffectObserved,
                observed_cgroup: None,
                observed_nftables_table_handle: None,
                resolved_at: now_timestamp(),
                resolution_hash: String::new(),
            };
            resolution.resolution_hash = hash_omitting(
                "substrate.e3.kernel-effect-resolution.v1",
                "resolution",
                &resolution,
                "resolution_hash",
            )
            .unwrap();
            let path = root
                .join("kernel-effects/resolutions")
                .join(format!("{}.json", resolution.resolution_id));
            let intent_path = root.join("kernel-effects/intents").join(format!(
                "{}.json",
                resolution.effect_intent_ref.effect_intent_id
            ));
            let retained_intent = std::fs::read(&intent_path).unwrap();
            let mut calls = 0;
            for fail_cleanup in [true, false] {
                let outcome = registry.publish_kernel_effect_resolution(
                    &resolution,
                    Some(&mut || {
                        calls += 1;
                        assert!(!path.exists(), "resolution must follow successful cleanup");
                        assert_eq!(std::fs::read(&intent_path).unwrap(), retained_intent);
                        for lock_path in [&parent.parent_lock, &root.join("lock")] {
                            let competing = File::open(lock_path).unwrap();
                            assert_eq!(
                                unsafe {
                                    libc::flock(
                                        competing.as_raw_fd(),
                                        libc::LOCK_EX | libc::LOCK_NB,
                                    )
                                },
                                -1
                            );
                            assert_eq!(
                                std::io::Error::last_os_error().raw_os_error(),
                                Some(libc::EWOULDBLOCK)
                            );
                        }
                        if fail_cleanup {
                            Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
                        } else {
                            Ok(None)
                        }
                    }),
                    None,
                );
                if fail_cleanup {
                    assert_eq!(
                        outcome,
                        Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
                    );
                    assert!(!path.exists());
                    registry
                        .recover(None)
                        .map(|readback| readback.store)
                        .unwrap();
                } else {
                    assert_eq!(outcome.unwrap(), resolution);
                }
            }
            assert_eq!(calls, 2);
            assert_eq!(std::fs::read(&intent_path).unwrap(), retained_intent);
            assert_eq!(
                std::fs::read(&path).unwrap(),
                ConfigProjectionCodecV1::encode_canonical_json(&resolution).unwrap()
            );
            assert_eq!(
                registry
                    .publish_kernel_effect_resolution(
                        &resolution,
                        Some(&mut || { panic!("an exactly completed cleanup must not run twice") }),
                        None
                    )
                    .unwrap(),
                resolution
            );
            let mut conflicting = resolution.clone();
            conflicting.resolution_id = id("ekr_");
            conflicting.resolution_hash = hash_omitting(
                "substrate.e3.kernel-effect-resolution.v1",
                "resolution",
                &conflicting,
                "resolution_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_kernel_effect_resolution(
                    &conflicting,
                    Some(&mut || { panic!("a conflicting resolution must not mutate the kernel") }),
                    None
                ),
                Err(ConfigProjectionFailureV1::Conflict)
            );
        }

        #[test]
        fn e3_e_kernel_effect_runs_after_durable_readback_under_both_locks() {
            let (_temp, parent, registry, store) = test_registry();
            let fixture = PreparedGatewayFixture::new(&registry, &store, None);
            let root = parent.authority.join(CHILD_NAME);
            let original_path = root.join("kernel-effects/intents").join(format!(
                "{}.json",
                fixture.boundary.kernel_effect_intent_ref.effect_intent_id
            ));
            let mut intent: E3KernelEffectIntentV1 =
                ConfigProjectionCodecV1::decode_canonical_json(
                    &std::fs::read(original_path).unwrap(),
                )
                .unwrap();
            intent.effect_intent_id = id("eki_");
            intent.intent_hash = hash_omitting(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                &intent,
                "intent_hash",
            )
            .unwrap();
            let path = root
                .join("kernel-effects/intents")
                .join(format!("{}.json", intent.effect_intent_id));
            let expected = ConfigProjectionCodecV1::encode_canonical_json(&intent).unwrap();
            assert_eq!(
                registry.publish_kernel_effect_intent(
                    &intent,
                    Some(&mut |_| {
                        panic!("a boundary effect cannot precede its gateway identity")
                    }),
                    None
                ),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );
            assert!(!path.exists());
            let gateway_path = root
                .join("gateways")
                .join(format!("{}.json", fixture.gateway.gateway_instance_id));
            let gateway_bytes =
                ConfigProjectionCodecV1::encode_canonical_json(&fixture.gateway).unwrap();
            let mut called = false;
            let result = registry.publish_kernel_effect_intent(
                &intent,
                Some(&mut |observed_ref| {
                    called = true;
                    assert_eq!(observed_ref, &kernel_effect_intent_ref(&intent));
                    assert_eq!(std::fs::read(&path).unwrap(), expected);
                    assert_eq!(std::fs::read(&gateway_path).unwrap(), gateway_bytes);
                    for lock_path in [&parent.parent_lock, &root.join("lock")] {
                        let competing = File::open(lock_path).unwrap();
                        assert_eq!(
                            unsafe {
                                libc::flock(competing.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB)
                            },
                            -1,
                            "effect must run while both authority locks remain held"
                        );
                        assert_eq!(
                            std::io::Error::last_os_error().raw_os_error(),
                            Some(libc::EWOULDBLOCK)
                        );
                    }
                    Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
                }),
                Some(&fixture.gateway),
            );
            assert!(called);
            assert_eq!(
                result,
                Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture)
            );
            // A failed effect does not erase its crash-recovery intent or expose a projection.
            assert_eq!(std::fs::read(&path).unwrap(), expected);
            assert!(!root
                .join("series")
                .join(&intent.series_id)
                .join("head.json")
                .exists());
            registry
                .recover(None)
                .map(|readback| readback.store)
                .unwrap();
            assert_eq!(
                registry.publish_kernel_effect_intent(
                    &intent,
                    Some(&mut |_| {
                        panic!("a retained intent must not replay its effect after recovery")
                    }),
                    None
                ),
                Err(ConfigProjectionFailureV1::Conflict)
            );
            assert_eq!(
                registry
                    .publish_kernel_effect_intent(&intent, None, None)
                    .unwrap(),
                kernel_effect_intent_ref(&intent)
            );
            for lock_path in [&parent.parent_lock, &root.join("lock")] {
                let competing = File::open(lock_path).unwrap();
                assert_eq!(
                    unsafe { libc::flock(competing.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
                    0,
                    "failure must release both locks"
                );
            }
            let mut invalid = intent.clone();
            invalid.intent_hash = "00".repeat(32);
            assert!(registry
                .publish_kernel_effect_intent(
                    &invalid,
                    Some(&mut |_| {
                        panic!("an unauthenticated intent must never run a kernel effect")
                    }),
                    None
                )
                .is_err());

            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record, None).unwrap();
            let evidence = terminal_evidence(&store, &record, &projection_ref);
            seed_terminal_evidence(&registry, &evidence);
            let retirement = retirement(&record, &projection_ref, &evidence);
            seed_revoked_boundary(&registry, &record, &retirement);
            registry.retire(&retirement).unwrap();
            let mut retired_intent = intent.clone();
            retired_intent.effect_intent_id = id("eki_");
            retired_intent.series_id = record.identity.series_id;
            retired_intent.intent_hash = hash_omitting(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                &retired_intent,
                "intent_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_kernel_effect_intent(
                    &retired_intent,
                    Some(&mut |_| {
                        panic!("a retired series must never perform a new kernel effect")
                    }),
                    None
                ),
                Err(ConfigProjectionFailureV1::RetiredSeries)
            );
            assert!(!root
                .join("kernel-effects/intents")
                .join(format!("{}.json", retired_intent.effect_intent_id))
                .exists());
        }

        #[test]
        fn e3_e_kernel_effect_requires_exact_cgroup_registration_before_return() {
            for outcome in ["registered", "missing", "wrong-binding"] {
                let (_temp, parent, registry, store) = test_registry();
                let registration_id = id("ecg_");
                let parent_cgroup = crate::CanonicalCgroupIdentityV1 {
                    cgroup_v2_mount_device_id: 11,
                    cgroup_v2_mount_inode: 12,
                    cgroup_directory_inode: 13,
                    cgroup_relative_path: "substrate/world-7".to_string(),
                };
                let component = format!(
                    "substrate-e3-{}",
                    &ordinary_sha256(registration_id.as_bytes())[..24]
                );
                let relative = format!("{}/{}", parent_cgroup.cgroup_relative_path, component);
                let mut intent = E3KernelEffectIntentV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    series_id: id("cps_"),
                    effect_intent_id: id("eki_"),
                    preparation_id: id("e3p_"),
                    fence_id: id("cpf_"),
                    effect: E3KernelEffectKindV1::CreateChildCgroup {
                        cgroup_registration_id: registration_id.clone(),
                        role: crate::E3TerminalProcessRoleV1::ManagedGateway,
                        parent_cgroup: parent_cgroup.clone(),
                        child_component: component,
                        expected_relative_path: relative.clone(),
                    },
                    created_at: now_timestamp(),
                    intent_hash: String::new(),
                };
                intent.intent_hash = hash_omitting(
                    "substrate.e3.kernel-effect-intent.v1",
                    "intent",
                    &intent,
                    "intent_hash",
                )
                .unwrap();
                let mut registration = E3ChildCgroupRegistrationV1 {
                    schema_version: 1,
                    authority_store_id: store.authority_store_id.clone(),
                    series_id: intent.series_id.clone(),
                    cgroup_registration_id: registration_id.clone(),
                    kernel_effect_intent_ref: kernel_effect_intent_ref(&intent),
                    fence_id: intent.fence_id.clone(),
                    turn_id: None,
                    role: crate::E3TerminalProcessRoleV1::ManagedGateway,
                    cgroup: crate::CanonicalCgroupIdentityV1 {
                        cgroup_directory_inode: 14,
                        cgroup_relative_path: relative,
                        ..parent_cgroup
                    },
                    kernel_boot_id: Uuid::now_v7().to_string(),
                    registered_at: now_timestamp(),
                    cgroup_registration_hash: String::new(),
                };
                if outcome == "wrong-binding" {
                    registration.fence_id = id("cpf_");
                }
                registration.cgroup_registration_hash = hash_omitting(
                    "substrate.e3.child-cgroup-registration.v1",
                    "cgroup_registration",
                    &registration,
                    "cgroup_registration_hash",
                )
                .unwrap();
                let root = parent.authority.join(CHILD_NAME);
                let registration_path = root
                    .join("child-cgroups")
                    .join(&intent.series_id)
                    .join(format!("{registration_id}.json"));
                let mut calls = 0;
                let result = registry.publish_kernel_effect_intent(
                    &intent,
                    Some(&mut |reference| {
                        calls += 1;
                        assert_eq!(reference, &registration.kernel_effect_intent_ref);
                        assert!(root
                            .join("kernel-effects/intents")
                            .join(format!("{}.json", reference.effect_intent_id))
                            .is_file());
                        assert!(!registration_path.exists());
                        Ok((outcome != "missing").then(|| registration.clone()))
                    }),
                    None,
                );
                assert_eq!(calls, 1);
                if outcome == "registered" {
                    assert_eq!(result.unwrap(), kernel_effect_intent_ref(&intent));
                    assert_eq!(
                        std::fs::read(&registration_path).unwrap(),
                        ConfigProjectionCodecV1::encode_canonical_json(&registration).unwrap()
                    );
                    assert_eq!(
                        registry
                            .publish_child_cgroup_registration(&registration)
                            .unwrap(),
                        registration
                    );
                } else {
                    assert_eq!(result, Err(ConfigProjectionFailureV1::WrongBinding));
                    assert!(!registration_path.exists());
                }
                registry
                    .recover(None)
                    .map(|readback| readback.store)
                    .unwrap();
                assert_eq!(
                    registry.publish_kernel_effect_intent(
                        &intent,
                        Some(&mut |_| panic!("no effect replay after either success or failure")),
                        None
                    ),
                    Err(ConfigProjectionFailureV1::Conflict)
                );
            }
        }

        #[test]
        fn e3_d_registry_publishes_exact_kernel_and_child_identity_chain() {
            let (temp, _parent, registry, store) = test_registry();
            let root = temp.path().join("authority-v1").join(CHILD_NAME);
            let series_id = id("cps_");
            let record = test_record(&store, series_id.clone());
            let projection_ref = registry.publish_dormant(&record, None).unwrap();
            let cgroup_registration_id = id("ecg_");
            let parent_cgroup = crate::CanonicalCgroupIdentityV1 {
                cgroup_v2_mount_device_id: 11,
                cgroup_v2_mount_inode: 12,
                cgroup_directory_inode: 13,
                cgroup_relative_path: "substrate/world-7".to_string(),
            };
            let child_component = format!(
                "substrate-e3-{}",
                &ordinary_sha256(cgroup_registration_id.as_bytes())[..24]
            );
            let mut intent = crate::E3KernelEffectIntentV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                series_id: series_id.clone(),
                effect_intent_id: id("eki_"),
                preparation_id: id("e3p_"),
                fence_id: id("cpf_"),
                effect: crate::E3KernelEffectKindV1::CreateChildCgroup {
                    cgroup_registration_id: cgroup_registration_id.clone(),
                    role: crate::E3TerminalProcessRoleV1::Codex,
                    parent_cgroup: parent_cgroup.clone(),
                    child_component: child_component.clone(),
                    expected_relative_path: format!(
                        "{}/{}",
                        parent_cgroup.cgroup_relative_path, child_component
                    ),
                },
                created_at: now_timestamp(),
                intent_hash: String::new(),
            };
            intent.intent_hash = hash_omitting(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                &intent,
                "intent_hash",
            )
            .unwrap();
            let intent_ref = registry
                .publish_kernel_effect_intent(&intent, None, None)
                .expect("publish intent");
            assert_eq!(
                registry
                    .publish_kernel_effect_intent(&intent, None, None)
                    .unwrap(),
                intent_ref
            );
            move_final_to_temp(
                &root.join("kernel-effects/intents"),
                &format!("{}.json", intent.effect_intent_id),
                "kernel-effect-intent",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .expect("recover child-cgroup intent");

            let child_cgroup = crate::CanonicalCgroupIdentityV1 {
                cgroup_v2_mount_device_id: 11,
                cgroup_v2_mount_inode: 12,
                cgroup_directory_inode: 14,
                cgroup_relative_path: format!(
                    "{}/{}",
                    parent_cgroup.cgroup_relative_path, child_component
                ),
            };
            let mut cgroup = crate::E3ChildCgroupRegistrationV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                series_id: series_id.clone(),
                cgroup_registration_id: cgroup_registration_id.clone(),
                kernel_effect_intent_ref: intent_ref,
                fence_id: intent.fence_id.clone(),
                turn_id: Some(id("turn_")),
                role: crate::E3TerminalProcessRoleV1::Codex,
                cgroup: child_cgroup.clone(),
                kernel_boot_id: Uuid::now_v7().to_string(),
                registered_at: now_timestamp(),
                cgroup_registration_hash: String::new(),
            };
            cgroup.cgroup_registration_hash = hash_omitting(
                "substrate.e3.child-cgroup-registration.v1",
                "cgroup_registration",
                &cgroup,
                "cgroup_registration_hash",
            )
            .unwrap();
            registry
                .publish_child_cgroup_registration(&cgroup)
                .expect("publish child cgroup");
            move_final_to_temp(
                &root.join("child-cgroups").join(&series_id),
                &format!("{}.json", cgroup.cgroup_registration_id),
                "child-cgroup",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .expect("recover child-cgroup registration");

            let mut child = crate::E3ChildProcessRegistrationV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                series_id: series_id.clone(),
                registration_id: id("ecp_"),
                cgroup_registration_id: cgroup_registration_id.clone(),
                cgroup_registration_hash: cgroup.cgroup_registration_hash.clone(),
                fence_id: intent.fence_id,
                role: crate::E3TerminalProcessRoleV1::Codex,
                pid: std::process::id(),
                pid_start_time_ticks: 1,
                process_cgroup: child_cgroup.clone(),
                kernel_boot_id: cgroup.kernel_boot_id.clone(),
                parent_service_instance_id: id("wsi_"),
                registered_at: now_timestamp(),
                registration_hash: String::new(),
            };
            child.registration_hash = hash_omitting(
                "substrate.e3.child-process-registration.v1",
                "registration",
                &child,
                "registration_hash",
            )
            .unwrap();
            registry
                .publish_child_process_registration(&child)
                .expect("publish child process");
            move_final_to_temp(
                &root.join("child-processes").join(&series_id),
                &format!("{}.json", child.registration_id),
                "child-process",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .expect("recover child-process registration");

            let mut incomplete = crate::E3TerminalChildQuiescenceEvidenceV1 {
                schema_version: 1,
                authority_store_id: record.identity.authority_store_id.clone(),
                series_id: series_id.clone(),
                evidence_id: id("tce_"),
                final_projection_ref: projection_ref.clone(),
                world_id: record.identity.world_id.clone(),
                world_generation: record.identity.world_generation,
                ordered_terminal_processes: Vec::new(),
                ordered_empty_cgroups: Vec::new(),
                observed_at: now_timestamp(),
                evidence_hash: String::new(),
            };
            incomplete.evidence_hash = hash_omitting(
                "substrate.e3.terminal-child-quiescence.v1",
                "evidence",
                &incomplete,
                "evidence_hash",
            )
            .unwrap();
            assert_eq!(
                registry.publish_terminal_child_evidence(&incomplete),
                Err(ConfigProjectionFailureV1::WrongBinding)
            );

            let mut evidence = incomplete;
            evidence.evidence_id = id("tce_");
            evidence.ordered_terminal_processes = vec![crate::E3TerminalProcessObservationV1 {
                registration_id: child.registration_id.clone(),
                registration_hash: child.registration_hash.clone(),
                role: child.role,
                pid: child.pid,
                pid_start_time_ticks: child.pid_start_time_ticks,
                observation: crate::E3TerminalProcessObservationKindV1::ParentWaitid {
                    terminal_wait_status: 0,
                },
            }];
            evidence.ordered_empty_cgroups = vec![crate::E3TerminalCgroupQuiescenceV1 {
                cgroup_registration_id: cgroup.cgroup_registration_id.clone(),
                cgroup_registration_hash: cgroup.cgroup_registration_hash.clone(),
                role: cgroup.role,
                cgroup: cgroup.cgroup.clone(),
                cgroup_events_sha256: ordinary_sha256(b"populated 0\n"),
                cgroup_procs_sha256: ordinary_sha256(b""),
                populated: false,
                ordered_live_pids: Vec::new(),
            }];
            evidence.evidence_hash = hash_omitting(
                "substrate.e3.terminal-child-quiescence.v1",
                "evidence",
                &evidence,
                "evidence_hash",
            )
            .unwrap();
            registry
                .publish_terminal_child_evidence(&evidence)
                .expect("publish complete terminal-child evidence");
            move_final_to_temp(
                &root.join("terminal-child-evidence").join(&series_id),
                &format!("{}.json", evidence.evidence_id),
                "terminal-child",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .expect("recover terminal-child evidence");

            let access_boundary_id = id("gab_");
            let mut boundary_intent = crate::E3KernelEffectIntentV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id.clone(),
                series_id,
                effect_intent_id: id("eki_"),
                preparation_id: id("e3p_"),
                fence_id: id("cpf_"),
                effect: crate::E3KernelEffectKindV1::InstallGatewayBoundary {
                    table_name: format!(
                        "substrate_e3_{}",
                        &ordinary_sha256(access_boundary_id.as_bytes())[..24]
                    ),
                    access_boundary_id,
                    network_namespace_inode: 41,
                    chain_name: "gateway_output".to_string(),
                },
                created_at: now_timestamp(),
                intent_hash: String::new(),
            };
            boundary_intent.intent_hash = hash_omitting(
                "substrate.e3.kernel-effect-intent.v1",
                "intent",
                &boundary_intent,
                "intent_hash",
            )
            .unwrap();
            let boundary_intent_ref = registry
                .publish_kernel_effect_intent(&boundary_intent, None, None)
                .expect("publish boundary intent");
            let mut resolution = crate::E3KernelEffectResolutionV1 {
                schema_version: 1,
                authority_store_id: store.authority_store_id,
                resolution_id: id("ekr_"),
                effect_intent_ref: boundary_intent_ref,
                disposition: crate::E3KernelEffectResolutionDispositionV1::NoEffectObserved,
                observed_cgroup: None,
                observed_nftables_table_handle: None,
                resolved_at: now_timestamp(),
                resolution_hash: String::new(),
            };
            resolution.resolution_hash = hash_omitting(
                "substrate.e3.kernel-effect-resolution.v1",
                "resolution",
                &resolution,
                "resolution_hash",
            )
            .unwrap();
            registry
                .publish_kernel_effect_resolution(&resolution, None, None)
                .expect("publish no-effect recovery resolution");
            move_final_to_temp(
                &root.join("kernel-effects/resolutions"),
                &format!("{}.json", resolution.resolution_id),
                "kernel-effect-resolution",
            );
            registry
                .recover(None)
                .map(|readback| readback.store)
                .expect("recover exact E3-D publications");
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::*;
