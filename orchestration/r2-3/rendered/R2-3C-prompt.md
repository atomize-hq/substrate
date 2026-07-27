Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3C. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 50e56a111930e6d2afba6e1f0c5de712b01d58251a0030e738d53743e602eaa1
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3C
- packet_id: A1.1d-5R2-3C
- next_increment: R2-3M1

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
- expected base commit: cd21bc95575b9c35a1802814b847f5d63b86e38f
- expected base tree: a8e9cd52f2b9034896627218306a4c26a2e9bcf8
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3C

Use:

- using-agent-skills
- context-engineering
- source-driven-development
- api-and-interface-design
- test-driven-development
- incremental-implementation
- gitnexus-impact-analysis
- security-and-hardening
- code-review-and-quality
- git-workflow-and-versioning

Every subagent must be a GPT-5.4 subagent with Extra High reasoning and standard/default speed.
Never use fast speed. Ground every decision in current repository truth, official Microsoft API
documentation, and the published runtime-refactor control pack. Do not reopen R2-2, RP3, RP4,
RP5, R2-3A, R2-3B, or their publication without a concrete contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`
- parent A1.1d-5R2-3 row, Windows principal/installed-witness requirements, common subdivision
  rules, and R2-3C section in `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- PI-042–PI-046, PI-048–PI-049, PI-052, PI-068–PI-070, PI-076, PI-079, PI-081, and PI-090
  current-state and required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
- the host-context construction, Windows principal, installed-product self-derivation,
  target-Windows observation, and common bounded-review contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- R2-SHIM-01, R2-DIAG-01, and R2-MAP-WIN-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication

SELECTED OUTCOME

Provide only the target-Windows shell observation primitives needed to bind the current
account+SID, token-bound LocalApplicationData Known Folder, and the installed physical
`A\bin\substrate.exe` invocation witness.

Do not connect these primitives to routing, installers, uninstallers, world backends, WSL,
forwarders, shims, diagnostics, or any other runtime consumer in this increment.

EXACT COMPLETION CLAIM

R2-3C completes only the target-Windows principal, Known Folder, and installed-release-copy
witness observation prerequisite. It supports but completes no PI row and claims no native
Windows mapping proof.

EXACT ALLOWLIST

Only:

- `crates/shell/Cargo.toml`
- `crates/shell/src/execution/install_bootstrap.rs`

Tests may be added only as colocated target-Windows tests in
`crates/shell/src/execution/install_bootstrap.rs`.

Do not edit:

- `Cargo.lock`
- any other manifest
- any routing, invocation-plan, installer, uninstaller, shim, world, WSL, forwarder, diagnostic,
  replay, trace, or platform file
- any PowerShell or shell script
- transport-api-types or its existing model
- control-pack documents
- generated files
- `AGENTS.md` or `CLAUDE.md`

If another file is required, stop with `BLOCKED_SCOPE_EXPANSION`. A routing-file edit, new
dependency/version, lockfile hunk, or runtime-consumer edit stops C.

PI POSTURE

R2-3C supports but completes none of:

- PI-042–PI-046
- PI-048–PI-049
- PI-052
- PI-068–PI-070
- PI-076
- PI-079
- PI-081
- PI-090

R2-3A remains the shared carrier/mapping model prerequisite. R2-3B remains the exact dependency
edge prerequisite. Installer propagation, WSL mapping, forwarder state, diagnostics, factory
consumption, physical-shim observation, and native Windows evidence remain owned by later
increments.

EXACT NEW SYMBOLS

Add exactly these target-Windows counterparts in
`crates/shell/src/execution/install_bootstrap.rs`:

- `construct_windows_install_bootstrap_context`
- `decode_and_bind_windows_install_bootstrap_context`
- `bind_windows_install_bootstrap_context`
- `current_windows_principal_and_known_folder`
- `windows_known_folder_for_principal`
- `resolve_windows_install_prefix_from_invocation`
- `require_same_windows_file_identity`

Do not add another public or crate-visible symbol. Private helpers may exist only when directly
necessary to call the authorized Windows APIs, perform canonical conversion, validate a witness,
or keep cfg-specific code isolated. Do not create a local principal/carrier type, authority cache,
side table, registry, mutable global, observer service, or alternate path normalizer.

UNIX REGRESSION BOUNDARY

Preserve every existing Unix function, caller contract, test name, and behavior. The file currently
has a Unix file-level cfg; restructure cfg placement only as narrowly as necessary to compile the
new Windows counterparts from the same allowed file.

