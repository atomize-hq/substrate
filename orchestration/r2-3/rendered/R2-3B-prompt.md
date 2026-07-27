Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3B. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 1114957dde7f5a5ee57802f61396ad15f91f7be2adb8579b07cb37a1849f86c4
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3B
- packet_id: A1.1d-5R2-3B
- next_increment: R2-3C

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
- expected base commit: f2797fa84168c173e05d466e05dec8b7e85af92a
- expected base tree: 11fe4da11d67461d17e1af1534e0617b0fcc5c5a
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3B

Use:

- using-agent-skills
- context-engineering
- source-driven-development
- incremental-implementation
- gitnexus-exploring
- code-review-and-quality
- git-workflow-and-versioning

Every subagent must be a GPT-5.4 subagent with Extra High reasoning and standard/default speed.
Never use fast speed. Ground every decision in current repository truth and the published
runtime-refactor control pack. Do not reopen R2-2, RP3, RP4, RP5, their publication, R2-3A, or
the post-publication fixes without a concrete contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`
- parent A1.1d-5R2-3 row, closed R2-3 dependency surface, common subdivision rules, and R2-3B
  section in `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- PI-054, PI-060, PI-090, PI-115, and PI-116 current-state and required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
- the common bounded-review contract in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- R2-SHIM-01, R2-MAP-MAC-01, and R2-MAP-WIN-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication

SELECTED OUTCOME

Add only the closed dependency and target-feature edges required by later typed R2-3 consumers.
This increment changes no Rust source and no runtime behavior.

EXACT COMPLETION CLAIM

R2-3B completes only the manifest and lockfile dependency-edge prerequisite for later consumers.
It completes no PI row, adds no runtime behavior, and claims no native platform evidence.

EXACT ALLOWLIST

Only:

- `crates/world-backend-factory/Cargo.toml`
- `crates/forwarder/Cargo.toml`
- `crates/shim/Cargo.toml`
- `Cargo.lock`

Do not edit:

- any Rust source or test
- any other manifest
- the shell manifest
- control-pack documents
- installers or uninstallers
- generated files
- `AGENTS.md` or `CLAUDE.md`

If another file is required, stop with `BLOCKED_SCOPE_EXPANSION`. Any shell manifest change or
unexpected lock hunk stops this increment.

PI POSTURE

R2-3B supports but completes none of:

- PI-054
- PI-060
- PI-090
- PI-115
- PI-116

R2-3A remains the already-landed shared wire-model prerequisite. All consumer implementation,
platform observation, transport behavior, lifecycle behavior, and native evidence remain owned by
later increments.

PRE-EDIT SOURCE CLOSURE

This is manifest-only, so no symbol impact call is required and no Rust symbol may be edited.
Before asserting that each dependency is necessary, use read-only GitNexus/source inspection to
close these exact later-consumer paths:

- `world-backend-factory/src/lib.rs::factory`
- `forwarder/src/pipe.rs::PipeListener::new`
- `forwarder/src/bridge.rs::spawn_bridge`
- `shim/src/context.rs::ShimContext::from_current_exe`
- `shim/src/exec/logging.rs::ManagerHintEngine::new`

Record why the shared `transport-api-types` edge is required by each owning crate. Record why the
shim alone needs the frozen Unix and Windows observation features. Do not edit or implement any
consumer. If inspection reveals a different dependency owner, local duplicate type, alternate
authority root, or additional feature is needed, stop with `BLOCKED_CONTRADICTION`.

EXACT MANIFEST CHANGES

Add exactly this ordinary dependency to each of the three named crate manifests:

```toml
transport-api-types = { version = "0.2.8", path = "../transport-api-types" }
```

The three owners are:

- `crates/world-backend-factory/Cargo.toml`
- `crates/forwarder/Cargo.toml`
- `crates/shim/Cargo.toml`

In `crates/shim/Cargo.toml`, change the existing Unix dependency from:

```toml
nix = { version = "0.29", features = ["fs", "process"] }
```

to exactly:

```toml
nix = { version = "0.29", features = ["fs", "process", "user"] }
```

