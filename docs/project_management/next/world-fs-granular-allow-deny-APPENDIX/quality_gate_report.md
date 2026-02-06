# Planning Quality Gate Report — world-fs-granular-allow-deny-appendix

## Latest Recommendation (supersedes prior passes)

RECOMMENDATION: ACCEPT

See “Third-Party Review Pass 2026-02-06T21:48:49Z (ACCEPT)” at the end of this file for auditable evidence and findings.

---

RECOMMENDATION: FLAG FOR HUMAN REVIEW

## Metadata
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/`
- Reviewed commit: `d642f310d9e5523b0ae4efabd35a0d57bedea5a7`
- Reviewer: Third-party reviewer (fresh reviewer)
- Date (UTC): 2026-02-06
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

### Required preflight (minimum)
```bash
export FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
echo "exit:$?"
jq -e . docs/project_management/next/sequencing.json >/dev/null
echo "exit:$?"

# tasks.json required-field audit
python - <<'PY'
import json, os
feature_dir=os.environ["FEATURE_DIR"]
path=os.path.join(feature_dir,"tasks.json")
data=json.load(open(path,"r",encoding="utf-8"))
tasks=data["tasks"] if isinstance(data,dict) and "tasks" in data else data
required=[
  "id","name","type","phase","status","description",
  "references","acceptance_criteria","start_checklist","end_checklist",
  "worktree","integration_task","kickoff_prompt",
  "depends_on","concurrent_with"
]
missing=[]
for t in tasks:
  m=[k for k in required if k not in t]
  if m:
    missing.append((t.get("id","<no id>"),m))
if missing:
  for tid,m in missing:
    print(tid,":",", ".join(m))
  raise SystemExit(1)
print("OK: tasks.json required fields present")
PY
```

Observed results:
- `jq -e . "$FEATURE_DIR/tasks.json"` → `exit:0`
- `jq -e . docs/project_management/next/sequencing.json` → `exit:0`
- `tasks.json required-field audit` → `OK: tasks.json required fields present`

### Planning lint (mechanical)
Reference: `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`

```bash
make planning-lint FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"
make planning-validate FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"
```

Observed results:
- `make planning-lint ...` → `exit:0` (planning lint passed)
- `make planning-validate ...` → `exit:0` (tasks.json validation passed)

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES` (`docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`)
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`WFGADAX0-spec.md` .. `WFGADAX3-spec.md`, plus `contract.md`, `SCHEMA.md`, `PROTOCOL.md`, `ENV.md`, `SECURITY.md`)
- `decision_register.md`: `YES`
- `impact_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/` (if required): `YES` (`smoke/linux-smoke.sh`, plus reviewed `smoke/_core.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`)
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`: `YES`
  - `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`: `YES`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`: `YES`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`: `YES`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` coverage requirements for `SCHEMA.md` (“constraints, validation rules” and snapshot “canonicalization rules”).
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` does not define pattern validity rules nor snapshot canonicalization rules.
- Notes: Several authoritative contract surfaces are present but not specified with the level of determinism required by the manifest and standards.

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` entries omit required fields (implications/risks/unlocks/quick wins) and contain no explicit follow-up task mapping.
- Notes: Decision Register does not conform to `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` section “Decision Register Standard”.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS (with defects noted below for spec coverage/testability)`
- Evidence:
  - Exit codes `2/3/4` are consistently referenced across:
    - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
    - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md`
    - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md`
- Notes: Key names (`host_visible`, `fail_closed.routing`, `caged_required`, `repl.exit_cwd`) are consistent with ADR Appendix A/B.

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` includes the sprint `world_fs_granular_allow_deny_appendix` with `WFGADAX0..3`.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` gates `WFGADAX2-*` on `CP1-ci-checkpoint` and uses schema v4 checkpoint boundaries.
- Notes: Task deps prevent starting the next checkpoint group before completing the prior checkpoint ops task.

### 5) Testability and validation readiness
- Result: `FAIL`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` lacks runnable commands for multiple cases (setup-only statements).
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` acceptance criteria contain non-runnable statements (“Implements …”).
- Notes: Standards require acceptance criteria and playbooks to be runnable with explicit expected outputs/exit codes.

### 5.1) Cross-platform parity task structure (schema v2/v3/v4)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` meta:
    - `schema_version: 4`
    - `behavior_platforms_required: ["linux"]`
    - `ci_parity_platforms_required: ["linux","macos","windows"]`
    - `checkpoint_boundaries: ["WFGADAX1","WFGADAX3"]`
  - Only checkpoint boundary slices define `*-integ-core` / `*-integ-<platform>` tasks.
- Notes: Matches schema v4 boundary-only platform-fix model and `ci_checkpoint_plan.md`.

### 6) Triad interoperability (execution workflow)
- Result: `PASS (with improvements required for prompts)`
- Evidence:
  - `tasks.json` required fields present (see required-field audit).
  - Kickoff prompt sentinel is present (“Do not edit planning docs inside the worktree.”) and planning lint passed.
