Status: exploratory
Scope: lift
Authority: non-canonical
Artifact-boundary impact: possible

# Lift Open Questions

These questions remain intentionally open and should not be treated as settled
by note structure alone.

## Open questions

1. What is the smallest useful freshness and provenance surface that Lift
   should guarantee for MVP exports?
2. Which changed-scope distinctions are mandatory for MVP:
   semantic change, structural refactor, formatter-only churn, production-only,
   test-only, or some smaller subset?
3. How much confidence detail should Lift emit directly versus leaving to
   downstream interpretation?
4. Which execution-time consumers need direct access to Lift-derived structural
   evidence versus summarized planner output?
5. How far should Lift go in surfacing candidate conflict or freeze hints
   before it starts leaking planning semantics?

## Keep provisional

These questions should be promoted only when repeated implementation pressure or
repeated workflow failures require a canonical answer.
