//! Durable local request arrival. These receipts carry unverified sender claims;
//! only the authenticated active Resume materializer can publish canonical work.
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::execution::agent_runtime::{
    host_inbox::{HostInboxMaterializationState, HostInboxRecord},
    host_session_authority::{
        canonical_json,
        schema::{AuthorityObjectCommitmentV1, CanonicalDirectoryV1},
        store::{
            BootstrapError, DirectHostInboxStorageAreaV1 as Area, DirectHostInboxTransactionV1,
        },
    },
};

pub(crate) const MAX_DIRECT_HOST_RECEIPTS: usize = 4096;
pub(crate) const MAX_DIRECT_HOST_RECEIPT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectHostInboxReceiptRefV1 {
    pub(crate) ordinal: u64,
    pub(crate) receipt_id: String,
    pub(crate) commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(crate) enum DirectHostInboxArrivalV1 {
    LocalRequest {
        raw_file_name: String,
        request_bytes: Vec<u8>,
    },
    InvalidArtifact {
        artifact_name: String,
        failure_bytes: Vec<u8>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectHostInboxReceiptV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) ordinal: u64,
    pub(crate) receipt_id: String,
    pub(crate) arrival: DirectHostInboxArrivalV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectHostInboxSourceHeadV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) previous_head: Option<AuthorityObjectCommitmentV1>,
    pub(crate) receipts: Vec<DirectHostInboxReceiptRefV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectHostInboxSourcePointerV1 {
    schema_version: u32,
    head: AuthorityObjectCommitmentV1,
}

/// A complete physical snapshot of the committed receipt prefix. No completeness
/// or canonical-meaning claim follows from an absent pointer or empty prefix.
#[derive(Clone, Debug)]
pub(crate) struct DirectHostInboxSourceReadV1 {
    pub(crate) head_commitment: Option<AuthorityObjectCommitmentV1>,
    pub(crate) receipts: Vec<(DirectHostInboxReceiptRefV1, DirectHostInboxReceiptV1)>,
    pointer_bytes: Option<Vec<u8>>,
}

pub(crate) fn direct_host_inbox_commitment<T: Serialize>(
    domain: &str,
    value: &T,
) -> Result<AuthorityObjectCommitmentV1, BootstrapError> {
    let bytes = canonical_json::to_vec(value)
        .map_err(|_| BootstrapError::direct_host_inbox("encode strict direct-host inbox object"))?;
    let mut hash = Sha256::new();
    hash.update(domain.as_bytes());
    hash.update([0]);
    hash.update(bytes);
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: format!("{:x}", hash.finalize()),
    })
}

pub(crate) fn direct_host_inbox_commitment_file(
    commitment: &AuthorityObjectCommitmentV1,
) -> Result<String, BootstrapError> {
    match commitment {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex }
            if digest_hex.len() == 64
                && digest_hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) =>
        {
            Ok(format!("{digest_hex}.json"))
        }
        _ => Err(BootstrapError::direct_host_inbox(
            "direct-host inbox requires a canonical SHA256 commitment",
        )),
    }
}

fn inbox_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, BootstrapError> {
    canonical_json::to_vec(value)
        .map_err(|_| BootstrapError::direct_host_inbox("encode direct-host inbox publication"))
}

fn inbox_decode<T: serde::de::DeserializeOwned + Serialize>(
    bytes: &[u8],
) -> Result<T, BootstrapError> {
    canonical_json::from_slice(bytes).map_err(|_| {
        BootstrapError::direct_host_inbox("decode strict direct-host inbox publication")
    })
}

fn inbox_request_bytes(record: &HostInboxRecord) -> Result<Vec<u8>, BootstrapError> {
    let value = serde_json::to_value(record)
        .map_err(|_| BootstrapError::direct_host_inbox("encode typed local inbox request"))?;
    inbox_bytes(&value)
}

fn inbox_request_decode(bytes: &[u8]) -> Result<HostInboxRecord, BootstrapError> {
    let value: serde_json::Value = inbox_decode(bytes)?;
    let record: HostInboxRecord = serde_json::from_value(value.clone())
        .map_err(|_| BootstrapError::direct_host_inbox("decode typed local inbox request"))?;
    if serde_json::to_value(&record)
        .map_err(|_| BootstrapError::direct_host_inbox("encode typed local inbox request"))?
        != value
    {
        return Err(BootstrapError::direct_host_inbox(
            "local inbox request has unknown or omitted fields",
        ));
    }
    Ok(record)
}

fn inbox_identity(
    tx: &DirectHostInboxTransactionV1<'_, '_>,
) -> Result<(String, CanonicalDirectoryV1), BootstrapError> {
    let root = tx
        .authority_root()
        .ok_or(BootstrapError::direct_host_inbox(
            "receipt requires activated authority",
        ))?;
    if root.bootstrap_home() != tx.bootstrap_home() {
        return Err(BootstrapError::direct_host_inbox(
            "receipt authority physical identity changed",
        ));
    }
    Ok((
        root.authority_store_id().to_owned(),
        tx.bootstrap_home().clone(),
    ))
}

pub(crate) fn read_direct_host_inbox_source(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
) -> Result<DirectHostInboxSourceReadV1, BootstrapError> {
    let (store, home) = inbox_identity(tx)?;
    let pointer_bytes = tx.read_file(Area::Source, &["source-head.json"])?;
    let Some(bytes) = &pointer_bytes else {
        return Ok(DirectHostInboxSourceReadV1 {
            head_commitment: None,
            receipts: Vec::new(),
            pointer_bytes,
        });
    };
    let pointer: DirectHostInboxSourcePointerV1 = inbox_decode(bytes)?;
    if pointer.schema_version != 1 {
        return Err(BootstrapError::direct_host_inbox(
            "unsupported direct-host inbox source pointer",
        ));
    }
    let name = direct_host_inbox_commitment_file(&pointer.head)?;
    let bytes = tx
        .read_file(Area::Source, &["source-heads", &name])?
        .ok_or(BootstrapError::direct_host_inbox(
            "committed direct-host inbox source head is absent",
        ))?;
    let head: DirectHostInboxSourceHeadV1 = inbox_decode(&bytes)?;
    if head.schema_version != 1
        || head.authority_store_id != store
        || head.bootstrap_home != home
        || head.receipts.len() > MAX_DIRECT_HOST_RECEIPTS
        || direct_host_inbox_commitment("substrate.direct-host-inbox.source-head.v1", &head)?
            != pointer.head
    {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host inbox source head identity or bound changed",
        ));
    }
    verify_direct_host_inbox_source_history(tx, &head)?;
    let mut receipts = Vec::with_capacity(head.receipts.len());
    let mut ids = std::collections::BTreeSet::new();
    for (index, reference) in head.receipts.into_iter().enumerate() {
        if reference.ordinal != index as u64 + 1 || !ids.insert(reference.receipt_id.clone()) {
            return Err(BootstrapError::direct_host_inbox(
                "direct-host inbox receipt prefix is not uniquely contiguous",
            ));
        }
        let name = direct_host_inbox_commitment_file(&reference.commitment)?;
        let bytes = tx.read_file(Area::Source, &["receipts", &name])?.ok_or(
            BootstrapError::direct_host_inbox("committed direct-host inbox receipt is absent"),
        )?;
        if bytes.len() > MAX_DIRECT_HOST_RECEIPT_BYTES {
            return Err(BootstrapError::direct_host_inbox(
                "direct-host inbox receipt exceeds its byte bound",
            ));
        }
        let receipt: DirectHostInboxReceiptV1 = inbox_decode(&bytes)?;
        if receipt.schema_version != 1
            || receipt.authority_store_id != store
            || receipt.bootstrap_home != home
            || receipt.ordinal != reference.ordinal
            || receipt.receipt_id != reference.receipt_id
            || direct_host_inbox_commitment("substrate.direct-host-inbox.receipt.v1", &receipt)?
                != reference.commitment
        {
            return Err(BootstrapError::direct_host_inbox(
                "committed direct-host inbox receipt identity changed",
            ));
        }
        receipts.push((reference, receipt));
    }
    tx.verify()?;
    Ok(DirectHostInboxSourceReadV1 {
        head_commitment: Some(pointer.head),
        receipts,
        pointer_bytes,
    })
}

/// Traverse the entire bounded immutable source-head closure, including older
/// objects not needed to read the current receipt vector. Missing history is
/// unknown even when the current and immediately previous objects are intact.
fn verify_direct_host_inbox_source_history(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    head: &DirectHostInboxSourceHeadV1,
) -> Result<(), BootstrapError> {
    let mut current = head.clone();
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..=MAX_DIRECT_HOST_RECEIPTS {
        let Some(previous) = &current.previous_head else {
            return if current.receipts.len() <= 1 {
                Ok(())
            } else {
                Err(BootstrapError::direct_host_inbox(
                    "direct-host inbox source history has no exact origin",
                ))
            };
        };
        let name = direct_host_inbox_commitment_file(previous)?;
        if !seen.insert(name.clone()) {
            return Err(BootstrapError::direct_host_inbox(
                "direct-host inbox source history is cyclic",
            ));
        }
        let bytes = tx
            .read_file(Area::Source, &["source-heads", &name])?
            .ok_or(BootstrapError::direct_host_inbox(
                "older committed inbox source head is absent",
            ))?;
        let old: DirectHostInboxSourceHeadV1 = inbox_decode(&bytes)?;
        if old.schema_version != 1
            || old.authority_store_id != head.authority_store_id
            || old.bootstrap_home != head.bootstrap_home
            || old.receipts.len().checked_add(1) != Some(current.receipts.len())
            || current.receipts[..old.receipts.len()] != old.receipts
            || direct_host_inbox_commitment("substrate.direct-host-inbox.source-head.v1", &old)?
                != *previous
        {
            return Err(BootstrapError::direct_host_inbox(
                "direct-host inbox source history is not an exact immutable prefix",
            ));
        }
        current = old;
    }
    Err(BootstrapError::direct_host_inbox(
        "direct-host inbox source history exceeds its bound",
    ))
}

