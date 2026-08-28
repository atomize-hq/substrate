**Kind:** contract/gate
**Stable ID:** `A1.1d-5R2-2-renewed-closeout-family`
**Canonical for:** A1.1d-5R2-2 renewed closeout contracts and gates
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** composite of the preserved root compatibility spans listed in Source provenance below
**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2 renewed closeout contracts and gates

## Canonical shell-library broad-wall invocation contract

This is the single normative command contract for every live reference in the six-file control
pack to a broad shell wall, parallel wall, serial wall, canonical or differential baseline wall,
final change-detection wall, renewed closeout wall, or F/Harness wall. Historical transcripts may
retain the command they actually ran only when they are explicitly labeled historical and
non-normative. No other live command form may establish shell-library broad-wall authority.

### Fresh private root and preflight

Every broad wall creates a fresh disposable root `R`. Before the test process starts, no-follow
descriptor validation must prove all of the following for `R` and retain enough descriptor
identity to prove the same object remained bound through the wall:

1. `R` is an absolute path to a directory, not a symlink, owned by the current user, with exact
   mode `0700` and stable filesystem identity.
2. No group or world write authority exists, and no access or default ACL grants effective write
   authority to another principal. Unsupported, malformed, unreadable, or otherwise uncertain ACL
   state fails closed.
3. The complete ancestor chain is no-follow safe, identity-stable, and free of foreign effective
   write authority. `R` is created beneath a trusted, non-world-writable ancestor and is never
   beneath `/tmp`, `/var/tmp`, or another shared sticky directory.
4. `R` is not the product's real `SUBSTRATE_HOME`, is not reused by another concurrent wall, and
   cannot select or modify product/user authority state.
5. Fresh `R/tmp` and `R/xdg-runtime` exist before Cargo starts. Each is a current-user-owned,
   exact-`0700`, non-symlink directory whose descriptor identity remains stable through the wall.

The exact test environment is:

```text
TMPDIR="$R/tmp"
XDG_RUNTIME_DIR="$R/xdg-runtime"
```

Both values must be exported or prefixed on the exact Cargo invocation before the test process
starts. Ambient `TMPDIR` or `XDG_RUNTIME_DIR` is never authority. Omission or invalidity of either
value makes the wall ineligible; it does not establish a regression or authorize a baseline
transition.

### Canonical command forms

Default-parallel shell-library wall:

```bash
TMPDIR="$R/tmp" \
XDG_RUNTIME_DIR="$R/xdg-runtime" \
cargo test -p shell --lib -- --nocapture
```

One-thread serial shell-library wall:

```bash
TMPDIR="$R/tmp" \
XDG_RUNTIME_DIR="$R/xdg-runtime" \
cargo test -p shell --lib -- --nocapture --test-threads=1
```

P1 implements—not supersedes—these command forms. Inside its Stage-B containment, Cargo sees the
same lexical CWD, exact argv, and exact inherited environment E0 with only the two prefix
insertions/replacements above. P1 adds no Cargo flag or environment key, does not set
`CARGO_HOME`, `CARGO_TARGET_DIR`, `CARGO_NET_OFFLINE`, `CARGO`, `RUSTC`, `PATH`, or `HOME`, and
does not consume the repository's host `target`. Its private authenticated snapshots are mounted
at the same lexical repository/rustup-home/Cargo-home paths and its fresh writable target is
mounted at the canonical default `<repository>/target`, making containment invisible to command
semantics. The copied Cargo home preserves the exact `cargo -> rustup` and `rustc -> rustup`
symlinks, while the copied rustup home preserves the selected 1.89 toolchain and settings; the
verified repository `rust-toolchain.toml` remains the selection authority. Any
argv/CWD/environment or resolution-chain difference is `BaselineCommandMismatchConfirmed` and
the wall is ineligible.

If a live Rust/libtest version requires a different argument order, evidence must record the exact
verified equivalent while preserving these semantics: the same shell library target, `--nocapture`,
default libtest parallelism for the parallel form, exactly one libtest thread for the serial form,
and both validated private-root variables installed before process start.

### Reproducible creation and cleanup template

The following is a normative semantic template, not a repository script. The selected parent must
pass the trusted-ancestor preflight before `mktemp` runs. The `require_*`, `wait_*`, `preserve_*`,
and `remove_*` names denote mandatory harness operations backed by descriptor-based, no-follow
validation; they are not optional comments or ambient `PATH` commands. A harness must implement
them, retain the validated descriptors, and stop nonzero if any operation is absent, fails, or is
uncertain.

```bash
trusted_parent="${SUBSTRATE_TEST_TRUSTED_PARENT:-$HOME}"

# Before creation: validate the exact absolute trusted parent and its complete
# ancestor chain for owner, directory type, no symlink, mode, identity, and
# effective ACL safety. Reject shared sticky parents such as /tmp and /var/tmp.
require_trusted_wall_parent "$trusted_parent" || exit 1
wall_root="$(mktemp -d "${trusted_parent%/}/substrate-r2-wall.XXXXXX")" || exit 1
[ -n "$wall_root" ] || exit 1

chmod 0700 "$wall_root" || exit 1
mkdir -m 0700 "$wall_root/tmp" "$wall_root/xdg-runtime" || exit 1

# Before Cargo: no-follow open and validate exact owner, directory type, mode,
# identity, trusted ancestor chain, and effective ACL safety for wall_root and
# both children. Retain their identities through the wall.
require_validated_wall_root "$wall_root" "$wall_root/tmp" "$wall_root/xdg-runtime" || exit 1

if TMPDIR="$wall_root/tmp" \
   XDG_RUNTIME_DIR="$wall_root/xdg-runtime" \
   cargo test -p shell --lib -- --nocapture; then
    status=0
else
    status=$?
fi

# Preserve the complete log, counts, failure names, normalized signatures, and
# required provenance before cleanup. Only after Cargo and every child exit:
# revalidate the exact wall_root path, owner, type, and retained identity, then
# remove only that exact disposable root. Never remove a glob or parent.
wait_for_wall_children "$wall_root" || exit 1
preserve_wall_evidence "$wall_root" "$status" || exit 1
remove_revalidated_wall_root "$wall_root" || exit 1

exit "$status"
```

`SUBSTRATE_TEST_TRUSTED_PARENT`, when present, is test-harness placement input only. It cannot
weaken any owner/mode/type/identity/ancestor/ACL check, cannot select product authority, and cannot
name a shared sticky directory. `mktemp -d` without this explicit trusted template is forbidden
because its ambient default may be `/tmp`. Cleanup occurs only after every Cargo child has
terminated, revalidates the exact root before removal, and never recursively targets a glob or the
trusted parent. Failure logs and hash artifacts are preserved before cleanup. Actual product and
user state remain untouched.

### Required provenance and eligibility

A wall result is canonical only when its retained evidence contains:

- absolute `R`, current owner UID, exact mode, directory/no-symlink proof, retained filesystem
  identity, and the complete ancestor/ACL safety result;
- exact `TMPDIR` and `XDG_RUNTIME_DIR` values;
- full Cargo command, repository working directory, Rust version, and Cargo version;
- exit status and discovered/passed/failed/ignored counts; and
- the complete failure-name hash and normalized-signature hash.

Validate this invocation provenance before comparing any result with a baseline. A failed root
preflight, missing evidence field, changed target, omitted variable, or invalid root classifies the
wall as ineligible. Correct the invocation, not trusted-root validation, tests, fixtures,
production code, or the canonical baseline, then rerun the complete required wall. Only a
provenance-valid wall may trigger `PassToFail`, `NewFail`, changed-failure, or other baseline
classification.

### Frozen baseline and command-mismatch evidence

The provenance-valid canonical shell-library baseline remains:

| Evidence | Canonical value |
|---|---|
| Counts | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Failure-name SHA-256 | `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` |
| Normalized-signature SHA-256 | `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |

The renewed-closeout invocation that omitted the private roots is retained separately as
ineligible evidence:

| Evidence | Ineligible value |
|---|---|
| Command | `cargo test -p shell --lib` |
| Counts | 1,309 discovered; 1,205 passed; 104 failed; 0 ignored |
| Failure-name SHA-256 | `7d6564bbcfaaed1fdb784eb4610683a02e4464b362215761d728be3e7361d8e5` |
| Normalized-signature SHA-256 | `ae67f87fbdc155575e13467506405afbd4568fc08b9058cb4159daa539549579` |
| Additional failures | 59, all caused by `TempDir` placement beneath normal mode-`1777` `/tmp` and correct trusted-root rejection |
| Classification | `BaselineCommandMismatchConfirmed` |

The causally correct historical control used `/home/spenser/t` for both variables and restored the
canonical counts and hashes. That path is historical evidence only and is not a canonical
machine-specific path. Serial and isolated tests failed identically without the private `TMPDIR`;
the exact private-root invocation restored the 45-failure wall. Runtime source and tests were
unchanged. The 59 failures are not added to the baseline, waived, treated as product failures or
environment drift, used to weaken trusted-root validation, or used to justify serial-only proof.
## Normative renewed R2-2 publication contract

The selected model is **Docs-on-top → one fast-forward publication**. This section is the normative
publication contract for the renewed production-fix-free integration closeout. It becomes
operative from exact publication-authority commit
`928f94e7b4c498273b40385f7bffea9e4f949700`, whose local/remote parity at
`feat/preserve-a1-1d-5r2-2-publication-authority-20260722` must remain unchanged. That activation is
monotonic for exact descendants. The B1 invocation correction becomes controlling authority only
when its exact successor commit has local/remote parity at
`feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722` and remains an unchanged descendant of
`928f94e7`; it is not bound to the publication-authority branch. Neither activation marks the
closeout or R2-2 complete, and neither is lost when final or authorized append-only remediation
documentation follows.

### Mandatory sequence

1. Preserve the exact unchanged linear 17-commit runtime range from F docs closeout
   `2f6f1f69b3519dafff01ef543e7d260da2c37700` through
   `e5fbd2d4441d248e137d52c44e493fb0abe158f8`.
2. Retain publication-authority commit `928f94e7b4c498273b40385f7bffea9e4f949700`
   unchanged above that range and preserve it at
   `feat/preserve-a1-1d-5r2-2-publication-authority-20260722`.
3. Retain the six-file broad-wall invocation correction unchanged on exact `928f94e7` and its
   preservation ref
   `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722`.
4. Commit/review exactly this six-file remediation-planning authority on `f7ded83`, preserve
   only its dedicated planning ref, and leave the source remote unchanged.
5. Add one bounded R1 implementation commit using only its exact two-file/function/test allowlist;
   run its focused proof and obtain a fresh authority/security review CLEAN.
6. Add one bounded P1 implementation commit using only its exact two new test-harness files and
   helper/test allowlist; run all authenticated self-tests and obtain a fresh provenance/security
   review CLEAN.
7. Establish a fresh canonical baseline only through P1: three parallel walls and one serial
   wall, each with its own fresh validated private root and complete eligible evidence. Prior
   ineligible walls cannot contribute.
8. Run every focused and broad gate in the renewed production-fix-free integration closeout from
   that exact R1/P1/baseline state. During this closeout phase permit no production, test, fixture,
   script, schema, dependency, generated, configuration, or policy-implementation change.
9. If any planning, remediation, baseline, closeout, or review gate fails or stops, do not push
   the source branch. Classify the stop and obtain any new implementation authority docs-first;
   never rewrite a historical result as passing.
10. Under the preserved pre-disposition publication gate, only when integration is fully
    proof-clean and all four independent integration reviews are CLEAN, create the
    integration-clean preservation required by the closeout packet, draft the final six-file
    integration-closeout documentation on top, validate and review it to CLEAN, commit exactly
    those reviewed bytes, and rerun final post-commit documentation validation/reviews. A purely
    documentary post-commit finding permits only an append-only six-file remediation-doc successor
    followed by renewed documentation validation/reviews. A proof/topology-invalidating finding
    reruns affected proof/review. No finding authorizes rewriting or source publication while
    unresolved.
11. Immediately before publication, freshly read the source remote and require exact old OID
    `2f6f1f69b3519dafff01ef543e7d260da2c37700`. Bind an explicit expected-old-OID CAS/lease to that
    exact OID and independently prove the proposed update is a fast-forward from that OID to final
    local HEAD. The CAS/lease is only a race guard; it never authorizes a forced or non-fast-forward
    update. Publish the complete linear source branch exactly once. Any intervening ref change is
    a hard stop, even when the changed ref is an ancestor of local HEAD.
12. Require final local HEAD, upstream, and source remote to match exactly at zero ahead/zero
    behind.
13. Preserve every existing runtime commit byte-for-byte and identity-for-identity.
14. Keep every preservation-only and blocked-donor commit outside source ancestry.

Replay, rebase, commit rewriting, cherry-pick, merge commit, force push, and any multi-step source
publication are forbidden. The 17 commits already sit on the latest canonical F documentation;
F's earlier replay fulfilled its packet-specific docs-first requirement. Historical Route D's
docs-first replay was likewise Route D-specific. Another replay would add identity churn without
architectural, semantic, or evidentiary value. The renewed integration closeout certifies the
already assembled range as a whole, and its final closeout document therefore follows the range it
certifies.

The docs-on-top -> one ordinary fast-forward publication model above remains controlling. Its RP0
sequencing and the step-10 pre-disposition four-review-CLEAN gate are historical only. RP3 is
complete. RP4 product proof on exact integration commit/tree
`8c46135c861a468dea316cf9fd7d6c6bb15bddac` /
`5358497a8baec6f36e15aaef58415a759e64977d` is accepted clean by human disposition, while the raw
persistence review remains `REQUEST_CHANGES` for three cache-only orchestration/attestation
findings preserved as non-blocking process-audit debt outside RP4 product-proof scope. This exact
six-file change was the bounded RP5 closeout packet; source publication had not yet occurred in
that run and was the exact next step after the reviewed/committed RP5 docs before later completing
without rewrite.

### Required topology

At the historical B1 checkpoint, before source publication:

```text
source remote: 2f6f1f69b3519dafff01ef543e7d260da2c37700 (F docs closeout)
  ↓
17 existing runtime commits, unchanged, ending e5fbd2d4441d248e137d52c44e493fb0abe158f8
  ↓
publication-authority docs commit 928f94e7, local source only and dedicated-preservation remote
  ↓
broad-wall invocation docs commit, local source only and dedicated-preservation remote
```

At that checkpoint the source remote remained at
`2f6f1f69b3519dafff01ef543e7d260da2c37700`, the local source was 19 commits ahead after the B1
correction, and the source worktree/index/untracked set was clean. The remediation-planning
topology below supersedes only that B1 next-node state.

After a future successful closeout:

```text
source remote F docs closeout
  ↓
17 existing runtime commits, unchanged
  ↓
publication-authority docs commit
  ↓
broad-wall invocation docs commit
  ↓
remediation-planning docs commit
  ↓
bounded R1 implementation commit and focused security review
  ↓
bounded P1 implementation commit and adversarial provenance review
  ↓
fresh P1 canonical baseline
  ↓
renewed production-fix-free integration wall and reviews, with no implementation/test commit
  ↓
final integration-closeout docs commit
  ↓
optional append-only six-file remediation-doc successor(s), only for post-commit documentary findings
  ↓
