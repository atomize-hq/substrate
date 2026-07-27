Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3M1. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 73258995b6becd2f85335d3684762b424e304291958b722479de64ad42c7e02b
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3M1
- packet_id: A1.1d-5R2-3M1
- next_increment: R2-3M2

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
- expected base commit: 3264e21bbb3c0acc5f712af93bdc3d229bb8b9ad
- expected base tree: 80734514867ec0390ba4249878ed6cc30d597c72
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3M1

Use:

- using-agent-skills
- context-engineering
- source-driven-development
- test-driven-development
- incremental-implementation
- gitnexus-impact-analysis
- security-and-hardening
- code-review-and-quality
- git-workflow-and-versioning

This top-level task and every subagent must run on Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning. Ground decisions in current repository truth,
the published runtime-refactor control pack, and official Lima/macOS/POSIX documentation where
external behavior matters. Do not reopen R2-2, R2-3A, R2-3B, or R2-3C without a concrete
contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3M1 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- PI-039, PI-040, PI-041, PI-075, and PI-081 current-state and required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
- the host-context boundary-carrier, platform-mapping construction/verification, Lima two-stage,
  generated-projection/diagnostic, and common bounded-review contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- R2-MAP-MAC-01 and R2-DIAG-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication

SELECTED OUTCOME

Make the two direct macOS helpers consume one authenticated host install-bootstrap context, derive
the Lima control root from that committed host principal's account-database home, perform only the
declared-instance pre-mapping Stage 1, and then observe, construct, and verify the canonical
`PlatformBootstrapMappingV1` from an already-running guest before any R2-owned guest projection.

Do not activate or test forwarding. Do not implement typed Rust backend consumers, installer/unit
projection changes, factory migration, replay, shim, Windows, or R3 lifecycle behavior.

EXACT COMPLETION CLAIM

R2-3M1 owns PI-039 and the macOS half of PI-081. It supplies only the direct-helper/two-stage
prerequisites for PI-040, PI-041, and PI-075; it does not complete those rows, the macOS mapping
corridor, any native macOS proof, or the R2-3 parent.

EXACT ALLOWLIST

Production:

- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima-doctor.sh`

Tests:

- `tests/mac/lima_doctor_fixture.sh`
- `tests/mac/prefix_mapping_r2_3.sh` (new)

No other file may change. In particular, do not edit Lima profiles, unit/socket templates,
installers, Rust sources/manifests, `Cargo.lock`, stop/smoke scripts, control-pack documents,
generated files, `AGENTS.md`, or `CLAUDE.md`. If another file is required, stop with
`BLOCKED_SCOPE_EXPANSION`.

EXACT PRODUCTION SYMBOL SCOPE

Only these existing shell functions may receive production edits:

- `check_only_status`
- `render_profile`
- `vm_exists`
- `vm_status`
- `create_vm`
- `start_vm`
- `wait_for_running`
- `ensure_vm_ready`
- `configure_guest`
- `diagnose`
- `check_rendered_unit_parity`
- `run_breakglass_guest_checks`

Add exactly these production shell functions:

- `resolve_install_bootstrap_context_v1`
- `resolve_lima_control_root_v1`
- `run_limactl_with_mapping_env_v1`
- `observe_lima_mapping_v1`
- `verify_lima_mapping_v1`

No other production function may be added or edited. Test-only fixture helpers may be added inside
the two allowlisted test files.

The bodies and behavior of `destroy_vm`, `stage_workspace`, cleanup sections in
`write_systemd_units` and `enable_socket_activation`, and every forwarding action are frozen.
Existing staging, unit, socket, provisioning, and guest-build code may remain reachable only after
successful Stage-2 mapping validation on its historical normal warm path; none may be changed or
used as R2-3M1 proof. A layout mismatch must fail with the R3 prerequisite unmet instead of calling
the frozen delete/rebuild path.

PRE-EDIT IMPACT AND SOURCE CLOSURE

Before any edit:

- run file-disambiguated upstream GitNexus impact with tests included for every indexed existing
  function in the exact symbol list;
- perform an exact-name source-caller closure for every shell function GitNexus does not index;
- record direct callers, affected modules/processes, and risk;
- treat `ensure_vm_ready` and `configure_guest` as HIGH regardless of graph under-reporting; and
- warn before proceeding on any HIGH/CRITICAL result.

Continue only when the change remains inside the declared helper/mapping corridor. An unexpected
authority root, lifecycle edge, unrelated process, or scope expansion is a stop.

HOST CONTEXT AND PUBLIC/INTERNAL MODES

Both direct helpers must establish exactly one mode before any Lima command or diagnostic:

- Public mode accepts an explicit normalized `--install-prefix`, or uses the current non-root
  canonical Unix principal's account-database home plus `/.substrate` when absent. It constructs
  the existing canonical `InstallBootstrapContextV1`/carrier framing.
- Internal-child mode is selected only by the explicit
  `--install-bootstrap-context-v1 <carrier>` argv discriminator. It strictly validates canonical
  framing, commitment, current account+UID equality, and any declared prefix equality. It never
  reconstructs authority and never falls back to public mode after invalid input.

Ignore an outer carrier environment value as public authority. Reject root/ambiguous/non-round-
tripping principals. Never select A from `HOME`, `LIMA_HOME`, CWD, repository location, project
path, generated files, or guest state. Carrier/prefix/principal bytes are not printed or logged.
Checked environment projections are consistency checks only.

Use the already-published transport-api-types V1 framing exactly: strict LF-terminated record,
unpadded base64url, canonical decode/re-encode equality, recomputed commitment, and exact
`selected_host_prefix == host_substrate_home == host_substrate_root`. Do not invent another
context type, precedence branch, persistence record, side table, or normalizer.

LIMA CONTROL ROOT AND CHILD ENVIRONMENT

Resolve the committed Unix account+UID through the host account database, require account/UID
round-trip, normalize its absolute account home, and derive:

```text
host_account_home=<account-database home>
host_platform_control_root=<host_account_home>/.lima
```

This typed projection selects the Lima store before Stage 1. Public parents must discard outer
`HOME` and `LIMA_HOME`; every `limactl` child must run with:

```text
HOME=<host_account_home>
LIMA_HOME=<host_platform_control_root>
```

An internal child rejects a nonempty inherited `HOME` or `LIMA_HOME` that conflicts with those
exact projections. A public outer B must be scrubbed rather than allowed to select or block A.
`run_limactl_with_mapping_env_v1` is the sole new Lima-command wrapper for the edited paths.

TWO-STAGE LIMA CONTRACT

The declared/default VM name is normalized and fixed before Stage 1.

Stage 1 may only:

- inspect the declared instance's status;
- render the existing fixed profile;
- create that exact instance when absent;
- start that exact instance when stopped; and
- wait for that exact instance to report Running.

Before Stage 2, Stage 1 must not select or project guest home/principal, stage a workspace,
configure a service/unit/socket, launch forwarding, or perform deletion/rebuild/cleanup. An
existing instance requiring layout replacement, delete/rebuild, or another R3 action fails closed
with the R3 prerequisite unmet.

Once the exact instance is Running, Stage 2 observes from that guest:

- lowercase 32-hex `/etc/machine-id`;
- canonical `id -un` account;
- canonical decimal `id -u` UID;
- `getpwnam`/`getpwuid` round-trip equality; and
- the absolute normalized account-database home.

Missing, malformed, changing, non-round-tripping, ambiguous, relative, guessed, or environment-
derived values fail closed. Never fall back to `/home/<name>`, guest `$HOME`, a host-mounted home,
or host/guest account, UID, or path equality. Set
`realized_substrate_home=<guest_account_database_home>/.substrate`.

MAPPING CONSTRUCTION AND VERIFICATION

Construct/revalidate the existing canonical thirteen-line `PlatformBootstrapMappingV1` with:

- the recomputed IH commitment;
- `platform_kind=lima`;
- the exact declared VM name and observed guest machine ID;
- the account-database-derived host Lima control root;
- the observed guest realized home and Unix account+UID;
- `transport_kind=lima`;
- fixed future host socket `A/sock/agent.sock`; and
- fixed guest socket `/run/substrate.sock`.

Require strict framing, canonical base64url, digest/path/UID normalization, platform/transport kind
agreement, and exact semantic equality with live observations before any R2-owned guest projection.
Reject a conflicting carrier, commitment, control root, VM, machine ID, guest principal/home, or
transport. Do not create a mapping side table or generated authority record.

The mapping only describes the future V1 SSH-UDS target. R2-3M1 must not call a forwarder
constructor, select VSock/TCP/ambient endpoints, create or unlink a host socket, set
`StreamLocalBindUnlink`, kill/wait a forwarding child, exercise handle-drop teardown, or claim
product transport. Those remain R3 prerequisites.

DOCTOR AND CHECK-ONLY POSTURE

`lima-doctor.sh` and the warm helper's check-only path must use the same IH/control-root/mapping
observation rules. Diagnostics may report only non-secret selected commitment, declared instance,
control root, observed mapping fields, and explicit R3 forwarding prerequisite. Diagnostics never
establish authority and must not guess missing fields.

`diagnose`, `check_rendered_unit_parity`, and `run_breakglass_guest_checks` may be adapted only to
use the validated mapping environment and observed guest account-database home. Remove the current
`/home/<user>` fallback. Breakglass remains explicitly diagnostic and cannot satisfy product proof.
Check-only/doctor/mapping verification must perform no create/start/delete, staging, guest build,
unit/socket mutation, or forwarding action.

TEST-DRIVEN PROOF

Extend `tests/mac/lima_doctor_fixture.sh` and add
`tests/mac/prefix_mapping_r2_3.sh` to cover at least:

- explicit custom A and principal-derived default A;
- public conflicting outer B scrubbed before every Lima child;
- internal matching projections accepted and mismatched `HOME`/`LIMA_HOME` rejected;
- canonical carrier and PM encode/decode/revalidation, including tamper/noncanonical rejection;
- exact account-database Lima control root and overwritten child `HOME`/`LIMA_HOME`;
- declared VM only, exact Stage-1 status/create-or-start/wait ordering, and Stage 2 only after
  Running;
- guest machine ID, account, UID, account-database-home observation and round-trip validation;
- rejection of guessed `/home/<name>`, guest `$HOME`, malformed/changing machine ID, and mismatched
  principal/home;
- fixed `A/sock/agent.sock` to `/run/substrate.sock` mapping despite ambient VSock/TCP/socket
  availability;
- mapping/commitment/instance/projection mismatch rejection;
- layout mismatch stops without `destroy_vm`;
- check-only and doctor perform zero create/start/delete/stage/build/unit/socket/forwarder action;
  and
- the historical doctor fixture still proves its unchanged routed-readiness behavior where outside
  this mapping change.

Fixtures must be deterministic and non-destructive. Native macOS/pre-existing-Lima evidence remains
assigned to R2-3Z and must not be claimed here.

REQUIRED VERIFICATION

Run:

- `bash -n` on all four allowlisted shell files;
- `shellcheck` on the allowlisted files when already installed, recording exact availability;
- `bash tests/mac/lima_doctor_fixture.sh`;
- `bash tests/mac/prefix_mapping_r2_3.sh`;
- focused check-only/doctor fixture cases proving no lifecycle action;
- `git diff --check`;
- exact four-file unstaged and staged allowlist/status checks;
- proof that every other file, manifest, `Cargo.lock`, profile, unit/socket template, stop/smoke
  script, and frozen function/body is unchanged;
- exact-name caller/source closure for edited shell functions after the change; and
- `gitnexus_detect_changes()` before commit, with every reported flow inspected.

Any retry reduction, skipped test, weakened assertion, lifecycle action used as proof, or static
result relabeled as native evidence fails the increment.

SUBJECT FINGERPRINT

After deterministic syntax/fixture checks and before discovery review:

1. Record the pre-edit base commit.
2. Build a sorted manifest containing that commit plus each exact allowlisted path, its Git mode
   (or `NEW`), and `git hash-object --no-filters` blob ID (or `MISSING`).
3. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Open and validate one V1 record with packet ID `A1.1d-5R2-3M1`. Use fresh read-only lenses for:

1. IH/current-principal/control-root authority and strict carrier/PM validation;
2. Lima Stage-1/Stage-2 ordering, live guest identity, public-B scrubbing, and fail-closed behavior;
3. exact allowlist, frozen R3/lifecycle/forwarding bytes, diagnostic honesty, and fixture adequacy.

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated/inventoried before completion; if that requires a fifth tracked file, stop with
`AUTHORITY_REQUIRED` for a separate inventory-only action. CLEAN is terminal.

PUBLICATION

After every gate passes and review is CLEAN:

1. Stage only the exact four allowlisted paths.
2. Inspect the staged diff and rerun syntax, focused fixtures, allowlist, frozen-byte, and
   no-lifecycle checks.
3. Commit one atomic Conventional Commit of at most 72 characters.
4. Fetch/query the live product ref and require it still equals the expected base.
5. Perform a normal fast-forward push of `HEAD` to the target ref.
6. Verify live remote equality, refresh GitNexus, and finish clean with 0 ahead/0 behind.

Do not create another branch, merge, rebase, force-push, open a PR, or begin/render R2-3M2.

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

Do not generate the next increment prompt and do not begin R2-3M2. The meta
orchestrator owns independent verification and subsequent dispatch.
