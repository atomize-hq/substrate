# Review Surfaces - Best-Effort Distro Package Manager

These diagrams orient the pack. They show the actual product and validation shape expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.

## R1 - Hosted Installer Decision Pipeline

```mermaid
flowchart LR
  CLI["CLI args"] --> EX["Explicit selector stage"]
  ENV["PKG_MANAGER"] --> EX
  ORS["Selected os-release input"] --> PARSE["Safe parser + normalized distro fields"]
  PARSE --> MAP["Family mapping stage"]
  EX --> DECIDE["Final manager decision"]
  MAP --> DECIDE
  PATH["Fixed PATH probe"] --> FALLBACK["Fallback + ambiguity handling"]
  FALLBACK --> DECIDE
  DECIDE --> REPORT["Warning line + stable decision line"]
  REPORT --> INSTALL["Prerequisite installation"]
```

## R2 - Operator-Facing Surface That Lands

```mermaid
flowchart TB
  Direct["scripts/substrate/install-substrate.sh"] --> Out1["stderr decision line"]
  Direct --> Out2["exit 0 / 2 / 3 / 4"]
  Wrapper["scripts/substrate/install.sh"] --> Direct
  Wrapper --> Out3["wrapper preserves feature exits"]
  Contract["contract.md"] -.-> Direct
  Contract["contract.md"] -.-> Wrapper
  Docs1["docs/INSTALLATION.md"] -.-> Out1
  Docs1["docs/INSTALLATION.md"] -.-> Out3
  Docs2["docs/reference/env/contract.md"] -.-> ENV["PKG_MANAGER / SUBSTRATE_INSTALL_OS_RELEASE_PATH"]
```

## R3 - Validation And Evidence Topology

```mermaid
flowchart LR
  Harness["tests/installers/pkg_manager_detection_smoke.sh"] --> Installer["direct installer path"]
  Harness --> Wrapper["wrapper path"]
  Smoke["smoke/linux-smoke.sh"] --> Harness
  MacSmoke["scripts/mac/smoke.sh or equivalent macOS host run"] --> Lima["Lima-backed Linux installer path"]
  Lima --> Harness
  Playbook["manual_testing_playbook.md"] --> Smoke
  Playbook --> Harness
  Harness --> Evidence["contract evidence"]
  Smoke --> Evidence
  MacSmoke --> Evidence
  Playbook --> Evidence
```

## R4 - Checkpoint And Downstream Handoff

```mermaid
flowchart LR
  Evidence["SEAM-06 validation evidence"] --> CP1["CP1 checkpoint"]
  CP1 --> Parity["compile parity: linux / macos / windows"]
  CP1 --> Quick["CI testing quick: linux / macos / windows"]
  CP1 --> Behavior["Linux behavior smoke"]
  CP1 --> MacBehavior["macOS-hosted Lima-backed behavior evidence"]
  CP1 --> Closeout["SEAM-07 closeout + handoff record"]
  Closeout --> Downstream["persist-detected-linux-distro-pkg-manager"]
```

## Review Surface Notes

- These diagrams are pack-level orientation only.
- `SEAM-01` and `SEAM-02` still require seam-local `review.md` before they become `exec-ready`.
- Future seams will require seam-local review when promoted.

Active seam focus:

- selected-input contract and parser safety
- alternate-input hook and `<unknown>` degradation
- downstream inheritance boundary for parser/input truth
- later macOS-hosted Lima-backed runs must consume the same parser/input truth without drift

Next seam focus:

- family-table coverage and availability rules
- stable decision-line wording, timing, and suppression
- clean handoff into explicit-selector and fallback seams
- preserved semantics when the hosted install is exercised from macOS through the Lima backend