one fast-forward source push; local HEAD = upstream = remote
```

The final source contains the original 17 runtime commits plus the publication-authority,
broad-wall invocation, remediation-planning, bounded R1, bounded P1, and final-closeout commits and
only any review-required append-only remediation-doc successors, with no changed commit identity,
merge commit, or force push. The ordinary CLEAN path has four closeout-related documentation
commits—publication authority, B1 invocation authority, remediation planning, and final
closeout—plus the two bounded implementation commits. Immediately before that single publication,
the source remote must still be exact
`2f6f1f69b3519dafff01ef543e7d260da2c37700`; an explicit expected-old-OID CAS/lease binds that OID,
while a separate ancestry proof and server result require a normal fast-forward update. The lease
cannot authorize a forced update, and a concurrent source-ref change must fail rather than be
silently incorporated.

### Failure and no-push posture

If any renewed integration or final documentation gate fails or stops: do not modify runtime or
tests; do not publish the source branch; retain the exact local state and preservation refs;
classify the stop; and authorize any future remediation docs-first. A purely documentary finding
may be corrected docs-only before the relevant commit and reviewed by a fresh replacement. A
purely documentary post-commit finding permits only an append-only six-file remediation-doc
successor and renewed documentation validation/review. A finding that invalidates the proof state
or topology requires the affected proof/review to rerun from the corrected preserved state. A
commit containing a post-commit finding is never rewritten by amend, rebase, or force push. The
local publication-authority commit may remain preserved, but it is authority only and must not
imply that renewed proof ran,
R2-2 passed, product smoke passed, or any seam was promoted. At that checkpoint, R2-3 remained
blocked until the successful one-fast-forward publication completed; that publication later
completed, after which the accepted R2-3 suffix and refreshed native evidence closed R2-3. R2-4
and R3 remain later.

## A1.1d-5R2-2 closeout-remediation contracts (historical RP1/RP2 record)

These preserved contracts record the closed source facts and gates that governed the completed
bounded R1 and P1 increments. This exact six-document packet makes no implementation/test change
and does not reopen those contracts.

### R1 — release dry-run authenticated-carrier non-disclosure

#### Closed source facts

At `f7ded83ef147b748678ba6b028eea959870a04fe`,
`scripts/substrate/install-substrate.sh::run_cmd` has exactly four direct calls: three in
`run_with_sudo` and one in `deploy_shims`. Only `deploy_shims` carries
`--install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}"`. Dry-run `run_cmd` renders
verbatim `$*`; the renewed-closeout sentinel therefore reported
`carrier_marker_disclosed=true`. Live execution already uses exact `"$@"`.

`run_with_sudo` was closed across its current callers and error paths. Bare tools are resolved
through a fixed privileged PATH but are not checked against a named allowlist. The finite current
set is `groupadd`, `usermod`, package managers, `install`, `systemctl`, and `rm`, plus the one
permitted absolute ACL helper. Values include account/group names, packages, artifact/TMP and
destination paths, modes/owners, unit/socket names, and ACL actions/paths/groups. No current call
supplies an authenticated carrier, credential, authorization value, commitment preimage,
prompt/request material, secret token, or poisoned ambient selector. Other installer carrier
routes use fixed dry-run placeholders and xtrace suppression. The uninstaller has no `run_cmd`
route and remains frozen.

Existing redaction helpers are not reusable authority for this contract.
`scripts/linux/world-provision.sh::show_install_context_cmd` structurally displays with `%q` and
redacts the separate carrier and environment-assignment forms, but it is display-only, omits the
accepted equals form and the missing/duplicate/malformed/end-of-options contract, and is not
coupled to byte-exact execution argv. `crates/common::{redact_sensitive, redact_process_argv}` are
Rust-only logging helpers that apply heuristic sensitive-name/value matching; `redact_sensitive`
also has raw logging behavior. Importing or extending those helpers would broaden R1 beyond its
two-file allowlist. Both files and all three helpers remain unchanged.

#### Exact implementation allowlist

Only the following future edits are authorized:

1. `scripts/substrate/install-substrate.sh`
   - add exactly one adjacent helper named
     `run_cmd_with_redacted_install_bootstrap_carrier`;
   - change only `deploy_shims` to call that helper;
   - do not edit `run_cmd`, `run_with_sudo`, `resolve_install_bootstrap_context`, carrier
     construction/validation, another installer function, or the uninstaller.
2. `tests/installers/prefix_propagation_r2_2.sh`
   - add exactly `assert_shim_dry_run_carrier_non_disclosure`, reusing existing file-local
     assertion helpers without editing them;
   - add its exact top-level test-matrix invocation;
   - do not weaken or replace an existing assertion.

No other production or test file is authorized. `tests/installers/install_state_smoke.sh` and
applicable uninstaller tests are required unchanged compatibility commands, not editable surfaces.
GitNexus does not index these shell symbols, so the caller inventory above is the authoritative
manual impact result. Security impact is **CRITICAL** because the value is authenticated authority;
the implementation blast radius is deliberately bounded to one caller and one focused test file.

#### Two-channel command contract

`run_cmd_with_redacted_install_bootstrap_carrier` shall:

1. retain the original argv array as the only live-execution input and invoke exact `"$@"`;
2. structurally scan the original array before either display or execution, without copying a
   candidate value into diagnostics;
3. build a separate display argv only when `DRY_RUN=1`;
4. walk argv by array index and recognize both `--install-bootstrap-context-v1 VALUE` and
   `--install-bootstrap-context-v1=VALUE`; consume the separate following element or the equals
   suffix without rendering it, substitute exactly
   `<redacted-authenticated-bootstrap-carrier-v1>`, and preserve the input flag form in display;
5. boundary-safely render each remaining nonsensitive array element without `eval` or command
   reconstruction;
6. scan the complete argv so reordering cannot bypass the boundary;
7. fail with a fixed nonsecret error before display or execution on a missing/empty carrier or any
   duplicate, including one separate-form plus one equals-form occurrence; the exact stderr bytes
   for this and every structural rejection in items 8–10 are
   `[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n`, with no argv
   or ambient `INSTALLER_NAME` bytes, and the exact helper return status is `2`;
8. treat an element beginning with the exact bytes `--install-bootstrap-context-v1` but not
   matching the separate flag or a first-`=` split as a protected malformed near-match; reject
   before rendering any argv or invoking the CLI, while an equals value beginning with another
   `=` is still a redacted value and is left byte-exact for live CLI rejection;
9. track an exact standalone `--` end-of-options element; any later protected separate form,
   equals form, or near-match is not a carrier option and must be rejected with the same fixed
   nonsecret error before any display or execution, so the live Clap parser cannot echo it as an
   unrecognized subcommand;
10. reject a separate-form candidate whose first byte is `-` with the same fixed nonsecret error
   before display or execution, because Clap otherwise classifies and renders it as an unknown
   argument; an attached equals-form value may begin `-` or `=` and remains redacted while its
   original argv is passed byte-exactly for generic invalid-carrier rejection;
11. never emit the carrier, digest, prefix, suffix, length, fragment, commitment preimage, or a
   regex-derived approximation.

The release installer parser and production call construct `--flag value`; the invoked Clap-based
Substrate CLI also accepts `--flag=value`. R1 adds neither form and must safely display both. It
must not remove or transform the actual carrier, weaken duplicate/malformed validation, or change
install context, prefix selection, shim deployment, exit status, or xtrace behavior.
`deploy_shims` must continue disabling inherited xtrace before the helper and restoring its exact
prior state after success or failure. After a structural rejection it restores that state and
returns exact status `2`; after a structurally valid live invocation it propagates the child's
exact status unchanged. Generic `run_cmd` output remains byte-compatible for every nonsensitive
caller.

#### R1 proof gate

The focused test must prove:

- dry-run output uses the exact fixed placeholder and contains no carrier marker;
- normal execution receives byte-exact original argv and exact argument boundaries;
- spaces, quotes, newlines, Unicode, leading dashes, and reordered arguments cannot break
  redaction;
- separate and equals forms redact without changing their live execution argv; missing, empty,
  duplicate, mixed-form duplicate, malformed near-match, separate leading-dash, and invalid
  carrier inputs, plus every protected form/near-match after `--`, fail without rendering the
  candidate value and preserve existing parser/contract behavior; every helper structural
  rejection emits only exact stderr
  `[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n`, returns exact
  status `2`, and a valid child status is propagated unchanged;
- inherited xtrace cannot disclose the value and is restored exactly;
- sentinel values representing credentials, tokens, authorization, commitment preimages,
  prompts/requests, and unselected private-path markers that exist only inside the carrier do not
  enter stdout, stderr, logs, traces, or generated files; intentionally public selected-prefix and
  executable-path dry-run displays remain compatible;
- install behavior, authenticated context validation, account/UID binding, prefix selection, and
  shim deployment remain unchanged;
- the unchanged full installer dry-run and installer/uninstaller compatibility suites remain
  CLEAN.

Any extra live disclosure is a fresh security finding and requires new docs-first authorization;
it must not silently broaden R1.

### P1 — canonical broad-wall provenance runner

#### Source-closure decision

No existing tracked test helper provides all required properties. The failed external
`run_private_wall.py` used `subprocess.run`, waited only for Cargo, unconditionally asserted child
exit, closed validation FDs, and then called pathname `shutil.rmtree`. Its wrapper made a PID
namespace but left shell/chroot/Python layers without a verified subreaper-to-eligibility
handshake. Therefore:

1. Cargo exit did not prove every descendant exited;
2. a descendant could remain alive or retain a handle;
3. closing descriptors lost object-identity continuity;
4. pathname-only removal admitted replacement ambiguity;
5. matching counts/hashes could not establish provenance.

Installer/test fixture scripts do not solve descendant containment. Production `trusted_fs`,
world cgroups, service scripts, and platform provisioning are forbidden product surfaces. One
new bounded tracked runner is therefore the minimum correction. The source-closed environment
permits exact nested unprivileged Bubblewrap user/mount/PID namespaces with the invoking UID/GID
mapped one-to-one, so a concrete bounded mechanism exists and
`BroadWallContainmentDecisionRequired` is not triggered.

The source-closure candidate inventory is exact:

| Candidate | Useful capability | Rejection or selection |
|---|---|---|
| External failed `~/.cache/substrate-a1-r2-2-renewed-closeout-20260722/run_private_wall.py` | root validation, Cargo launch, logs/counts/hashes | Rejected: parent-only `subprocess.run`, no descendant authority, closes FDs, pathname `shutil.rmtree` |
| `scripts/ci/dispatch_ci_testing.sh` embedded Python launcher | bounded child wait/timeout | Rejected: one process only; no private-root, reparenting, descriptor, ACL, deletion, or evidence contract |
| `scripts/ci/check_self_hosted_runners.sh` | CI host prerequisite inspection | Rejected: no wall launch/containment/root/evidence lifecycle |
| `tests/installers/{prefix_propagation_r2_1,prefix_propagation_r2_2,dev_shim_bootstrap_context_r2_1,world_provision_context_r2_2,standalone_cli_prefix_r2_1}.sh` | safe-parent fixture roots and trap cleanup | Rejected: fixture-specific `mktemp`/trap; no descendants, stable descriptors, mount identity, or structured evidence |
| `crates/shell/src/execution/routing/test_utils.rs` and colocated `TempDir` helpers | Rust test fixture creation | Rejected: test-local path lifetime only |
| `crates/shell/src/execution/agent_runtime/host_session_authority/trusted_fs.rs` | production owner/mode/ACL/no-follow identity | Rejected: private production authority, not a standalone wall/deletion/supervision helper |
| `crates/world/src/cgroups.rs` and world isolation modules | cgroup attachment and product-world teardown | Rejected: product/privileged state and lifecycle semantics; no P1 edit/import |
| `/usr/bin/unshare` and `/usr/bin/systemd-run --user` | namespace or user-cgroup launch | Rejected: neither combines authenticated runner bytes, immutable inputs, descendant/result handshake, continuous descriptor deletion, and evidence; systemd-run mutates user-manager state |
| `/usr/bin/bwrap` 0.11.0-1 | unprivileged nested user/mount/PID namespaces, private tmpfs, read-only projections, die-with-parent, JSON child status, default namespace reaper or explicit `--as-pid-1` command | **Selected external platform-TCB primitive** after exact identity/hash and clean nested probe. A detached-descendant probe returned the initial child status immediately under the default reaper, so default Bubblewrap wait/PID 1 is fail-safe teardown only. Stage A may use that fail-safe; Stage B must use `--as-pid-1` so the authenticated worker itself is the reported namespace PID 1 and its verified normal reap-to-`ECHILD` record is mandatory |

No other repository Python, shell, Rust test utility, cgroup, subreaper, checkpoint, or cleanup
helper supplies the combined contract. Bubblewrap is not extended or edited.

#### Historical runner allowlist (retired diagnostic-only)

The following Python-runner design is preserved as historical diagnostic evidence only. It is not
the current broad shell-wall authority, and B1_B2_1 retires the tracked runner files instead of
extending them.

The historical P1 design authorized only these new files:

1. `scripts/ci/canonical_shell_wall_runner.py`, Linux-only, Python standard library only, with:
   exact constants `BOOTSTRAP_V2_SOURCE`, `PLATFORM_STARTUP_TCB_V1`,
   `HOST_INVOCATION_ARGV_TEMPLATE_V1`, `BWRAP_STAGE_A_ARGV_TEMPLATE_V1`,
   `BWRAP_STAGE_B_ARGV_TEMPLATE_V1`, and
   `CANONICAL_STDLIB_MANIFEST_V1`; `main`, `parse_args`, `inline_sha1`, `inline_sha256`,
   `_resolved_commit_oid`,
   `_resolved_authority_commit_oid`, `_resolved_reviewed_authority_commit_oid`,
   `validate_platform_startup_tcb`, `exec_held_executable`,
   `verify_expected_head_object_chain`, `read_expected_head_blob`,
   `parse_stdlib_manifest`, `preload_validated_modules`, `load_authenticated_module`,
   `run_authenticated_self_tests`, `construct_repository_snapshot`,
   `construct_rustup_home_snapshot`, `construct_cargo_home_seed`,
   `construct_cargo_home_runtime`, `validate_cargo_home_runtime_writes`,
   `reconstruct_locked_vendor`,
   `seal_snapshot_mounts`, `create_sealed_runner_memfd`,
   `_initialize_staged_artifact_authorities`, `launch_stage_a_bwrap`,
   `launch_stage_b_bwrap`, `parse_bwrap_status`, `create_worker_control_listener`,
   `accept_authenticated_worker`, `collect_stage_b_output`,
   `write_preopened_provenance`, `_publish_evidence_directory`,
   `host_main`, `stage_a_main`, `stage_b_worker_main`,
   `set_child_subreaper`, `wait_for_containment_empty`, `terminate_contained_children`,
   `prepare_backing_root`, `verify_namespace_mount_teardown`,
   `open_validated_directory`, `validate_safe_ancestor_chain`, `read_effective_acl`,
   `statx_identity`, `revalidate_named_identity`, `verify_clean_repository`,
   `_verify_clean_repository_with_gitdir`,
   `validate_python_runtime`, `resolve_validated_executable`, `validate_rustup_resolution`,
   `hash_open_file`, `remove_tree_at`,
   `unlink_validated_entry_at`, `preserve_evidence`, `summarize_test_log`,
   `_validate_provenance_record`, `_verify_runtime_argv`,
   `_validate_stage_b_projection_authority`, `_default_provenance`,
   `_read_authenticated_staged_provenance`, `_finalize_ineligible`,
   `finalize_after_stage_a_exit`, `_run_bounded_fixture`, and `write_provenance`.
2. `scripts/ci/test_canonical_shell_wall_runner.py`, containing one
   `CanonicalShellWallRunnerTests` suite; exact fixture/helper symbols `setUp`, `tearDown`,
   `make_safe_parent`, `make_disposable_repo`, `fixture_git_environment`,
   `make_fake_rustup_home`,
   `make_fake_cargo_home_seed`, `make_registry_archives`, `invoke_bootstrap`,
   `invoke_self_test`,
   `run_bounded_fixture`, `read_final_provenance`,
   `_assert_primary_command_uses_private_umask_without_mutating_parent`,
   `_assert_stage_b_rejects_runtime_cwd_drift_from_projection_authority`, and
   `assert_ineligible`; and only the
   self-tests named below.

Fixture-only descendant repositories and any stubbed inner self-test payloads are unit coverage
for ancestry/bootstrap wiring only. They do not satisfy the authenticated proof requirement; the
required proof remains the later disposable full non-worktree checkout described in this contract.

No repository/Python dependency, generated file, configuration, production test, runtime
shell/world/policy/service, lifecycle, capability, secure-FD, receipt/supervisor, cleanup, or
placement symbol may change. The fixed interpreter is `/usr/bin/python3.13`; the runner uses
standard-library `os`/`ctypes` bindings for `prctl`, `statx`, xattr, pidfd, signal, wait,
`renameat2`, and descriptor operations. Bubblewrap alone performs namespace, UID/GID-map, tmpfs,
bind, `/proc`, and mount teardown operations under the exact reviewed argv. The runner resolves no
namespace helper or executable through ambient `PATH`, requires exact current-account
`/etc/subuid` and `/etc/subgid` entries plus exact held `/usr/bin/newuidmap` and
`/usr/bin/newgidmap`, and requires no privileged service, daemon, or product-state mutation. If an
exact Bubblewrap feature
or required Linux syscall is unavailable, it emits an ineligible/environment result and stops;
no weaker fallback is permitted. Exact isolated interpreter flags also exclude ambient Python
startup state.

