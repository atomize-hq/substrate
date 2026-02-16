# Rollout plan
## Guiding principles
- Do not break existing macOS users.
- Provide clear fallback to Lima while VF matures.
- Collect actionable diagnostics when VF fails.
## Phases
### Phase A: VF-Linux (behind feature flag)
- Add backend selection:
  - `SUBSTRATE_WORLD_BACKEND=vf` (or config file setting)
- Ship VF backend with Linux guest only.
- Keep Lima as default.
- Validate parity:
  - file sharing
  - command execution
  - network attach/detach
- Gather telemetry and issue reports.
### Phase B: VF-Linux default on Apple Silicon
- Promote VF-Linux to default backend on Apple Silicon if:
  - crash rate < agreed threshold
  - boot success rate > agreed threshold
  - user-reported regressions resolved
- Lima remains available as fallback.
### Phase C: VF-macOS (opt-in)
- Add macOS guest world flavor:
  - `world_os=macos`
- Provide provisioning docs and troubleshooting.
- Limit to Apple Silicon.
### Phase D: VF-macOS production-ready + policy hardening
- Expand test coverage
- Improve egress controls beyond NIC attach/detach where feasible
- Consider privileged helper only if required
## Rollback plan
- Users can revert to Lima backend via config/env.
- If a release causes widespread failures:
  - hotfix: change default backend back to Lima
  - keep VF behind feature flag until fixed
