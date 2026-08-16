use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::io::Write;
use substrate_r3_macos_signer_acl::experiment::{
    verify_closed_process_surface, wrong_states, MarkerRoot, MarkerState,
};
use substrate_r3_macos_signer_acl::{
    compiled_creator_route_config, FixedRepetitionV2, NonInteractiveSecurity, MARKER_PATH,
    WRONG_IDENTITY_EXECUTABLE_PATH,
};

const SCHEMA: &str = "substrate.r3-macos-signer-acl.wrong-identity-receipt.v3";
const EXPECTED_LOOKUP_STATUS: i32 = -25_308;

#[derive(Debug, Serialize)]
struct WrongIdentityReceipt {
    schema: &'static str,
    repetition: FixedRepetitionV2,
    creator_scope_id: &'static str,
    executable_path: &'static str,
    marker_path: &'static str,
    process_interaction_disable_raw_os_status: i32,
    precommitted_expected_lookup_raw_os_status: i32,
    tag_scoped_private_key_lookup_raw_os_status: i32,
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
    // This must be the first Security.framework call in this fresh wrong-identity process.
    let security = NonInteractiveSecurity::establish_first()?;
    let tag_scoped_private_key_lookup_raw_os_status =
        security.tag_scoped_private_key_lookup_raw_os_status(&config)?;
    if tag_scoped_private_key_lookup_raw_os_status != EXPECTED_LOOKUP_STATUS {
        bail!(
            "wrong-identity tag-scoped private-key lookup returned OSStatus {tag_scoped_private_key_lookup_raw_os_status}, expected {EXPECTED_LOOKUP_STATUS}"
        )
    }
    marker.transition(invoked, after)?;
    let receipt = WrongIdentityReceipt {
        schema: SCHEMA,
        repetition,
        creator_scope_id: repetition.creator_scope(),
        executable_path: WRONG_IDENTITY_EXECUTABLE_PATH,
        marker_path: MARKER_PATH,
        process_interaction_disable_raw_os_status: 0,
        precommitted_expected_lookup_raw_os_status: EXPECTED_LOOKUP_STATUS,
        tag_scoped_private_key_lookup_raw_os_status,
    };
    std::io::stdout()
        .lock()
        .write_all(&canonical_receipt_line(&receipt)?)
        .context("write canonical wrong-identity receipt")
}

fn canonical_receipt_line<T: Serialize>(receipt: &T) -> Result<Vec<u8>> {
    let mut bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(receipt)?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use substrate_common::macos_retirement_v2::{canonical_bytes_v2, parse_canonical_v2};

    #[test]
    fn wrong_identity_receipt_emits_one_canonical_newline_terminated_record() {
        let value = WrongIdentityReceipt {
            schema: SCHEMA,
            repetition: FixedRepetitionV2::First,
            creator_scope_id: FixedRepetitionV2::First.creator_scope(),
            executable_path: WRONG_IDENTITY_EXECUTABLE_PATH,
            marker_path: MARKER_PATH,
            process_interaction_disable_raw_os_status: 0,
            precommitted_expected_lookup_raw_os_status: EXPECTED_LOOKUP_STATUS,
            tag_scoped_private_key_lookup_raw_os_status: EXPECTED_LOOKUP_STATUS,
        };

        let emitted = canonical_receipt_line(&value).expect("encode wrong-identity receipt");
        assert_eq!(emitted.last(), Some(&b'\n'));
        assert!(!emitted[..emitted.len() - 1].contains(&b'\n'));
        let body = &emitted[..emitted.len() - 1];
        assert_eq!(body, canonical_bytes_v2(&value).unwrap());
        let parsed =
            parse_canonical_v2::<Value>(body).expect("accept canonical wrong-identity receipt");
        assert_eq!(
            parsed.get("schema").and_then(Value::as_str),
            Some("substrate.r3-macos-signer-acl.wrong-identity-receipt.v3")
        );
        assert!(parsed.get("exact_delete").is_none());
        assert!(parsed.get("exact_identity_preserved_after").is_none());

        let declaration_order = serde_json::to_vec(&value).unwrap();
        assert_ne!(declaration_order, body);
        assert!(parse_canonical_v2::<Value>(&declaration_order).is_err());
    }
}
