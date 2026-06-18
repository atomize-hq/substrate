# locked-acceptance

Packets `SO-5.1` and `SO-5.2` seed the required locked acceptance cases here:

- `instruction-surface-agents-skill-update/`
- `instruction-surface-available-skills-review/`
- `wdap0-integ-linux-kickoff/`
- `wdap0-integ-macos-kickoff/`

The WDAP fixtures must keep the `## Scope` mission as the canonical objective while explicitly
forbidding the subordinate `Run this task on a <platform> machine.` checklist line from being
promoted to `goal`.

The preserved instruction-surface fixtures must prove the extractor keeps deliberate `AGENTS.md`,
`<skill>`, `Available skills`, and similar instruction-surface targets even when surrounding
boilerplate includes filesystem, approval, or plugin-instruction text.
