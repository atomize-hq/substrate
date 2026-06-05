Status: synthesis
Scope: exec
Authority: non-canonical
Artifact-boundary impact: possible

# Exec Implications

This note records the strongest runtime-specific implications from the current research set.

## Immediate implications

1. `exec` should treat plan artifacts as immutable runtime intent.
2. Blocked conditions should be explicit runtime artifacts with preserved evidence.
3. Runtime should shift from optimistic to fail-closed once concrete instability is observed.
4. Checkpoints should cluster at source lock, materialization, merge, and validation transitions.
5. Speculative merge/build/test validation is a strong later addition, but should not block the first materializer MVP.

## MVP design bias

Preferred early runtime order:

1. verify source lock
2. materialize worktrees and branches
3. write durable runtime truth
4. checkpoint materialization state
5. preserve blocked conditions and gate evidence
6. close only through merged-tree validation

## Promotion caution

This note supports stronger blocked-state and runtime-evidence semantics.
It does not by itself settle the final top-level runtime artifact taxonomy.
