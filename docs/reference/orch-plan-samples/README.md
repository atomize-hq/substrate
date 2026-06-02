# ORCH_PLAN Sample Set

This directory preserves a small representative sample of historical `ORCH_PLAN` documents as stable reference material for code-intelligence workstream/worktree orchestration design.

These are copied here intentionally because the larger `llm-last-mile/ORCH_PLAN-*.md` corpus may not remain available forever.

This sample set is:

- reference material
- operator precedent
- evidence of different orchestration shapes

It is not:

- the canonical contract for future orchestration artifacts
- a field-for-field schema template
- the active source of truth for the current code-intelligence orchestration design

The active design authority is [docs/code-intelligence-workstream-orchestration.md](/Users/spensermcconnell/.codex/worktrees/9b83/substrate/docs/code-intelligence-workstream-orchestration.md).

---

## Selection rules

The files in this sample were chosen to cover materially different orchestration shapes rather than to preserve every historical variation.

The goal is to keep a compact corpus that shows:

- parent-owned critical paths
- real parallel windows
- intentionally serialized runs
- continuation or mid-run revision behavior
- cross-platform or parity-oriented runs
- durable host-session closeout patterns

---

## Included samples

### [ORCH_PLAN-04.md](/Users/spensermcconnell/.codex/worktrees/9b83/substrate/docs/reference/orch-plan-samples/ORCH_PLAN-04.md)

Why it is included:

- early backend-only orchestration shape
- strongly parent-owned critical path
- useful as a baseline for narrow runtime-binding work

### [ORCH_PLAN-13.md](/Users/spensermcconnell/.codex/worktrees/9b83/substrate/docs/reference/orch-plan-samples/ORCH_PLAN-13.md)

Why it is included:

- explicit three-lane parallel execution
- installer-sensitive and platform-posture-aware
- useful for seeing a larger parallel window than the common two-lane shape

### [ORCH_PLAN-17.md](/Users/spensermcconnell/.codex/worktrees/9b83/substrate/docs/reference/orch-plan-samples/ORCH_PLAN-17.md)

Why it is included:

- intentionally serialized execution
- useful counterexample to over-parallelization
- shows how repo-truth narrowing can collapse a previously broader plan into one honest lane

### [ORCH_PLAN-19.md](/Users/spensermcconnell/.codex/worktrees/9b83/substrate/docs/reference/orch-plan-samples/ORCH_PLAN-19.md)

Why it is included:

- mid-run orchestration revision after a blocked attempt
- explicit live-root supersession and accepted-work preservation
- useful for understanding how orchestration artifacts may need to absorb real run-state drift

### [ORCH_PLAN-21.md](/Users/spensermcconnell/.codex/worktrees/9b83/substrate/docs/reference/orch-plan-samples/ORCH_PLAN-21.md)

Why it is included:

- cross-platform parity rollout
- distinct worker branches and validation/doc closeout patterns
- useful for studying orchestration outside the earlier Linux-only emphasis

### [ORCH_PLAN-25.md](/Users/spensermcconnell/.codex/worktrees/9b83/substrate/docs/reference/orch-plan-samples/ORCH_PLAN-25.md)

Why it is included:

- durable host-session closeout and truth-freeze shape
- mixed lane profile: code, docs, and late cleanup
- strong fit with the current host orchestrator truth surfaces

---

## How to use this sample set

Use this directory to identify recurring orchestration invariants across materially different runs.

Examples:

- source-lock or authoritative-baseline concepts
- parent-only integration and gate ownership
- explicit worker ownership boundaries
- blocked conditions and stop rules
- validation-wall placement
- truthful limits on parallelism

Do not assume that repeated wording across these samples automatically implies a frozen product contract.

Repeated patterns are evidence to examine, not names or schemas to copy blindly.
