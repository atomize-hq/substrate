use anyhow::{bail, Context, Result};
use serde::Serialize;
use substrate_r3_macos_signer_acl::experiment::{
    verify_closed_process_surface, wrong_states, MarkerRoot, MarkerState,
};
use substrate_r3_macos_signer_acl::{
    compiled_creator_route_config, ExactDeleteClassification, ExactDeleteReceipt,
    FixedRepetitionV2, QueryUiFailSecurity, MARKER_PATH, WRONG_IDENTITY_EXECUTABLE_PATH,
};

const SCHEMA: &str = "substrate.r3-macos-signer-acl.wrong-identity-receipt.v2";
const EXPECTED_DELETE_STATUS: i32 = -25_308;

#[derive(Debug, Serialize)]
struct WrongIdentityReceipt {
    schema: &'static str,
    repetition: FixedRepetitionV2,
    creator_scope_id: &'static str,
    executable_path: &'static str,
    marker_path: &'static str,
    precommitted_expected_raw_os_status: i32,
    exact_delete: ExactDeleteReceipt,
    exact_identity_preserved_after: bool,
}

fn main() -> Result<()> {
    verify_closed_process_surface(WRONG_IDENTITY_EXECUTABLE_PATH)?;
    // The sealed root runner measures this exact post-exec process while it is stopped.  No
    // Security.framework call or marker mutation may precede the rendezvous.
    if unsafe { libc::raise(libc::SIGSTOP) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("enter wrong-identity attestation rendezvous");
    }
    let marker = MarkerRoot::open()?;
    let state = marker.read_state()?;
    let repetition = match state {
        MarkerState::FirstWrongPrepared => FixedRepetitionV2::First,
        MarkerState::SecondWrongPrepared => FixedRepetitionV2::Second,
        MarkerState::FirstWrongInvoked | MarkerState::SecondWrongInvoked => {
            bail!("wrong-identity denial outcome is ambiguous and must never be reinvoked")
        }
        _ => bail!("wrong-identity attempt is outside its one closed prepared cursor"),
    };
    let (prepared, invoked, after) = wrong_states(repetition);
    marker.transition(prepared, invoked)?;
    let config = compiled_creator_route_config(repetition)?;
    let mut security = QueryUiFailSecurity::claim_process()?;
    if !security.exact_identity_present(&config)? {
        bail!("wrong-identity attempt found the exact signer absent")
    }
    let exact_delete = security.exact_delete_receipt(&config)?;
    let exact_identity_preserved_after = security.exact_identity_present(&config)?;
    if exact_delete.raw_os_status != EXPECTED_DELETE_STATUS
        || exact_delete.classification != ExactDeleteClassification::InteractionNotAllowed
        || !exact_delete.present_after
        || !exact_identity_preserved_after
    {
        bail!("wrong-identity deletion differed from frozen noninteractive preservation")
    }
    marker.transition(invoked, after)?;
    let receipt = WrongIdentityReceipt {
        schema: SCHEMA,
        repetition,
        creator_scope_id: repetition.creator_scope(),
        executable_path: WRONG_IDENTITY_EXECUTABLE_PATH,
        marker_path: MARKER_PATH,
        precommitted_expected_raw_os_status: EXPECTED_DELETE_STATUS,
        exact_delete,
        exact_identity_preserved_after,
    };
    println!(
        "{}",
        serde_json::to_string(&receipt).context("serialize wrong-identity receipt")?
    );
    Ok(())
}
