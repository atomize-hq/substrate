# world-deps-apt-provisioning contract

This draft-pack path is kept as a compatibility shim for tests, historical links, and older
references. The canonical contract now lives at
`docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`.

This shim does not define new behavior; it only preserves the legacy APT host-mutation guardrails
so older references keep resolving during the transition.

Preserved WDAP1 guidance:
- Runtime APT mutation remains unsupported on Windows.
- Substrate will not mutate the host OS during runtime `world deps current ...` flows.
- Operator remediation remains `substrate world enable --provision-deps`.
