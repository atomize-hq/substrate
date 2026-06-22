# macOS Hardened Same-User Lima Audit Findings

Date: 2026-06-22  
Repo: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Primary landing reviewed: commit `928ef583a` (`Feat/macos hardening (#78)`)

## Purpose

Capture the full set of findings from the multi-review audit of
`macos-hardening/macos-hardened-same-user-lima/`, including:

- code/runtime issues that still need fixing,
- docs that need alignment or cleanup,
- places where the current proof story is weaker than the docs claim,
- places where the landing is real but the authority stack is stale.

## Overall verdict

The main hardening landing is real in committed code, but it is **not fully
review-clean**.

Broadly:

- the key runtime slices mostly landed correctly,
- the original research is still mostly aligned at the threat-model level,
- the biggest remaining problem is **doc authority drift**,
- there are also a few **medium proof / architecture / source-of-truth gaps**
  that should be cleaned up before calling the feature fully aligned.

## Confirmed landed surfaces

These are the main areas that looked genuinely landed in committed code:

1. **Policy-input parity now fail-closes**
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:431-452`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:635-690`

2. **Default extra-listener removal landed**
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/lib.rs:155-192`

3. **Ingress hardening / staged-workspace cutover landed**
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/substrate.yaml:10-17`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:282-369`

4. **Canonical unit template flow landed**
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/units/substrate-world-service.service.tmpl:11-29`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/units/substrate-world-service.socket:5-12`

5. **Routed-path-first doctor/smoke/operator posture is real**
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh:300-347`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:349-453`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/macos-lima-setup.md:5-27`

## Findings that need action

### High

#### 1. Feature-level and phase-level authority docs are stale and contradict HEAD

This is the biggest issue.

Stale docs still describe pre-landing drift as if it is current truth:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/README.md:23-29`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/README.md:14-18`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/README.md:28-31`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md:17-20`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/README.md:21-29`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/README.md:71-83`

These docs still imply or state that HEAD has:

- permissive backend policy synthesis,
- default TCP injection / broader listener posture,
- broad `$HOME` mounts in the hardened default,
- guest unit source-of-truth drift,
- not-yet-cut-over operator docs.

But current repo truth says otherwise:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:426-437`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:635-647`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/substrate.yaml:16-17`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:282-340`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:785-822`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:916-926`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:201-240`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/macos-lima-setup.md:5-27`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/macos-lima-setup.md:292-320`

**Required action**

- Rewrite the feature README plus phase READMEs so they describe current HEAD,
  not pre-landing intent.
- Preserve what is still genuinely incomplete, but stop describing already-landed
  seams as open.

#### 2. `ROADMAP.md` is no longer trustworthy as live execution authority

Stale file:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/ROADMAP.md:205-212`

Conflicting closeout authority:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md:309-326`

`ROADMAP.md` still reads like a bootstrap doc for creating the `spec/` stack,
even though `SPEC-01` through `SPEC-12` already exist and Slice 12 records
Phase 3 closeout.

**Required action**

- Either update `ROADMAP.md` into a post-landing retrospective/current-state
  roadmap, or explicitly demote/archive it so it stops implying it is live
  authority.

### Medium

#### 3. `docs/WORLD.md` still advertises a retired verification path

Doc says this is a valid mode:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:238-243`

But the script hard-fails it:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:940-943`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:996-998`

Help text still advertises it:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:22-40`

**Required action**

- Remove or rewrite the retired `--bedpm-installer-conformance` references in:
  - `docs/WORLD.md`
  - `scripts/mac/smoke.sh` help text
- Re-run the Slice 12 doc-cutover review wall afterward.

#### 4. Breakglass relabeling is incomplete in `docs/WORLD.md`

Phase 3.2 says remaining guest-direct commands should be explicitly marked
breakglass:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md:104-115`

But `docs/WORLD.md` still includes insufficiently labeled guest-direct examples:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:210-211`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:471-476`

**Required action**

