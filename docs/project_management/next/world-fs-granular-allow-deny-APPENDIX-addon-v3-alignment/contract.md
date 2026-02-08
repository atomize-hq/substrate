# world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment — contract surface (add-on)

This add-on pack does **not** redefine Appendix A+B. The authoritative contract remains:
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ENV.md`

This file consolidates the **operator-facing deltas** required to bring the implementation into compliance.

## 1) Discovered discrepancy (must be fixed)
`substrate policy show` currently serializes an effective policy whose `world_fs` block is still V2-shaped (via `crates/broker/src/policy.rs` V2 serializer), contradicting Appendix A.6.

## 2) CLI contract: `substrate policy show` (Appendix A.6; normative)
Authoritative requirement: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` §1.3.

Required behavior:
- Effective policy output uses V3 operator-facing keys (no V2 keys rendered as operator-facing fields).
- When `world_fs.host_visible=false`, output MUST include:
  - `world_fs.discover`, `world_fs.read`, `world_fs.write` (explicitly),
  - each with `allow_list` and `deny_list`,
  - and empty deny lists rendered explicitly (`deny_list: []` in YAML; `"deny_list":[]` in JSON).

## 3) No backwards compatibility (policy keys + snapshots)
Authoritative requirement:
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` §5
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` §1.2 + §2.1
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md` (version lockstep)

Concrete enforcement:
- Legacy policy keys MUST be rejected as invalid config (exit `2`) on host.
- World-agent MUST reject any snapshot `schema_version != 3` as a protocol error (HTTP 400 / fatal WS error frame).
- Shell MUST emit `PolicySnapshotV3` (`schema_version=3`) on world-agent requests.

## 4) Exit codes (host)
Taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`.

Preservation requirement:
- Actionable config/policy errors (invalid keys, invalid combinations, invalid patterns) remain exit `2` (Appendix contract §2.1).

## 5) Platform guarantees
- Linux/macOS/Windows: policy display and policy patch parsing behavior are consistent.
- Linux world-agent: strict snapshot schema validation behavior matches Appendix protocol (HTTP + WS).
