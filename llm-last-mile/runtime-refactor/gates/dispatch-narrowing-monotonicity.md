**Kind:** gate
**Status:** canonical
**Canonical for:** complete extracted resolver formulas, V1 field rules table, path-containment rules 1–8, host-visible fail-closed exception, rejected-not-ignored narrowing rule, broker versus world-service enforcement boundary, and the bounded E1 terminal disposition
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules`](../04-contracts-and-gates.md#13-dispatch-narrowing-monotonicity-rules), baseline lines 181–214; the exact 1930-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `b12ef2b8c8bcffad5139e196df49ba2f59094ddf024f59d2d2a645eb9be2210a`

## E1 terminal implementation disposition

E1's request-scoped `current_parent AND dispatch_patch` primitive and all V1 field/path rows below
are terminally closed at exact closure commit
`18f719898ce2a48f65e95b3b23f3b2cfd685c4af`, tree
`60e94c7b5c985463d3311b8f556520dea8244287`, over implementation commit
`6194788d45267d91b4428a42e24c02dfcaae3c1e`, implementation tree
`6be705b4e957c071c05ec3a97fc973c05fa1f302`, and reviewed fingerprint
`sha256:35b77c26e04b55b9f355268a5f28be1ff1d7cfeb9aef6b6751894cda95ab3dac`.
The Linux ABI-7 real-service proof covers exact-file success, sibling/outside denial, and ancestor
symlink-escape denial without changing the parent snapshot; focused Landlock coverage separately
proves final-component symlink rejection. The `worker_cap`, future `turn`, and `fork_cap` formulas
remain E2-owned and unimplemented under the independent
[`DispatchPolicyCommitmentV1`](../contracts/dispatch-policy-commitment-v1.md). E2 owns neither
receipt nor full-manifest construction. D2 per-operation mediation and E4 host-visibility
synchronization remain open. This bounded disposition completes E1 and makes E2 eligible only for
another fresh admission and explicit dispatch; it does not admit or dispatch E2.

For the future E2 formulas, current parent is resolved independently at Continue/Fork time and the
immutable worker/source-worker cap is authenticated from launch/fork truth. The retained-target
resolver must not require launch policy/ref/revision to equal current parent. All non-policy target,
admission, backend, world, ancestry, lifecycle, and routing checks remain. Current-parent narrowing
may further restrict future work; broadening remains bounded by the immutable cap. Missing
verifiable cap bytes/ref/hash returns typed `UnsupportedLegacyState` before dispatch.

<!-- exact-extracted-body:start -->
## 13. Dispatch narrowing monotonicity rules

The resolver computes:

```text
ephemeral = current_parent AND dispatch_patch
worker_cap = current_parent_at_spawn AND spawn_patch
turn = current_parent_now AND worker_cap AND turn_patch
fork_cap = current_parent_now AND source_worker_cap AND fork_patch
```

V1 field rules:

| Field | Allowed narrowing | Rejected broadening |
|---|---|---|
| `world_fs.host_visible` | `true -> false` | `false -> true` |
| `world_fs.fail_closed.routing` | `false -> true` | `true -> false` |
| `world_fs.caged_required` | `false -> true` | `true -> false` |
| `world_fs.write.enabled` | `true -> false` | `false -> true` |
| `world_fs.deny_enforcement` | same or stronger rank | weaker rank or removal |
| discover/read/write `allow_list` | each requested path is contained by at least one parent path | root/directory/file widening |
| discover/read/write `deny_list` | add denials or retain parent denials | remove parent denial |

Path-containment rules:

1. Normalize relative to the authoritative world/project root.
2. `.` contains `src` and `src/parser.rs`; `src` contains `src/parser.rs`; a file contains only itself.
3. Reject absolute host paths, `..` escape, unsupported glob semantics, NUL, symlink escape, and paths outside the root.
4. Compare canonical policy paths without requiring the target file to already exist; runtime resolution must re-check symlink/ancestor escape at enforcement time.
5. `host_visible=true` with deny-list or unprovable isolation semantics fails closed unless the patch legally narrows to supported full isolation.
6. A patch supplied while `agents.world_dispatch.allow_capability_narrowing=false` is rejected, not ignored.
7. Narrowing may not enable a dispatch action, backend, mode, capability, network route, or side-effect channel forbidden by the parent.
8. Adapter config may receive policy hints, but only broker/world-service enforcement counts.

<!-- exact-extracted-body:end -->