fn publish_inbox_immutable<T: Serialize>(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    directory: &str,
    domain: &str,
    value: &T,
) -> Result<AuthorityObjectCommitmentV1, BootstrapError> {
    let commitment = direct_host_inbox_commitment(domain, value)?;
    let name = direct_host_inbox_commitment_file(&commitment)?;
    let bytes = inbox_bytes(value)?;
    match tx.read_file(Area::Source, &[directory, &name])? {
        Some(existing) if existing == bytes => {}
        Some(_) => {
            return Err(BootstrapError::direct_host_inbox(
                "conflicting immutable direct-host inbox publication",
            ))
        }
        None => tx.write_file(Area::Source, &[directory, &name], None, &bytes)?,
    }
    Ok(commitment)
}

/// Arrival commits only at source-head publication. An immutable object or raw
/// file left by an interrupted publication is an orphan, never an arrival.
pub(crate) fn publish_direct_host_inbox_arrival(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    receipt_id: &str,
    arrival: DirectHostInboxArrivalV1,
) -> Result<DirectHostInboxReceiptRefV1, BootstrapError> {
    HostInboxRecord::validate_record_id(receipt_id)
        .map_err(|_| BootstrapError::direct_host_inbox("invalid direct-host inbox receipt ID"))?;
    let source = read_direct_host_inbox_source(tx)?;
    if let Some((reference, old)) = source
        .receipts
        .iter()
        .find(|(_, r)| r.receipt_id == receipt_id)
    {
        if old.arrival == arrival {
            tx.verify()?;
            return Ok(reference.clone());
        }
        return Err(BootstrapError::direct_host_inbox(
            "direct-host inbox receipt ID already names different bytes",
        ));
    }
    if source.receipts.len() >= MAX_DIRECT_HOST_RECEIPTS {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host inbox receipt count exceeds its bound",
        ));
    }
    let (authority_store_id, bootstrap_home) = inbox_identity(tx)?;
    let receipt = DirectHostInboxReceiptV1 {
        schema_version: 1,
        authority_store_id: authority_store_id.clone(),
        bootstrap_home: bootstrap_home.clone(),
        ordinal: source.receipts.len() as u64 + 1,
        receipt_id: receipt_id.to_owned(),
        arrival,
    };
    if inbox_bytes(&receipt)?.len() > MAX_DIRECT_HOST_RECEIPT_BYTES {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host inbox receipt exceeds its byte bound",
        ));
    }
    if let DirectHostInboxArrivalV1::LocalRequest {
        raw_file_name,
        request_bytes,
    } = &receipt.arrival
    {
        let record = inbox_request_decode(request_bytes)?;
        record
            .validate()
            .map_err(|_| BootstrapError::direct_host_inbox("invalid typed local inbox request"))?;
        if record.materialization_state != HostInboxMaterializationState::Pending
            || receipt_id != record.record_id
            || *raw_file_name != format!("{}.json", record.record_id)
        {
            return Err(BootstrapError::direct_host_inbox(
                "direct-host inbox arrival is not a pending local request",
            ));
        }
        match tx.read_file(Area::RawInbox, &[raw_file_name])? {
            Some(old) if old == *request_bytes => {}
            Some(_) => {
                return Err(BootstrapError::direct_host_inbox(
                    "raw direct-host inbox ID already names different bytes",
                ))
            }
            None => tx.write_file(Area::RawInbox, &[raw_file_name], None, request_bytes)?,
        }
    }
    let commitment = publish_inbox_immutable(
        tx,
        "receipts",
        "substrate.direct-host-inbox.receipt.v1",
        &receipt,
    )?;
    let reference = DirectHostInboxReceiptRefV1 {
        ordinal: receipt.ordinal,
        receipt_id: receipt_id.to_owned(),
        commitment,
    };
    let mut refs = source
        .receipts
        .into_iter()
        .map(|(r, _)| r)
        .collect::<Vec<_>>();
    refs.push(reference.clone());
    let head = DirectHostInboxSourceHeadV1 {
        schema_version: 1,
        authority_store_id,
        bootstrap_home,
        previous_head: source.head_commitment,
        receipts: refs,
    };
    let head = publish_inbox_immutable(
        tx,
        "source-heads",
        "substrate.direct-host-inbox.source-head.v1",
        &head,
    )?;
    #[cfg(test)]
    direct_host_publication_boundary_for_test("receipt-before-head")?;
    tx.write_file(
        Area::Source,
        &["source-head.json"],
        source.pointer_bytes.as_deref(),
        &inbox_bytes(&DirectHostInboxSourcePointerV1 {
            schema_version: 1,
            head,
        })?,
    )?;
    tx.verify()?;
    Ok(reference)
}

