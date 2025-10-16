# Transport Parity Architecture Sketch

Status: Draft (Phase W design)
Last Updated: 2025-09-23T00:00:00Z
Owner: Substrate Core
Related Spike: docs/SPIKE_TRANSPORT_PARITY_PLAN.md

## Overview

This document captures the transport architecture needed for cross-platform parity.

- Host binaries (`substrate-shell`, `host-proxy`, tooling)
  communicate only through the `agent-api-client` transport abstraction.
- The Windows forwarder mediates between host Named Pipes and the world
  agent running in WSL.
- `world-agent` exposes both its canonical Unix domain socket and a
  gated loopback TCP listener.
- Telemetry records the active transport in every span to aid
  troubleshooting and parity verification.
- macOS Lima uses the Unix connector via the shared socket today.
  Provisioning will switch to vsock once the Lima package bundles the required tooling.

## Component Relationships (ASCII Diagram)

```text
+-------------------+            +-----------------------+
| Host CLI (all OS) |--HTTP----->| agent-api-client      |
| substrate, proxy  |            | - Connector trait     |
+-------------------+            | - Endpoint enum       |
                                  +-----------+-----------+
                                              |
       +--------------------+-----------------+--------------------+
       |                    |                                      |
  (Linux / macOS hosts)    (Windows host)                  (Future)
  UDS Connector              Named Pipe Connector             Vsock Connector
  hyperlocal client          named pipe + HTTP bridge         (planned vsock connector)
       |                    v
       |        +----------------------------+
       |        | Windows Forwarder          |
       |        | - Named Pipe listener      |
       |        | - Bridge to TCP/UDS        |
       |        +---------+------------------+
       |                  |
       v                  v
+-------------+    +-------------------+
| world-agent |    | world-agent       |
| Unix socket |    | Loopback TCP      |
| /run/...    |    | 127.0.0.1:<port> |
+-------------+    +-------------------+
        \____________________  _____________________/
                             \/
                     Telemetry (transport.mode)
```

## Connector State Transitions

1. **Initialization**
   - Determine platform plus operator configuration (CLI flags, env vars,
     config files).
   - Map to `Endpoint::Unix`, `Endpoint::NamedPipe`, or `Endpoint::Tcp`.
   - Construct the connector and register telemetry metadata.
2. **Request Lifecycle**
   - Build HTTP request with shared headers.
   - Dispatch via connector; on success record transport metadata on the
     span.
   - On error, retry (per policy) or propagate with transport context for
     diagnostics.
3. **Shutdown**
   - Dispose of resources (close pipe handles, drop sockets).
   - Forwarder stops gracefully, closing Named Pipe to avoid stale handles.

## Forwarder Target Selection

- Configuration file: `%LOCALAPPDATA%/Substrate/forwarder.toml`.

  ```toml
  [target]
  mode = "tcp"           # other option: "uds"
  tcp_port = 61337
  uds_path = "\\\\wsl$\\substrate-wsl\run\substrate.sock"
  ```

- Environment override: `SUBSTRATE_FORWARDER_TARGET=tcp|uds` for testing.
- Logging: startup emits `forwarder.target=<mode>` in JSON logs.

## World Agent Dual Listener

- Unix socket remains `/run/substrate.sock` with permissions 0666 inside
  the world.
- Loopback TCP listener turns on when `SUBSTRATE_AGENT_TCP_PORT` is
  present.
- Security guardrail: bind only to `127.0.0.1` and validate the port
  before starting.
- Systemd snippet:

  ```ini
  [Service]
  Environment="SUBSTRATE_AGENT_TCP_PORT=61337"
  ```

## Telemetry Integration

- `transport.mode` holds `named_pipe`, `unix`, or `tcp`.
- Optional `transport.endpoint` provides sanitized path, pipe, or port
  information.
- Smoke suites assert the expected mode on each platform.

## Open Items

- Vsock connector design (post-spike follow-up).
- CI integration to run connector integration tests on each platform.

## References

- docs/SPIKE_TRANSPORT_PARITY_PLAN.md
- docs/project_management/logs/windows_always_world.md
- docs/dev/wsl_world_setup.md (update pending)
- docs/dev/windows_host_transport_plan.md (Windows host integration addendum)
