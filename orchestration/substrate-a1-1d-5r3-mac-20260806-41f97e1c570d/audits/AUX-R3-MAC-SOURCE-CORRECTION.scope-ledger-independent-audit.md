# Independent scope-ledger audit

- Orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- Audit agent: `/root/audit_source_scope_ledger_v2`
- Model/reasoning: `gpt-5.6-terra` / Extra High
- Mode: fresh read-only subagent audit
- Product worktree: `/Users/spensermcconnell/.codex/worktrees/60cc/substrate`
- Base/tree: `d8a65fc8890dd37584aaeac2984c906188e5f06e` / `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`
- Verdict: `LEDGER_CONFIRMED_WITH_CORRECTIONS`

## Findings

### P1 — exclusive single-stdio design is encoded

- `crates/common/src/managed_artifact.rs:841-905,2320-2487` adds
  `GuestPublisherPairingGuestChannelV1`, the four-frame `lima-stdio-v1` protocol, and validation
  rejecting SSH/socket/alternate transport.
- `src/bin/substrate-lifecycle-control.rs:106-127,628-638` fails closed for the one-stream/TTY
  conflict but also treats `ssh` as an impermissible alternate. Amendment 0008 does not authorize
  that inference: a distinct logical session over the same PM-bound SSH transport is not
  automatically prohibited.
- `src/bin/substrate-lifecycle-linux.rs:362-575` implements the unapproved one-stream guest
  protocol even though the supported control route currently stops before ticket issuance.

### P1 — independent guest TTY is not realizable by the implementation

`src/bin/substrate-lifecycle-control.rs:91-127` validates and then returns
`BLOCKED_SCOPE_EXPANSION`. One `limactl shell` duplex stream cannot safely carry retained protocol
data plus independent guest `/dev/tty` confirmation. The reported transport/TTY P1 is confirmed.

### P1 — mapped lifecycle dispatch is incompatible with the executor

- `scripts/mac/lima-lifecycle.sh:205-237,260-275` sends generic lifecycle actions to
  `...publisher.v1 lima-action`.
- `src/bin/substrate-lifecycle-macos.rs:96-155` accepts only `run-publisher`,
  `bootstrap-publisher`, and fixed XPC relays. `lima-action` is rejected; the otherwise present
  `execute_mac_managed_action_v1` at `:705-727` is not wired to `main`.

The reported mapped-lifecycle P1 is confirmed.

### P2 — ordinary Linux behavior claim requires correction

The Linux host provider still returns unavailable for issue/advance/consume and its Linux
`SOCK_CLOEXEC` behavior remains. However, `src/bin/substrate-lifecycle-linux.rs:362-369` now accepts
the public `guest-pairing-stdio` command instead of returning usage error. It is intended for the
Lima guest executor but is not compile- or runtime-gated to Lima.

### P2 — static macOS fixture masks the mapped-lifecycle P1

`tests/mac/lifecycle_r3.sh:351-364` substitutes a mock executor that requires `lima-action` and
emits synthetic attestation. It proves script-to-mock behavior rather than compatibility with the
real hardened executor.

### P2 — two ledger entries are mixed at hunk granularity

- `tests/installers/dev_install_bash32_fd_regression.sh:93-95,163-180` covers both blocker 1 and
  amendment-0007 Mac control/executor build-and-copy behavior.
- `tests/mac/dev_install_compile_surface_r3.sh:21-45` covers blocker 2 plus amendment-0007 control
  and Mac lifecycle surfaces.

## Reverification

- HEAD/tree exactly match the bound base/tree.
- Zero staged paths; 26 tracked paths; two untracked paths; 28 status entries; `+8821/-681`.
- Tracked diff SHA-256:
  `39682fd4415a00ae4099580135a687d3e11858f62f84e9ef6ff3771bafd4b54c`.
- Both untracked hashes match amendment 0008.
- `git diff --check` is clean.
- Live target ref equals the base commit/tree.
- No tests, install, Lima, Keychain, XPC, evidence, or product mutation ran during the audit.

## Confirmed classifications

- **Blocker 1:** installer FD lifecycle plus Bash-3.2 portions of its regression.
- **Blocker 2:** macOS cfg/compile admissions, non-Linux FD fallback, and Linux-only test guards.
- **Blocker 4:** expected-project-ID validator, tests, and exactly three runtime-refactor documents.
  Those documents do not widen pairing, transport, endpoint, or successor authority.
- **Amendment 0007:** shared pairing contracts, lifecycle client APIs, macOS Keychain/XPC executor,
  Stage-1 state, control binary, Lima guest executor, and pairing fixtures.
- **Windows:** compile-only shared adaptation; all pairing operations still return unavailable.

## Verdict

`LEDGER_CONFIRMED_WITH_CORRECTIONS`

The snapshot, Windows classification, non-pairing blockers, document scope, and both reported P1s
are confirmed. The ledger must correct the encoded blanket SSH/channel prohibition, ordinary-Linux
claim, and mixed blocker/0007 test hunks.

## Recommended boundary

Retain blockers 1, 2, and 4. They do not depend on the amendment-0007 pairing implementation.

At minimum, remove the unapproved `lima-stdio-v1`/exclusive-channel contracts, Linux command,
control gates/tests, and mocked fixture assertions. The conservative reduction is to remove all
amendment-0007 pairing code and dependencies as exact hunks while preserving blockers 1, 2, and 4.
No removal was performed by this audit.

## Remaining authority decisions

1. Define whether and how distinct logical data and operator-TTY sessions may coexist on the same
   PM-bound SSH transport, including identity binding, lifetime, replay, retry, and confirmation.
2. Authorize a trusted replacement for generic `lima-action` mapped-lifecycle dispatch and its
   ordering into the hardened executor.
3. Decide whether any Keychain/XPC/Stage-1 preparation should remain after the protocol decision.

No present code or test authorizes publication, installation, evidence, or successor dispatch.
