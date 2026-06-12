# TASKS-06: Routed-Path-First Doctor, Smoke, and Readiness Truth

Source spec:
- [`SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`](./SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md)

Source plan:
- [`PLAN-06.md`](./PLAN-06.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `06` as the next seam,
2. Slice `07` remains reserved for listener-surface hardening,
3. Slice `12` remains reserved for broader breakglass reclassification and docs
   cutover,
4. the implementation owner accepts that this slice is source-driven and must
   use current official Lima shell, SSH, port-forwarding, environment-variable,
   and breaking-change docs,
5. if any runtime symbol in `crates/shell/src/execution/platform/macos.rs`
   needs to change, GitNexus impact analysis is treated as mandatory before the
   edit and `gitnexus_detect_changes()` is treated as mandatory before
   committing,
6. warm/provision redesign, listener removal, ingress/mount hardening, and
   guest-unit sandbox work remain out of default scope.

## Slice contract

This slice should land one explicit routed-path-first readiness contract for:

1. happy-path macOS readiness validation through `substrate host doctor`,
   `substrate world doctor`, and gateway lifecycle/status surfaces,
2. helper-script proof order that fails on routed readiness problems before
   guest-direct success can mask them,
3. readiness-oriented docs that lead with owned CLI proof and classify direct
   guest access as breakglass or post-failure diagnosis,
4. optional small doctor-output clarity work only if required to explain the
   routed contract honestly.

This slice must **not**:

1. redesign backend transport or policy carriers,
2. replace `scripts/mac/lima-warm.sh`,
3. remove every internal `limactl shell` use,
4. widen into listener, ingress, mount, guest-unit, or ownership-boundary
   hardening,
5. perform a full feature-wide docs cutover beyond readiness-oriented paths.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `scripts/mac/lima-doctor.sh`,
3. `scripts/mac/smoke.sh`,
4. `docs/WORLD.md`,
5. `docs/reference/world/platforms/macos-lima-setup.md`,
6. `docs/USAGE.md` only if readiness wording there needs alignment,
7. `docs/contracts/gateway/operator-contract.md` only if wording alignment is
   required,
8. `docs/contracts/gateway/status-schema.md` only if wording alignment is
   required,
9. `crates/shell/src/execution/platform/macos.rs` only if the cutover needs a
   small routed-evidence clarity change,
10. targeted tests and syntax checks for the touched surfaces only.

Treat edits to `scripts/mac/lima-warm.sh`, `crates/world-mac-lima/`,
`crates/world-api/`, or later hardening/doc-cutover surfaces as scope expansion
unless the orchestrator can point to a direct contradiction that the user
explicitly approves for this slice.

## Execution packets

### Packet 1: Readiness-order freeze and source gate

Session goal:

1. confirm the authority stack, official source set, and live repo drift,
2. freeze the exact routed-path-first readiness order,
3. inventory the script/doc sections that still normalize guest-direct proof.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and live readiness drift
  - Acceptance: the implementation pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `1`, milestone `1.3`,
    `DESIGN-macos-lima-transport-contract.md`,
    `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and the official Lima
    shell, SSH, port-forwarding, environment-variable, and breaking-change
    docs. The pass also records the relevant repo-truth drift in
    `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`,
    `docs/WORLD.md`, `docs/reference/world/platforms/macos-lima-setup.md`, and
    `crates/shell/src/execution/platform/macos.rs`.
  - Verify:
    - manual authority review
    - manual official-source review
    - manual repo-truth review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md`

- [x] Task 1.2: Freeze the canonical readiness evidence order and scope boundary
  - Acceptance: the slice makes it explicit that the happy-path readiness order
    for an already provisioned backend is:
    `substrate host doctor` / `substrate world doctor` first, gateway
    lifecycle/status second, routed smoke proof third, and guest-direct
    diagnosis only after failure or as clearly labeled breakglass. The docs
    also make explicit that `SUBSTRATE_WORLD_SOCKET` remains
    advanced/test/breakglass on macOS.
  - Verify:
    - `rg -n "host doctor|world doctor|gateway status|gateway sync|gateway restart|limactl shell|SUBSTRATE_WORLD_SOCKET" scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md docs/USAGE.md`
    - manual coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-06-routed-path-first-doctor-smoke-and-readiness-truth.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-06.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-06.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the happy-path readiness order is explicit,
2. the contradiction surfaces are explicit,
3. the slice still has not widened into lifecycle redesign or broader docs
   cutover.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Align helper scripts to routed-path-first proof

Session goal:

1. make `scripts/mac/lima-doctor.sh` fail on routed readiness problems first,
2. make `scripts/mac/smoke.sh` prove routed readiness and gateway lifecycle
   before guest-direct diagnosis,
3. preserve direct guest commands only as fallback or breakglass.

#### Tasks

- [x] Task 2.1: Reframe `scripts/mac/lima-doctor.sh` around owned doctor proof
  - Acceptance: `scripts/mac/lima-doctor.sh` no longer reads as a guest-first
    health script. It now leads with owned CLI doctor surfaces for readiness
    validation, and any remaining guest-direct checks are clearly labeled
    fallback or repair-oriented checks. Routed readiness failure is able to
    fail the script even if guest-direct socket or service checks succeed.
  - Verify:
    - `bash -n scripts/mac/lima-doctor.sh`
    - `rg -n "host doctor|world doctor|limactl shell|systemctl|curl" scripts/mac/lima-doctor.sh`
    - manual success/failure-path review
  - Files:
    - `scripts/mac/lima-doctor.sh`

- [x] Task 2.2: Reframe `scripts/mac/smoke.sh` around routed smoke proof first
  - Acceptance: `scripts/mac/smoke.sh` proves routed doctor/gateway/PTY/non-PTY
    readiness before any direct guest `curl`, binary checks, or guest
    `systemctl` checks participate in the proof story. Any remaining
    guest-direct checks are clearly post-failure diagnostics or compatibility
    evidence rather than the happy path.
  - Verify:
    - `bash -n scripts/mac/smoke.sh`
    - `rg -n "gateway sync|gateway status|gateway restart|world doctor|limactl shell|systemctl|curl" scripts/mac/smoke.sh`
    - manual proof-order review around the readiness sections
  - Files:
    - `scripts/mac/smoke.sh`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. the helper scripts encode the canonical readiness order,
2. routed-path failures can no longer be masked by guest-direct success in the
   intended proof flow,
3. guest-direct checks remain available only as explicit fallback/breakglass.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Cut over readiness-oriented docs and optional doctor clarity

Session goal:

1. make the readiness docs match the new routed-path-first script truth,
2. adjust owned doctor output only if the docs/scripts still need clearer
   routed evidence.

#### Tasks

- [ ] Task 3.1: Update readiness-oriented macOS docs to lead with owned CLI proof
  - Acceptance: the relevant readiness sections in `docs/WORLD.md` and
    `docs/reference/world/platforms/macos-lima-setup.md` lead with
    `substrate host doctor`, `substrate world doctor`, and gateway
    lifecycle/status flows for an already provisioned backend. Direct guest
    `limactl shell`, guest `systemctl`, guest `curl`, and guest journal usage
    remain documented only as breakglass or post-failure diagnosis. Same-user
    limitations remain explicit.
  - Verify:
    - `rg -n "host doctor|world doctor|gateway status|gateway sync|gateway restart|limactl shell|systemctl|curl|journalctl" docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
    - manual wording review against the support taxonomy
  - Files:
    - `docs/WORLD.md`
    - `docs/reference/world/platforms/macos-lima-setup.md`
    - `docs/USAGE.md` only if alignment is required
    - `docs/contracts/gateway/operator-contract.md` only if alignment is required
    - `docs/contracts/gateway/status-schema.md` only if alignment is required

- [ ] Task 3.2: Make small owned-doctor clarity changes only if the cutover requires them
  - Acceptance: `crates/shell/src/execution/platform/macos.rs` changes only if
    the script/docs cutover proves that the current owned doctor surfaces are
    too unclear about routed versus guest-direct fallback evidence. If no such
    contradiction exists, leave the file untouched. If the file is edited,
    GitNexus impact analysis must be recorded before editing the touched
    symbol(s).
  - Verify:
    - `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus status`
    - `cargo test -p shell doctor_ok_json -- --nocapture`
    - `cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture`
    - `cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture`
  - Files:
    - `crates/shell/src/execution/platform/macos.rs` only if required

### Packet 3 checkpoint

Packet `3` is complete only when:

1. readiness-oriented docs match the routed-path-first contract,
2. guest-direct instructions are clearly exceptional,
3. any optional runtime-code change remained small and justified.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final validation and next-slice handoff clarity

Session goal:

1. validate that Slice `06` stayed narrow,
2. run final targeted verification across scripts, docs, and optional doctor
   output,
3. leave a clean handoff to Slice `07` and the later broader docs-cutover
   slice.

#### Tasks

- [ ] Task 4.1: Final targeted regression and scope check
  - Acceptance: script syntax checks pass, targeted shell tests remain green,
    and the final diff remains limited to the intended readiness-oriented
    scripts/docs plus any tightly justified doctor-output clarity change.
  - Verify:
    - `bash -n scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
    - `cargo test -p shell doctor_ok_json -- --nocapture`
    - `cargo test -p shell world_doctor_json_uses_override_vm_name -- --nocapture`
    - `cargo test -p shell world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned -- --nocapture`
    - `cargo test -p shell macos_gateway_client_endpoint -- --nocapture`
    - `git diff --stat -- scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md docs/USAGE.md docs/contracts/gateway/operator-contract.md docs/contracts/gateway/status-schema.md crates/shell/src/execution/platform/macos.rs macos-hardening/macos-hardened-same-user-lima/spec`
    - `git status --short`
    - `gitnexus_detect_changes()` before committing if runtime symbols changed
  - Files:
    - touched files only

- [ ] Task 4.2: Record the handoff boundary honestly
  - Acceptance: the final closeout states explicitly that Slice `06` landed the
    readiness proof-story cutover only. Listener removal remains Slice `07`,
    and broader breakglass/docs cutover remains Slice `12`.
  - Verify:
    - manual closeout review
  - Files:
    - implementation closeout or PR description

### Packet 4 checkpoint

Packet `4` is complete only when:

1. the slice remained readiness-scoped,
2. verification evidence shows routed-path-first proof clearly,
3. the next deferred seams are stated plainly.
