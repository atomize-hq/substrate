# Review Surfaces - Best-Effort Distro Package Manager

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.

## R1 - Selection Precedence Flow

```mermaid
flowchart TD
    Start([Install Substrate]) --> LinuxCheck{Linux host?}

    LinuxCheck -->|No| ExistingPath[Use existing platform behavior]
    LinuxCheck -->|Yes| ParseArgs[Parse CLI arguments]

    ParseArgs --> FlagCheck{--pkg-manager set?}
    FlagCheck -->|Yes| ValidateFlag{Valid?}
    ValidateFlag -->|No| Exit2[Exit 2 - Invalid flag]
    ValidateFlag -->|Yes| CheckPath1{In PATH?}
    CheckPath1 -->|No| Exit3[Exit 3 - Missing binary]
    CheckPath1 -->|Yes| SelectFlag[Select: flag]

    FlagCheck -->|No| EnvCheck{PKG_MANAGER set?}
    EnvCheck -->|Yes| ValidateEnv{Valid?}
    ValidateEnv -->|No| Exit2b[Exit 2 - Invalid env]
    ValidateEnv -->|Yes| CheckPath2{In PATH?}
    CheckPath2 -->|No| Exit3b[Exit 3 - Missing binary]
    CheckPath2 -->|Yes| SelectEnv[Select: env]

    EnvCheck -->|No| OsRelease[Read /etc/os-release]
    OsRelease --> MapFamily{Match distro family?}
    MapFamily -->|Yes| CheckFamilyPath{Binary in PATH?}
    CheckFamilyPath -->|Yes| SelectOsRelease[Select: os_release]
    CheckFamilyPath -->|No| ProbePath[Probe PATH]

    MapFamily -->|No| ProbePath
    ProbePath --> ProbeCheck{Any found?}
    ProbeCheck -->|No| Exit4[Exit 4 - None found]
    ProbeCheck -->|Yes| MultiCheck{Multiple found?}
    MultiCheck -->|Yes| EmitWarning[Emit multi-manager warning]
    EmitWarning --> SelectPath[Select first in order: path_probe]
    MultiCheck -->|No| SelectPath

    SelectFlag --> EmitLine[Emit decision line]
    SelectEnv --> EmitLine
    SelectOsRelease --> EmitLine
    SelectPath --> EmitLine

    EmitLine --> InstallPkgs[Install prerequisites]
    InstallPkgs --> Success[Exit 0]

    ExistingPath --> Success

    style Exit2 fill:#ffcccc
    style Exit2b fill:#ffcccc
    style Exit3 fill:#ffcccc
    style Exit3b fill:#ffcccc
    style Exit4 fill:#ffcccc
    style SelectFlag fill:#ccffcc
    style SelectEnv fill:#ccffcc
    style SelectOsRelease fill:#ccffcc
    style SelectPath fill:#ccffcc
```

## R2 - Package Manager Interface

```mermaid
flowchart LR
    subgraph CLI["CLI Surface"]
        InstallDirect["install-substrate.sh<br/>Direct entrypoint"]
        InstallWrapper["install.sh<br/>Wrapper entrypoint"]
        Flag["--pkg-manager &lt;manager&gt;"]
    end

    subgraph Env["Environment"]
        PkgManager["PKG_MANAGER=&lt;manager&gt;"]
        OsReleasePath["SUBSTRATE_INSTALL_OS_RELEASE_PATH"]
        PathEnv["PATH"]
    end

    subgraph Detection["Detection Engine"]
        Parser["os-release Parser<br/>Safe line-oriented"]
        Mapper["Distro Family Mapper<br/>Debian|Fedora/RHEL|Arch|SUSE"]
        Prober["PATH Prober<br/>Fixed order: apt-get→dnf→yum→pacman→zypper"]
    end

    subgraph Output["Output Contract"]
        Stderr["stderr: Decision line"]
        ExitCode["Exit code taxonomy"]
    end

    subgraph Docs["Operator Docs"]
        InstallDoc["INSTALLATION.md"]
        EnvContract["env/contract.md"]
        ThisContract["contract.md"]
    end

    InstallDirect --> Detection
    InstallWrapper -.->|Preserves exit codes| InstallDirect
    Flag --> Detection
    PkgManager --> Detection
    OsReleasePath -.->|Test hook| Parser
    PathEnv --> Prober

    Detection --> Stderr
    Detection --> ExitCode

    ThisContract -.->|Defines| Detection
    ThisContract -.->|Defines| Output
    InstallDoc -.->|Documents| CLI
    EnvContract -.->|Documents| Env
```