pub(crate) fn publish_direct_host_inbox_request(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    record: &HostInboxRecord,
) -> Result<DirectHostInboxReceiptRefV1, BootstrapError> {
    publish_direct_host_inbox_arrival(
        tx,
        &record.record_id,
        DirectHostInboxArrivalV1::LocalRequest {
            raw_file_name: format!("{}.json", record.record_id),
            request_bytes: inbox_request_bytes(record)?,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_runtime::{
        host_session_authority::store::{bootstrap, with_locked_direct_host_inbox_transaction},
        obligation_ledger::OrchestrationObligationKind,
    };
    use std::os::unix::fs::PermissionsExt as _;

    fn receipt_home() -> tempfile::TempDir {
        let parent = std::env::var_os("XDG_RUNTIME_DIR").expect("owned nonwriting fixture parent");
        let home = tempfile::tempdir_in(parent).unwrap();
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        bootstrap(home.path()).unwrap();
        home
    }

    fn request(id: &str) -> HostInboxRecord {
        HostInboxRecord::new(
            "session",
            id,
            OrchestrationObligationKind::RuntimeAlert,
            "unverified request",
            "local-host",
            "local-input",
            "sender-claim",
        )
    }

    #[test]
    fn test_direct_host_inbox_receipt_exact_retry_after_later_arrival() {
        let home = receipt_home();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            let one = request("request-one");
            let first = publish_direct_host_inbox_request(tx, &one)?;
            publish_direct_host_inbox_request(tx, &request("request-two"))?;
            assert_eq!(publish_direct_host_inbox_request(tx, &one)?, first);
            let source = read_direct_host_inbox_source(tx)?;
            assert_eq!(source.receipts.len(), 2);
            assert_eq!(source.receipts[0].0, first);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_direct_host_inbox_receipt_conflict_preserves_original() {
        let home = receipt_home();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            let mut one = request("request-one");
            let original = publish_direct_host_inbox_request(tx, &one)?;
            one.summary.push_str(" changed");
            assert!(publish_direct_host_inbox_request(tx, &one).is_err());
            assert_eq!(read_direct_host_inbox_source(tx)?.receipts[0].0, original);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_direct_host_inbox_receipt_orphan_is_not_arrival() {
        let home = receipt_home();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            let bytes = inbox_request_bytes(&request("orphan"))?;
            tx.write_file(Area::RawInbox, &["orphan.json"], None, &bytes)?;
            let source = read_direct_host_inbox_source(tx)?;
            assert!(source.head_commitment.is_none());
            assert!(source.receipts.is_empty());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_direct_host_inbox_receipt_oversize_refuses_before_raw_publication() {
        let home = receipt_home();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            let mut one = request("too-large");
            one.summary = "x".repeat(MAX_DIRECT_HOST_RECEIPT_BYTES);
            assert!(publish_direct_host_inbox_request(tx, &one).is_err());
            assert!(tx.read_file(Area::RawInbox, &["too-large.json"])?.is_none());
            assert!(read_direct_host_inbox_source(tx)?.head_commitment.is_none());
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_direct_host_inbox_receipt_missing_committed_object_is_unknown() {
        let home = receipt_home();
        let name = with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            let reference = publish_direct_host_inbox_request(tx, &request("one"))?;
            direct_host_inbox_commitment_file(&reference.commitment)
        })
        .unwrap();
        std::fs::remove_file(
            home.path()
                .join("direct-host-inbox-ledger-v1/receipts")
                .join(name),
        )
        .unwrap();
        assert!(
            with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
                read_direct_host_inbox_source(tx)
            })
            .is_err()
        );
    }

    #[test]
    fn test_direct_host_inbox_receipt_copied_source_has_wrong_physical_identity() {
        let first = receipt_home();
        let second = receipt_home();
        with_locked_direct_host_inbox_transaction(first.path(), None, |tx| {
            publish_direct_host_inbox_request(tx, &request("one"))?;
            Ok(())
        })
        .unwrap();
        let pointer = std::fs::read(
            first
                .path()
                .join("direct-host-inbox-ledger-v1/source-head.json"),
        )
        .unwrap();
        with_locked_direct_host_inbox_transaction(second.path(), None, |tx| {
            tx.write_file(Area::Source, &["source-head.json"], None, &pointer)?;
            assert!(read_direct_host_inbox_source(tx).is_err());
            Ok(())
        })
        .unwrap();
    }
    #[test]
    fn test_direct_host_inbox_receipt_missing_older_head_is_unknown() {
        let home = receipt_home();
        let first_head = with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            publish_direct_host_inbox_request(tx, &request("one"))?;
            let first = read_direct_host_inbox_source(tx)?.head_commitment.unwrap();
            publish_direct_host_inbox_request(tx, &request("two"))?;
            publish_direct_host_inbox_request(tx, &request("three"))?;
            assert_eq!(read_direct_host_inbox_source(tx)?.receipts.len(), 3);
            direct_host_inbox_commitment_file(&first)
        })
        .unwrap();
        std::fs::remove_file(
            home.path()
                .join("direct-host-inbox-ledger-v1/source-heads")
                .join(first_head),
        )
        .unwrap();
        let error = with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            read_direct_host_inbox_source(tx)
        })
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "older committed inbox source head is absent"
        );
    }

    // This creates raw-source infrastructure only. It cannot enroll a session
    // or create an authenticated birth/canonical producer capability.
    fn unenrolled_zero_source(
        tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    ) -> Result<(), BootstrapError> {
        let (authority_store_id, bootstrap_home) = inbox_identity(tx)?;
        let head = DirectHostInboxSourceHeadV1 {
            schema_version: 1,
            authority_store_id,
            bootstrap_home,
            previous_head: None,
            receipts: Vec::new(),
        };
        let head = publish_inbox_immutable(
            tx,
            "source-heads",
            "substrate.direct-host-inbox.source-head.v1",
            &head,
        )?;
        tx.write_file(
            Area::Source,
            &["source-head.json"],
            None,
            &inbox_bytes(&DirectHostInboxSourcePointerV1 {
                schema_version: 1,
                head,
            })?,
        )
    }

    #[test]
    fn test_direct_host_inbox_receipt_explicit_zero_prefix_is_not_enrollment() {
        let home = receipt_home();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            unenrolled_zero_source(tx)?;
            let source = read_direct_host_inbox_source(tx)?;
            assert!(source.head_commitment.is_some());
            assert!(source.receipts.is_empty());
            direct_host_source_inventory_complete(tx, &source, &[])
        })
        .unwrap();
        assert_eq!(
            read_direct_host_inbox_session(home.path(), "session")
                .unwrap_err()
                .to_string(),
            "direct-host session has no authenticated genesis"
        );
    }

    #[test]
    fn test_direct_host_inbox_receipt_orphan_inventory_is_unknown() {
        let home = receipt_home();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            unenrolled_zero_source(tx)?;
            tx.write_file(
                Area::Source,
                &["receipts", "uncommitted.json"],
                None,
                b"orphan",
            )?;
            let source = read_direct_host_inbox_source(tx)?;
            assert_eq!(
                direct_host_source_inventory_complete(tx, &source, &[])
                    .unwrap_err()
                    .to_string(),
                "orphan direct-host receipt prevents complete source coverage"
            );
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_direct_host_inbox_receipt_canonical_source_refuses_noncontiguous_origin() {
        let home=receipt_home();
        with_locked_direct_host_inbox_transaction(home.path(),None,|tx| {
            let (store,physical)=inbox_identity(tx)?;
            let reference=DirectHostInboxReceiptRefV1 {ordinal:2,receipt_id:"bad-origin".into(),
                commitment:direct_host_inbox_commitment("raw-only-unverified",&"no-receipt")?};
            let head=DirectHostInboxSourceHeadV1 {schema_version:1,authority_store_id:store,
                bootstrap_home:physical,previous_head:None,receipts:vec![reference]};
            let commitment=publish_inbox_immutable(tx,"source-heads","substrate.direct-host-inbox.source-head.v1",&head)?;
            let error=read_direct_host_source_at(tx,&commitment,1).unwrap_err();
            assert!(error.to_string().contains("not uniquely contiguous"));
            assert!(read_direct_host_inbox_source(tx)?.head_commitment.is_none(),"an uncommitted source object is not an arrival");
            Ok(())
        }).unwrap();
    }

    fn physical_snapshot(
        home: &std::path::Path,
    ) -> std::collections::BTreeMap<String, (bool, u32, u64, String)> {
        let mut result = std::collections::BTreeMap::new();
        let mut pending = vec![home.to_path_buf()];
        while let Some(path) = pending.pop() {
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            let directory = metadata.is_dir();
            let digest = if directory {
                String::new()
            } else {
                format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()))
            };
            result.insert(
                path.strip_prefix(home)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                (
                    directory,
                    metadata.permissions().mode(),
                    if directory { 0 } else { metadata.len() },
                    digest,
                ),
            );
            if directory {
                pending.extend(
                    std::fs::read_dir(&path)
                        .unwrap()
                        .map(|entry| entry.unwrap().path()),
                );
            }
        }
        result
    }

    #[test]
    fn test_direct_host_inbox_receipt_readonly_refuses_without_creation() {
        let parent = receipt_home();
        let absent = parent.path().join("absent-home");
        let before = physical_snapshot(parent.path());
        assert!(read_direct_host_inbox_session(&absent, "session").is_err());
        assert!(!absent.exists());
        assert_eq!(physical_snapshot(parent.path()), before);
        let empty = parent.path().join("empty-home");
        std::fs::create_dir(&empty).unwrap();
        std::fs::set_permissions(&empty, std::fs::Permissions::from_mode(0o700)).unwrap();
        let before = physical_snapshot(&empty);
        assert!(read_direct_host_inbox_session(&empty, "session").is_err());
        assert_eq!(physical_snapshot(&empty), before);
        let home = receipt_home();
        let lock = home.path().join("authority-v1/lock/root.lock");
        std::fs::remove_file(&lock).unwrap();
        let before = physical_snapshot(home.path());
        assert!(read_direct_host_inbox_session(home.path(), "session").is_err());
        assert!(!lock.exists());
        assert_eq!(physical_snapshot(home.path()), before);
    }

    #[test]
    fn test_direct_host_inbox_receipt_readonly_has_no_repair_or_write_power() {
        use crate::execution::agent_runtime::host_session_authority::store::with_existing_direct_host_inbox_snapshot;
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt as _;
        let home = receipt_home();
        let temp = home
            .path()
            .join("authority-v1/tmp/root--r999--11111111111111111111111111111111.tmp");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)
            .unwrap();
        file.write_all(b"uncommitted root candidate").unwrap();
        file.sync_all().unwrap();
        let before = physical_snapshot(home.path());
        with_existing_direct_host_inbox_snapshot(home.path(), |tx| {
            assert_eq!(
                tx.write_file(Area::Source, &["forbidden.json"], None, b"no")
                    .unwrap_err()
                    .to_string(),
                "direct-host snapshot has no mutation authority"
            );
            Ok(())
        })
        .unwrap();
        assert_eq!(physical_snapshot(home.path()), before);
        assert!(read_direct_host_inbox_session(home.path(), "session").is_err());
        assert_eq!(physical_snapshot(home.path()), before);
        assert_eq!(std::fs::read(temp).unwrap(), b"uncommitted root candidate");
    }

    #[test]
    fn test_direct_host_inbox_receipt_readonly_detects_root_lock_replacement() {
        use crate::execution::agent_runtime::host_session_authority::store::with_existing_direct_host_inbox_snapshot;
        use std::os::unix::fs::MetadataExt as _;
        let home = receipt_home();
        let lock = home.path().join("authority-v1/lock/root.lock");
        let original_inode = std::fs::metadata(&lock).unwrap().ino();
        let authority_path = home.path().join("authority-v1/state-root-v1.json");
        let authority_bytes = std::fs::read(&authority_path).unwrap();
        let saved = home.path().join("saved-root-lock");
        let result = with_existing_direct_host_inbox_snapshot(home.path(), |tx| {
            assert!(tx.authority_root().is_some());
            std::fs::rename(&lock, &saved).unwrap();
            std::fs::write(&lock, b"replacement").unwrap();
            std::fs::set_permissions(&lock, std::fs::Permissions::from_mode(0o600)).unwrap();
            assert_ne!(std::fs::metadata(&lock).unwrap().ino(), original_inode);
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read(authority_path).unwrap(), authority_bytes);
        assert_eq!(std::fs::read(lock).unwrap(), b"replacement");
        assert_eq!(std::fs::metadata(saved).unwrap().ino(), original_inode);
        assert!(!home.path().join("direct-host-inbox-ledger-v1").exists());
    }
}