- Mark these examples in place as breakglass/advanced, or replace them with the
  owned CLI path where possible.

#### 5. Rendered-unit parity proof does not obviously use the same source
resolution as warm

Docs imply the parity proof checks the same rendered-unit inputs warm used:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:238-239`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/macos-lima-setup.md:316-320`

Warm resolves canonical units from project-path or script-dir:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:86-94`

Doctor/smoke hardcode repo-root unit sources instead:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh:7-8`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh:151-187`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:10-12`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:139-174`

**Why this matters**

If warm provisioned from a different project/release tree than the repo-root
tree used by doctor/smoke, the parity proof may not be checking the exact same
inputs.

**Required action**

- Make doctor/smoke resolve unit sources through the same logic/path contract
  warm uses, or
- weaken the doc claim so it does not overstate what parity is being proven.

#### 6. Obsolete pre-unification macOS service authority remains in-tree and is
still referenced

Current canonical source:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/units/substrate-world-service.service.tmpl:11-29`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/units/substrate-world-service.socket:5-12`

Obsolete file still present:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/substrate-world-service.service:6-26`

Still referenced by internal docs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/deps.md:29-35`

**Required action**

- Delete or clearly deprecate the old standalone service file.
- Update internal docs to point only at the Lima unit template/socket authority.

#### 7. Slice 04 transport architecture did not fully converge to one shell-side
authority

Promised by the slice docs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md:178-184`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md:350-353`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-04.md:131-135`

Actual code still has duplicated transport-ordering/probing logic:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:208-256`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs:88-112`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs:179-183`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs:292-334`

**Required action**

- Either finish the architectural convergence, or
- update the slice docs so they describe the actual split honestly.

#### 8. `17788` still has split semantics between “selected transport” and
“transport the backend can actually realize”

Promise:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-03-canonical-guest-endpoint-and-transport-contract.md:87-99`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:223-229`

Current behavior:

- transport auto-selects TCP in some cases:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs:54-67`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs:79-100`
- backend forwarding intentionally skips SSH TCP fallback:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:93-118`

**Required action**

- Tighten code or docs so “selected transport” and “realizable forwarding path”
  are not described as if they are always the same thing.

#### 9. Normal sync/copy is still only partially productized

Current code still says `workspace sync` is not yet the frozen normal macOS
same-user Lima sync/copy path:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs:532-538`

Current real operator contract still depends on:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:916-926`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh:300-304`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/macos-lima-setup.md:7-14`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/world/platforms/macos-lima-setup.md:150-173`

**Required action**

- Do not overclaim “full lifecycle/sync productization landed.”
- Either keep docs explicit about the degraded-but-supported interim sync path,
  or do the remaining productization work and then update docs.

### Low

#### 10. Canonical guest socket path is not fully single-sourced in shell doctor
fallback code

Authority exists here:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/transport.rs:7-17`

But shell doctor fallback still hardcodes the socket path:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs:186-206`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs:357-373`

**Required action**

- Optional cleanup: route these through the shared authority if practical.

#### 11. Support taxonomy wording is not propagated consistently

Canonical taxonomy:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/README.md:64-76`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md:75-80`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md:114-129`

Inconsistent later wording:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/README.md:53-57`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md:83-86`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md:55-60`

**Required action**

- Normalize on one exact label set:
  - `supported`
  - `degraded-but-supported`
  - `breakglass`

#### 12. Internal transport doc is stale

Stale internal doc:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/transport-parity.md:96-109`

Current code:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/lib.rs:155-192`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/forwarding.rs:109-118`

**Required action**

- Update or archive the internal transport doc so it no longer describes the
  old dual-listener posture with `SUBSTRATE_AGENT_TCP_PORT=61337`.

#### 13. `scripts/mac/smoke.sh` help is internally inconsistent

Current issues:

