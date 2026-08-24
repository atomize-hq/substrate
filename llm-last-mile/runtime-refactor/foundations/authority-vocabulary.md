**Kind:** foundation
**Status:** canonical
**Canonical for:** authority vocabulary

## Authority vocabulary

- **Authority:** decides durable meaning and validates state transitions.
- **Host transition intent:** a durable, revision-bound, single-application request for `Start`, `Attach`, or `ResumeOneTurn`; helper plans and episodes transport it but never constitute its claim/application or erase its authority state.
- **Persistence:** stores authority decisions; it does not invent them.
- **Transport:** delivers requests/events; reachability is a signal, not durable truth.
- **Projection:** derives a view or runtime-native artifact from canonical truth.
- **Enforcement:** makes the policy unavoidable on the side-effecting path.
- **Receipt:** durable accepted-work identity returned before terminal completion.
- **Runtime event carrier:** producer-assigned stable stream/frame/event/terminal identity and
  monotonic ordering; it transports fact but owns neither durable observation nor semantics.
- **Producer replay registry:** a bounded, process-memory world-service index that retains exact
  B0 frames for one exact acceptance-record/stream/cursor lookup; it is transport availability,
  not durable supervisor or lifecycle truth.
- **Supervisor:** restart-safe owner of post-acceptance observation and closeout.
- **Supervisor recovery activation hook:** a production startup call that invokes one canonical
  supervisor recovery entry point and retains its observation tasks; it owns no discovery,
  reconciliation, journal interpretation, or terminal decision.
- **Materialization cut:** the ObligationLedger-owned proof that canonical obligation
  materialization covers an exact terminal event identity and sequence for one scoped run.
- **Secret handoff:** one-time secure-FD delivery from host credential authority to the in-world Substrate gateway; never a UAA-native credential file projection.
- **Install bootstrap context:** the one normalized, principal-bound host prefix selected at a public
  install/uninstall entry point and transported without child reinterpretation; in V1 its selected
  prefix, `SUBSTRATE_HOME`, and `SUBSTRATE_ROOT` are identical.
- **Platform bootstrap mapping:** an explicit commitment-preserving realization of that host context
  inside one exact Lima or WSL instance; it does not imply host/guest path or principal equality.
- **Runtime-family adapter:** provider mechanics only; never Substrate lifecycle or policy semantics.
- **Runtime placement versus session binding:** `AgentDescriptorV1.execution_scope` and the matching
  launch knob select where the runtime process executes. `DurableSessionAuthorityV1.world_binding`
  records the durable parent session's exact available world substrate. These are independent
  authority dimensions, not a bijection.

**Source provenance:** extracted byte-for-byte from [`00-README.md#authority-vocabulary`](../00-README.md#authority-vocabulary), baseline lines 148–179
**Baseline span SHA-256:** `9a08c1887f75093c73f60fe40928387b27be340bb754620ff73ce0b2b659902c`