const DIRECT_HOST_SOURCE_PROTOCOL: &str = "substrate.direct-host-inbox.resume-only.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectHostInitialInventoryEntryV1 {
    file_name: String,
    byte_length: u64,
    raw_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectHostInboxGenesisV1 {
    schema_version: u32,
    source_protocol: String,
    birth: crate::execution::agent_runtime::host_session_authority::direct_host_resume::DirectHostBirthBindingV1,
    initial_source_head: AuthorityObjectCommitmentV1,
    initial_receipt_high_water: u64,
    initial_inventory: Option<Vec<DirectHostInitialInventoryEntryV1>>,
    initial_inventory_complete: bool,
    legacy_canonical_sources_absent: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectHostCanonicalRecordReadV1 {
    pub(crate) obligation_id: String,
    pub(crate) obligation_revision: u64,
    pub(crate) canonical_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) original_receipt_commitment: AuthorityObjectCommitmentV1,
    pub(crate) original_publication_commitment: AuthorityObjectCommitmentV1,
    pub(crate) unverified_request: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DirectHostInboxSessionReadV1 {
    pub(crate) birth_binding: crate::execution::agent_runtime::host_session_authority::direct_host_resume::DirectHostBirthBindingV1,
    pub(crate) source_protocol: String,
    pub(crate) initial_inventory_complete: bool,
    pub(crate) initial_receipt_high_water: u64,
    pub(crate) genesis_commitment: AuthorityObjectCommitmentV1,
    pub(crate) source_head_commitment: Option<AuthorityObjectCommitmentV1>,
    pub(crate) ledger_head_commitment: AuthorityObjectCommitmentV1,
    pub(crate) ledger_revision: u64,
    pub(crate) records: Vec<DirectHostCanonicalRecordReadV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectHostInboxLedgerHeadV1 {
    schema_version: u32,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    orchestration_session_id: String,
    genesis_commitment: AuthorityObjectCommitmentV1,
    previous_head: Option<AuthorityObjectCommitmentV1>,
    ledger_revision: u64,
    source_head_commitment: AuthorityObjectCommitmentV1,
    receipt_high_water: u64,
    publication_commitment: Option<AuthorityObjectCommitmentV1>,
    records: Vec<DirectHostCanonicalRecordReadV1>,
}

fn direct_host_session_key(session_id: &str) -> Result<String, BootstrapError> {
    if session_id.trim().is_empty() || session_id != session_id.trim() {
        return Err(BootstrapError::direct_host_inbox(
            "invalid direct-host enrolled session ID",
        ));
    }
    direct_host_inbox_commitment_file(&direct_host_inbox_commitment(
        "substrate.direct-host-inbox.session-key.v1",
        &session_id,
    )?)
}

fn read_inbox_immutable<T: serde::de::DeserializeOwned + Serialize>(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    directory: &str,
    domain: &str,
    commitment: &AuthorityObjectCommitmentV1,
) -> Result<T, BootstrapError> {
    let name = direct_host_inbox_commitment_file(commitment)?;
    let bytes = tx.read_file(Area::Source, &[directory, &name])?.ok_or(
        BootstrapError::direct_host_inbox("committed direct-host ledger object is absent"),
    )?;
    let value: T = inbox_decode(&bytes)?;
    if direct_host_inbox_commitment(domain, &value)? != *commitment {
        return Err(BootstrapError::direct_host_inbox(
            "immutable direct-host ledger commitment changed",
        ));
    }
    Ok(value)
}

fn read_inbox_pointer(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    key: &str,
    name: &str,
) -> Result<Option<AuthorityObjectCommitmentV1>, BootstrapError> {
    tx.read_file(Area::Source, &["sessions", key, name])?
        .map(|bytes| {
            let p: DirectHostInboxSourcePointerV1 = inbox_decode(&bytes)?;
            if p.schema_version != 1 {
                return Err(BootstrapError::direct_host_inbox(
                    "unsupported direct-host session pointer",
                ));
            }
            direct_host_inbox_commitment_file(&p.head)?;
            Ok(p.head)
        })
        .transpose()
}

fn initialize_inbox_pointer(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    key: &str,
    name: &str,
    commitment: &AuthorityObjectCommitmentV1,
) -> Result<(), BootstrapError> {
    match read_inbox_pointer(tx, key, name)? {
        Some(old) if old == *commitment => Ok(()),
        Some(_) => Err(BootstrapError::direct_host_inbox(
            "conflicting immutable direct-host session pointer",
        )),
        None => tx.write_file(
            Area::Source,
            &["sessions", key, name],
            None,
            &inbox_bytes(&DirectHostInboxSourcePointerV1 {
                schema_version: 1,
                head: commitment.clone(),
            })?,
        ),
    }
}

/// Enrollment establishes an explicit empty source head; source absence itself
/// never certifies a clean prefix. Only the authenticated pre-barrier birth
/// proof can initialize this global zero-receipt head.
fn ensure_enrollment_source_head(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    birth: &crate::execution::agent_runtime::host_session_authority::direct_host_resume::AuthenticatedDirectHostBirthV1,
) -> Result<DirectHostInboxSourceReadV1, BootstrapError> {
    let source = read_direct_host_inbox_source(tx)?;
    if source.head_commitment.is_some() {
        return Ok(source);
    }
    if !birth.may_initialize() {
        return Err(BootstrapError::direct_host_inbox(
            "missing source genesis after Start submission barrier",
        ));
    }
    let binding = birth.binding();
    let head = DirectHostInboxSourceHeadV1 {
        schema_version: 1,
        authority_store_id: binding.authority_store_id.clone(),
        bootstrap_home: binding.bootstrap_home.clone(),
        previous_head: None,
        receipts: Vec::new(),
    };
    let head = publish_inbox_immutable(
        tx,
        "source-heads",
        "substrate.direct-host-inbox.source-head.v1",
        &head,
    )?;
    tx.write_file(
        Area::Source,
        &["source-head.json"],
        None,
        &inbox_bytes(&DirectHostInboxSourcePointerV1 {
            schema_version: 1,
            head,
        })?,
    )?;
    read_direct_host_inbox_source(tx)
}

/// Validate the whole physical ingress/source prefix. A stray file, orphan,
/// invalid marker or missing object is unknown, never an implicit zero cut.
fn direct_host_source_inventory_complete(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    source: &DirectHostInboxSourceReadV1,
    raw: &[(String, Vec<u8>)],
) -> Result<(), BootstrapError> {
    let mut expected_raw = std::collections::BTreeMap::new();
    let mut receipt_names = std::collections::BTreeSet::new();
    for (reference, receipt) in &source.receipts {
        receipt_names.insert(direct_host_inbox_commitment_file(&reference.commitment)?);
        let DirectHostInboxArrivalV1::LocalRequest {
            raw_file_name,
            request_bytes,
        } = &receipt.arrival
        else {
            return Err(BootstrapError::direct_host_inbox(
                "invalid direct-host source artifact has unknown disposition",
            ));
        };
        let request = inbox_request_decode(request_bytes)?;
        request.validate().map_err(|_| {
            BootstrapError::direct_host_inbox("invalid committed direct-host local request")
        })?;
        if request.materialization_state != HostInboxMaterializationState::Pending
            || request.record_id != receipt.receipt_id
            || *raw_file_name != format!("{}.json", request.record_id)
            || expected_raw
                .insert(raw_file_name.clone(), request_bytes.clone())
                .is_some()
        {
            return Err(BootstrapError::direct_host_inbox(
                "ambiguous direct-host source request identity",
            ));
        }
    }
    if raw
        .iter()
        .cloned()
        .collect::<std::collections::BTreeMap<_, _>>()
        != expected_raw
    {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host raw inventory is not the exhaustive committed receipt prefix",
        ));
    }
    let actual_receipts = tx
        .list_files(Area::Source, &["receipts"])?
        .into_iter()
        .map(|(n, _)| n)
        .collect::<std::collections::BTreeSet<_>>();
    if actual_receipts != receipt_names {
        return Err(BootstrapError::direct_host_inbox(
            "orphan direct-host receipt prevents complete source coverage",
        ));
    }
    let mut head_names = std::collections::BTreeSet::new();
    let mut next = source.head_commitment.clone();
    for _ in 0..=MAX_DIRECT_HOST_RECEIPTS {
        let Some(commitment) = next.take() else {
            break;
        };
        let name = direct_host_inbox_commitment_file(&commitment)?;
        if !head_names.insert(name) {
            return Err(BootstrapError::direct_host_inbox(
                "cyclic direct-host source head inventory",
            ));
        }
        let head: DirectHostInboxSourceHeadV1 = read_inbox_immutable(
            tx,
            "source-heads",
            "substrate.direct-host-inbox.source-head.v1",
            &commitment,
        )?;
        next = head.previous_head;
    }
    if next.is_some()
        || tx
            .list_files(Area::Source, &["source-heads"])?
            .into_iter()
            .map(|(n, _)| n)
            .collect::<std::collections::BTreeSet<_>>()
            != head_names
    {
        return Err(BootstrapError::direct_host_inbox(
            "orphan or incomplete direct-host source head inventory",
        ));
    }
    tx.verify()
}

pub(crate) fn enroll_direct_host_inbox_session(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    birth: &crate::execution::agent_runtime::host_session_authority::direct_host_resume::AuthenticatedDirectHostBirthV1,
) -> Result<(), BootstrapError> {
    birth.verify_workspace()?;
    let binding = birth.binding();
    let (store, home) = inbox_identity(tx)?;
    if binding.authority_store_id != store || binding.bootstrap_home != home {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host enrollment proof belongs to another physical store",
        ));
    }
    let session_id = &binding.birth_authority.orchestration_session_id;
    if tx.direct_host_prior_canonical_source_present(session_id)? {
        return Err(BootstrapError::direct_host_inbox(
            "unexplained existing canonical source prevents direct-host enrollment",
        ));
    }
    let key = direct_host_session_key(session_id)?;
    let existing = read_inbox_pointer(tx, &key, "genesis.json")?;
    verify_direct_host_genesis_inventory(tx, session_id, existing.as_ref())?;
    let (genesis_commitment, genesis) = if let Some(commitment) = existing {
        let genesis: DirectHostInboxGenesisV1 = read_inbox_immutable(
            tx,
            "genesis",
            "substrate.direct-host-inbox.genesis.v1",
            &commitment,
        )?;
        if genesis.birth != *binding
            || genesis.schema_version != 1
            || genesis.source_protocol != DIRECT_HOST_SOURCE_PROTOCOL
        {
            return Err(BootstrapError::direct_host_inbox(
                "existing direct-host genesis has different authenticated birth",
            ));
        }
        (commitment, genesis)
    } else {
        if !birth.may_initialize() {
            return Err(BootstrapError::direct_host_inbox(
                "missing direct-host genesis after Start submission barrier",
            ));
        }
        let source = ensure_enrollment_source_head(tx, birth)?;
        let raw = tx.list_files(Area::RawInbox, &[]).ok();
        let complete = raw
            .as_ref()
            .is_some_and(|files| direct_host_source_inventory_complete(tx, &source, files).is_ok());
        let inventory = raw.map(|files| {
            files
                .into_iter()
                .map(|(file_name, bytes)| DirectHostInitialInventoryEntryV1 {
                    file_name,
                    byte_length: bytes.len() as u64,
                    raw_sha256: format!("{:x}", Sha256::digest(&bytes)),
                })
                .collect()
        });
        let genesis = DirectHostInboxGenesisV1 {
            schema_version: 1,
            source_protocol: DIRECT_HOST_SOURCE_PROTOCOL.into(),
            birth: binding.clone(),
            initial_source_head: source.head_commitment.ok_or(
                BootstrapError::direct_host_inbox("enrolled source head is absent"),
            )?,
            initial_receipt_high_water: source.receipts.len() as u64,
            initial_inventory: inventory,
            initial_inventory_complete: complete,
            legacy_canonical_sources_absent: true,
        };
        let commitment = publish_inbox_immutable(
            tx,
            "genesis",
            "substrate.direct-host-inbox.genesis.v1",
            &genesis,
        )?;
        initialize_inbox_pointer(tx, &key, "genesis.json", &commitment)?;
        (commitment, genesis)
    };
    if let Some(head_commitment) = read_inbox_pointer(tx, &key, "ledger-head.json")? {
        let head: DirectHostInboxLedgerHeadV1 = read_inbox_immutable(
            tx,
            "ledger-heads",
            "substrate.direct-host-inbox.ledger-head.v1",
            &head_commitment,
        )?;
        if head.schema_version != 1
            || head.authority_store_id != store
            || head.bootstrap_home != home
            || head.orchestration_session_id != *session_id
            || head.genesis_commitment != genesis_commitment
            || head.ledger_revision != 0
            || head.previous_head.is_some()
            || head.publication_commitment.is_some()
            || !head.records.is_empty()
            || head.receipt_high_water != 0
            || head.source_head_commitment != genesis.initial_source_head
        {
            return Err(BootstrapError::direct_host_inbox(
                "existing direct-host initial ledger head is invalid",
            ));
        }
        verify_direct_host_genesis_source(tx, &genesis)?;
        birth.verify_workspace()?;
        return tx.verify();
    }
    if !birth.may_initialize() {
        return Err(BootstrapError::direct_host_inbox(
            "missing direct-host initial ledger head after submission barrier",
        ));
    }
    let head = DirectHostInboxLedgerHeadV1 {
        schema_version: 1,
        authority_store_id: store,
        bootstrap_home: home,
        orchestration_session_id: session_id.clone(),
        genesis_commitment,
        previous_head: None,
        ledger_revision: 0,
        source_head_commitment: genesis.initial_source_head,
        receipt_high_water: 0,
        publication_commitment: None,
        records: Vec::new(),
    };
    let commitment = publish_inbox_immutable(
        tx,
        "ledger-heads",
        "substrate.direct-host-inbox.ledger-head.v1",
        &head,
    )?;
    initialize_inbox_pointer(tx, &key, "ledger-head.json", &commitment)?;
    birth.verify_workspace()?;
    tx.verify()
}