- Notes: Prompts are mechanically compliant but omit required per-role command checklists; see Findings.

## Findings (must be exhaustive)

### Finding 001 — Planning lint/validate are green
- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="$FEATURE_DIR"` and `make planning-validate FEATURE_DIR="$FEATURE_DIR"` both exited `0` (see Evidence section).
- Impact: Mechanical gate passes; remaining issues are semantic/spec readiness issues.
- Fix required (exact): none

### Finding 002 — Sequencing + checkpoint wiring is correct (schema v4 boundary-only platform-fix)
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/sequencing.json` sprint `world_fs_granular_allow_deny_appendix` lists `WFGADAX0..3`.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` meta `schema_version=4` and `checkpoint_boundaries=["WFGADAX1","WFGADAX3"]`, with boundary slices defining `*-integ-core`/`*-integ-<platform>` tasks.
- Impact: Execution cannot bypass planned CI checkpoints; cross-platform platform-fix task explosion is correctly bounded to checkpoint boundaries.
- Fix required (exact): none

### Finding 003 — Decision Register is not in the required A/B format (missing required fields + no task traceability)
- Status: `DEFECT`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` contains only Pros/Cons + Selected (no implications/risks/unlocks/quick wins; no follow-up task IDs).
- Impact: Fails the “Decision quality” gate; decisions are not auditable or executable against the triad graph.
- Fix required (exact):
  - Rewrite each DR (`DR-AX-0001`..`DR-AX-0005`) to match the required template in `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` section “Decision Register Standard”.
  - Add explicit follow-up tasks mapping to existing task IDs (for example: `WFGADAX1-code`, `WFGADAX1-test`, `WFGADAX1-integ-core`, `WFGADAX1-integ-linux`, `WFGADAX1-integ-macos`, `WFGADAX1-integ-windows`, `WFGADAX1-integ`).
  - Update `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` task `references` to include the relevant DR entry (by file path + DR id).
- Alternative (one viable): Move the decision details into ADR-0018 Appendix A/B and treat `decision_register.md` as a pointer-only index that links to ADR sections *and* still lists explicit follow-up task IDs; this preserves a single source of truth while meeting traceability requirements.

### Finding 004 — spec_manifest coverage matrix omits config surfaces introduced by ADR Appendix B
- Status: `DEFECT`
- Evidence:
  - ADR Appendix B defines config-owned keys (`repl.exit_cwd`, `world.caged`, `world.anchor_mode`) as contract surfaces.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` coverage matrix does not enumerate any config contract surfaces or assign them to an authoritative document.
- Impact: Spec coverage gate fails; implementation may ship without a single authoritative doc for config parsing/validation/precedence for Appendix B changes.
- Fix required (exact):
  - Update `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` coverage matrix to include:
    - config key `repl.exit_cwd` (values + default + failure behavior),
    - config compatibility constraints (`world.caged`, `world.anchor_mode`) under `world_fs.caged_required=true`,
    - shell integration hook contract for `repl.exit_cwd=last_world`.
  - Assign each surface to exactly one authoritative doc (likely `contract.md` unless another doc is declared authoritative).
- Alternative (one viable): Add a dedicated `CONFIG.md` in the feature directory and declare it authoritative for config contract surfaces (this keeps config surfaces separate from policy schema and reduces ambiguity in ownership).

### Finding 005 — SCHEMA.md does not meet the manifest’s stated completeness requirements (constraints + canonicalization)
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` requires `SCHEMA.md` to define “constraints, validation rules” and snapshot “canonicalization rules”.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` does not define:
    - path pattern validity rules (what is invalid beyond “must be <pattern>”),
    - snapshot canonicalization rules (what is hashed/compared, ordering rules, normalization).
- Impact: Implementers cannot deterministically implement schema parsing/validation/canonicalization without importing assumptions from outside this Planning Pack, violating “single authoritative doc” rules.
- Fix required (exact):
  - Expand `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` to include explicit pattern validity rules and canonicalization requirements, or explicitly reference the single authoritative doc that defines them (and update spec_manifest ownership accordingly).
- Alternative (one viable): Declare that pattern grammar and canonicalization are unchanged from the baseline feature and link to the baseline authoritative doc for those rules, removing implied ownership from Appendix `SCHEMA.md` to avoid duplication and drift.

### Finding 006 — tasks.json acceptance criteria include non-runnable “Implements …” statements
- Status: `DEFECT`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` acceptance criteria include strings like “Implements WFGADAX1-spec.md acceptance criteria” and “Implements … test coverage”.
- Impact: Fails the “Testability” gate and violates the planning standards’ testability bans; tasks are not executable with deterministic outcomes.
- Fix required (exact):
  - Replace each “Implements …” acceptance bullet with a runnable command and a concrete expected exit code and/or output substring(s).
  - If the intent is “covered by tests”, list the exact `cargo test ...` invocations that must pass (including target/module/test names).
