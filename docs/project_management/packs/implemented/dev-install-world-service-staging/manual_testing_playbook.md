# dev-install-world-service-staging — manual testing playbook

This playbook records the operator evidence required before execution closes the feature.
It is bound to the source pack at `docs/project_management/packs/draft/dev-install-world-service-staging/`; the extracted `-fse` seam docs remain planning inputs only.

## Shared setup
- Platform: Linux for behavior cases.
- Run the Linux runtime smoke script first:
  - `bash docs/project_management/packs/draft/dev-install-world-service-staging/smoke/linux-smoke.sh`
- Run installer smoke separately as dev-install staging / `C-04` regression evidence:
  - `tests/installers/install_smoke.sh --scenario dev`
  - `tests/installers/install_smoke.sh --scenario dev-no-world`
- Treat installer smoke as a staging regression surface, not as a second Linux behavior-delta proof surface.
- Use a clean temp home for every case:
  - `export SUBSTRATE_HOME="$(mktemp -d)/substrate-home"`
- Use the repo checkout root as the working directory.

## Case 1 — `--no-world` stages `world-service` for the selected profile
- Command:
  - `scripts/substrate/dev-install-substrate.sh --prefix "$SUBSTRATE_HOME" --profile debug --no-world`
- Verify:
  - `test -x target/bin/world-service`
  - `test -x target/bin/linux/world-service`
  - `readlink target/bin/world-service`
  - `readlink target/bin/linux/world-service`
  - `grep -n "enabled: false" "$SUBSTRATE_HOME/config.yaml"`
- Expected result:
  - both accepted staged paths resolve to the absolute `$(pwd)/target/debug/world-service`
  - config persists `world.enabled: false`
  - no provisioning or systemd mutation occurs during the dev-install run

## Case 2 — repeated dev installs refresh stale bridge targets
- Command sequence:
  - `scripts/substrate/dev-install-substrate.sh --prefix "$SUBSTRATE_HOME" --profile debug --no-world`
  - `scripts/substrate/dev-install-substrate.sh --prefix "$SUBSTRATE_HOME" --profile release --no-world`
- Verify:
  - `readlink target/bin/world-service`
  - `readlink target/bin/linux/world-service`
- Expected result:
  - both accepted staged paths resolve to the absolute `$(pwd)/target/release/world-service` after the second run

## Case 3 — missing-artifact dry-run fails before helper launch
- Setup:
  - `rm -f target/bin/world-service target/bin/linux/world-service`
- Command:
  - `set +e; "$SUBSTRATE_HOME/bin/substrate" world enable --home "$SUBSTRATE_HOME" --dry-run; status=$?; set -e`
- Verify:
  - `echo "$status"`
  - stderr names `target/bin/world-service`
  - stderr names `target/bin/linux/world-service`
  - stderr names `scripts/substrate/dev-install-substrate.sh --no-world`
  - stderr names `cargo build -p world-service`
- Expected result:
  - exit code is `3`
  - no helper log is written
  - `config.yaml` remains unchanged

## Case 4 — missing-artifact non-dry-run fails before provisioning
- Setup:
  - restore the same missing-artifact state as Case 3
- Command:
  - `set +e; "$SUBSTRATE_HOME/bin/substrate" world enable --home "$SUBSTRATE_HOME"; status=$?; set -e`
- Verify:
  - `echo "$status"`
  - no systemd unit or service mutation occurs
  - `grep -n "enabled: false" "$SUBSTRATE_HOME/config.yaml"`
- Expected result:
  - exit code is `3`
  - `world.enabled` stays `false`

## Case 5 — success path flips `world.enabled` only after verification
- Setup:
  - run `scripts/substrate/dev-install-substrate.sh --prefix "$SUBSTRATE_HOME" --profile release --no-world`
- Command:
  - `"$SUBSTRATE_HOME/bin/substrate" world enable --home "$SUBSTRATE_HOME" --dry-run`
  - `"$SUBSTRATE_HOME/bin/substrate" world enable --home "$SUBSTRATE_HOME"`
- Verify:
  - dry-run exits `0`
  - non-dry-run completes helper execution and health verification
  - `grep -n "enabled: true" "$SUBSTRATE_HOME/config.yaml"`
- Expected result:
  - dry-run performs no writes
  - non-dry-run writes `world.enabled: true` only after successful verification

## Evidence capture
- Paste the command lines, exit codes, and the two `readlink` results into the `DIWAS1-integ-core` and `CP1-ci-checkpoint` session-log entries.
- Capture the missing-artifact stderr block once and reuse it in the feature closeout notes.
- Record installer smoke output separately under the `C-04` regression evidence note; do not fold it into the Linux behavior-delta proof surface.
