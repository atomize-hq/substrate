# Operator Prompt Library

**Status:** workflow aid; not semantic authority

Use these prompts to run the remaining R6 -> R7 -> R8 sequence with routine work delegated and the
operator involved only when repository truth cannot determine a safe choice or an external action is
required. The phase runner, phase map, active authority, source, and behavior-level proof remain the
governing contracts.

## Operating Model

- `PHASE_ID` is the canonical session boundary. `ACTIVE_PACKET` narrows work inside it.
- A phase prompt may complete every already-authorized task in that phase, including commit,
  review, fix, and closeout loops.
- A bounded-task prompt completes only the named task or packet and leaves phase status unchanged.
- Implementation/fix and review work use fresh built-in `default` subagents. The orchestration
  agent stays thin, checks gates and commits, and never self-approves.
- Routine failures are handled autonomously. A failed acceptance control is preserved and routed
  to the already-authorized conditional-gap process; it is not automatically a product decision.
- A phase transition is committed and review-clean before the next phase starts. The next phase
  starts in a fresh session.
- No prompt may upgrade proof, scorer disposition, or family status merely because work was
  attempted or a document was edited.

## When The Operator Is Involved

Do **not** ask for routine confirmation of plans, file choices, test commands, commit messages,
review fixes, or the next task already ordered by the active plan.

Escalate only when at least one of these is true:

1. canonical authorities conflict and live behavior cannot resolve intended product semantics;
2. two or more materially different behaviors satisfy the written contract and choosing between
   them changes product meaning;
3. GitNexus reports HIGH or CRITICAL upstream impact;
4. proceeding requires deleting, reverting, overwriting, or including unrelated work;
5. a public schema/version boundary, terminal disposition, merge/deprecation choice, or family
   scope must change without existing authority;
6. required rollout, fixture, permission, credential, service, or platform evidence is unavailable
   and cannot be replaced honestly; or
7. review findings prove the task cannot remain inside its authorized packet.

Use `DECISION REQUIRED` for a choice and `ACTION REQUIRED` for an external prerequisite. A blocker
that the agent can diagnose or fix inside the active scope is neither.

### Decision report shape

```text
DECISION REQUIRED
ID: <stable identifier>
PHASE/PACKET: <id>
QUESTION: <one precise question>
REPO EVIDENCE: <paths, symbols, tests, and commits>
WHY AUTHORITY CANNOT DECIDE: <specific contradiction or missing contract>
OPTIONS:
A. <option, effect, cost, risk>
B. <option, effect, cost, risk>
RECOMMENDATION: <A or B and why>
SAFE WORK ALREADY COMPLETED: <what is landed/proven>
BLOCKED SCOPE ONLY: <what will not proceed>
REPLY FORMAT: DECISION <ID>: <A|B|explicit alternative>
```

### External-action report shape

```text
ACTION REQUIRED
ID: <stable identifier>
PHASE/PACKET: <id>
NEEDED ACTION: <one concrete action>
WHY THE AGENT CANNOT DO IT: <permission, external system, unavailable artifact, etc.>
SAFE WORK ALREADY COMPLETED: <what is landed/proven>
RESUME CONDITION: <observable condition or command>
RESUME PROMPT: <exact prompt or Prompt 6 fields>
```

## Prompt Selection

| Interaction | Prompt |
|---|---|
| Start and autonomously finish one authorized phase | **Prompt 1** |
| Run one bounded task or packet without closing the phase | **Prompt 2** |
| Resume after interruption, compaction, or an uncertain handoff | **Prompt 3** |
| Finish an incomplete independent review/fix loop | **Prompt 4** |
| Audit closeout and activate the next phase without starting its work | **Prompt 5** |
| Apply a human decision or external unblock and continue | **Prompt 6** |

Most normal operation uses Prompt 1 at the start of a phase. Prompts 2-6 are recovery and control
surfaces, not required ceremony for every successful phase.

## Prompt 1 — Execute One Phase Autonomously

Change only `PHASE_ID` and `ACTIVE_PACKET`. Use `none` when the whole phase is active.