#### Historical runner interface (retired diagnostic-only)

The following interface is preserved only to explain the retired diagnostic runner. It must never
be treated as the current broad shell-wall entrypoint.

The concrete host-side controller is the authenticated in-memory runner's `host_main`, executed
directly by exact root-owned `/usr/bin/python3.13` with no enclosing Bubblewrap. Platform
containment authority begins only when that controller, after authenticating the live
expected-head runner/test blobs against a reviewed authority commit and held platform inputs,
directly launches exact installed Bubblewrap
`/usr/bin/bwrap`, package `bubblewrap 0.11.0-1`, version stdout `bubblewrap 0.11.0\n`, SHA-256
`11e350f9154f8c8e30482fb087f4c3d033907452b7a5d82d027b6662d07f3ab6`, under the root-owned,
mode-`0755`, no-foreign-write-ACL platform TCB. Its exact ELF closure also includes
`/usr/lib/libcap.so.2` SHA-256
`925dd48b2062a4f434981ad351e433dd68167de69e1ed39a1ff2e69ca5d4ecb2`,
`/usr/lib/libgcc_s.so.1`
`847b4db3641fb57c5b8d5ec7ff501d77ee9e0e0a98ab3facfdfbdf57a789446e`,
`/usr/lib/libc.so.6`
`6fa19a84c327820e76051b86a8f42c1863e946519f9d3185f27db8937203780d`, and
`/usr/lib64/ld-linux-x86-64.so.2`
`2da5672efa5bba637eb7e01d485227790255088778435f1a6ea5c5db485a4a0e`.
The SHA-256 of bytewise path-sorted exact `PATH SP SHA256 LF` records for Bubblewrap plus those
four files is
`4c954d6217bbd1c42b51486388120e47f10cd834624002aa34861711f4c1f40b`. Each is root-owned,
no-foreign-write-ACL, and identity/hash pinned. Bubblewrap is the source-closed concrete
unprivileged namespace authority; a nested user/PID-namespace probe completed cleanly on this
checkpoint. `/usr/bin/python3.13` plus exact `REVIEWED_BOOTSTRAP_V2_BYTES` and authenticated
`host_main` are the concrete host-controller authority. No shell, tracked launcher pathname,
unnamed supervisor, or Stage-A self-observation is part of the success chain.

```text
executable = /usr/bin/python3.13 from the immutable platform TCB
envp       = canonical inherited wall environment E0, with every loader/Python injection key absent
argv       = /usr/bin/python3.13 -I -S -B -c REVIEWED_BOOTSTRAP_V2_BYTES
             host --mode MODE --label LABEL --timeout-seconds SECONDS
             --evidence-parent ABSOLUTE_SAFE_DIRECTORY
             --expected-head FORTY_LOWERCASE_HEX
             --authority-commit-oid FORTY_LOWERCASE_HEX
             --reviewed-authority-commit-oid FORTY_LOWERCASE_HEX
```

The displayed vector is the structural rendering of
`HOST_INVOCATION_ARGV_TEMPLATE_V1`, not one invariant runtime hash. The template is the exact
NUL-terminated concatenation of its argv elements, with seven whole-element ASCII slot tokens:
`{MODE}`, `{LABEL}`, `{TIMEOUT_DECIMAL}`, `{EVIDENCE_PARENT}`, `{EXPECTED_HEAD}`,
`{AUTHORITY_COMMIT_OID}`, and `{REVIEWED_AUTHORITY_COMMIT_OID}`. No slot may appear inside an
option name or a partial element. Runtime construction validates each value by the CLI contract,
substitutes each slot exactly once,
forbids NUL, and hashes the resulting NUL-terminated argv bytes as that wall's
`host_argv_sha256`.

Stage-A and Stage-B constants use the same framing. Their closed whole-element slot sets are:

- Stage A: `{MODE}`, `{LABEL}`, `{TIMEOUT_DECIMAL}`, `{CURRENT_UID_DECIMAL}`,
  `{CURRENT_GID_DECIMAL}`, `{STAGE_A_USERNS_FD_DECIMAL}`, `{STAGE_B_USERNS_SYNC_FD_DECIMAL}`,
  `{RUNNER_DATA_FD_DECIMAL}`, `{STATUS_FD_DECIMAL}`, `{EVIDENCE_PARTIAL}`, `{BACKING_PATH}`,
  `{REPOSITORY_SOURCE}`, `{RUSTUP_SOURCE}`, `{CARGO_SOURCE}`, `{EXPECTED_HEAD}`,
  `{AUTHORITY_COMMIT_OID}`, and `{REVIEWED_AUTHORITY_COMMIT_OID}`;
- Stage B: `{MODE}`, `{LABEL}`, `{TIMEOUT_DECIMAL}`, `{CURRENT_UID_DECIMAL}`,
  `{CURRENT_GID_DECIMAL}`, `{RUNNER_DATA_FD_DECIMAL}`, `{STATUS_FD_DECIMAL}`, `{ROOT}`,
  `{REPOSITORY_SNAPSHOT}`, `{RUSTUP_SNAPSHOT}`, `{CARGO_HOME_RUNTIME}`,
  `{CONTROL_DIRECTORY}`, `{REPOSITORY_CWD}`, `{EXPECTED_HEAD}`, `{AUTHORITY_COMMIT_OID}`, and
  `{REVIEWED_AUTHORITY_COMMIT_OID}`.

Every path slot is the already descriptor-validated exact absolute pathname for that invocation;
FD slots name only the exact explicitly passed Bubblewrap-consumed descriptors below. The host
controller propagates its already validated mode, label, and timeout only through those exact
whole-element Stage-A slots; Stage A repeats their validation and propagates the same byte-exact
values only through the corresponding Stage-B slots. They never use the status socket or
environment as authority. Unknown, missing, duplicate, partial, or unused slots reject. Actual
Stage-A/Stage-B argv hashes are
computed after substitution and recorded per wall; they are never commit trailers.

Invocation authority is not an external pathname or replaceable side artifact. The one bounded P1
implementation commit must carry, in its immutable commit message, exact unique trailers
`P1-Bootstrap-Bytes`, `P1-Bootstrap-SHA256`, `P1-Host-Argv-Template-SHA256`,
`P1-Stage-A-Argv-Template-SHA256`, and `P1-Stage-B-Argv-Template-SHA256`. The expected-HEAD runner
blob named by the reviewed authority commit contains exact `BOOTSTRAP_V2_SOURCE` and the three
template constants. Because the templates contain literal `{EXPECTED_HEAD}`,
`{AUTHORITY_COMMIT_OID}`, and `{REVIEWED_AUTHORITY_COMMIT_OID}` rather than resulting commit OIDs,
their hashes have no self-reference. The fresh P1 provenance/security reviewer independently
recomputes the byte length and four
constant/template hashes from that reviewed blob, requires exact trailer equality, records the
resulting exact P1 commit OID, and returns CLEAN only for that one commit. This commit object plus
its independently verified runner blob is
`P1InvocationAuthorityV1`; Git object framing supplies immutable identity, so no artifact file
location, owner, mode, or mutable producer exists.

Before each wall, the closeout initiator reads `BOOTSTRAP_V2_SOURCE` from that exact reviewed Git
blob—not the worktree—substitutes the host template with `{EXPECTED_HEAD}` equal to the live
descendant commit under test, `{AUTHORITY_COMMIT_OID}` equal to the reviewer-recorded P1 OID, and
`{REVIEWED_AUTHORITY_COMMIT_OID}` equal to that same reviewer-recorded P1 OID, then submits the
resulting vector through a direct array-based `execve(2)`/spawn primitive with the bootstrap as
one `-c` element. This is the only permitted control-plane operation outside the tracked runner;
it performs no shell rendering, command substitution, environment interpolation, temporary
launcher creation, or tracked-path execution. If the closeout environment cannot supply that
direct-array primitive, stop as `BroadWallContainmentDecisionRequired`; an ambient/ad hoc launcher
is not a fallback. At its first instruction V2 binds `/proc/self/cmdline` to the exact validated
runtime instantiation of `HOST_INVOCATION_ARGV_TEMPLATE_V1`, authenticates the live expected-head
commit and reviewed authority commit through retained Git plus independent object framing, proves
that the authority commit is equal to or an ancestor of the live head, requires
`--reviewed-authority-commit-oid` to exactly equal `--authority-commit-oid`, requires exact
authenticated runner/test blob continuity across those two commits, recomputes the same
constant/template trailers from the reviewed authority blob, reconstructs the exact runtime vector
from the validated slot values, and compiles `host_main`. Live `HEAD`, CLI `--expected-head`, CLI
`--authority-commit-oid`, CLI `--reviewed-authority-commit-oid`, the reviewed P1 OID,
authenticated commit objects, runner/test blobs, V2 bytes, template hashes, and per-wall actual
argv hashes must agree. Stage A and Stage B repeat the corresponding template/substitution check,
and the Stage-B authority record carries both reviewed-authority OIDs plus the projected
repository CWD. Stage B additionally binds that projected repository CWD as authenticated
projection authority. The independent evidence gate repeats every comparison against the
reviewer's recorded P1 OID and the live descendant head; `reviewed_oid_matches` is derived from
the exact equality check between the two reviewed-authority inputs rather than asserted by the
caller. Once a trusted external control plane supplies that reviewed authority OID independently
of the live descendant, caller/artifact/bootstrap co-variation cannot turn a later descendant into
the reviewed authority. The runner alone does not prove independent review for a same-head
invocation where `expected_head == authority_commit_oid == reviewed_authority_commit_oid`.

`E0` is the exact environment inherited by the canonical Cargo command at the immutable
checkpoint. P1 control is carried only in validated argv/FDs, never environment. The launcher
rejects any `LD_*`, `GLIBC_TUNABLES`, `MALLOC_*`, `PYTHON*`, locale-path, dynamic-loader, or
Python-startup injection key; all other E0 entries remain byte-identical and unreported.
`host_main` validates and retains Bubblewrap and every source descriptor, creates the evidence
partial directory and empty backing mountpoint, seals the authenticated runner blob in a memfd,
and launches Stage A as its retained direct child. Stage A uses only fixed
`--die-with-parent`, a new user/mount/PID namespace, private propagation, fixed root-owned OS-TCB
read-only projections, read-only input projections, `--ro-bind-data` of that sealed runner memfd
at one fixed private path, one private tmpfs snapshot root, `/proc`, `/dev`, and the exact
host-created evidence partial directory writable-bound at fixed
`/run/substrate-wall-evidence` plus the one exact empty backing directory hidden beneath `M1`.
The evidence parent and its siblings are not visible in Stage A. Its exact ordered argv is frozen
by `BWRAP_STAGE_A_ARGV_TEMPLATE_V1`; no optional flag, ambient path, child-surviving arbitrary
non-stdio FD, or shell expansion is accepted. The only Stage-A Bubblewrap-process exceptions are
the exact sealed runner memfd named by `--ro-bind-data` and the exact status-pipe writer named by
`--json-status-fd`. `host_main` clears close-on-exec only for those two numeric descriptors in the
forked Bubblewrap child and retains their authoritative counterparts. Bubblewrap consumes and
closes the runner-data FD before the Stage-A command starts; its monitor alone retains the status
writer through the final `exit-code` record and then closes it. The Stage-A payload inherits
neither numeric descriptor, verifies that absence in its FD table, and verifies the read-only
runner blob hash before compiling it as `stage_a_main`.
If exact Bubblewrap identity/features or nested unprivileged launch is unavailable, stop as
`BroadWallContainmentDecisionRequired`; post-start self-hashing is not a substitute.

For human review only, the structural rendering of the host-controller argv is:

```bash
/usr/bin/python3.13 -I -S -B -c 'REVIEWED_BOOTSTRAP_V2_BYTES' host \
  --mode parallel \
  --label LABEL \
  --timeout-seconds SECONDS \
  --evidence-parent ABSOLUTE_SAFE_DIRECTORY \
  --expected-head FORTY_LOWERCASE_HEX \
  --authority-commit-oid FORTY_LOWERCASE_HEX \
  --reviewed-authority-commit-oid FORTY_LOWERCASE_HEX
```

That rendering is never eligibility evidence and must not be executed through an ambient shell;
the exact direct host vector and its nested Stage-A/Stage-B vectors are mandatory.

`REVIEWED_BOOTSTRAP_V2_BYTES` is the exact LF-terminated UTF-8 bootstrap literal frozen by the P1
implementation commit's authenticated runner blob and commit trailers, passed as the single `-c`
argument. The fresh reviewer records the exact P1 commit OID and independently reproduces the
bytes, length, SHA-256, and three argv-template hashes; the bytes are never read from a worktree pathname,
environment variable, shell substitution, temporary file, or mutable external artifact. The
implementation is ineligible until that exact commit receives fresh CLEAN review. No code that
has already begun executing is claimed to authenticate its own pre-execution origin.

The following V1 prototype is retained only as rejected planning evidence. It is **not**
authorized implementation and must never be invoked: it imported pathname-backed `hashlib` and
`subprocess` before trust validation, closed the validated Git FD, then executed mutable
`/usr/bin/git` by pathname.

```python
import sys
if not (sys.flags.isolated == 1 and sys.flags.no_site == 1 and sys.flags.no_user_site == 1 and sys.flags.ignore_environment == 1 and sys.flags.dont_write_bytecode == 1 and sys.flags.safe_path):
    raise SystemExit(66)
if sys.path != ["/usr/lib/python313.zip", "/usr/lib/python3.13", "/usr/lib/python3.13/lib-dynload"]:
    raise SystemExit(66)
import hashlib
import os
import stat
import subprocess
sys.excepthook = lambda *_: os._exit(65)
PYTHON = "/usr/bin/python3.13"
GIT = "/usr/bin/git"
RUNNER = "scripts/ci/canonical_shell_wall_runner.py"
PYTHON_SHA256 = "44c4dbc2292ba0f2f1b6eba5328ceefd41261e962b129c7eb9a670b1640a1acf"
GIT_SHA256 = "1a16ab6ec08c8e22b27360c1e12bfc426aa676cac113a14a42a8530baecdf060"
def read_regular(path):
    fd = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        st = os.fstat(fd)
        if not stat.S_ISREG(st.st_mode):
            raise SystemExit(65)
        data = bytearray()
        while True:
            part = os.read(fd, 1048576)
            if not part:
                return bytes(data), st
            data.extend(part)
    finally:
        os.close(fd)
python_bytes, python_st = read_regular(PYTHON)
if hashlib.sha256(python_bytes).hexdigest() != PYTHON_SHA256:
    raise SystemExit(65)
self_st = os.stat("/proc/self/exe")
if (self_st.st_dev, self_st.st_ino) != (python_st.st_dev, python_st.st_ino):
    raise SystemExit(65)
git_bytes, _ = read_regular(GIT)
if hashlib.sha256(git_bytes).hexdigest() != GIT_SHA256:
    raise SystemExit(65)
args = sys.argv[1:]
positions = [i for i, value in enumerate(args) if value == "--expected-head"]
if not args or args[0] != "run" or len(positions) != 1 or positions[0] + 1 >= len(args):
    raise SystemExit(64)
expected = args[positions[0] + 1]
if len(expected) != 40 or any(c not in "0123456789abcdef" for c in expected):
    raise SystemExit(64)
root = os.getcwd()
if os.path.realpath(root) != root:
    raise SystemExit(65)
gitdir = os.open(os.path.join(root, ".git"), os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | os.O_NOFOLLOW)
def read_git_regular(relative):
    parts = relative.split("/")
    if any(not part or part in (".", "..") for part in parts):
        raise SystemExit(65)
    current = os.dup(gitdir)
    try:
        for part in parts[:-1]:
            following = os.open(part, os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=current)
            os.close(current)
            current = following
        fd = os.open(parts[-1], os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW, dir_fd=current)
        try:
            st = os.fstat(fd)
            if not stat.S_ISREG(st.st_mode):
                raise SystemExit(65)
            data = bytearray()
            while True:
                part = os.read(fd, 4096)
                if not part:
                    return bytes(data)
                data.extend(part)
        finally:
            os.close(fd)
    finally:
        os.close(current)
head = read_git_regular("HEAD")
if not head.startswith(b"ref: refs/heads/") or not head.endswith(b"\n"):
    raise SystemExit(65)
ref = head[5:-1].decode("ascii", "strict")
if read_git_regular(ref) != (expected + "\n").encode("ascii"):
    raise SystemExit(65)
for forbidden in ("objects/info/alternates", "info/grafts", "shallow"):
    try:
        read_git_regular(forbidden)
    except FileNotFoundError:
        pass
    else:
        raise SystemExit(65)
env = {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "PATH": "/usr/bin:/bin", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_OPTIONAL_LOCKS": "0", "GIT_NO_REPLACE_OBJECTS": "1"}
result = subprocess.run([GIT, "--git-dir=/proc/self/fd/%d" % gitdir, "--work-tree=" + root, "--no-optional-locks", "--no-replace-objects", "cat-file", "blob", expected + ":" + RUNNER], stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, close_fds=True, pass_fds=(gitdir,), check=False)
os.close(gitdir)
if result.returncode != 0 or result.stderr:
    raise SystemExit(65)
source = result.stdout.decode("utf-8", "strict")
cmdline = open("/proc/self/cmdline", "rb").read().split(b"\0")
if len(cmdline) < 6 or cmdline[:5] != [PYTHON.encode("ascii"), b"-I", b"-S", b"-B", b"-c"]:
    raise SystemExit(65)
bootstrap = cmdline[5]
scope = {"__name__": "__main__", "__file__": "<git-blob:" + expected + ":" + RUNNER + ">", "__package__": None, "__CANONICAL_BOOTSTRAP_SHA256": hashlib.sha256(bootstrap).hexdigest(), "__CANONICAL_RUNNER_SHA256": hashlib.sha256(result.stdout).hexdigest()}
exec(compile(source, scope["__file__"], "exec", dont_inherit=True), scope, scope)
```

