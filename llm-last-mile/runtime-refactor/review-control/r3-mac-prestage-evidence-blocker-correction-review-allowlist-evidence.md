# R3 MAC prestage evidence-blocker correction — allowlist/evidence review

Final non-review subject: `sha256:414537800707e8c90904ea38cb375540f74fa1a1f7aa08b6ae96ed84eb7638f9`

## Containment

The non-review subject has 13 changed paths, all inside the 0046 fence:

1. `crates/common/src/lib.rs`
2. `crates/common/src/managed_artifact.rs`
3. `crates/shell/src/execution/managed_lifecycle.rs`
4. `crates/shell/tests/managed_lifecycle_v1.rs`
5. `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
6. `scripts/mac/lima-lifecycle.sh` (mode remains `100755`)
7. `scripts/mac/lima-warm.sh`
8. `scripts/substrate/dev-install-substrate.sh`
9. `src/bin/substrate-lifecycle-macos.rs`
10. `tests/installers/dev_install_bash32_fd_regression.sh`
11. `tests/mac/dev_install_compile_surface_r3.sh`
12. `tests/mac/lifecycle_r3.sh`
13. `tests/mac/prestage_artifact_route_r3.sh`

Manual change detection was used because GitNexus has no index for this exact task worktree.
Windows/WSL, Linux host paths, Cargo manifests, Cargo lockfile, and the Linux lifecycle binary are
byte-identical to the base. Cargo metadata exposes exactly the four required existing binary names
and no `world` binary target. Added-byte credential-shaped scanning and `git diff --check` pass.

## Proof

Focused static/fixture checks pass:
`prestage_artifact_route_r3.sh`, `lifecycle_r3.sh`, `dev_install_compile_surface_r3.sh`, and the
Bash-3.2 installer FD/retained-bundle fixture. The final external proof directory is
`/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/AUX-R3-MAC-PRESTAGE-EVIDENCE-BLOCKER-CORRECTION/fixedseq-final-aarch64-proof.xfnruy`.
It ran locked/offline `aarch64-unknown-linux-gnu` builds for exactly `substrate-lifecycle-linux`,
`world-service`, `substrate-gateway`, and `substrate`, all identified as Linux ELF64 AArch64.

Final ELF SHA-256:

- `substrate-lifecycle-linux`: `e74736109dd58a9ad1bf36cfc47f90588d8f66859ceb433a8480888e22527bc0`
- `world-service`: `5546cd720329858f37ee92dc910a00cf3a2f243803f6a065907ad771edefd14d`
- `substrate-gateway`: `39beca555391d82a6654e018dd77130f79b553ba3d783067289d34534e5d2625`
- `substrate`: `78f7bb409f9ee42ea9b6fbb72333f9490b762cc0c97ec97c2177831622a49d78`