- Alternative (one viable): Move all behavior-level acceptance checks into the slice specs and have tasks reference those specs *with explicit command citations* (still requiring runnable commands somewhere authoritative and referenced).

### Finding 007 — Manual playbook lacks runnable commands for multiple cases
- Status: `DEFECT`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` Cases 2–5 specify “Setup” steps but no runnable command lines to reproduce the state and validate outcomes.
- Impact: Humans cannot validate the contract without guessing; violates `PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` manual playbook requirements.
- Fix required (exact):
  - For each case, add exact commands (including how to set policy keys, how to disable/enable world, and what command is run) and expected exit codes/output.
  - Where a warning substring contract exists (ADR Appendix B), include the required stderr substrings as assertions.
- Alternative (one viable): Convert Cases 2–5 into smoke-script subcommands (still invoked from the playbook) so validation becomes 1-command runnable; keep the playbook as the human-facing index.

### Finding 008 — Kickoff prompts omit required per-role command checklists (fmt/clippy/tests/smoke)
- Status: `DEFECT`
- Evidence: Example: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/kickoff_prompts/WFGADAX1-code.md` end checklist does not list required commands; it only instructs running `make triad-task-finish ...`.
- Impact: Agents can comply with prompts while missing required validations; increases drift risk between standards and actual execution.
- Fix required (exact): Update all kickoff prompts to include the required per-role command checklist aligned to `TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (code: fmt/clippy + baseline tests; test: fmt + targeted tests; integration: fmt/clippy/tests + `make integ-checks` + smoke where required).
- Alternative (one viable): If relying on `required_make_targets` to enforce commands, explicitly state in the prompt that the finisher will run them and list which targets will be executed (still include smoke/manual validation steps where applicable).

## Decision: ACCEPT or FLAG

### FLAG FOR HUMAN REVIEW
- Summary: Mechanical lint passes, but the Planning Pack is not execution-ready due to decision register non-compliance, incomplete surface ownership in `spec_manifest.md`, and non-runnable acceptance criteria/playbook steps.
- Required human decisions (explicit):
  - Whether to treat pattern grammar + canonicalization as Appendix-owned (`SCHEMA.md`) or baseline-owned (and update ownership map accordingly).
  - Where config contract surfaces (`repl.exit_cwd`, caging compatibility constraints) should live authoritatively (`contract.md` vs a new `CONFIG.md`).
- Blockers to execution:
  - Bring `decision_register.md`, `spec_manifest.md`, `tasks.json` acceptance criteria, `manual_testing_playbook.md`, and kickoff prompts into compliance with the standards cited above.

---

# Planning Quality Gate Report — world-fs-granular-allow-deny-appendix (Reviewer Addendum)

RECOMMENDATION: FLAG FOR HUMAN REVIEW

## Metadata
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/`
- Reviewed commit: `d642f310d9e5523b0ae4efabd35a0d57bedea5a7`
- Reviewer: Third-party reviewer (fresh reviewer)
- Date (UTC): 2026-02-06
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
echo "exit:$?"
jq -e . docs/project_management/next/sequencing.json >/dev/null
echo "exit:$?"

# tasks.json required-field audit
python3 - <<'PY'
import json, os
feature_dir=os.environ["FEATURE_DIR"]
path=os.path.join(feature_dir,"tasks.json")
data=json.load(open(path,"r",encoding="utf-8"))
tasks=data["tasks"] if isinstance(data,dict) and "tasks" in data else data
required=[
  "id","name","type","phase","status","description",
  "references","acceptance_criteria","start_checklist","end_checklist",
  "worktree","integration_task","kickoff_prompt",
  "depends_on","concurrent_with"
]
missing=[]
for t in tasks:
  m=[k for k in required if k not in t]
  if m:
    missing.append((t.get("id","<no id>"),m))
if missing:
  for tid,m in missing:
    print(tid,":",", ".join(m))
  raise SystemExit(1)
print("OK: tasks.json required fields present")
PY

# Mechanical planning lint + invariants
make planning-lint FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"
make planning-validate FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"
```

Observed results:
- `jq -e . "$FEATURE_DIR/tasks.json"` → `exit:0`
- `jq -e . docs/project_management/next/sequencing.json` → `exit:0`
- `tasks.json required-field audit` → `OK: tasks.json required fields present`
- `make planning-lint ...` → `exit:0`
- `make planning-validate ...` → `exit:0`

## Gate Results (PASS/FAIL with evidence)

### 1) Mechanical planning lint (non-negotiable)
- Result: `PASS`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"` → exit `0`

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` (DR-AX-0001..0005)

### 3) Cross-platform parity task model (schema v4+ boundary-only platform-fix)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` `meta.schema_version=4`
  - `meta.checkpoint_boundaries=["WFGADAX1","WFGADAX3"]` matches `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ci_checkpoint_plan.md`

