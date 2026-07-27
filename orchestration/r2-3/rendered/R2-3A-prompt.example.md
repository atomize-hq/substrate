Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3A. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 00000000000000000000000000000000
- meta_thread_id: BOUND_AT_META_BOOTSTRAP
- meta_host_id: BOUND_AT_META_BOOTSTRAP
- increment: R2-3A
- packet_id: A1.1d-5R2-3A
- next_increment: R2-3B

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the task-assigned Codex worktree

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: c0d9cd4f5c60d6505429de75a2fa5a52887b9879
- expected base tree: e715b4ce5fe6c227542e646fe53416438af44525
- required ancestor: 0f1e147fb735791b44a65099a65167cbdc1803af

Before editing, fetch/query the live remote and verify the exact base, tree, ancestry, cleanliness,
0 ahead/0 behind, and current GitNexus index. If they differ, send BASE_DRIFT or
BLOCKED_CONTRADICTION. Do not reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3A

Use:

- using-agent-skills
- context-engineering
- api-and-interface-design
- test-driven-development
- incremental-implementation
- gitnexus-impact-analysis
- code-review-and-quality
- git-workflow-and-versioning

Ground every decision in current repository truth and the published runtime-refactor control pack.
Do not reopen R2-2, RP3, RP4, RP5, their publication, or the four post-publication fixes without a
concrete contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`
- parent A1.1d-5R2-3 row, common subdivision rules, and R2-3A section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- named R2-3A PI rows from `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
- bounded-review and `PlatformBootstrapMappingV1` construction/verification contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- R2-MAP-MAC-01 and R2-MAP-WIN-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication

SELECTED OUTCOME

Add only the shared canonical `PlatformBootstrapMappingV1` wire/value model and its strict
validation. Do not add a runtime consumer, platform observation, dependency edge, installer
propagation, backend behavior, forwarding behavior, or lifecycle action.

EXACT COMPLETION CLAIM

R2-3A completes only the shared PM wire-model prerequisite. It completes no consumer PI row and
claims no native macOS or Windows mapping proof.

EXACT ALLOWLIST

Only:

`crates/transport-api-types/src/lib.rs`

Tests may be added only to the colocated `#[cfg(test)]` module in that file.

Do not edit:

- any Cargo manifest or `Cargo.lock`
- any other Rust source
- control-pack documents
- installers or uninstallers
- platform, Lima, WSL, forwarder, shim, shell, replay, trace, or factory code
- generated files
- `AGENTS.md` or `CLAUDE.md`

If another file is required, stop with `BLOCKED_SCOPE_EXPANSION`.

PI POSTURE

R2-3A supports but completes none of:

- PI-039–PI-041
- PI-048–PI-049
- PI-052
- PI-054–PI-060
- PI-075–PI-076
- PI-079
- PI-081
- PI-090–PI-091
- PI-112
- PI-115

PI-059 remains harness-only. PI-077 and PI-078 remain byte-frozen. PI-050 remains R2-4-owned.
PI-080 remains satisfied by R2-2. Every R3 row remains excluded.

EXACT NEW PUBLIC SYMBOLS

Add exactly:

- `PlatformInstanceIdentityV1`
- `PlatformTransportIdentityV1`
- `PlatformBootstrapMappingV1`
- `WindowsForwarderScopeV1`
- `normalize_windows_pipe_path`

Add exactly these inherent operations:

`PlatformBootstrapMappingV1`:

- `new_lima`
- `new_wsl`
- `validate`
- `encode`
- `decode`

`WindowsForwarderScopeV1`:

- `derive`

No other public symbol is authorized. Private helpers may exist only when directly necessary for
the frozen encoding and validation rules. Do not create a second authority model, carrier
hierarchy, mapping registry, global cache, side table, observer, command runner, or platform
adapter.

EXISTING BEHAVIOR AND IMPACT

Existing `InstallBootstrapContextV1`, `InstallBootstrapContextCarrierV1`,
`PlatformPrincipalV1`, path normalization, encoding, and commitment behavior must remain
unchanged.

Before editing an existing symbol, run upstream GitNexus impact analysis with file disambiguation
and tests included. At minimum obtain context for:

- `InstallBootstrapContextCarrierV1`
- `InstallBootstrapContextV1`
- `PlatformPrincipalV1`

If modifying or reusing through edits, impact:

- `normalize_windows_install_bootstrap_path`
- `encode_inner_field`
- `decode_inner_field`
- `record_value`
- `require_record_value`
- `is_lower_hex_digest`

Report direct callers, affected processes/modules, and risk before editing. New symbols record
`N/A (new)`. Warn for HIGH or CRITICAL. Continue only when confined to the expected
`transport-api-types` contract surface. Stop for an unexpected authority root, runtime flow, or
cross-module behavior change.

WIRE CONTRACT

`PlatformBootstrapMappingV1` encodes exactly thirteen LF-terminated lines in this order:

```text
domain=substrate.platform_bootstrap_mapping
version=1
host_context_commitment=<64-lowercase-hex>
platform_kind=<lima-or-wsl>
instance_name=<B64(exact vm name or registered distro spelling)>
guest_machine_id=<32-lowercase-hex>
host_platform_control_root=<B64(normalized host absolute path)>
realized_substrate_home=<B64(normalized guest absolute path)>
realized_principal_account=<B64(guest account)>
realized_principal_uid=<U32 guest uid>
transport_kind=<same lima-or-wsl>
transport_host=<B64(normalized host socket or normalized pipe)>
transport_guest_socket=<B64(normalized guest socket)>
```

