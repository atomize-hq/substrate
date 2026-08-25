**Kind:** evidence/regression
**Stable ID:** `A1.1d-5R2-2-renewed-closeout-family`
**Canonical for:** A1.1d-5R2-2 renewed closeout evidence and regression
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** [`05-debug-regression-ledger.md#renewed-r2-2-publication-decision-ledger`](../05-debug-regression-ledger.md#renewed-r2-2-publication-decision-ledger) lines 2856–3166
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2 renewed closeout evidence and regression

## Renewed R2-2 publication decision ledger

| Decision/gate | Status | Evidence and distinction | Binding resolution / next action |
|---|---|---|---|
| `PublicationDecisionRequired` | Resolved and preserved at `928f94e7b4c498273b40385f7bffea9e4f949700` | The completed F closeout required its documentation followed by exact replay, and that replay produced the current linear 17-commit range from `2f6f1f69b3519dafff01ef543e7d260da2c37700` through `e5fbd2d4441d248e137d52c44e493fb0abe158f8`. Historical Route D docs-first replay governed only its own pre-carrier correction. The selected docs-on-top publication authority is preserved locally/remotely at `feat/preserve-a1-1d-5r2-2-publication-authority-20260722`. | Keep commit and range identities unchanged. The later RP4 proof and current RP5 packet do not rewrite this preserved publication-authority checkpoint. |
| `BaselineCommandMismatchConfirmed` | Historical B1 correction complete; later renewed wall evidence ineligible | The first renewed-closeout prompt omitted the private-root invocation provenance. That wall was ineligible and added 59 correct trusted-root rejections below normal mode-`1777` `/tmp`; runtime source and tests did not change. B1 corrected that invocation contract. A later renewed attempt produced four count/hash-matching walls, but their parent-only wait and descriptor-before-delete closure made them provenance-ineligible. | Historical B1 disposition was to commit the [canonical broad-wall invocation contract](../04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract) on exact `928f94e7`, preserve it only on the B1 branch, and stop. The current RP4/RP5 disposition is recorded below; no historical result is rewritten, and source publication was still the next step at that checkpoint. |
| Existing runtime identity | Frozen | All 17 current commits are linear, exact, locally reviewed, and preserved. They already rest on the latest F documentation. Replaying them again would create new identities without changing semantic content or proof value. | No replay, rebase, rewrite, cherry-pick, merge commit, or force push. Preserve every runtime byte and commit identity; keep blocked and preservation-only donors outside source ancestry. |
| Renewed integration proof | Historical ineligible attempt preserved; later RP4 product proof accepted clean by human disposition | Neither the publication-authority commit nor the B1 broad-wall correction itself ran an integration wall. The earlier renewed attempt ran four matching walls but was provenance-ineligible because R1 was authority/security NOT CLEAN and P1 failed the descendant/continuous-authority contract. The preserved RP4 packet later established focused exact PASS, authenticated runner self-test `99/99`, four eligible canonical walls, zero differentials, `6 launched / 6 reaped / 0 live`, and exact original/integration restoration on proofed integration commit/tree `8c46135c861a468dea316cf9fd7d6c6bb15bddac` / `5358497a8baec6f36e15aaef58415a759e64977d`. | Raw review files remain unchanged: gates/authority/sequencing `APPROVE`; persistence `REQUEST_CHANGES` preserved as three cache-only non-blocking process-audit debt items outside RP4 product-proof scope. This exact six-file RP5 docs packet preserves that distinction; source publication was still the next step at that checkpoint. |
| Final publication | Historical blocked state later resolved by the single ordinary fast-forward publication | A closeout document had to follow the already assembled range and evidence it certified. The source remote intentionally contained documentation only because source publication had not yet occurred in that RP5 run. | This exact six-file RP5 docs packet had to be reviewed/committed first. After those reviewed bytes were committed, source publication later completed through one ordinary fast-forward without rewrite. That completed publication then allowed the later R2-3 suffix chain and refreshed native evidence to close R2-3. |

This decision does not alter any implementation status recorded above. F remains complete under its
own packet-specific replay history. RP3 is complete, RP4 product proof is human-accepted clean, and
this exact six-file change is the bounded RP5 closeout packet. Source publication had not yet
occurred at that checkpoint; it later completed, after which the R2-3 suffix chain and refreshed
native evidence closed R2-3. R2-4 and R3
remain later, privileged product smoke remains unclaimed, and no seam is promoted.

## A1.1d-5R2-2-B1 broad-wall invocation evidence correction

Every live broad shell wall, parallel/serial authority wall, canonical or differential baseline
wall, final change-detection wall, renewed closeout wall, and F/Harness wall in this ledger uses the
[canonical shell-library broad-wall invocation contract](../04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract).
Historical commands below are retained as exact evidence and are non-normative when they omit that
contract.

### Verified B0 evidence and causal reconstruction

The immutable B0 packet is
`/home/spenser/.gstack/projects/atomize-hq-substrate/checkpoints/20260722-a1-r2-2-b0-evidence.md`,
SHA-256 `75a15639ccb60d4d1b8206d6fa23622d5146213ebf8409b1e78fcc6d0417d776`.
It binds branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`, HEAD
`928f94e7b4c498273b40385f7bffea9e4f949700`, tree
`9f0f9b2362ba878f45a30ab169a7c1e3e4569df2`, unchanged source remote
`2f6f1f69b3519dafff01ef543e7d260da2c37700`, and runtime ordered-commit hash
`ffe57dc349b462b318752a346e6577cff80bd15ba264720b967b888778f45700`.

The failed renewed-closeout command was historically:

```text
cargo test -p shell --lib
```

It discovered 1,309 tests, passed 1,205, failed 104, ignored 0, and produced failure-name hash
`7d6564bbcfaaed1fdb784eb4610683a02e4464b362215761d728be3e7361d8e5` plus normalized-signature
hash `ae67f87fbdc155575e13467506405afbd4568fc08b9058cb4159daa539549579`.
Omitting `TMPDIR` made Rust `TempDir` roots for six StateStore fixture families direct descendants
of `/tmp`. `/tmp` had its normal root-owned mode `1777`; retained trusted-root validation walked the
ancestor chain and correctly rejected its world-writable authority before persistent state
mutation.

The causally correct historical control was:

```text
XDG_RUNTIME_DIR=/home/spenser/t \
TMPDIR=/home/spenser/t \
cargo test -p shell --lib -- --nocapture
```

`/home/spenser/t` is historical evidence only and must never become the canonical machine-specific
path. The control restored 1,309 discovered, 1,264 passed, 45 failed, and 0 ignored, with canonical
failure-name hash `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`
and normalized-signature hash
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.

All 59 additions reduce to this single root-opening cause. Exact serial and isolated tests fail
identically without private `TMPDIR`; private-root parity makes all six families pass. Runtime
source, tests, manifests, lockfile, toolchain, target, profile, and features are unchanged. This is
not environment drift, process-global test isolation, or a production regression, and it provides
no reason to change fixture behavior or trusted-root validation.

### Baseline and future classification authority

The terminal classification is `BaselineCommandMismatchConfirmed`. The 59 additions remain
separate ineligible evidence. They are not added to the 45-failure baseline, waived, treated as
product failures or environment drift, used to weaken security, or used to justify serial-only
proof.

Future wall processing validates root and command provenance before result comparison. A missing
or invalid `TMPDIR`, missing or invalid `XDG_RUNTIME_DIR`, unsafe/sticky ancestor, incomplete
owner/mode/no-symlink/identity/ACL proof, or incomplete command/toolchain/result/hash record makes
the entire wall ineligible. Correct the invocation and rerun the complete required wall. Only a
provenance-valid wall can create a `PassToFail`, `NewFail`, changed-failure, or baseline transition.

The B1 correction changed exactly the six canonical Markdown files and no runtime/test artifact.
It ran no integration wall, did not create an integration-clean branch, and left the source
remote unchanged. Its historical B1-only binding sequence was:

```text
B1 broad-wall invocation docs correction
  -> renewed production-fix-free integration wall using validated private roots
  -> fresh independent reviews
  -> final six-file integration-closeout docs
  -> one ordinary fast-forward source push
```

At that B1 checkpoint, R2-2 remained incomplete and the exact historical next task was **Resume
A1.1d-5R2-2 renewed production-fix-free integration closeout using provenance-validated private
roots**. The current planning ledger below supersedes only that next-task disposition.

## Closeout-remediation planning ledger (historical RP0 checkpoint)

### Immutable planning checkpoint

| Field | Verified value |
|---|---|
| Branch | `feat/internal-host-orchestrator-world-dispatch-bootstrap` |
| HEAD / parent / tree | `f7ded83ef147b748678ba6b028eea959870a04fe` / `928f94e7b4c498273b40385f7bffea9e4f949700` / `e0836009318259e168acf3fe7837c57ec61c73fd` |
| Source remote / divergence | `2f6f1f69b3519dafff01ef543e7d260da2c37700`; `0 behind / 19 ahead` |
| Repository state | worktree, index, and untracked set CLEAN |
| Publication preservation | `feat/preserve-a1-1d-5r2-2-publication-authority-20260722` → `928f94e7b4c498273b40385f7bffea9e4f949700` |
| Invocation preservation | `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722` → `f7ded83ef147b748678ba6b028eea959870a04fe` |
| GitNexus | repository `substrate`, indexed at exact `f7ded83`; 1,442 files, 34,536 graph nodes, 70,731 graph edges, 1,564 communities, 300 execution flows |

The runtime and documentation range was unchanged at checkpoint. No source checkpoint commit,
integration-clean preservation, wall invocation, implementation/test edit, or source publication
occurred in this planning investigation.

### Renewed-closeout blocker record

| Review | Verdict | Controlling evidence |
|---|---|---|
| Authority/security | NOT CLEAN | `deploy_shims` passes the authenticated carrier to `run_cmd`; dry-run `$*` emitted the marker and `carrier_marker_disclosed=true` |
| Lifecycle/product | CLEAN | No separate lifecycle/product blocker found; this does not override R1/P1 |
| Inventory/source | CLEAN | Owned production routes closed; this does not override R1/P1 |
| Baseline/publication | BLOCKED | Harness did not prove descendant exit and released descriptor authority before deletion |

The four prior walls each reported `1309 discovered / 1264 passed / 45 failed / 0 ignored`,
failure-name hash
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`, and normalized-signature
hash `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.
They remain **EVIDENCE-INELIGIBLE**. No baseline transition is inferred from them and no historical
review is rewritten.

### R1 source-closure evidence

At the checkpointed source:

| Source | Current behavior | Closure |
|---|---|---|
| `run_cmd` | Dry-run prints `"$*"`; live mode executes `"$@"` | Generic behavior retained; four direct calls inventoried |
| `run_with_sudo` no-sudo call | Finite current bare-tool set resolved through fixed privileged PATH, or one permitted absolute ACL helper; account/group, package, artifact/TMP/destination path, mode/owner, unit/socket, and ACL operands | No protected value class found |
| `run_with_sudo` `sudo -n` call | Exact sudo/env wrapper plus that current tool/operand set | No protected value class found |
| `run_with_sudo` interactive-sudo call | Exact sudo/env wrapper plus that current tool/operand set; no-TTY error may render original operands | No protected value class found |
| `deploy_shims` call | Carrier plus `--shim-deploy`; xtrace disabled/restored around `run_cmd` | Sole current sensitive direct caller; R1 target |
| Other install carrier routes | Dedicated fixed dry-run placeholders and xtrace suppression | No additional live disclosure found |
| `world-provision.sh::show_install_context_cmd` | `%q` display helper redacts the separate carrier and environment-assignment forms | Not reusable: display-only, no equals-form or structural-rejection grammar, and not coupled to exact execution; frozen |
| `crates/common::{redact_sensitive, redact_process_argv}` | Rust logging redactors use heuristic sensitive-name/value matching; raw logging behavior exists | Not exact Bash carrier authority; frozen |
| Uninstaller | Separate carrier handling; no `run_cmd` | Frozen and required as unchanged compatibility proof |

The search covered carriers, credentials, authorization values, commitment preimages,
prompt/request values, private host paths/selectors, dry-run/live output, failure rendering,
xtrace, and installer/uninstaller compatibility. `run_with_sudo` has no named bare-tool allowlist;
the static current callers above are the closed set. GitNexus could not resolve these shell
symbols, so the exact manual call-site inventory controls. R1 may not expand beyond the one named
production helper/caller and one focused test without new evidence and docs-first authorization.
Every helper structural rejection has one exact argument-independent result: stderr
`[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n` and status `2`;
valid child status propagation and exact post-call xtrace restoration remain unchanged.

### P1 failed-harness evidence

The prompt-local harness:

1. used `subprocess.run` and waited only for Cargo;
2. recorded `process_and_children_exited=true` without a descendant-authority observation;
3. could not prove reparented or process-group-changing descendants were gone;
4. closed root/child validation descriptors before cleanup;
5. called pathname `shutil.rmtree` after losing continuous identity;
6. checked pathname absence only after that unauthoritative deletion.

The shell wrapper's PID namespace eventually bounded teardown, but it did not connect namespace
emptiness to the Python worker's wall-eligibility decision. Matching output cannot repair those
facts.

### P1 source-closure and decision record

Searches covered installer and test harnesses, checkpoint helpers, Rust test utilities, Python and
shell helpers, production trusted-root utilities, process groups, cgroups, namespaces, subreapers,
log/hash preservation, and deletion. No tracked helper met the combined contract. Reusing
production trusted-root, cgroup, service, or provisioning code would exceed the test-only
allowlist. The selected minimum is the two-file Linux standard-library runner in
`04-contracts-and-gates.md`.

The proof basis is:

- exact installed Bubblewrap plus the pinned root-owned OS/Python startup closure are the explicit
  immutable platform TCB; exact direct Python/V2 plus authenticated in-memory `host_main` is the
  concrete host controller; the exact reviewed P1 commit binds bootstrap and three argv-template
  trailers to independently verified runner-blob constants, while each runtime instantiation
  records separate actual argv hashes, with no mutable external authority artifact; a
  clean nested unprivileged probe supplies namespace authority, and Python does not claim to
  preauthenticate already-executed startup bytes;
- reviewed V2 bootstrap bytes execute retained Git by FD, independently verify the
  expected-commit/tree/blob chain, authenticate runner/test blobs for in-memory execution, and
  supply Stage-A/Stage-B runner bytes only by sealed memfd plus `--ro-bind-data`;
- Stage A constructs private checksum-authenticated repository/rustup/Cargo immutable seeds and a
  separate writable Cargo runtime; nested Stage B read-only projects repository/rustup and
  writable-projects only the private Cargo runtime at canonical paths, so verified
  `cargo -> rustup` plus repository `rust-toolchain.toml` still selects Rust 1.89 while preserving
  exact Cargo argv/CWD/E0 except TMPDIR/XDG. The seed omits Cargo metadata; only exact regular
  `.package-cache`, `.package-cache-mutate`, and `.global-cache` may be created or changed;
  host modify/replace/restore races
  cannot become Cargo input;
- exact nested Bubblewrap bounds every wall process in a private PID namespace; the
  detached-descendant probe proved default Bubblewrap parent wait/PID 1 is fail-safe teardown
  only, so Stage B uses `--as-pid-1` and the authenticated worker itself is both the
  status-reported application and namespace adoption root;
- the Stage B worker connects to a fixed private `SOCK_SEQPACKET` path and is accepted only when
  `SO_PEERCRED` matches Bubblewrap's retained child pidfd; both endpoints are close-on-exec, the
  pathname is absent before START, Cargo retains original stdin, one exact Stage-A-owned combined
  stdout/stderr pipe, and no control/evidence FD; Stage A drains that pipe with bounded
  backpressure through EOF after Stage-B reap and preserves exact eligible bytes/hash; the worker
  becomes the verified pre-Cargo child subreaper that receives reparented descendants;
- worker reap-to-`ECHILD`, its authenticated completion record, and clean retained Bubblewrap
  status together prove containment emptiness;
- timeout/survivor is ineligible before scoped teardown;
- open no-follow parent/root/child descriptors retain identity through final validation,
  descriptor-relative unlink, and pathname-absence proof; Stage A removes all private entries but
  never claims its own enclosing mount is gone;
- authenticated host `host_main` retains separate backing/evidence authority, proves Stage-A
  process/namespace teardown, removes and proves absence of the exact underlying mountpoint, and
  alone finalizes eligibility;
- evidence is preserved outside `R`, and unrelated sentinels survive.

The future runner's own self-tests precede every product wall. Only after those tests and fresh
P1 review are CLEAN may a new three-parallel/one-serial baseline be established.

## RP3/RP4/RP5 closeout ledger

This section is the controlling current status. It preserves the historical RP0 planning ledger
above without rewriting any raw review verdict or historical wall result.

| Packet / evidence | Current status | Preserved evidence | Binding boundary |
|---|---|---|---|
| RP3 — canonical baseline | Complete | Canonical baseline remains `1309 discovered / 1264 passed / 45 failed / 0 ignored`, failure-name hash `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`, normalized-signature hash `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`, and six pairwise differentials `0`. | The historical 45-failure set is unchanged. |
| RP4 — renewed production-fix-free closeout | Product proof accepted clean by human disposition | Packet `/home/spenser/.cache/substrate-runtime-refactor/rp4-rp5-closeout-20260725T184536Z/rp4-proof-es7gs6z2/rp4-closeout-run-packet.json`; packet SHA `ea2935d04062c9e7e1327d5ecf88122513164b14fb19c7a2122c32b71b326072`; summary SHA `a51cf659647d10803004e7ad49c94ce580f9f23d5e4526608d60eb1a70789684`; supervisor result SHA `8e087dc6a4b627f61a97ade65bd4a13680113d79a9c718974671e5e91f20cee9`; supervisor launch manifest SHA `efbdd4ee8cf60aaa2b9128ca4c6844b82bb804f73dde5d53bcd8369906d173b8`; focused exact PASS sentinel; authenticated runner self-test `99/99`; four distinct canonical walls each `1309/1264/45/0`, `eligible=true`, `wall_gate=clean`, `canonical_match=true`; proof processes `6 launched / 6 reaped / 0 live`; exact original/integration restoration plus no-edit/config/branch/commit/publication guards passed; proofed integration commit/tree `8c46135c861a468dea316cf9fd7d6c6bb15bddac` / `5358497a8baec6f36e15aaef58415a759e64977d`. | No product/runtime rerun or remediation is authorized in this packet. |
| RP4 raw review distinction | Raw verdicts preserved unchanged | `rp4-attempt8-review-gates.txt` SHA `0e482944222da337ce3f6b1b4a3d2d243e550b8091d7720282c3b08b0dc361dc` `APPROVE`; `rp4-attempt8-review-authority.txt` SHA `5ecadc8fa03c546722be0f854665e49db30520610c60670a0abaddda8df11c67` `APPROVE`; `rp4-attempt8-review-sequencing.txt` SHA `654d8e30e5a7ec22a8afe307f9dd5602a5b15e9867f6d32f9f46d9c43fa44e44` `APPROVE`; `rp4-attempt8-review-persistence.txt` SHA `4489c53ae85555c45b32485cdff1fa6ceb4af194eecf6ce2668512620d06eebe` `REQUEST_CHANGES`. | The persistence verdict is not rewritten to `APPROVE` or `CLEAN`. |
| Persistence review findings | Non-blocking process-audit debt outside RP4 product-proof scope | Controller pathname/hash not execution-bound; supervisor parent-directory `fsync` omitted after replace; persisted controller packet omits its post-persistence final verdict. These are cache-only orchestration/attestation issues, not Substrate product/runtime defects and not canonical wall-runner defects. | They do not invalidate the focused, authenticated, canonical-wall, differential, authority, or restoration evidence. |
| RP5 — bounded final docs closeout | This exact six-file docs change | Preserves **Docs-on-top -> one ordinary fast-forward publication** and makes no wall rerun, no publication claim, and no parity claim. | At that checkpoint, source publication had not yet occurred, and the exact next step after the reviewed/committed RP5 docs was one ordinary fast-forward source publication. That publication later completed without rewrite. |

R2-3ZP2 is a later runner-contract repair, not a rewrite of the historical RP4 packet above. It
separates live descendant `expected_head` from reviewed `authority_commit_oid`, adds a separate
control-plane `reviewed_authority_commit_oid` that must exactly equal
`authority_commit_oid`, requires the reviewed authority commit to be equal to or an ancestor of
the live head, requires exact authenticated runner/self-test blob continuity across that boundary,
propagates both reviewed-authority fields plus projected repository CWD into Stage B, makes
`reviewed_oid_matches` a verified equality result rather than a caller assertion, and raises the
authenticated self-test gate to the exact 101-method allowlist. This closes
reviewed-authority/descendant drift once the reviewed authority OID is supplied independently by
the trusted control plane, but same-head review provenance remains external to the runner rather
than derivable from caller-selected OIDs alone. The preserved RP4 packet still remains historical
evidence for July 25, 2026, including its then-authenticated `99/99` self-test result on proofed
integration commit `8c46135c861a468dea316cf9fd7d6c6bb15bddac`; that evidence is not retroactively
relabeled as a post-ZP2 proof. The next reviewed authority commit for future canonical walls must
be recomputed from final post-ZP2 bytes and trailers rather than inferred from this ledger entry.

`R2-3ZP3` is an unpublished proof-infrastructure attempt and is permanently deferred from the
blocking R2-3 path. Commits `2cb796ffef68c2b049376984a90ce0382e5f3980`,
`9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`, `50948dbeb921582515a34bb6b7be21c46f29d008`, and
`868994efaf132bb04c6cdd7c82da9433333c94e5` remain diagnostic evidence only. They are not accepted
product source and must not be cherry-picked, pushed, or represented as landed.

The recurring canonical-runner defect is now explicit. Authenticated wall executions can
materialize honest shell results yet still finalize ineligible with `evidence_write_failed` and
`mount_teardown_failed`, because the runner still lacks authenticated Stage-A completion/teardown
proof and hidden backing-path removal proof. R2-3 closeout therefore claims no eligible
canonical-runner provenance. This remains open proof-infrastructure work outside the R2-3
completion gate.

That ordinary fast-forward publication subsequently completed at
`0f1e147fb735791b44a65099a65167cbdc1803af`. The R2-3 suffix then landed as
`R2-3ZT1 -> R2-3ZH1 -> R2-3ZM5 -> fresh native macOS and Windows evidence -> R2-3Z`, with final
source checkpoint `c583c5f293644fab75d8d42bd3bcad63f114d4fe` / tree
`a070f5f5787c27180f13dece9a1c3c3241728fda`.

The accepted direct shell-library proof at that final source is
`1322 discovered / 1274 passed / 48 failed / 0 ignored`, with failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`. Count-only equivalence
remains insufficient: the historical 45-failure set must remain intact, and the only additional
failures may be the separately classified non-R2-3 expectations
`builtins::shim_doctor::report::tests::world_deps_fixture_cannot_establish_runtime_health_or_cross_a`,
`builtins::shim_doctor::report::tests::world_deps_section_forwards_authenticated_a_under_conflicting_ambient_b`,
and `builtins::world_deps::tests::doctor_snapshot_uses_authenticated_a_under_conflicting_ambient_b_without_mutation`.
The sole current broad shell-wall entrypoints for this exact source are `make shell-lib-wall` and
`make shell-lib-wall-serial`. The provenance-ineligible authenticated runner's
`1322 discovered / 1277 passed / 45 failed / 0 ignored` result, plus the noncanonical 100- and
76-failure `R2-3ZM5` reruns, remain diagnostic-only environment evidence and never replace this
accepted direct proof.

The refreshed source-bound native receipts are authoritative at this closeout:

- macOS receipt `sha256:3b44f6387070b7aaea4306ae58d7f280b1cee3e163ee59219d4900ce3f53dfaf`,
  artifact `sha256:64a726d45b8bb6724fe43f566903d7b5e5ed6ef16119f14d30cd5a03edcc7e50`,
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-MAC-01`;
- Windows receipt `sha256:4ee942690655c1fac185244438d14e2561df52c306dea7e5428d556b530fd28c`,
  artifact `sha256:2c36a8dae9f3bcfa1c240c62a1e44f93aee0798702ae78a0076d43778b93d46e`,
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-WIN-01`.

Both bind the exact source commit/tree/ref, confirm prohibited actions, and leave their platform
checkouts unchanged. Neither claims forwarding activation, provisioning, timeout kill, stop,
PID/artifact deletion, unregister, cleanup, rollback, or convergence authority.

R2-3 therefore closes only PI-009, PI-022, PI-039–PI-046, PI-048–PI-049, PI-052, PI-054–PI-058,
PI-060, PI-068–PI-070, PI-075–PI-076, PI-079, PI-081, PI-090–PI-091, PI-112, PI-115–PI-116, and
PI-118. PI-059 remains harness-only; PI-077 and PI-078 remain byte-frozen fail-closed guards;
PI-050 remains the R2-4 guardrail; PI-080 remains satisfied by earlier R2-2 Linux restart-scope
work and is not reopened here; and every R3 lifecycle/forwarding/provisioning/cleanup/rollback/
convergence action remains open. R2-4 and R3 remain later. Privileged product smoke, broader non-Linux product
proof, direct-member adoption, and seam promotion remain unclaimed.

**Source provenance:**
- extracted from [`05-debug-regression-ledger.md#renewed-r2-2-publication-decision-ledger`](../05-debug-regression-ledger.md#renewed-r2-2-publication-decision-ledger), lines 2856–3166; baseline span SHA-256 `49b6b16491cc9a450e602772c3b73e098593e47be5d0a42bfb798c8a2f472c09`
