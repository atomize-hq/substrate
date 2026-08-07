# Allowlist/evidence discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-R2

## Review boundary

Fresh read-only allowlist/evidence review by a `gpt-5.6-terra` Extra High reviewer of the
ten-path R2 implementation subject bound to
`sha256:03bb9c919cd142e1b65079eddd188602410e1d36a3fce740df2979ec41d89b74`.
The reviewer did not mutate, test, install, stage, or publish.

## CLEAN lens result

- The exact nine tracked production/test paths plus the one untracked MAC compile-surface test
  match the R2 implementation allowlist; `git diff --check` is clean.
- Every introduced MAC admission uses a positive `target_os = "macos"` predicate or the existing
  positive `IS_MAC` installer branch. No introduced `not(target_os = "linux")` predicate remains.
- The 27 tracked Windows/WSL-named source paths are byte-identical to base under aggregate
  SHA-256 `3bf4f3b4dcc134a7febae7175497fffb9f9886094c7e851f76c8a5c79253280c`.
- The Linux-selected socket route retains `SOCK_SEQPACKET | SOCK_CLOEXEC`; only the MAC fcntl
  fallback is admitted. The earlier 20 Windows target failures are recorded out-of-corridor
  baseline debt under Amendment 0016 and are not an R2 remediation target.
- The MAC compile test covers the native and `x86_64-apple-darwin` ordinary shell/host surface
  and statically asserts the exact two-bin installer source shape. It deliberately preserves the
  mandatory deferred R4 lifecycle-macos compile gate rather than substituting native evidence.

No allowlist/evidence P1–P4 finding was reported.
