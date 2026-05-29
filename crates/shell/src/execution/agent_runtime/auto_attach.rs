use anyhow::Result;

use super::obligation_ledger::{OrchestrationObligationKind, OrchestrationObligationRecord};

pub(crate) const MANUAL_REATTACH_ATTACH_RESTORED_REASON: &str =
    "session_attach_restored_by_manual_reattach";

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SessionAutoAttachClaim {
    NoCandidate {
        reason: &'static str,
    },
    AlreadyClaimed {
        obligation_id: String,
    },
    Claimed {
        obligation_id: String,
        attach_claim_owner: String,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SessionAutoAttachSettleResult {
    pub satisfied_obligation_ids: Vec<String>,
    pub superseded_obligation_ids: Vec<String>,
}

#[allow(dead_code)]
pub(crate) fn claimed_obligation_id(
    obligations: &[OrchestrationObligationRecord],
) -> Result<Option<&str>> {
    let mut claimed = obligations
        .iter()
        .filter(|obligation| obligation.is_auto_attach_claimed());
    let Some(first) = claimed.next() else {
        return Ok(None);
    };
    if let Some(second) = claimed.next() {
        anyhow::bail!(
            "session {} has multiple claimed auto-attach obligations ({} and {})",
            first.orchestration_session_id,
            first.obligation_id,
            second.obligation_id
        );
    }

    Ok(Some(first.obligation_id.as_str()))
}

#[allow(dead_code)]
pub(crate) fn select_attach_candidate(
    obligations: &[OrchestrationObligationRecord],
) -> Option<&OrchestrationObligationRecord> {
    let mut candidates = obligations
        .iter()
        .filter(|obligation| obligation.is_auto_attach_eligible())
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        candidate_priority(left.kind)
            .cmp(&candidate_priority(right.kind))
            .then(left.created_at.cmp(&right.created_at))
            .then(left.obligation_id.cmp(&right.obligation_id))
    });
    candidates.into_iter().next()
}

fn candidate_priority(kind: OrchestrationObligationKind) -> u8 {
    match kind {
        OrchestrationObligationKind::ApprovalRequired => 0,
        OrchestrationObligationKind::Blocked => 1,
        OrchestrationObligationKind::ForkRequest => 2,
        OrchestrationObligationKind::FollowUpRequired => 3,
        _ => u8::MAX,
    }
}
