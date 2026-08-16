use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::Write;
use substrate_r3_macos_signer_acl::experiment::{
    absent_retry_states, fresh_create_states, fresh_delete_states, query_states,
    verify_closed_process_surface, CreatorRollbackMarkerV2, CreatorRollbackPhaseV2,
    CreatorRollbackReceiptV2, MarkerRoot, MarkerState,
};
use substrate_r3_macos_signer_acl::{
    compiled_creator_route_config, ExactDeleteClassification, ExactDeleteReceipt,
    FixedRepetitionV2, NonInteractiveSecurity, ProductEquivalentCreationReceipt,
    QueryUiFailSecurity, CREATOR_EXECUTABLE_PATH, MARKER_PATH,
};

const SCHEMA: &str = "substrate.r3-macos-signer-acl.creator-route-receipt.v2";

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum CreatorRoutePhase {
    QueryUiFailCreateThenDelete,
    FreshProcessCreateThenExit,
    FreshProcessCreateRecovered,
    FreshProcessFirstCallDisableThenDelete,
    FreshDeleteRecoveredAbsent,
    AlreadyAbsentRetry,
}

#[derive(Debug, Serialize)]
struct CreatorRouteReceipt {
    schema: &'static str,
    repetition: FixedRepetitionV2,
    creator_scope_id: &'static str,
    creator_executable_path: &'static str,
    marker_path: &'static str,
    phase: CreatorRoutePhase,
    state_before: &'static str,
    state_after: &'static str,
    process_interaction_disable_raw_os_status: Option<i32>,
    first_creation: Option<ProductEquivalentCreationReceipt>,
    exact_delete: Option<ExactDeleteReceipt>,
    final_present: bool,
}

fn main() -> Result<()> {
    verify_closed_process_surface(CREATOR_EXECUTABLE_PATH)?;
    // The sealed root runner measures this exact post-exec process while it is stopped.  No
    // Security.framework call or marker mutation may precede the rendezvous.
    if unsafe { libc::raise(libc::SIGSTOP) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("enter creator attestation rendezvous");
    }
    let marker = MarkerRoot::open()?;
    if let Some(rollback) = marker.read_rollback()? {
        return run_emergency_rollback(&marker, rollback);
    }
    let state = marker.read_state()?;
    let (repetition, phase) = classify_state(state)?;
    let config = compiled_creator_route_config(repetition)?;

    let receipt = match phase {
        Route::QueryPrepared => {
            let (prepared, invoked, after) = query_states(repetition);
            marker.transition(prepared, invoked)?;
            let mut security = QueryUiFailSecurity::claim_process()?;
            if security.exact_identity_present(&config)? {
                bail!("query UI-fail repetition found an unexpected exact signer")
            }
            let creation = security.create_product_equivalent_signer_without_access(&config)?;
            let exact_delete = security.exact_delete_receipt(&config)?;
            require_deleted(&exact_delete, "query UI-fail deletion")?;
            marker.transition(invoked, after)?;
            receipt(
                repetition,
                CreatorRoutePhase::QueryUiFailCreateThenDelete,
                invoked,
                after,
                None,
                Some(creation),
                Some(exact_delete),
                false,
            )
        }
        Route::QueryInvoked => {
            bail!("query UI-fail invocation outcome is ambiguous; it must never be reinvoked")
        }
        Route::FreshCreatePrepared => {
            let (prepared, invoked, after) = fresh_create_states(repetition);
            marker.transition(prepared, invoked)?;
            let mut security = QueryUiFailSecurity::claim_process()?;
            if security.exact_identity_present(&config)? {
                bail!("fresh-create repetition found an unexpected exact signer")
            }
            let creation = security.create_product_equivalent_signer_without_access(&config)?;
            if !security.exact_identity_present(&config)? {
                bail!("fresh-created exact signer is absent before durable transition")
            }
            marker.transition(invoked, after)?;
            receipt(
                repetition,
                CreatorRoutePhase::FreshProcessCreateThenExit,
                invoked,
                after,
                None,
                Some(creation),
                None,
                true,
            )
        }
        Route::FreshCreateInvoked => {
            let (_, invoked, after) = fresh_create_states(repetition);
            let security = QueryUiFailSecurity::claim_process()?;
            if !security.exact_identity_present(&config)? {
                bail!("fresh-create Invoked cursor is ambiguous while the key is absent")
            }
            marker.transition(invoked, after)?;
            receipt(
                repetition,
                CreatorRoutePhase::FreshProcessCreateRecovered,
                invoked,
                after,
                None,
                None,
                None,
                true,
            )
        }
        Route::FreshDeletePrepared => {
            let (prepared, invoked, after) = fresh_delete_states(repetition);
            marker.transition(prepared, invoked)?;
            // This must be the first Security call in this fresh process.
            let mut security = NonInteractiveSecurity::establish_first()?;
            let exact_delete = security.exact_delete_receipt(&config)?;
            require_deleted(&exact_delete, "fresh-process interaction-disabled deletion")?;
            marker.transition(invoked, after)?;
            receipt(
                repetition,
                CreatorRoutePhase::FreshProcessFirstCallDisableThenDelete,
                invoked,
                after,
                Some(0),
                None,
                Some(exact_delete),
                false,
            )
        }
        Route::FreshDeleteInvoked => {
            let (_, invoked, after) = fresh_delete_states(repetition);
            let security = NonInteractiveSecurity::establish_first()?;
            if security.exact_identity_present(&config)? {
                bail!("fresh-delete Invoked cursor remains present and must never be reinvoked")
            }
            marker.transition(invoked, after)?;
            receipt(
                repetition,
                CreatorRoutePhase::FreshDeleteRecoveredAbsent,
                invoked,
                after,
                Some(0),
                None,
                None,
                false,
            )
        }
        Route::AbsentRetryPrepared => {
            let (prepared, invoked, after) = absent_retry_states(repetition);
            marker.transition(prepared, invoked)?;
            let mut security = NonInteractiveSecurity::establish_first()?;
            let exact_delete = security.exact_delete_receipt(&config)?;
            require_already_absent(&exact_delete)?;
            marker.transition(invoked, after)?;
            receipt(
                repetition,
                CreatorRoutePhase::AlreadyAbsentRetry,
                invoked,
                after,
                Some(0),
                None,
                Some(exact_delete),
                false,
            )
        }
        Route::AbsentRetryInvoked => {
            bail!("already-absent retry outcome is ambiguous and must never be reinvoked")
        }
        Route::WrongPrepared | Route::WrongInvoked => {
            bail!("creator executable cannot perform the separately signed wrong-identity arm")
        }
        Route::Complete => bail!("both closed creator repetitions are complete"),
    };

    std::io::stdout()
        .lock()
        .write_all(&canonical_receipt_line(&receipt)?)
        .context("write canonical creator-route receipt")
}