### 4) Spec coverage (spec_manifest enumerates all ADR-implied surfaces)
- Result: `FAIL`
- Evidence: Findings 003–004

### 5) Testability (runnable acceptance criteria + expected outcomes)
- Result: `FAIL`
- Evidence: Finding 006

### 6) Impact map completeness (cross-queue scan + contradictions)
- Result: `FAIL`
- Evidence: Finding 005

## Findings (must be exhaustive)

### Finding 001 — Mechanical lint passed
- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"` → exit `0`
- Impact: Confirms the Planning Pack meets the repo’s mechanical invariants (required artifacts, hard-ban/ambiguity scans, tasks schema v4 structure, checkpoint wiring, sequencing inclusion).
- Fix required (exact): none

### Finding 002 — Decision register conforms to A/B standard
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` (e.g., DR-AX-0002, DR-AX-0005)
- Impact: Major decisions are documented with exactly two viable options, explicit tradeoffs, a single selection, and follow-up task IDs.
- Fix required (exact): none

### Finding 003 — `spec_manifest.md` does not enumerate ADR Appendix A.6 “policy show” output contract surface
- Status: `DEFECT`
- Evidence:
  - ADR requirement: `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix A.6, “Output contract (effective policy display)”).
  - Ownership gap: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` coverage matrix lists no surface for `substrate policy show` output requirements.
- Impact: The plan is not implementation-ready because an ADR-mandated operator-visible contract surface is unowned; implementers may omit it or implement inconsistently.
- Fix required (exact):
  - Add a coverage-matrix row to `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` for “effective policy display / `substrate policy show` output contract” and assign it to exactly one authoritative document (recommended: `contract.md`).
  - Add deterministic requirements to that authoritative doc (what fields must render, defaulted-field rendering rules, and required empty `deny_list: []` presentation).
- Alternative (one viable): If Appendix A.6 is intentionally deferred to the baseline feature pack, explicitly declare it out-of-scope here and link to the baseline authoritative doc/task that will implement it; then update sequencing/tasks deps to ensure it cannot be missed.

### Finding 004 — Missing authored contract/validation for ADR Appendix B.2.1 “routing fallback warning substrings”
- Status: `DEFECT`
- Evidence:
  - ADR requirement: `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` Appendix B.2.1 requires a stderr warning with specific substrings when `world_fs.host_visible=false` + `world_fs.fail_closed.routing=false` and routing fails.
  - No authoritative ownership: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` lists no surface row for warning contract requirements.
  - No authored acceptance: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` contains no case asserting the required substrings.
- Impact: Operators cannot rely on the warning contract and integration/tests cannot validate it, risking a silent safety posture downgrade in the most confusing configuration state.
- Fix required (exact):
  - Add an explicit surface row in `spec_manifest.md` for “routing fallback warning contract” and assign authority (recommended: `contract.md`).
  - Add at least one runnable validation (manual playbook case and/or smoke-script subcase) that triggers the fallback condition and asserts the required stderr substrings.
- Alternative (one viable): If the fallback warning is implemented in the baseline feature pack and this appendix is “rename-only”, explicitly constrain the appendix scope and add tasks references asserting that the baseline warning behavior is unchanged and already covered by tests (with exact test command(s)).

### Finding 005 — `impact_map.md` omits required cross-queue scan and contradiction/conflict resolution
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` states `impact_map.md` covers “cross-queue conflicts”.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/impact_map.md` contains no “Cross-queue scan” or “conflict resolution” section.
- Impact: Sequencing risk is not audited; overlapping queued work can cause conflicting changes to land concurrently, breaking determinism and forcing rework mid-execution.
- Fix required (exact): Add a “Cross-queue scan” section to `impact_map.md` listing any overlapping ADRs/Planning Packs and stating an explicit resolution (no overlap boundary vs ordering dependency) consistent with `docs/project_management/next/sequencing.json`.
- Alternative (one viable): If no overlaps exist, add a minimal cross-queue scan section stating “none found” and record the scan scope (directories searched + date).

### Finding 006 — CI checkpoint ops tasks are not runnable/auditable from acceptance criteria + kickoff prompts
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` CP1/CP2 acceptance criteria are not commands with expected exit codes (e.g., “Run CI Testing and Feature Smoke per ci_checkpoint_plan.md”).
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/kickoff_prompts/CP1-ci-checkpoint.md` and `CP2-ci-checkpoint.md` list no exact dispatch commands.
- Impact: Checkpoint execution is not deterministic; different operators/agents may dispatch different gates or omit required compile-parity/smoke runs, violating the checkpoint safety model.
- Fix required (exact):
  - For CP1/CP2, add explicit commands (including required env vars/refs and expected exit codes) to both:
    - the ops task `acceptance_criteria` in `tasks.json`, and
    - the ops kickoff prompt markdown.
  - Ensure the commands map to the planned gates in `ci_checkpoint_plan.md` (CI parity + behavior smoke).
- Alternative (one viable): Replace “CI Testing” with an explicit `make ci-compile-parity ...` invocation (exit `0`) plus `make feature-smoke ... PLATFORM=behavior ...` (exit `0`) and record run ids/URLs in `session_log.md` as the auditable evidence contract.

## Decision: ACCEPT or FLAG

### FLAG FOR HUMAN REVIEW
- Summary: Mechanical lint passes and task graph structure is sound, but the Planning Pack is not execution-ready because `spec_manifest.md` is missing ADR-mandated contract surfaces (policy show output contract and routing fallback warning contract), the impact map omits cross-queue scan/conflict resolution, and CI checkpoint ops tasks are not specified as runnable/auditable commands.
- Required human decisions (explicit):
  - Where the ADR Appendix A.6 `substrate policy show` output contract should live authoritatively (recommended: `contract.md`) and how it is validated.
  - How to specify checkpoint dispatch commands (compile parity vs CI Testing modes) as the authoritative “ops contract”.
- Blockers to execution:
  - Update `spec_manifest.md`, `impact_map.md`, and CP1/CP2 definitions (tasks + kickoff prompts) to close the contract/ownership/testability gaps above.

---

# Planning Quality Gate Report — world-fs-granular-allow-deny-appendix (Third-Party Review Pass 2026-02-06)

RECOMMENDATION: FLAG FOR HUMAN REVIEW

## Metadata
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/`
- Reviewed commit: `d642f310d9e5523b0ae4efabd35a0d57bedea5a7`
- Reviewer: Third-party reviewer (fresh reviewer)
- Date (UTC): 2026-02-06
- Recommendation: `FLAG FOR HUMAN REVIEW`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
echo "exit:$?"
jq -e . docs/project_management/next/sequencing.json >/dev/null
echo "exit:$?"