fn verify_direct_host_genesis_source(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    genesis: &DirectHostInboxGenesisV1,
) -> Result<(), BootstrapError> {
    let initial: DirectHostInboxSourceHeadV1 = read_inbox_immutable(
        tx,
        "source-heads",
        "substrate.direct-host-inbox.source-head.v1",
        &genesis.initial_source_head,
    )?;
    if !genesis.legacy_canonical_sources_absent
        || initial.schema_version != 1
        || initial.authority_store_id != genesis.birth.authority_store_id
        || initial.bootstrap_home != genesis.birth.bootstrap_home
        || initial.receipts.len() as u64 != genesis.initial_receipt_high_water
    {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host genesis source prefix context changed",
        ));
    }
    verify_direct_host_inbox_source_history(tx, &initial)
}

/// Inspect committed canonical provenance without creating or repairing state.
/// Historical publications use exact durable joins, never old helper guards.
pub(crate) fn read_direct_host_inbox_session(
    home: &std::path::Path,
    session_id: &str,
) -> Result<DirectHostInboxSessionReadV1, BootstrapError> {
    crate::execution::agent_runtime::host_session_authority::store::with_existing_direct_host_inbox_snapshot(home,|tx| {
        let read=read_direct_host_enrolled_ledger_locked(tx,session_id)?;
        tx.verify()?;
        Ok(DirectHostInboxSessionReadV1 {birth_binding:read.genesis.birth,source_protocol:read.genesis.source_protocol,
            initial_inventory_complete:read.genesis.initial_inventory_complete,initial_receipt_high_water:read.genesis.initial_receipt_high_water,
            genesis_commitment:read.genesis_commitment,source_head_commitment:Some(read.head.source_head_commitment),
            ledger_head_commitment:read.head_commitment,ledger_revision:read.head.ledger_revision,records:read.head.records})
    })
}

/// Durable semantics are intentionally confined to pending local attention.
/// Sender participant, router and review claims remain in unverified_request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectHostPendingRequestV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    genesis_commitment: AuthorityObjectCommitmentV1,
    original_receipt: DirectHostInboxReceiptRefV1,
    obligation_id: String,
    obligation_revision: u64,
    kind: super::OrchestrationObligationKind,
    #[serde(serialize_with = "serialize_direct_host_pending_severity")]
    severity: super::OrchestrationObligationSeverity,
    summary: String,
    attention_required: bool,
    state: String,
    unverified_request: serde_json::Value,
}

fn serialize_direct_host_pending_severity<S: serde::Serializer>(
    severity: &super::OrchestrationObligationSeverity,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use super::OrchestrationObligationSeverity;
    serializer.serialize_str(match severity {
        OrchestrationObligationSeverity::Info => "info",
        OrchestrationObligationSeverity::Warning => "warning",
        OrchestrationObligationSeverity::Error => "error",
    })
}

#[cfg(test)]
#[test]
fn test_direct_host_inbox_pending_record_severity_canonical_roundtrip() {
    use super::{OrchestrationObligationKind, OrchestrationObligationSeverity};

    #[derive(Serialize)]
    struct UnadaptedSeverity {
        severity: OrchestrationObligationSeverity,
    }

    // Preserve the general codec's refusal; only this pending-record field has
    // a closed local string representation.
    assert_eq!(
        canonical_json::to_vec(&UnadaptedSeverity {
            severity: OrchestrationObligationSeverity::Info,
        })
        .unwrap_err()
        .to_string(),
        "A1 enum tags are valid only inside a kind field"
    );
    let commitment = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "0".repeat(64),
    };
    for severity in [
        OrchestrationObligationSeverity::Info,
        OrchestrationObligationSeverity::Warning,
        OrchestrationObligationSeverity::Error,
    ] {
        let record = DirectHostPendingRequestV1 {
            schema_version: 1,
            authority_store_id: "encoding-only-store".into(),
            orchestration_session_id: "encoding-only-session".into(),
            genesis_commitment: commitment.clone(),
            original_receipt: DirectHostInboxReceiptRefV1 {
                ordinal: 1,
                receipt_id: "encoding-only-receipt".into(),
                commitment: commitment.clone(),
            },
            obligation_id: "encoding-only-obligation".into(),
            obligation_revision: 1,
            kind: OrchestrationObligationKind::FollowUpRequired,
            severity,
            summary: "pending attention".into(),
            attention_required: true,
            state: "pending".into(),
            unverified_request: serde_json::json!({
                "kind": "follow_up_required",
                "severity": severity,
                "source_participant_id": "unverified-participant-claim",
                "causation_request_id": "unverified-request-claim"
            }),
        };
        let bytes = canonical_json::to_vec(&record).expect("local pending record encoding");
        assert_eq!(
            canonical_json::from_slice::<DirectHostPendingRequestV1>(&bytes).unwrap(),
            record
        );
        let mut unknown = serde_json::to_value(&record).unwrap();
        unknown["severity"] = serde_json::json!("unknown-severity");
        let unknown_bytes = canonical_json::to_vec(&unknown).unwrap();
        assert!(canonical_json::from_slice::<DirectHostPendingRequestV1>(&unknown_bytes).is_err());
    }
}


#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectHostInboxPublicationV1 {
    schema_version: u32,
    genesis_commitment: AuthorityObjectCommitmentV1,
    binding:
        crate::execution::agent_runtime::host_session_authority::schema::DirectHostResumeBindingV1,
    previous_ledger_head: AuthorityObjectCommitmentV1,
    ledger_revision: u64,
    source_head_commitment: AuthorityObjectCommitmentV1,
    receipt_high_water: u64,
    new_records: Vec<AuthorityObjectCommitmentV1>,
}

struct DirectHostEnrolledLedgerReadV1 {
    genesis_commitment: AuthorityObjectCommitmentV1,
    genesis: DirectHostInboxGenesisV1,
    head_commitment: AuthorityObjectCommitmentV1,
    head: DirectHostInboxLedgerHeadV1,
}

fn direct_host_pending_request(
    genesis: &DirectHostInboxGenesisV1,
    genesis_commitment: &AuthorityObjectCommitmentV1,
    reference: &DirectHostInboxReceiptRefV1,
    receipt: &DirectHostInboxReceiptV1,
) -> Result<Option<DirectHostPendingRequestV1>, BootstrapError> {
    let DirectHostInboxArrivalV1::LocalRequest { request_bytes, .. } = &receipt.arrival else {
        return Err(BootstrapError::direct_host_inbox(
            "invalid artifact has no canonical pending-request interpretation",
        ));
    };
    let request = inbox_request_decode(request_bytes)?;
    request
        .ensure_pending_materialization_candidate()
        .map_err(|_| {
            BootstrapError::direct_host_inbox("source request is not a valid pending local request")
        })?;
    if request.orchestration_session_id != genesis.birth.birth_authority.orchestration_session_id {
        return Ok(None);
    }
    let local_host = gethostname::gethostname()
        .to_string_lossy()
        .trim()
        .to_owned();
    request
        .ensure_targets_local_host(&local_host)
        .map_err(|_| {
            BootstrapError::direct_host_inbox("source request targets a different host")
        })?;
    // Reuse existing local pending-attention meaning, with a deterministic
    // original ingress timestamp. No C1 record or router attachment is written.
    let local = request
        .materialize_as_local_obligation(request.ingress_received_at)
        .map_err(|_| {
            BootstrapError::direct_host_inbox("pending request local semantics are invalid")
        })?;
    Ok(Some(DirectHostPendingRequestV1 {
        schema_version: 1,
        authority_store_id: genesis.birth.authority_store_id.clone(),
        orchestration_session_id: request.orchestration_session_id.clone(),
        genesis_commitment: genesis_commitment.clone(),
        original_receipt: reference.clone(),
        obligation_id: local.obligation_id,
        obligation_revision: 1,
        kind: local.kind,
        severity: local.severity,
        summary: local.summary,
        attention_required: true,
        state: "pending".into(),
        unverified_request: serde_json::to_value(&request).map_err(|_| {
            BootstrapError::direct_host_inbox("encode original unverified local request")
        })?,
    }))
}