The rejected V1 bootstrap block is 4,598 bytes including its final LF and has SHA-256
`0c3bc02f4c83ffda76a3b901151923400905a80333449d590aed7ed331a069e1`. It executes no tracked
pathname only as a historical prototype claim; that claim is superseded and cannot establish
authority.

V2 has this closed algorithm:

1. The external platform TCB is explicit, not self-attested. On this Linux/x86-64 packet it is the
   kernel/ELF loader, exact Bubblewrap and its complete libcap/libgcc/libc/loader closure above,
   and root-owned, non-foreign-writable Python/glibc startup closure for Manjaro
   packages `python 3.13.7-1` and `glibc 2.42+r17+gd7274d718e6f-1`. It includes exact
   `/usr/bin/python3.13` hash
   `44c4dbc2292ba0f2f1b6eba5328ceefd41261e962b129c7eb9a670b1640a1acf`,
   `/usr/lib/libpython3.13.so.1.0`
   `e1c00674c629637066ac7195ce3da29bdcabc7459e575aedb10b253984882549`,
   `/usr/lib/libc.so.6`
   `6fa19a84c327820e76051b86a8f42c1863e946519f9d3185f27db8937203780d`,
   `/usr/lib/libm.so.6`
   `6c34914125e318409f1fc28503245e854aa773d6c1cee82d6824fef6fb03d0b4`,
   `/usr/lib64/ld-linux-x86-64.so.2`
   `2da5672efa5bba637eb7e01d485227790255088778435f1a6ea5c5db485a4a0e`,
   and the only pathname-backed modules loaded before the first `-c` statement:
   `encodings/{__init__,aliases,utf_8}.py` with hashes
   `78c4744d407690f321565488710b5aaf6486b5afa8d185637aa1e7633ab59cd8`,
   `cac92d68c7ea5bc0f05b448b9144e3bdf236d0b7d27ab66112e96d43aad15b3f`,
   `ba0cac060269583523ca9506473a755203037c57d466a11aa89a30a5f6756f3d`,
   and `linecache.py`
   `0939f472e54b4aadbf261d3c45a92d5aafe16e749c199917df9f4e47ce4cc4da`.
   The implementation review must confirm this exact startup set under the exact invocation,
   package versions, hashes, root ownership, modes, ancestor state, and absence of foreign
   effective write ACLs. Drift is `python_startup_tcb_invalid` and no P1 wall may start. This TCB
   is a stated platform prerequisite; the runner must not claim that code can authenticate bytes
   which necessarily executed before its first instruction.
2. Before any runner-controlled pathname import, V2 uses only `sys`, built-in/frozen
   `posix`/`os`/`stat`, builtins, and an inline audited pure-Python SHA-256 implementation. It
   revalidates and retains the TCB files, validates exact isolated flags/path and the complete
   preloaded-module set, then no-follow opens and retains exact root-owned safe `/usr/bin/git`.
   It never imports `hashlib`, `subprocess`, `ctypes`, or another pathname-backed module in this
   phase.
3. Git is SHA-validated through that retained FD. V2 requires `os.execve in os.supports_fd`,
   forks with bounded memfd/pipe capture, and calls `os.execve(git_fd, argv, fixed_env)` on the
   exact open object. The retained gitdir FD is passed only for
   `--git-dir=/proc/self/fd/N`; the Git pathname is never executed. After every Git operation V2
   re-fstats and rehashes the held FD, verifies the pathname still names it, and keeps it open
   through final runner/self-test blob authentication. Replacement makes the invocation
   ineligible but never executes the replacement. Git output is never object authority by itself:
   inline SHA-1 independently verifies `type + SP + decimal-size + NUL + content` for the expected
   commit, each traversed tree, and both final blobs; a strict parser traverses raw path components
   from verified commit tree to `scripts/ci/{canonical_shell_wall_runner.py,test_canonical_shell_wall_runner.py}`.
   Any OID/content mismatch, malformed/duplicate/ambiguous tree entry, wrong mode/type, replace,
   alternate, or path substitution rejects before compile. The same verified commit/tree/blob
   chain is the sole expected-tree authority for repository snapshot construction.
4. The held Git object reads the expected-HEAD blobs only after that independent object-chain
   proof. Every runner and self-test blob begins with
   one strict, sorted `canonical-stdlib-manifest-v1` comment header naming every built-in, frozen,
   source, and extension module it may load and every file-origin hash. V2 parses that header as
   bytes, no-follow opens and retains every file origin, validates root owner/mode/ancestor/ACL
   safety and hashes before importing it, preloads the complete union, verifies every resulting
   `__spec__.origin`, then sets `sys.path=[]` and installs a closed importer that permits only the
   already loaded manifest. No top-level runner/test import executes before this prevalidation.
   Every module FD remains open and is revalidated/rehashed after wall or self-test cleanup.
5. `host` compiles only the authenticated runner blob with synthetic
   `<git-blob:EXPECTED_HEAD:scripts/ci/canonical_shell_wall_runner.py>` and injects the V2,
   runner, TCB, and module-manifest hashes/FD identities. It rejects unless
   `/proc/self/cmdline` binds the exact validated runtime instantiation of
   `HOST_INVOCATION_ARGV_TEMPLATE_V1`. Authenticated `host_main` remains
   outside Stage A, retains the Stage-A pidfd/status and backing/evidence authority, and alone may
   finalize `eligible=true` after Stage A exits and its namespace is gone.
6. `host_main` supplies the exact authenticated runner bytes to Stage A by sealed memfd consumed
   only through Bubblewrap `--ro-bind-data`; no child depends on inherited arbitrary FDs. Stage-A
   V2 reauthenticates those bytes, then `stage_a_main` constructs three byte-owned read-only seeds
   under private tmpfs: repository, selected rustup/toolchain, and Cargo-home seed. The Cargo seed
   contains the exact `cargo`/`rustc` symlinks, copied rustup executable, and checksum-selected
   registry inputs. Stage A makes a separate writable byte-copy, `cargo-home-runtime`, before
   Stage B. Stage B read-only projects repository and rustup snapshots, but projects only that
   private Cargo runtime writable at the canonical Cargo-home path. The runtime begins as an exact
   seed copy with none of Cargo's three lock/cache metadata paths; Cargo 1.89 may create or mutate
   exactly regular files `.package-cache`, `.package-cache-mutate`, and `.global-cache`. Any other
   runtime write, created entry, removal, rename, symlink, or type/mode/owner drift is ineligible.
   The seed remains unchanged, the runtime is post-validated and descriptor-removed, and neither
   is a host alias.
7. Stage A creates current-UID mode-`0600` `AF_UNIX SOCK_SEQPACKET` listener
   `R/control/worker.sock` before nested Bubblewrap. Stage B bind-projects `R/control` at fixed
   `/run/substrate-wall/control`, authenticates the read-only runner blob supplied by its own
   sealed-memfd/`--ro-bind-data` input, and `stage_b_worker_main` connects to that fixed socket.
   Stage A accepts only `SO_PEERCRED` matching Bubblewrap's retained `child-pid`/pidfd. Both
   endpoints are close-on-exec; the listener pathname is unlinked and its absence proved before
   START. Cargo receives the original stdin unchanged and inherits no control descriptor.
   Repository/rustup read-only mounts, the private writable Cargo runtime, and namespace
   isolation make transient host/source replace-and-restore irrelevant. Stage A retains every
   in-namespace snapshot/runtime descriptor through validation and cleanup; `host_main` retains
   independent host-side teardown/finalization authority.
8. `self-test` uses the same held Git FD to authenticate both expected-HEAD blobs, registers the
   runner only as an in-memory module, compiles the test blob with its synthetic Git filename, and
   invokes exactly `CanonicalShellWallRunnerTests`; it performs no discovery, worktree import, or
   tracked-path execution. Self-test output is explicitly non-eligible evidence.

The sole self-test request uses the same host Python/TCB/E0/bootstrap direct-exec boundary and
exact argv; individual mechanics tests may launch only their bounded dummy Bubblewrap fixtures:

```text
["/usr/bin/python3.13","-I","-S","-B","-c",REVIEWED_BOOTSTRAP_V2_BYTES,
 "self-test","--evidence-parent",ABSOLUTE_SAFE_DIRECTORY,
 "--expected-head",FORTY_LOWERCASE_HEX,
 "--authority-commit-oid",FORTY_LOWERCASE_HEX,
 "--reviewed-authority-commit-oid",FORTY_LOWERCASE_HEX]
```

The authenticated loader constructs one ordered suite containing every allowlisted method below
exactly once and rejects any extra `test_*` method. It performs no `unittest` discovery. Success
requires all `101` methods and emits exact compact sorted-key stdout
`{"failed":0,"run":101,"schema":"substrate.canonical_shell_wall_runner_selftest.v1","status":"clean"}\n`,
zero stderr, and exit `0`. Loader/authority rejection exits `65`; invalid self-test CLI exits
`64`; a completed test failure emits the same four-key schema with actual integer `failed` and
`run`, status `failed`, exits `68`, and can never create wall eligibility or a canonical baseline.

`--mode serial` is the only other mode value. `host` requires seven public options:
`--mode`, `--label`, `--timeout-seconds`, `--evidence-parent`, `--expected-head`,
`--authority-commit-oid`, and `--reviewed-authority-commit-oid`. `self-test` requires four public
options: `--evidence-parent`, `--expected-head`, `--authority-commit-oid`, and
`--reviewed-authority-commit-oid`. `LABEL` must match `[a-z0-9][a-z0-9-]{0,63}`; `SECONDS` is an
integer in `1..=21600`; the evidence parent must already exist and pass the current-UID,
directory/no-follow, mode, ACL, and safe-ancestor checks; `--expected-head` must equal live
`HEAD`; `--authority-commit-oid` must equal or be an ancestor of that live head; and
`--reviewed-authority-commit-oid` must exactly equal `--authority-commit-oid`. The invocation CWD
must equal the no-symlink repository root. `host` and `self-test` are the only public roles.
`stage-a` and `stage-b-worker` are internal-only roles accepted solely with the exact sealed-blob
hash, fixed namespace/mount identity, and per-invocation private control/backing identities
created by authenticated `host_main`; a public attempt fails before setup. There is no tracked-path
execution, re-exec override, arbitrary command, root,
`TMPDIR`, `XDG_RUNTIME_DIR`, environment, shell, CWD, executable, hash, count, or baseline
override. `host_main` launches Stage A and retains host-side teardown/finalization authority;
`stage_a_main` constructs the private seeds/runtime and launches exact nested Bubblewrap Stage B
as its retained direct child; Bubblewrap status plus the fixed private credential socket
authenticate the Stage B worker. Immutable validated configuration crosses Bubblewrap only by
read-only bind/data projections and fixed argv, never assumed non-stdio FD inheritance.
Self-tests use only the authenticated in-memory loader above and may not add a public command
escape.

Before namespace setup the runner requires:

1. exact Bubblewrap identity/version/hash and nested unprivileged feature probe above, repeated
   through the same platform-TCB path before any wall;
2. exact `/usr/bin/python3.13` as its `/proc/self/exe`, SHA-256
   `44c4dbc2292ba0f2f1b6eba5328ceefd41261e962b129c7eb9a670b1640a1acf`, whose
   `["/usr/bin/python3.13", "--version"]` stdout is exact `Python 3.13.7\n`;
3. exact root-owned `/usr/bin/git`, SHA-256
   `1a16ab6ec08c8e22b27360c1e12bfc426aa676cac113a14a42a8530baecdf060`, whose
   `["/usr/bin/git", "--version"]` stdout is exact `git version 2.51.0\n`;
4. exact current-account regular executable `~/.cargo/bin/rustup`, SHA-256
   `20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c`;
   exact current-account symlinks `~/.cargo/bin/cargo -> rustup` and
   `~/.cargo/bin/rustc -> rustup`; and exact `rustup --version` stdout
   `rustup 1.28.2 (e4f3ad6f8 2025-04-28)\n` plus stderr
   ``info: This is the version for the rustup toolchain manager, not the rustc compiler.\ninfo: The currently active `rustc` version is `rustc 1.89.0 (29483883e 2025-08-04)`\n``;
5. exact account-database-home `~/.rustup/settings.toml`, SHA-256
   `46d451a6b8ea2bed375971c34fee742235e1cd143e34368c6f9211802126de45`,
   and exact verified repository `rust-toolchain.toml`, SHA-256
   `8aa4ad836dc3e6479a7677e869fd3ad6a9314e5624d76b5e7f666fe7820b30c8`,
   selecting the held no-follow directory
   `~/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu`; within it, `bin/cargo` and `bin/rustc`
   have SHA-256
   `5b32bd53b8a08d8e206daed51f3fce40cdc0729038ff96bd0ca845596a2c1019` and
   `c6ac0142c05a60b0f6c15e1a118bd5ac3bd04924cd6dc0c328f501c5f86e4142`, with exact versions
   `cargo 1.89.0 (c24e10642 2025-06-23)\n` and
   `rustc 1.89.0 (29483883e 2025-08-04)\n`.

Each executable's version probe is exact argv `[canonical-argv-zero, "--version"]`, executes the
retained FD with `execve(fd, argv, env)`, uses empty stdin, exact environment
`LANG=C.UTF-8` and `LC_ALL=C.UTF-8`, captures stdout/stderr separately, and accepts no whitespace
normalization. Bubblewrap, Python, Git, pinned Cargo, and pinned rustc require the exact single
stdout line above and zero stderr bytes; rustup requires its exact two stderr lines above. Two
additional proxy probes execute the retained rustup FD with canonical argv zero equal to the
`cargo` and `rustc` shim path respectively; they require the exact Cargo/rustc stdout lines above
and zero stderr. This proves the ordinary shim selection without executing a mutable pathname.

The complete held toolchain root has exactly 26 directories, 213 regular files, zero symlinks,
and 1,071,859,799 regular-file bytes. Its canonical binary manifest is sorted by raw relative-path
bytes. Each record is `type-byte` (`D`, `F`, or `L`), 8-byte big-endian path length, raw path, and
4-byte big-endian permission mode; a regular-file record then has 8-byte big-endian size and raw
32-byte SHA-256, while a symlink record instead has 8-byte big-endian target length and raw target.
The SHA-256 of the concatenated records is exact
`62de669575b22b82124eacb7caf100fd1e1041e42bd66b2a148f2ff6d14540c0`.
Any unsupported type, source submount, count/byte/hash/mode drift, or pre/post manifest difference
is `executable_identity_invalid`/`executable_identity_drift`.

