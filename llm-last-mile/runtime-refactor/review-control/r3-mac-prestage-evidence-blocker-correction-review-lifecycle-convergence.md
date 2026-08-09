# R3 MAC prestage evidence-blocker correction — lifecycle/convergence review

Final subject: `sha256:414537800707e8c90904ea38cb375540f74fa1a1f7aa08b6ae96ed84eb7638f9`

## Fixed installation convergence

`mac_execute_fixed_install_sequence_v1` derives its sequence solely from the signed Stage-1
successor manifest and literal `mac_fixed_install_steps_v1`. The fixed sequence begins after the
Stage-1 instance receipt and retains the four existing AArch64 guest artifacts. The plan is
forward-only (`Create`, then socket `Enable`/`Start`) and is not a catalogue traversal.

Each durable prefix validates the canonical manifest, receipt index, head, signature, planned
receipt identity, role/action, and exact principal/attempt. For the only permitted crash window
between `resume_action_receipt_commit_v1` and protected-state CAS, recovery permits exactly one
receipt ahead only when the canonical index/head chain to the retained current anchor and the tail
matches the current signed prepared record and next literal plan step. The existing idempotent
post-PM executor replays/completes that exact prepared transaction; the outer loop then reopens
and validates the advanced anchor. Any absent, extra, substituted, stale, or ambiguous state stops.

The normal and Stage-1 post-CAS-resume paths refresh the ordinary stop catalogue only after private
fixed-install convergence. This preserves the existing `lima-stop -> post_pm_action` path without
letting installer/warm execute it.

## Deterministic checks

- The recovery static proof was red before implementation and green after implementation in
  `tests/mac/lifecycle_r3.sh` and `tests/mac/prestage_artifact_route_r3.sh`.
- `cargo test --locked --offline -p substrate --bin substrate-lifecycle-macos`: 10 passed.
- `cargo check --locked --offline -p substrate --bin substrate-lifecycle-macos`: passed.
- `cargo test --locked --offline -p shell --test managed_lifecycle_v1`: 6 passed.
- `cargo test --locked --offline -p substrate-common --lib`: 86 passed.
- No installer, Lima, Keychain, launchd, or evidence action was executed.