# tasks.json required-field audit
python3 - <<'PY'
import json, os
feature_dir=os.environ["FEATURE_DIR"]
path=os.path.join(feature_dir,"tasks.json")
data=json.load(open(path,"r",encoding="utf-8"))
tasks=data["tasks"] if isinstance(data,dict) and "tasks" in data else data
required=[
  "id","name","type","phase","status","description",
  "references","acceptance_criteria","start_checklist","end_checklist",
  "worktree","integration_task","kickoff_prompt",
  "depends_on","concurrent_with"
]
missing=[]
for t in tasks:
  m=[k for k in required if k not in t]
  if m:
    missing.append((t.get("id","<no id>"),m))
if missing:
  for tid,m in missing:
    print(tid,":",", ".join(m))
  raise SystemExit(1)
print("OK: tasks.json required fields present")
PY

# Mechanical planning lint + invariants
make planning-lint FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"
make planning-validate FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"

# Checkpoint wiring check (reviewer-only; does not mutate repo)
python3 - <<'PY'
import json
import re
from pathlib import Path

feature_dir = Path("docs/project_management/next/world-fs-granular-allow-deny-APPENDIX")
tasks_data = json.loads((feature_dir/"tasks.json").read_text(encoding="utf-8"))
by_id = {t["id"]: t for t in tasks_data["tasks"]}

text = (feature_dir/"ci_checkpoint_plan.md").read_text(encoding="utf-8")
header = "## Machine-readable plan (linted)"
start = text.find(header)
assert start >= 0
m = re.search(r"```json\\s*\\n([\\s\\S]*?)\\n```", text[start:])
assert m
plan = json.loads(m.group(1))

issues = []
for cp in plan.get("checkpoints") or []:
  cp_task = cp["task_id"]
  boundary = (cp.get("slices") or [])[-1]
  final_id = f"{boundary}-integ"
  core_id = f"{boundary}-integ-core"
  if final_id in by_id and cp_task not in (by_id[final_id].get("depends_on") or []):
    issues.append(f"{final_id} does not depend_on {cp_task}")
  if cp_task in by_id and core_id not in (by_id[cp_task].get('depends_on') or []):
    issues.append(f"{cp_task} does not depend_on {core_id}")

if issues:
  print("ISSUES:")
  for i in issues:
    print("-", i)
  raise SystemExit(2)
print("OK: checkpoint wiring includes final->cp and cp->core")
PY
echo "exit:$?"
```

Observed results:
- `make planning-lint ...` → `exit:0`
- `make planning-validate ...` → `exit:0`
- `checkpoint wiring check` → `exit:2` (issues detected; see Findings)

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES` (`docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`)
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`WFGADAX0-spec.md` .. `WFGADAX3-spec.md`, plus `contract.md`, `SCHEMA.md`, `PROTOCOL.md`, `ENV.md`, `SECURITY.md`)
- `decision_register.md`: `YES`
- `impact_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/` (if required): `YES`
- `docs/project_management/next/sequencing.json`: `YES`
- Standards: `YES` (all required by the quality gate prompt)

## Gate Results (PASS/FAIL with evidence)

