# R3 MAC allowlist and evidence discovery review

Discovery subject fingerprint:
`sha256:1097874180b86a3331c7d14c06d2b803cfd545273576e691b8eb1f1ec5bb4bca`

The frozen non-review subject paths are exactly:

1. `src/bin/substrate-lifecycle-macos.rs`
2. `crates/shell/src/execution/managed_lifecycle/macos_client.rs`
3. `scripts/mac/com.substrate.lifecycle.publisher.v1.plist`
4. `scripts/mac/lima-lifecycle.sh`
5. `scripts/mac/lima-stop.sh`
6. `scripts/mac/lima-warm.sh`
7. `scripts/mac/lima/units/substrate-world-service.socket`
8. `crates/world-mac-lima/src/forwarding.rs`
9. `crates/world-mac-lima/src/lib.rs`
10. `tests/mac/installer_parity_fixture.sh`
11. `tests/mac/lifecycle_r3.sh`
12. `llm-last-mile/runtime-refactor/00-README.md`
13. `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
14. `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
15. `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
16. `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`

The exact post-subject review set is this document, `r3-mac-review-authority-security.md`,
`r3-mac-review-lifecycle-convergence.md`, and `r3-mac-review-cycle-record.json`; it is excluded
from the subject fingerprint. No evidence JSON/receipt is created in this implementation packet.

Pre-edit GitNexus impacts were completed with tests included. `ForwardingHandle` has three direct
callers and is manually treated as HIGH because Drop kills/waits and removes a pathname. The
forwarding constructor, typed backend methods, legacy gate, MAC client functions, and affected
embedded tests were all LOW graph risk; no graph process was reported. GitNexus FTS is degraded,
so Bash/config callers were manually source-closed. New symbols are recorded as `N/A (new)` with
impact inherited from their exact replaced callers.

Protocol discovery verdict: `FINDINGS`.

## Closure-1 delta review and remediation

Fresh closure subject:
`sha256:f4db3ee786ba2c0e8ace7bda5a1064705b90afa488ce1d8eac089126c786dc65`.

The closure review found no path-fence escape and no P3/P4 result. Remediation stayed inside the
previous frozen production/test/review paths. The new fake SSH and copied-executor checks are
non-native fixtures: they do not invoke Lima, launchd, code signing, host Keychain, publisher
installation, or an executable built artifact. The copied mock is used only after the production
wrapper's fixed literal has been substituted inside a disposable temporary file.

Protocol closure-1 verdict: `FINDINGS`; no evidence byte has been created or committed.

## Supplemental-causal-1 review

Fresh subject:
`sha256:1cabe3430ef0a28a88ef560cca26cfc4d934c1107ca2512d66f22c126583fa31`.

The reviewer confirmed the changed-file fence and absence of native evidence work. The supplemental
findings are P1/P2 only and are direct consequences of prior MAC remediation. The second and final
supplemental causal pass may repair only the existing MAC client/executor, warm script, forwarding,
and permitted fixture/embedded-test surfaces; it must not add evidence bytes or alter the frozen
run-only fixtures.


## Supplemental-causal-2 final review — allowlist/evidence disposition

The fresh final review subject was
`sha256:e60963026aa39be67e334119ff7ae61f3f15f19dbfd31539d790f8735b493924`.
All 16 frozen non-review manifest blobs matched the review subject; changed and untracked paths
remain inside the MAC production/test/documentation fence plus the four exact review-control
paths. No evidence artifact was created, and native Lima, launchd, Keychain, code-signing, or XPC
execution did not occur.

The reviewer returned only the P1/P2 findings recorded in the companion authority and lifecycle
lenses. It admitted no P3/P4 inventory item. `git diff --check`, Bash syntax checks, rustfmt
check, `tests/mac/lifecycle_r3.sh`, and `cargo test -p world-mac-lima` (52 tests) passed. The
unchanged baseline installer fixture still fails at `install-substrate.sh:98` (`exec:
{context_fd}: not found`), and the lifecycle binary check remains blocked before target
compilation by 11 unchanged shell errors.

No further code change is authorized: this is the second supplemental causal result and the
validated review protocol requires `bounded_stop` / `budget_exhausted`.