The immutable platform TCB—not Python self-authentication—covers the code that necessarily runs
before the first bootstrap instruction. At that first instruction V2 verifies flags
`isolated=1`, `no_site=1`, `no_user_site=1`, `ignore_environment=1`,
`dont_write_bytecode=1`, and `safe_path=true`; exact `sys.path`
`["/usr/lib/python313.zip", "/usr/lib/python3.13", "/usr/lib/python3.13/lib-dynload"]`; the exact
initial module and `/proc/self/maps` manifests; absent `/etc/ld.so.preload`; and the exact
forbidden-key-free E0 environment contract. Before compile or execution of a runner/test blob, V2
validates and retains
the authenticated blob's complete standard-library manifest and preloads it. Every origin is
interpreter-defined `built-in`/`frozen` or a no-follow-opened root-owned regular file beneath one
validated `sys.path` entry with safe ancestors, no foreign effective write ACL, and exact hash.
Only after every origin is loaded and checked does V2 set `sys.path=[]` and close imports to the
fixed snapshot. Symlink, user-owned origin, `.pth`, `sitecustomize`, `usercustomize`,
cwd/script-directory origin, user site, ambient loader/Python influence, late import, module-file
drift, or TCB/bootstrap/runner/test hash mismatch is
`python_startup_tcb_invalid`/`executable_identity_invalid`/`executable_identity_drift`. The runner
never claims that already-executed platform bytes self-authenticate, and neither runner nor test
worktree pathname is executed or imported. Stage B Bubblewrap, its worker-as-PID-1 command, and
Cargo inherit none of host's retained TCB/module-origin FDs.

Each real executable is no-follow opened, regular, executable, owner/ancestor/ACL safe,
identity- and SHA-validated, retained by FD through the wall, and rehashed/revalidated after all
descendants. The two shim names are separately no-follow `lstat`-validated as current-account
symlinks with the exact relative target `rustup`; their path identities are retained and
revalidated. Stage A byte-copies the complete held toolchain plus exact rustup settings into its
private rustup-home snapshot and copies the held rustup bytes, two exact symlinks, and verified
registry inputs into its immutable Cargo-home seed. It then makes one separate private writable
byte-copy of that seed as the Cargo-home runtime. Stage B read-only projects the rustup snapshot
and writable-projects only the Cargo runtime at the original canonical paths, so E0 resolves
`cargo`/`rustc` through copied rustup exactly as the ordinary invocation did while no host source
alias can mutate executed bytes. The seed must remain byte-identical. The runtime initially
contains none of the three Cargo metadata entries; runtime changes are limited to creation and
content/locking state of exact root regular files `.package-cache`,
`.package-cache-mutate`, and `.global-cache`; any other difference fails closed. Exact private
rustc `--print sysroot` must emit that canonical toolchain path plus LF, and, inside the verified
Stage B worker containment, a
bounded pre-Cargo probe must compile and execute a
minimal no-dependency Rust test in exact `R/tmp/toolchain-probe`, reap it, and descriptor-remove
that probe directory before Cargo so the fresh target remains empty. The byte-identical E0
`PATH` must resolve `cargo`/`rustc` to the projected Cargo-home symlinks, which must resolve to
the projected rustup bytes and select the projected pinned toolchain. Wrappers, aliases, extra
symlinks, or resolution elsewhere are ineligible. P1 does not add `CARGO`, `RUSTC`, `CARGO_HOME`,
`CARGO_TARGET_DIR`, `RUSTUP_HOME`, `PATH`, `HOME`, or another environment selector.

Held `/usr/bin/git` runs only with explicit `--git-dir=<repo>/.git --work-tree=<repo>`,
`--no-optional-locks`, `--no-replace-objects`, `GIT_CONFIG_NOSYSTEM=1`,
`GIT_CONFIG_GLOBAL=/dev/null`, `GIT_OPTIONAL_LOCKS=0`, `GIT_NO_REPLACE_OBJECTS=1`, fixed C locale,
fixed `/usr/bin:/bin` PATH, and no other `GIT_*` input. The runner rejects object alternates,
replace refs, grafts, a shallow repository, a non-directory/symlinked gitdir, merge state,
worktree config, `info/attributes`,
and configured fsmonitor/sparse-index/untracked-cache authority. It no-follow opens and retains
`.git/config` and `.git/info/exclude`; the latter is hashed evidence only and is never consulted
for cleanliness. Fixed `git config --local --null --list --no-includes` must find only exact
`core.repositoryformatversion=0`, `core.filemode=true`, `core.bare=false`,
`core.logallrefupdates=true`, plus keys matching only
`remote.<name>.{url,fetch}` or `branch.<name>.{remote,merge,vscode-merge-base}`. Includes,
`core.excludesFile`, `core.worktree`, any other `core.*`, `extensions.*`, status/diff/filter/hook
configuration, or any unrecognized key is rejected. `.git/config.worktree` and
`.git/info/attributes` must be absent before and after the wall.

Cleanliness does not use `git status` or mutable local/global exclude authority. The independently
verified expected commit/tree/blob chain is authority. Fixed `git ls-tree -rz` is accepted only
when its complete raw result equals the runner's strict tree parser; Git output alone is never
authority. The runner obtains exact stage-zero index
path/mode/blob entries and flags with fixed `git ls-files --stage -v -z`, and requires the two sets be
byte-for-byte equal after both are normalized only to `mode<NUL>blob-id<NUL>path<NUL>`, with no
unmerged, sparse, intent-to-add, skip-worktree, assume-unchanged,
submodule, or non-stage-zero entry. It also parses the retained raw index header/extensions and
rejects split-index `link`, sparse-directory `sdir`, untracked-cache `UNTR`, and fsmonitor `FSMN`
extensions. Independently it no-follow opens every tracked worktree path,
requires regular/symlink type and executable mode match the tree entry, hashes Git blob framing
plus exact file or symlink-target bytes in Python, and requires the resulting blob ID equal both
tree and index. It records a bytewise path/mode/blob/SHA-256 manifest plus the index and HEAD/ref
descriptor identities/hashes and repeats every check after all descendants.

Untracked classification uses only tracked, byte-verified `.gitignore` files named by that exact
HEAD tree. The runner rejects an untracked `.gitignore` unless its own containing subtree was
already excluded by a tracked ancestor `.gitignore` without consulting that file; such a nested
file can affect only an already-ignored subtree and is excluded from authority. It then invokes fixed
`git ls-files --others --exclude-per-directory=.gitignore -z` without `--exclude-standard`; any
output is dirty. It separately hashes the ignored-output list from
`git ls-files --others --ignored --exclude-per-directory=.gitignore -z` for evidence only.
Therefore `.git/info/exclude`, global excludes, and local config cannot hide an untracked path,
while build/agent artifacts ignored by reviewed tracked policy remain allowed. Pre/post config,
exclude, ignore-source, tree, index, tracked-worktree, untracked, and ignored-list identities and
hashes must match except that bytes below tracked-policy-ignored paths may change. The runner
derives the logical exact command from `--mode`, creates the root itself, and embeds the canonical
`1309/1264/45/0` plus name/signature hash expectations.

Cargo receives exact E0, byte-for-byte, with only `TMPDIR=R/tmp` and
`XDG_RUNTIME_DIR=R/xdg-runtime` inserted/replaced. This is exactly the canonical shell-prefix
semantics; P1 adds, removes, or rewrites no other environment key. It rejects loader/Python
injection keys before Stage A and never emits E0 values. The evidence records only the sorted key
names hash, entry count, the two public overrides, and booleans proving every other entry was
byte-identical and no protected value was serialized. Parallelism is selected only by omission of
`--test-threads`; serial mode is selected only by the exact command suffix.

Stage B binds a fresh writable private target directory at exact lexical
`<repository>/target`, so unmodified Cargo default target selection sees a clean target without a
`CARGO_TARGET_DIR` override. The private writable Cargo-home runtime is projected at the exact path
E0 would normally select. Its immutable seed contains only the verified rustup regular file, exact
`cargo`/`rustc` symlinks, and Cargo.lock-selected checksum-verified registry subset; it contains
no config, credential, Git cache, lock/cache metadata, wrapper, unrelated executable, or extra
package. Stage A proves the runtime was an exact seed copy before Stage B and, after containment,
permits creation/content change only for the three named root regular metadata files. Any other
new path, deletion, rename, symlink/type/mode/owner change, or mutation is
`snapshot_identity_invalid`. The private rustup home contains only exact settings and the
selected complete pinned toolchain. Repository target and host Cargo/rustup-home bytes are never
Cargo inputs. Cargo-home runtime, target, temp, control, and XDG directories are
descriptor-removed only after containment emptiness.
Before and after the wall, the runner no-follow proves `config` and `config.toml` absent from every
Cargo search location from the repository `.cargo` ancestors through the account `.cargo`, and
proves account credential files are not mounted into the fresh home. Appearance, replacement, or
read ambiguity is `repository_identity_mismatch`/`executable_identity_drift` and invalidates the
wall.

The runner prints only `wall_gate=VALUE invocation_id=INVOCATION_ID` to stdout; it never prints the
evidence parent, repository/root paths, command argv, environment, or raw log. Fixed error
classifications go to stderr. Exit codes are:

| Code | Meaning |
|---|---|
| `0` | provenance eligible and canonical result match (`wall_gate=clean`) |
| `64` | invalid public invocation |
| `65` | repository/CWD/HEAD/tree/cleanliness or executable identity mismatch |
| `66` | Linux namespace/current-ID-map/kernel prerequisite unavailable; no fallback |
| `67` | safe-parent/root/descriptor/ACL/mount setup rejection |
| `68` | Stage B/worker handshake, containment, subreaper, timeout, survivor, or forced-teardown rejection |
| `69` | provenance eligible but counts/names/signatures differ (`wall_gate=regression`) |
| `70` | descriptor identity or exact cleanup/absence rejection |
| `71` | bounded output/evidence capture or finalization failure |
| `72` | fail-closed internal invariant failure |

#### Descendant-containment contract

At `host_main` entry the authenticated host controller completes TCB/repository/input validation,
captures fixed monotonic `snapshot_deadline = start + 900 seconds`, opens the evidence/backing
authority, and launches retained Stage A. Stage A authenticates the sealed runner projection,
constructs the three immutable seeds plus writable Cargo runtime under that deadline, captures
`stage_b_setup_deadline = now + 60 seconds`, creates `R/control/worker.sock` as a
current-UID mode-`0600` `AF_UNIX SOCK_SEQPACKET|SOCK_CLOEXEC` listener, and launches exact nested
Bubblewrap Stage B as its direct child. Stage A retains a pidfd for that exact Bubblewrap process
and the read end of Bubblewrap's fixed `--json-status-fd`. Stage B uses new user, mount, and PID
namespaces, one-to-one current UID/GID mapping, `--die-with-parent`, private propagation, and
exact `--as-pid-1`. The authenticated Python worker is therefore the namespace PID 1 and the
`child-pid` reported to Stage A, rather than a child hidden behind Bubblewrap's default reaper.

Before Stage B launch, Stage A also opens exact partial-evidence `cargo.log` by its retained
directory FD and creates one `pipe2(O_CLOEXEC)` stream. In the forked Bubblewrap child only, it
`dup2`s that single write endpoint onto both stdout and stderr, leaves original stdin byte-exact,
closes every other pipe endpoint, and executes retained Bubblewrap. Stage A closes its writer
immediately after successful spawn and continuously drains the sole read endpoint through
`collect_stage_b_output` into the already-open `cargo.log`; neither Stage B nor Cargo can see the
partial-evidence mount. Pinned Bubblewrap and the authenticated worker must emit zero
successful-path stdout/stderr of their own, so the single pipe is the exact combined Cargo byte stream in
kernel delivery order. Cargo inherits only original fd 0 and those exact fd 1/2 duplicates; it
inherits neither the read end, an evidence FD, nor a control FD.

The collector begins before Stage B can write, uses bounded reads/writes and continuous
backpressure, and retains its pipe/file identities through EOF, `fsync`, byte count, and SHA-256.
EOF is mandatory only after authenticated `ECHILD`, worker exit, and exact Stage-B Bubblewrap
reap; early EOF, a surviving writer, short write, collector failure, missing EOF, or descriptor
identity drift is ineligible. Up to 4 GiB the eligible `cargo.log` is byte-exact. On the first byte
beyond the cap, Stage A irreversibly marks `evidence_limit_exceeded`, continues draining to avoid
deadlock, preserves only the bounded diagnostic prefix, performs no result analysis, and can never
make that wall eligible. The fixed private control protocol remains numeric/status-only and never
carries log bytes or `SCM_RIGHTS`.

Bubblewrap's status stream must provide exactly two LF-terminated JSON objects with duplicate and
unknown keys rejected for pinned Bubblewrap 0.11.0: first exact keys `child-pid`,
`mnt-namespace`, and `pid-namespace`, naming the Stage B Python worker in the Stage-A PID namespace
and its namespace identities; then the sole key `exit-code` after completion. Stage A opens a
pidfd for the reported worker and verifies the namespace/mount identities. The Stage B worker
connects to fixed `/run/substrate-wall/control/worker.sock`; Stage A accepts exactly one peer and
requires `SO_PEERCRED` to identify that retained worker pidfd. The worker's first instructions set
and read back `PR_SET_CHILD_SUBREAPER`, verify its fixed projected control path and namespace
identity, and send `READY`. Stage A unlinks `R/control/worker.sock`, proves its name absent, and
sends the one-byte `START` record only after the Stage B pidfd/status record, worker pidfd,
credentials, namespace identities, mount manifest, listener absence, and setup deadline all
validate. Both accepted endpoints are close-on-exec; the worker keeps its own endpoint while its
Cargo child loses the inherited duplicate on exec. Cargo's stdin is the byte-identical original
stdin. Before `START`, the worker mounts nothing, forks nothing, and cannot start Cargo.

The source-closure detached-descendant probe used Bubblewrap's default namespace reaper and
returned the initial child status in 3 ms while a detached two-second child remained, proving that
default parent-only Bubblewrap wait can coincide with forced namespace teardown. Stage B therefore
uses `--as-pid-1`: the authenticated worker is both the JSON-reported child and namespace adoption
root, sets/reads back `PR_SET_CHILD_SUBREAPER`, and must normally reap to `ECHILD` before it exits.
The worker's PID-1 role does not make a Bubblewrap exit, JSON exit status, Cargo exit, process
group, `pgrep -P`, sleep, or one-time snapshot sufficient by itself.
Missing/spoofed/duplicate/out-of-order status/control records, early EOF, credential mismatch,
nonzero Stage B runner status, premature Bubblewrap/worker exit, or setup expiry makes the wall
ineligible before any cleanup result is considered.

After `START`, the worker captures
`wall_deadline = now + timeout_seconds`, starts exact Cargo as its child, waits for Cargo, and
then continues `waitid`/`waitpid` over every child until the kernel reports `ECHILD`.
Because the worker became a verified subreaper before Cargo, reparented, double-forked,
new-session, process-group-changing, or nested-PID-namespace descendants remain its waitable
children. A descendant that outlives Cargo is waited normally within the same wall deadline.
Only after `ECHILD` may the worker send one fixed authenticated `CONTAINMENT_EMPTY` record
containing bounded numeric Cargo status/reap counts and exit zero. Stage A requires that record
from the retained worker identity, then requires the worker and exact Stage B Bubblewrap process
to exit normally and the status FD to report exit code zero. This authenticated
subreaper-to-`ECHILD` chain is the sole descendant-completion authority.

On wall expiry or a surviving descendant, the worker first irreversibly records
`eligible=false` and bounded numeric diagnostics. It enumerates only processes visible inside
its private `/proc`, verifies the exact PID-namespace inode, excludes only itself, opens each
remaining target with `pidfd_open`, sends `SIGTERM`, and reaps for at most five
seconds; verified survivors receive `SIGKILL` through the same pidfds and are reaped for at
most five more seconds. Any signal, timeout, survivor, or Bubblewrap-forced teardown remains
ineligible. No numeric-PID-only signal, host-namespace enumeration, or unrelated target is
permitted.

