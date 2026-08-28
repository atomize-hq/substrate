**Kind:** current-state projection
**Stable ID:** `A1.1d-5R2-2F-family`
**Canonical for:** A1.1d-5R2-2F family current-state and closeout projection bundle
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** [`00-README.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction`](../00-README.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction) lines 454–793
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2F family current-state and closeout projection

## A1.1d-5R2-2F0-HC shell-harness closure audit and environment correction

`A1.1d-5R2-2F0-HC` is corrected and authorized as a bounded evidence prerequisite inside canonical
slice A1. It is not a new top-level slice, micro-packet, or product seam. The correction started
from published control-pack commit `9a3b54edcf1560d50b6aafda7293699fce85c719` (tree
`740c19fa4099161f243220c2c25916c76485d7bf`) and audited immutable replayed-E source
`a6c4a5a52c7e33d4efcc18f70dc2b7301ab0fcca` (tree
`17b83d26b3f9a6258d0234461543528c269338bb`). It changes documentation and evidence only.

The corrected shell-library process closure contains 38 resource rows: environment 4, fixed file
descriptors 3, process working state 3, global hooks/subscribers 4, static registries/singletons 11,
filesystem/sockets/ports 4, time/scheduling 3, background lifetime 3, and test-runner coordination
3. Every row has exactly one primary disposition: 7 `UnifiedProcessStateLock`, 4
`ExplicitDependencyInjection`, 1 `TerminationConfirmedTeardown`, 1
`DeterministicSynchronization`, 7 `SubprocessIsolation`, 17 `ProvenConcurrencySafe`, and 1
`SeparatelyOwnedDeferred`. The reconciled row ledger and proof are in
[04-contracts-and-gates.md](../04-contracts-and-gates.md#a11d-5r2-2f0-hc-corrected-complete-process-resource-ledger),
and empirical transitions are in
[05-debug-regression-ledger.md](../05-debug-regression-ledger.md#a11d-5r2-2f0-hc-empirical-closure-record).

The environment inventory is now exactly 86 names: 74 parent-process mutations and 12 child-only
projections or read-only names. The six names omitted by the prior 80-name ledger are
`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, `SUBSTRATE_OVERRIDE_ANCHOR_MODE`,
`SUBSTRATE_OVERRIDE_ANCHOR_PATH`, and `SUBSTRATE_OVERRIDE_CAGED`. The first three are installed by
`AmbientSelectionGuard::set` in the existing authenticated-gateway negative-authority test; the
last three are arguments to the settings-test `EnvGuard::new`. Complete callsite closure corrects
the dependent total from 518 to 534 parent-mutating tests in the same 35 files. A2 is corrected
from 113 to 129 direct tests: 116 additive plus 13 existing-F0a tests.

The exact test correction is not a blind sixteen-test increment. The old scanner found 506 tests
by literal plus same-file helper traversal, then manually added twelve disjoint dynamic-wrapper
tests to publish 518. Corrected cross-file and duplicate-wrapper closure finds eighteen real
mutators the published union omitted and removes two false positives caused by ambiguous
same-file bare-name fanout, yielding 534. The added set is the named XDG gateway test,
`codex_auth_projection_errors_redact_committed_account_home`, both macOS
`update_world_env_sets_*_flags` tests, and fourteen `with_test_mode`-only PTY tests. The removed
non-mutators are the two `b1_*_acknowledgement_*` tests named in `05`.

Full wrapper closure also corrects the classification of already-listed `SUBSTRATE_SHELL`: the
host-replay test mutates it through shared `set_env`/`restore_env`, and `SpanBuilder::new` is an
overlapping stable reader. That test and file were already in the 116-test/35-file A2 manifest, so
they do not contribute to the correction delta. Its manual restoration can be bypassed by early return and is not exact
for non-Unicode prior values; the future unified-lane migration must make restoration RAII and
exact `OsString`/absence.

Complete dynamic-sink closure also adds the two existing
`execution/platform/macos.rs::platform_tests::update_world_env_sets_*_flags` tests to A2. They
were absent from the defective 518-test total, although their file already belonged to the
35-file universe, and their local `snapshot`/`restore` helper
captures only `String`/absence and restores only `SUBSTRATE_WORLD` and
`SUBSTRATE_WORLD_ENABLED` even though production-frozen `update_world_env` mutates six exact
names. Their future test-only migration must snapshot and restore all six as exact
`OsString`/absence under the existing unified environment coordinator; production
`update_world_env` remains frozen.

The prior audit resolved mutation-wrapper bodies but not every wrapper argument/callsite. Its
manual dynamic-wrapper repair likewise retained the settings tests without extracting their three
override-name arguments and omitted the gateway guard entirely. The correction therefore requires
two independent inventories—direct primitives and wrapper/callsite source closure—and fails if a
wrapper callsite, literal, constant, table entry, parameterized name, dynamic name, parent/child
classification, or mutating test lacks an explicit inventory and migration disposition. The
validator now freezes 43 dynamic mutation-sink owners, 1,005 resolved parent-mutation callsites,
and the exact 534-test manifest; source-parses the projection and local-key tables; checks every
cross-file/duplicate-name wrapper family; and runs negative perturbation checks for a missing
callsite, new table name, or `with_store` callsite.

The audit proves eight additional harness-interference families beyond the already-known HOME,
world-socket, and descriptor-capture families: ambient selector versus stable reader, current
directory versus stable path derivation, global trace-output retargeting, environment-dependent
socket-activation cache persistence, private retry-hook replacement, dispatch-tracker fixture-key
collision, mutable global-broker policy replacement, and active-PTY control replacement. It also
finds a tenth abort-without-awaited-termination test and predictable
private stop-socket paths that survive a failed test process. These findings are consolidated into
one future harness boundary; they do not create F0c/F0d/F0e packets.

At that checkpoint, the canonical sequence was:

`Routes A–E` → `F0/F0a/F0b planned` → `F0-HC corrected and authorized` →
`combined F0/F0a/F0b/Harness implementation and canonical closeout` → `F` →
`renewed R2-2 integration closeout` → `R2-3` → `R2-4` → `R3`.

That checkpoint did not complete F0, F0a, F0b, or their combined closeout. The canonical closeout
below completed them. At that historical point F and the later nodes remained unstarted; the
completed-F section below supersedes that packet status. No user-facing behavior change, production
registry, capability change, policy change, or seam promotion is authorized by F0-HC. The two
preserved F0/F0a candidates remain evidence only; the corrected post-fork-remediation full-index
SHA-256 is `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`.

## A1.1d-5R2-2F0 historical differential authority correction

`HistoricalParallelArtifactUnavailable` is the bounded evidence result. The historical clean
parallel wall remains authenticated at source commit `06c928443a93579899e5e5827b151f530e1be933`
(tree `dfe15d93366655e9855bca40ad0607887f545637`) and aggregate outcome 1,263 discovered,
1,113 passed, 150 failed, and 0 ignored. Unioning the panic headers and final-summary tail from the
same retained transcript recovers all 150 names, but complete panic output for normalized
signatures survives for only 37. The run therefore lacks a complete authenticated name/signature
artifact. Its aggregate remains diagnostic evidence of prior parallel interference, not
transition-matrix authority. A rerun of the old,
already-proven nondeterministic harness would be a new observation rather than recovery of the
historical artifact.

The corrected proof contract is an evidence-authority correction, not a waiver:

- the complete historical serial artifact is semantic authority;
- three final-candidate parallel walls and one final-candidate serial wall with identical counts,
  failure-name sets, and normalized signatures are concurrency authority; and
- the historical parallel aggregate is diagnostic evidence only.

The authoritative serial transition matrix is exactly `PassToPass=1202`, `PassToFail=0`,
`FailToSameFailure=45`, `FailToChangedFailure=0`, `FailToPass=16`, `Removed=0`,
`RenamedOrSubstituted=0`, `NewPass=17`, `NewFail=0`, and `NewIgnored=0`. Every `FailToPass` remains
subject to causal audit, and all 17 `NewPass` rows are the authorized added tests listed in `02`.
The correction does not permit a historical-parallel failure-name/signature comparison, a
parallel `PassToFail` value, a claim that every current failure belonged to the historical
parallel set, or a claim that exactly 105 named historical parallel failures became passes.

The exact 45-file candidate is preserved remotely at branch
`feat/preserve-a1-1d-5r2-2-harness-final-baseline-blocker-7ab220a7`, commit
`86ed6f5620787121b1c2e5b033ee8d6f9ff369d3`, parent
`1e1be221ca8a9f4e94af93fbda7bda6e94901d01`, and tree
`f6480f3986d43bb41e5387fa1ba5b68ae53f598b`. Its manifest SHA-256 is
`b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; per-file fingerprint
aggregate is `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; ordinary patch is
`7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; full-index patch is
`adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`; and its diff is 45 files,
5,630 insertions, and 2,104 deletions. Three implementation reviews—environment closure,
renderer/injection, and lifecycle/subprocess—are `CLEAN`. Fresh reviewer
`/root/final_containment_corrected_authority`, thread
`019f8681-7957-7cc3-88fa-37ab3ad2fc87`, returned `CLEAN` under the corrected authority model.

The correction itself changed no product behavior and started no runtime packet. The subsequently
reviewed harness closeout also changed no product or user-facing behavior and promoted no seam. At
that historical checkpoint F remained unstarted; the completed-F section below supersedes that
packet status. The binding sequence was:

`Routes A–E` → `F0/F0a/F0b/F0-HC complete` → `F` →
`renewed R2-2 closeout` → `R2-3` → `R2-4` → `R3`.

## F0/F0a/F0b/F0-HC canonical closeout

F0, F0a, F0b, and F0-HC are complete. The reviewed runtime commit is
`770a6a9de9f537f7bc179c75421abbc3fff05b8d`, tree
`61fdd2e9476f1ce3e041720ce106f7c3427895be`, preserved at
`feat/preserve-a1-1d-5r2-2-harness-review-clean-7ab220a7`. Its 45-file runtime patch is
byte-identical to the preserved candidate: 5,630 insertions and 2,104 deletions; manifest
`b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; per-file fingerprint
aggregate `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; ordinary patch
`7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; and full-index patch
`adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`.

The corrected inventory revalidated at 86 names, 74 parent-mutated, 12 child-only/read-only, 395
primitives, 73 dynamic calls, 43 sinks, 1,005 resolved callsites, 534 parent-mutating tests across
35 files, 38 resource rows, and zero unresolved dynamic or ownership rows. All 38 dispositions are
implemented or retained. Focused proof covers HOME/socket overlap, stdout/stderr injection,
XDG/account-home negative projection, environment/CWD restoration, trace/cache/hook/tracker/broker/
ACTIVE_PTY isolation, fork publication, async termination, subprocess isolation, exact absence and
non-Unicode restoration, panic/poison/nesting/reacquisition, and child failure propagation.

Three fresh parallel walls and one fresh serial wall each discovered 1,280 tests, passed 1,235,
failed 45, and ignored 0. Their failure-name set SHA-256 is
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; their consistently
normalized signature set SHA-256 is
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.
Historical serial semantic authority yields exactly 1,202 `PassToPass`, zero `PassToFail`, 45
`FailToSameFailure`, zero `FailToChangedFailure`, 16 causally audited `FailToPass`, zero removed or
renamed/substituted tests, 17 exact authorized `NewPass`, zero `NewFail`, and zero `NewIgnored`.

Historical parallel evidence remains diagnostic only: `1263/1113/150/0`, all 150 names recovered,
37 complete panic bodies, and 113 normalized signatures unavailable. No historical parallel
transition matrix, 105-transition claim, or reconstructed signature is asserted. GitNexus reports
45 files and 365 changed indexed symbols; its sole medium process attribution resolves to
`AuthorityEnvTestTempDir::new` under `#[cfg(test)]`, not a production flow. The only production hunk
is the authorized mechanical F0b writer delegation with unchanged output/error behavior. The prior
three implementation reviewers and the fresh containment reviewer are `CLEAN`. There is no product
or user-facing behavior change and no seam promotion. F was not started.

At that historical checkpoint the exact next packet was **A1.1d-5R2-2F — Authenticated world-deps
and truthful doctor composition**. The later historical F5-PD correction, whose boundary the
completed-F record below retains, superseded this next-task statement.

## A1.1d-5R2-2F historical readiness-boundary correction

This section records the historical F3/F4 readiness correction and superseded only the earlier
statements that F was wholly unstarted and the earlier F request-builder/readiness allowlist.
Historical F0/F0a/F0b/F0-HC evidence remains unchanged.

| F checkpoint | Canonical status |
|---|---|
| F1 | Complete and locally committed as `eae02af959f0b7066015bb242ffa45fc7a01d591` (`feat: bind authenticated world-deps context`), tree `7bd7e8b5ca2d563b2991b4fea62ca4ea26266b0c`. |
| F2 | Complete and locally committed as `d30d8cec764e2338fb48475747733091d3af22bf` (`feat: propagate authenticated world-deps scope`), tree `a6161c7664fa6ca1173302dd3570b5d8d7e09ecc`. |
| F3/F4 | Incomplete and blocked. The exact seven-file candidate is preserved at `a343f0796d19d66c168c5bb2797856710cff5708`, tree `a377daa6f41693454e38c39163cc9895bbf3e828`, on `feat/preserve-a1-1d-5r2-2f-f3-f4-blocked-7224dd30`. Its ordinary/full-index patch SHA-256 values are `7224dd30c04e5fcd85913bfdddb62deb497f0b3eaba416b2f689d392f47b9b0c` and `76fb0c5c183880dca8f31576647c8e9372d1ad026e2a731b48d5ef2642ef0a22`. Preservation is evidence, not approval. |
| F5 and final F walls | Unstarted. Renewed R2-2 integration closeout, R2-3, R2-4, and R3 remain blocked and unstarted. |

The blocking review found two coupled boundary defects. First, the candidate copied ambient
`SUBSTRATE_*` and `WORLD_*` values into authenticated world requests, allowing authority selectors,
credentials, tokens, configuration overrides, and conflicting B state to cross beside A. Second,
the candidate connected directly to the fixed Linux socket without invoking the existing
`world_ops.rs::ensure_world_service_ready` lifecycle owner. Duplicating its probe, activation,
stale-socket, spawn, timeout, or error logic is forbidden.

Future F3/F4 work may therefore add one private Linux explicit-target readiness core in
`world_ops.rs`. The existing no-argument owner remains the compatibility entry point and every
existing caller remains behaviorally unchanged. The authenticated F builder supplies the fixed
product socket `/run/substrate.sock` and an immutable installed-product service posture; it never
selects a socket or service binary from the environment, HOME/XDG state, CWD, or a global side
table. Request construction occurs only after authenticated context validation and successful
explicit readiness.

The outbound command environment is an exact generated projection, not inherited state:
`SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR`, `PATH`, `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`,
`XDG_CACHE_HOME`, and `TERM`, each set to the fixed guest-runtime value specified in `04`. No
ambient value is forwarded, including locale values. Structured request fields remain the sole
transport for CWD, profile, policy snapshot, network policy, and filesystem mode; authority,
policy, socket, carrier, credential, prompt, and request preimage values are excluded from the
command environment.

That correction changed no production capability or policy, promoted no seam, and authorized no
runtime implementation in that documentation packet. At that checkpoint F remained incomplete;
the exact next task was to resume **A1.1d-5R2-2F3/F4** under the explicit-readiness and
exact-environment contracts, and F5/final walls could not begin.

## A1.1d-5R2-2F5-PD canonical correction

This latest section supersedes the live-status and next-task wording above while retaining it as
historical readiness-boundary evidence. F3 and F4 are complete local runtime increments at the
pre-documentation-replay commits `54a5ce663cf2724b69b0c124acf2c2758ff622dd` and
`7802c44198625ea6849933140c390900ebe84190`. Their clean wall discovered 1,300 tests, passed 1,255,
failed 45, and ignored 0. The retained failure-name and normalized-signature SHA-256 values are
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` and
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`. These facts complete only
F3/F4; they do not complete F.

The attempted F5 candidate proved 13 decoder tests and 17 of 18 shim-doctor integration tests, then
stopped at the nested-child lifecycle boundary. It is preserved remotely at
`feat/preserve-a1-f5-blocked-candidate-20260722`, commit
`c4b93506480ae5e9708417067980e08e0e754b7a`, parent
`7802c44198625ea6849933140c390900ebe84190`, tree
`03bec2f517866e5213b8a0bed9ad99058c25ebc3`. Preservation is archaeology, not authorization; the
candidate must not be restored wholesale. Its exact four-file manifest and hashes are recorded in
`05-debug-regression-ledger.md`.

The contradiction is now resolved by the bounded prerequisite
**A1.1d-5R2-2F5-PD — Non-mutating nested doctor boundary**. The normal public
`substrate world doctor --json` remains behaviorally unchanged and is not represented as
side-effect-free. Only the authenticated F5 composition child receives a hidden, typed passive
mode. That child validates the canonical hidden argv carrier and current principal before any
projection installation, home scaffold, trace initialization, config/policy selection, platform
resolution, readiness, socket, service, transport, world, or probe action. It then returns a
bounded fail-closed diagnostic. No existing durable runtime-health artifact is authoritative for
this purpose, so the initial passive result is honestly `unavailable`; it never fabricates
coherent success.

Whenever authenticated Linux composition has World enabled and reaches
`gather_world_doctor_snapshot`, it invokes that child and no longer selects
`A/health/world_doctor.json`. The existing World-disabled `build_report` short-circuit remains
unchanged: it returns `disabled_world_doctor_snapshot` without a child or fixture lookup. On the
enabled passive path World Doctor fixtures remain `cfg(test)` evidence only. Existing non-Linux
compatibility stays behavior-frozen,
including its legacy fixture/public-child mechanics; it is forbidden evidence for F5-PD/F5 and
cannot claim A-bound or native proof. The Linux parent
requires duplicate-rejecting raw JSON decode plus exact schema/exit/A identity, rejects mixed or
secret/request-bearing payloads, discards raw child JSON and stderr, and maps only to the existing bounded unavailable/incoherent
`NeedsAttention` representation. The hidden field lives on `WorldAction::Doctor`, leaving the
top-level `Cli` and its `auto_sync.rs` literal unchanged. This additive Rust enum layout does not
change normal public `world doctor --json` CLI behavior.

At the completed Routes A–F checkpoint, the historical binding sequence was:

`Routes A–E` → `F0/F0a/F0b/F0-HC complete` → `F1/F2 complete` → `F3/F4 complete` →
`F5-PD` → `F5` → `final F walls` → `F closeout` →
`renewed R2-2 integration closeout` → `R2-3` → `R2-4` → `R3`.

F5-PD is a prerequisite beneath F5, not a new slice and not a seam promotion. It changes no world
execution capability, policy, network, filesystem, caging, placement, credential, service, or
lifecycle behavior. Public World doctor compatibility is frozen. Authenticated shim/Health
composition will no longer activate infrastructure merely to collect its nested diagnostic; when
passive evidence is absent, user-visible composition reports unavailable rather than success or
implicit activation. At that checkpoint, F5, the final F walls, F closeout, renewed R2-2
closeout, R2-3, R2-4, and R3 remained unstarted.

The exact next task at that checkpoint was **Implement F5-PD, re-review it, then resume F5**. The
canonical F closeout below supersedes that live-status conclusion without changing the historical
boundary decision.

## A1.1d-5R2-2F canonical closeout

F1 through F5, including the inserted F5-PD passive boundary, are complete. The exact runtime proof
sequence before this documentation closeout and its required replay is:

`922e1792` F1 → `77fdcd8a` F2 → `0cd1d7af` F3 → `af2a3da6` F4 → `653a7d91` F5-PD →
`2bb4696d` F5.

F5 is commit `2bb4696d7181d974c1b02e33d82e09422cdae7de`, parent
`653a7d91489563bc2a8e3395feeb53e159254240`, tree
`313a8a613a6cd91e72c0cc1664f4b9842fefa305`, subject
`fix: compose truthful authenticated doctor evidence`. It changes only
`crates/shell/src/builtins/{shim_doctor/report.rs,world_deps/mod.rs}` and
`crates/shell/tests/{shim_doctor.rs,shim_health.rs}`. Its ordinary patch, full-index patch, and
stable patch ID are respectively
`991859870123dca56f3ee080876742abb2e556f9ee0bdf62f3577fdf1ccfb189`,
`8e2be2f2b84dc5d3cd573b9670e4d3eccc3060893f478a414d057e39b002a943`, and
`b768cb94c51802e2440fbf1eb7f29fc0693a446b`.

Authenticated Linux World-enabled composition now joins the world-deps snapshot to exact A and
reports bounded unavailable truth because no permitted passive runtime-health evidence exists.
Fixtures remain compatibility/test evidence, must match A when they participate, cannot establish
runtime health, and cannot be retained as production truth. Missing evidence is unavailable;
mixed, malformed, duplicate, unknown, tampered, partial, secret-bearing, or conflicting evidence
is incoherent/fail-closed. The World-disabled branch remains child- and fixture-free. The
no-fixture World branch still uses the authenticated passive child and cannot reach readiness,
socket activation, world-service, `/v1/doctor/world`, `/v1/execute`, active probes, world creation,
repair, provisioning, cleanup, or synchronization. Public `world doctor --json` is unchanged.

The final shell-library authority is three default-parallel walls and one one-thread serial wall,
each with 1,309 discovered, 1,264 passed, 45 failed, and 0 ignored. The inherited failure-name and
normalized-signature SHA-256 values remain exactly
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` and
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`. Relative to clean F5-PD,
the two new library tests are `NewPass`; the three new integration tests also pass in their exact
suites. `PassToFail`, `NewFail`, `FailToChangedFailure`, removed, renamed/substituted, newly ignored,
and weakened are all zero.

Raw shell all-target Clippy remains truthfully non-green only because the exact inherited 20
`clippy::needless_borrow` findings are still present in frozen code. The documented differential
that allows only that lint passes with every other warning denied. Shell/workspace all-target
checks, formatting, diff checks, final GitNexus containment, and three fresh whole-F reviews pass.
Linux is the only authenticated F proof platform; non-Linux behavior remains frozen compatibility
or unavailable/unproven. Managed secure-FD behavior, direct-member compatibility, policy, network,
filesystem, caging, capability, lifecycle, receipt, supervisor, cleanup, and rollback ownership are
unchanged.

At the completed Routes A–F checkpoint, the then-binding sequence was:

`Routes A–F complete` → `renewed R2-2 production-fix-free integration closeout` → `R2-3` →
`R2-4` → `R3`.

At that completed Routes A–F checkpoint, the exact historical next task was **A1.1d-5R2-2
renewed production-fix-free integration closeout**. The current remediation-planning authority
[in the renewed-closeout current-state record](../a1.1d-5r2-2-renewed-closeout/current-state.md#a11d-5r2-2-closeout-remediation-planning-authority-historical-rp0-checkpoint)
supersedes that historical next-task statement without rewriting its result.

**Source provenance:**
- extracted from [`00-README.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction`](../00-README.md#a11d-5r2-2f0-hc-shell-harness-closure-audit-and-environment-correction), lines 454–793; baseline span SHA-256 `56cc72e5d86e5cd9ac4779c041cd1ab7af3d83cab251d89b512879f581716935`