Before editing, obtain file-disambiguated upstream GitNexus impact with tests included for:

- `construct_unix_install_bootstrap_context`
- `decode_and_bind_unix_install_bootstrap_context`
- `bind_unix_install_bootstrap_context`
- `current_unix_principal_and_home`
- `resolve_unix_install_prefix_from_invocation`
- `require_same_file_identity`

Also obtain context for the shared projection helpers before changing any of them. Record direct
callers, affected processes/modules, and risk. Warn on HIGH or CRITICAL. Continue only when the
change remains confined to cfg organization and the exact Windows counterparts. An unexpected
Unix behavior change, routing dependency, authority root, or execution-flow change is a stop.

WINDOWS PRINCIPAL CONTRACT

Use only the current process token and `windows-sys` APIs already authorized by the parent.

- resolve the current canonical Windows logon account and canonical `S-1-...` SID
- reject anonymous, missing, malformed, or untranslatable identity
- bind an incoming carrier only when its `PlatformPrincipalV1::Windows { account, sid }` exactly
  equals the current token identity
- do not accept a Unix principal on Windows
- do not install or observe on behalf of a different principal
- do not read `USERNAME`, `USERDOMAIN`, `USERPROFILE`, `HOME`, `LOCALAPPDATA`, or any ambient
  account/folder variable as authority
- do not use raw local FFI; call only bindings exposed by the frozen `windows-sys 0.52` surface

The functions observe and validate identity only. They do not persist it, mutate process
environment, write files, create directories, or dispatch a consumer.

TOKEN-BOUND KNOWN FOLDER CONTRACT

Resolve `FOLDERID_LocalAppData` for the same verified current Windows token through
`SHGetKnownFolderPath`, release the returned buffer through `CoTaskMemFree`, convert it
canonically, and normalize it through the existing Windows install-bootstrap path rules.

When no prefix is declared, construct the default as exactly:

```text
<token LocalApplicationData>\Substrate
```

Never use `LOCALAPPDATA`, `USERPROFILE`, another environment value, a repository path, CWD, or an
executable parent as the default. `windows_known_folder_for_principal` must require the supplied
Windows principal to match the current token before returning the Known Folder.

WINDOWS INSTALLED-WITNESS CONTRACT

Use the same authority shape as the frozen product contract:

- command name is exactly `substrate` or `substrate.exe` as allowed by the canonical invocation
  form, and the accepted witness is the physical Windows file
  `A\bin\substrate.exe`
- an explicit absolute invocation, or an explicit relative invocation containing a separator,
  resolves only that spelling; CWD is used only to make that explicit relative spelling absolute
- a bare invocation enumerates only absolute, nonempty PATH entries and collects every matching
  installed witness
- PATH order never selects a prefix; exactly one valid candidate is required
- zero or multiple valid candidates fail closed
- a repository binary, arbitrary executable-parent shape, version payload copy without the public
  `A\bin` witness, wrong command name, or different physical file fails
- validate the witness without following a reparse/symlink alias and require exact Windows file
  identity with the running executable for the operation
- use volume serial number plus file ID from held handles; case, short-name, junction, mount,
  reparse, or alternate spelling does not become a second commitment
- normalize the recovered A through the existing
  `normalize_windows_install_bootstrap_path`; do not create another normalizer

The release payload at `A\versions\<version>\bin\substrate.exe` is a projection and need not have
the same file identity. Only the physical public `A\bin\substrate.exe` witness selects A in this
increment.

CARRIER CONSTRUCTION AND BINDING

- a declared prefix and the Known-Folder default use the same existing Windows normalizer
- construct the existing `InstallBootstrapContextV1::new_windows` model and canonical carrier;
  do not alter transport-api-types
- decoding requires canonical carrier validation, current-token account/SID equality, and equality
  with any declared prefix
- inherited H/R/carrier/commitment/principal projections are consistency checks only where the
  existing shared helper contract already defines them; they cannot select the prefix or principal
- malformed, tampered, forged-principal, noncanonical, or conflicting input fails before any
  mutation

Do not add an installer action, hidden route, argument parser, environment projection writer,
backend construction, mapping construction, or diagnostic output.

EXACT MANIFEST CHANGE

In the existing target-Windows `windows-sys = "0.52"` dependency in
`crates/shell/Cargo.toml`, preserve:

