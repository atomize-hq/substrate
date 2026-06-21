# Objective acceptance fixture contract

Packet `SO-4.1` establishes only the committed harness scaffold and deterministic fixture layout.
Packet `SO-4.2` now commits the **expected-shape contract** for future cases, and `SO-5.1` through
`SO-5.3` seed the first committed corpus. The harness must evaluate structured correctness from
fixture metadata rather than reducing success to one exact objective string.

The committed root must stay bounded to:

- `README.md`
- `design-set/`
- `locked-acceptance/`
- `stretch-external/`

`design-set/` is now seeded by `SO-5.3` with the concise /goal control
`concise-goal-architecture-doc-review/`. That small design-set control keeps an obvious
spec-review prompt available for future rule tuning without turning the design set into a second
locked wall.

`locked-acceptance/` is now seeded by `SO-5.1` through `SO-5.3` with:

- WDAP kickoff cases `wdap0-integ-linux-kickoff/` and `wdap0-integ-macos-kickoff/`
- preserved instruction-surface controls `instruction-surface-agents-skill-update/` and
  `instruction-surface-available-skills-review/`
- concise /goal control `concise-goal-objective-rs-review/`
- review/no-code control `review-no-code-so-5-3-audit/`
- planning/docs control `plan-docs-only-phase1-plan/`
- research/docs control `research-evaluation-wall-summary/`

`SO-2.3D` then adds two semantic-honesty controls:

- `review-implementation-noun-review-intent/` — a review prompt whose goal clause contains the noun
  "implementation"; `primary_intent` must stay `review`, not `implement` (intent is the request
  action, not an incidental substring).
- `orchestration-scaffolding-field-honesty/` — a concise `/goal` review plus a separate developer
  scaffolding row containing success/deliverable phrasing; `success_conditions` and `deliverables`
  must stay unknown rather than be pooled from the off-goal-surface boilerplate.

The `R5.75-1` Issue 1/2/3 anchoring fix adds two more controls:

- `orchestration-evaluate-ask-anchor/` — the minimized `019eb47f` shape: a developer scaffolding row
  and pasted AGENTS.md / `<skill>` bodies (all carrying goal-shaped verbs, a `/run/substrate.sock`
  path, and a cargo-style success ladder) precede the real user ask, whose phrasing ("use the
  `$code-review-and-quality` skill to evaluate if what was implemented landed correctly and
  completely") misses the goal-keyword heuristics. The structured objective must anchor its `goal` to
  that user ask (`primary_intent` `review`, `target` the grounded `docs/specs/r5` doc), never promote
  a `goal` onto the system-instruction or pasted-skill-body rows, and leave `success_conditions` /
  `deliverables` unknown instead of pooling them from the scaffolding.
- `orchestration-marker-free-boilerplate-exclusion/` — the robustness boundary for the anchoring
  fix: a developer instruction row dense with goal-shaped verbs ("Review and validate every change,
  ensure the suite stays green, and confirm the implementation is complete") but **without** the
  `AGENTS.md` / `<skill>` corpus markers that drive the row scorer negative. The real user ask
  ("Debug why crates/net/src/client.rs drops the retry header on the second attempt.") must still own
  the `goal` with `primary_intent` `debug` and a grounded file target. This proves the exclusion is
  carried by source-gating the `Goal` role to user/goal surfaces — a marker-independent mechanism —
  not solely by the `objective_score` markers, so goal-shaped instruction text on a system/developer
  surface can never become the goal even when it lacks the literal corpus tokens.

Those controls prove Phase 1 does not overfit to WDAP alone: deliberate `AGENTS.md`, `<skill>`,
`Available skills`, concise /goal, do not change code, docs-only, and research prompts all stay
semantically distinct, and intent/field assembly stays grounded to the active goal surface.
stretch-external/ remains placeholder-only. The harness must enumerate every committed
`<case-id>/raw.json` and `<case-id>/expected.json` entry deterministically.

## Case contract

Each future case directory must contain:

- `raw.json`
- `expected.json`

`raw.json` is the deterministic analyzer input for one acceptance case. Phase-1 cases should keep
this format simple and explicit:

```json
{
  "rows": [
    {
      "kind": "user_message",
      "user_message_role": "prompt",
      "text": "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
    }
  ]
}
```

`expected.json` defines the **expected shape** the harness validates. The harness must score
structured semantics, grounding, forbidden promotions, compatibility rendering, and unknown-field
honesty from this metadata:

```json
{
  "case_id": "explicit_file_target_grounding_contract",
  "objective_class": "task_statement",
  "primary_intent": "review",
  "target": {
    "kind": "file_or_directory",
    "display_contains": [
      "crates/agent-drift-analyzer/src/context/objective.rs"
    ]
  },
  "verification_commands": [],
  "role_spans": [
    {
      "role": "goal",
      "source_kind": "user_prompt",
      "excerpt_contains": "crates/agent-drift-analyzer/src/context/objective.rs",
      "exact_ref": {
        "source_file_suffix": "explicit_file_target_grounding_contract.jsonl",
        "event_index": 0,
        "row_ordinal": 0
      }
    }
  ],
  "field_evidence": {
    "target": [{
      "role": "goal",
      "source_kind": "user_prompt",
      "excerpt_contains": "crates/agent-drift-analyzer/src/context/objective.rs",
      "exact_ref": {
        "source_file_suffix": "explicit_file_target_grounding_contract.jsonl",
        "event_index": 0,
        "row_ordinal": 0
      }
    }],
    "constraints": [
      [
        {
          "role": "constraint",
          "source_kind": "user_prompt",
          "excerpt_contains": "do not change code",
          "exact_ref": {
            "source_file_suffix": "constraint_and_deliverable_grounding_contract.jsonl",
            "event_index": 0,
            "row_ordinal": 0
          }
        }
      ]
    ]
  },
  "forbidden_role_promotions": [
    {
      "forbidden_role": "goal",
      "source_kind": "user_prompt",
      "section_kind": "checklist",
      "excerpt_contains": "Run this task on a linux machine."
    }
  ],
  "compatibility_rendering": {
    "acceptable_any_of": [
      "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
    ],
    "comparison_key": "review|file_or_directory|crates_agent_drift_analyzer_src_context_objective_rs"
  },
  "required_unknown_fields": [],
  "forbidden_unknown_fields": [
    "target"
  ]
}
```

Notes:

- `acceptable_any_of` is the compatibility wall. Cases may allow multiple rendered strings when the
  structured frame is correct, but the harness must still enforce the semantic `comparison_key`
  whenever the case specifies one.
- `exact_ref` is the grounding-ref wall. Cases can pin an expected span to the originating
  synthetic or committed row via `source_file_suffix`, `event_index`, `row_ordinal`, and, when the
  extractor makes them available, `section_index` / `clause_index`.
- `role_spans` and `field_evidence` are separate on purpose: the first proves clause-role labeling,
  the second proves the extracted field stayed grounded to the right span.
- `field_evidence.constraints`, `field_evidence.success_conditions`, and
  `field_evidence.deliverables` are positional arrays that align with the corresponding
  `constraints`, `success_conditions`, and `deliverables` lists when a case wants to pin grounding
  for those structured fields.
- `forbidden_role_promotions` is the anti-WDAP guardrail. Checklist or scaffolding text must be
  able to fail the case even if the final rendered string still looks plausible.
- `required_unknown_fields` and `forbidden_unknown_fields` keep the extractor honest when evidence
  is weak. A case passes by leaving unsupported fields unknown rather than fabricating them.