fn read_direct_host_source_at(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    commitment: &AuthorityObjectCommitmentV1,
    high_water: u64,
) -> Result<DirectHostInboxSourceHeadV1, BootstrapError> {
    let head: DirectHostInboxSourceHeadV1 = read_inbox_immutable(
        tx,
        "source-heads",
        "substrate.direct-host-inbox.source-head.v1",
        commitment,
    )?;
    let (store, home) = inbox_identity(tx)?;
    if head.schema_version != 1
        || head.authority_store_id != store
        || head.bootstrap_home != home
        || head.receipts.len() > MAX_DIRECT_HOST_RECEIPTS
        || head.receipts.len() as u64 != high_water
    {
        return Err(BootstrapError::direct_host_inbox(
            "canonical ledger source prefix context changed",
        ));
    }
    let mut ids=std::collections::BTreeSet::new();
    if head.receipts.iter().enumerate().any(|(index,r)| r.ordinal!=index as u64+1 || !ids.insert(r.receipt_id.clone())) {
        return Err(BootstrapError::direct_host_inbox("canonical source prefix is not uniquely contiguous"));
    }
    verify_direct_host_inbox_source_history(tx, &head)?;
    Ok(head)
}

fn direct_host_record_read(
    record: &DirectHostPendingRequestV1,
    commitment: AuthorityObjectCommitmentV1,
    publication: AuthorityObjectCommitmentV1,
) -> DirectHostCanonicalRecordReadV1 {
    DirectHostCanonicalRecordReadV1 {
        obligation_id: record.obligation_id.clone(),
        obligation_revision: record.obligation_revision,
        canonical_record_commitment: commitment,
        original_receipt_commitment: record.original_receipt.commitment.clone(),
        original_publication_commitment: publication,
        unverified_request: record.unverified_request.clone(),
    }
}

/// Validate every head, publication and immutable record in this session's
/// carried chain. No live historical helper guard or erased input is needed.
fn read_direct_host_enrolled_ledger_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    session_id: &str,
) -> Result<DirectHostEnrolledLedgerReadV1, BootstrapError> {
    let key = direct_host_session_key(session_id)?;
    let genesis_commitment = read_inbox_pointer(tx, &key, "genesis.json")?.ok_or(
        BootstrapError::direct_host_inbox("direct-host session has no authenticated genesis"),
    )?;
    let genesis: DirectHostInboxGenesisV1 = read_inbox_immutable(
        tx,
        "genesis",
        "substrate.direct-host-inbox.genesis.v1",
        &genesis_commitment,
    )?;
    verify_direct_host_genesis_inventory(tx, session_id, Some(&genesis_commitment))?;
    let head_commitment = read_inbox_pointer(tx, &key, "ledger-head.json")?.ok_or(
        BootstrapError::direct_host_inbox("direct-host session ledger head is absent"),
    )?;
    let (store, home) = inbox_identity(tx)?;
    if genesis.schema_version != 1
        || genesis.source_protocol != DIRECT_HOST_SOURCE_PROTOCOL
        || genesis.birth.authority_store_id != store
        || genesis.birth.bootstrap_home != home
        || genesis.birth.birth_authority.orchestration_session_id != session_id
    {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host enrolled genesis context changed",
        ));
    }
    verify_direct_host_genesis_source(tx, &genesis)?;
    crate::execution::agent_runtime::host_session_authority::direct_host_resume::verify_direct_host_birth_binding_locked(tx,&genesis.birth)?;
    if tx.direct_host_prior_canonical_source_present(session_id)? {
        return Err(BootstrapError::direct_host_inbox(
            "unexplained canonical source prevents whole-session interpretation",
        ));
    }
    let mut chain = Vec::new();
    let mut next = Some(head_commitment.clone());
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..=MAX_DIRECT_HOST_RECEIPTS {
        let Some(commitment) = next.take() else { break };
        if !seen.insert(direct_host_inbox_commitment_file(&commitment)?) {
            return Err(BootstrapError::direct_host_inbox(
                "cyclic direct-host canonical ledger history",
            ));
        }
        let head: DirectHostInboxLedgerHeadV1 = read_inbox_immutable(
            tx,
            "ledger-heads",
            "substrate.direct-host-inbox.ledger-head.v1",
            &commitment,
        )?;
        if head.schema_version != 1
            || head.authority_store_id != store
            || head.bootstrap_home != home
            || head.orchestration_session_id != session_id
            || head.genesis_commitment != genesis_commitment
            || head.records.len() > MAX_DIRECT_HOST_RECEIPTS
            || head.receipt_high_water > MAX_DIRECT_HOST_RECEIPTS as u64
        {
            return Err(BootstrapError::direct_host_inbox(
                "direct-host canonical ledger identity changed",
            ));
        }
        next = head.previous_head.clone();
        chain.push((commitment, head));
    }
    if next.is_some() || chain.is_empty() {
        return Err(BootstrapError::direct_host_inbox(
            "incomplete direct-host canonical ledger history",
        ));
    }
    chain.reverse();
    let (_, zero) = &chain[0];
    if zero.ledger_revision != 0
        || zero.previous_head.is_some()
        || zero.publication_commitment.is_some()
        || !zero.records.is_empty()
        || zero.receipt_high_water != 0
        || zero.source_head_commitment != genesis.initial_source_head
    {
        return Err(BootstrapError::direct_host_inbox(
            "direct-host initial ledger/genesis context changed",
        ));
    }
    let mut record_names = std::collections::BTreeSet::new();
    let mut publication_names = std::collections::BTreeSet::new();
    let mut obligation_ids = std::collections::BTreeSet::new();
    for pair in chain.windows(2) {
        let (previous_commitment, previous) = &pair[0];
        let (commitment, head) = &pair[1];
        if head.ledger_revision
            != previous
                .ledger_revision
                .checked_add(1)
                .ok_or(BootstrapError::direct_host_inbox(
                    "canonical ledger revision overflow",
                ))?
            || head.previous_head.as_ref() != Some(previous_commitment)
            || head.receipt_high_water <= previous.receipt_high_water
            || !head.records.starts_with(&previous.records)
        {
            return Err(BootstrapError::direct_host_inbox(
                "canonical ledger does not preserve original records and revisions",
            ));
        }
        let publication_commitment =
            head.publication_commitment
                .as_ref()
                .ok_or(BootstrapError::direct_host_inbox(
                    "canonical ledger publication is absent",
                ))?;
        publication_names.insert(direct_host_inbox_commitment_file(publication_commitment)?);
        let publication: DirectHostInboxPublicationV1 = read_inbox_immutable(
            tx,
            "publications",
            "substrate.direct-host-inbox.publication.v1",
            publication_commitment,
        )?;
        if publication.schema_version != 1
            || publication.genesis_commitment != genesis_commitment
            || publication.binding.authority_store_id != store
            || publication.binding.bootstrap_home != home
            || publication.binding.orchestration_session_id != session_id
            || publication.previous_ledger_head != *previous_commitment
            || publication.ledger_revision != head.ledger_revision
            || publication.source_head_commitment != head.source_head_commitment
            || publication.receipt_high_water != head.receipt_high_water
        {
            return Err(BootstrapError::direct_host_inbox(
                "canonical publication exact source/actor context changed",
            ));
        }
        let historical_cut=crate::execution::agent_runtime::host_session_authority::direct_host_resume::verify_direct_host_resume_publication_binding_locked(tx,&publication.binding)?;
        if let Some(cut) = historical_cut {
            let anchor = chain
                .iter()
                .find(|(c, _)| c == &cut.ledger_head_commitment)
                .map(|(_, h)| h)
                .ok_or(BootstrapError::direct_host_inbox(
                    "canonical publication was not anchored by its committed HSA cut",
                ))?;
            if cut.genesis_commitment != genesis_commitment
                || cut.authority_store_id != store
                || cut.orchestration_session_id != session_id
                || cut.source_protocol != DIRECT_HOST_SOURCE_PROTOCOL
                || cut.ledger_revision != anchor.ledger_revision
                || cut.ledger_revision < head.ledger_revision
                || cut.source_head_commitment != anchor.source_head_commitment
                || cut.receipt_high_water != anchor.receipt_high_water
                || cut.unresolved_attention != direct_host_attention_entries(&anchor.records)
            {
                return Err(BootstrapError::direct_host_inbox(
                    "historical canonical cut does not anchor exact carried records",
                ));
            }
        }
        let prefix =
            read_direct_host_source_at(tx, &head.source_head_commitment, head.receipt_high_water)?;
        if previous.ledger_revision > 0 {
            let old = read_direct_host_source_at(
                tx,
                &previous.source_head_commitment,
                previous.receipt_high_water,
            )?;
            if !prefix.receipts.starts_with(&old.receipts) {
                return Err(BootstrapError::direct_host_inbox(
                    "canonical ledger source prefix was replaced",
                ));
            }
        } else if !prefix.receipts.starts_with(
            &read_direct_host_source_at(
                tx,
                &genesis.initial_source_head,
                genesis.initial_receipt_high_water,
            )?
            .receipts,
        ) {
            return Err(BootstrapError::direct_host_inbox(
                "canonical ledger lost enrolled source ancestry",
            ));
        }
        let mut new_reads = Vec::new();
        let mut new_commitments = Vec::new();
        for reference in prefix
            .receipts
            .iter()
            .skip(previous.receipt_high_water as usize)
        {
            let receipt: DirectHostInboxReceiptV1 = read_inbox_immutable(
                tx,
                "receipts",
                "substrate.direct-host-inbox.receipt.v1",
                &reference.commitment,
            )?;
            if receipt.schema_version != 1
                || receipt.authority_store_id != store
                || receipt.bootstrap_home != home
                || receipt.ordinal != reference.ordinal
                || receipt.receipt_id != reference.receipt_id
            {
                return Err(BootstrapError::direct_host_inbox(
                    "canonical record original receipt context changed",
                ));
            }
            if let Some(expected) =
                direct_host_pending_request(&genesis, &genesis_commitment, reference, &receipt)?
            {
                if !obligation_ids.insert(expected.obligation_id.clone()) {
                    return Err(BootstrapError::direct_host_inbox(
                        "canonical request obligation identity repeated",
                    ));
                }
                let record_commitment = direct_host_inbox_commitment(
                    "substrate.direct-host-inbox.pending-request.v1",
                    &expected,
                )?;
                let record: DirectHostPendingRequestV1 = read_inbox_immutable(
                    tx,
                    "records",
                    "substrate.direct-host-inbox.pending-request.v1",
                    &record_commitment,
                )?;
                if record != expected {
                    return Err(BootstrapError::direct_host_inbox(
                        "canonical record no longer preserves original request semantics",
                    ));
                }
                record_names.insert(direct_host_inbox_commitment_file(&record_commitment)?);
                new_commitments.push(record_commitment.clone());
                new_reads.push(direct_host_record_read(
                    &record,
                    record_commitment,
                    publication_commitment.clone(),
                ));
            }
        }
        if publication.new_records != new_commitments
            || head.records[previous.records.len()..] != new_reads
        {
            return Err(BootstrapError::direct_host_inbox(
                "canonical publication does not exhaust its whole-session source delta",
            ));
        }
        let _ = commitment;
    }
    // Global object directories are shared by enrolled sessions. Inspect every
    // object, but attribute orphan refusal only to this exact session/genesis.
    for (directory, expected) in [
        ("records", record_names),
        ("publications", publication_names),
        ("ledger-heads", seen),
    ] {
        let mut actual = std::collections::BTreeSet::new();
        for (name, bytes) in tx.list_files(Area::Source, &[directory])? {
            let belongs = match directory {
                "records" => {
                    let r: DirectHostPendingRequestV1 = inbox_decode(&bytes)?;
                    r.orchestration_session_id == session_id
                }
                "publications" => {
                    let p: DirectHostInboxPublicationV1 = inbox_decode(&bytes)?;
                    p.binding.orchestration_session_id == session_id
                }
                _ => {
                    let h: DirectHostInboxLedgerHeadV1 = inbox_decode(&bytes)?;
                    h.orchestration_session_id == session_id
                }
            };
            if belongs {
                actual.insert(name);
            }
        }
        if actual != expected {
            return Err(BootstrapError::direct_host_inbox(
                "orphan canonical object prevents complete session history",
            ));
        }
    }
    tx.verify()?;
    let (head_commitment, head) = chain.pop().ok_or(BootstrapError::direct_host_inbox(
        "canonical head disappeared",
    ))?;
    Ok(DirectHostEnrolledLedgerReadV1 {
        genesis_commitment,
        genesis,
        head_commitment,
        head,
    })
}