## R3 - Component Touch Surface

```mermaid
flowchart TB
    subgraph Installer["Installer Scripts"]
        InstallSubstrate["scripts/substrate/install-substrate.sh<br/>Core detection + selection"]
        InstallWrapper["scripts/substrate/install.sh<br/>Exit-code pass-through"]
    end

    subgraph Docs["Documentation"]
        InstallMd["docs/INSTALLATION.md<br/>Operator guide"]
        EnvContract["docs/reference/env/contract.md<br/>Env var contract"]
        Contract["contract.md<br/>Feature contract"]
    end

    subgraph Validation["Validation"]
        HermeticTest["tests/installers/pkg_manager_detection_smoke.sh<br/>Hermetic test suite"]
        SmokeScript["smoke/linux-smoke.sh<br/>Thin wrapper"]
        ManualPlaybook["manual_testing_playbook.md<br/>Operator validation"]
    end

    subgraph Pack["Planning Pack"]
        Slices["slices/BEDPM{0,1,2,3}/<br/>Slice specs"]
        Tasks["tasks.json<br/>Automation graph"]
        Plan["plan.md<br/>Execution order"]
    end

    InstallSubstrate --> Contract
    InstallWrapper --> Contract
    Contract --> InstallMd
    Contract --> EnvContract

    HermeticTest -->|Validates| InstallSubstrate
    SmokeScript -->|Calls| HermeticTest
    ManualPlaybook -->|References| SmokeScript

    Slices -->|Implement| InstallSubstrate
    Tasks -->|Orchestrate| Slices
    Plan -->|Sequences| Tasks
```

## R4 - Cross-Pack Contract Boundaries

```mermaid
flowchart LR
    subgraph ThisPack["best-effort-distro-package-manager<br/>(This Pack)"]
        Detect["Detect + Select<br/>SEAM-01, SEAM-02"]
        Report["Report + Fail<br/>SEAM-03"]
        Validate["Validate<br/>SEAM-04"]
    end

    subgraph Downstream["persist-detected-linux-distro-pkg-manager<br/>(Downstream Pack)"]
        Persist["Persist to<br/>install_state.json"]
    end

    subgraph OtherRelated["Related ADRs"]
        ADR30["ADR-0030<br/>Provisioning Otter<br/>(Guest world deps)"]
        ADR32["ADR-0032<br/>Stashing Ferret<br/>(Persistence)"]
        ADR35["ADR-0035<br/>Summoning Wombat<br/>(Install improvements)"]
    end

    Detect -->|pkg_manager.selected| Report
    Report -->|Validated contract| Validate
    Validate -->|C-01, C-02 contracted| Detect

    Detect -.->|Contracts C-01, C-02| Downstream
    Detect -.->|Detection boundary| ADR32
    Report -.->|Host vs guest boundary| ADR30
    Detect -.->|Shared file coordination| ADR35

    style ThisPack fill:#e6f3ff
    style Downstream fill:#f0f0f0
    style OtherRelated fill:#f0f0f0
```

## Review Surface Notes

**Active seam (SEAM-01) focus areas:**
- os-release parser safety (no shell execution)
- Mapping table accuracy (Debian, Fedora/RHEL, Arch, SUSE families)
- Decision-line format and placement
- `<unknown>` sentinel handling

**Next seam (SEAM-02) focus areas:**
- Precedence chain: flag → env → os_release → path_probe
- Exit-code taxonomy alignment with shared standards
- Multi-manager warning template
- Failure remediation content

**Future seams (SEAM-03, SEAM-04) will inherit:**
- Stable contracts from SEAM-01 and SEAM-02
- Defined threading state
- Validation requirements from hermetic test design

## Seam-Local Review Required

- `SEAM-01` requires seam-local `review.md` before decomposing into sub-slices
- `SEAM-02` requires seam-local `review.md` before decomposing into sub-slices
- Future seams will require review when promoted to `active` or `next` horizon