fn canonical_receipt_line<T: Serialize>(receipt: &T) -> Result<Vec<u8>> {
    let mut bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(receipt)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn run_emergency_rollback(marker_root: &MarkerRoot, marker: CreatorRollbackMarkerV2) -> Result<()> {
    marker.validate()?;
    if marker.phase == CreatorRollbackPhaseV2::Observed {
        bail!("creator rollback is already observed; recover its durable runner receipt")
    }
    let invoked = if marker.phase == CreatorRollbackPhaseV2::Prepared {
        let invoked = CreatorRollbackMarkerV2 {
            phase: CreatorRollbackPhaseV2::Invoked,
            receipt_sha256: None,
            ..marker.clone()
        };
        marker_root.replace_rollback(CreatorRollbackPhaseV2::Prepared, &invoked)?;
        invoked
    } else {
        marker
    };
    let config = compiled_creator_route_config(invoked.repetition)?;
    let mut security = QueryUiFailSecurity::claim_process()?;
    let exact_delete = if security.exact_identity_present(&config)? {
        let value = security.exact_delete_receipt(&config)?;
        require_deleted(&value, "creator emergency rollback")?;
        Some(value)
    } else {
        None
    };
    if security.exact_identity_present(&config)? {
        bail!("creator emergency rollback left its exact signer present")
    }
    let receipt = CreatorRollbackReceiptV2 {
        schema_owner: "substrate.r3-macos-signer-acl.creator-emergency-rollback-receipt".to_owned(),
        schema_version: 2,
        repetition: invoked.repetition,
        failure_observation_sha256: invoked.failure_observation_sha256.clone(),
        exact_delete,
        exact_identity_absent: true,
    };
    receipt.validate(&invoked)?;
    let bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(&receipt)?;
    let observed = CreatorRollbackMarkerV2 {
        phase: CreatorRollbackPhaseV2::Observed,
        receipt_sha256: Some(substrate_common::macos_retirement_v2::sha256_hex_v2(&bytes)),
        ..invoked
    };
    marker_root.replace_rollback(CreatorRollbackPhaseV2::Invoked, &observed)?;
    println!("{}", String::from_utf8(bytes)?);
    Ok(())
}

#[derive(Clone, Copy)]
enum Route {
    QueryPrepared,
    QueryInvoked,
    FreshCreatePrepared,
    FreshCreateInvoked,
    WrongPrepared,
    WrongInvoked,
    FreshDeletePrepared,
    FreshDeleteInvoked,
    AbsentRetryPrepared,
    AbsentRetryInvoked,
    Complete,
}

fn classify_state(state: MarkerState) -> Result<(FixedRepetitionV2, Route)> {
    use MarkerState as M;
    let value = match state {
        M::FirstQueryPrepared => (FixedRepetitionV2::First, Route::QueryPrepared),
        M::FirstQueryInvoked => (FixedRepetitionV2::First, Route::QueryInvoked),
        M::FirstFreshCreatePrepared => (FixedRepetitionV2::First, Route::FreshCreatePrepared),
        M::FirstFreshCreateInvoked => (FixedRepetitionV2::First, Route::FreshCreateInvoked),
        M::FirstWrongPrepared => (FixedRepetitionV2::First, Route::WrongPrepared),
        M::FirstWrongInvoked => (FixedRepetitionV2::First, Route::WrongInvoked),
        M::FirstFreshDeletePrepared => (FixedRepetitionV2::First, Route::FreshDeletePrepared),
        M::FirstFreshDeleteInvoked => (FixedRepetitionV2::First, Route::FreshDeleteInvoked),
        M::FirstAbsentRetryPrepared => (FixedRepetitionV2::First, Route::AbsentRetryPrepared),
        M::FirstAbsentRetryInvoked => (FixedRepetitionV2::First, Route::AbsentRetryInvoked),
        M::SecondQueryPrepared => (FixedRepetitionV2::Second, Route::QueryPrepared),
        M::SecondQueryInvoked => (FixedRepetitionV2::Second, Route::QueryInvoked),
        M::SecondFreshCreatePrepared => (FixedRepetitionV2::Second, Route::FreshCreatePrepared),
        M::SecondFreshCreateInvoked => (FixedRepetitionV2::Second, Route::FreshCreateInvoked),
        M::SecondWrongPrepared => (FixedRepetitionV2::Second, Route::WrongPrepared),
        M::SecondWrongInvoked => (FixedRepetitionV2::Second, Route::WrongInvoked),
        M::SecondFreshDeletePrepared => (FixedRepetitionV2::Second, Route::FreshDeletePrepared),
        M::SecondFreshDeleteInvoked => (FixedRepetitionV2::Second, Route::FreshDeleteInvoked),
        M::SecondAbsentRetryPrepared => (FixedRepetitionV2::Second, Route::AbsentRetryPrepared),
        M::SecondAbsentRetryInvoked => (FixedRepetitionV2::Second, Route::AbsentRetryInvoked),
        M::Complete => (FixedRepetitionV2::Second, Route::Complete),
    };
    Ok(value)
}

#[allow(clippy::too_many_arguments)]
fn receipt(
    repetition: FixedRepetitionV2,
    phase: CreatorRoutePhase,
    before: MarkerState,
    after: MarkerState,
    disable_status: Option<i32>,
    creation: Option<ProductEquivalentCreationReceipt>,
    exact_delete: Option<ExactDeleteReceipt>,
    final_present: bool,
) -> CreatorRouteReceipt {
    CreatorRouteReceipt {
        schema: SCHEMA,
        repetition,
        creator_scope_id: repetition.creator_scope(),
        creator_executable_path: CREATOR_EXECUTABLE_PATH,
        marker_path: MARKER_PATH,
        phase,
        state_before: state_name(before),
        state_after: state_name(after),
        process_interaction_disable_raw_os_status: disable_status,
        first_creation: creation,
        exact_delete,
        final_present,
    }
}

fn state_name(state: MarkerState) -> &'static str {
    std::str::from_utf8(state.marker())
        .expect("compiled marker is UTF-8")
        .trim_end()
}

