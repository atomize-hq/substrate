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
        ConfigProjectionAuthoringInputRefV1, ConfigProjectionCodecV1,
        ConfigProjectionConsumerKindV1, ConfigProjectionConsumerLeasePostureV1,
        ConfigProjectionConsumerLeaseV1, ConfigProjectionFailureV1, ConfigProjectionHeadV1,
        ConfigProjectionIdentityV1, ConfigProjectionRefV1, ConfigProjectionRetirementV1,
        ConfigProjectionStoreV1, ConfigProjectionSubjectBindingV1, DirectoryPhysicalIdentityV1,
        E3ConfigExplainOriginKindV1, E3TerminalChildQuiescenceEvidenceV1,
        EffectiveSubstrateConfigSourceV1, GatewayAccessBoundaryRefV1, GatewayAccessBoundaryV1,
        GatewayAccessPostureV1, InstalledAcceptedHomeBootstrapHeadV1,
        InstalledAcceptedHomeBootstrapRecordV1, ManagedGatewayProjectionPostureV1,
        NativeAgentConfigProjectionV1, NativeProjectedFileRoleV1, NativeProjectionSourceManifestV1,
        NftablesRuleRoleV1, RuntimeArtifactAuthorityRoleV1, RuntimeArtifactProvenanceV1,
        SecretDeliveryMechanismV1, SecretHandoffStateV1, Timestamp,
        TrustedRuntimeArtifactManifestV1,
    };

    const CHILD_NAME: &str = "agent-config-projection-v1";
    const MAX_AUTHORITY_OBJECT_BYTES: u64 = 16 * 1024 * 1024;
    const OPENAT2_RESOLVE: u64 = libc::RESOLVE_BENEATH
        | libc::RESOLVE_NO_MAGICLINKS
        | libc::RESOLVE_NO_SYMLINKS
        | libc::RESOLVE_NO_XDEV;
    static LIVE_CAPABILITIES: OnceLock<Mutex<BTreeMap<(String, String), usize>>> = OnceLock::new();

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

        pub fn recover(&self) -> Result<ConfigProjectionStoreV1, ConfigProjectionFailureV1> {
            self.with_transaction(|transaction| {
                let store = ensure_store(transaction, None)?;
                validate_registry_tree(transaction, true)?;
                Ok(store)
            })
        }

        pub fn resolve(
            &self,
            identity: &ConfigProjectionIdentityV1,
            held_lease: &ConfigProjectionConsumerLeaseV1,
        ) -> Result<ConfigProjectionResolutionV1, ConfigProjectionFailureV1> {
            let parent = Arc::clone(&self.parent);
            self.with_transaction(|transaction| {
                let Some(store) =
                    transaction.read_resolution_object::<ConfigProjectionStoreV1>(&[
                        "store.json".to_string(),
                    ])?
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
            })
        }

        pub fn publish_dormant(
            &self,
            record: &AgentConfigProjectionRecordV1,
        ) -> Result<ConfigProjectionRefV1, ConfigProjectionFailureV1> {
            self.publish(
                record,
                record.predecessor_ref.as_ref(),
                ManagedGatewayProjectionPostureV1::Dormant,
            )
        }

        pub fn publish_ready_closed(
            &self,
            expected_head: &ConfigProjectionRefV1,
            record: &AgentConfigProjectionRecordV1,
        ) -> Result<ConfigProjectionRefV1, ConfigProjectionFailureV1> {
            self.publish(
                record,
                Some(expected_head),
                ManagedGatewayProjectionPostureV1::ReadyClosed,
            )
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

        pub fn import_runtime_artifacts(
            &self,
            effective_config: &EffectiveSubstrateConfigSourceV1,
            agent_inventory: &AgentInventorySourceMaterialV1,
            created_at: Timestamp,
        ) -> Result<ConfigProjectionAuthoringInputRefV1, ConfigProjectionFailureV1> {
            self.with_transaction(|transaction| {
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
            native: &NativeAgentConfigProjectionV1,
            created_at: Timestamp,
        ) -> Result<NativeProjectionSourceManifestV1, ConfigProjectionFailureV1> {
            self.with_transaction(|transaction| {
                let store = require_store(transaction)?;
                validate_native_projection_source_input(native, series_id, fence_id)?;
                validate_timestamp(&created_at)?;
                publish_native_source_directory(
                    transaction,
                    &store,
                    series_id,
                    fence_id,
                    native,
                    created_at,
                )
            })
        }

        pub fn acquire_consumer_lease(
            &self,
            acquired_projection_ref: &ConfigProjectionRefV1,
            consumer_kind: ConfigProjectionConsumerKindV1,
            acquired_at: Timestamp,
        ) -> Result<ConfigProjectionConsumerLeaseV1, ConfigProjectionFailureV1> {
            self.with_transaction(|transaction| {
                let store = require_store(transaction)?;
                let head = read_series_head(transaction, &acquired_projection_ref.series_id)?;
                if read_retirement(transaction, &acquired_projection_ref.series_id)?.is_some() {
                    return Err(ConfigProjectionFailureV1::RetiredSeries);
                }
                if head.head_ref != *acquired_projection_ref {
                    return Err(ConfigProjectionFailureV1::StaleRevision);
                }
                validate_timestamp(&acquired_at)?;
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
            self.with_transaction(|transaction| {
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

        pub fn retire(
            &self,
            retirement: &ConfigProjectionRetirementV1,
        ) -> Result<ConfigProjectionRetirementV1, ConfigProjectionFailureV1> {
            self.with_transaction(|transaction| {
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
                ensure_all_leases_released(transaction, &retirement.series_id)?;
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
            self.with_transaction(|transaction| {
                let store = ensure_store(
                    transaction,
                    Some(record.identity.authority_store_id.as_str()),
                )?;
                validate_record(record, &store)?;
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
                        if revision == expected.revision.saturating_add(1)
                            && predecessor == expected =>
                    {
                        publish_successor(transaction, &store, record, expected, &reference)?
                    }
                    _ => return Err(ConfigProjectionFailureV1::StaleRevision),
                }
                Ok(reference)
            })
        }

        fn with_transaction<T>(
            &self,
            operation: impl FnOnce(
                &mut ConfigProjectionChildTransactionV1,
            ) -> Result<T, ConfigProjectionFailureV1>,
        ) -> Result<T, ConfigProjectionFailureV1> {
            let mut operation = Some(operation);
            let mut captured = None;
            let mut callback = |authority_fd: BorrowedFd<'_>| {
                let result = ConfigProjectionChildTransactionV1::begin(authority_fd).and_then(
                    |mut transaction| {
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
                    },
                );
                captured = Some(result);
                Ok(())
            };
            self.parent.with_locked_parent(&mut callback)?;
            captured.ok_or(ConfigProjectionFailureV1::Conflict)?
        }
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
        observed_resolution_files: Vec<(Vec<String>, Vec<u8>)>,
        observed_newer_schema: bool,
    }

    impl ConfigProjectionChildTransactionV1 {
        fn begin(authority_fd: BorrowedFd<'_>) -> Result<Self, ConfigProjectionFailureV1> {
            let authority_metadata = fstat(authority_fd.as_raw_fd())?;
            if authority_metadata.st_mode & libc::S_IFMT != libc::S_IFDIR {
                return Err(ConfigProjectionFailureV1::UnsupportedSecurityPosture);
            }
            let root =
                open_or_create_directory(authority_fd, CHILD_NAME, authority_metadata.st_uid)?;
            verify_directory(&root, authority_metadata.st_uid, 0o700)?;
            let root_metadata = fstat(root.as_raw_fd())?;
            let lock =
                open_or_create_regular_file(root.as_fd(), "lock", authority_metadata.st_uid)?;
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
            "terminal-child-evidence",
            "retirement",
        ] {
            open_or_create_directory(transaction.root.as_fd(), name, transaction.owner_uid)?;
        }
        let inputs = open_directory_at(transaction.root.as_fd(), "inputs")?;
        for name in ["effective-config", "agent-inventory"] {
            open_or_create_directory(inputs.as_fd(), name, transaction.owner_uid)?;
        }
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
        if current.head_ref == *reference {
            let existing = read_record(transaction, reference)?;
            validate_record(&existing, store)?;
            return if existing == *record {
                Ok(())
            } else {
                Err(ConfigProjectionFailureV1::Conflict)
            };
        }
        if current.head_ref != *expected {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        let predecessor = read_record(transaction, &current.head_ref)?;
        validate_record(&predecessor, store)?;
        if predecessor.identity != record.identity {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        validate_record_transition(&predecessor, record)?;
        let records = open_directory_at(series.as_fd(), "records")?;
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
            head_revision: current.head_revision + 1,
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
        if successor.revision != predecessor.revision.saturating_add(1)
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
        native: &NativeAgentConfigProjectionV1,
        created_at: Timestamp,
    ) -> Result<NativeProjectionSourceManifestV1, ConfigProjectionFailureV1> {
        let native_sources = open_directory_at(transaction.root.as_fd(), "native-sources")?;
        let series =
            open_or_create_directory(native_sources.as_fd(), series_id, transaction.owner_uid)?;
        let config = &native.files[0];
        let config_bytes = STANDARD
            .decode(&config.bytes_base64)
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        match open_directory_at(series.as_fd(), fence_id) {
            Ok(_) => {
                let existing = validate_native_source_directory(
                    &series,
                    fence_id,
                    None,
                    Some(&config_bytes),
                    transaction.owner_uid,
                )?;
                let expected_physical_path = format!(
                    "{}/authority-v1/agent-config-projection-v1/native-sources/{series_id}/{fence_id}",
                    store.accepted_home.physical_path
                );
                if existing.authority_store_id == store.authority_store_id
                    && existing.series_id == series_id
                    && existing.fence_id == fence_id
                    && existing.source_root.physical_path == expected_physical_path
                    && existing.native_projection_hash == native.projection_hash
                    && existing.ordered_file_hashes == [config.sha256.clone()]
                    && existing.created_at == created_at
                {
                    return Ok(existing);
                }
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {}
            Err(error) => return Err(error),
        }
        let temp_name = format!(".e3-native-source-tmp.{}", Uuid::now_v7());
        let temp = open_or_create_directory(series.as_fd(), &temp_name, transaction.owner_uid)?;
        let codex_home =
            open_or_create_directory(temp.as_fd(), "codex-home", transaction.owner_uid)?;
        let system_empty =
            open_or_create_directory(temp.as_fd(), "system-empty", transaction.owner_uid)?;
        if !list_names(&system_empty)?.is_empty() {
            return Err(ConfigProjectionFailureV1::Conflict);
        }
        write_exclusive_file(
            codex_home.as_fd(),
            "config.toml",
            &config_bytes,
            transaction.owner_uid,
        )?;

        let temp_metadata = fstat(temp.as_raw_fd())?;
        if native.root.owner_uid != transaction.owner_uid as u64
            || native.root.owner_gid != temp_metadata.st_gid as u64
        {
            return Err(ConfigProjectionFailureV1::WrongBinding);
        }
        let source_root = CanonicalDirectoryV1 {
            physical_path: format!(
                "{}/authority-v1/agent-config-projection-v1/native-sources/{series_id}/{fence_id}",
                store.accepted_home.physical_path
            ),
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
            ordered_file_hashes: vec![config.sha256.clone()],
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
        write_exclusive_file(
            temp.as_fd(),
            "source-manifest.json",
            &manifest_bytes,
            transaction.owner_uid,
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
                    Some(&config_bytes),
                    transaction.owner_uid,
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
            Some(&config_bytes),
            transaction.owner_uid,
        )?;
        if readback == manifest {
            Ok(manifest)
        } else {
            Err(ConfigProjectionFailureV1::PartialPublication)
        }
    }

    fn write_exclusive_file(
        parent: BorrowedFd<'_>,
        name: &str,
        bytes: &[u8],
        owner_uid: libc::uid_t,
    ) -> Result<(), ConfigProjectionFailureV1> {
        let mut file = open_file_at(
            parent,
            name,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        verify_regular_file(&file, owner_uid, 0o600)?;
        file.write_all(bytes)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        fsync(&file)?;
        fsync_fd(parent.as_raw_fd())
    }

    fn validate_native_source_directory(
        series: &File,
        name: &str,
        expected_manifest: Option<&NativeProjectionSourceManifestV1>,
        expected_config: Option<&[u8]>,
        owner_uid: libc::uid_t,
    ) -> Result<NativeProjectionSourceManifestV1, ConfigProjectionFailureV1> {
        let root = open_directory_at(series.as_fd(), name)?;
        verify_directory(&root, owner_uid, 0o700)?;
        if list_names(&root)? != ["codex-home", "source-manifest.json", "system-empty"] {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let codex_home = open_directory_at(root.as_fd(), "codex-home")?;
        let system_empty = open_directory_at(root.as_fd(), "system-empty")?;
        if list_names(&codex_home)? != ["config.toml"] || !list_names(&system_empty)?.is_empty() {
            return Err(ConfigProjectionFailureV1::PartialPublication);
        }
        let config = read_file_at(codex_home.as_fd(), "config.toml")?;
        let manifest: NativeProjectionSourceManifestV1 =
            read_canonical_at(root.as_fd(), "source-manifest.json")?;
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
            )?;
            match open_directory_at(series.as_fd(), &candidate.fence_id) {
                Ok(_) => {
                    let final_manifest = validate_native_source_directory(
                        &series,
                        &candidate.fence_id,
                        Some(&candidate),
                        None,
                        transaction.owner_uid,
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
        let mut prior_process_key = None;
        for observation in &evidence.ordered_terminal_processes {
            validate_prefixed_uuid(&observation.registration_id, "ecp_")?;
            validate_sha256(&observation.registration_hash)?;
            if observation.pid == 0 || observation.pid_start_time_ticks == 0 {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            validate_terminal_observation_shape(observation)?;
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
        let mut prior_cgroup_key = None;
        for entry in &evidence.ordered_empty_cgroups {
            validate_prefixed_uuid(&entry.cgroup_registration_id, "ecg_")?;
            validate_sha256(&entry.cgroup_registration_hash)?;
            validate_sha256(&entry.cgroup_events_sha256)?;
            validate_sha256(&entry.cgroup_procs_sha256)?;
            validate_cgroup_identity(&entry.cgroup)?;
            if entry.populated || !entry.ordered_live_pids.is_empty() {
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
            if lease.posture != ConfigProjectionConsumerLeasePostureV1::Released {
                return Err(ConfigProjectionFailureV1::Conflict);
            }
            validate_lease(&lease, &lease.acquired_projection_ref)?;
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

    fn open_or_create_directory(
        parent: BorrowedFd<'_>,
        name: &str,
        owner_uid: libc::uid_t,
    ) -> Result<File, ConfigProjectionFailureV1> {
        validate_component(name)?;
        match open_directory_at(parent, name) {
            Ok(directory) => {
                verify_directory(&directory, owner_uid, 0o700)?;
                Ok(directory)
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {
                let name_c =
                    CString::new(name).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
                if unsafe { libc::mkdirat(parent.as_raw_fd(), name_c.as_ptr(), 0o700) } != 0 {
                    let error = std::io::Error::last_os_error();
                    if error.raw_os_error() != Some(libc::EEXIST) {
                        return Err(ConfigProjectionFailureV1::PartialPublication);
                    }
                }
                let directory = open_directory_at(parent, name)?;
                verify_directory(&directory, owner_uid, 0o700)?;
                fsync_fd(parent.as_raw_fd())?;
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
                verify_regular_file(&file, owner_uid, 0o600)?;
                Ok(file)
            }
            Err(ConfigProjectionFailureV1::MissingPreparation) => {
                let file = open_file_at(
                    parent,
                    name,
                    libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
                    0o600,
                )?;
                verify_regular_file(&file, owner_uid, 0o600)?;
                fsync(&file)?;
                fsync_fd(parent.as_raw_fd())?;
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
        verify_regular_file(&temp, owner_uid, 0o600)?;
        temp.write_all(bytes)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        fsync(&temp)?;
        rename_noreplace(parent, &temp_name, final_name)?;
        fsync_fd(parent.as_raw_fd())?;
        if read_file_at(parent, final_name)? != bytes {
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
        verify_regular_file(&temp, owner_uid, 0o600)?;
        temp.write_all(replacement)
            .map_err(|_| ConfigProjectionFailureV1::PartialPublication)?;
        fsync(&temp)?;
        let rechecked = match read_file_at(parent, "head.json") {
            Ok(bytes) => Some(bytes),
            Err(ConfigProjectionFailureV1::MissingPreparation) => None,
            Err(error) => return Err(error),
        };
        if rechecked.as_deref() != expected {
            return Err(ConfigProjectionFailureV1::StaleRevision);
        }
        if expected.is_none() {
            rename_noreplace(parent, &temp_name, "head.json")?;
        } else {
            rename_replace(parent, &temp_name, "head.json")?;
        }
        fsync_fd(parent.as_raw_fd())?;
        if read_file_at(parent, "head.json")? != replacement {
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
                ensure_all_leases_released(transaction, &retirement.series_id)?;
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
    ) -> Result<(), ConfigProjectionFailureV1> {
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
        validate_series_tree(transaction, verify_objects)?;
        validate_lease_tree(transaction, verify_objects)?;
        validate_gateway_boundary_tree(transaction, verify_objects)?;
        validate_terminal_evidence_tree(transaction, verify_objects)?;
        validate_retirement_tree(transaction, verify_objects)?;
        Ok(())
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

    fn validate_terminal_evidence_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
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
                    }
                }
                let _ = read_file_at(series.as_fd(), &name)?;
            }
        }
        Ok(())
    }

    fn validate_gateway_boundary_tree(
        transaction: &ConfigProjectionChildTransactionV1,
        verify_objects: bool,
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
                    ensure_all_leases_released(transaction, series_id)?;
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
                    | "lease"
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
            let store = registry.recover().unwrap();
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
                        "preparation_id": "preparation", "selected_backend_id": "cli:codex-world",
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
                kernel_effect_intent_ref: crate::E3KernelEffectIntentRefV1 {
                    authority_store_id: record.identity.authority_store_id.clone(),
                    effect_intent_id: "eki_01890f3e-7b8c-7a11-8c55-0242ac120002".to_string(),
                    intent_hash: digest(),
                },
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

        #[test]
        fn e3_b_registry_first_writer_and_expected_head_cas_are_strict() {
            let (_temp, parent, registry, store) = test_registry();
            let dormant = test_record(&store, id("cps_"));
            let dormant_ref = registry.publish_dormant(&dormant).unwrap();
            assert_eq!(registry.publish_dormant(&dormant).unwrap(), dormant_ref);

            let ready = successor(&dormant, ManagedGatewayProjectionPostureV1::ReadyClosed);
            let ready_ref = registry.publish_ready_closed(&dormant_ref, &ready).unwrap();
            assert_eq!(
                registry.publish_ready_closed(&dormant_ref, &ready).unwrap(),
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
            let projection_ref = registry.publish_dormant(&record).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                )
                .unwrap();

            assert!(matches!(
                registry.resolve(&record.identity, &held).unwrap(),
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
            let reference = registry
                .with_transaction(|transaction| {
                    publish_authoring_inputs(transaction, &store, &effective, &inventory, &manifest)
                })
                .unwrap();
            reference.validate().unwrap();

            let child = temp.path().join("authority-v1").join(CHILD_NAME);
            let effective_path = child
                .join("inputs/effective-config")
                .join(format!("{}.json", effective.source_hash));
            let interrupted = effective_path.parent().unwrap().join(temp_name("input"));
            std::fs::rename(&effective_path, &interrupted).unwrap();
            registry.recover().unwrap();
            assert!(effective_path.is_file());

            let malformed = effective_path.parent().unwrap().join(temp_name("input"));
            std::fs::write(&malformed, b"{").unwrap();
            std::fs::set_permissions(&malformed, std::fs::Permissions::from_mode(0o600)).unwrap();
            assert!(registry.recover().is_err());
            assert!(malformed.is_file());
        }

        #[test]
        fn e3_b_registry_resolve_reports_only_complete_unsigned_newer_schema() {
            let (temp, _parent, registry, store) = test_registry();
            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
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
                registry.resolve(&record.identity, &held).unwrap(),
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
                let Err(error) = registry.resolve(&record.identity, &held) else {
                    panic!("invalid discriminator unexpectedly resolved");
                };
                assert_eq!(error, ConfigProjectionFailureV1::Malformed);
            }
        }

        #[test]
        fn e3_b_registry_resolve_binds_subject_record_and_held_lease() {
            let (temp, _parent, registry, store) = test_registry();
            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                )
                .unwrap();
            let capability = match registry.resolve(&record.identity, &held).unwrap() {
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
                registry.resolve(&record.identity, &unheld),
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
                registry.resolve(&record.identity, &held),
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
                    let code = i32::from(child_registry.publish_dormant(&record).is_err());
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
            let cas_base = test_record(&store, id("cps_"));
            let cas_base_ref = registry.publish_dormant(&cas_base).unwrap();
            let cas_lease = registry
                .acquire_consumer_lease(
                    &cas_base_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                )
                .unwrap();
            assert!(matches!(
                registry
                    .resolve(&test_record(&store, id("cps_")).identity, &cas_lease)
                    .unwrap(),
                ConfigProjectionResolutionV1::Missing
            ));
            let first_successor =
                successor(&cas_base, ManagedGatewayProjectionPostureV1::ReadyClosed);
            let second_successor =
                successor(&cas_base, ManagedGatewayProjectionPostureV1::ReadyClosed);
            let mut cas_children = Vec::new();
            for record in [first_successor, second_successor] {
                let child = unsafe { libc::fork() };
                assert!(child >= 0);
                if child == 0 {
                    let child_registry = ConfigProjectionRegistryV1::open(parent.clone()).unwrap();
                    let code = match child_registry.publish_ready_closed(&cas_base_ref, &record) {
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
            let projection_ref = registry.publish_dormant(&dormant).unwrap();
            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
                )
                .unwrap();
            let resolution = registry.resolve(&dormant.identity, &held).unwrap();
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
                registry.resolve(&dormant.identity, &held).unwrap(),
                ConfigProjectionResolutionV1::Retired { .. }
            ));
            assert_eq!(
                registry.acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:04:00.000000Z".to_string())
                ),
                Err(ConfigProjectionFailureV1::RetiredSeries)
            );
            assert_eq!(
                registry.publish_dormant(&dormant),
                Err(ConfigProjectionFailureV1::RetiredSeries)
            );
        }

        fn seed_revoked_boundary(
            registry: &ConfigProjectionRegistryV1,
            record: &AgentConfigProjectionRecordV1,
            retirement: &ConfigProjectionRetirementV1,
        ) {
            let initial = dormant_boundary(record);
            assert_eq!(
                boundary_ref(&initial),
                record.managed_gateway.access_boundary_ref
            );
            let revoked = successor_boundary(&initial, GatewayAccessPostureV1::Revoked);
            assert_eq!(boundary_ref(&revoked), retirement.revoked_boundary_ref);
            registry
                .with_transaction(|transaction| {
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
                .with_transaction(|transaction| {
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
                registry.with_transaction::<()>(|_| Err(ConfigProjectionFailureV1::Conflict)),
                Err(ConfigProjectionFailureV1::Conflict)
            );
            let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = registry.with_transaction::<()>(|_| panic!("intentional child unwind"));
            }));
            assert!(unwind.is_err());
            assert_eq!(registry.recover().unwrap(), store);
            let child = unsafe { libc::fork() };
            assert!(child >= 0);
            if child == 0 {
                registry
                    .with_transaction::<()>(|_| unsafe { libc::_exit(0) })
                    .unwrap();
                unreachable!();
            }
            let mut status = 0;
            assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
            assert_eq!(libc::WEXITSTATUS(status), 0);
            assert_eq!(registry.recover().unwrap(), store);

            let store_bytes = ConfigProjectionCodecV1::encode_canonical_json(&store).unwrap();
            registry
                .with_transaction(|transaction| {
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
            registry.recover().unwrap();
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
                registry.recover(),
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
            assert_eq!(registry.recover().unwrap(), store);

            std::fs::remove_file(child_root.join("store.json")).unwrap();
            let mut invalid = store.clone();
            invalid.store_hash = digest();
            let invalid_bytes = ConfigProjectionCodecV1::encode_canonical_json(&invalid).unwrap();
            let invalid_temp = child_root.join(temp_name("store"));
            std::fs::write(&invalid_temp, invalid_bytes).unwrap();
            std::fs::set_permissions(&invalid_temp, std::fs::Permissions::from_mode(0o600))
                .unwrap();
            assert_eq!(
                registry.recover(),
                Err(ConfigProjectionFailureV1::HashInvalid)
            );
            assert!(!child_root.join("store.json").exists());
            assert!(invalid_temp.exists());
        }

        #[test]
        fn e3_b_registry_recovers_each_admitted_publication_boundary() {
            let (temp, _parent, registry, store) = test_registry();
            let record = test_record(&store, id("cps_"));
            let projection_ref = registry.publish_dormant(&record).unwrap();
            let root = temp.path().join("authority-v1").join(CHILD_NAME);

            let subject = subject_hash(&record.identity).unwrap();
            move_final_to_temp(
                &root.join("subjects"),
                &format!("{subject}.json"),
                "subject",
            );
            registry.recover().unwrap();

            let records = root
                .join("series")
                .join(&record.identity.series_id)
                .join("records");
            move_final_to_temp(&records, &record_name(&record), "record");
            registry.recover().unwrap();

            let series = root.join("series").join(&record.identity.series_id);
            move_final_to_temp(&series, "head.json", "head");
            registry.recover().unwrap();

            let held = registry
                .acquire_consumer_lease(
                    &projection_ref,
                    ConfigProjectionConsumerKindV1::MemberDispatchV2,
                    Timestamp("2026-09-10T00:01:00.000000Z".to_string()),
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
            registry.recover().unwrap();
            move_final_to_temp(&consumer, "head.json", "head");
            registry.recover().unwrap();

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
            registry.recover().unwrap();

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
            registry.recover().unwrap();

            let retirement = retirement(&record, &projection_ref, &evidence);
            seed_revoked_boundary(&registry, &record, &retirement);
            registry.retire(&retirement).unwrap();
            move_final_to_temp(
                &root.join("retirement"),
                &format!("{}.json", record.identity.series_id),
                "retirement",
            );
            registry.recover().unwrap();
            assert!(matches!(
                registry.resolve(&record.identity, &held).unwrap(),
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
            let result = registry.with_transaction::<()>(|transaction| {
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
            other_registry.publish_dormant(&dormant).unwrap();
            let subject = subject_hash(&dormant.identity).unwrap();
            let subject_path = other_registry
                .with_transaction(|transaction| {
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
    }
}

#[cfg(target_os = "linux")]
pub use linux::*;
