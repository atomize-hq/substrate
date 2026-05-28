# CLOSEOUT-30 Packet 4 Linux Manual Smoke (2026-05-28)

Status: completed. Packet 4 closed on 2026-05-28 after the Linux manual smoke was rerun with valid world-socket access and the full automated validation wall stayed green.

## Assumptions

1. Packet 4 closeout must rely on real Linux command evidence, not on a weakened validation contract.
2. Docs-only updates are still the right scope for this pass because the runtime behavior and tests were already in the intended Packet 4 shape.
3. Non-Linux fail-closed behavior remains pinned by the public control suite; this closeout note records the required Linux manual smoke.

## Baseline Linux Runtime Access

Commands run:

1. `target/debug/substrate world doctor --json`
2. `target/debug/substrate host doctor --json`
3. `whoami`
4. `id`
5. `getent group substrate`
6. `sudo target/debug/substrate world doctor --json`
7. `sg substrate -c 'id && target/debug/substrate world doctor --json'`

Observed outcomes:

1. The initial blocker was real but local to the current shell credentials, not to the world service itself:
   - `id` for the active shell did not include supplementary group `substrate`,
   - `/run/substrate.sock` was `root:substrate` with mode `0660`,
   - `getent group substrate` showed `azureuser` was enrolled in that group.
2. `sudo target/debug/substrate world doctor --json` succeeded and reported:
   - `host.world_socket.probe_ok = true`
   - `world.status = "ok"`
3. `sg substrate -c 'id && target/debug/substrate world doctor --json'` also succeeded, which confirmed the blocker was stale supplementary-group membership in the current session rather than runtime failure inside world-service.
4. The rerun commands below therefore used `sg substrate -c ...` so the Linux manual smoke exercised the real world-backed path from a correct authorization context.

## Host-Scoped Public Root Start Smoke

Fixture:

1. root: `/tmp/slice30-host-smoke-final-II5JT1`
2. host-scoped `codex` CLI backend
3. fake persistent binary at `/tmp/slice30-host-smoke-final-II5JT1/fake-codex.sh`
4. toolbox enabled over UDS

Commands run:

1. `sg substrate -c "HOME=/tmp/slice30-host-smoke-final-II5JT1/home SUBSTRATE_HOME=/tmp/slice30-host-smoke-final-II5JT1/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent start --backend cli:codex --scope host --prompt 'hello host smoke final' --json"`
2. `sg substrate -c "HOME=/tmp/slice30-host-smoke-final-II5JT1/home SUBSTRATE_HOME=/tmp/slice30-host-smoke-final-II5JT1/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent status --json"`
3. `sg substrate -c "HOME=/tmp/slice30-host-smoke-final-II5JT1/home SUBSTRATE_HOME=/tmp/slice30-host-smoke-final-II5JT1/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent toolbox status --json"`
4. `sg substrate -c "HOME=/tmp/slice30-host-smoke-final-II5JT1/home SUBSTRATE_HOME=/tmp/slice30-host-smoke-final-II5JT1/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent toolbox env --json"`
5. `sg substrate -c "HOME=/tmp/slice30-host-smoke-final-II5JT1/home SUBSTRATE_HOME=/tmp/slice30-host-smoke-final-II5JT1/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent doctor --json"`
6. `sg substrate -c "HOME=/tmp/slice30-host-smoke-final-II5JT1/home SUBSTRATE_HOME=/tmp/slice30-host-smoke-final-II5JT1/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent stop --session 019e6c35-4be2-7ac0-851d-3fdf29c175fd --json"`

Observed outcomes:

1. `agent start --scope host` emitted:
   - `accepted.scope = "host"`
   - `completed.turn_outcome = "success"`
   - `completed.session_posture = "active"`
2. The fake agent argv captured `exec`, and stdin captured `hello host smoke final`, so the host-scoped public happy path stayed on the normal host exec startup path.
3. `agent status --json` remained readable and showed the host orchestrator row as:
   - `backend_id = "cli:codex"`
   - `execution.scope = "host"`
   - `posture = "parked_resumable"`
4. `agent toolbox status --json` degraded readably to:
   - `eligibility.state = "dependency_unavailable"`
   - `reason = "no live host-scoped orchestrator participant found for the selected orchestrator"`
5. `agent toolbox env --json` failed closed with:
   - `no live host-scoped orchestrator participant found for the selected orchestrator`
6. `agent doctor --json` stayed healthy and reported all checks passing, including `world_boundary`.

## Explicit World-Backed Public Root Start Smoke

Fixture:

1. root: `/tmp/slice30-world-explicit-live-yM1IIM`
2. host-scoped `codex` orchestrator backend
3. world-scoped `claude_code` backend
4. real Linux world-service socket via the default `/run/substrate.sock`

Commands run:

1. `sg substrate -c "HOME=/tmp/slice30-world-explicit-live-yM1IIM/home SUBSTRATE_HOME=/tmp/slice30-world-explicit-live-yM1IIM/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent start --backend cli:claude_code --scope world --prompt 'hello explicit world smoke live' --json"`
2. `sg substrate -c "HOME=/tmp/slice30-world-explicit-live-yM1IIM/home SUBSTRATE_HOME=/tmp/slice30-world-explicit-live-yM1IIM/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent status --json"`
3. `sg substrate -c "HOME=/tmp/slice30-world-explicit-live-yM1IIM/home SUBSTRATE_HOME=/tmp/slice30-world-explicit-live-yM1IIM/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent toolbox status --json"`
4. `sg substrate -c "HOME=/tmp/slice30-world-explicit-live-yM1IIM/home SUBSTRATE_HOME=/tmp/slice30-world-explicit-live-yM1IIM/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent toolbox env --json"`
5. `sg substrate -c "HOME=/tmp/slice30-world-explicit-live-yM1IIM/home SUBSTRATE_HOME=/tmp/slice30-world-explicit-live-yM1IIM/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent doctor --json"`
6. `sg substrate -c "HOME=/tmp/slice30-world-explicit-live-yM1IIM/home SUBSTRATE_HOME=/tmp/slice30-world-explicit-live-yM1IIM/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent turn --session 019e6c2c-f13e-7751-82b0-077cc7e3d75c --backend cli:claude_code --prompt 'next world smoke live' --json"`
7. `sg substrate -c "HOME=/tmp/slice30-world-explicit-live-yM1IIM/home SUBSTRATE_HOME=/tmp/slice30-world-explicit-live-yM1IIM/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent stop --session 019e6c2c-f13e-7751-82b0-077cc7e3d75c --json"`

