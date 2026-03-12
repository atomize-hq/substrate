# Workstream triage fixture

### `WDRA-PWS-contract_typo` — contract

- Goal: fixture

### `WDRA-PWS-tasks_checkpoints` — tasks_checkpoints

- Goal: fixture

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "WDRA",
  "accepted_slice_order": [
    "WDRA0"
  ],
  "pws": [
    {
      "id": "WDRA-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [],
      "owns": [
        "contract.md"
      ]
    },
    {
      "id": "WDRA-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "WDRA-PWS-contract"
      ],
      "assumes": [],
      "owns": [
        "tasks.json",
        "session_log.md",
        "kickoff_prompts/",
        "slices/WDRA0/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->
