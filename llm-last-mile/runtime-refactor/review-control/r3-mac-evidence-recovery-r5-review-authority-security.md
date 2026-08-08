# R5 authority/security review — final supplemental causal stop

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R5`  
Base: `05d655fa1458a276f179d12057cbda772cc51eb6`  
Final reviewed implementation/test subject: `sha256:d1f4e28e5bd3ca3ccb66e981397698e527403a647ce2103021debdae5a67007d`

## Review sequence

- `discovery-1` found the seven original direct-bootstrap, FD3, admission, Stage-1, ordinary-shape,
  receipt, and frame-boundary defects.
- `closure-1` found `P1-R5-IH-ONLY-RANDOMIZED-DIRECT-BOOTSTRAP-RETRY` and the raw warm-session
  surface.
- `supplemental-causal-1` found that removing the raw normal warm continuation exposed the absent
  ordinary post-PM transaction.
- `supplemental-causal-2` independently re-reviewed the final 0031 remediation. It found the
  blocking post-PM defects recorded below. No further remediation cycle is authorized.

## Confirmed authority result

The final reviewer found no new P1/P2 in the retained direct IH-only bootstrap locator. It keeps the
request limited to the IH carrier; the fixed Keychain locator is selected from retained facts, is
allocated before bootstrap effects, reconstructs the original authorization in memory, and reopens
state/capsule joins before emitting a completed bounded response.

That result does not make the complete packet clean. The ordinary `post_pm_action` authority is still
not closed to a usable exhaustive authoritative transaction.

## Blocking findings

- `P1-R5-POST-PM-INSTANCE-RECEIPT-PLAN-GAP`: the executor admits the ordinary instance
  `Start`, `Stop`, `Remove`, and `Restore` rows, including the typed `lima-stop` caller, but the
  generation-two signed plan has an instance receipt only for the Stage-1 `Create` action. The
  executor rejects each admitted ordinary instance action before its effect because no matching
  planned receipt exists.
- `P1-R5-POST-PM-PREPARED-CAS-UNRECOVERABLE`: an exact request cannot resume an unambiguous
  state after the prepared-record CAS but before the effect-start marker. The current counter gate
  rejects it before the prepared-record join is reached.
- `P1-R5-POST-PM-GUEST-EXECUTOR-CIRCULAR-BOOTSTRAP`: fixed guest `r5-post-pm` invocations are
  neither installed nor attested after Stage-1. The publisher-executor row tries to invoke that
  same absent artifact, so the post-PM guest effect closure is circular.

No native action, Keychain mutation, XPC launch, Lima operation, socket/known-host mutation, or
publication was performed by this implementation/review task.

## Verdict

**NOT CLEAN — bounded stop / review budget exhausted.** The task must preserve this worktree. The
three P1s are R5 ordinary-post-PM defects, but the 0031 authority permits no remediation after this
final causal review.