Observed outcomes:

1. `agent start --scope world` emitted:
   - `accepted.scope = "world"`
   - `accepted.backend_id = "cli:claude_code"`
   - `completed.turn_outcome = "success"`
   - `completed.session_posture = "active"`
2. The fake orchestrator argv still captured `exec`, and stdin captured `hello explicit world smoke live`, which confirms the inaugural prompt remained host-routed even though the requested public scope was world.
3. The authoritative session truth at `/tmp/slice30-world-explicit-live-yM1IIM/substrate-home/run/agent-hub/sessions/019e6c2c-f13e-7751-82b0-077cc7e3d75c/session.json` persisted:
   - `world_id = "wld_019e6c2c-f17b-7092-87dc-ca8edf12d8d5"`
   - `world_generation = 0`
   - `host_attach_contract.execution_scope = "host"`
   - `host_attach_contract.attach_launch_knobs.requested_execution_scope = "host"`
4. `agent status --json` stayed on the normal host lifecycle truth and showed:
   - orchestrator `backend_id = "cli:codex"`
   - `execution.scope = "host"`
   - `posture = "active_attached"`
   - `attached_participant_id` present
   - no `born_unattached` posture
5. `agent toolbox status --json` succeeded and reported:
   - `eligibility.state = "allowed"`
   - `active_orchestration_session_id = "019e6c2c-f13e-7751-82b0-077cc7e3d75c"`
   - `active_world_binding.world_id = "wld_019e6c2c-f17b-7092-87dc-ca8edf12d8d5"`
6. `agent toolbox env --json` succeeded and emitted:
   - `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`
   - `SUBSTRATE_AGENT_TOOLBOX_VERSION = "1"`
7. `agent doctor --json` stayed healthy and reported all checks passing, including `world_boundary`.
8. The later follow-up command failed closed with:
   - `backend_not_in_session: orchestration session 019e6c2c-f13e-7751-82b0-077cc7e3d75c has no exact backend slot for cli:claude_code`
   This preserves the host-mediated follow-up contract instead of silently bootstrapping a public world-first member path.

## Omitted-Scope World-Default Smoke

Fixture:

1. root: `/tmp/slice30-world-omitted-live-GlRd3e`
2. host-scoped `codex` orchestrator backend
3. unscoped `claude_code` backend
4. workspace override at `/tmp/slice30-world-omitted-live-GlRd3e/workspace/.substrate/workspace.yaml` setting `agents.defaults.execution.scope: world`

Commands run:

1. `sg substrate -c "cd /tmp/slice30-world-omitted-live-GlRd3e/workspace && HOME=/tmp/slice30-world-omitted-live-GlRd3e/home SUBSTRATE_HOME=/tmp/slice30-world-omitted-live-GlRd3e/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent start --backend cli:claude_code --prompt 'hello omitted world smoke live' --json"`
2. `sg substrate -c "cd /tmp/slice30-world-omitted-live-GlRd3e/workspace && HOME=/tmp/slice30-world-omitted-live-GlRd3e/home SUBSTRATE_HOME=/tmp/slice30-world-omitted-live-GlRd3e/substrate-home /home/azureuser/__Active_Code/atomize-hq/substrate/target/debug/substrate agent stop --session 019e6c2e-476c-7e10-930c-73f918190770 --json"`

Observed outcomes:

1. The omitted-scope start emitted:
   - `accepted.scope = "world"`
   - `accepted.backend_id = "cli:claude_code"`
   - `completed.turn_outcome = "success"`
2. The fake orchestrator argv again captured `exec`, and stdin captured `hello omitted world smoke live`.
3. The authoritative session truth at `/tmp/slice30-world-omitted-live-GlRd3e/substrate-home/run/agent-hub/sessions/019e6c2e-476c-7e10-930c-73f918190770/session.json` persisted:
   - `world_id = "wld_019e6c2e-4776-7420-93b6-0de1509c2267"`
   - `world_generation = 0`
   - `host_attach_contract.execution_scope = "host"`
4. This confirms omitted `--scope` honored the workspace-default preferred scope and still delivered the same host-first world-backed session truth.

## Automated Validation Wall

All required automated commands were rerun successfully on 2026-05-28:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
4. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
5. `cargo test --workspace -- --nocapture`

## Honest Packet 4 Closeout Status

Packet 4 is honestly closed on 2026-05-28.

Why this now clears the slice:

1. The Linux-first public world-backed happy path now has real command evidence for successful start, persisted host/world truth, readable `agent status`, allowed toolbox surfaces, healthy doctor output, omitted-scope world-default routing, and later host-mediated fail-closed follow-up behavior.
2. The host-scoped public happy path remains unchanged and still shows the readable-status versus fail-closed-control split when no live host-scoped orchestrator participant remains.
3. The public control suites still pin the non-Linux `unsupported_platform_or_posture` fail-closed behavior and the legacy `born_unattached` shape as specialized truth rather than the default public happy path.
