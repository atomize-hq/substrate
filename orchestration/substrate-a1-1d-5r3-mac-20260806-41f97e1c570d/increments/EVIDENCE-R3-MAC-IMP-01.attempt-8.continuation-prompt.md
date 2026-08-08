META CONTINUATION BINDING AND START AUTHORITY — EVIDENCE ATTEMPT 8

Continue the existing evidence task with a fresh nonce. Bind exactly:

- task: `019fe162-c98c-7c82-94d0-155d6cab5122` / `local`
- worktree: `/Users/spensermcconnell/.codex/worktrees/1626/substrate`
- orchestration/evidence: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d` / `EVIDENCE:R3-MAC-IMP-01`
- attempt/allocation: `8` / `2`
- dispatch_nonce: `8c5f7e13d8f10d0e6744518a0b79bb4d575d4a6cb26483f2005fef8172c72fa2`
- source commit/tree: `1126b907df6e38043e9071da222f6c6a377b341c` / `3a29fa323a8b9099d894c11d98cc085ecf4807c5`
- target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- project: `local-b2016f8a311fef93149e42eaec4d704d` at `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- skill suite: `25 skills / 54 files / sha256:e3de93f358594033ed59696a9624d2d0ac2fd0c2aa7ddf4a532810a05b262282`
- contract: `increments/EVIDENCE-R3-MAC-IMP-01.attempt-8.contract.md`
- contract_sha256: `sha256:6dfe42fdb25e3a74e2932e1cfb62b3062e98177c2c0fa39518da1620f46e6823`
- retained amendment 0040 sha256: `sha256:cdaaa4ad590d4a110dd8ae5b03c36ac4833abd64e92e0e0ad055826d78495c14`
- invocation correction amendment 0041 sha256: `sha256:3eb4250e6995c6dd91614778b7d448dc74b680e199a47d2e754330cdd0eeed38`
- model/reasoning: `gpt-5.6-terra` / Extra High
- start_authorized: `true`

Attempt 7 is accepted only as a validated blocked artifact with exact restoration. Its blocker was an
invocation mistake, not a source or platform authority gap: the installer process inherited a PATH
that selected `/opt/homebrew/bin/limactl`. Do not repeat that.

Before mutation, reverify the full attempt-8 contract and current exact state. Meta has independently
verified the task and attached mirror are exact/clean, mirror `target/` is absent, prefix parity is
restored, remote is exact, fixed `/usr/local/bin/limactl` and sudo sentinel are unchanged, sudo works,
and the canonical Lima instance/control directory are absent.

For the canonical installer process itself, invoke from the exact attached mirror using this
literal start environment:

`/usr/bin/env -u CARGO_TARGET_DIR PATH=/usr/local/bin:/Users/spensermcconnell/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin HOME=/Users/spensermcconnell /opt/homebrew/bin/bash scripts/substrate/dev-install-substrate.sh ...`

Immediately before exec, under that exact environment, record Bash `type -P` resolving:

- `limactl` -> `/usr/local/bin/limactl`
- `cargo` -> `/Users/spensermcconnell/.cargo/bin/cargo`
- `rustc` -> `/Users/spensermcconnell/.cargo/bin/rustc`
- `bash` -> `/opt/homebrew/bin/bash`

The PATH must be applied from the installer's first instruction. A PATH used only for preflight or
later child commands is insufficient. Do not invoke the Homebrew Lima binary.

Amendment 0040's exact mirror `target/` exception remains in force and must return to absence. All
other source, MAC-only, direct-TTY, sentinel, restoration, no-user-interaction, and successor fences
remain unchanged. Restart the complete packet from a fresh pre-mutation baseline; do not reuse
attempt-7 evidence as success. Run through install, Stage-1/PM, code-sign/bootstrap, pairing,
lifecycle, retirement, restoration, artifact validation, and receipt validation. Send the single
structured receipt to meta as the final tool action. Do not run MAC-CLOSEOUT or archive anything.