```text
/goal Execute the selected hybrid-drift phase through its review-clean exit gate, involving me only
for a structured DECISION REQUIRED or ACTION REQUIRED escalation.

REPO: /Users/spensermcconnell/.codex/worktrees/97a0/substrate
PHASE_ID: <exact phase id>
ACTIVE_PACKET: <none or exact packet id>
AUTONOMY_MODE: escalation-only
STOP_AT: current phase transition is committed and review-clean; do not start next-phase work

Read and follow:
- docs/specs/hybrid-drift-r6-r8-control-pack/04-reusable-phase-runner.md
- the selected phase manifest in
  docs/specs/hybrid-drift-r6-r8-control-pack/03-selective-context-manifests.md
- the escalation contract in
  docs/specs/hybrid-drift-r6-r8-control-pack/06-operator-prompt-library.md

You are the orchestration agent. Keep your own context thin. Use fresh built-in default subagents
for bounded implementation/fix work and fresh built-in default subagents for independent review.
Do not use shell-launched reviewer stand-ins.

Operate autonomously:
1. verify the phase entry gate and live repository state;
2. load only the selected manifest and active authority;
3. execute every already-authorized task inside this phase in plan order;
4. run GitNexus impact before indexed-symbol edits and detect changes before each commit;
5. run focused proof before the family wall;
6. commit each atomic task or fix batch;
7. dispatch a fresh reviewer after each review boundary;
8. fix actionable findings in a new commit and repeat fresh review until clean;
9. update canonical tasks, proof ledger, and status wording with actual results;
10. verify the phase exit gate; and
11. land and independently review the narrow phase-transition update.

Do not ask me to approve routine plans, tests, commits, reviewer fixes, or the next task already
authorized by the phase. If a control fails, preserve the witness and follow the phase map's bounded
gap rule. Escalate only under the library's operator-escalation contract.

Finish with:
- phase result and exit-gate evidence;
- commits and exact verification;
- review/fix-loop result;
- dispositions and ledger/status changes;
- unresolved blocker, if any; and
- a filled Prompt 1 invocation for the next eligible phase, without starting it.
```

## Prompt 2 — Execute One Bounded Task Or Packet

Use when a phase needs an intentionally separate packet, scorer, contract seam, or proof task.

```text
/goal Execute exactly one bounded hybrid-drift task/packet through commit and independent
review-clean status, involving me only for a structured decision or external action.

REPO: /Users/spensermcconnell/.codex/worktrees/97a0/substrate
PHASE_ID: <current phase id>
ACTIVE_PACKET: <exact packet id>
TASK_ID: <exact task or ledger id>
ACCEPTANCE: <one behavior-level or docs-consistency completion statement>
AUTONOMY_MODE: escalation-only
STOP_AT: named task/packet review-clean; leave phase status unchanged

Follow the reusable phase runner, matching selective-context manifest, and operator-escalation
contract in docs/specs/hybrid-drift-r6-r8-control-pack/.

Verify that the named task belongs to the current phase and that its prerequisites are landed. Use
fresh built-in default implementation/fix and review subagents. Complete the task's plan -> proof ->
atomic commit -> fresh review -> fix commit -> fresh review loop without asking for routine approval.
Update only the task-local authority and ledger rows justified by actual evidence.

Do not start another task, close the phase, or change the next phase's status. If the task exposes a
contract choice or would exceed its packet, return the structured DECISION REQUIRED report. If it
only exposes an ordinary in-scope defect, fix it and continue.

Finish with the task result, commits, proof, review result, ledger updates, remaining phase work,
and the exact next task id.
```

## Prompt 3 — Resume From Live Repository Truth

Use after interruption, context compaction, an agent handoff, or uncertainty about what actually
landed.

```text
/goal Reconstruct the current hybrid-drift continuation point from live repository truth and resume
only the already-authorized work, involving me only if the continuation is genuinely ambiguous.

REPO: /Users/spensermcconnell/.codex/worktrees/97a0/substrate
EXPECTED_PHASE: <phase id or unknown>
EXPECTED_PACKET: <packet id, none, or unknown>
AUTONOMY_MODE: escalation-only

Do not trust conversation summaries as landed-state authority. Read AGENTS.md, then inspect git
status, HEAD and recent commits, the control-pack status/phase maps, canonical tasks, open ledger
rows, and test/review receipts. Determine:
- the last review-clean commit;
- whether uncommitted changes belong to the active work;
- the active phase and packet;
- the first incomplete acceptance item; and
- whether a review or transition step is unfinished.

If the continuation is unambiguous, follow the reusable phase runner and continue from that exact
step through its normal commit/review boundary. Do not repeat proven work and do not advance merely
because an earlier response claimed completion.

If two materially different continuation points remain possible, stop only the conflicting scope
and return a structured DECISION REQUIRED report with a recommendation. If an external artifact or
permission is missing, return ACTION REQUIRED. Preserve unrelated worktree changes.
```

## Prompt 4 — Complete Independent Review And Auto-Fix

Use when implementation or docs are committed but a clean independent review has not been proven.

