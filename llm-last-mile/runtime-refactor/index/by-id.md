**Kind:** index
**Stable ID:** `runtime-refactor-by-id-index`
**Status:** non-authoritative navigation
**Canonical for:** identifier navigation only

# Runtime-refactor navigation by identifier

> **Authority boundary:** This file is an index only. It resolves stable identifiers, packet and task IDs, seam names, contract names and versions, gate IDs, evidence and issue IDs, and review-finding IDs to their canonical owners or most precise remaining historical owners when no standalone gate file exists; it does not authorize work or reinterpret those records.

## Decision stable titles

| Stable title | Canonical owner | Current status or scope |
|---|---|---|
| Decision — Linux-First Runtime-Refactor Resumption | [`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md) | `accepted for active scheduling; documentation-only decision` (2026-08-20) |
| Decision — macOS Developer Parity Boundary | [`macos-dev-parity/DECISION.md`](../macos-dev-parity/DECISION.md) | `accepted macOS-lane decision; global blocking status superseded on 2026-08-20` |

## Packet and packet-family identifiers

> **Registry note:** When one canonical file stores multiple exact packet or task rows in a single bounded table, this index links to that owning file or nearest real heading rather than inventing a synthetic per-row anchor.

| Identifier | Kind | Canonical owner | Current status or scope |
|---|---|---|---|
| `A1.1` | packet | [`a1-2-earlier-histories/slice-and-task.md#a11-aggregate-packet-row`](../a1-2-earlier-histories/slice-and-task.md#a11-aggregate-packet-row) | `historical authority-store aggregate packet record` |
| `A1.1a` | subpacket | [`a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record`](../a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record) | `historical internal codec and canonical-hash subpacket row` |
| `A1.1b` | subpacket | [`a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record`](../a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record) | `historical trusted-store filesystem-boundary subpacket row` |
| `A1.1c` | subpacket | [`a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record`](../a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record) | `historical bootstrap and crash-reconciliation subpacket row` |
| `A1.1d` | subpacket | [`a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record`](../a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record) | `historical transaction/CAS and writer-exclusion subpacket row` |
| `A1.1e` | subpacket | [`a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record`](../a1-2-earlier-histories/slice-and-task.md#a12-and-earlier-packet-histories-slice-and-task-record) | `historical facade integration and exact-resolution subpacket row` |
| `A1.1d-1` | checkpoint packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical retained opened-root transaction capability checkpoint` |
| `A1.1d-2` | checkpoint packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical guarded legacy-writer adoption checkpoint` |
| `A1.1d-3` | checkpoint packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical exact-retry and publication-candidate closure checkpoint` |
| `A1.1d-4` | checkpoint packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical cross-process replacement, crash, and regression closure checkpoint` |
| `A1.1d-5` | checkpoint packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical private SUBSTRATE_HOME creation and capability-parity checkpoint` |
| `A1.1d-5I` | audit packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical installer and bootstrap compatibility-audit packet` |
| `A1.1d-5R1` | packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical exact-allowlist packet record` |
| `A1.1d-5R2-0` | packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical prefix-propagation implementation-packet freeze` |
| `A1.1d-5R2-1` | packet | [`a1-2-earlier-histories/slice-and-task.md`](../a1-2-earlier-histories/slice-and-task.md) | `historical host-context construction and Unix dev-propagation packet` |
| `A1.2` | packet | [`a1-2-earlier-histories/slice-and-task.md#a12-aggregate-packet-row`](../a1-2-earlier-histories/slice-and-task.md#a12-aggregate-packet-row) | `historical aggregate intent issuance, application, and successor packet record` |
| `A1.2a` | subpacket | [`a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split`](../a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split) | `historical greenfield current-authority establishment/read prerequisite row` |
| `A1.2a-WB` | correction packet | [`a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split`](../a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split) | `historical Host/world-binding validation correction row` |
| `A1.2a-S` | prerequisite packet | [`a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split`](../a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split) | `historical bounded internal Start-adoption prerequisite row` |
| `A1.2b` | successor packet | [`a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split`](../a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split) | `historical successor and post-turn completion row` |
| `A1.3-P1` | completed packet | [`linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md`](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md) | `terminally complete` |
| `A1.3-P0` | held packet | [`linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](../linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md) | `held/non-implementable historical preparatory packet` |
| `A1.3` | held packet | [`linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md`](../linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md) | `preserved held historical Linux-first packet` |
| `A1.1d-5R2-2` | packet | [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f-family-slice-and-task-record`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f-family-slice-and-task-record) | `historical Unix release, sudo, Linux service, and runtime-propagation packet record` |
| `A1.1d-5R2-2E` | packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical authenticated world-gateway projection packet` |
| `A1.1d-5R2-2F0` | packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical deterministic world-socket test-isolation packet` |
| `A1.1d-5R2-2F0a` | packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical SUBSTRATE_HOME test-isolation packet` |
| `A1.1d-5R2-2F0b` | packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical renderer-output isolation packet` |
| `A1.1d-5R2-2F5-PD` | prerequisite packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical full passive-world-doctor prerequisite packet identity` |
| `A1.1d-5R2-2-B1` | phase | [`a1.1d-5r2-2-renewed-closeout/slice-and-task.md#a11d-5r2-2-b1-broad-wall-provenance-phase`](../a1.1d-5r2-2-renewed-closeout/slice-and-task.md#a11d-5r2-2-b1-broad-wall-provenance-phase) | `historical renewed-closeout broad-wall provenance phase` |
| `A1.1d-5R2-2F` | packet | [`a1.1d-5r2-2f/slice-and-task.md#historical-a11d-5r2-2f--authenticated-world-deps-and-truthful-doctor-composition`](../a1.1d-5r2-2f/slice-and-task.md#historical-a11d-5r2-2f--authenticated-world-deps-and-truthful-doctor-composition) | `historical authenticated world-deps and truthful doctor packet record` |
| `F0` | packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical socket-isolation packet preserved inside the completed F-family record` |
| `F0a` | packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical HOME-isolation packet preserved inside the completed F-family record` |
| `F0b` | packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical renderer-isolation packet preserved inside the completed F-family record` |
| `F0-HC` | packet | [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-hc-packet-insertion-and-authorization`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-hc-packet-insertion-and-authorization) | `historical harness-correction packet preserved inside the completed F-family record` |
| `A1.1d-5R2-2F0-HC` | packet | [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-hc-packet-insertion-and-authorization`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-hc-packet-insertion-and-authorization) | `exact internal F0-HC packet identity` |
| `F` | packet | [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f--completed-phase-record`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f--completed-phase-record) | `historical authenticated world-deps and truthful doctor packet shorthand within the completed F-family record` |
| `F5-PD` | prerequisite packet | [`a1.1d-5r2-2f/slice-and-task.md`](../a1.1d-5r2-2f/slice-and-task.md) | `historical passive-world-doctor prerequisite preserved inside the completed F-family record` |
| `A1.1d-5R2-4` | packet index | [`a1.1d-5r2-4/README.md`](../a1.1d-5r2-4/README.md) | `closed only as the bounded R2 propagation join` |
| `A1.1d-5R2-3` | packet | [`a1.1d-5r2-3/slice-and-task.md#a11d-5r2-3--platform-native-mapping-adapters`](../a1.1d-5r2-3/slice-and-task.md#a11d-5r2-3--platform-native-mapping-adapters) | `canonical platform-native mapping packet record` |
| `A1.1d-5R3` | packet family record | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | `archived authoritative R3 implementation index preserved as planning evidence` |
| `A1.1d-5R3-HOME` | task | [`a1.1d-5r3/slice-and-task.md#a11d-5r3-home`](../a1.1d-5r3/slice-and-task.md#a11d-5r3-home) | `archived R3 HOME implementation task in preserved planning evidence` |
| `A1.1d-5R3-MANIFEST` | task | [`a1.1d-5r3/slice-and-task.md#a11d-5r3-manifest`](../a1.1d-5r3/slice-and-task.md#a11d-5r3-manifest) | `archived R3 manifest implementation task in preserved planning evidence` |
| `A1.1d-5R3-LINUX` | task | [`a1.1d-5r3/slice-and-task.md#a11d-5r3-linux`](../a1.1d-5r3/slice-and-task.md#a11d-5r3-linux) | `archived R3 Linux implementation task in preserved planning evidence` |
| `A1.1d-5R3-LINUX-CLOSEOUT` | closeout task | [`a1.1d-5r3/slice-and-task.md#evidencer3-linux-imp-01-and-a11d-5r3-linux-closeout`](../a1.1d-5r3/slice-and-task.md#evidencer3-linux-imp-01-and-a11d-5r3-linux-closeout) | `archived R3 Linux closeout task in preserved planning evidence` |
| `A1.1d-5R3-MAC` | task | [`a1.1d-5r3/slice-and-task.md#a11d-5r3-mac`](../a1.1d-5r3/slice-and-task.md#a11d-5r3-mac) | `archived R3 macOS implementation task in preserved planning evidence` |
| `A1.1d-5R3-MAC-CLOSEOUT` | closeout task | [`a1.1d-5r3/slice-and-task.md#evidencer3-mac-imp-01-and-a11d-5r3-mac-closeout`](../a1.1d-5r3/slice-and-task.md#evidencer3-mac-imp-01-and-a11d-5r3-mac-closeout) | `archived R3 macOS closeout task in preserved planning evidence` |
| `A1.1d-5R3-WIN` | task | [`a1.1d-5r3/slice-and-task.md#a11d-5r3-win`](../a1.1d-5r3/slice-and-task.md#a11d-5r3-win) | `archived R3 Windows implementation task in preserved planning evidence` |
| `A1.1d-5R3-WIN-CLOSEOUT` | closeout task | [`a1.1d-5r3/slice-and-task.md#evidencer3-win-imp-01-and-a11d-5r3-win-closeout`](../a1.1d-5r3/slice-and-task.md#evidencer3-win-imp-01-and-a11d-5r3-win-closeout) | `archived R3 Windows closeout task in preserved planning evidence` |
| `A1.1d-5R3-UNIX` | task | [`a1.1d-5r3/slice-and-task.md#a11d-5r3-unix`](../a1.1d-5r3/slice-and-task.md#a11d-5r3-unix) | `archived R3 cross-platform Unix task in preserved planning evidence` |
| `A1.1d-5R3-CLOSEOUT` | closeout task | [`a1.1d-5r3/slice-and-task.md#a11d-5r3-closeout`](../a1.1d-5r3/slice-and-task.md#a11d-5r3-closeout) | `archived final R3 closeout task in preserved planning evidence` |
| `B0` | packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical durable runtime event identity and ordering carrier row` |
| `B1` | packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical receipt-core family packet record` |
| `B1-3a` | subpacket | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical activated-store receipt transaction substrate subpacket row` |
| `B1-3b` | subpacket | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical authority-bound receipt-registry adoption subpacket row` |
| `B1-3a/B1-3b` | combined subpacket row | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `exact combined receipt-core subpacket row covering the activated-store substrate and authority-bound registry adoption` |
| `B2.1` | packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical durable observation, replay, and restart family packet record` |
| `B2.1-1` | subpacket | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical no-gap durable claim handoff subpacket row` |
| `B2.1-2` | subpacket | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical durable frame and event journal subpacket row` |
| `B2.1-3` | subpacket | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical restart and terminal reconciliation subpacket row` |
| `B2.1-1/B2.1-2/B2.1-3` | combined subpacket row | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `exact combined supervisor subpacket row covering no-gap handoff, journaling, and restart reconciliation` |
| `R0` | packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `historical retained-target protocol prerequisite shorthand within the B1/B2.1 family record` |
| `B1/B2.1-R0` | prerequisite packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `exact canonical retained-target protocol prerequisite row` |
| `B3.2a` | prerequisite packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `exact retained creation/admission bridge prerequisite row` |
| `B3.2a-WA` | prerequisite packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `exact bound-world ownership adoption prerequisite row` |
| `B1/B2.1-0` | prerequisite packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `exact action-scoped dispatch-preparation prerequisite row` |
| `B1/B2.1` | joint closeout packet | [`b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record`](../b1-b2-1/slice-and-task.md#b1b21-family-slice-and-task-record) | `exact joint production integration closeout row` |
| `B3.1` | packet | [`b3-1-c1/slice-and-task.md#b31-packet-row`](../b3-1-c1/slice-and-task.md#b31-packet-row) | `exact retained-turn event and causation prerequisite row` |
| `C1` | packet | [`b3-1-c1/slice-and-task.md#c1-packet-row`](../b3-1-c1/slice-and-task.md#c1-packet-row) | `exact event-to-obligation materializer and semantic-cut row` |
| `E2-RM` | prerequisite packet | [`contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite) | `specified-but-unadmitted E2-owned accepted-work receipt-material projection required before fresh B2.2 admission` |
| `A1.3-family` | packet index | [`linux-first-runtime-resumption/README.md`](../linux-first-runtime-resumption/README.md) | `non-authoritative navigation for completed A1.3-P1 and its held A1.3-P0/A1.3 predecessors` |
| `macOS-dev-parity-family` | lane index | [`macos-dev-parity/README.md`](../macos-dev-parity/README.md) | `non-authoritative navigation for the macOS developer-parity decision, extracted projections, and lane-local gate` |
| `A1.1d-5R3-family` | packet index | [`a1.1d-5r3/README.md`](../a1.1d-5r3/README.md) | `archived for active scheduling; non-authoritative navigation for the extracted R3 implementation-family owners` |
| `r3-mac-evidence-recovery-family` | planning index | [`r3-mac-evidence-recovery/README.md`](../r3-mac-evidence-recovery/README.md) | `planning-only authority preserved; non-authoritative navigation for extracted recovery status and correction projections` |
| `A1.2-earlier-histories-family` | packet-family index | [`a1-2-earlier-histories/README.md`](../a1-2-earlier-histories/README.md) | `historical completed-family navigation for the internal A1.2 corridor and earlier A0/A1.1/A1.1d/A1.1e packet histories` |
| `B3.1-C1-family` | packet-family index | [`b3-1-c1/README.md`](../b3-1-c1/README.md) | `historical completed successor-family navigation for typed retained-event semantics and obligation materialization/semantic-cut closure` |
| `B1-B2.1-family` | packet-family index | [`b1-b2-1/README.md`](../b1-b2-1/README.md) | `historical completed-family navigation for receipt acceptance, durable supervision/replay, prerequisites, and joint production closeout` |
| `A1.1d-5R2-3-family` | packet index | [`a1.1d-5r2-3/README.md`](../a1.1d-5r2-3/README.md) | `closed only as the bounded docs-first mapping and closeout family` |
| `A1.1d-5R2-2-renewed-closeout-family` | packet index | [`a1.1d-5r2-2-renewed-closeout/README.md`](../a1.1d-5r2-2-renewed-closeout/README.md) | `historical closeout-only navigation for the renewed R2-2 publication, remediation, and RP records` |
| `A1.1d-5R2-2F-family` | packet index | [`a1.1d-5r2-2f/README.md`](../a1.1d-5r2-2f/README.md) | `historical family navigation for F0/F0a/F0b/F0-HC, F, and F5-PD closure` |

## Contract names and versions

> **Registry note:** This table covers both standalone extracted contract files and exact historical or family-local schema identifiers that remain canonically housed inside bounded contract-and-gate records.

| Identifier | Preferred canonical owner | Scope |
|---|---|---|
| `CanonicalJsonV1` | [`a1-2-earlier-histories/contracts-and-gates.md#canonicaljsonv1-and-timestamp-encoding`](../a1-2-earlier-histories/contracts-and-gates.md#canonicaljsonv1-and-timestamp-encoding) | `exact A1 canonical JSON codec and hash-input encoding contract` |
| `TimestampV1` | [`a1-2-earlier-histories/contracts-and-gates.md#canonicaljsonv1-and-timestamp-encoding`](../a1-2-earlier-histories/contracts-and-gates.md#canonicaljsonv1-and-timestamp-encoding) | `exact canonical timestamp encoding member of the A1 codec contract` |
| `PrivateSubstrateHomeV1` | [`a1-2-earlier-histories/contracts-and-gates.md#privatesubstratehomev1-acceptance-rules`](../a1-2-earlier-histories/contracts-and-gates.md#privatesubstratehomev1-acceptance-rules) | `exact authenticated selected-home and path-identity contract` |
| `InstallBootstrapContextV1` | [`a1-2-earlier-histories/contracts-and-gates.md#installbootstrapcontextv1-propagation-rules`](../a1-2-earlier-histories/contracts-and-gates.md#installbootstrapcontextv1-propagation-rules) | `exact install, uninstall, and world-enable authenticated bootstrap-context contract` |
| `InstallBootstrapContextCarrierV1` | [`a1-2-earlier-histories/contracts-and-gates.md#installbootstrapcontextv1-propagation-rules`](../a1-2-earlier-histories/contracts-and-gates.md#installbootstrapcontextv1-propagation-rules) | `exact boundary-carrier encoding for InstallBootstrapContextV1` |
| `PlatformBootstrapMappingV1` | [`a1-2-earlier-histories/contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification`](../a1-2-earlier-histories/contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification) | `exact platform bootstrap-mapping contract for authenticated projections` |
| `AuthorityStoreInitializationV1` | [`a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace`](../a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace) | `exact greenfield authority-store initialization contract` |
| `CanonicalDirectoryV1` | [`a1-2-earlier-histories/contracts-and-gates.md#exact-directory-identity`](../a1-2-earlier-histories/contracts-and-gates.md#exact-directory-identity) | `exact trusted opened-directory identity contract` |
| `StateRootV1` | [`a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace`](../a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace) | `strict pre-A1 and A1 root schema` |
| `StateRootV2` | [`a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace`](../a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace) | `strict A1.2a root schema` |
| `GreenfieldNamespaceCertificateV1` | [`a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace`](../a1-2-earlier-histories/contracts-and-gates.md#strict-staterootv1staterootv2-and-the-session-namespace) | `exact greenfield namespace-certificate schema` |
| `LegacyStateStoreTransactionV1` | [`a1-2-earlier-histories/contracts-and-gates.md#legacystatestoretransactionv1-retained-root-contract`](../a1-2-earlier-histories/contracts-and-gates.md#legacystatestoretransactionv1-retained-root-contract) | `exact bounded retained-root legacy state-store transaction contract` |
| `DurableSessionAuthorityV1` | [`a1-2-earlier-histories/contracts-and-gates.md#1-durablesessionauthorityv1`](../a1-2-earlier-histories/contracts-and-gates.md#1-durablesessionauthorityv1) | `exact durable session-authority schema` |
| `HostSessionTransitionIntentV1` | [`a1-2-earlier-histories/contracts-and-gates.md#1a-strict-hostsessiontransitionintentv1hostsessiontransitionintentv2`](../a1-2-earlier-histories/contracts-and-gates.md#1a-strict-hostsessiontransitionintentv1hostsessiontransitionintentv2) | `strict V1 transition-intent contract` |
| `HostSessionTransitionIntentV2` | [`a1-2-earlier-histories/contracts-and-gates.md#1a-strict-hostsessiontransitionintentv1hostsessiontransitionintentv2`](../a1-2-earlier-histories/contracts-and-gates.md#1a-strict-hostsessiontransitionintentv1hostsessiontransitionintentv2) | `strict V2 transition-intent contract` |
| `AuthenticatedWorldDepsContextV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact authenticated world-deps context contract` |
| `WorldDoctorReportV1` | [`a1-2-earlier-histories/contracts-and-gates.md#route-c-authenticated-hostworld-doctor-projection`](../a1-2-earlier-histories/contracts-and-gates.md#route-c-authenticated-hostworld-doctor-projection) | `exact authenticated Host and World doctor projection contract` |
| `WorldDepsDoctorSnapshotV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact world-deps doctor snapshot contract` |
| `PassiveWorldDoctorHostV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact passive nested world-doctor host projection contract` |
| `PassiveWorldDoctorWorldV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact passive nested world-doctor world projection contract` |
| `PassiveWorldDoctorChildV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact passive nested world-doctor child projection contract` |
| `HostExecutionEpisodeV1` | [`contracts/host-execution-episode-v1.md`](../contracts/host-execution-episode-v1.md) | `exact extracted D10 contract owner` |
| `Development review and remediation contract` | [`contracts/development-review-and-remediation-contract.md`](../contracts/development-review-and-remediation-contract.md) | `exact named development-review contract` |
| `Deferred retained-spawn admission recovery contract` | [`contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | `exact extracted D10 contract owner` |
| `RetainedWorkerAdmissionCommitmentCarrierV1` | [`contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | `exact extracted recovery-carrier schema` |
| `RetainedWorkerLaunchAuthorityProofV1` | [`contracts/deferred-retained-spawn-admission-recovery-contract.md`](../contracts/deferred-retained-spawn-admission-recovery-contract.md) | `exact extracted recovery-proof schema` |
| `ActiveEphemeralTaskReceiptV1` | [`contracts/active-ephemeral-task-receipt-v1.md`](../contracts/active-ephemeral-task-receipt-v1.md) | `exact extracted D10 contract owner` |
| `WorldWorkTerminalV1` | [`contracts/active-ephemeral-task-receipt-v1.md`](../contracts/active-ephemeral-task-receipt-v1.md) | `exact active-receipt terminal schema reused by ephemeral and retained receipt records` |
| `ActiveRetainedTurnReceiptV1` | [`contracts/active-retained-turn-receipt-v1.md`](../contracts/active-retained-turn-receipt-v1.md) | `exact extracted D10 contract owner` |
| `RetainedWorkerManifestV1` | [`contracts/retained-worker-manifest-v1.md`](../contracts/retained-worker-manifest-v1.md) | `exact extracted D10 contract owner` |
| `DispatchPolicyNarrowingPatchV1` | [`contracts/dispatch-policy-narrowing-patch-v1.md`](../contracts/dispatch-policy-narrowing-patch-v1.md) | `exact extracted D10 contract owner` |
| `DispatchCapabilitySubjectV1` | [`contracts/dispatch-policy-narrowing-patch-v1.md`](../contracts/dispatch-policy-narrowing-patch-v1.md) | `exact extracted narrowing-subject enum` |
| `RestrictedPolicyPatchV1` | [`contracts/dispatch-policy-narrowing-patch-v1.md`](../contracts/dispatch-policy-narrowing-patch-v1.md) | `exact extracted restricted patch schema name` |
| `DispatchPolicyCommitmentV1` | [`contracts/dispatch-policy-commitment-v1.md`](../contracts/dispatch-policy-commitment-v1.md) | `canonical E2 immutable policy commitment/cap record` |
| `DispatchPolicyCommitmentSubjectV1` | [`contracts/dispatch-policy-commitment-v1.md`](../contracts/dispatch-policy-commitment-v1.md) | `subject-discriminated ephemeral/launch/turn/fork identity` |
| `PolicyCommitmentAuthorityLinkV1` | [`contracts/dispatch-policy-commitment-v1.md`](../contracts/dispatch-policy-commitment-v1.md) | `subject-specific B1, fresh-Spawn B3.2a, or E2 fork-dispatch link` |
| `DispatchPolicyCommitmentRefV1` | [`contracts/dispatch-policy-commitment-v1.md`](../contracts/dispatch-policy-commitment-v1.md) | `exact downstream receipt/manifest linkage identity` |
| `DispatchPolicyCommitmentLookupKeyV1` | [`contracts/dispatch-policy-commitment-v1.md`](../contracts/dispatch-policy-commitment-v1.md) | `durable unique request/subject exact-retry index` |
| `RetainedWorkerCapLinkV1` | [`contracts/dispatch-policy-commitment-v1.md`](../contracts/dispatch-policy-commitment-v1.md) | `self-safe new-cap or exact existing-cap linkage` |
| `PolicyCommitmentCompatibilityResultV1` | [`contracts/dispatch-policy-commitment-v1.md`](../contracts/dispatch-policy-commitment-v1.md) | `typed E2 mixed-version continue/fork result` |
| `AcceptedWorkReceiptMaterialResolutionV1` | [`contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite) | `specified crate-internal resolved-or-UnsupportedLegacyState result for E2-RM` |
| `AcceptedWorkReceiptMaterialLegacyReasonV1` | [`contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite) | `specified exact no-V1-footprint legacy classification for E2-RM` |
| `AuthenticatedAcceptedWorkReceiptMaterialV1` | [`contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite) | `specified opaque read-only historic E2 material projection for later B2.2 consumption` |
| `AuthenticatedAcceptedWorkExecutionClaimV1` | [`contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite) | `specified preserved B2.1 claim identity, canonical preimage, durable key, and authenticated linkage hash` |
| `AuthenticatedAcceptedWorkRetainedCapV1` | [`contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite) | `specified exact retained-worker cap ref/hash/bytes projection within E2-RM` |
| `ConfigProjectionIdentityV1` | [`contracts/agent-config-projection-v1.md`](../contracts/agent-config-projection-v1.md) | `exact immutable E3 projection-series identity` |
| `ConfigProjectionRefV1` | [`contracts/agent-config-projection-v1.md`](../contracts/agent-config-projection-v1.md) | `exact accepted-home E3 record reference` |
| `AgentConfigProjectionRecordV1` | [`contracts/agent-config-projection-v1.md`](../contracts/agent-config-projection-v1.md) | `complete logical/effective/native/gateway/handoff projection record` |
| `ManagedGatewayActivationIntentV1` | [`contracts/managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md) | `exact preactivation intent under the zero-live fence` |
| `ManagedGatewayActivationAckV1` | [`contracts/managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md) | `non-secret exact gateway/handoff/readiness/access-boundary evidence` |
| `InWorldGatewayRefV1` | [`contracts/managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md) | `preallocated immutable E3 gateway identity reference` |
| `GatewayAccessBoundaryV1` | [`contracts/managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md) | `Linux exact-member cgroup/nftables access boundary` |
| `WorldRuntimeAdapterExecutionEnvelopeV1` | [`contracts/world-runtime-adapter-execution-envelope-v1.md`](../contracts/world-runtime-adapter-execution-envelope-v1.md) | `exact extracted D10 contract owner` |
| `LaunchTimeSecretHandoffV1` | [`contracts/launch-time-secret-handoff-v1.md`](../contracts/launch-time-secret-handoff-v1.md) | `exact extracted D10 contract owner` |
| `GatewayAuthBundleV1` | [`contracts/launch-time-secret-handoff-v1.md`](../contracts/launch-time-secret-handoff-v1.md) | `exact extracted gateway-handoff primitive` |
| `Cancel outcome categories` | [`contracts/cancel-outcome-categories.md`](../contracts/cancel-outcome-categories.md) | `exact named cancel-outcome contract owner` |
| `CancelWorldWorkOutcomeV1` | [`contracts/cancel-outcome-categories.md`](../contracts/cancel-outcome-categories.md) | `exact extracted cancel-outcome enum` |
| `RuntimeFrameIdentityV1` | [`b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier`](../b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier) | `exact B0 stream frame-identity carrier` |
| `RuntimeEventIdentityV1` | [`b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier`](../b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier) | `exact B0 semantic event-identity carrier` |
| `RuntimeTerminalIdentityV1` | [`b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier`](../b1-b2-1/contracts-and-gates.md#2a-runtime-event-identity-and-ordering-carrier) | `exact B0 terminal identity carrier` |
| `GreenfieldHostStartProposalV1` | [`b1-b2-1/contracts-and-gates.md#b1b21-bounded-read-only-dispatch-authority-adapter`](../b1-b2-1/contracts-and-gates.md#b1b21-bounded-read-only-dispatch-authority-adapter) | `exact internal A1.2a-S greenfield Start-proposal contract consumed by B-owned paths` |
| `RetainedWorkerAuthorityRegistrationV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact non-transition retained-worker authority-registration proof` |
| `ExactBoundWorldOwnershipAdoptionV1` | [`b1-b2-1/contracts-and-gates.md#b32a-wa-exactboundworldownershipadoptionv1`](../b1-b2-1/contracts-and-gates.md#b32a-wa-exactboundworldownershipadoptionv1) | `exact B3.2a-WA internal world-ownership adoption contract` |
| `WorldWorkAcceptanceContextV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact accepted-work submission-context contract` |
| `WorldWorkAcceptanceRecordV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact immutable acceptance-record contract` |
| `MemberDispatchRequestV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact member-dispatch transport request schema carrying retained launch proof when present` |
| `MemberDispatchRequestV2` | [`contracts/agent-config-projection-v1.md#memberdispatchrequestv2-carrier-and-d1s-later-v3`](../contracts/agent-config-projection-v1.md#memberdispatchrequestv2-carrier-and-d1s-later-v3) | `strict additive E3 config-projection carrier; V1 byte/behavior unchanged` |
| `MemberTurnSubmitRequestV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact retained member-turn submission contract carrying acceptance context` |
| `PolicySnapshotRefV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact accepted policy-snapshot reference contract` |
| `ValidatedWorldDispatchRequestV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact validated world-dispatch request contract` |
| `WorldWorkReceiptRegistryStateV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact receipt-registry state schema` |
| `WorldWorkReceiptRegistryStorageV1` | [`b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1`](../b1-b2-1/contracts-and-gates.md#2c-worldworkacceptancerecordv1) | `exact receipt-registry storage capability and schema` |
| `WorldWorkExecutionSupervisorStorageV1` | [`b1-b2-1/contracts-and-gates.md#11-supervisor-idempotency-and-restart-rules`](../b1-b2-1/contracts-and-gates.md#11-supervisor-idempotency-and-restart-rules) | `exact supervisor storage capability and schema` |
| `PolicySnapshotV3` | [`gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md`](../gates/final-receipt-immutable-policy-snapshot-v3-acceptance.md) | `exact immutable final-receipt acceptance identifier` |
| `NormalizedWorldWorkerEventFacetV1` | [`b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope`](../b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope) | `exact B3.1 producer-normalized retained event facet` |
| `OpaqueAuthorityCommitmentV1` | [`b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope`](../b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope) | `exact unchanged copied authority-commitment carrier used by B3.1 and C1` |
| `WorldWorkerEventV1` | [`b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope`](../b3-1-c1/contracts-and-gates.md#2b-bounded-retained-worker-event-envelope) | `exact retained worker-to-host event-envelope contract consumed by B3.1 and C1` |
| `ObligationLedgerSnapshotReadV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact obligation-ledger snapshot read contract consumed by A1.2b and C1` |
| `AgentDescriptorV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `ApplicationResultHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `AuthoritativeLineageHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `AuthorityObjectKindV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `AuthorityObjectRefV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `AuthorityStoreCommitmentKeyFileV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `CurrentAttemptTempRollbackV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `DirectoryPhysicalIdentityV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `DurableSessionAuthorityHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `ExecutorBuildEvidenceV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherBootstrapHelloV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherBootstrapTranscriptV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherPairingGuestIntentV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherPairingTicketV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherReservationUnusedAcknowledgementV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherReservationUnusedProofV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherRetirementReservationV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherTestRetirementAcknowledgementV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherTestRetirementAuthorizationV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherTestRetirementCommitmentV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `GuestPublisherTestRetirementReceiptV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `HostAttachContractV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `HostPostTurnProtocolEventKindV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `HostSessionPostTurnApplicationV2` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `HostSessionTransitionApplicationJournalV2` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `HostSessionTransitionAttemptRejectionV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `HostSessionTransitionPayloadHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `HostStartupOwnershipEvidenceV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `HostStartupOwnershipProtocolEventV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `InputAcceptanceHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `LifecyclePublisherAnchorV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `LifecyclePublisherProtectedStateV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `LifecycleSignatureV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `LimaStageOneAuthorizationV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedActionPreparedRecordV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedActionReceiptIndexEntryV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedActionReceiptIndexV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedActionReceiptV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedAdoptionAuthorizationV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedArtifactRoleV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedLifecyclePublisherRequestV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ManagedManifestHeadV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ObligationAttentionDispositionV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `ObligationSnapshotRecordHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `PlatformPrincipalV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `PolicyObjectHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `PostTurnApplicationJournalV2` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `PostTurnCompletionHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `PostTurnProtocolEventHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `PublisherBootstrapAuthorizationV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `PublisherBootstrapComponentRoleV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `PublisherBootstrapComponentV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `PublisherTestRetirementAcknowledgementV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `PublisherTestRetirementAuthorizationV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `PublisherTestRetirementReceiptV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `ResumeHandleHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `RetainedWorkerObjectHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `StartupOwnershipResultHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `StartupOwnershipTerminalApplicationJournalV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `SubstrateLifecyclePublisherV1` | [`a1.1d-5r3/contracts-and-gates.md`](../a1.1d-5r3/contracts-and-gates.md) | `exact archived R3 contract or schema identifier preserved in the linked canonical owner` |
| `TerminalHandoffHashInputV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `TerminalHandoffHashInputV2` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `TransitionTransportPayloadObjectV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |
| `UnsupportedNonGreenfieldRootV1` | [`a1-2-earlier-histories/contracts-and-gates.md`](../a1-2-earlier-histories/contracts-and-gates.md) | `exact historical A1 and A1.2 contract or schema identifier preserved in the linked canonical owner` |

| `R3-CANDIDATE-01` | [`a1.1d-5r3/contracts-and-gates.md#r3-candidate-01--exact-synchronous-candidate-rollback`](../a1.1d-5r3/contracts-and-gates.md#r3-candidate-01--exact-synchronous-candidate-rollback) | `exact archived R3 named lifecycle contract identifier; never current scheduling or action authority` |
| `R3-MANIFEST-01` | [`a1.1d-5r3/contracts-and-gates.md#r3-manifest-01--canonical-managed-artifact-authority`](../a1.1d-5r3/contracts-and-gates.md#r3-manifest-01--canonical-managed-artifact-authority) | `exact archived R3 named lifecycle contract identifier; never current scheduling or action authority` |
| `R3-TEMP-ROLLBACK-01` | [`a1.1d-5r3/contracts-and-gates.md#r3-temp-rollback-01--pre-manifest-current-attempt-temporary-trees`](../a1.1d-5r3/contracts-and-gates.md#r3-temp-rollback-01--pre-manifest-current-attempt-temporary-trees) | `exact archived R3 named lifecycle contract identifier; never current scheduling or action authority` |
| `R3-CLASS-01` | [`a1.1d-5r3/contracts-and-gates.md#r3-class-01--mutually-explicit-lifecycle-classes`](../a1.1d-5r3/contracts-and-gates.md#r3-class-01--mutually-explicit-lifecycle-classes) | `exact archived R3 named lifecycle contract identifier; never current scheduling or action authority` |
| `R3-ACTION-01` | [`a1.1d-5r3/contracts-and-gates.md#r3-action-01--mutation-and-crashretry-protocol`](../a1.1d-5r3/contracts-and-gates.md#r3-action-01--mutation-and-crashretry-protocol) | `exact archived R3 named lifecycle contract identifier; never current scheduling or action authority` |
| `R3-PRESERVE-01` | [`a1.1d-5r3/contracts-and-gates.md#r3-preserve-01--exact-removal-and-restoration`](../a1.1d-5r3/contracts-and-gates.md#r3-preserve-01--exact-removal-and-restoration) | `exact archived R3 named lifecycle contract identifier; never current scheduling or action authority` |

## Gate IDs

| Gate ID | Preferred canonical owner | Scope |
|---|---|---|
| `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` | [`gates/authority-required-macos-dev-parity.md`](../gates/authority-required-macos-dev-parity.md) | dedicated current macOS-lane gate |
| `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` | [`gates/authority-required-runtime-refactor-reentry.md`](../gates/authority-required-runtime-refactor-reentry.md) | dedicated closed global selection gate |
| `AUTHORITY_REQUIRED:B1_B2_1_JOINT_CLOSEOUT` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical predecessor gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:R3_RESUME` | [`a1.1d-5r2-4/terminal-gate-disposition.md`](../a1.1d-5r2-4/terminal-gate-disposition.md) | historical next-gate disposition after R2-4 |
| `AUTHORITY_REQUIRED:R3_IMPLEMENTATION` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 implementation-start gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-MANIFEST` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 successor gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 Linux gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 Linux closeout gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-MAC` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 macOS gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 macOS closeout gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-WIN` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 Windows gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-WIN-CLOSEOUT` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 Windows closeout gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-UNIX` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical R3 UNIX gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-MAC-01` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical native-evidence successor gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-WIN-01` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical native-evidence successor gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-5R3-CLOSEOUT` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical final R3 closeout gate in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:A1.1d-CLOSEOUT` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical terminal closeout successor in the authoritative R3 implementation index |
| `AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-IMPLEMENTATION` | [`r3-mac-evidence-recovery/SPEC.md`](../r3-mac-evidence-recovery/SPEC.md) | planning-only recovery successor gate with no standalone gate file |

## Evidence IDs

| Evidence ID | Preferred canonical owner | Scope |
|---|---|---|
| `EVIDENCE:R3-LINUX-IMP-01` | [`a1.1d-5r3/evidence-regression.md`](../a1.1d-5r3/evidence-regression.md) | historical Linux proof/evidence owner |
| `EVIDENCE:R3-MAC-IMP-01` | [`a1.1d-5r3/evidence-regression.md`](../a1.1d-5r3/evidence-regression.md) | historical macOS proof/evidence owner |
| `EVIDENCE:R3-WIN-IMP-01` | [`a1.1d-5r3/evidence-regression.md`](../a1.1d-5r3/evidence-regression.md) | historical Windows proof/evidence owner |
| `EVIDENCE:R3-NATIVE-LINUX-01` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical native Linux proof task in the authoritative R3 implementation index |
| `EVIDENCE:R3-NATIVE-MAC-01` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical native macOS proof task in the authoritative R3 implementation index |
| `EVIDENCE:R3-NATIVE-WIN-01` | [`a1.1d-5r3/slice-and-task.md`](../a1.1d-5r3/slice-and-task.md) | historical native Windows proof task in the authoritative R3 implementation index |

## Stable IDs

| Stable ID | Preferred canonical owner | Status |
|---|---|---|
| `A1.1d-5R2-2-renewed-closeout-family` | [`a1.1d-5r2-2-renewed-closeout/README.md`](../a1.1d-5r2-2-renewed-closeout/README.md) | non-authoritative navigation |
| `A1.1d-5R2-2F-family` | [`a1.1d-5r2-2f/README.md`](../a1.1d-5r2-2f/README.md) | non-authoritative navigation |
| `A1.1d-5R2-3-family` | [`a1.1d-5r2-3/README.md`](../a1.1d-5r2-3/README.md) | non-authoritative navigation |
| `A1.1d-5R2-4` | [`a1.1d-5r2-4/README.md`](../a1.1d-5r2-4/README.md) | non-authoritative navigation |
| `A1.1d-5R3-family` | [`a1.1d-5r3/README.md`](../a1.1d-5r3/README.md) | non-authoritative navigation |
| `A1.2-earlier-histories-family` | [`a1-2-earlier-histories/README.md`](../a1-2-earlier-histories/README.md) | non-authoritative navigation |
| `A1.3-family` | [`linux-first-runtime-resumption/README.md`](../linux-first-runtime-resumption/README.md) | non-authoritative navigation |
| `B1-B2.1-family` | [`b1-b2-1/README.md`](../b1-b2-1/README.md) | non-authoritative navigation |
| `B3.1-C1-family` | [`b3-1-c1/README.md`](../b3-1-c1/README.md) | non-authoritative navigation |
| `a0-authority-leak-inventory` | [`slices/a0-authority-leak-inventory.md`](../slices/a0-authority-leak-inventory.md) | canonical slice row record |
| `a1-4-auto-attach-producer-adoption` | [`slices/tasks/a1-4-auto-attach-producer-adoption.md`](../slices/tasks/a1-4-auto-attach-producer-adoption.md) | canonical task row record |
| `a1-host-session-authority` | [`slices/a1-host-session-authority.md`](../slices/a1-host-session-authority.md) | canonical slice record |
| `a2-host-execution-episode-demotion` | [`slices/a2-host-execution-episode-demotion.md`](../slices/a2-host-execution-episode-demotion.md) | canonical slice row and terminal closure record |
| `a3-persistence-and-compatibility-split` | [`slices/a3-persistence-and-compatibility-split.md`](../slices/a3-persistence-and-compatibility-split.md) | canonical slice row and terminal closure record |
| `b2-2-foreground-receipt-return` | [`slices/b2-2-foreground-receipt-return.md`](../slices/b2-2-foreground-receipt-return.md) | canonical slice row record |
| `b3-2-retained-receipt-messaging-and-lifecycle` | [`slices/b3-2-retained-receipt-messaging-and-lifecycle.md`](../slices/b3-2-retained-receipt-messaging-and-lifecycle.md) | canonical slice row record |
| `b4-receipt-targeted-cancel-inspect-stop` | [`slices/b4-receipt-targeted-cancel-inspect-stop.md`](../slices/b4-receipt-targeted-cancel-inspect-stop.md) | canonical slice row record |
| `c2-inbox-and-auto-attach-projections` | [`slices/c2-inbox-and-auto-attach-projections.md`](../slices/c2-inbox-and-auto-attach-projections.md) | canonical slice row record |
| `c3-router-ownership-restoration` | [`slices/c3-router-ownership-restoration.md`](../slices/c3-router-ownership-restoration.md) | canonical slice row record |
| `configuration-and-gateway-adoption-family` | [`seams/configuration-and-gateway-adoption.md`](../seams/configuration-and-gateway-adoption.md) | canonical current seam-family record |
| `d1-world-adapter-execution-envelope` | [`slices/d1-world-adapter-execution-envelope.md`](../slices/d1-world-adapter-execution-envelope.md) | canonical slice row record |
| `d2-world-command-execution-broker` | [`slices/d2-world-command-execution-broker.md`](../slices/d2-world-command-execution-broker.md) | canonical slice row record |
| `d3-codex-uaa-end-to-end-closure` | [`slices/d3-codex-uaa-end-to-end-closure.md`](../slices/d3-codex-uaa-end-to-end-closure.md) | canonical slice row record |
| `dispatch-and-episode-transport-family` | [`seams/dispatch-and-episode-transport.md`](../seams/dispatch-and-episode-transport.md) | canonical current seam-family record |
| `e1-restricted-world-fs-narrowing` | [`slices/e1-restricted-world-fs-narrowing.md`](../slices/e1-restricted-world-fs-narrowing.md) | terminally complete slice and exact closure record |
| `e2-policy-commitments-on-work-and-workers` | [`slices/e2-policy-commitments-on-work-and-workers.md`](../slices/e2-policy-commitments-on-work-and-workers.md) | corrected E2 authority record and terminal implementation closure |
| `e3-agent-config-projection-and-gateway-adoption` | [`slices/e3-agent-config-projection-and-gateway-adoption.md`](../slices/e3-agent-config-projection-and-gateway-adoption.md) | canonical controlling specification; not admitted, dispatched, or implemented |
| `agent-config-projection-v1` | [`contracts/agent-config-projection-v1.md`](../contracts/agent-config-projection-v1.md) | canonical E3 contract specification; unadmitted |
| `managed-gateway-adoption-v1` | [`contracts/managed-gateway-adoption-v1.md`](../contracts/managed-gateway-adoption-v1.md) | canonical E3 contract specification; unadmitted |
| `e4-host-visible-write-sync-contract` | [`slices/e4-host-visible-write-sync-contract.md`](../slices/e4-host-visible-write-sync-contract.md) | canonical slice row record |
| `host-session-authority-family` | [`seams/host-session-authority.md`](../seams/host-session-authority.md) | canonical current seam-family record |
| `macOS-dev-parity-family` | [`macos-dev-parity/README.md`](../macos-dev-parity/README.md) | non-authoritative navigation |
| `obligations-and-host-re-engagement-family` | [`seams/obligations-and-host-re-engagement.md`](../seams/obligations-and-host-re-engagement.md) | canonical current seam-family record |
| `persistence-and-compatibility-family` | [`seams/persistence-and-compatibility.md`](../seams/persistence-and-compatibility.md) | canonical current seam-family record |
| `policy-and-narrowing-family` | [`seams/policy-and-narrowing.md`](../seams/policy-and-narrowing.md) | canonical current seam-family record |
| `r3-mac-evidence-recovery-family` | [`r3-mac-evidence-recovery/README.md`](../r3-mac-evidence-recovery/README.md) | non-authoritative navigation |
| `root-contracts-and-gates-compatibility-index` | [`04-contracts-and-gates.md`](../04-contracts-and-gates.md) | non-authoritative typed compatibility index |
| `root-debug-regression-compatibility-index` | [`05-debug-regression-ledger.md`](../05-debug-regression-ledger.md) | non-authoritative typed compatibility index |
| `root-phase-slice-map-compatibility-index` | [`03-phase-slice-map.md`](../03-phase-slice-map.md) | non-authoritative typed compatibility index |
| `root-seam-crosswalk-compatibility-index` | [`02-seam-crosswalk.md`](../02-seam-crosswalk.md) | non-authoritative typed compatibility index |
| `root-target-architecture-compatibility-index` | [`01-target-architecture.md`](../01-target-architecture.md) | non-authoritative typed compatibility index |
| `runtime-event-receipt-supervision-and-retained-runtime-family` | [`seams/runtime-event-receipt-supervision-and-retained-runtime.md`](../seams/runtime-event-receipt-supervision-and-retained-runtime.md) | canonical current seam-family record |
| `runtime-refactor-by-id-index` | [`index/by-id.md`](../index/by-id.md) | non-authoritative navigation |
| `runtime-refactor-by-kind-index` | [`index/by-kind.md`](../index/by-kind.md) | non-authoritative navigation |
| `runtime-refactor-by-packet-index` | [`index/by-packet.md`](../index/by-packet.md) | non-authoritative navigation |
| `runtime-refactor-control-pack-entrypoint` | [`00-README.md`](../00-README.md) | canonical stable human entrypoint |
| `runtime-refactor-current-state` | [`index/current.md`](../index/current.md) | non-authoritative current-state projection |
| `runtime-refactor-navigation-index` | [`index/README.md`](../index/README.md) | non-authoritative navigation |
| `shared-authority-map` | [`architecture/authority-map.md`](../architecture/authority-map.md) | canonical shared-architecture record |
| `shared-evidence-navigation` | [`evidence/README.md`](../evidence/README.md) | canonical extracted body with non-authoritative navigation |
| `shared-executive-target-decision` | [`architecture/executive-target.md`](../architecture/executive-target.md) | canonical shared-architecture record |
| `shared-invariant-01` | [`architecture/invariants/01-durable-session-truth-is-process-independent.md`](../architecture/invariants/01-durable-session-truth-is-process-independent.md) | canonical shared-architecture record |
| `shared-invariant-02` | [`architecture/invariants/02-private-transports-are-fast-paths.md`](../architecture/invariants/02-private-transports-are-fast-paths.md) | canonical shared-architecture record |
| `shared-invariant-03` | [`architecture/invariants/03-routing-is-exact-and-fail-closed.md`](../architecture/invariants/03-routing-is-exact-and-fail-closed.md) | canonical shared-architecture record |
| `shared-invariant-04` | [`architecture/invariants/04-long-lived-work-accepts-before-it-completes.md`](../architecture/invariants/04-long-lived-work-accepts-before-it-completes.md) | canonical shared-architecture record |
| `shared-invariant-05` | [`architecture/invariants/05-cancel-targets-active-work.md`](../architecture/invariants/05-cancel-targets-active-work.md) | canonical shared-architecture record |
| `shared-invariant-06` | [`architecture/invariants/06-obligations-are-event-derived-canonical-truth.md`](../architecture/invariants/06-obligations-are-event-derived-canonical-truth.md) | canonical shared-architecture record |
| `shared-invariant-07` | [`architecture/invariants/07-auto-attach-restores-ownership-only.md`](../architecture/invariants/07-auto-attach-restores-ownership-only.md) | canonical shared-architecture record |
| `shared-invariant-08` | [`architecture/invariants/08-world-placement-and-policy-mediation-are-separate-proofs.md`](../architecture/invariants/08-world-placement-and-policy-mediation-are-separate-proofs.md) | canonical shared-architecture record |
| `shared-invariant-09` | [`architecture/invariants/09-external-sandbox-assigns-responsibility.md`](../architecture/invariants/09-external-sandbox-assigns-responsibility.md) | canonical shared-architecture record |
| `shared-invariant-10` | [`architecture/invariants/10-dispatch-policy-only-narrows.md`](../architecture/invariants/10-dispatch-policy-only-narrows.md) | canonical shared-architecture record |
| `shared-invariant-11` | [`architecture/invariants/11-existing-world-fs-enforcement-is-the-execution-path.md`](../architecture/invariants/11-existing-world-fs-enforcement-is-the-execution-path.md) | canonical shared-architecture record |
| `shared-invariant-12` | [`architecture/invariants/12-runtime-native-configuration-is-projection.md`](../architecture/invariants/12-runtime-native-configuration-is-projection.md) | canonical shared-architecture record |
| `shared-invariant-13` | [`architecture/invariants/13-credentials-are-launch-time-gateway-handoff-not-projected-files.md`](../architecture/invariants/13-credentials-are-launch-time-gateway-handoff-not-projected-files.md) | canonical shared-architecture record |
| `shared-invariant-14` | [`architecture/invariants/14-substrate-home-is-private-per-user-authority-state.md`](../architecture/invariants/14-substrate-home-is-private-per-user-authority-state.md) | canonical shared-architecture record |
| `shared-invariants` | [`architecture/invariants/README.md`](../architecture/invariants/README.md) | non-authoritative navigation |
| `shared-review-question` | [`architecture/review-question.md`](../architecture/review-question.md) | canonical shared-architecture record |
| `shared-seam-crosswalk` | [`seams/README.md`](../seams/README.md) | canonical shared seam-crosswalk record |
| `shared-slice-map` | [`slices/README.md`](../slices/README.md) | canonical shared slice-map record |
| `shared-target-architecture` | [`architecture/README.md`](../architecture/README.md) | non-authoritative navigation |
| `track-a-authority-and-surface-neutrality` | [`slices/track-a-authority-and-surface-neutrality.md`](../slices/track-a-authority-and-surface-neutrality.md) | non-authoritative navigation; Track A terminally complete |
| `track-b-world-dispatch-receipts-supervision-and-cancel` | [`slices/track-b-world-dispatch-receipts-supervision-and-cancel.md`](../slices/track-b-world-dispatch-receipts-supervision-and-cancel.md) | non-authoritative navigation |
| `track-c-obligations-inbox-auto-attach-and-router-attach` | [`slices/track-c-obligations-inbox-auto-attach-and-router-attach.md`](../slices/track-c-obligations-inbox-auto-attach-and-router-attach.md) | non-authoritative navigation |
| `track-d-uaa-execution-envelope-and-side-effect-mediation` | [`slices/track-d-uaa-execution-envelope-and-side-effect-mediation.md`](../slices/track-d-uaa-execution-envelope-and-side-effect-mediation.md) | non-authoritative navigation |
| `track-e-dispatch-policy-and-config-projection` | [`slices/track-e-dispatch-policy-and-config-projection.md`](../slices/track-e-dispatch-policy-and-config-projection.md) | non-authoritative navigation |
| `uaa-provider-realization-and-side-effect-mediation-family` | [`seams/uaa-provider-realization-and-side-effect-mediation.md`](../seams/uaa-provider-realization-and-side-effect-mediation.md) | canonical current seam-family record |


## D12 historical owner stable IDs

| Stable ID | Kind | Canonical owner | Current status or scope |
|---|---|---|---|
| `runtime-refactor-cross-cutting-control-checkpoints-history` | historical record | [`history/cross-cutting-control-pack-checkpoints.md`](../history/cross-cutting-control-pack-checkpoints.md) | exact noncontiguous root-00 historical checkpoint bodies; never current scheduling authority |
| `runtime-refactor-authenticated-runtime-projections-history` | historical architecture record | [`history/authenticated-runtime-projections.md`](../history/authenticated-runtime-projections.md) | exact deferred R2-2E/F0a/F0b packet/history block |

## Slice and task stable IDs

| Stable ID | Canonical owner | Status |
|---|---|---|
| `a0-authority-leak-inventory` | [`slices/a0-authority-leak-inventory.md`](../slices/a0-authority-leak-inventory.md) | canonical slice row record |
| `a1-4-auto-attach-producer-adoption` | [`slices/tasks/a1-4-auto-attach-producer-adoption.md`](../slices/tasks/a1-4-auto-attach-producer-adoption.md) | canonical task row record |
| `a1-host-session-authority` | [`slices/a1-host-session-authority.md`](../slices/a1-host-session-authority.md) | canonical slice record |
| `a2-host-execution-episode-demotion` | [`slices/a2-host-execution-episode-demotion.md`](../slices/a2-host-execution-episode-demotion.md) | canonical slice row and terminal closure record |
| `a3-persistence-and-compatibility-split` | [`slices/a3-persistence-and-compatibility-split.md`](../slices/a3-persistence-and-compatibility-split.md) | canonical slice row and terminal closure record |
| `b2-2-foreground-receipt-return` | [`slices/b2-2-foreground-receipt-return.md`](../slices/b2-2-foreground-receipt-return.md) | canonical slice row record |
| `b3-2-retained-receipt-messaging-and-lifecycle` | [`slices/b3-2-retained-receipt-messaging-and-lifecycle.md`](../slices/b3-2-retained-receipt-messaging-and-lifecycle.md) | canonical slice row record |
| `b4-receipt-targeted-cancel-inspect-stop` | [`slices/b4-receipt-targeted-cancel-inspect-stop.md`](../slices/b4-receipt-targeted-cancel-inspect-stop.md) | canonical slice row record |
| `c2-inbox-and-auto-attach-projections` | [`slices/c2-inbox-and-auto-attach-projections.md`](../slices/c2-inbox-and-auto-attach-projections.md) | canonical slice row record |
| `c3-router-ownership-restoration` | [`slices/c3-router-ownership-restoration.md`](../slices/c3-router-ownership-restoration.md) | canonical slice row record |
| `d1-world-adapter-execution-envelope` | [`slices/d1-world-adapter-execution-envelope.md`](../slices/d1-world-adapter-execution-envelope.md) | canonical slice row record |
| `d2-world-command-execution-broker` | [`slices/d2-world-command-execution-broker.md`](../slices/d2-world-command-execution-broker.md) | canonical slice row record |
| `d3-codex-uaa-end-to-end-closure` | [`slices/d3-codex-uaa-end-to-end-closure.md`](../slices/d3-codex-uaa-end-to-end-closure.md) | canonical slice row record |
| `e1-restricted-world-fs-narrowing` | [`slices/e1-restricted-world-fs-narrowing.md`](../slices/e1-restricted-world-fs-narrowing.md) | terminally complete slice and exact closure record |
| `e2-policy-commitments-on-work-and-workers` | [`slices/e2-policy-commitments-on-work-and-workers.md`](../slices/e2-policy-commitments-on-work-and-workers.md) | corrected E2 authority record and terminal implementation closure |
| `e3-agent-config-projection-and-gateway-adoption` | [`slices/e3-agent-config-projection-and-gateway-adoption.md`](../slices/e3-agent-config-projection-and-gateway-adoption.md) | canonical controlling specification; not admitted, dispatched, or implemented |
| `e4-host-visible-write-sync-contract` | [`slices/e4-host-visible-write-sync-contract.md`](../slices/e4-host-visible-write-sync-contract.md) | canonical slice row record |
| `shared-slice-map` | [`slices/README.md`](../slices/README.md) | canonical shared slice-map record |
| `track-a-authority-and-surface-neutrality` | [`slices/track-a-authority-and-surface-neutrality.md`](../slices/track-a-authority-and-surface-neutrality.md) | non-authoritative navigation; Track A terminally complete |
| `track-b-world-dispatch-receipts-supervision-and-cancel` | [`slices/track-b-world-dispatch-receipts-supervision-and-cancel.md`](../slices/track-b-world-dispatch-receipts-supervision-and-cancel.md) | non-authoritative navigation |
| `track-c-obligations-inbox-auto-attach-and-router-attach` | [`slices/track-c-obligations-inbox-auto-attach-and-router-attach.md`](../slices/track-c-obligations-inbox-auto-attach-and-router-attach.md) | non-authoritative navigation |
| `track-d-uaa-execution-envelope-and-side-effect-mediation` | [`slices/track-d-uaa-execution-envelope-and-side-effect-mediation.md`](../slices/track-d-uaa-execution-envelope-and-side-effect-mediation.md) | non-authoritative navigation |
| `track-e-dispatch-policy-and-config-projection` | [`slices/track-e-dispatch-policy-and-config-projection.md`](../slices/track-e-dispatch-policy-and-config-projection.md) | non-authoritative navigation |

## Seam-family names

| Family name | Canonical owner |
|---|---|
| `Host/session authority` | [`seams/host-session-authority.md`](../seams/host-session-authority.md) |
| `Persistence and compatibility projection` | [`seams/persistence-and-compatibility.md`](../seams/persistence-and-compatibility.md) |
| `Dispatch and episode transport` | [`seams/dispatch-and-episode-transport.md`](../seams/dispatch-and-episode-transport.md) |
| `Policy and narrowing` | [`seams/policy-and-narrowing.md`](../seams/policy-and-narrowing.md) |
| `Runtime event, receipt, supervision, and retained runtime` | [`seams/runtime-event-receipt-supervision-and-retained-runtime.md`](../seams/runtime-event-receipt-supervision-and-retained-runtime.md) |
| `Obligations and host re-engagement` | [`seams/obligations-and-host-re-engagement.md`](../seams/obligations-and-host-re-engagement.md) |
| `Configuration and gateway adoption` | [`seams/configuration-and-gateway-adoption.md`](../seams/configuration-and-gateway-adoption.md) |
| `UAA/provider realization and side-effect mediation` | [`seams/uaa-provider-realization-and-side-effect-mediation.md`](../seams/uaa-provider-realization-and-side-effect-mediation.md) |

## Seam names

| Seam name | Canonical owner |
|---|---|
| `SurfaceAdapter / HostExecutionEpisode` | [`seams/host-session-authority.md`](../seams/host-session-authority.md) |
| `HostSessionAuthority` | [`a1-2-earlier-histories/crosswalk.md#hostsessionauthority-family-row`](../a1-2-earlier-histories/crosswalk.md#hostsessionauthority-family-row) |
| `StateStore` | [`seams/persistence-and-compatibility.md`](../seams/persistence-and-compatibility.md) |
| `CompatibilityReadModel` | [`seams/persistence-and-compatibility.md`](../seams/persistence-and-compatibility.md) |
| `InternalToolboxTransport` | [`seams/dispatch-and-episode-transport.md`](../seams/dispatch-and-episode-transport.md) |
| `RuntimeToolInvocationAdapter` | [`seams/dispatch-and-episode-transport.md`](../seams/dispatch-and-episode-transport.md) |
| `WorldDispatchControl` | [`seams/dispatch-and-episode-transport.md`](../seams/dispatch-and-episode-transport.md) |
| `SteeringPolicyEngine` | [`seams/policy-and-narrowing.md`](../seams/policy-and-narrowing.md) |
| `EffectivePolicyResolver` | [`seams/policy-and-narrowing.md`](../seams/policy-and-narrowing.md) |
| `DispatchPolicyNarrowingPatch` | [`seams/policy-and-narrowing.md`](../seams/policy-and-narrowing.md) |
| `RuntimeEventTransport` | [`seams/runtime-event-receipt-supervision-and-retained-runtime.md`](../seams/runtime-event-receipt-supervision-and-retained-runtime.md) |
| `WorldWorkReceiptRegistry` | [`seams/runtime-event-receipt-supervision-and-retained-runtime.md`](../seams/runtime-event-receipt-supervision-and-retained-runtime.md) |
| `WorldWorkExecutionSupervisor` | [`seams/runtime-event-receipt-supervision-and-retained-runtime.md`](../seams/runtime-event-receipt-supervision-and-retained-runtime.md) |
| `WorldWorkerMessagingProtocol` | [`b3-1-c1/crosswalk.md#worldworkermessagingprotocol`](../b3-1-c1/crosswalk.md#worldworkermessagingprotocol) |
| `RetainedWorkerRuntime` | [`seams/runtime-event-receipt-supervision-and-retained-runtime.md`](../seams/runtime-event-receipt-supervision-and-retained-runtime.md) |
| `ObligationLedger` | [`b3-1-c1/crosswalk.md#obligationledger`](../b3-1-c1/crosswalk.md#obligationledger) |
| `InboxProjection` | [`seams/obligations-and-host-re-engagement.md`](../seams/obligations-and-host-re-engagement.md) |
| `AutoAttachProjection` | [`seams/obligations-and-host-re-engagement.md`](../seams/obligations-and-host-re-engagement.md) |
| `RouterAttachTrigger` | [`seams/obligations-and-host-re-engagement.md`](../seams/obligations-and-host-re-engagement.md) |
| `AgentConfigProjectionService` | [`seams/configuration-and-gateway-adoption.md`](../seams/configuration-and-gateway-adoption.md) |
| `WorldRuntimeAdapterExecutionEnvelope` | [`seams/configuration-and-gateway-adoption.md`](../seams/configuration-and-gateway-adoption.md) |
| `WorldCommandExecutionBroker` | [`seams/uaa-provider-realization-and-side-effect-mediation.md`](../seams/uaa-provider-realization-and-side-effect-mediation.md) |
| `RuntimeFamilyRealizationAdapter` | [`seams/uaa-provider-realization-and-side-effect-mediation.md`](../seams/uaa-provider-realization-and-side-effect-mediation.md) |

## Issue IDs

| Issue ID | Canonical owner |
|---|---|
| `RG-ADMISSION-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-ATTACH-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-ATTACH-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-AUTH-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-AUTH-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-AUTH-03` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-BASE-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-BASE-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-BASE-03` | [`evidence/baseline-behaviors-and-smoke-scenarios.md#a3-scoped-rg-base-03-disposition`](../evidence/baseline-behaviors-and-smoke-scenarios.md#a3-scoped-rg-base-03-disposition) |
| `RG-BASE-04` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-CANCEL-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-CANCEL-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-CLOSE-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-CONFIG-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-CONFIG-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-CONFIG-03` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-CONFIG-04` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-DIFF-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-EVENT-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-HOME-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-INSTALL-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-MSG-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-OBL-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-OBL-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-OBS-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-POLICY-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-POLICY-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-POLICY-03` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-RECEIPT-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-RECEIPT-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-RECEIPT-03` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-RECEIPT-04` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-SUP-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-SUP-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-SYNC-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-UAA-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-UAA-02` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-UAA-03` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-WORKER-EXIT-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |
| `RG-WORLD-ADOPT-01` | [`evidence/canonical-issue-ledger.md`](../evidence/canonical-issue-ledger.md) |

## Review finding IDs

| Review finding ID | Canonical owner |
|---|---|
| `RR-RF-0001` | [`06-review-finding-inventory.md`](../06-review-finding-inventory.md) |
| `RR-RF-0002` | [`06-review-finding-inventory.md`](../06-review-finding-inventory.md) |
| `RR-RF-0003` | [`06-review-finding-inventory.md`](../06-review-finding-inventory.md) |
| `RR-RF-0004` | [`06-review-finding-inventory.md`](../06-review-finding-inventory.md) |
| `RR-RF-0005` | [`06-review-finding-inventory.md`](../06-review-finding-inventory.md) |
