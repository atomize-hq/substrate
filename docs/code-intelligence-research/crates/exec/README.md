Status: synthesis
Scope: exec
Authority: non-canonical
Artifact-boundary impact: none

# Exec Research Notes

This directory keeps runtime-specific implications without letting them silently redefine broader Substrate runtime models.

## Governing authority docs

- [code-intelligence-program.md](../../../code-intelligence-program.md)
- [code-intelligence-workstream-orchestration.md](../../../code-intelligence-workstream-orchestration.md)
- [code-intelligence-contracts-and-gates.md](../../../code-intelligence-contracts-and-gates.md)

## Most relevant overlay notes

- [runtime-validation-and-fail-closed.md](../../overlays/effort-exec/runtime-validation-and-fail-closed.md)
- [parallel-planning-and-conflict.md](../../overlays/effort-exec/parallel-planning-and-conflict.md)
- [architecture-synthesis.md](../../overlays/effort-exec/architecture-synthesis.md)

## Local notes

- [implications.md](./implications.md)
- [open-questions.md](./open-questions.md)

## Scope boundary

This directory is for:

- source-lock enforcement
- materialization state
- blocked-condition recording
- checkpoint placement
- validation-wall implications

It is not for:

- lane planning semantics
- promoting broader runtime convergence by implication alone
- freezing the exact `ExecutionStateV1` family shape early