fn direct_host_attention_entries(
    records: &[DirectHostCanonicalRecordReadV1],
) -> Vec<crate::execution::agent_runtime::host_session_authority::schema::DirectHostCanonicalAttentionEntryV1>{
    let mut entries=records.iter().map(|r| crate::execution::agent_runtime::host_session_authority::schema::DirectHostCanonicalAttentionEntryV1 {
        obligation_id:r.obligation_id.clone(),obligation_revision:r.obligation_revision,
        canonical_record_commitment:r.canonical_record_commitment.clone(),original_receipt_commitment:r.original_receipt_commitment.clone(),
        original_publication_commitment:r.original_publication_commitment.clone(),
    }).collect::<Vec<_>>();
    entries.sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));
    entries
}

/// The only canonical producer. Its opaque capability was authenticated from
/// the actual applied Accepted Resume and retains the inherited helper guard.
/// The caller must additionally observe a real active native turn before use.
pub(crate) fn materialize_direct_host_inbox_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    producer: &crate::execution::agent_runtime::host_session_authority::direct_host_resume::AuthenticatedDirectHostResumeV1,
) -> Result<crate::execution::agent_runtime::host_session_authority::schema::DirectHostCanonicalAttentionCutV1,BootstrapError>{
    producer.verify_active(tx)?;
    let binding = producer.binding();
    let key = direct_host_session_key(&binding.orchestration_session_id)?;
    let enrolled = read_direct_host_enrolled_ledger_locked(tx, &binding.orchestration_session_id)?;
    if producer.birth_authority_commitment()
        != &enrolled.genesis.birth.birth_authority_record_commitment
        || !enrolled.genesis.initial_inventory_complete
    {
        return Err(BootstrapError::direct_host_inbox(
            "canonical materializer has unknown enrollment/source applicability",
        ));
    }
    let source = read_direct_host_inbox_source(tx)?;
    let raw = tx.list_files(Area::RawInbox, &[])?;
    direct_host_source_inventory_complete(tx, &source, &raw)?;
    let source_commitment =
        source
            .head_commitment
            .as_ref()
            .ok_or(BootstrapError::direct_host_inbox(
                "canonical materializer has no committed enrolled source",
            ))?;
    let old_prefix = read_direct_host_source_at(
        tx,
        &enrolled.head.source_head_commitment,
        if enrolled.head.ledger_revision == 0 {
            enrolled.genesis.initial_receipt_high_water
        } else {
            enrolled.head.receipt_high_water
        },
    )?;
    if !source
        .receipts
        .iter()
        .map(|(r, _)| r)
        .cloned()
        .collect::<Vec<_>>()
        .starts_with(&old_prefix.receipts)
    {
        return Err(BootstrapError::direct_host_inbox(
            "canonical materializer source is not the enrolled carried prefix",
        ));
    }
    if source.receipts.len() as u64 > enrolled.head.receipt_high_water {
        let revision = enrolled.head.ledger_revision.checked_add(1).ok_or(
            BootstrapError::direct_host_inbox("canonical ledger revision overflow"),
        )?;
        let mut records = enrolled.head.records.clone();
        let mut new_records = Vec::new();
        let mut prepared = Vec::new();
        for (reference, receipt) in source
            .receipts
            .iter()
            .skip(enrolled.head.receipt_high_water as usize)
        {
            if let Some(record) = direct_host_pending_request(
                &enrolled.genesis,
                &enrolled.genesis_commitment,
                reference,
                receipt,
            )? {
                if records
                    .iter()
                    .any(|r| r.obligation_id == record.obligation_id)
                {
                    return Err(BootstrapError::direct_host_inbox(
                        "materializer cannot revise an existing pending canonical record",
                    ));
                }
                let commitment = direct_host_inbox_commitment(
                    "substrate.direct-host-inbox.pending-request.v1",
                    &record,
                )?;
                new_records.push(commitment.clone());
                prepared.push((record, commitment));
            }
        }
        let publication = DirectHostInboxPublicationV1 {
            schema_version: 1,
            genesis_commitment: enrolled.genesis_commitment.clone(),
            binding: binding.clone(),
            previous_ledger_head: enrolled.head_commitment.clone(),
            ledger_revision: revision,
            source_head_commitment: source_commitment.clone(),
            receipt_high_water: source.receipts.len() as u64,
            new_records,
        };
        let publication_commitment = direct_host_inbox_commitment(
            "substrate.direct-host-inbox.publication.v1",
            &publication,
        )?;
        producer.verify_active(tx)?;
        for (record, commitment) in prepared {
            let actual = publish_inbox_immutable(
                tx,
                "records",
                "substrate.direct-host-inbox.pending-request.v1",
                &record,
            )?;
            if actual != commitment {
                return Err(BootstrapError::direct_host_inbox(
                    "canonical record publication digest changed",
                ));
            }
            records.push(direct_host_record_read(
                &record,
                commitment,
                publication_commitment.clone(),
            ));
        }
        publish_inbox_immutable(
            tx,
            "publications",
            "substrate.direct-host-inbox.publication.v1",
            &publication,
        )?;
        let head = DirectHostInboxLedgerHeadV1 {
            schema_version: 1,
            authority_store_id: binding.authority_store_id.clone(),
            bootstrap_home: binding.bootstrap_home.clone(),
            orchestration_session_id: binding.orchestration_session_id.clone(),
            genesis_commitment: enrolled.genesis_commitment,
            previous_head: Some(enrolled.head_commitment.clone()),
            ledger_revision: revision,
            source_head_commitment: source_commitment.clone(),
            receipt_high_water: source.receipts.len() as u64,
            publication_commitment: Some(publication_commitment),
            records,
        };
        let next = publish_inbox_immutable(
            tx,
            "ledger-heads",
            "substrate.direct-host-inbox.ledger-head.v1",
            &head,
        )?;
        let expected = inbox_bytes(&DirectHostInboxSourcePointerV1 {
            schema_version: 1,
            head: enrolled.head_commitment,
        })?;
        producer.verify_active(tx)?;
        #[cfg(test)]
        direct_host_publication_boundary_for_test("materializer-before-head")?;
        tx.write_file(
            Area::Source,
            &["sessions", &key, "ledger-head.json"],
            Some(&expected),
            &inbox_bytes(&DirectHostInboxSourcePointerV1 {
                schema_version: 1,
                head: next,
            })?,
        )?;
        #[cfg(test)]
        direct_host_publication_boundary_for_test("materializer-after-head")?;
    } else if source_commitment != &enrolled.head.source_head_commitment {
        return Err(BootstrapError::direct_host_inbox(
            "zero-delta source identity changed",
        ));
    }
    read_direct_host_attention_cut_locked(tx, producer)
}

