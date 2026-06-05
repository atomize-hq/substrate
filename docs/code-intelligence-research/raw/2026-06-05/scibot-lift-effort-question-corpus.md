Status: exploratory
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: none

# Sci-Bot Lift/Effort Question Corpus

This note preserves the five Sci-Bot question pages and their cited paper sets
so the reference corpus does not live only in the temporary repo-root ledger.

## Questions and direct links

1. [What input fields are required for deterministic software task planning from repository analysis?](https://sci-bot.ru/what-input-fields-are-required-aae0)
2. [How should dependency type, confidence, and blocking risk be represented in an agent planning graph?](https://sci-bot.ru/how-should-dependency-type-confidence-b76b)
3. [What planner inputs best predict safe parallelization versus merge or coordination conflicts?](https://sci-bot.ru/what-planner-inputs-best-predict-a8f7)
4. [How should historical execution data be encoded so a planner can improve decisions without becoming nondeterministic?](https://sci-bot.ru/how-should-historical-execution-data-ac8b)
5. [What artifact schema best bridges code intelligence outputs into lane-based execution plans and handoff packets?](https://sci-bot.ru/what-artifact-schema-best-bridges-b849)

## 1. Deterministic software task planning inputs

### Durable takeaways

- deterministic planning needs multiple structured input categories, not only source structure
- useful input families span project sizing, technical/environment, team/organization, issue, repository history, low-level activity, and architecture-evolution layers
- the strongest repo-local implication is that planning quality depends on structured upstream inputs across repo, task, and architecture surfaces

### Referenced papers

- [Selecting best predictors from large software repositories for highly accurate software effort estimation](https://doi.org/10.1002/smr.2271)
- [Estimating Story Points from Issue Reports](https://doi.org/10.1145/2972958.2972959)
- [A Retrospective Study of Software Analytics Projects: In-Depth Interviews with Practitioners](https://doi.org/10.1109/ms.2013.93)
- [Recommendation system to enhance planning of software development using r](https://doi.org/10.1145/2593822.2593831)
- [Mining Software Repositories to Assist Developers and Support Managers](https://doi.org/10.1109/icsm.2006.38)
- [An analysis of developers' tasks using low-level, automatically collected data](https://doi.org/10.1145/1287624.1287715)
- [An approach to software development effort estimation using machine learning](https://doi.org/10.1109/iccp.2017.8117004)
- [Software evolutionary architecture: Automated planning for functional changes](https://doi.org/10.1016/j.scico.2023.102978)
- [Evaluating Pred(p) and standardized accuracy criteria in software development effort estimation](https://doi.org/10.1002/smr.1925)

## 2. Dependency type, confidence, and blocking risk

### Durable takeaways

- dependency type should not be flattened into one generic edge
- different dependency families imply different graph constructs:
  causal, resource, interaction, and precedence
- confidence should attach explicitly to actions, nodes, or graph decisions
- blocking risk should be explicit conflict or synchronization structure, not an informal afterthought

### Referenced papers

- [Representing and planning with interacting actions and privacy](https://doi.org/10.1016/j.artint.2019.103200)
- [Explaining interdependent action delays in multiagent plans execution](https://doi.org/10.1007/s10458-015-9298-0)
- [A Graph-Based Multi-Agent Planning Algorithm with QoS Guarantees](https://doi.org/10.1109/iat.2007.87)
- [Rationale in planning: causality, dependencies, and decisions](https://doi.org/10.1017/s026988899800201x)
- [A Collaborative Multiagent Framework Based on Online Risk-Aware Planning and Decision-Making](https://doi.org/10.1109/ictai.2016.0015)
- [Multiagent Conflict Resolution Planning](https://doi.org/10.1109/smc.2013.57)
- [Conflict estimation of abstract plans for multiagent systems](https://doi.org/10.1145/1329125.1329279)
- [A privacy-preserving model for multi-agent propositional planning](https://doi.org/10.1080/0952813x.2018.1456786)

## 3. Inputs that predict safe parallelization

### Durable takeaways

- shared-resource footprints are the strongest conflict predictor
- explicit concurrency constraints matter more than broad optimism about independent work
- cross-plan causal links predict coupling and coordination cost
- safe parallelization should be derived from explicit resource, dependency, and conflict reasoning

### Referenced papers

- [Solving Multiagent Planning Problems with Concurrent Conditional Effects](https://doi.org/10.1609/aaai.v33i01.33017594)
- [Theory and algorithms for plan merging](https://doi.org/10.1016/0004-3702(92)90016-q)
- [Conflict solving into the multi-agent distributed planning](https://doi.org/10.1109/icsmc.1998.728083)
- [Predicting possible conflicts in hierarchical planning for multi-agent systems](https://doi.org/10.1145/1082473.1082597)
- [An efficient algorithm for multiagent plan coordination](https://doi.org/10.1145/1082473.1082599)
- [How to solve deadlock situations within the plan-merging paradigm for multi-robot cooperation](https://doi.org/10.1109/iros.1997.656573)
- [Parallel Performance Problems on Shared-Memory Multicore Systems: Taxonomy and Observation](https://doi.org/10.1109/tse.2016.2519346)
- [A satisficing multiagent plan coordinating algorithm for dynamic domains](https://doi.org/10.1145/375735.376032)
- [An extension of the plan-merging paradigm for multi-robot coordination](https://doi.org/10.1109/robot.2001.933066)

## 4. Encoding historical execution data

### Durable takeaways

- raw history should be turned into deterministic planner artifacts, not stochastic online behavior
- useful encoding families include fragility costs, deterministic case retrieval, learned control rules, macro-operators, nogoods, CSP-learned action models, and planner-performance feature vectors
- the repo-local implication is that historical evidence should become costs, constraints, features, causal edges, or explicit hints

### Referenced papers

- [Integrating Planning, Execution, and Learning to Improve Plan Execution](https://doi.org/10.1111/j.1467-8640.2012.00447.x)
- [Case-based planning](https://doi.org/10.1017/s0269888906000592)
- [Identifying and Exploiting Features for Effective Plan Retrieval in Case-Based Planning](https://doi.org/10.3233/fi-2016-1447)
- [Knowledge Transfer between Automated Planners](https://doi.org/10.1609/aimag.v32i2.2334)
- [Using Data Mining to Enhance Automated Planning and Scheduling](https://doi.org/10.1109/cidm.2007.368881)
- [Learning temporal action models from multiple plans: A constraint satisfaction approach](https://doi.org/10.1016/j.engappai.2021.104590)
- [Learning from planner performance](https://doi.org/10.1016/j.artint.2008.11.009)
- [Extending BDI plan selection to incorporate learning from experience](https://doi.org/10.1016/j.robot.2010.05.008)

## 5. Artifact schema bridging into lane plans and handoff packets

### Durable takeaways

- the strongest schema pattern is explicit-external rather than implicit-internal
- a multi-layer representation is better than one flat artifact
- source-based mappings and artifact-centric process models are strong bridging techniques

### Referenced papers

- [Where's the schema? A taxonomy of patterns for software exchange](https://doi.org/10.1109/wpc.2002.1021320)
- [Issues in Integrating Schemas for Reverse Engineering](https://doi.org/10.1016/j.entcs.2004.01.002)
- [Analyzing Inaccurate Artifact Usages in a Workflow Schema](https://doi.org/10.1109/compsac.2006.113)
- [An intermediate representation for integrating reverse engineering analyses](https://doi.org/10.1109/wcre.1998.723194)
- [An XML-based framework for language neutral program representation and generic analysis](https://doi.org/10.1109/cmpsac.2004.1342654)
- [Linking Analysis and Transformation Tools with Source-Based Mappings](https://doi.org/10.1109/scam.2006.18)
- [A Programmable Analysis and Transformation Framework for Reverse Engineering](https://doi.org/10.1016/j.entcs.2004.01.005)
- [Dataflow plan execution for software agents](https://doi.org/10.1145/336595.337087)
- [Tool Integration Models](https://doi.org/10.1109/apsec.2013.70)
- [Package-oriented programming of engineering tools](https://doi.org/10.1145/253228.253501)
- [Tools cooperation in an integration environment by message-passing mechanism](https://doi.org/10.1109/cmpsac.1994.342763)
- [Towards an Integration System for Artifact-centric Processes](https://doi.org/10.1145/2926693.2929904)
- [Bridging the Concrete and Logical Domains for Software Architecture Reconstruction](https://doi.org/10.1109/wicsa.2005.18)
- [Integration Workbench: Integrating Schema Integration Tools](https://doi.org/10.1109/icdew.2006.69)
- [Integration and Analysis of Design Artefacts in Embedded Software Development](https://doi.org/10.1109/compsacw.2012.94)

## Boundary caution

This note preserves the question corpus and citations.
Use overlay and crate notes when you want interpretation or repo-local design
implications.