```text
/goal Bring the named committed change through a fresh independent review -> fix -> commit -> fresh
review loop until review-clean, without widening its phase or packet.

REPO: /Users/spensermcconnell/.codex/worktrees/97a0/substrate
PHASE_ID: <phase id>
ACTIVE_PACKET: <packet id or none>
CHANGE_RANGE: <commit, base..head, or derive from the packet ledger>
AUTONOMY_MODE: escalation-only

Read the selected phase authority, manifest, and proof output. Dispatch a fresh built-in default
review subagent with only the committed diff, relevant authority, manifest, and verification
receipts. Require findings-first output with exact evidence and an explicit REVIEW-CLEAN or CHANGES
REQUIRED verdict.

For CHANGES REQUIRED, dispatch a fresh built-in default fix subagent, keep the patch inside the
named phase/packet, run applicable impact analysis and proof, commit the non-empty fix batch, and
dispatch another fresh reviewer. Repeat automatically until clean. Do not self-approve and do not
use shell reviewer stand-ins.

Ignore optional/nit findings unless they materially improve the scoped change without widening it.
Escalate only if a required finding demands a new product decision, HIGH/CRITICAL blast radius,
destructive handling, or scope expansion not authorized by the packet.

Finish with reviewed commits, required findings and resolutions, exact verification, final verdict,
and whether the phase closeout gate is now eligible.
```

## Prompt 5 — Close One Phase And Activate The Next Gate

Use when the work appears complete but phase status and the next entry gate have not been reconciled.

```text
/goal Audit and, only if proven, close the current hybrid-drift phase and activate the next eligible
phase without beginning its implementation.

REPO: /Users/spensermcconnell/.codex/worktrees/97a0/substrate
PHASE_ID: <current phase id>
EXPECTED_NEXT_PHASE: <next phase id>
AUTONOMY_MODE: escalation-only
STOP_AT: transition commit review-clean; no next-phase implementation

Read the phase/gate map, current phase authority, tasks, open ledger rows, committed proof receipts,
and fresh-review verdicts. Re-run the smallest deterministic verification needed to validate the
exit gate. Distinguish task completion, review cleanliness, invariance evidence, behavioral proof,
and terminal disposition.

If the exit gate is proven:
1. update canonical status documents, task ledgers, this pack, and the proof ledger consistently;
2. mark the current phase COMPLETE and the next phase ACTIVE when its entry gate is proven, or keep
   it BLOCKED and record the exact unmet entry gate;
3. update the verified commit reference without overstating behavior;
4. commit the transition atomically;
5. dispatch a fresh built-in default reviewer over the transition diff; and
6. fix and re-review any cross-document findings automatically.

If the gate is not proven, do not edit status to simulate closure. If missing work is already
ordered and unambiguous, return a filled Prompt 1 or Prompt 2 for that work; do not ask me to decide.
Use DECISION REQUIRED only when repository authority cannot determine the needed behavior or scope.

Finish with the gate verdict, transition commit/review result if any, and a filled Prompt 1 for the
next eligible phase. Do not start that phase.
```

## Prompt 6 — Apply A Decision Or External Unblock And Resume

Use after responding to a structured escalation.

```text
/goal Record the supplied decision or verify the supplied external unblock, then resume the exact
blocked hybrid-drift scope through its normal proof, commit, and review-clean boundary.

REPO: /Users/spensermcconnell/.codex/worktrees/97a0/substrate
PHASE_ID: <phase id>
ACTIVE_PACKET: <packet id or none>
ESCALATION_ID: <DECISION/ACTION id>
RESOLUTION: <exact selected option or completed action>
RATIONALE: <optional operator rationale>
AUTONOMY_MODE: escalation-only

First verify that the resolution matches the reported escalation and that no intervening repository
change invalidated it. Record a product/authority decision in the owning spec, task, finding, ledger,
or ADR as appropriate; do not bury it only in conversation history. For an external action, verify
the stated resume condition directly.

Then continue only the previously blocked scope using the reusable phase runner. Complete ordinary
implementation, proof, commit, fresh review, and fix loops autonomously. Do not reopen the resolved
choice unless new evidence directly invalidates its assumptions; if that happens, issue a new
stable escalation id and explain the new evidence.

Finish with where the resolution was recorded, resumed work and commits, verification, review
result, current phase status, and the next eligible interaction.
```

## Current First Invocation

Final gap implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` received fresh
independent built-in `default` `REVIEW CLEAN`. Authority transition series `56bb9966f` +
`07a3b1fe5` also received fresh independent built-in `default` `REVIEW CLEAN`, marks aggregate
`R6-GAP-*` complete, and makes `R6-REPLAY` the sole concrete active phase with active packet `none`.
The next eligible default invocation uses Prompt 1 with:

```text
PHASE_ID: R6-REPLAY
ACTIVE_PACKET: none
```

The session must re-check that status from live repo truth and execute only the selected replay
phase. The review-clean transition did not assign terminal scorer dispositions and did not start
replay, `R6-CLOSE`, R7, or R8 work. This example is not a permanent status override.