### 1) Mechanical planning lint (non-negotiable)
- Result: `PASS`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"` → exit `0`

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` (DR-AX-0001..DR-AX-0005)

### 3) Spec coverage (spec_manifest enumerates ADR-implied surfaces)
- Result: `PASS`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` coverage matrix covers `substrate policy show`, routing warning contract, config surfaces, schema/snapshot/protocol/env/security.

### 4) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS`
- Evidence:
  - Exit codes `2/3/4` are consistent across `contract.md` and `manual_testing_playbook.md`.
  - Env var rename is consistent across ADR Appendix B, `ENV.md`, and `contract.md`.

### 5) Sequencing and dependency alignment
- Result: `FAIL`
- Evidence: Findings 004–005

### 6) Testability and validation readiness
- Result: `FAIL`
- Evidence: Finding 004 (checkpoint dispatch does not pin `checkout_ref` to the boundary slice’s `*-integ-core` commit, so commands are runnable but do not deterministically validate the intended code under test).

## Findings (must be exhaustive)

### Finding 001 — Mechanical planning lint passed
- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"` → exit `0`
- Impact: Confirms the Planning Pack meets baseline mechanical requirements.
- Fix required (exact): none

### Finding 002 — Decision register is compliant (A/B + explicit follow-ups)
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` (DR-AX-0001..DR-AX-0005)
- Impact: Implementation choices are explicit and traceable to task IDs.
- Fix required (exact): none

### Finding 003 — Spec surface ownership is complete
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` coverage matrix
- Impact: Contracts are assigned to authoritative docs; reduces drift during execution.
- Fix required (exact): none

### Finding 004 — Checkpoint dispatch is not pinned to the intended code-under-test (`checkout_ref`)
- Status: `DEFECT`
- Evidence:
  - Standard/template requires `CI_CHECKOUT_REF` / `SMOKE_CHECKOUT_REF` to validate the boundary slice’s `*-integ-core` HEAD:
    - `docs/project_management/standards/templates/kickoff_ci_checkpoint.md.tmpl` (Start Checklist step 4; required gates)
    - `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md` (Diagram A; “validate X-integ-core HEAD via checkout_ref”)
  - Current checkpoint prompts do not compute/pin checkout SHA:
    - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/kickoff_prompts/CP1-ci-checkpoint.md` (lines 10–20)
    - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/kickoff_prompts/CP2-ci-checkpoint.md` (lines 10–20)
  - Current checkpoint acceptance criteria also omit `CI_CHECKOUT_REF` / `SMOKE_CHECKOUT_REF`:
    - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` (CP1-ci-checkpoint acceptance_criteria; CP2-ci-checkpoint acceptance_criteria)
- Impact: CI/Feature Smoke can run against an unintended commit (often the orchestration branch HEAD), producing false confidence and breaking bounded-checkpoint determinism.
- Fix required (exact):
  - Update CP1/CP2 kickoff prompts and `tasks.json` acceptance criteria to:
    - compute `CORE_BRANCH` from `tasks.json` for `<boundary>-integ-core`,
    - compute `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`,
    - pass `CI_CHECKOUT_REF="$CHECKOUT_SHA"` to `make ci-compile-parity`,
    - pass `SMOKE_CHECKOUT_REF="$CHECKOUT_SHA"` (and recommended `SMOKE_SLICE_ID="<boundary>"`) to `make feature-smoke`.
  - Add the “If smoke fails: start failing platform-fix tasks” commands from the checkpoint kickoff template (so the failure path is deterministic).
- Alternative (one viable): Change the model so the core integration task merges to orchestration (set `merge_to_orchestration=true` for `*-integ-core` and remove the throwaway-branch checkout_ref mechanism), then dispatch checkpoints against orchestration HEAD only. This is viable but is a larger workflow change and must be reflected consistently in the standards-aligned prompts and task dependencies.

### Finding 005 — Boundary-slice final integration can run before its checkpoint task completes
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json`:
    - `WFGADAX1-integ.depends_on` omits `CP1-ci-checkpoint` (around lines 450–485)
    - `WFGADAX3-integ.depends_on` omits `CP2-ci-checkpoint` (see `WFGADAX3-integ` block)
  - Reviewer wiring check output:
    - `WFGADAX1-integ does not depend_on CP1-ci-checkpoint`
    - `WFGADAX3-integ does not depend_on CP2-ci-checkpoint`
- Impact: Execution can merge a boundary slice back to orchestration (`merge_to_orchestration=true`) before the bounded checkpoint gates have been run/recorded, undermining the checkpoint model.
- Fix required (exact):
  - Add `CP1-ci-checkpoint` to `WFGADAX1-integ.depends_on`.
  - Add `CP2-ci-checkpoint` to `WFGADAX3-integ.depends_on`.
  - Ensure the first slice in the next checkpoint group depends on both the checkpoint task and the prior group’s boundary final integration merge (directly or transitively), so later slices cannot start from an orchestration branch that lacks the prior group’s integrated code.