- synopsis omits `--world-disabled-diagnostics` even though the option exists:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:22-40`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:35-36`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:982-983`
- same help block still advertises retired `--bedpm-installer-conformance`

**Required action**

- Fix the smoke help text to match the real command surface.

## Better proof still needed

These are the main places where the current proof story is weaker than claimed:

1. **Rendered-unit parity proof**
   - Prove doctor/smoke are checking the same source inputs that warm used, or
     narrow the documentation claim.

2. **Transport semantics proof**
   - Add explicit proof or tests around when `17788` is only compatibility
     naming versus when a realizable forwarding path actually exists.

3. **Doc-cutover proof**
   - After doc cleanup, rerun the Slice 12 review wall against:
     - `docs/WORLD.md`
     - `docs/reference/world/platforms/macos-lima-setup.md`
     - `docs/USAGE.md`
     - `scripts/mac/lima-doctor.sh`
     - `scripts/mac/lima-warm.sh`
     - `scripts/mac/smoke.sh`
     - `scripts/mac/orchestration-smoke.sh`

4. **Source-of-truth proof for units**
   - Either remove the old standalone macOS service file, or prove it is
     intentionally retained and never authoritative.

## Suggested remediation order

1. **High priority docs cleanup**
   - feature README
   - phase READMEs
   - `ROADMAP.md`

2. **Slice 12 cutover cleanup**
   - remove retired smoke mode references
   - relabel breakglass examples in `docs/WORLD.md`
   - fix smoke help text

3. **Unit/source-of-truth cleanup**
   - align doctor/smoke unit-source resolution with warm
   - remove/deprecate old standalone service file
   - fix internal docs still pointing at obsolete authority

4. **Architecture/proof cleanup**
   - transport helper convergence
   - clarify `17788` semantics in code/docs/tests

5. **Optional polish**
   - single-source `/run/substrate.sock` in shell doctor fallback
   - normalize taxonomy wording everywhere

## Landing task breakdown

This section breaks the remediation into separate landing tasks. Each task has a
clear objective and explicit acceptance criteria for considering it complete.

### Task 1: Rewrite the feature-level README to match current HEAD

**Objective**

Update
`/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/README.md`
so it describes the current landed same-user Lima posture rather than the
pre-landing or mid-landing state.

**Acceptance criteria**

- The README no longer claims that HEAD still depends on permissive backend
  policy synthesis, default TCP injection, broad default host-home mounts,
  guest-unit drift, or pending operator-doc cutover.
- The README clearly distinguishes:
  - what is landed,
  - what is still degraded-but-supported,
  - what remains breakglass,
  - what is still not fully productized.
- README wording is consistent with:
  - `docs/WORLD.md`
  - `docs/reference/world/platforms/macos-lima-setup.md`
  - `spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md`

### Task 2: Rewrite all phase README files to current repo truth

**Objective**

Bring the phase-level authority docs into alignment with committed HEAD so they
describe current status honestly.

**Acceptance criteria**

- The following files are updated to stop describing already-landed seams as
  open:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/README.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/README.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md`
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/macos-hardening/macos-hardened-same-user-lima/phase-3-substrate-owned-operations/README.md`
- Each phase README explicitly states:
  - what that phase landed,
  - what remains intentionally deferred,
  - what later docs or surfaces are now authoritative.
- No phase README contradicts the code or operator docs on:
  - policy parity,
  - listener posture,
  - mount posture,
  - unit authority,
  - operator cutover status.

### Task 3: Resolve `ROADMAP.md` authority drift

**Objective**

Make `ROADMAP.md` either truthful as a post-landing/current-state roadmap or
explicitly archival so it no longer reads like live execution authority.

**Acceptance criteria**

- `ROADMAP.md` no longer instructs readers to create a `spec/` scaffold or begin
  `SPEC-01` / `SPEC-02` as if those steps are still pending.
- The file either:
  - reflects the existence and closeout status of `SPEC-01` through `SPEC-12`,
    or
  - is clearly marked superseded/archival and points readers to the live
    authority stack.
- `spec/README.md`, `ROADMAP.md`, and `EXECUTION-RUBRIC.md` do not conflict on
  what is the current planning/execution authority.

### Task 4: Finish Slice 12 doc cutover for retired smoke paths

**Objective**

Remove or correct documentation and help text that still advertises retired or
invalid smoke modes.

**Acceptance criteria**

- `docs/WORLD.md` no longer presents
  `scripts/mac/smoke.sh --bedpm-installer-conformance` as a live verification
  surface.
- `scripts/mac/smoke.sh --help` no longer advertises retired modes.
- `scripts/mac/smoke.sh --help` includes all actually supported flags,
  including `--world-disabled-diagnostics` if it remains supported.
- `bash -n scripts/mac/smoke.sh` passes after the edits.

### Task 5: Complete breakglass relabeling in top-level macOS docs

**Objective**

Make all remaining guest-direct macOS instructions in top-level docs explicitly
match the supported / degraded-but-supported / breakglass taxonomy.

**Acceptance criteria**

- `docs/WORLD.md` explicitly labels direct `limactl shell`, guest `systemctl`,
  guest `journalctl`, guest socket `curl`, and host-side
  `SUBSTRATE_WORLD_SOCKET` override use as breakglass or advanced where
  appropriate.
- No guest-direct example in `docs/WORLD.md` appears inline as a normal default
  operator step without taxonomy context.
- Wording in `docs/WORLD.md`, `docs/reference/world/platforms/macos-lima-setup.md`,
  `scripts/mac/lima-doctor.sh`, and `scripts/mac/lima-warm.sh` is aligned.

### Task 6: Align rendered-unit parity proof with warm’s unit-source resolution

**Objective**

Ensure doctor/smoke parity checks are proving the same unit inputs that
provisioning/warm actually used, or narrow the claim if that stronger proof is
not possible.

**Acceptance criteria**

- `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` resolve unit sources
  through the same project-path/script-dir logic as `scripts/mac/lima-warm.sh`,
  or the docs are explicitly narrowed to describe the weaker proof honestly.
- The proof path is documented clearly in:
  - `docs/WORLD.md`
  - `docs/reference/world/platforms/macos-lima-setup.md`
- A reviewer can explain exactly which unit inputs are being compared, from
  which tree, and why that proves parity.

### Task 7: Clean up obsolete macOS unit authority

**Objective**

Remove ambiguity around which macOS unit files are authoritative.

**Acceptance criteria**

- The repo has one clearly authoritative macOS same-user Lima unit source:
  - the files under `scripts/mac/lima/units/`
- The old standalone file
  `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/substrate-world-service.service`
  is either:
  - deleted, or
  - clearly marked obsolete/non-authoritative and not referenced as live
    guidance.
- Internal docs such as
  `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/world/deps.md`
  point only at the true unit authority.

### Task 8: Update stale internal transport and dependency docs

**Objective**

Bring internal docs back into line with the landed hardening behavior.

**Acceptance criteria**

- `docs/internals/world/transport-parity.md` no longer describes the old
  dual-listener default or stale `SUBSTRATE_AGENT_TCP_PORT=61337` assumptions.
- Any internal docs still implying older listener or unit-authority models are
  updated or archived.
- Internal docs do not contradict:
  - `crates/world-service/src/lib.rs`
  - `crates/world-mac-lima/src/forwarding.rs`
  - `scripts/mac/lima/units/*`

### Task 9: Decide and document the transport-authority architecture honestly

**Objective**

Either finish the intended shell-side transport convergence or document the
remaining split honestly so Slice 04 is no longer overstated.

**Acceptance criteria**

- One of the following is true:
  1. transport/probing logic is actually consolidated enough that the slice
     claim is true, or
  2. the slice docs (`SPEC-04`, `PLAN-04`, related docs) are updated to state
     the remaining duplication and why it is acceptable.
- A reviewer can point to the authoritative place(s) for:
  - PTY/non-PTY transport choice,
  - routed doctor probing,
  - readiness fallback behavior.
- There is no misleading “single authority” claim left if the code is still
  intentionally split.

### Task 10: Clarify and prove `17788` semantics

**Objective**

Resolve the gap between “selected transport” semantics and “forwarding path the
backend can actually realize,” especially around compatibility TCP.

**Acceptance criteria**

- Code and docs clearly distinguish:
  - canonical guest endpoint,
  - compatibility host transport naming,
  - forwarding modes the backend will actually establish.
- `docs/WORLD.md`, transport docs, and `crates/world-mac-lima` code do not
  imply that SSH TCP fallback exists when it is intentionally skipped.
- Tests or explicit verification cover the intended behavior for:
  - VSock available,
  - SSH UDS fallback,
  - compatibility TCP semantics,
  - absence of SSH TCP fallback.

### Task 11: Decide whether to finish sync/copy productization or freeze the
interim contract more explicitly

**Objective**

Make the normal same-user Lima sync/copy story fully honest and stable.

**Acceptance criteria**

- The repo does not overclaim that normal macOS same-user Lima sync/copy is
  fully productized if `workspace sync` is still not the frozen normal path.
- Either:
  - the current interim contract (`substrate world enable` /
    `scripts/mac/lima-warm.sh` staged copy) is documented consistently as the
    truthful live path, or
  - the remaining productization work lands and the docs are updated to match.
- `crates/shell/src/execution/workspace_cmd.rs`, `docs/USAGE.md`,
  `docs/WORLD.md`, and the macOS setup guide all agree on the normal path.

### Task 12: Normalize support taxonomy wording everywhere

**Objective**

Remove inconsistent taxonomy aliases and use one frozen label set across the
feature docs and live operator docs.

**Acceptance criteria**

- All relevant docs use the same exact labels:
  - `supported`
  - `degraded-but-supported`
  - `breakglass`
- Terms like `breakglass/unsupported` are either removed or explicitly defined
  as non-canonical and not used in the authoritative docs.
- The canonical wording source remains
  `spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md`, and other
  docs align to it.

### Task 13: Optional cleanup to single-source the canonical guest socket path

**Objective**

Reduce avoidable duplication around `/run/substrate.sock`.

**Acceptance criteria**

- If practical, shell doctor fallback code reads the canonical guest socket path
  from the shared authority instead of hardcoding it.
- If not practical, the remaining hardcoded usage is documented as intentional
  and low-risk.
- No docs claim fully single-sourced socket authority if the code still has
  intentional duplication.

### Task 14: Final closeout proof pass

**Objective**

After the above tasks land, rerun the review wall and confirm the feature can be
described as aligned without caveats that belong in active docs.

**Acceptance criteria**

- A final review pass confirms:
  - feature README is current,
  - phase READMEs are current,
  - `ROADMAP.md` authority drift is resolved,
  - top-level operator docs are taxonomy-aligned,
  - smoke help and verification surfaces are accurate,
  - unit authority is unambiguous,
  - transport docs are honest about realized behavior.
- The closeout proof includes at least:
  - `bash -n scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh scripts/mac/orchestration-smoke.sh`
  - targeted Rust tests for `world-mac-lima`, `world-service`, and the shell
    routing surfaces touched by the cleanup
  - a final grep/review sweep across `docs/WORLD.md`,
    `docs/reference/world/platforms/macos-lima-setup.md`,
    `docs/USAGE.md`, `scripts/mac/*`, and the feature docs for stale taxonomy
    and stale command references
- The remaining caveats, if any, are intentional and documented as such rather
  than being accidental drift.

## Worktree status note

At audit time, the worktree had local dirt in:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs`

These did **not** appear to be part of the same-user Lima hardening landing.

## Verification already run during audit

- `bash -n scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh scripts/mac/orchestration-smoke.sh`
- `cargo test -p world-mac-lima -- --nocapture`
- `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

All of the above passed during the audit run.
