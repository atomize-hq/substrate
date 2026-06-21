# locked-acceptance

Packets `SO-5.1` through `SO-5.3` seed the required locked acceptance cases here:

- `concise-goal-objective-rs-review/`
- `instruction-surface-agents-skill-update/`
- `instruction-surface-available-skills-review/`
- `plan-docs-only-phase1-plan/`
- `research-evaluation-wall-summary/`
- `review-no-code-so-5-3-audit/`
- `wdap0-integ-linux-kickoff/`
- `wdap0-integ-macos-kickoff/`

`SO-2.3D` adds two semantic-honesty controls (`review-implementation-noun-review-intent/`,
`orchestration-scaffolding-field-honesty/`), and the `R5.75-1` Issue 1/2/3 fix adds two anchoring
controls:

- `orchestration-evaluate-ask-anchor/` — the minimized `019eb47f` shape: the goal must anchor to the
  real evaluate/review ask, never to the pasted system/AGENTS/`<skill>` scaffolding.
- `orchestration-marker-free-boilerplate-exclusion/` — the robustness boundary: goal-shaped developer
  instruction text (`Review and validate every change ...`) that lacks the `AGENTS.md` / `<skill>`
  markers must still lose to the real user ask (`Debug why crates/net/src/client.rs ...`). This proves
  the `Goal`-role source-gate, not the `objective_score` markers, keeps instruction text off the goal.

The WDAP fixtures must keep the `## Scope` mission as the canonical objective while explicitly
forbidding the subordinate `Run this task on a <platform> machine.` checklist line from being
promoted to `goal`.

The preserved instruction-surface fixtures must prove the extractor keeps deliberate `AGENTS.md`,
`<skill>`, `Available skills`, and similar instruction-surface targets even when surrounding
boilerplate includes filesystem, approval, or plugin-instruction text.

The `SO-5.3` fixtures extend that locked wall with:

- a concise `/goal` review control that should stay obvious and grounded,
- a review/no-code control that must not collapse into implementation intent,
- a docs-only planning control, and
- a research-summary control over the acceptance-wall design doc.
