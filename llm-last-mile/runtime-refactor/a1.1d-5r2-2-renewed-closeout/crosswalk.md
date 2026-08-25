**Kind:** crosswalk
**Stable ID:** `A1.1d-5R2-2-renewed-closeout-family`
**Canonical for:** A1.1d-5R2-2 renewed closeout crosswalk
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** [`02-seam-crosswalk.md#broad-wall-invocation-provenance-crosswalk`](../02-seam-crosswalk.md#broad-wall-invocation-provenance-crosswalk) lines 1368–1504
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2 renewed closeout crosswalk

## Broad-wall invocation provenance crosswalk

This is an evidence-authority correction, not a new target seam or implementation owner. Every
live broad shell wall, parallel/serial wall, canonical or differential baseline wall, final
change-detection wall, renewed closeout wall, and F/Harness wall referenced anywhere in this
crosswalk uses the
[canonical shell-library broad-wall invocation contract](../04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract).

| Evidence boundary | Current classification | Required action | Forbidden inference | Status after B1 |
|---|---|---|---|---|
| Shell-library broad-wall invocation provenance | Missing from the first renewed-closeout prompt; runtime/tests unchanged | Create a fresh validated private `R`, bind exact `R/tmp` and `R/xdg-runtime` through `TMPDIR` and `XDG_RUNTIME_DIR` before Cargo starts, and retain the complete root/command/toolchain/result/hash provenance | Ambient `/tmp`, an invalid root, a missing variable, or incomplete provenance cannot establish product, environment-drift, isolation, baseline, or serial-only truth | Contract corrected in B1 docs; proof was then unrun; the later four matching walls are separately ineligible; no seam promotion |
| Provenance-valid canonical baseline | `1309/1264/45/0`; names `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; signatures `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` | Preserve as the comparison authority for the complete future wall | Do not replace, add the 59 mismatches, waive a failure, weaken trusted-root validation, or infer from totals alone | Frozen |
| Omitted-root renewed-closeout wall | `1309/1205/104/0`; names `7d6564bbcfaaed1fdb784eb4610683a02e4464b362215761d728be3e7361d8e5`; signatures `ae67f87fbdc155575e13467506405afbd4568fc08b9058cb4159daa539549579` | Historical B1 disposition: retain separately as `BaselineCommandMismatchConfirmed`, correct the invocation, and defer rerun. Current ordering is R1 → P1 → fresh P1 baseline before a further renewed closeout | The 59 additional trusted-root rejections are not product failures, environment drift, test isolation, or baseline additions | Ineligible historical evidence only |

The root contract preserves the private-home security boundary: `/tmp` is normally mode `1777`,
so correct trusted-root validation rejects it as an ancestor. `SUBSTRATE_TEST_TRUSTED_PARENT` is
test-harness placement only, cannot select product authority, and cannot weaken owner/mode/type/
identity/ancestor/ACL checks. Cleanup touches only the revalidated disposable `R` after every child
exits and after logs/hashes are preserved; real `SUBSTRATE_HOME` and user/product state are untouched.

## Renewed R2-2 publication crosswalk

The selected model is **Docs-on-top → one fast-forward publication**.

| Publication node | Bound inputs and outputs | Required ordering | Forbidden disposition | Gate/status |
|---|---|---|---|---|
| Publication-authority correction | Source remote F docs closeout `2f6f1f69b3519dafff01ef543e7d260da2c37700`; exact unchanged 17-commit runtime range ending `e5fbd2d4441d248e137d52c44e493fb0abe158f8`; publication-authority commit `928f94e7b4c498273b40385f7bffea9e4f949700` | Keep the source branch unpushed; retain exact local/remote preservation at `feat/preserve-a1-1d-5r2-2-publication-authority-20260722` | Any replay, rebase, rewrite, cherry-pick, merge commit, force push, runtime/test change, or source publication | Complete and preserved historical predecessor to the current RP4/RP5 disposition |
| B1 broad-wall invocation correction | Exact publication-authority parent `928f94e7b4c498273b40385f7bffea9e4f949700`; its six-file documentation successor; no implementation change | Historical completed action: committed locally on top, kept the source branch unpushed, and preserved at `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722` | Replaying or rewriting the authority/runtime range; treating B1 as current; pushing source; changing code/tests/fixtures/scripts/schemas/dependencies/generated files | Complete and preserved historical predecessor to the current RP4/RP5 disposition |
| Remediation-planning authority | Exact B1 commit `f7ded83ef147b748678ba6b028eea959870a04fe`; the two source-closed blockers; exactly six reviewed control-pack files | Commit the planning bytes directly on `f7ded83`; preserve only at `feat/preserve-a1-1d-5r2-2-remediation-planning-20260723`; keep source remote unchanged | Any implementation, wall run, source push, history rewrite, or classification of the prior walls as eligible | Historical RP0 checkpoint; superseded by completed RP3 and the current RP4/RP5 closeout disposition |
| R1 implementation/review | Reviewed planning commit; exact two-file R1 allowlist and fixed display/rejection contract | One bounded R1 commit, focused proof, then fresh authority/security review CLEAN | P1 work, broader caller/helper changes, carrier removal, weaker validation, or source push | Ordered after planning and before P1 |
| P1 implementation/review | R1 implementation/review CLEAN; exact two-file P1 allowlist and authenticated tracked-runner contract | One bounded P1 commit, all adversarial self-tests, then fresh provenance/security review CLEAN | Product runtime/test semantics, dependencies, privileged state, fallback harness, or source push | Ordered after R1 and before any new wall |
| Fresh canonical baseline | Reviewed P1 commit; three fresh parallel roots and one fresh serial root | Run all four walls only through P1; require independently valid provenance, canonical counts/hashes, and zero differential | Reuse of any prior wall/root, parent-only waiting, pathname-only cleanup, or implementation remediation | Evidence node, not an implementation commit |
| Renewed production-fix-free closeout | R1/P1 implementation and reviews CLEAN plus the fresh P1 baseline; complete focused/integration wall and four independent read-only reviews | Run closeout proofs, then reviews, with no implementation/test commit during the closeout itself | Repairing production, tests, fixtures, scripts, schemas, dependencies, or generated files during closeout; claiming product smoke or seam promotion | Failure or stop leaves source remote unchanged and requires docs-first future remediation |
| Final integration-closeout docs | Exact already assembled runtime range plus the RP4 proof packet accepted clean by human disposition; final six-file RP5 closeout commit | Follow the range and evidence it certifies; draft/validate/review to CLEAN, commit the exact reviewed bytes, then rerun final validation/reviews | Placing closeout docs below the certified range, treating authority docs as proof, rewriting a finding, or publishing on any unresolved finding | Preserved RP5 packet. The raw persistence review remains `REQUEST_CHANGES`; its three cache-only findings are preserved as non-blocking process-audit debt outside RP4 product-proof scope. At that checkpoint, source publication was the next step; it later completed before the accepted R2-3 suffix and refreshed native evidence closed R2-3. |
| Source publication | Original 17 runtime commits plus publication-authority docs, B1 broad-wall invocation docs, remediation-planning docs, the bounded R1 and P1 implementation commits, and final integration-closeout docs | Freshly read the source remote at exact `2f6f1f69b3519dafff01ef543e7d260da2c37700`; bind an explicit expected-old-OID CAS/lease to that exact OID; independently prove the update is an ordinary fast-forward; require final local HEAD, upstream, and remote parity | Using the CAS/lease to permit a forced/non-fast-forward update; push after a changed old OID; second/staged source push; replay, rebase, rewrite, cherry-pick, merge commit, force push; or admission of blocked/preservation-only donor ancestry | At that checkpoint, source publication had not yet occurred; after the reviewed/committed RP5 docs it was the exact next step and later completed without rewrite. |

The earlier F and Route D replay procedures remain packet-specific historical evidence. The 17
runtime commits already rest on the latest F documentation, so another replay would change commit
identity without changing the crossed seams or their proof content. Every existing runtime commit
must remain byte- and identity-identical; preservation-only and blocked-donor commits remain
outside source ancestry.

## Closeout-remediation crosswalk

The following crosswalk is the historical RP0 source-closure authorization that governed the later
completed RP1/RP2 increments. Status `AUTHORIZED, NOT IMPLEMENTED` is preserved as checkpoint
history, not as the current state.

| Increment | Proven defect and source route | Exact future editable surface | Required proof | Frozen boundaries | Status |
|---|---|---|---|---|---|
| R1 — carrier non-disclosure | `scripts/substrate/install-substrate.sh::deploy_shims` passes `--install-bootstrap-context-v1 VALUE` to generic `run_cmd`; dry-run renders `$*` verbatim and disclosed the authenticated carrier | Existing production file: add `run_cmd_with_redacted_install_bootstrap_carrier`; change only `deploy_shims`. Existing test file: add exactly `assert_shim_dry_run_carrier_non_disclosure` and its top-level invocation in `tests/installers/prefix_propagation_r2_2.sh`; existing assertion helpers are reused unchanged | Fixed-placeholder dry-run; byte-exact live argv; boundary cases for spaces, quotes, newlines, Unicode, leading dashes, reordering, end-of-options, and missing/duplicate/malformed flags; inherited xtrace non-disclosure/restoration; stdout/stderr/log/trace/generated-file sentinel scan; unchanged full dry-run and uninstall compatibility run without editing them | `run_cmd`, `run_with_sudo`, parser/carrier construction, validation, commitment, prefix selection, install/uninstall semantics, shim behavior, credentials, and tests outside the named file are frozen | AUTHORIZED, NOT IMPLEMENTED |
| P1 — canonical provenance runner | Failed external harness waited only on Cargo, asserted descendant exit without proof, closed root descriptors, then used pathname `shutil.rmtree`; all four matching walls are ineligible | New test-harness files only: `scripts/ci/canonical_shell_wall_runner.py` and `scripts/ci/test_canonical_shell_wall_runner.py`, with the exact symbols/tests in `04-contracts-and-gates.md`; exact direct Python/V2 authenticated `host_main` controller under the installed OS/Python/Bubblewrap TCB; immutable P1-commit trailers bind independently verified bootstrap and host/Stage-A/Stage-B argv-template constants, while each invocation records its substituted actual hashes; held-FD Git/object verification; sealed `--ro-bind-data` runner projections; authenticated `stage_a_main` and `stage_b_worker_main` | Reviewed P1 OID plus commit-trailer/template/substituted-argv and TCB proof; Bubblewrap identity/nested/detached-descendant probes; held-Git-FD object-chain proof; authenticated in-memory 95-test loader; immutable repository/rustup/Cargo seeds plus separately writable Cargo runtime limited to `.package-cache`, `.package-cache-mutate`, `.global-cache`; fixed private `SOCK_SEQPACKET` worker path, matching `SO_PEERCRED`/child pidfd, close-on-exec and unchanged stdin; one explicit Stage-A-owned combined-output pipe with EOF/backpressure/bounds/hash proof and no Cargo control/evidence FD; Stage-B `--as-pid-1` worker subreaper/reap to `ECHILD`; Stage-A in-namespace deletion; retained host proof of Stage-A namespace teardown and underlying backing absence before final eligibility; then a new three-parallel/one-serial canonical baseline | No production shell/world/policy/service/lifecycle/capability/secure-FD symbol; no dependency; Linux only; no privileged daemon/service/product mutation; no mutable external invocation artifact, ambient shell, unnamed supervisor, tracked runner/test-path execution, mutable Git/object authority, host source/cache/toolchain execution, repository target authority, parent-only wait, assumed arbitrary control/evidence FD inheritance, Stage-A self-finalization, mutable local Git exclude/config authority, `pgrep -P`, sleep, one-time snapshot observation, glob, pathname-only cleanup, real product home, or Cargo/rustup selector override | AUTHORIZED, NOT IMPLEMENTED |

### P1 historical runner note and current authority

[The R2-3ZP2 runner refinement](../a1.1d-5r2-3/current-state.md#a11d-5r2-3-closeout-status)
is preserved as source-bound historical evidence only. The
tracked Python runner's authenticated `1322 discovered / 1277 passed / 45 failed / 0 ignored`
result remained provenance-ineligible and is diagnostic only, not current baseline authority.

For this exact source, `make shell-lib-wall` and `make shell-lib-wall-serial` are the sole
normative broad shell-wall entrypoints. They are Linux-only repo-root public commands that validate
a compact trusted private root, set private `TMPDIR` and `XDG_RUNTIME_DIR`, run the exact Cargo
argv, stream Cargo output unchanged, preserve the underlying Cargo recipe status through exact-root
cleanup, and leave GNU Make's standard public exit mapping intact. The historical 45-failure set
remains intact; only the three separately classified world-deps/report failures documented above
may appear in addition to it.

### R1 direct-caller inventory

| Direct `run_cmd` call at `f7ded83` | Value class | Disclosure disposition |
|---|---|---|
| `run_with_sudo`, no-sudo execution | Static current bare-tool set resolved through fixed privileged PATH, or the one permitted absolute ACL helper; account/group, package, artifact/TMP/destination path, mode/owner, unit/socket, and ACL operands | No authenticated carrier or other closed sensitive value class found; retain generic behavior |
| `run_with_sudo`, noninteractive sudo | Fixed sudo/env wrapper around that same source-closed current tool/operand set | No authenticated carrier or other closed sensitive value class found; retain generic behavior |
| `run_with_sudo`, interactive sudo | Fixed sudo/env wrapper around that same source-closed current tool/operand set; no-TTY error may render those original operands | No authenticated carrier or other closed sensitive value class found; retain generic behavior |
| `deploy_shims` | authenticated bootstrap carrier plus `--shim-deploy` | Sensitive; route through the dedicated structured display boundary |

The inventory also covered credentials, authorization values, commitment preimages,
prompt/request material, private-host selectors, dry-run output, live execution, error rendering,
xtrace, installer compatibility, and uninstaller compatibility. No second live sensitive
`run_cmd` caller was found. `run_with_sudo` accepts any bare tool resolvable through its fixed PATH
and forwards its operands; it is the finite current caller set—not a named bare-tool allowlist—that
was source-closed. The no-TTY error can render those operands, but no current caller supplies the
protected classes above; this packet does not broaden R1 to speculative callers.

Existing redaction helpers were source-closed and frozen. Linux
`world-provision.sh::show_install_context_cmd` is a `%q`-based display-only helper that redacts the
separate carrier and environment-assignment forms, but not the accepted equals form, and it does
not own missing/duplicate/malformed/end-of-options validation or byte-exact execution. Rust
`crates/common::{redact_sensitive, redact_process_argv}` are heuristic logging redactors, not an
exact Bash carrier grammar. Reusing or extending either surface would broaden the R1 allowlist.
The dedicated installer helper instead returns exact status `2` with exact stderr
`[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n` for every
structural rejection; valid live child status propagation remains unchanged.

### P1 source-closure decision

Repository shell/install test helpers can create fixtures but do not contain reparented
descendants. Runtime `trusted_fs` contains useful low-level ideas but is private production
authority. Product cgroups and service scripts require or mutate product state. The failed
prompt-local Python/shell harness is untracked and structurally insufficient. Extending any of
those would either leave the proof gap or exceed the no-product-change boundary, so one new
bounded tracked test runner is the selected form.

The selected unprivileged Linux mechanism was demonstrated to have the required local user,
private-mount, and PID namespace syscalls plus direct one-entry current UID/GID mappings. It needs
no external `unshare` program, but it does require exact current-account subordinate-ID entries in
`/etc/subuid` and `/etc/subgid` plus exact held `newuidmap`/`newgidmap` helpers under the startup
TCB. Therefore
`BroadWallContainmentDecisionRequired` is not active for this authorization. If those prerequisites
are missing at execution time, the runner fails closed as an environment/ineligible-wall result;
it never falls back to parent-only waiting.

The exact reviewed P1 commit OID binds its immutable bootstrap and three argv-template trailers
to independently verified runner-blob constants; each runtime instantiation records separate
actual argv hashes, and no external invocation artifact is authority. The exact topology
is host → Stage A → Stage B. Authenticated `host_main` remains
outside both Bubblewrap stages and retains backing/evidence authority through Stage-A process and
namespace disappearance. `stage_a_main` receives its authenticated blob through sealed
`--ro-bind-data`, constructs immutable repository/rustup/Cargo seeds plus a separately writable
Cargo runtime, and launches Stage B. The seed omits the three Cargo metadata files; only their
exact regular paths may be created or changed in the runtime. The worker connects to the fixed private
`R/control/worker.sock` projection; Stage A accepts only peer credentials matching the retained
Bubblewrap child pidfd, unlinks the pathname before START, and ensures Cargo inherits original
stdin plus only one explicit Stage-A-owned stdout/stderr capture pipe and no control/evidence FD.
Stage A drains exact bounded output through EOF after Stage-B reap. The worker's `ECHILD` record
proves descendant emptiness. Stage A then removes all in-namespace entries under descriptors and exits; host authority alone proves
namespace teardown, removes the exact underlying backing path, and finalizes evidence. Any other
Cargo runtime change is ineligible.

At that preserved RP5 checkpoint, the crosswalk transition was this bounded RP5 closeout docs
packet. RP3 was complete, RP4 product proof was accepted clean by human disposition, and the raw
persistence review remained `REQUEST_CHANGES` for three cache-only non-blocking process-audit debt
items outside RP4 product-proof scope. The docs-on-top -> one ordinary fast-forward publication
model stayed unchanged. Source publication had not yet occurred in that RP5 run; it later
completed, after which the R2-3 suffix and refreshed native evidence closed R2-3. R2-4 and R3
remain later than that completed publication/closeout chain.

**Source provenance:**
- extracted from [`02-seam-crosswalk.md#broad-wall-invocation-provenance-crosswalk`](../02-seam-crosswalk.md#broad-wall-invocation-provenance-crosswalk), lines 1368–1504; baseline span SHA-256 `e78fad39c1a89c2b9c83f6d077e57e1e0dea1dfb6ce20e0a53682a480a25051b`