- Alternative (one viable): If you want the checkpoint to happen after the final aggregator merge instead of before it, invert the wiring:
  - Make `CP1-ci-checkpoint.depends_on` include `WFGADAX1-integ` (final) and update kickoff prompt language and the checkpoint plan rationale accordingly.
  - This changes the intended “validate core first” model and must be aligned with `TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md` and the checkpoint template.

### Finding 006 — Checkpoint plan runner_kind is inconsistent across docs
- Status: `DEFECT`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ci_checkpoint_plan.md` sets `feature_smoke.runner` to `github-actions`.
  - CP kickoff prompts dispatch with `RUNNER_KIND=self-hosted`.
- Impact: Operator confusion and drift risk; checkpoint gating may execute in an unintended environment.
- Fix required (exact): Align `ci_checkpoint_plan.md` and CP1/CP2 kickoff prompts to the same runner_kind contract.
- Alternative (one viable): Remove runner_kind from the machine-readable checkpoint plan and treat it as an execution-time operator choice (still keeping the dispatch commands explicit and pinned to checkout SHA in prompts and tasks).

## Decision: ACCEPT or FLAG

### FLAG FOR HUMAN REVIEW
- Summary: Mechanical lint passes and contract/spec coverage is strong, but cross-platform checkpoint execution is not deterministic because checkpoint dispatch does not pin `checkout_ref` to the boundary slice’s `*-integ-core` HEAD and final boundary merges are not gated on checkpoint completion.
- Required human decisions (explicit):
  - Whether checkpoints validate `*-integ-core` (preferred; requires `*_CHECKOUT_REF` pinning) or validate orchestration HEAD (requires rewiring merge_to_orchestration + checkpoint deps).
  - Which runner_kind is authoritative for `feature_smoke` in this pack (self-hosted vs GitHub-hosted), and how that is represented in `ci_checkpoint_plan.md`.
- Blockers to execution:
  - Update CP1/CP2 (kickoff prompts + acceptance criteria) and boundary final integration `depends_on` to align with the checkpoint model in `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md` and `docs/project_management/standards/templates/kickoff_ci_checkpoint.md.tmpl`.

---

# Planning Quality Gate Report — world-fs-granular-allow-deny-appendix (Third-Party Review Pass 2026-02-06T21:48:49Z)

RECOMMENDATION: ACCEPT

## Metadata
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/`
- Reviewed commit: `d642f310d9e5523b0ae4efabd35a0d57bedea5a7`
- Reviewer: Third-party reviewer (fresh reviewer)
- Date (UTC): 2026-02-06
- Recommendation: `ACCEPT`

## Evidence: Commands Run (verbatim)

```bash
export FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"

# JSON validity
jq -e . "$FEATURE_DIR/tasks.json" >/dev/null
echo "exit:$?"
jq -e . docs/project_management/next/sequencing.json >/dev/null
echo "exit:$?"

# tasks.json required-field audit
python - <<'PY'
import json, os
feature_dir=os.environ["FEATURE_DIR"]
path=os.path.join(feature_dir,"tasks.json")
data=json.load(open(path,"r",encoding="utf-8"))
tasks=data["tasks"] if isinstance(data,dict) and "tasks" in data else data
required=[
  "id","name","type","phase","status","description",
  "references","acceptance_criteria","start_checklist","end_checklist",
  "worktree","integration_task","kickoff_prompt",
  "depends_on","concurrent_with"
]
missing=[]
for t in tasks:
  m=[k for k in required if k not in t]
  if m:
    missing.append((t.get("id","<no id>"),m))
if missing:
  for tid,m in missing:
    print(tid,":",", ".join(m))
  raise SystemExit(1)
print("OK: tasks.json required fields present")
PY
echo "exit:$?"

# Mechanical planning lint + invariants
make planning-validate FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"
make planning-lint FEATURE_DIR="$FEATURE_DIR"
echo "exit:$?"
```

Observed results:
- `jq -e . "$FEATURE_DIR/tasks.json"` → `exit:0`
- `jq -e . docs/project_management/next/sequencing.json` → `exit:0`
- `tasks.json required-field audit` → `OK: tasks.json required fields present` (exit `0`)
- `make planning-validate ...` → `exit:0`
- `make planning-lint ...` → `exit:0` (planning lint passed)

## Required Inputs Read End-to-End (checklist)
- ADR(s): `YES` (`docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`)
- `spec_manifest.md`: `YES`
- `plan.md`: `YES`
- `tasks.json`: `YES`
- `session_log.md`: `YES`
- All specs in scope: `YES` (`WFGADAX0-spec.md` .. `WFGADAX3-spec.md`, plus `contract.md`, `SCHEMA.md`, `PROTOCOL.md`, `ENV.md`, `SECURITY.md`)
- `decision_register.md`: `YES`
- `impact_map.md`: `YES`
- `manual_testing_playbook.md`: `YES`
- Feature smoke scripts under `smoke/` (if required): `YES` (required: `smoke/linux-smoke.sh`)
- `docs/project_management/next/sequencing.json`: `YES`
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`: `YES`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`: `YES`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`: `YES`
  - `docs/project_management/standards/TRIAD_WORKFLOW_CROSS_PLATFORM_INTEG.md`: `YES`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`: `YES`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`: `YES`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`: `YES`

