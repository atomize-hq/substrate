# DIWAS1-spec — Linux dev-install world-service staging for enable-later

## Behavior delta (single)
- Existing: `scripts/substrate/dev-install-substrate.sh --no-world` keeps the world disabled and can leave the accepted staged path set without a `world-service` bridge.
- New: on Linux, `scripts/substrate/dev-install-substrate.sh --no-world` still stages `world-service` into both accepted staged paths while keeping the world disabled.
- Why: the enable-later workflow becomes repeatable without a manual artifact step.

## Scope
- Linux dev-install staging and refresh behavior
- selected-profile mapping for staged `world-service`
- shared accepted path rule for runtime and provisioning
- Linux smoke and installer regression evidence

## Behavior (authoritative)
### Linux dev-install staging
- On Linux, `scripts/substrate/dev-install-substrate.sh --no-world --profile <debug|release>` stages `target/bin/world-service` and `target/bin/linux/world-service` from `target/<profile>/world-service`.
- The selected dev-install profile controls the staged bridge target.
- `substrate world enable --profile` does not change the staged bridge target.

### World-disabled metadata and provisioning posture
- `scripts/substrate/dev-install-substrate.sh --no-world` keeps `world.enabled: false`.
- The script skips Linux provisioning and systemd mutation when `--no-world` is set.

### Refresh rule and shared search order
- Re-running `scripts/substrate/dev-install-substrate.sh --no-world` refreshes both staged links with `ln -sfn`.
- The accepted runtime and provisioning search order remains `bin/world-service`, then `bin/linux/world-service`.
- `scripts/substrate/install-substrate.sh --no-world` keeps its current production-install posture aside from reading the same accepted path rule.

## Acceptance criteria
- AC-DIWAS1-01: On Linux, `scripts/substrate/dev-install-substrate.sh --no-world --profile debug` stages `target/bin/world-service` and `target/bin/linux/world-service` as executable links to `target/debug/world-service`.
- AC-DIWAS1-02: On Linux, `scripts/substrate/dev-install-substrate.sh --no-world --profile release` stages `target/bin/world-service` and `target/bin/linux/world-service` as executable links to `target/release/world-service`.
- AC-DIWAS1-03: `scripts/substrate/dev-install-substrate.sh --no-world` keeps `world.enabled: false` and skips Linux provisioning and systemd mutation.
- AC-DIWAS1-04: Re-running `scripts/substrate/dev-install-substrate.sh --no-world` refreshes both staged world-service links with `ln -sfn`, so stale targets from an earlier profile or build are replaced.
- AC-DIWAS1-05: The accepted runtime and provisioning search order remains `bin/world-service`, then `bin/linux/world-service`, and `scripts/substrate/install-substrate.sh --no-world` keeps its current production-install posture aside from that shared accepted path rule.

## Out of scope
- building `world-service` inside `substrate world enable`
- macOS Lima workflow changes
- Windows support changes