- `Win32_Foundation`
- `Win32_System_Console`

and add exactly:

- `Win32_Security`
- `Win32_Storage_FileSystem`
- `Win32_System_Threading`
- `Win32_System_Com`
- `Win32_UI_Shell`

Do not change dependency version, dependency kind, target section, or any other feature or
dependency. This is a feature-only edit and must not change `Cargo.lock` or the package graph.

TEST-DRIVEN IMPLEMENTATION

Add focused colocated target-Windows tests covering:

- canonical current account/SID observation and binding
- token-bound Known Folder and exact default `\Substrate` suffix
- declared-prefix normalization and carrier round trip
- physical release-copy public witness self-derivation under conflicting ambient B
- explicit absolute and explicit-relative invocation handling
- bare PATH enumeration with exactly one witness
- zero and multiple candidate rejection
- no-follow exact file-identity acceptance and mismatch/reparse rejection
- forged Windows principal rejection
- malformed/tampered carrier rejection
- conflicting declared prefix and existing checked projection rejection
- proof that ambient `LOCALAPPDATA`, `USERPROFILE`, `HOME`, PATH order, and executable-parent
  spelling cannot select A

Keep tests deterministic and non-destructive. Use temporary directories/files only. Do not add a
native evidence claim: native supported-Windows execution remains assigned to R2-3Z.

REQUIRED VERIFICATION

Run:

- formatting for the touched Rust file/workspace
- `cargo check --locked -p shell`
- focused existing Unix install-bootstrap tests on the current host
- `cargo test --locked -p shell install_bootstrap`
- `cargo clippy --locked -p shell --all-targets -- -D warnings`
- static target-Windows `cargo check`/test compilation for an already-installed supported Windows
  Rust target where available; record exact target/tool availability and do not install or simulate
  a missing native platform
- `git diff --check`
- exact two-file allowlist/status check
- confirmation that `Cargo.lock` and every other manifest are unchanged
- semantic manifest comparison proving only the five exact `windows-sys 0.52` features were added
- confirmation that no routing, installer, world, WSL, forwarder, shim, diagnostic, replay, trace,
  platform, or control-pack file changed
- source-level and test proof that every Unix function/test remains present and behavior-equivalent
- `gitnexus_detect_changes()` before commit

Inspect every affected flow. Only the exact install-bootstrap observation surface is expected.
Any unrelated production flow or Windows consumer activation fails the increment.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. Record the pre-edit base commit.
2. Build a sorted manifest containing the pre-edit commit plus, for each exact subject path, its
   repository-relative path, Git mode, and `git hash-object --no-filters` blob ID.
3. SHA-256 the manifest.
4. Use `sha256:<digest>` as the review subject fingerprint.

The subject contains only:

- `crates/shell/Cargo.toml`
- `crates/shell/src/execution/install_bootstrap.rs`

BOUNDED REVIEW

Open a new validated V1 record with packet ID `A1.1d-5R2-3C`. Keep process-only evidence outside
the tracked checkout.

Required fresh read-only lenses:

1. current-token account/SID and token-bound Known Folder authority; no ambient fallback
2. installed-witness/no-follow Windows file identity, PATH ambiguity, and fail-closed behavior
3. Unix regression, exact API/manifest/two-file allowlist, tests, and no premature consumer work

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated/inventoried before completion. If that requires expanding the C allowlist, stop with
`AUTHORITY_REQUIRED` for a separate inventory-only action. CLEAN is terminal.

PUBLICATION

After and only after every gate passes and the final review is CLEAN:

1. Stage only the exact two allowlisted paths.
2. Inspect the staged diff and rerun staged allowlist, diff, manifest, lockfile, and Unix-regression
   checks.
3. Commit one atomic Conventional Commit of at most 72 characters.
4. Fetch/query the live product target and require it still equals the expected base.
5. Perform a normal fast-forward push of HEAD to the product target ref.
6. Fetch/query live remote and verify commit equality.
7. Refresh GitNexus to the published commit.
8. Restore analyzer-only `AGENTS.md`/`CLAUDE.md` count changes to committed bytes if necessary.
9. Finish clean, 0 ahead/0 behind, with the live product ref at the landed commit.

Do not create another product branch, merge, rebase, force-push, open a PR, or begin R2-3M1.

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

Do not generate the next increment prompt and do not begin R2-3M1. The meta
orchestrator owns independent verification and subsequent dispatch.