fn require_already_absent(receipt: &ExactDeleteReceipt) -> Result<()> {
    if receipt.raw_os_status != -25_300
        || receipt.classification != ExactDeleteClassification::AlreadyAbsent
        || receipt.present_after
    {
        bail!("exact already-absent retry did not return frozen item-not-found")
    }
    Ok(())
}

fn require_deleted(receipt: &ExactDeleteReceipt, label: &str) -> Result<()> {
    if receipt.raw_os_status != 0
        || receipt.classification != ExactDeleteClassification::DeletedAndAbsent
        || receipt.present_after
    {
        bail!("{label} did not produce exact success-and-absence")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use substrate_common::macos_retirement_v2::{canonical_bytes_v2, parse_canonical_v2};

    #[test]
    fn creator_receipt_emits_one_canonical_newline_terminated_record() {
        let value = receipt(
            FixedRepetitionV2::First,
            CreatorRoutePhase::QueryUiFailCreateThenDelete,
            MarkerState::FirstQueryInvoked,
            MarkerState::FirstFreshCreatePrepared,
            None,
            None,
            None,
            false,
        );

        let emitted = canonical_receipt_line(&value).expect("encode creator receipt");
        assert_eq!(emitted.last(), Some(&b'\n'));
        assert!(!emitted[..emitted.len() - 1].contains(&b'\n'));
        let body = &emitted[..emitted.len() - 1];
        assert_eq!(body, canonical_bytes_v2(&value).unwrap());
        parse_canonical_v2::<Value>(body).expect("accept canonical creator receipt");

        let declaration_order = serde_json::to_vec(&value).unwrap();
        assert_ne!(declaration_order, body);
        assert!(parse_canonical_v2::<Value>(&declaration_order).is_err());
    }
}
