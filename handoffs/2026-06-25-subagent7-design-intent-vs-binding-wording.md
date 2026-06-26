# Investigation: design intent vs binding wording

Date: 2026-06-25
Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
Mode: read-only investigation, paper-trail only

## Question
Did the wording around fresh toolbox world-dispatch read like a temporary slice/bootstrap limitation, while implementation froze it as a stronger semantic prerequisite that the host orchestrator session must already carry authoritative world binding before it can delegate world work?

## Verdict
Medium merit.

There is real ambiguity in the operator-facing wording, but the deeper design and runtime validators consistently treat authoritative parent-session world binding as part of the dispatch contract, not just a docs-only temporary workaround.

## Root-cause hypothesis
Two layers got compressed into one story:

1. **Semantic invariant**: world dispatch requests must carry `world_id` and `world_generation` that match the authoritative parent session binding.
2. **Current bootstrap limitation**: the public human surface does not let toolbox calls create that first binding, so the supported public path is `substrate agent start --backend <world-backend> --scope world`.

Because docs/prompt wording emphasizes "currently", "internal-only in this slice", and "supported public bootstrap", a reader can infer the restriction is only a temporary public-surface limitation. But the implementation injects and validates world binding as a required field, so in practice the stronger reading is frozen today.

## Strongest support for the misunderstanding theory
- `docs/USAGE.md:92-93`
  - "The first live toolbox-backed world-dispatch bootstrap remains internal-only in this slice."
  - "The supported public bootstrap for that posture is `substrate agent start ... --scope world` ..."
  - This reads like a phased public-surface rule.
- `crates/shell/src/execution/prompt_fulfillment.rs:99-100`
  - "host-only sessions cannot bootstrap the first binding through these tools"
  - Again, tool-surface wording, not deep contract wording.
- `handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md:26,58,77-78`
  - The June 21 paper trail explicitly frames `missing_world_binding` as the original blocker, resolved by the public world bootstrap posture, and notes the prompt wording was added to explain when fresh world-dispatch verbs are actionable.
- Git history strengthens that reading:
  - `docs/USAGE.md:92` came from `6c5c11d77` on 2026-05-30 as slice-scoped wording.
  - `docs/USAGE.md:93` and `prompt_fulfillment.rs:99-100` were added later by `41816ed07` on 2026-06-21 during live failure diagnosis.

## Strongest evidence against the misunderstanding theory
- `llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md:191-193`
  - `world_id` and `world_generation` are "required for world-bound execution and steering" and "must match the authoritative parent session binding."
- `.../DESIGN-host-orchestrator-world-dispatch-contract.md:308-312`
  - exact identity rules freeze world binding match as part of the contract.
- `.../DESIGN-host-orchestrator-world-dispatch-contract.md:336-342`
  - the policy model explicitly includes whether steering is restricted to the same authoritative world binding.
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:224-237`
  - retained continuity survives bootstrap exit, but exact backend/world binding remain authoritative routing inputs.
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md:363-372`
  - world mismatch is an invalidation cause and continuation must fail closed.

## Implementation surfaces that froze the stronger reading
- `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs:605-641`
  - fresh `run_world_task` / `spawn_world_worker` translation requires a `world_binding` input before building the request.
- `.../tool_invocation_contract.rs:1039-1053`
  - `world_id` and `world_generation` are always injected from runtime-owned binding into the internal dispatch request.
- `.../tool_invocation_contract.rs:1263-1278`
  - missing authoritative session world binding is an immediate `missing_world_binding` error.
- `crates/shell/src/execution/orchestrator_world_dispatch.rs:2387-2421`
  - dispatch validation fail-closes if the session has no authoritative world binding, or if request binding mismatches it.
- `crates/world-service/src/service.rs:2645-2681`
  - member dispatch requires authoritative active shared world binding on the world-service side too.
- `crates/shell/src/execution/prompt_fulfillment.rs:99-101,454-455`
  - the stronger reading is not just docs, it is baked into the runtime-owned prompt contract and pinned by tests.

## Classification
Best fit: **wording ambiguity plus intentional phased limitation**, not pure implementation drift.

Why:
- The semantic contract already assumed authoritative parent-session world binding.
- The operator-facing wording then described the current public bootstrap path in temporary slice language.
- The runtime/tooling implemented the semantic contract directly, so the stronger operational rule is real today.
- What drifted was the reader's likely interpretation, not the validator behavior.

## Important nuance
The stronger runtime rule is about the **session carrying authoritative world binding**, not about the host orchestrator itself becoming the world execution surface.

`docs/USAGE.md:125` is explicit that `substrate agent start --scope world` still creates a **host-rooted durable session** and merely persists host-attach truth plus world binding before `start` returns.

## Recommendation
Short term: rewrite the operator-facing wording to separate these two statements explicitly:
1. "Current public bootstrap path for first world binding"
2. "Semantic dispatch invariant once using world-dispatch tools"

If the product intent is eventually to allow toolbox-driven first-binding creation, document that as future scope. If not, say plainly that first-binding creation is intentionally out of contract for toolbox dispatch v1.
