# Objective acceptance fixture contract

Packet `SO-4.1` establishes only the committed harness scaffold and deterministic fixture layout.
Packet `SO-4.2` now commits the **expected-shape contract** for future cases, but the real family
corpus is still intentionally tiny until `SO-5.*` seeds the locked acceptance wall. The harness
must evaluate structured correctness from fixture metadata rather than reducing success to one exact
objective string.

The committed root must stay bounded to:

- `README.md`
- `design-set/`
- `locked-acceptance/`
- `stretch-external/`

`locked-acceptance/` is now seeded by `SO-5.1` with the WDAP kickoff cases
`wdap0-integ-linux-kickoff/` and `wdap0-integ-macos-kickoff/`. `design-set/` and
`stretch-external/` remain placeholder-only until their later `SO-5.*` follow-on packets land.
The harness must enumerate every committed `<case-id>/raw.json` and `<case-id>/expected.json`
entry deterministically.

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
