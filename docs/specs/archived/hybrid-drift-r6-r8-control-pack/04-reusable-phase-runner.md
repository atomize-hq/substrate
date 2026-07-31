# Reusable Hybrid Drift Phase Runner

Copy this prompt into a fresh session. Change `PHASE_ID`; set `ACTIVE_PACKET` only for a bounded
packet inside that phase.

```text
REPO: /Users/spensermcconnell/.codex/worktrees/97a0/substrate
PHASE_ID: <PACK-0 | R6-C.0A | R6-C.1-SPEC | R6-C.1-CONTROLS | R6-GAP-* | R6-REPLAY | R6-CLOSE | R7-PROMOTE | R7-0..R7-6 | R8-SPEC | R8-IMPLEMENT>
ACTIVE_PACKET: <none or exact packet id>
AUTONOMY_MODE: escalation-only

OBJECTIVE
Execute exactly the selected hybrid-drift phase/packet using the R6-R8 control pack. Ground every
status and behavior claim in live repo truth. Do not advance to the next phase implicitly.

STARTUP
1. Read `AGENTS.md` and invoke `using-agent-skills` plus the phase skills named by
   `docs/specs/hybrid-drift-r6-r8-control-pack/03-selective-context-manifests.md`.
2. Run:
   - `git status --short --branch`
   - `git rev-parse HEAD`
   - `npx gitnexus status`
3. If GitNexus is stale, run:
   - `npx gitnexus analyze --name 97a0-substrate --index-only`
4. Read:
   - `docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`
   - the selected phase in `02-phase-and-gate-map.md`
   - the matching manifest in `03-selective-context-manifests.md`
   - open selected-phase rows in `05-proof-decision-regression-ledger.md`
5. Load only the canonical docs, source, tests, and fixtures named by that manifest.

AUTHORITY
- This pack is a context router, not semantic authority.
- Live source/tests establish current behavior; active finding/MAP/SPEC/PLAN/TASKS establish scope
  and acceptance.
- If they conflict, stop implementation, name the conflict, and reconcile authority first.
- Preserve historical packet truth without treating old status prose as current.

SCOPE INVARIANTS
- Do not force irrelevant context layers into scorers.
- Do not reopen `semantic_goal_drift` without new behavior-level failing evidence.
- Do not begin R7 before R6 is CLOSED with terminal scorer dispositions.
- Do not make R7 absorb ordinary single-session scorer gaps.
- Never infer child work from parent orchestration alone.
- Once R7 has stabilized the analyzer-owned contract and `R8-SPEC` is active at entry, R8
  MAP/SPEC/PLAN/TASKS authoring may begin.
- Do not begin R8 implementation or `R8-IMPLEMENT` until those R8 MAP/SPEC/PLAN/TASKS artifacts are
  landed and review-clean.
- R8 means Sentinel Interpretation Consolidation / Integration; analyzer semantics stay
  analyzer-owned.
- Preserve unrelated worktree changes.

PLANNING
Before editing, state assumptions and produce a lightweight plan with one in-progress step. For
R6-C.1-SPEC and R8-SPEC, write and review the specification/plan/task artifacts before code. For a
failed control, preserve the witness and create a bounded fix packet before production changes.
Do not pause for routine approval of a repo-grounded plan when `AUTONOMY_MODE` is
`escalation-only`.

CODE INTELLIGENCE
Before editing any function, class, or method:
1. run GitNexus upstream impact analysis for the exact symbol;
2. report direct callers, affected flows, and risk;
3. stop and warn on HIGH or CRITICAL risk; and
4. never rename with search/replace.

IMPLEMENTATION
- Use test-driven development: failing behavior witness first, smallest fix second, focused proof
  third.
- Keep one scorer/failure or one R7/R8 contract seam per commit.
- Do not mix docs-only authority repair with production behavior.
- Run the packet-local wall before widening to the family wall.
- Update findings/tasks/ledger with actual results, not intended results.

R6 CONTROL RULE
R6-C.1 begins with acceptance controls only. If existing behavior passes honestly, record
fit-for-purpose proof. If it fails, preserve the failing test and open a scorer-specific fix packet.
If the charter is overbroad, narrow it instead of manufacturing behavior.

R6 CLOSE RULE
R6 may close only when every material scorer is cutover complete, a behaviorally proven
fit-for-purpose exception, merged/deprecated, or explicitly deferred outside R6 with justification,
owner, and trigger. No ordinary implementation or acceptance-proof gap may remain open.

R7 RULE
Implement direct reciprocal parent/child linkage first. Keep parent and child trajectories separate.
The compactor parses raw metadata; analyzer and sentinel consume typed contracts. Add no new drift
class without acceptance evidence. R8 consolidation stays out.

R8 RULE
Before code, create and review R8 MAP/SPEC/PLAN/TASKS from the stabilized R7 contract. Consolidate
replay/live checkpoint interpretation behind one seam, centralize compatibility, and keep operator
surface presentation-first. Do not redesign scheduling/adjudication unless separately approved.

VERIFICATION AND COMMIT
1. Run packet-local tests, then the applicable family wall.
2. Stage only the intended files with `git add -- <intended-files-only>`; leave unrelated dirt unstaged.
3. Run `npx gitnexus detect-changes --scope staged -r 97a0-substrate`.
4. Run `git diff --cached --check`.
5. Inspect the complete staged diff with `git diff --cached`.
6. Commit atomically with a descriptive Conventional Commit message.

FRESH REVIEW WALL
After the commit, dispatch a fresh built-in `default` subagent. The reviewer must read the selected
phase authority, inspect the committed diff, and return findings-first with exact evidence. Do not
self-approve. If findings exist, fix them in a new commit and repeat with a fresh reviewer until
clean. Do not use shell or `codex exec` reviewer stand-ins.

OPERATOR ESCALATION
- Continue autonomously through ordinary implementation, focused test failures, bounded fixes,
  commits, reviewer findings, and ledger updates that live authority already determines.
- A failing R6 acceptance control is not by itself a reason to ask the operator: preserve it and
  prepare the bounded conditional gap packet required by the phase map.
- Involve the operator only for an unresolved product/authority choice, a HIGH/CRITICAL GitNexus
  blast radius, destructive or unrelated-work handling, a required external action, or a scope
  change not already authorized.
- Use the structured `DECISION REQUIRED` or `ACTION REQUIRED` report in
  `06-operator-prompt-library.md`; do not ask open-ended progress questions.

STOP CONDITIONS
- selected phase entry gate is not met;
- authority documents contradict and live repo truth cannot resolve the intended contract;
- a supposedly fit-for-purpose scorer control fails;
- work would cross into the next family;
- a required real rollout/fixture is unavailable or untrusted;
- GitNexus impact is HIGH/CRITICAL and the user has not accepted the blast radius; or
- unrelated worktree changes cannot be isolated safely.

CLOSEOUT
Report:
- phase/packet completed or exact blocking condition;
- files and commits;
- behavior-level tests and bounded replay evidence;
- scorer/contract dispositions changed;
- ledger and authority updates;
- fresh-review result; and
- the next phase gate, without starting it.
```

## Runner Notes

- `PHASE_ID` is the canonical session tracking unit. `ACTIVE_PACKET` narrows work without changing
  the family phase.
- Routine implementation stays in one focused session; phase transitions start fresh sessions.
- A review session receives only the committed diff, phase authority, applicable manifest, and
  proof output—not the entire authoring conversation.
- The copy-paste interaction prompts and escalation contract live in
  `06-operator-prompt-library.md`.
- If a task ends with “thoughts?” or similar, provide analysis only and make no edits.
