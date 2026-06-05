Status: exploratory
Scope: exec
Authority: non-canonical
Artifact-boundary impact: possible

# Exec Open Questions

These questions remain intentionally open and should not be treated as settled by note structure alone.

## Open questions

1. What is the smallest useful blocked-condition artifact family for MVP?
2. Which runtime evidence belongs directly in `ExecutionStateV1` versus referenced through other artifacts?
3. When should speculative validation run relative to the materialization checkpoint?
4. How should runtime record replan-required states without silently mutating the original plan?
5. How far should this runtime model converge with broader Substrate runtime concepts, if at all?

## Keep provisional

These questions should be promoted only when implementation pressure or repeated operator patterns require a canonical answer.
