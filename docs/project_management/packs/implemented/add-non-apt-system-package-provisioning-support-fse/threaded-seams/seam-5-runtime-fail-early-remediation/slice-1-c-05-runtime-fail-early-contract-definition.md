---
slice_id: S1
seam_id: SEAM-5
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - C-03 changes pacman-backed schema/view semantics used for runtime derivation
  - C-04 changes normalized requirement-set or manager-aware rendering assumptions
  - runtime wording or docs drift back toward mutation-at-runtime semantics
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-03
  - THR-04
  - THR-05
contracts_produced:
  - C-05
contracts_consumed:
  - C-01
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S1 - Define `C-05` runtime fail-early and remediation contract

#### Goal

Make `C-05` explicit enough that downstream validation and reconciliation can treat runtime system-package fail-early behavior as one deterministic read-only contract.

#### `C-05` - Runtime fail-early and remediation contract (authoritative pre-exec text)

##### 1) Runtime read-only invariant

- `substrate world deps current sync|install` MUST NOT invoke mutating `apt`, `apt-get`, `dpkg`, or `pacman` commands.
- Runtime system-package handling MAY use only read-only `dpkg-query` and `pacman -Q` probes.
- Missing probe executables count as unsatisfied runtime prerequisites, not as a reason to fall back to mutation.

##### 2) Runtime in-scope derivation

- Runtime derives one normalized APT requirement set and one normalized pacman requirement set from the runtime in-scope item set.
- `deps current sync` uses the effective enabled set.
- `deps current sync --all` widens the in-scope set as defined by the upstream world-deps contract.
- `deps current install <ITEM...>` scopes system-package fail-early only to the explicit expanded item set.

##### 3) Unsatisfied requirement posture

- If one or more derived system-package requirements are unsatisfied, runtime exits `4` before any non-system-package mutation runs.
- Runtime MAY contain both APT-backed and pacman-backed requirements in scope at the same time; it does not use the provisioning mixed-manager rejection rule.
- Manager-aware missing-requirement rendering must remain stable and deterministic across APT-backed and pacman-backed requirements.

##### 4) Remediation wording

- Remediation MUST include the exact command:
  - `substrate world enable --provision-deps`
- Remediation MUST state that runtime system-package mutation is not supported.
- Linux host-native and Windows guidance MUST remain fail-closed and must not imply host OS mutation.

##### 5) Dry-run and verbose behavior

- Dry-run performs runtime requirement derivation and read-only probe planning but performs no mutation.
- Verbose output may render normalized APT and pacman requirement sets, but must not widen into provisioning-time execution semantics.
- No-op behavior remains defined by all derived system-package requirements already being satisfied.

#### Verification checklist (contract gate input)

- Runtime read-only posture is explicit.
- Explicit-item runtime scope is explicit.
- Exit `4` posture for unsatisfied runtime requirements is explicit.
- Remediation wording is concrete, exact, and backend-aware.
- Dry-run / verbose behavior is concrete enough for implementation.