Requirements:

- exactly thirteen lines including the final LF
- outer representation uses existing-model unpadded base64url
- reject unknown, missing, duplicate, reordered, malformed, or noncanonical fields
- decoding re-encodes canonically
- platform and transport kinds match
- host commitment equals the recomputed supplied IH carrier commitment
- no second host-prefix/home selector
- no host/guest path, account, or UID equality assertion
- distinct typed Lima and WSL instance variants
- distinct typed Lima and WSL transport variants
- Unix guest account and UID
- machine ID exactly 32 lowercase hexadecimal characters
- commitments/digests exactly 64 lowercase hexadecimal characters
- constructors accept already-observed values and perform no OS, filesystem, environment, Lima,
  WSL, account-database, or command observation

WINDOWS PIPE CONTRACT

Canonical form:

```text
\\.\pipe\<name>
```

The name is 1–128 ASCII characters from `[A-Za-z0-9._-]` and is lowercased. Reject slash variants,
nested names, whitespace, controls, empty names, and all other spellings.

WINDOWS FORWARDER SCOPE

`WindowsForwarderScopeV1` derives SHA-256 over exactly six LF-terminated ASCII lines:

```text
domain=substrate.windows_forwarder_scope
version=1
windows_sid=<B64(canonical SID)>
distro_name=<B64(exact registered distro spelling)>
guest_machine_id=<32-lowercase-hex>
pipe_path=<B64(normalized pipe path)>
```

The result is exactly 64 lowercase hexadecimal characters. It is only a deterministic scope
component, not an install commitment, ownership manifest, deletion/PID/termination authority, or
lifecycle/convergence authority.

TEST-DRIVEN IMPLEMENTATION

Write focused tests first with exactly these names:

- `platform_bootstrap_mapping_lima_golden_vector_is_exact`
- `platform_bootstrap_mapping_wsl_golden_vector_is_exact`
- `platform_bootstrap_mapping_rejects_noncanonical_or_tampered_records`
- `windows_forwarder_scope_and_pipe_normalization_are_canonical`

Cover:

- exact Lima and WSL golden bytes
- exact thirteen-line order and terminal LF
- canonical encode/decode round trip
- host commitment recomputation and mismatch
- mismatched platform/transport kind
- unknown, missing, duplicate, and reordered fields
- malformed and padded/noncanonical base64url
- malformed or uppercase digests/machine IDs
- malformed UID
- noncanonical paths
- host/guest independence
- valid and invalid pipe spellings and case aliasing
- exact six-line scope input/digest
- tampered scope inputs

Public APIs require rustdoc with compiling examples. Implement the smallest design that satisfies
the tests; do not generalize for later consumers.

REQUIRED VERIFICATION

Run:

- formatting for the touched Rust file/workspace
- `cargo check -p transport-api-types`
- `cargo test -p transport-api-types`
- `cargo test -p transport-api-types --doc`
- `cargo clippy -p transport-api-types --all-targets -- -D warnings`
- `git diff --check`
- exact one-file allowlist/status check
- confirmation that `Cargo.lock` and every manifest are unchanged
- confirmation that no runtime/platform file changed
- `gitnexus_detect_changes()` before commit

Inspect every affected flow. Any unexpected production flow fails the increment.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. Record the pre-edit base commit.
2. Build a sorted manifest containing the pre-edit commit plus, for the one subject path, its
   repository-relative path, Git mode, and `git hash-object --no-filters` blob ID.
3. SHA-256 the manifest.
4. Use `sha256:<digest>` as the review subject fingerprint.

The subject contains only `crates/transport-api-types/src/lib.rs`.

BOUNDED REVIEW

Open a new validated V1 record with packet ID `A1.1d-5R2-3A`. Keep process-only evidence outside
the tracked checkout.

Required fresh read-only lenses:

1. canonical framing, validation, and tamper rejection
2. host-authority/mapping separation and absence of lifecycle authority
3. one-file allowlist, public API discipline, and regression sufficiency

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated/inventoried before completion. If that requires expanding the A file allowlist, stop
with `AUTHORITY_REQUIRED` for a separate inventory-only action. CLEAN is terminal.

PUBLICATION

After and only after every gate passes and the final review is CLEAN:

1. Stage only `crates/transport-api-types/src/lib.rs`.
2. Inspect the staged diff and rerun staged allowlist/diff checks.
3. Commit one atomic Conventional Commit of at most 72 characters.
4. Perform a normal fast-forward push of HEAD to the product target ref.
5. Fetch/query live remote and verify commit equality.
6. Refresh GitNexus to the published commit.
7. Restore analyzer-only `AGENTS.md`/`CLAUDE.md` count changes to committed bytes if necessary.
8. Finish clean and with the live product ref at the landed commit.

Do not create another product branch, merge, rebase, force-push, open a PR, or begin R2-3B.

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run `gitnexus_detect_changes()` and inspect every affected flow.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

When authorized by the increment contract, create one atomic commit and perform a normal
fast-forward push of `HEAD` to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live remote equals the
landed commit, refresh GitNexus, restore analyzer-only generated count changes if necessary, and
finish clean.

Send a `codex.top-level-task-receipt.v1` message to the meta task. For success, use
`LANDED_CLEAN` and include your bound increment-task thread/host IDs, expected base, landed
commit/tree, changed paths, subject fingerprint, validated review record and digest, finding
disposition, checks, GitNexus result, clean status, and next increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt.

The `send_message_to_thread` call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin R2-3B. The meta
orchestrator owns independent verification and subsequent dispatch.
