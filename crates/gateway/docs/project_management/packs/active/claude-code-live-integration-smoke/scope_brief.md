---
pack_id: claude-code-live-integration-smoke
pack_version: v1
pack_status: extracted
source_ref: follow-on to landed gateway and Azure transport packs plus current Claude Code operator anchors in gateway/
execution_horizon:
  active_seam: null
  next_seam: null
---

# Scope Brief - Claude Code Live Integration Smoke

- **Goal**: plan the remaining operator-facing work so a real user with valid Azure Foundry credentials can configure Claude Code against this gateway, route think turns to `Kimi-K2-Thinking`, route default/execution turns to `Kimi-K2.5`, complete a live smoke run, and troubleshoot failures confidently.
- **Why now**: the prior packs already landed the normalized event core, Anthropic `/v1/messages` surface, planner/executor policy boundary, Azure Foundry transport contract, and gateway-backed live smoke contract. What remains is the exact operator path above those contracts: the reproducible setup, Claude Code attachment, real session smoke procedure, operator-visible evidence surfaces, and failure ownership map that make the live path executable in practice instead of only understandable from code and lower-layer docs.
- **Primary user(s) + JTBD**: Claude Code operators with real Azure Foundry access need to stand up the gateway, point Claude Code at it, run normal and think-mode turns plus tool-loop continuation, confirm the intended routing behavior, and know whether a failure belongs to Claude Code setup, gateway runtime/config, or Azure transport.
- **In-scope**:
  - canonical live setup flow for Azure credentials, gateway config, gateway startup, and Claude Code attachment
  - environment/config prerequisites and sequencing needed before a live run is meaningful
  - gateway startup and config-validation path as consumed by operators rather than only developers reading code
  - Claude Code client configuration path, including the gateway base URL, local auth placeholder posture, and any statusline or tracing hooks needed for operator evidence
  - live smoke-test scenarios for normal execution, think/planner mode, and tool-loop continuation behavior from actual Claude Code usage
  - expected routing evidence and operator-visible verification artifacts, including statusline, trace, log, or redacted transcript surfaces
  - troubleshooting flow and ownership boundaries for the most likely integration failures
  - operator-facing docs, examples, scripts, and checklists needed to make the live path reproducible
  - verification and closeout surfaces that prove end-to-end behavior in practice rather than only through unit or transport tests
- **Out-of-scope**:
  - redesigning Azure Foundry transport or auth behavior already covered by `azure-foundry-provider-transport`
  - redesigning normalized event semantics, the public `/v1/messages` surface, planner/executor policy, or downstream Substrate boundary contracts already covered by `azure-kimi-claude-gateway`
  - speculative production deployment work beyond what a realistic operator smoke path requires
  - generic provider expansion or multi-provider parity work beyond Azure-hosted Kimi
  - broad docs cleanup unrelated to the Claude Code live integration path
- **Success criteria**:
  - the pack identifies a bounded active seam that proves normal, think, and tool-loop continuation behavior from real Claude Code sessions on top of the published bootstrap path
  - the pack identifies a bounded active seam that freezes the troubleshooting ownership boundary against published bootstrap and live-smoke truth instead of provisional assumptions
  - the pack preserves landed `C-03`, `C-04`, `C-05`, `C-07`, and `C-08` truth and uses those contracts as basis instead of reopening them
  - a reviewer can explain which artifacts prove the live path works, which artifacts are only orientation, and which failures belong to Claude Code setup versus gateway runtime/config versus Azure transport
  - the pack leaves downstream seam planners with one clear path to make the live integration executable and supportable without inventing new threading rules
- **Constraints**:
  - `docs/project_management/packs/active/azure-kimi-claude-gateway` and `docs/project_management/packs/active/azure-foundry-provider-transport` are closeout-backed upstream basis and must be consumed rather than re-planned
  - `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/foundation/planner-executor-c04-policy-contract.md`, `docs/foundation/substrate-boundary-c05-contract.md`, `docs/foundation/substrate-structured-events-c06-contract.md`, `docs/foundation/azure-foundry-c07-runtime-transport-contract.md`, and `docs/foundation/azure-foundry-c08-operator-verification-contract.md` remain authoritative constraints
  - the live verification path must stay capability-oriented and preserve one logical backend identity, even when operators inspect internal routing evidence
  - the smoke path must use Claude Code through the landed Anthropic-compatible gateway route instead of provider-only bypasses
  - real Azure credentials are operator-owned and may not be available in CI, so redacted evidence and bounded manual procedures remain first-class outputs
- **External systems / dependencies**:
  - Claude Code client runtime and its Anthropic-compatible environment variables
  - Azure Foundry deployments for `Kimi-K2-Thinking` and `Kimi-K2.5`
  - gateway runtime/config surfaces in `gateway/src/server/mod.rs`, `gateway/src/router/mod.rs`, `gateway/src/providers/registry.rs`, `gateway/src/providers/openai.rs`, `gateway/src/cli/mod.rs`, `gateway/config/default.example.toml`, `gateway/config/models.example.toml`, and `gateway/README.md`
  - operator-local shell environment, config files, and any statusline or tracing artifacts under `~/.substrate-gateway/` and `~/.claude/`
- **Known unknowns / risks**:
  - Claude Code may impose operator-facing behaviors or sharp edges that are not covered by the gateway-only `/v1/messages` contract, especially around tool continuation and session lifecycle
  - routing evidence that is obvious to developers may still be too implicit for operators unless the pack freezes exactly which artifacts count as proof
  - statusline, trace, or log surfaces may expose too much provider or deployment detail unless the operator evidence contract is explicit
  - troubleshooting guidance can become muddled if the pack does not clearly separate Claude Code setup issues from gateway runtime/config issues and Azure transport issues
  - a smoke path described only as localhost convenience could drift into an unintended deployment assumption unless boundary language stays explicit
- **Assumptions**:
  - the upstream gateway and Azure transport pack closeouts remain current basis, including landed `C-07` and `C-08`
  - `Kimi-K2-Thinking` remains the intended internal target for think/planner behavior and `Kimi-K2.5` remains the intended internal target for default/execution behavior
  - the canonical operator path still runs through `ANTHROPIC_BASE_URL`, `ANTHROPIC_API_KEY`, and the landed `/v1/messages` surface, with the gateway preserving the outer capability boundary
  - `SEAM-1`, `SEAM-2`, and `SEAM-3` are now landed basis, `THR-10` is published, and no active seam remains in this pack's forward planning window