On Stage B setup expiry or premature worker/Bubblewrap loss, Stage A marks the wall ineligible,
signals only the retained Stage B Bubblewrap pidfd with TERM, waits two seconds, then KILL if
needed, and reaps that exact direct child. Exact `--die-with-parent` supplies bounded namespace
teardown when Stage A disappears; it is never a success signal. Stage A never enumerates or
signals unrelated host processes. Stage B must be fully reaped before in-namespace root deletion
begins.

When normal emptiness or invalid-wall teardown finishes, Stage A captures
`cleanup_deadline = now + 120 seconds` covering log analysis, post-identity/snapshot manifests,
descriptor cleanup, and staged evidence hashing. It writes an authenticated bounded Stage-A
record into the host-created partial evidence directory and exits; it never claims that its own
enclosing mount namespace is gone. `host_main` then requires exact Stage-A Bubblewrap status,
pidfd/process disappearance, Stage-A mount-namespace disappearance, retained backing identity,
empty underlying mountpoint, and descriptor-relative backing removal before
`finalize_after_stage_a_exit` may atomically publish `eligible=true`. Any stuck Stage A or Stage B,
residual namespace/mount, deadline, forced termination, or missing authenticated emptiness record
remains ineligible even if diagnostic cleanup later succeeds.

Before Cargo, the runner must:

1. validate a current-UID-owned `0700` backing/evidence parent outside `/tmp`, `/var/tmp`,
   and every shared sticky or world-writable ancestor;
2. establish exact nested unprivileged Bubblewrap user/mount/PID containment and map only the
   invoking current UID/GID one-to-one;
3. create one fresh private root distinct from the real product `SUBSTRATE_HOME` and every
   other wall;
4. authenticate the Stage B worker from Bubblewrap status plus accepted fixed-socket
   `SO_PEERCRED`, set/read back
   subreaping, and complete the START handshake;
5. place Cargo, every test binary, and every descendant below that worker in the PID namespace.

Parent-only `wait`, `pgrep -P`, sleeps, one-time process snapshots, process-group membership,
or Bubblewrap's cleanup reaper cannot establish containment. A timeout or surviving descendant
makes the wall ineligible; diagnostic teardown never repairs it. The runner must never kill an
unrelated process.

#### Exact namespace and mount lifecycle

The authenticated host controller and two source-closed Bubblewrap stages perform this exact
sequence:

1. `host_main` opens and retains the safe evidence/backing parent, creates one empty exact
   backing mountpoint, validates sources/TCB, seals the authenticated runner blob, then launches
   Stage A as its exact direct Bubblewrap child. It retains Stage-A pidfd/status, namespace IDs,
   backing-parent/mountpoint descriptors, and the evidence partial-directory descriptor.
2. Stage A starts before any snapshot or Cargo process with fixed `--die-with-parent`, a new user,
   mount, and PID namespace, recursively private propagation, fixed root-owned OS-TCB projections,
   read-only no-follow source projections, the runner supplied by `--ro-bind-data`, writable bind
   of only the host-created partial evidence directory at fixed
   `/run/substrate-wall-evidence`, the exact empty backing path mounted at a fixed location, and
   one private `tmpfs` `M1` over that location. The evidence parent/siblings remain hidden. Exact
   UID/GID mappings bind only the invoking IDs. Any
   option/order/map/mount/blob drift rejects.
3. Authenticated `stage_a_main` records `M1`, creates current-UID `0700` `R`, `R/tmp`,
   `R/xdg-runtime`, `R/control`, writable private `target`, repository/rustup-home/Cargo-home-seed
   trees, and a separate writable `cargo-home-runtime`. Every entry is descriptor-relative and
   stays on `M1`.
4. From held no-follow inputs it constructs and revalidates the three authenticated immutable
   seeds defined above. Repository bytes/modes must match the independently verified expected
   tree; rustup settings/toolchain bytes must match their pinned complete manifests; the
   Cargo-home seed must contain exact rustup/shim/registry bytes and no Cargo lock/cache metadata
   file. The runtime is an exact byte-copy of that seed before Stage B.
   Unknown executable/package/config/credential/Git-cache content rejects.
5. Stage A creates the fixed private listener and launches Stage B with the exact validated
   runtime instantiation of `BWRAP_STAGE_B_ARGV_TEMPLATE_V1`: new user/mount/PID namespace,
   `--die-with-parent`, private
   propagation, a new `/proc`, read-only binds of repository/rustup-home snapshots, writable bind
   of the private Cargo runtime at canonical Cargo home, writable binds only for target, `R/tmp`,
   `R/xdg-runtime`, and the fixed control directory, plus the exact projected Stage-B repository
   CWD (`/home/spenser/__Active_code/substrate`).
   Stage-B runner bytes arrive through a separate sealed-memfd/`--ro-bind-data` projection.
   Stage A clears close-on-exec only for that exact data FD and the exact
   `--json-status-fd` writer in the forked Bubblewrap child. Bubblewrap consumes and closes the
   runner-data FD before starting the worker; its monitor alone retains the status writer through
   the final `exit-code` record and then closes it. The worker inherits neither numeric descriptor
   and proves that absence.
   Exact `--as-pid-1` makes the Python worker both namespace PID 1 and Bubblewrap's reported
   application; it establishes the verified subreaper/fixed-socket handshake before Cargo.
   Stage-B fd 1 and
   fd 2 are the same Stage-A-owned capture-pipe writer; fd 0 is original stdin; no evidence path
   or other capture descriptor is projected.
6. Cargo executes only after Stage B proves each projected target is on `M1`, has the expected
   seed/runtime manifest and mount flags, exposes no host source inode, the control pathname is
   absent, the worker peer matches the retained Bubblewrap child, stdin is unchanged, and Cargo
   will inherit no control FD. A concurrent host source mutation can affect neither copied bytes
   nor a Stage-B mount.
7. After Cargo and every descendant exit, the Stage B worker proves `ECHILD`, revalidates its
   snapshot/runtime/target/root views, sends authenticated completion, and exits zero. Exact
   Bubblewrap then tears down its private `/proc` and binds without lazy/forced unmount; Stage A
   requires its zero status and independently verifies that every Stage B mount is gone.
8. Stage A first requires capture-pipe EOF after Stage-B reap, fsyncs and hashes exact bounded
   `cargo.log`, then validates that the Cargo seed is unchanged and the runtime changed only in
   the three authorized metadata files. It preserves all remaining evidence outside `R`, removes
   target/tmp/XDG/control, runtime, and every private seed tree descriptor-relatively, retaining
   each in-`M1` parent/entry FD through that exact name-absence proof. It closes each removed
   entry FD, removes `R`, proves `R` absent, closes remaining in-`M1` FDs, writes the authenticated
   Stage-A completion record, and exits. It does not remove its own enclosing host backing path
   or claim its enclosing namespace unmounted.
9. Stage-A Bubblewrap tears down `M1` by normal namespace exit. Only authenticated `host_main`,
   still holding the separate underlying backing-parent/mountpoint authority, proves the exact
   Stage-A process and mount namespace gone, revalidates the now-empty underlying mountpoint,
   removes it descriptor-relatively, proves name absence, and closes that authority. It also
   requires host repository/rustup/toolchain/Cargo-home/cache pre/post manifests unchanged, then
   and only then finalizes the evidence record.

Lazy/forced unmount, busy/residual mount, changed mount ID/source/type/flags/propagation, unknown
snapshot content, a new Cargo config/child, or any Stage-A/Stage-B protocol drift is ineligible.
The runner touches no evidence-parent entry except its exact partial/final evidence directory; the
self-test-owned unrelated sibling sentinel is outside runner-managed paths.

#### Root, descriptor, and deletion contract

For the evidence parent, disposable backing mountpoint/private mounted parent, root `R`, `R/tmp`,
`R/xdg-runtime`, `R/control`, private target, three immutable seed trees, and writable Cargo
runtime, the runner shall:

1. open no-follow descriptors before use and record owner UID, exact `0700` mode, directory type,
   device/inode, `statx` mount ID, ancestor safety, and effective ACL state;
2. keep each authoritative parent/root/child descriptor open through setup, Cargo, all
   descendants, log preservation, final validation, that entry's unlink, and that entry's
   pathname-absence proof;
3. reject replacement, symlink substitution, owner/mode/ACL drift, descriptor/path identity
   mismatch, unexpected mount, unexpected entry type, shared/reused roots, and sticky or
   world-writable ancestry;
4. after containment emptiness, preserve output outside `R`, revalidate each descriptor and
   pathname, and recursively remove entries with no-follow descriptor-relative
   `openat`/`fstatat`/`statx`/`unlinkat` operations;
5. retain each authoritative parent/entry FD until the exact name is removed and absence is
   proved, then close that removed in-mount FD before Stage A exits; retain the separate
   host-owned underlying backing-parent/mountpoint authority in `host_main` across Stage-A exit
   and `M1` teardown, remove only that exact empty mountpoint after process/namespace/mount
   disappearance is proved, prove its pathname absent, and only then close the underlying
   authority;
6. reject any attempt to use a glob, ambient path, recursive parent removal, real product home, or
   pathname-only `shutil.rmtree`; the external self-test creates and verifies the unrelated
   sentinel before and after the run.

ACL inspection uses the standard-library xattr interface and parses the Linux
`system.posix_acl_access` and `system.posix_acl_default` values under the effective mask. A missing
ACL is accepted only when the mode check is exact; a malformed/unsupported ACL result, named user
or group with effective write, group-class effective write, other-class write, or ACL drift fails
closed.

Deletion before containment emptiness or closure of all authority before absence proof is an
ineligible provenance result.

#### Evidence layout, bounds, and atomic finalization

`host_main` creates a random 128-bit lowercase-hex `INVOCATION_ID` and a current-UID `0700`
directory `<evidence-parent>/.<LABEL>-<INVOCATION_ID>.partial` before Stage A. Stage A may write
only its fixed staged artifact basenames there; `host_main` retains the directory authority,
revalidates every staged file after Stage-A exit, adds the host teardown/finalization fields, and
alone performs the final rename. Every file is `0600`. The maximal result-complete final layout,
in its fixed artifact order, is:

```text
<evidence-parent>/<LABEL>-<INVOCATION_ID>/
  cargo.log
  containment.jsonl
  failure-names.txt
  normalized-signatures.txt
  summary.json
  provenance.json
  manifest.sha256
```

Every finalized evidence directory contains `provenance.json` and `manifest.sha256`. Of the five
result artifacts, `containment.jsonl` is present iff at least one containment event was committed,
`cargo.log` is present iff Cargo started, `summary.json` is present iff `result` is nonnull, and
`failure-names.txt` plus `normalized-signatures.txt` are both present iff their derivation
succeeded. A result-complete record contains all five. An invalid-analysis record may omit only
the two derived text files. A pre-Cargo staged failure therefore has zero or one of the five
artifacts according to whether containment events began; it does not synthesize an empty Cargo
log or summary. No other basename is allowed in a final directory.

For an evidence-eligible wall, `cargo.log` is the exact single-pipe combination of Cargo
stdout/stderr bytes in kernel delivery order and is never echoed to the runner console. It is
capped at 4 GiB. Overflow makes the wall ineligible, retains only the bounded diagnostic prefix,
drains the remaining pipe without analysis, and records no exact-log claim; `containment.jsonl`
is capped at 65,536 fixed-schema
events, 4,096 survivor PID records, 512 bytes per line, and 32 MiB total;
`failure-names.txt` is capped at 4,096 records and 16 MiB;
`normalized-signatures.txt` is capped at 4,096 records and 256 MiB; every derived line is capped at
1 MiB; each JSON document is capped at 8 MiB; and `manifest.sha256` is capped at 1 MiB. Any overflow is
`evidence_limit_exceeded`, makes provenance ineligible, and is never silently truncated as valid.
The runner receives no carrier/credential input and adds no ambient environment to evidence.
Product-emitted raw Cargo output remains restricted evidence; its directory/file modes and hashes
are retained. `summary.json` is retained in that restricted evidence directory; the console emits
only the fixed wall-gate/invocation-ID line defined above.

All files are first written in the partial directory, flushed, and `fsync`ed. Evidence needed from
`R` is copied before cleanup, but `provenance.json` cannot contain `eligible=true` until the
authenticated worker has reported `ECHILD`, Stage B has exited and unmounted cleanly, every root
descriptor/path revalidation passes, the descriptor-relative root removal succeeds, pathname
absence is proved, the in-`M1` descriptors close, Stage A exits cleanly, `host_main` proves the
Stage-A process and namespace gone, and the retained host backing authority removes and proves
absence of the exact underlying mountpoint. Only `host_main` may then write the final
`eligible=true` provenance and close the evidence/backing authority. `manifest.sha256` contains
lowercase SHA-256 hex, two spaces, the exact
basename, and LF for every other final file in bytewise basename order. After every file and the
partial directory are `fsync`ed, `renameat2(RENAME_NOREPLACE)` atomically gives the whole directory
its final basename and the evidence parent is `fsync`ed. An unavailable no-replace syscall,
preexisting final name, or replacement is `evidence_write_failed`; no overwrite-capable rename
fallback is allowed. A `.partial` directory is diagnostic debris only and never a provenance
record.

#### Exact provenance schema

`provenance.json` is UTF-8 JSON with sorted keys, no duplicate keys, LF termination, schema
`substrate.canonical_shell_wall_provenance.v1`, and exactly these keys. Invalid CLI (exit `64`) and
an unsafe/unwritable evidence parent (the corresponding exit `67`) produce no final evidence
directory. Evidence-finalization failure (exit `71`) may leave only `.partial` diagnostic debris.
Every other exit except provenance-eligible result mismatch/invalid analysis (`69`) must finalize
an ineligible staged record if the already validated evidence parent remains writable. Exit `69`
must instead finalize `eligible=true`, `wall_gate=regression`, and `record_stage=complete`.
“Nullable” means JSON `null` is permitted only as stated; no unknown key is permitted.

