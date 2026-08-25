**Kind:** architecture
**Stable ID:** `A1.1d-5R2-2-renewed-closeout-family`
**Canonical for:** A1.1d-5R2-2 renewed closeout architecture
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** [`01-target-architecture.md#renewed-r2-2-attestation-and-publication-ordering`](../01-target-architecture.md#renewed-r2-2-attestation-and-publication-ordering) lines 1036–1289
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2 renewed closeout architecture

## Renewed R2-2 attestation and publication ordering

The renewed closeout publishes an attestation over an already assembled and reviewed runtime
range; it does not construct a new runtime range. The exact 17 commits from F documentation
closeout `2f6f1f69b3519dafff01ef543e7d260da2c37700` through local runtime HEAD
`e5fbd2d4441d248e137d52c44e493fb0abe158f8` already sit on the latest canonical F documentation.
F's earlier replay fulfilled F's own docs-first requirement. Historical Route D replay remains
scoped to Route D and does not create a general rule to replay a renewed closeout.

The selected architecture is **Docs-on-top → one fast-forward publication**. Local publication-
authority commit `928f94e7b4c498273b40385f7bffea9e4f949700` follows the unchanged runtime range;
the six-file broad-wall invocation correction follows that authority commit. The reviewed
six-file remediation-planning commit then authorizes one bounded R1 implementation commit and
focused security review, followed by one bounded P1 implementation commit and adversarial review.
A fresh canonical baseline made only through P1 precedes the renewed production-fix-free
integration wall and independent reviews; the closeout itself still creates no implementation
commit. The final integration-closeout documentation commit follows the complete range and proof
it certifies. This ordering keeps the evidence statement after its subject while preserving every
existing runtime and documentation commit byte-for-byte and identity-for-identity.
Replaying or rewriting those commits would add identity churn without architectural or evidentiary
value. The authority state must be preserved remotely through dedicated branch
`feat/preserve-a1-1d-5r2-2-publication-authority-20260722`, not through the source branch, before the
renewed proof begins.

This ordering changes no authority, platform, credential, policy, network, filesystem, caging,
capability, lifecycle, receipt, supervisor, retained-worker, cleanup, rollback, secure-FD, or
compatibility boundary. The source remote remains documentation-only through this bounded RP5
closeout packet. The preserved pre-disposition four-review-CLEAN gate is historical only. The
controlling state at the historical RP5 documentation checkpoint recorded below was that RP4
product proof was accepted clean by human
disposition on the exact integration commit/tree, while the raw persistence `REQUEST_CHANGES`
remains preserved as three cache-only orchestration/attestation findings carried as non-blocking
process-audit debt outside RP4 product-proof scope. At that checkpoint publication remained
unperformed and was only the next step after the reviewed/committed RP5 docs. A failed or stopped
wall would publish nothing and authorize no runtime/test repair. At that checkpoint R2-2 remained
incomplete, non-Linux posture remained compatibility/unavailable/unproven, product smoke stayed
unclaimed, no seam was promoted, and R2-3 remained blocked. Final closeout documentation was
reviewed before its exact bytes were committed and
revalidated/re-reviewed afterward; any post-commit finding stops publication. A purely documentary
finding may use only an append-only six-file remediation-doc successor followed by renewed
documentation validation/review; a finding that invalidates proof or topology reruns the affected
proof/review. Neither permits a rewrite. The eventual source update requires both an explicit
expected-old-OID CAS/lease fixed to
`2f6f1f69b3519dafff01ef543e7d260da2c37700` and an independent proof that the update is a normal
fast-forward; the lease is never authority for a forced update. RP5 publication subsequently
completed by ordinary fast-forward at `0f1e147fb735791b44a65099a65167cbdc1803af`. The accepted
R2-3 suffix and refreshed native evidence then closed R2-3 at
`7a9ded10482dee2c1383477950a321ed3ee046f5`. R2-4 subsequently closed as the bounded
documentation/evidence-only propagation join recorded in the
[R2-4 bounded closeout architecture disposition](../a1.1d-5r2-4/architecture-disposition.md#r2-4-bounded-closeout-architecture-disposition);
R3 and all
explicitly later or unclaimed work remain open.

The docs-on-top -> one ordinary fast-forward publication model above remains controlling. Its RP0
sequencing is historical only; the controlling current RP3/RP4/RP5 closeout status is recorded
below.

## Broad-wall evidence architecture

Invocation provenance is part of the proof boundary, not ambient harness setup. Every live broad
shell wall, parallel or serial authority wall, differential baseline wall, final change-detection
wall, renewed closeout wall, and F/Harness wall in this document is governed by the
[canonical shell-library broad-wall invocation contract](../04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract).
The contract creates a fresh private `R`, binds exact validated `R/tmp` and `R/xdg-runtime` before
Cargo starts, retains owner/mode/no-symlink/filesystem/ancestor/ACL identity through the wall, and
removes only the revalidated disposable root after all children exit and evidence is preserved.

This boundary preserves the existing trusted-root architecture. A mode-`1777` `/tmp` ancestor is
supposed to fail private-root validation; the correct response is to place test-owned temporary
roots below a trusted non-world-writable parent, never to weaken ancestor validation or fixtures.
`SUBSTRATE_TEST_TRUSTED_PARENT` is placement input only and cannot select product authority. The
real `SUBSTRATE_HOME` and all user/product state stay outside the disposable wall root.

The omitted-root wall is therefore ineligible evidence, classified
`BaselineCommandMismatchConfirmed`, and cannot authorize `PassToFail`, `NewFail`, baseline, or
serial-only conclusions. The provenance-valid baseline remains `1309/1264/45/0`; the ineligible
wall remains separately recorded as `1309/1205/104/0`, with all 59 additions caused by normal
`/tmp` placement and correct trusted-root rejection. Runtime source and tests are unchanged.

At the historical B1 checkpoint, the next architectural action was the renewed
production-fix-free integration closeout from the B1-preserved docs-on-top state using
provenance-validated private roots. The historical remediation architecture below supersedes that
next-action statement. At that checkpoint, R2-2 product proof and source publication were still
pending, and R2-3, R2-4, and R3 remained blocked.

## Closeout-remediation architecture: R1 and P1 (historical RP0/RP1/RP2 record)

This historical planning authority changed no product architecture. It defined the two
proof-preserving repair boundaries required before the renewed production-fix-free closeout could
start again.

### R1: separate execution argv from display argv

The release installer currently has one generic executor, `run_cmd`, with four direct call sites:
three inside `run_with_sudo` and one inside `deploy_shims`. Source closure found that only the
`deploy_shims` call carries the authenticated bootstrap carrier. The source-closed
`run_with_sudo` callers are a finite current set resolved through the fixed privileged PATH:
`groupadd`, `usermod`, package managers, `install`, `systemctl`, `rm`, and the one permitted
absolute ACL helper. Their operands include account/group names, packages, artifact/TMP and
destination paths, modes/owners, unit/socket names, and ACL actions/paths/groups, but no carrier,
credential, authorization value, commitment preimage, prompt/request material, or secret token.
`run_with_sudo` does not itself enforce a named allowlist for bare tools; this conclusion is the
exact current-call-site inventory. Other carrier-bearing installer routes already use dedicated
fixed dry-run placeholders and xtrace suppression.

Source closure also rejected reuse of the existing structured display helper
`scripts/linux/world-provision.sh::show_install_context_cmd`. It uses `%q` and redacts the
separate carrier form plus an environment assignment, but it is display-only, does not support
the accepted equals form, does not enforce the missing/duplicate/malformed/end-of-options
contract, and is not coupled to byte-exact execution argv. The Rust-only
`crates/common::{redact_sensitive, redact_process_argv}` logging helpers use heuristic
sensitive-name/value matching, and `redact_sensitive` has raw logging behavior; neither can be
authenticated-carrier authority for this Bash installer. Importing or extending any of these
helpers would broaden R1 beyond its exact two-file allowlist, so they remain unchanged.

R1 therefore adds a dedicated `run_cmd_with_redacted_install_bootstrap_carrier` boundary and changes
only `deploy_shims` to use it. Its architecture is two-channel:

- execution keeps the original array and invokes exact `"$@"`;
- dry-run display walks argv structurally, recognizes the production form
  `--install-bootstrap-context-v1 VALUE` and the live Substrate CLI's accepted
  `--install-bootstrap-context-v1=VALUE` form, replaces only the value with the fixed token
  `<redacted-authenticated-bootstrap-carrier-v1>`, preserves the input flag form for display, and
  boundary-safely renders only nonsensitive arguments;
- no `eval`, reconstructed command execution, regex secret guessing, digest, prefix, suffix,
  length, carrier fragment, or preimage is permitted;
- a missing/empty value or duplicate sensitive flag, including mixed-form duplicates, fails closed
  before display or execution with exact stderr
  `[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n` and status
  `2`; that argument-independent result also governs protected malformed near-matches, separate
  values beginning `-`, and protected forms or near-matches after exact `--`, before Clap or any
  display can echo candidate bytes.

The release installer parser and production call construct the separate `--flag value` form; the
invoked Clap-based Substrate CLI also accepts `--flag=value`. R1 adds no grammar. Normal execution
of either accepted form, argument ordering/boundaries, authenticated carrier validation,
account/UID binding, prefix selection, shim deployment, inherited-xtrace suppression, and exact
xtrace restoration remain unchanged. Generic `run_cmd` and all nonsensitive dry-run output remain
compatible. A structurally valid live child retains its exact exit status; only the helper's
structural rejections return `2`, after `deploy_shims` restores the inherited xtrace state.

### P1: retired runner history and current Make authority

The tracked P1 Python runner and its self-tests are retained only as historical diagnostic
evidence. Their authenticated `1322 discovered / 1277 passed / 45 failed / 0 ignored` result is
not baseline authority because the runner never achieved eligible provenance, and this increment
retires `scripts/ci/canonical_shell_wall_runner.py` plus
`scripts/ci/test_canonical_shell_wall_runner.py` rather than extending them.

For the exact bound source, broad shell-wall authority is now two Linux-only repo-root Make
targets: `make shell-lib-wall` and `make shell-lib-wall-serial`. Each target validates a trusted
current-user-owned mode-`0700` parent, creates one compact private mode-`0700` root with private
`TMPDIR` and `XDG_RUNTIME_DIR`, rejects overlong runtime socket layouts, runs exact Cargo argv
`cargo test -p shell --lib -- --nocapture` with only `--test-threads=1` appended for the serial
target, streams Cargo output unchanged, preserves Cargo's recipe exit status through exact-root
cleanup, and leaves GNU Make's standard public zero/nonzero mapping unchanged. No Python, new
repository script, production helper, dependency, or semantic result parser participates in the
current authority path.
repository/rustup snapshots and writable-projects only that private Cargo runtime at the canonical
Cargo-home path. The seed omits Cargo 1.89 metadata files; the runtime may create or change only
exact regular files `.package-cache`, `.package-cache-mutate`, and `.global-cache`. Every other
runtime write is ineligible. Canonical
`PATH` still reaches the copied `cargo -> rustup` shim, and the verified repository
`rust-toolchain.toml` still selects Rust 1.89. Neither mutable tracked pathname nor host
source/cache/toolchain inode is executed by Cargo. If
the exact Bubblewrap/TCB premise is unavailable, P1 stops as
`BroadWallContainmentDecisionRequired`; post-start self-hashing is not a substitute.
The layers are:

1. authenticated host `host_main` retains the evidence/backing descriptors and exact Stage-A
   pidfd/status while Stage A Bubblewrap creates a private user/mount namespace and tmpfs;
2. authenticated `stage_a_main` constructs immutable seeds plus the writable Cargo runtime and
   fresh current-UID `0700` root, then creates fixed mode-`0600`
   `R/control/worker.sock` and launches exact nested Stage B Bubblewrap as a retained direct child
   with a new PID namespace, `--die-with-parent`, JSON status FD, and one Stage-A-owned capture
   pipe whose sole writer is explicitly duplicated to Stage-B stdout/stderr;
3. Stage B authenticates the same runner through sealed `--ro-bind-data`; its worker connects to
   the fixed private socket, and Stage A accepts only `SO_PEERCRED` matching Bubblewrap's reported
   child pidfd. Both endpoints are close-on-exec, the socket pathname is absent before START,
   Cargo inherits original stdin, only the one stdout/stderr pipe, and no control/evidence FD.
   Stage A drains that pipe to the exact bounded log until EOF after Stage-B reap. Stage B uses
   `--as-pid-1`, so the authenticated worker is both Bubblewrap's status-reported application and
   the namespace adoption root; Stage A's default Bubblewrap PID 1 remains fail-safe teardown only;
4. the worker sets/reads back `PR_SET_CHILD_SUBREAPER` before START/Cargo, waits for Cargo, then
   waits/reaps to kernel `ECHILD`; Stage A requires that authenticated record, cleans every
   in-namespace tree under continuous descriptors, and exits without claiming its own enclosing
   namespace is gone;
5. `host_main` alone proves Stage-A process/namespace teardown, removes the exact retained
   underlying backing path under descriptor authority, and only then atomically finalizes
   `eligible=true`.

Repository cleanliness is independently bound to expected-HEAD tree modes/blob IDs, stage-zero
index entries, and no-follow worktree bytes. Only tracked, verified `.gitignore` files may classify
untracked output; repository-local config, `.git/info/exclude`, worktree config, and global
excludes cannot hide source state.

Cargo uses a fresh private target mounted at canonical `<repository>/target`, never the host
repository target and without a `CARGO_TARGET_DIR` override. The exact `cargo`/`rustc` symlinks to
the copied rustup binary, selected rustup settings, complete pinned Rust 1.89 toolchain, and
checksum-selected registry inputs are byte-copied into immutable seeds. Only the private Cargo
runtime copy is writable, and its post-manifest may differ from the seed solely by creating or
changing the three named Cargo metadata regular files; no `CARGO_*`, `RUSTUP_HOME`, `PATH`, or
`HOME` override is added. Exact
proxy-version and contained compile probes prove the mounted resolution chain before the wall.

This is grounded in the Linux contracts for
[`PR_SET_CHILD_SUBREAPER`](https://man7.org/linux/man-pages/man2/PR_SET_CHILD_SUBREAPER.2const.html),
[PID namespaces](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html),
[`waitid`/`waitpid`](https://man7.org/linux/man-pages/man2/waitpid.2.html),
[`openat`/`unlinkat`](https://man7.org/linux/man-pages/man2/unlinkat.2.html), and
[`statx`](https://man7.org/linux/man-pages/man2/statx.2.html). Reparenting, double-forking, or
process-group changes remain inside the namespace and converge on the verified Stage-B worker,
which runs as namespace PID 1 and sets/reads back subreaper state. Stage A's default Bubblewrap
PID 1 remains fail-safe teardown only. A timeout or surviving descendant is an
ineligible wall; forced namespace teardown is diagnostic cleanup only and can never turn that
wall valid.

The disposable root lives below a private namespace mount, remains distinct from the real product
`SUBSTRATE_HOME`, and is never shared between walls. Owner, mode, type, device/inode, mount ID,
ancestor safety, and effective access/default ACLs are checked before use and again under the same
open descriptors. Named-user, named-group, group-class, or other-class effective write permission
is forbidden. All deletion is no-follow and directory-FD-relative; pathname identity must still
match before unlink, absence must be proved before descriptors close, and an unrelated sentinel
must survive.

P1 changes no shell, world, policy, capability, service, secure-FD, lifecycle, cleanup, placement,
or product-runtime symbol. It is proof infrastructure only. Native macOS/Windows and privileged
product proofs remain deferred to their existing owners.

### Historical RP0 architecture status

R1 is a correction to display non-disclosure, not carrier authority. P1 is an evidence-authority
runner, not a product supervisor or lifecycle owner. Neither creates a new runtime seam, process
family, platform adapter, credential route, or cleanup authority. The four prior broad walls remain
ineligible, R2-2 remained incomplete, and R2-3 remained blocked. The exact next architectural node
at that checkpoint was the bounded R1 increment. The controlling current status follows.

## RP3/RP4/RP5 closeout architecture status and subsequent completion

The architecture above remains unchanged. RP3 is complete with the same canonical
`1309/1264/45/0` baseline and zero pairwise differentials. RP4 product proof is accepted clean by
human disposition on exact integration commit/tree
`8c46135c861a468dea316cf9fd7d6c6bb15bddac` /
`5358497a8baec6f36e15aaef58415a759e64977d`; the raw persistence review remains
`REQUEST_CHANGES` and is preserved as three cache-only non-blocking process-audit debt items
outside RP4 product-proof scope. This packet claims no non-Linux proof, privileged product smoke,
or seam promotion.

That exact six-file change was the bounded RP5 closeout packet. It expanded no authority boundary
and added no new architecture. The docs-on-top -> one ordinary fast-forward publication model was
followed: source publication completed at `0f1e147fb735791b44a65099a65167cbdc1803af` without
rewriting historical proof. The accepted R2-3 suffix and refreshed native evidence subsequently
closed R2-3 at `7a9ded10482dee2c1383477950a321ed3ee046f5`. R2-4 subsequently closed as the bounded
documentation/evidence-only propagation join recorded in the
[R2-4 bounded closeout architecture disposition](../a1.1d-5r2-4/architecture-disposition.md#r2-4-bounded-closeout-architecture-disposition).
R3, direct-member Codex/UAA gateway
adoption, and all other explicitly later or unclaimed work remain open.

**Source provenance:**
- extracted from [`01-target-architecture.md#renewed-r2-2-attestation-and-publication-ordering`](../01-target-architecture.md#renewed-r2-2-attestation-and-publication-ordering), lines 1036–1289; baseline span SHA-256 `3839dd2e7a9924241313e2de0e56d1172813e8476f30a6e615585cf08e446b0d`