Also add exactly this target-Windows dependency section to `crates/shim/Cargo.toml`:

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_Security",
    "Win32_Storage_FileSystem",
    "Win32_System_Threading",
    "Win32_System_Com",
    "Win32_UI_Shell",
] }
```

The final two features are reserved for later token-bound `FOLDERID_LocalAppData` observation and
freeing its returned buffer. Do not add raw FFI, another Windows crate/version, another `nix`
feature, a shim-local principal type, or any runtime use in this increment.

EXACT LOCKFILE CONTRACT

Regenerate or update `Cargo.lock` only through locked dependency resolution consistent with the
three manifest edits. The only authorized lockfile semantic changes are:

- add `transport-api-types` to the existing `world-backend-factory` dependency list
- add `transport-api-types` to the existing `substrate-forwarder` dependency list
- add `transport-api-types` and `windows-sys 0.52.0` to the existing `substrate-shim`
  dependency list

No package stanza, package count, package version, source, checksum, target, or any other package
dependency list may change. The `nix` feature-only change creates no lock hunk. The existing
`windows-sys 0.52.0` package is reused; no new package/version/checksum is authorized.

Do not hand-edit around an unexplained Cargo resolution result. If Cargo produces any other lock
hunk, stop with `BLOCKED_CONTRADICTION`.

NO PRODUCT TEST OR GATE

There is no test file and no new product gate in this increment. Do not add one. Verification is
the parent dependency-closure gate: locked metadata/build resolution plus exact manifest and
lockfile semantic comparison.

REQUIRED VERIFICATION

Run:

- `cargo metadata --locked --format-version 1`
- `cargo check --locked -p world-backend-factory`
- `cargo check --locked -p substrate-forwarder`
- `cargo check --locked -p substrate-shim`
- `cargo fmt --all -- --check`
- `git diff --check`
- exact four-file allowlist and status checks
- semantic manifest comparison proving only the three shared edges, shim `nix` feature `user`, and
  the exact shim target-Windows `windows-sys 0.52` feature set changed
- semantic lock comparison proving only the three named dependency lists changed exactly as
  authorized
- proof that package count, names, versions, sources, checksums, and every other package dependency
  list are unchanged
- proof that no Rust source, test, shell manifest, runtime/platform file, or control document changed
- `gitnexus_detect_changes()` before commit

Inspect every detected path and flow. Because this increment is manifest-only, an affected
execution flow, Rust symbol edit, package-set change, or runtime behavior change is unexpected and
fails the increment.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. Record the pre-edit base commit.
2. Build a sorted manifest containing the pre-edit commit plus, for each of the four subject paths,
   its repository-relative path, Git mode, and `git hash-object --no-filters` blob ID.
3. SHA-256 the manifest.
4. Use `sha256:<digest>` as the review subject fingerprint.

The subject contains only the exact four allowlisted paths.

BOUNDED REVIEW

Open a new validated V1 record with packet ID `A1.1d-5R2-3B`. Keep process-only evidence outside
the tracked checkout.

Required fresh read-only lenses:

1. exact dependency ownership and manifest/target-feature closure
2. lockfile semantic exactness, package-set stability, and locked resolution
3. absence of Rust/runtime behavior, scope expansion, or premature later-increment work

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated/inventoried before completion. If that requires expanding the B allowlist, stop with
`AUTHORITY_REQUIRED` for a separate inventory-only action. CLEAN is terminal.

PUBLICATION

After and only after every gate passes and the final review is CLEAN:

1. Stage only the exact four allowlisted paths.
2. Inspect the staged diff and rerun staged allowlist, diff, manifest, and lock semantic checks.
3. Commit one atomic Conventional Commit of at most 72 characters.
4. Fetch/query the live product target and require it still equals the expected base.
5. Perform a normal fast-forward push of HEAD to the product target ref.
6. Fetch/query live remote and verify commit equality.
7. Refresh GitNexus to the published commit.
8. Restore analyzer-only `AGENTS.md`/`CLAUDE.md` count changes to committed bytes if necessary.
9. Finish clean, 0 ahead/0 behind, with the live product ref at the landed commit.

Do not create another product branch, merge, rebase, force-push, open a PR, or begin R2-3C.

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

Do not generate the next increment prompt and do not begin R2-3C. The meta
orchestrator owns independent verification and subsequent dispatch.