| JSON path | Type / nullability | Exact contract |
|---|---|---|
| `schema` | string, nonnull | Exact schema string above |
| `record_state` | string, nonnull | Exact `final` |
| `record_stage` | enum, nonnull | Last completed stage: `preflight`, `setup`, `containment`, `cleanup`, or `complete` |
| `invocation_id` | string, nonnull | 32 lowercase hex |
| `label` | string, nonnull | Validated label |
| `mode` | enum, nonnull | `parallel` or `serial` |
| `eligible` | boolean, nonnull | May be true only at `record_stage=complete` after all containment, post-identity, cleanup, unmount, and absence gates |
| `wall_gate` | enum, nonnull | `clean`, `regression`, or `ineligible` |
| `ineligibility_reasons` | array of enum, nonnull | Empty iff eligible; sorted unique values from the closed enum below |
| `runner_exit_code` | integer, nonnull | One exit code from the table above and precedence rule below |
| `started_at_utc`, `finalized_at_utc` | RFC 3339 UTC string; `finalized_at_utc` nullable until finalization | Invocation start and final-record times |
| `request` | object, nonnull | Exact keys `launcher`, `command` (string array), `environment_contract`, `tmpdir`, `xdg_runtime_dir`, `target_dir` (absolute strings or null until root setup), `snapshot_timeout_seconds` (exact `900`), `setup_timeout_seconds` (exact `60`), `wall_timeout_seconds` (requested integer). `launcher` has only `trust_model` (exact `immutable-root-platform-plus-reviewed-git-object`), `controller` (exact `python-v2-host-main`), `authority_commit_oid` (40 lowercase hex), `reviewed_authority_commit_oid` (40 lowercase hex that must exactly equal `authority_commit_oid`), `python_path`, `bootstrap_sha256`, `runner_blob_sha256`, `bubblewrap_path`, `bubblewrap_version`, `bubblewrap_sha256`, `bubblewrap_elf_closure_manifest_sha256`, `host_argv_template_sha256`, `stage_a_argv_template_sha256`, `stage_b_argv_template_sha256` (always 64 lowercase hex), `host_argv_sha256` (64 lowercase hex, nullable until the host argv has been recorded), `stage_a_argv_sha256`, `stage_b_argv_sha256` (64 lowercase hex, nullable only while `mounts.stage_a` / `mounts.stage_b` are null; once the corresponding stage record exists each value must be non-null and equal that stage record's `argv_sha256`), `authority_commit_matches_live_head`, `authority_commit_trailers_verified`, `reviewed_oid_matches`, `direct_array_spawn`, `sealed_stage_a_runner`, `sealed_stage_b_runner`, `nested_probe_clean` (booleans). `reviewed_oid_matches` must be exact `true`, and the validator recomputes the `authority_commit_oid == reviewed_authority_commit_oid` equality even while `repository` is null before preflight completes. `environment_contract` has only `inherited_entry_count` (integer), `inherited_key_names_sha256` (64 lowercase hex), `forbidden_startup_keys_absent`, `tmpdir_overridden`, `xdg_runtime_dir_overridden`, `all_other_entries_byte_equal`, `protected_values_not_recorded` (booleans) |
| `repository` | object or null | Null only before successful repository preflight; otherwise exact keys `branch`, `head`, `tree`, `cwd` (strings), `clean`, `alternates_absent`, `replace_refs_absent`, `grafts_absent`, `shallow_absent`, `local_config_allowlist_valid`, `worktree_config_absent`, `info_attributes_absent`, `tree_index_equal`, `ignore_sources_tracked`, `untracked_absent` (booleans), `gitdir_identity` (directory identity), `index_pre`, `index_post`, `head_file_pre`, `head_file_post`, `branch_ref_pre`, `branch_ref_post`, `local_config_pre`, `local_config_post`, `info_exclude_pre`, `info_exclude_post` (regular-file identities; each post value nullable only if failure prevents post-check), `tree_manifest_sha256`, `index_manifest_pre_sha256`, `index_manifest_post_sha256`, `ignore_sources_manifest_pre_sha256`, `ignore_sources_manifest_post_sha256`, `tracked_manifest_pre_sha256`, `tracked_manifest_post_sha256`, `untracked_paths_pre_sha256`, `untracked_paths_post_sha256`, `ignored_paths_pre_sha256`, `ignored_paths_post_sha256` (64 lowercase hex; post values nullable only if failure prevents post-check), `tracked_count` (integer) |
| `toolchain` | object or null | Null only before executable preflight; exact keys `platform`, `machine` (strings), `executables`, `python_runtime`, `rustup_resolution`, and `rust_toolchain_root` |
| `toolchain.executables` | array of eight objects | Ordered `bubblewrap`, `python`, `git`, `newuidmap`, `newgidmap`, `rustup`, `toolchain-cargo`, `toolchain-rustc`; exact keys `role`, `path`, `version`, `pre`, `post`. `pre`/`post` each use the executable identity schema below; `post` is null only if failure prevents post-check |
| `toolchain.python_runtime` | object | Exact keys `startup_tcb_trust_model` (exact `immutable-root-platform`), `startup_tcb_manifest_sha256`, `initial_modules_sha256`, `proc_maps_sha256` (64 lowercase hex), `ld_so_preload_absent`, `host_controller_verified`, `bubblewrap_launcher_verified`, `canonical_environment_preserved`, `git_executed_from_fd`, `git_object_chain_verified`, `late_imports_absent` (booleans), `flags` (map containing only the six exact flag/value pairs above), `sys_path` (the exact ordered three-string array above), `bootstrap_bytes` (positive integer fixed by the reviewed V2 artifact), `bootstrap_sha256` (that artifact's 64-lowercase-hex hash), `runner_blob_git_oid` (40 lowercase hex), `runner_blob_sha256`, `stdlib_manifest_sha256` (64 lowercase hex), `runner_file` (exact synthetic string), `runner_blob_matches_manifest` (boolean), `self_test_blob_git_oid`, `self_test_blob_sha256`, `self_test_file` (null for `host`; exact authenticated values for `self-test`), and `modules` (module-origin array sorted by module name) |
| `toolchain.rustup_resolution` | object | Exact keys `cargo_shim_pre`, `cargo_shim_post`, `rustc_shim_pre`, `rustc_shim_post` (symlink identities), `settings_pre`, `settings_post`, `repo_toolchain_file_pre`, `repo_toolchain_file_post` (regular-file identities), `rustup_version_stdout_sha256`, `rustup_version_stderr_sha256`, `cargo_proxy_version_stdout_sha256`, `rustc_proxy_version_stdout_sha256` (64 lowercase hex), `proxy_stderr_empty`, `canonical_selection` (booleans). Post identities are nullable only if failure prevents post-check |
| `toolchain.rust_toolchain_root` | object | Exact keys `source_path`, `source_pre`, `source_post` (directory identities; post nullable only if unavailable), `directory_count` (exact `26`), `file_count` (exact `213`), `symlink_count` (exact `0`), `regular_file_bytes` (exact `1071859799`), `manifest_pre_sha256`, `manifest_post_sha256` (exact pinned 64-lowercase-hex hash; post nullable only if unavailable) |
| module-origin entry | object | Exact keys `name` (string), `origin_kind` (`built-in`, `frozen`, or `file`), `path` (absolute string only for `file`, otherwise null), `pre`, `post` (regular-file identities only for `file`, otherwise null; `post` may be null only if failure prevents post-check) |
| executable identity | object, nonnull when present | Exact keys `dev`, `ino`, `uid`, `mode` (integers), `kind` (exact `regular`), `sha256` (64 lowercase hex), `path_matches_fd`, `owner_ancestor_acl_safe`, `executable` (booleans) |
| symlink identity | object, nonnull when present | Exact keys `dev`, `ino`, `uid`, `mode` (integers), `kind` (exact `symlink`), `target` (exact `rustup`), `path_matches_lstat`, `owner_ancestor_acl_safe` (booleans) |
| regular-file identity | object, nonnull when present | Exact keys `dev`, `ino`, `uid`, `mode` (integers), `kind` (exact `regular`), `sha256` (64 lowercase hex), `path_matches_fd`, `owner_ancestor_acl_safe` (booleans) |
| `namespace` | object, nonnull | Always present; exact keys `host_uid`, `wall_uid`, `host_pid`, `stage_a_bwrap_pid`, `stage_b_bwrap_pid`, `worker_stage_a_pid`, `host_start_time_ticks`, `stage_a_bwrap_start_time_ticks`, `stage_b_bwrap_start_time_ticks`, `worker_start_time_ticks`, `stage_a_pid_namespace_inode`, `stage_a_mount_namespace_inode`, `stage_b_pid_namespace_inode`, `stage_b_mount_namespace_inode` (integers or null until that stage), `uid_map`, `gid_map` (exact strings or null until captured), `stage_a_bwrap_pidfd_opened`, `stage_a_status_received`, `stage_b_bwrap_pidfd_opened`, `stage_b_status_received`, `worker_pidfd_opened`, `worker_peer_credentials_verified`, `worker_ready_received`, `control_path_absence_proved`, `worker_start_sent`, `worker_subreaper_verified`, `worker_echild_received`, `stage_b_exit_received`, `stage_a_exit_received`, `stage_a_namespace_absent` (booleans) |
| `mounts` | object, nonnull | Always present; exact keys `propagation_private` (boolean), `evidence_parent`, `backing_mountpoint`, `tmpfs`, `stage_a`, `stage_b`, `snapshots`, `cargo_home_runtime`, `proc` |
| `mounts.evidence_parent` | object | Exact `pre`, `post` directory identities; `post` is nullable only before cleanup |
| `mounts.backing_mountpoint` | object | Exact keys `path`, `underlying_pre`, `mounted`, `underlying_post` (directory identities; later values nullable by stage), `removed`, `name_absence_proved`, `descriptor_closed_after_absence` (booleans or null until cleanup) |
| `mounts.tmpfs`, `mounts.proc` | object or null | Null until that mount exists; otherwise exact keys `source`, `fstype`, `flags` (strings), `mount_id` (integer), `unmounted` (boolean or null until teardown) |
| `mounts.stage_a`, `mounts.stage_b` | object or null | Null until that stage mount exists; otherwise exact keys `argv_template_sha256`, `argv_sha256` (64 lowercase hex and required to match the corresponding `request.launcher.stage_{a,b}_argv_template_sha256` / `request.launcher.stage_{a,b}_argv_sha256` values), `user_namespace`, `mount_namespace`, `pid_namespace`, `json_status_fd`, `as_pid_1`, `builtin_pid1_fail_safe_only` (booleans; Stage A has `as_pid_1=false`/`builtin_pid1_fail_safe_only=true`, Stage B has `as_pid_1=true`/`builtin_pid1_fail_safe_only=false`; the other four are true for both), `uid_map`, `gid_map` (strings), `private_propagation`, `die_with_parent`, `completed`, `unmounted` (booleans) |
| `mounts.snapshots` | array of zero to three objects | Ordered prefix of `repository`, `rustup-home`, `cargo-home-seed`; exact keys `role`, `source_manifest_sha256`, `snapshot_manifest_sha256` (64 lowercase hex), `source_entry_count`, `snapshot_entry_count` (integers), `private_tmpfs`, `no_host_alias`, `stage_b_read_only`, `post_manifest_equal`, `removed` (booleans or null by stage). Repository and rustup-home have `stage_b_read_only=true`; Cargo-home seed is not mounted into Stage B and therefore has `stage_b_read_only=false`. Rustup-home hashes bind exact settings plus the pinned toolchain manifest; repository hash derives from the independently verified expected tree; Cargo-home seed binds exact rustup bytes, two shim symlinks, and Cargo.lock/checksum-selected registry subset while proving all three authorized runtime metadata paths absent |
| `mounts.cargo_home_runtime` | object or null | Null until copied; otherwise exact keys `seed_manifest_sha256`, `pre_manifest_sha256`, `post_manifest_sha256` (64 lowercase hex), `seed_entry_count`, `pre_entry_count`, `post_entry_count` (integers), `private_tmpfs`, `no_host_alias`, `stage_b_writable`, `seed_copy_equal`, `only_authorized_metadata_changed`, `removed` (booleans or null by stage), `authorized_mutable_paths` (exact ordered array `[".global-cache",".package-cache",".package-cache-mutate"]`), and `changed` (sorted unique subset of that ordered allowlist) |
| directory identity | object, nonnull when present | Exact keys `dev`, `ino`, `mount_id`, `uid`, `mode` (integers), `kind` (exact `directory`), `acl_sha256` (64 lowercase hex), `foreign_effective_write`, `path_matches_fd` (booleans) |
| `containment` | object, nonnull | Always present; exact keys `cargo_pid`, `cargo_exit`, `stage_b_exit`, `stage_a_exit` (integers or null if never started/not yet exited), `reaped_descendants`, `event_count`, `output_pipe_dev`, `output_pipe_ino`, `output_bytes_preserved` (integers or null until observed), `stage_b_setup_timed_out`, `wall_timed_out`, `premature_stage_b_exit`, `echld_observed`, `authenticated_empty_record`, `forced_teardown`, `event_overflow`, `output_pipe_opened_before_stage_b`, `single_pipe_for_stdout_stderr`, `original_stdin_preserved`, `cargo_control_fd_absent`, `output_eof_after_stage_b_reap`, `output_overflow`, `stage_a_namespace_absent` (booleans), `events_file` (string or null before creation) |
| `roots` | array of zero to nine objects | Present roles in fixed order `root`, `tmp`, `xdg-runtime`, `control`, `target`, `repository-snapshot`, `rustup-home-snapshot`, `cargo-home-seed`, `cargo-home-runtime`; each has exact keys `role`, `path`, `pre`, `post`, `removed`, `name_absence_proved`, `descriptor_closed_after_required_absence`; `post` and the three booleans are null until that phase |
| `roots[].pre`, `roots[].post` | directory identity or null | `pre` nonnull for every listed role. Child/snapshot post is immediately before its removal; root post is after all child/snapshot absence and immediately before root removal |
| `evidence` | array of zero to five objects | Present artifacts in fixed order `cargo.log`, `containment.jsonl`, `failure-names.txt`, `normalized-signatures.txt`, `summary.json`; exact keys `name`, `bytes`, `sha256`. Presence follows the stage rules above. A result-complete record has all five; invalid analysis has either both derived text files after successful derivation or neither after failed derivation, never only one. `manifest.sha256` binds every present artifact plus `provenance.json` |
| `result` | object or null | Null before Cargo-log analysis; otherwise exact keys `analysis_status`, `analysis_reasons`, `cargo_exit_code`, `final_status`, `discovered`, `passed`, `failed`, `ignored`, `failure_name_count`, `failure_names_sha256`, `signature_count`, `normalized_signatures_sha256`, `raw_log_sha256`, `canonical_match`, `transitions` |
| `result.analysis_status` | enum | `complete` or `invalid`; invalid forces `canonical_match=false`, `wall_gate=regression`, and exit `69` when provenance is otherwise eligible |
| `result.analysis_reasons` | sorted unique enum array | Values from `missing_final_result`, `invalid_result_grammar`, `cargo_exit_status_mismatch`, `failure_count_mismatch`, `duplicate_failure_name`, `missing_panic_block`, `duplicate_panic_block`, `invalid_panic_header`, `invalid_encoding`, `nul_byte` |
| `result.cargo_exit_code`, `result.final_status` | integer / nullable enum | Cargo exit is always the exact collected integer. Final status is `FAILED` or `ok` after a valid final-result selection and is null iff no valid final-result line was selected. `FAILED` is valid only with code `101`, and `ok` only with code `0`; any other nonnull combination is invalid analysis |
| `result` counts/hashes | integer/string or null | Nonnegative integers and 64-lowercase-hex hashes when analysis is complete; otherwise nullable exactly for the unavailable artifact. Whenever counts are nonnull, `discovered` is defined exactly as `passed + failed + ignored` |
| `result.canonical_match` | boolean | True only when analysis is complete, Cargo exit is exact `101`, final status is exact `FAILED`, counts are exact `1309/1264/45/0`, failure-name and signature counts are each `45`, and both artifact hashes are canonical |
| `result.transitions` | object | Exact keys `pass_to_fail`, `new_fail`, `fail_to_changed_failure`, `fail_to_pass`, `removed`, `renamed_or_substituted`, `newly_ignored`, `weakened`; every value is null because the runner accepts no historical comparison artifact. The renewed closeout computes these only against its separately authenticated baseline inventory |

The closed ineligibility-reason enum is:
`environment_unavailable`, `invocation_authority_invalid`, `repository_identity_mismatch`,
`repository_dirty`,
`python_startup_tcb_invalid`, `executable_identity_invalid`, `executable_identity_drift`,
`safe_parent_rejected`, `snapshot_identity_invalid`, `snapshot_timeout`,
`cargo_home_runtime_drift`, `root_validation_failed`, `namespace_setup_failed`,
`stage_a_liveness_failed`, `stage_b_setup_timeout`, `stage_b_liveness_failed`,
`premature_stage_b_exit`, `subreaper_unverified`, `control_channel_invalid`,
`containment_timeout`, `surviving_descendant`, `worker_handshake_failed`,
`output_capture_failed`, `diagnostic_scope_violation`, `evidence_limit_exceeded`,
`root_identity_drift`, `unexpected_mount`, `acl_drift`, `mount_teardown_failed`,
`cleanup_identity_mismatch`, `cleanup_failed`, `evidence_write_failed`, and
`internal_invariant_failed`. Result mismatch is not an ineligibility reason: a provenance-valid
mismatch or invalid result analysis produces `eligible=true`, `wall_gate=regression`, and exit
`69`.

If multiple failures occur, exit precedence is `72 > 71 > 70 > 68 > 67 > 66 > 65 > 69 > 0`;
all applicable ineligibility reasons remain in the sorted array. Invalid result analysis is
recorded only in `result.analysis_reasons` and maps to `69`; output capture/evidence bounds map to `71`;
cleanup/mount teardown map to `70`; setup/root/mount construction map to `67`; repository or
executable preflight maps to `65`.

`summary.json` is the exact `result` object above. `containment.jsonl` has one sorted-key UTF-8
object per LF-terminated line with exact keys `seq`, `monotonic_ns` (nonnegative integers),
`pid_namespace_inode` (nonnegative integer or null before PID-namespace creation), `kind` (one of
`stage_a_started`, `stage_a_status_received`, `output_pipe_opened`, `stage_b_started`,
`stage_b_status_received`,
`worker_pidfd_opened`, `worker_peer_credentials_verified`, `worker_ready_received`,
`control_path_absence_proved`, `worker_subreaper_verified`, `worker_start_sent`, `cargo_started`,
`cargo_exited`, `child_reaped`, `stage_b_setup_timeout`, `wall_timeout`, `sigterm_sent`,
`sigkill_sent`, `echild_observed`, `containment_empty_received`, `premature_stage_b_exit`,
`stage_b_exited`, `output_pipe_eof`, `stage_a_exited`, `stage_a_namespace_absent`, `teardown_started`,
`teardown_finished`), and `pid`, `ppid` (integer or null), `state` (one kernel state byte or
null). It contains no other string.

#### Exact log-summary compatibility contract

`summarize_test_log` reproduces the prior canonical analyzer inside the tracked runner; it may not
invoke or import the untracked analyzer. The self-test file embeds one fixed UTF-8 Cargo-log byte
fixture and the exact expected `failure-names.txt`, `normalized-signatures.txt`, `summary.json`,
and three SHA-256 byte strings as literals; changing any parser byte requires docs-first
reauthorization. The implementation follows this complete grammar:

1. input is LF-delimited strict UTF-8; a NUL, undecodable sequence, or CR-only record makes
   analysis invalid and exit `69` when provenance is otherwise eligible. A line/artifact bound
   overflow or failure to write the required derived LF is `evidence_limit_exceeded`/
   `evidence_write_failed`, provenance-ineligible, and exit `71`;
2. a result line must match from byte 0 through LF:
   `^test result: (FAILED|ok)\. (0|[1-9][0-9]{0,9}) passed; (0|[1-9][0-9]{0,9}) failed; (0|[1-9][0-9]{0,9}) ignored; (0|[1-9][0-9]{0,9}) measured; (0|[1-9][0-9]{0,9}) filtered out;.*\n$`;
   every integer must be `<= 2147483647`, `ok` requires failed count zero, and `FAILED` requires a
   positive failed count. Exactly as the prior `tail -n 1` analyzer, the last matching result line
   is selected and earlier matching result lines are ignored. The selected status must agree with
   the collected Cargo exit: `FAILED` requires exact exit `101` and `ok` requires exact exit `0`;
   disagreement is `cargo_exit_status_mismatch`, invalid analysis, and exit `69` when provenance
   is otherwise eligible. `discovered` is computed only as the checked integer sum
   `passed + failed + ignored`; measured and filtered-out counts never contribute;
3. an exact `failures:\n` resets the candidate-name list and makes it active; until another exact
   reset or the selected result, any line matching `^    .*::.*\n$` contributes the bytes after
   the first four spaces. The active list immediately preceding the selected result is used,
   must contain exactly the selected failed count, and must contain no empty or duplicate name;
   an `ok` result produces exact zero-byte names/signatures files without requiring a failure
   section;
4. bytewise-sort the unique names and write each exact UTF-8 name plus LF, with no header or blank
   trailer. On a canonical log this is byte-identical to the prior analyzer;
5. a panic header must match the entire line
   `^thread '([^'\n]+)' panicked at ([^\n]+):\n$`. For every wanted name there must be exactly one
   header; a missing/duplicate wanted header or malformed would-be wanted header is invalid.
   Every valid header first flushes the active capture, then starts capture only for a wanted,
   unseen name; unwanted headers are otherwise ignored;
6. capture nonblank body lines after a wanted header until a line starts exactly `test `,
   `[codex-`, `running `, `test result:`, or `error:`, or equals `FAILED\n` or `failures:\n`.
   A line beginning exactly `note: run with` is discarded. EOF flushes the active block;
7. remove each input LF, join captured body lines with one ASCII space, replace
   `aos_[0-9a-f]+` with `aos_<id>`, replace `.tmp[A-Za-z0-9]+` with `.tmp<id>`, collapse only
   ASCII `[ \t\r\n\f\v]+` to one space, then remove at most the one collapsed leading/trailing
   space;
8. emit exact UTF-8 `NAME<TAB>WHERE<TAB>NORMALIZED_BODY<LF>`, bytewise-sort the complete unique
   records, require signature count equal failure-name count, and write no header/blank trailer;
9. compute SHA-256 over exact raw bytes of `cargo.log`, `failure-names.txt`, and
   `normalized-signatures.txt`, encoded as lowercase hex.

The embedded golden byte literals are exactly:

```python
GOLDEN_LOG = b"running 2 tests\ntest alpha::case ... FAILED\ntest zeta::case ... FAILED\n\nfailures:\n\n---- alpha::case stdout ----\nthread 'alpha::case' panicked at crates/shell/src/a.rs:10:2:\nleft  aos_deadbeef\n right: .tmpABC\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\nFAILED\n---- zeta::case stdout ----\nthread 'zeta::case' panicked at crates/shell/src/z.rs:20:4:\n spaced    body\nsecond\tline\nfailures:\n    zeta::case\n    alpha::case\n\ntest result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 1307 filtered out; finished in 0.01s\n"
GOLDEN_NAMES = b"alpha::case\nzeta::case\n"
GOLDEN_SIGNATURES = b"alpha::case\tcrates/shell/src/a.rs:10:2\tleft aos_<id> right: .tmp<id>\nzeta::case\tcrates/shell/src/z.rs:20:4\tspaced body second line\n"
```

Their byte lengths/SHA-256 values are respectively `552` /
`80e7c12e6f3711bc85d8b6871e7d8e68442634f6883b04257ae14d9618256b57`, `23` /
`375dafeeb68560e1316881e277d0242b20d3bfa50585a8ae4279de03f748e91f`, and `131` /
`df36b0aca2ed2d7e5d7efdc37bb4577960dcf643d77d120fa0365d6b8779f634`.
The exact sorted-key compact `summary.json` is:

```json
{"analysis_reasons":[],"analysis_status":"complete","canonical_match":false,"cargo_exit_code":101,"discovered":2,"failed":2,"failure_name_count":2,"failure_names_sha256":"375dafeeb68560e1316881e277d0242b20d3bfa50585a8ae4279de03f748e91f","final_status":"FAILED","ignored":0,"normalized_signatures_sha256":"df36b0aca2ed2d7e5d7efdc37bb4577960dcf643d77d120fa0365d6b8779f634","passed":0,"raw_log_sha256":"80e7c12e6f3711bc85d8b6871e7d8e68442634f6883b04257ae14d9618256b57","signature_count":2,"transitions":{"fail_to_changed_failure":null,"fail_to_pass":null,"new_fail":null,"newly_ignored":null,"pass_to_fail":null,"removed":null,"renamed_or_substituted":null,"weakened":null}}
```

It is `672` bytes including its LF and has SHA-256
`b63d9a162a47026329f8b018398418632b60e05366e74f080da46673b7cd461f`.

The canonical expectations embedded in P1 are failure-name count `45`, SHA-256
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`, signature count `45`,
and SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.

The only canonical wall commands are:

```bash
TMPDIR="$R/tmp" XDG_RUNTIME_DIR="$R/xdg-runtime" \
  cargo test -p shell --lib -- --nocapture
```

and:

```bash
TMPDIR="$R/tmp" XDG_RUNTIME_DIR="$R/xdg-runtime" \
  cargo test -p shell --lib -- --nocapture --test-threads=1
```

#### P1 self-test gate

`CanonicalShellWallRunnerTests` shall contain:

- `test_rejects_cargo_or_bwrap_exit_with_live_descendant`
- `test_reaps_reparented_descendant`
- `test_process_group_escape_remains_contained`
- `test_timeout_is_ineligible`
- `test_snapshot_timeout_is_ineligible`
- `test_copy_tree_no_follow_preserves_modes_despite_umask`
- `test_rejects_root_path_replacement`
- `test_rejects_root_symlink_substitution`
- `test_rejects_child_entry_replacement`
- `test_rejects_owner_drift`
- `test_rejects_mode_drift`
- `test_rejects_acl_drift`
- `test_rejects_descriptor_path_inode_mismatch`
- `test_rejects_premature_descriptor_closure`
- `test_rejects_deletion_before_containment_empty`
- `test_preserves_unrelated_sentinel`
- `test_rejects_shared_or_reused_root`
- `test_rejects_sticky_world_writable_ancestor`
- `test_rejects_unexpected_mount_id`
- `test_rejects_unexpected_entry_type`
- `test_rejects_subreaper_readback_failure`
- `test_rejects_bwrap_status_or_worker_ready_handshake_failure`
- `test_rejects_namespace_prerequisite_fallback`
- `test_teardown_never_targets_unrelated_process`
- `test_stage_b_setup_timeout_signals_only_retained_bwrap_pidfd`
- `test_stage_b_timeout_before_status_starts_no_cargo_and_reaps_exact_bwrap`
- `test_stage_b_bwrap_death_before_worker_ready_is_ineligible`
- `test_stage_a_maps_vanished_post_status_worker_to_premature_exit`
- `test_stage_a_parent_loss_before_stage_b_start_starts_no_wall`
- `test_stage_b_bwrap_loss_after_ready_before_start_is_ineligible`
- `test_stage_a_start_requires_bwrap_status_worker_pidfd_peer_credentials_and_ready`
- `test_stage_a_loss_before_stage_b_launch_starts_no_wall`
- `test_stage_a_loss_before_worker_ready_terminates_bwrap_and_worker`
- `test_stage_a_loss_after_ready_before_start_terminates_bwrap_and_worker`
- `test_stage_a_loss_after_start_invalidates_and_terminates_wall`
- `test_premature_stage_b_exit_after_start_is_ineligible`
- `test_stage_b_complete_requires_authenticated_echild_record_and_zero_exit`
- `test_fixed_private_socket_peer_status_cloexec_unchanged_stdin_and_no_cargo_control_fd`
- `test_sigterm_ignoring_descendant_requires_ineligible_sigkill`
- `test_stuck_stage_b_hits_bounded_stage_a_deadline`
- `test_rejects_forbidden_glob_ambient_parent_and_product_home_targets`
- `test_rejects_pathname_only_cleanup_api`
- `test_diagnostics_omit_comm_cmdline_environment_and_markers`
- `test_cli_rejects_arbitrary_command_root_and_environment_overrides`
- `test_public_request_cannot_select_internal_host_stage_worker_roles_or_control_path`
- `test_cli_rejects_invalid_label_timeout_cwd_and_head`
- `test_rejects_dirty_index_worktree_and_nonignored_untracked_state`
- `test_ignores_only_git_ignored_build_outputs`
- `test_rejects_git_environment_config_alternates_replace_and_fsmonitor`
- `test_descendant_authority_commit_must_be_ancestor_with_exact_runner_and_test_blobs`
- `test_descendant_authority_rejects_non_ancestor_and_authenticated_blob_drift`
- `test_rejects_local_git_config_include_mode_exclude_worktree_and_unknown_keys`
- `test_info_exclude_is_held_but_never_cleanliness_authority`
- `test_rejects_info_attributes_worktree_config_and_index_authority_extensions`
- `test_rejects_tree_index_stage_mode_flag_and_worktree_byte_drift`
- `test_tracked_gitignore_is_only_untracked_exclusion_authority`
- `test_rejects_untracked_gitignore_and_every_nonignored_untracked_path`
- `test_rejects_path_shim_and_executable_identity_drift`
- `test_executes_and_revalidates_held_bwrap_python_git_rustup_cargo_and_rustc`
- `test_trusted_bwrap_launcher_rejects_ambient_loader_and_startup_tcb_drift`
- `test_stage_b_as_pid1_matches_json_child_pid_peer_credentials_and_echild`
- `test_held_git_fd_path_replacement_never_executes_replacement`
- `test_isolated_python_startup_rejects_environment_site_path_and_module_poison`
- `test_python_runtime_rejects_origin_and_late_import_drift`
- `test_host_controller_bootstrap_commit_templates_and_slots_avoid_self_reference`
- `test_bootstrap_rejects_path_execution_cmdline_drift_and_runner_blob_mismatch`
- `test_self_test_loader_authenticates_exact_runner_and_test_blobs`
- `test_self_test_loader_rejects_path_import_discovery_and_blob_mismatch`
- `test_private_rustup_toolchain_snapshot_matches_pinned_complete_manifest`
- `test_mounted_rustc_reports_private_sysroot_and_compiles_minimal_test`
- `test_vendor_snapshot_reconstructs_only_lock_checksum_selected_archives`
- `test_constructed_cargo_home_resolves_locked_vendor_offline_without_drift`
- `test_private_snapshots_ignore_transient_source_modify_replace_and_restore`
- `test_cargo_environment_preserves_e0_with_only_tmpdir_xdg_overrides`
- `test_environment_values_and_protected_markers_are_never_serialized`
- `test_fresh_private_target_never_reads_repository_target`
- `test_private_target_is_descriptor_removed_after_containment`
- `test_cargo_home_seed_runtime_confines_lock_metadata_and_is_removed`
- `test_rejects_project_ancestor_or_account_cargo_config_appearance`
- `test_evidence_bounds_fail_closed`
- `test_schema_rejects_unknown_missing_and_wrong_type_fields`
- `test_schema_encodes_preflight_setup_containment_cleanup_failures`
- `test_atomic_finalization_hides_partial_eligibility`
- `test_final_evidence_rename_is_noreplace`
- `test_summarizer_golden_failure_names_and_signatures`
- `test_summarizer_normalization_sort_and_newline_bytes`
- `test_summarizer_last_result_and_failure_section_grammar`
- `test_summarizer_accepts_valid_all_pass_as_eligible_regression`
- `test_summarizer_rejects_cargo_exit_status_mismatch`
- `test_summarizer_rejects_integer_duplicate_missing_and_panic_grammar`
- `test_summarizer_rejects_invalid_encoding_nul_and_overflow`
- `test_mount_source_flags_ids_propagation_and_order_are_exact`
- `test_projected_root_ids_include_mapped_stage_identity`
- `test_cache_and_proc_mounts_unmount_before_tmpfs`
- `test_backing_mountpoint_removed_and_evidence_parent_preserved`
- `test_parallel_command_and_fresh_root`
- `test_serial_command_and_fresh_root`
- `test_stage_a_output_pipe_preserves_combined_order_eof_backpressure_bounds_and_hash`
- `test_success_removes_root_under_continuous_descriptor_authority`
- `test_success_preserves_unrelated_files`
- `test_host_finalizes_eligible_only_after_stage_a_exit_namespace_and_backing_absence`

The fixed-socket and output-pipe tests use real pinned Bubblewrap mechanics with bounded dummy
children. They prove one pipe write description is duplicated only to Stage-B fd 1/2, original
stdin survives, the worker control endpoint is close-on-exec and absent in Cargo, alternating and
larger-than-`PIPE_BUF` stdout/stderr bytes reproduce the exact captured stream/hash, a deliberately
retained writer prevents EOF and eligibility, collector backpressure does not deadlock, and the
first byte over the cap irreversibly selects bounded ineligible diagnostic handling.

Mechanics tests use bounded dummy subprocesses, not the product wall. After every self-test and
fresh P1 security/provenance review is CLEAN, a separate baseline phase must run three parallel
walls and one serial wall with four distinct roots. The old four walls are never inputs.

### RP5 publication and sequencing gate

This exact six-file docs packet may edit only these six control-pack Markdown files. RP3 is
complete. RP4 product proof on exact integration commit/tree
`8c46135c861a468dea316cf9fd7d6c6bb15bddac` /
`5358497a8baec6f36e15aaef58415a759e64977d` is accepted clean by human disposition; the raw
persistence review remains `REQUEST_CHANGES` for three cache-only process-audit findings preserved
as non-blocking debt outside RP4 product-proof scope. This exact six-file change is the bounded
RP5 closeout packet.

At that preserved RP5 checkpoint, the docs-on-top -> one ordinary fast-forward publication model
was unchanged. Source publication had not yet occurred in that run, and no local/upstream/remote
parity was claimed there. After the reviewed/committed RP5 docs, the exact next step was one
ordinary fast-forward source publication. That publication later completed, after which the R2-3
suffix and refreshed native evidence closed R2-3; R2-4 and R3 remain later. No historical result
is rewritten as passing.

**Source provenance:**
- extracted from [`04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract`](../04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract), lines 6843–7018; baseline span SHA-256 `5e719be01a55a3fb1a47e5c657186da0a0a582d61ea5f39e5667bd1b2b6609f3`
- extracted from [`04-contracts-and-gates.md#normative-renewed-r2-2-publication-contract`](../04-contracts-and-gates.md#normative-renewed-r2-2-publication-contract), lines 7085–8672; baseline span SHA-256 `ff67d14b1f49cc9232ef3ace2713fba07965a569ef26e581115a6ef1f09bb7d2`
