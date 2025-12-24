# WDL2-integ kickoff — System packages provisioning integration

Merge WDL2 code + tests, reconcile to `S2`, then run:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Relevant `cargo test ...`
- `make preflight`

Then run platform smoke checks as applicable and capture outputs:
- macOS: `scripts/mac/smoke.sh`
- Windows: `pwsh -File scripts/windows/wsl-smoke.ps1`

