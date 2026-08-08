# R5 lifecycle and convergence review — supplemental-causal-2 final stop

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R5`  
Subject: `sha256:d1f4e28e5bd3ca3ccb66e981397698e527403a647ce2103021debdae5a67007d`

## Retained direct-bootstrap convergence

The direct IH-only request still accepts no scope, attempt, locator, authorization, manifest,
time, path, or retry field. The final reviewer found no new P1/P2 in the locator path: it records a
fixed-key allocation before authorization/key/capsule effects; exact incomplete/completed requests
reopen retained facts; completed replay compares the retained protected state and Stage-1 capsule
before returning its canonical bounded response; corrupt, mismatched, expired-incomplete, or
ambiguous state is preserving-first.

## Ordinary post-PM convergence is not complete

`P1-R5-POST-PM-INSTANCE-RECEIPT-PLAN-GAP` blocks normal typed stop/start/remove/restore. The
manifest planning code reserves the Stage-1 `mac.lima.instance/Create` receipt only, while the
ordinary table and `scripts/mac/lima-stop.sh` submit other instance actions. The executor’s planned
receipt lookup therefore fails before effect for every such admitted request.

`P1-R5-POST-PM-PREPARED-CAS-UNRECOVERABLE` blocks exact crash recovery. A normal request’s current
anchor counter must equal the current protected-state counter. The prepared allocation increments
that counter before the effect-start marker. If execution stops in that interval, replay of the
same request is rejected by the normal counter gate and never reaches the later exact prepared
record join; this is a durable no-effect state that cannot converge.

`P1-R5-POST-PM-GUEST-EXECUTOR-CIRCULAR-BOOTSTRAP` blocks the remaining guest rows. The privileged
executor sends a fixed `r5-post-pm` command to the guest lifecycle binary, but no R5 path installs
and attests that binary after the marker-only Stage-1 profile. The row nominally responsible for
that guest binary invokes the same missing path. This is not a valid fixed installed-artifact
primitive.

## Test disposition

The current shell integration test exhaustively validates decoder acceptance/rejection, not the
manifest receipt plan or executor transaction. The MAC fixture is static/lexical and cannot prove
guest binary installation, identity, or the prepared-CAS retry boundary. This is recorded as
`P2-R5-POST-PM-EXECUTOR-TRANSACTION-PROOF-GAP` and is not the reason publication stops; the three
P1s independently stop it.

## Verdict

**NOT CLEAN — budget exhausted.** There is no authorized fifth remediation cycle. Preserve state;
do not retry, adopt, delete, or publish any lifecycle result.