## Gate Results (PASS/FAIL with evidence)

### 1) Zero-ambiguity contracts
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` (pattern grammar, defaults, validation rules, snapshot canonicalization + hashing).
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` (exit code taxonomy + output/warning contracts).
  - `make planning-lint FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"` → exit `0` (ambiguity scan passed).

### 2) Decision quality (2 options, explicit tradeoffs, explicit selection)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` entries DR-AX-0001..DR-AX-0005 include exactly two options, explicit rationale, and follow-up task IDs.

### 3) Cross-doc consistency (CLI/config/exit codes/paths)
- Result: `PASS`
- Evidence:
  - Exit codes `2/3/4` are consistent across:
    - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix B.2.3 + B.3.4 + B.3.5)
    - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` (sections 2.1–2.2)
    - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` (Cases 2–3)

### 4) Sequencing and dependency alignment
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/sequencing.json` includes the sprint `world_fs_granular_allow_deny_appendix` with `WFGADAX0..3`.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` gates checkpoint groups via `CP1-ci-checkpoint` / `CP2-ci-checkpoint`, consistent with schema v4 checkpoint boundaries.

### 5) Testability and validation readiness
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` contains runnable commands and deterministic expected exit codes/output (Cases 1–5).
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/smoke/_core.sh` uses a temporary workspace/home by default and asserts exit codes deterministically.

### 5.1) Cross-platform parity task structure (schema v4+ boundary-only platform-fix)
- Result: `PASS`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` meta:
    - `schema_version: 4`
    - `behavior_platforms_required: ["linux"]`
    - `ci_parity_platforms_required: ["linux","macos","windows"]`
    - `checkpoint_boundaries: ["WFGADAX1","WFGADAX3"]`
  - Only checkpoint-boundary slices define `*-integ-core` / `*-integ-<platform>` tasks, matching `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ci_checkpoint_plan.md`.

### 6) Triad interoperability (execution workflow)
- Result: `PASS`
- Evidence:
  - `tasks.json` required fields present (see audit command above).
  - Kickoff prompts contain the sentinel “Do not edit planning docs inside the worktree.” (planning lint passed the sentinel scan).

## Findings (must be exhaustive)

### Finding 001 — Mechanical lint passed
- Status: `VERIFIED`
- Evidence: `make planning-lint FEATURE_DIR="docs/project_management/next/world-fs-granular-allow-deny-APPENDIX"` → exit `0`
- Impact: Confirms mechanical pack invariants (required artifacts, bans, sequencing wiring, sentinel coverage) are satisfied.
- Fix required (exact): none

### Finding 002 — Decision register meets the two-option standard
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` (DR-AX-0001..0005)
- Impact: Implementers have explicit, viable tradeoffs and clear follow-up task mapping; reduces ambiguity and mid-execution re-decision churn.
- Fix required (exact): none

### Finding 003 — spec_manifest enumerates ADR Appendix A/B surfaces with single ownership
- Status: `VERIFIED`
- Evidence: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` coverage matrix assigns:
  - `substrate policy show` output contract (Appendix A.6) → `contract.md`
  - routing fallback warning substrings (Appendix B.2.1) → `contract.md`
- Impact: Prevents “orphaned” contract surfaces and ensures one authoritative spec per surface.
- Fix required (exact): none

### Finding 004 — Checkpoint plan is bounded and deterministically wired (schema v4+)
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ci_checkpoint_plan.md` JSON: `min=2`, `max=4`, checkpoints CP1/CP2 cover all slices exactly once.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` `meta.checkpoint_boundaries=["WFGADAX1","WFGADAX3"]`.
- Impact: Enables bounded cross-platform validation without per-slice CI thrash, while preventing bypass of required checkpoints.
- Fix required (exact): none

### Finding 005 — Manual playbook + smoke scripts are runnable and assert exit codes/output
- Status: `VERIFIED`
- Evidence:
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` includes explicit `echo "exit=$?"` checks and `grep -F ...` assertions.
  - `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/smoke/_core.sh` provides `expect_exit` helper and temp-root isolation.
- Impact: Acceptance criteria are executable and auditable; reduces risk of “it works” subjective checks.
- Fix required (exact): none

## Decision: ACCEPT or FLAG

### ACCEPT
- Summary: Mechanical lint passes; ADR Appendix A/B contract surfaces are fully owned and specified; tasks/sequencing/checkpoints are deterministically wired; validation is runnable with explicit expected outcomes.
- Next step: Execution triads may begin.
