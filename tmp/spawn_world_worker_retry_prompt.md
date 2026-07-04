In this one turn, do not explain tool availability.

You are already inside a host orchestrator session that is bound to an orchestration session/world. The callable interface for this task is the Substrate runtime-injected internal toolbox contract for this session. It is not expected to appear as a harness-listed tool and it is not expected to be discovered via tool_search. The runtime details for this toolbox are injected into your session instructions/environment.

Your job:
1. Use the runtime-injected Substrate internal toolbox action `spawn_world_worker` exactly once.
2. Use backend `cli:codex-world`.
3. Use retained mode.
4. Use the current orchestration session/world binding.
5. Give the worker a trivial prompt.
6. Do not use any other Substrate toolbox action.
7. Do not use run_world_task, fork_world_worker, continue_world_worker, inspect_world_worker, stop_world_worker, or cancel_world_work.
8. Do not answer with a refusal or a description of tools.
9. Return only the exact spawned worker `participant_id` and nothing else.

If you think the toolbox is unavailable, first rely on the injected Substrate toolbox contract for this session rather than the harness tool list, and then proceed with the single `spawn_world_worker` call.