/// Recompute the entire carried cut while the same physical root remains held.
/// HSA must retain this transaction through the exact authority publication.
pub(crate) fn read_direct_host_attention_cut_locked(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    producer: &crate::execution::agent_runtime::host_session_authority::direct_host_resume::AuthenticatedDirectHostResumeV1,
) -> Result<crate::execution::agent_runtime::host_session_authority::schema::DirectHostCanonicalAttentionCutV1,BootstrapError>{
    producer.verify_active(tx)?;
    let b = producer.binding();
    let enrolled = read_direct_host_enrolled_ledger_locked(tx, &b.orchestration_session_id)?;
    if !enrolled.genesis.initial_inventory_complete
        || producer.birth_authority_commitment()
            != &enrolled.genesis.birth.birth_authority_record_commitment
    {
        return Err(BootstrapError::direct_host_inbox(
            "whole-session cut has unknown birth/source coverage",
        ));
    }
    let source = read_direct_host_inbox_source(tx)?;
    let raw = tx.list_files(Area::RawInbox, &[])?;
    direct_host_source_inventory_complete(tx, &source, &raw)?;
    if source.head_commitment.as_ref() != Some(&enrolled.head.source_head_commitment)
        || source.receipts.len() as u64 != enrolled.head.receipt_high_water
    {
        return Err(BootstrapError::direct_host_inbox(
            "whole-session cut is stale relative to committed ingress",
        ));
    }
    let cut=crate::execution::agent_runtime::host_session_authority::schema::DirectHostCanonicalAttentionCutV1 {
        schema_version:1,authority_store_id:b.authority_store_id.clone(),orchestration_session_id:b.orchestration_session_id.clone(),
        genesis_commitment:enrolled.genesis_commitment,source_head_commitment:enrolled.head.source_head_commitment,
        ledger_head_commitment:enrolled.head_commitment,receipt_high_water:enrolled.head.receipt_high_water,
        ledger_revision:enrolled.head.ledger_revision,source_protocol:DIRECT_HOST_SOURCE_PROTOCOL.into(),
        unresolved_attention:direct_host_attention_entries(&enrolled.head.records),
    };
    producer.verify_active(tx)?;
    tx.verify()?;
    Ok(cut)
}

/// A genesis object does not enroll a session until its pointer is committed.
/// An object-only crash leaves Unknown; no later prefix can mint around it.
/// The retained directory inventory also prevents ambiguity through HSA commit.
fn verify_direct_host_genesis_inventory(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    session_id: &str,
    committed: Option<&AuthorityObjectCommitmentV1>,
) -> Result<(), BootstrapError> {
    let mut observed=Vec::new();
    for (name,bytes) in tx.list_files(Area::Source,&["genesis"])? {
        let genesis:DirectHostInboxGenesisV1=inbox_decode(&bytes)?;
        let commitment=direct_host_inbox_commitment("substrate.direct-host-inbox.genesis.v1",&genesis)?;
        if name!=direct_host_inbox_commitment_file(&commitment)? {
            return Err(BootstrapError::direct_host_inbox("genesis inventory object name does not match its commitment"));
        }
        if genesis.birth.birth_authority.orchestration_session_id==session_id {
            observed.push(commitment);
        }
    }
    match committed {
        Some(expected) if observed.len()==1 && observed[0]==*expected=>{},
        None if observed.is_empty()=>{},
        _=>return Err(BootstrapError::direct_host_inbox("orphan or ambiguous same-session genesis prevents complete enrollment")),
    }
    tx.verify()
}

#[cfg(test)]
thread_local! {
    static DIRECT_HOST_PUBLICATION_FAILURE: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn direct_host_publication_boundary_for_test(stage: &'static str) -> Result<(), BootstrapError> {
    DIRECT_HOST_PUBLICATION_FAILURE.with(|slot| {
        if slot.get() == Some(stage) {
            slot.set(None);
            Err(BootstrapError::direct_host_inbox(stage))
        } else {
            Ok(())
        }
    })
}

/// Crash windows use real local receipts and the normal authenticated Resume
/// materializer. No caller-created canonical record is accepted as positive proof.
#[cfg(test)]
pub(crate) fn exercise_direct_host_publication_boundaries_for_test(
    tx: &mut DirectHostInboxTransactionV1<'_, '_>,
    producer: &crate::execution::agent_runtime::host_session_authority::direct_host_resume::AuthenticatedDirectHostResumeV1,
) -> Result<(), BootstrapError> {
    use crate::execution::agent_runtime::host_session_authority::direct_host_resume::direct_host_protocol_probe_mode_for_test;
    let Some(mode) = direct_host_protocol_probe_mode_for_test() else {
        return Ok(());
    };
    if !matches!(
        mode.as_str(),
        "receipt-retry" | "materializer-before-head" | "materializer-after-head" | "missing-record"
    ) {
        return Ok(());
    }
    producer.verify_active(tx)?;
    let root_before = tx.authority_root().cloned();
    let enrolled =
        read_direct_host_enrolled_ledger_locked(tx, &producer.binding().orchestration_session_id)?;
    assert!(
        !enrolled.head.records.is_empty(),
        "genuine during-Accepted publication required"
    );
    if mode == "missing-record" {
        let record = &enrolled.head.records[0];
        let name = direct_host_inbox_commitment_file(&record.canonical_record_commitment)?;
        let path = std::path::Path::new(&producer.binding().bootstrap_home.physical_path)
            .join("direct-host-inbox-ledger-v1/records")
            .join(name);
        let original = std::fs::read(&path).unwrap();
        // Retain the original bytes as fixture evidence before simulating loss.
        let saved = std::path::Path::new(&producer.binding().bootstrap_home.physical_path)
            .parent()
            .unwrap()
            .join(format!("missing-record-{}.json", producer.binding().run_id));
        std::fs::rename(&path, &saved).unwrap();
        assert_eq!(std::fs::read(saved).unwrap(), original);
        let missing_error = materialize_direct_host_inbox_locked(tx, producer).unwrap_err();
        eprintln!(
            "actual protocol missing-record refusal after genuine object loss: {missing_error}"
        );
        assert_eq!(tx.authority_root().cloned(), root_before);
        return Err(BootstrapError::direct_host_inbox(
            "test probe: missing genuine canonical record",
        ));
    }
    let source = read_direct_host_inbox_source(tx)?;
    let request = HostInboxRecord::new(
        "other-enrolled-source-target",
        format!("boundary-{}", producer.binding().run_id),
        crate::execution::agent_runtime::obligation_ledger::OrchestrationObligationKind::FollowUpRequired,
        "typed local source delta for crash boundary",
        gethostname::gethostname().to_string_lossy().trim().to_string(),
        "unverified-sender", "unverified-upstream",
    );
    if mode == "receipt-retry" {
        DIRECT_HOST_PUBLICATION_FAILURE.with(|slot| slot.set(Some("receipt-before-head")));
        assert_eq!(
            publish_direct_host_inbox_request(tx, &request)
                .unwrap_err()
                .to_string(),
            "receipt-before-head"
        );
        assert_eq!(
            read_direct_host_inbox_source(tx)?.head_commitment,
            source.head_commitment
        );
        // The same interrupted arrival is repairable by its own exact retry;
        // its raw/immutable bytes alone were never an authoritative arrival.
        let reference = publish_direct_host_inbox_request(tx, &request)?;
        let after = read_direct_host_inbox_source(tx)?;
        assert_eq!(after.receipts.len(), source.receipts.len() + 1);
        assert_eq!(publish_direct_host_inbox_request(tx, &request)?, reference);
        assert_eq!(
            read_direct_host_inbox_source(tx)?.head_commitment,
            after.head_commitment
        );
    } else {
        publish_direct_host_inbox_request(tx, &request)?;
        let stage = if mode == "materializer-before-head" {
            "materializer-before-head"
        } else {
            "materializer-after-head"
        };
        DIRECT_HOST_PUBLICATION_FAILURE.with(|slot| slot.set(Some(stage)));
        assert_eq!(
            materialize_direct_host_inbox_locked(tx, producer)
                .unwrap_err()
                .to_string(),
            stage
        );
        assert_eq!(tx.authority_root().cloned(), root_before);
        if mode == "materializer-before-head" {
            assert_eq!(
                read_inbox_pointer(
                    tx,
                    &direct_host_session_key(&producer.binding().orchestration_session_id)?,
                    "ledger-head.json"
                )?,
                Some(enrolled.head_commitment)
            );
            let error = materialize_direct_host_inbox_locked(tx, producer).unwrap_err();
            assert!(error.to_string().contains("orphan canonical object"));
            eprintln!("actual protocol materializer-before-head orphan refusal: {error}");
            return Err(error);
        }
    }
    let cut = materialize_direct_host_inbox_locked(tx, producer)?;
    let after =
        read_direct_host_enrolled_ledger_locked(tx, &producer.binding().orchestration_session_id)?;
    assert_eq!(
        after.head.records, enrolled.head.records,
        "exact original provenance remains unchanged"
    );
    assert_eq!(
        after.head.ledger_revision,
        enrolled.head.ledger_revision + 1
    );
    assert_eq!(materialize_direct_host_inbox_locked(tx, producer)?, cut);
    assert_eq!(tx.authority_root().cloned(), root_before);
    eprintln!("actual authenticated publication boundary {mode}: original canonical record unchanged; exact retry semantics checked");
    Ok(())
}
