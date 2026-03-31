# DIWAS1-spec — Linux dev-install world-agent staging for enable-later

## Behavior delta (single)
- Existing: `scripts/substrate/dev-install-substrate.sh --no-world` keeps the world disabled and can leave the accepted staged path set without a `world-agent` bridge.
- New: on Linux, `scripts/substrate/dev-install-substrate.sh --no-world` still stages `world-agent` into both accepted staged paths while keeping the world disabled.
- Why: the enable-later workflow becomes repeatable without a manual artifact step.

## Scope
- Linux dev-install staging and refresh behavior
- selected-profile mapping for staged `world-agent`
- shared accepted path rule for runtime and provisioning
- Linux smoke and installer regression evidence

## Behavior (authoritative)
### Linux dev-install staging
- On Linux, `scripts/substrate/dev-install-substrate.sh --no-world --profile <debug|release>` stages `target/bin/world-agent` and `target/bin/linux/world-agent` from `target/<profile>/world-agent`.
- The selected dev-install profile controls the staged bridge target.
- `substrate world enable --profile` does not change the staged bridge target.

### World-disabled metadata and provisioning posture
- `scripts/substrate/dev-install-substrate.sh --no-world` keeps `world.enabled: false`.
- The script skips Linux provisioning and systemd mutation when `--no-world` is set.

### Refresh rule and shared search order
- Re-running `scripts/substrate/dev-install-substrate.sh --no-world` refreshes both staged links with `ln -sfn`.
- The accepted runtime and provisioning search order remains `bin/world-agent`, then `bin/linux/world-agent`.
- `scripts/substrate/install-substrate.sh --no-world` keeps its current production-install posture aside from reading the same accepted path rule.

## Acceptance criteria
- AC-DIWAS1-01: On Linux, `scripts/substrate/dev-install-substrate.sh --no-world --profile debug` stages `target/bin/world-agent` and `target/bin/linux/world-agent` as executable links to `target/debug/world-agent`.
- AC-DIWAS1-02: On Linux, `scripts/substrate/dev-install-substrate.sh --no-world --profile release` stages `target/bin/world-agent` and `target/bin/linux/world-agent` as executable links to `target/release/world-agent`.
- AC-DIWAS1-03: `scripts/substrate/dev-install-substrate.sh --no-world` keeps `world.enabled: false` and skips Linux provisioning and systemd mutation.
- AC-DIWAS1-04: Re-running `scripts/substrate/dev-install-substrate.sh --no-world` refreshes both staged world-agent links with `ln -sfn`, so stale targets from an earlier profile or build are replaced.
- AC-DIWAS1-05: The accepted runtime and provisioning search order remains `bin/world-agent`, then `bin/linux/world-agent`, and `scripts/substrate/install-substrate.sh --no-world` keeps its current production-install posture aside from that shared accepted path rule.

## Out of scope
- building `world-agent` inside `substrate world enable`
- macOS Lima workflow changes
- Windows support changes
