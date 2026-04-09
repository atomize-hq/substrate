import json
import os
import shutil
import subprocess
import unittest
from pathlib import Path


BEGIN = "<!-- PM_PWS_INDEX:BEGIN -->"
END = "<!-- PM_PWS_INDEX:END -->"


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _triage_text(*, heading_tokens: list[str]) -> str:
    pws = [
        {
            "id": "WDRA-PWS-contract",
            "role": "contract",
            "depends_on": [],
            "assumes": [],
            "owns": ["contract.md"],
        },
        {
            "id": "WDRA-PWS-tasks_checkpoints",
            "role": "tasks_checkpoints",
            "depends_on": ["WDRA-PWS-contract"],
            "assumes": [],
            "owns": [
                "tasks.json",
                "session_log.md",
                "kickoff_prompts/",
                "slices/WDRA0/kickoff_prompts/",
            ],
        },
    ]
    headings = []
    for p, heading_token in zip(pws, heading_tokens):
        headings.append(f"### {heading_token} — {p['role']}\n\n- Goal: fixture\n")
    idx = {
        "pws_index_version": 2,
        "slice_prefix": "WDRA",
        "accepted_slice_order": ["WDRA0"],
        "pws": pws,
    }
    return (
        "# Workstream triage fixture\n\n"
        + "\n".join(headings)
        + "\n"
        + f"{BEGIN}\n```json\n{json.dumps(idx, indent=2, sort_keys=False)}\n```\n{END}\n"
    )


def _checkpoint_plan_text() -> str:
    return "# Fixture checkpoint plan\n\n## Machine-readable plan (draft)\n\n```json\n{\"version\":1}\n```\n"


def _impact_map_text(*, create: list[str] | None = None, edit: list[str] | None = None) -> str:
    def _section(tokens: list[str] | None) -> str:
        if not tokens:
            return "- None"
        return "\n".join(f"- `{token}`" for token in tokens)

    return (
        "# Impact Map Fixture\n\n"
        "## Touch set (explicit)\n\n"
        "### Create\n"
        f"{_section(create)}\n\n"
        "### Edit\n"
        f"{_section(edit)}\n\n"
        "### Deprecate\n"
        "- None\n\n"
        "### Delete\n"
        "- None\n"
    )


class TestRunPlanningAgent(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / ".tmp_pm_script_tests" / "run_planning_agent"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.script = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "run_planning_agent.sh"
        )
        if not cls.script.is_file():
            raise unittest.SkipTest("run_planning_agent.sh not found at expected canonical path.")

    def _write_codex_stub(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail

output_last=""
emit_json=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-last-message)
      output_last="$2"
      shift 2
      ;;
    --json)
      emit_json=1
      shift
      ;;
    --cd|--config|--profile|--model)
      shift 2
      ;;
    exec|--dangerously-bypass-approvals-and-sandbox|-)
      shift
      ;;
    *)
      shift
      ;;
  esac
done

cat >/dev/null
write_mode="${TEST_RUNNER_WRITE_MODE:-staged}"

if [[ "${emit_json}" == "1" ]]; then
  printf '%s\n' '{"type":"thread.started","thread_id":"thread-test-123"}'
  printf '%s\n' '{"type":"turn.started"}'
  printf '%s\n' '{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"runner stub summary"}}'
  if [[ "${TEST_RUNNER_EMIT_TOOL_FAILURE:-0}" == "1" ]]; then
    printf '%s\n' '{"type":"item.completed","item":{"id":"item_2","type":"command_execution","command":"false","aggregated_output":"fixture failure","exit_code":1,"status":"failed"}}'
  fi
  printf '%s\n' '{"type":"turn.completed","usage":{"input_tokens":1,"cached_input_tokens":0,"output_tokens":1}}'
fi

if [[ -n "${TEST_RUNNER_PRIMARY_DEST:-}" ]]; then
  mkdir -p "$(dirname "${TEST_RUNNER_PRIMARY_DEST}")"
  cp "${TEST_RUNNER_PRIMARY_SOURCE}" "${TEST_RUNNER_PRIMARY_DEST}"
fi

if [[ "${write_mode}" == "staged_with_tasks" && -n "${TEST_RUNNER_SECONDARY_DEST:-}" ]]; then
  mkdir -p "$(dirname "${TEST_RUNNER_SECONDARY_DEST}")"
  cp "${TEST_RUNNER_SECONDARY_SOURCE}" "${TEST_RUNNER_SECONDARY_DEST}"
fi

printf 'runner stub summary\n' >"${output_last}"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_git_wrapper(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "ls-files" && "${2:-}" == "--others" ]]; then
  if [[ -n "${TEST_GIT_LS_FILES:-}" ]]; then
    printf '%s\n' "${TEST_GIT_LS_FILES}"
  fi
  exit 0
fi
if [[ "${1:-}" == "diff" && "${2:-}" == "--name-only" ]]; then
  exit 0
fi
exec "${REAL_GIT}" "$@"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _make_feature_dir(self, name: str) -> tuple[Path, Path]:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)
        feature_dir.mkdir(parents=True, exist_ok=True)

        adr_path = feature_dir / "ADR-0001-fixture.md"
        _write_text(adr_path, "# ADR fixture\n")
        _write_text(
            feature_dir / "tasks.json",
            json.dumps(
                {
                    "meta": {
                        "schema_version": 4,
                        "slice_spec_version": 2,
                        "adr_paths": [str(adr_path)],
                        "cross_platform": True,
                        "automation": {"enabled": True, "orchestration_branch": "feat/test"},
                    },
                    "tasks": [],
                },
                indent=2,
            ),
        )
        return feature_dir, adr_path

    def _step_dir_name(self, agent: str) -> str:
        return {
            "spec_manifest": "spec-manifest",
            "impact_map": "impact-map",
            "min_spec_draft": "min-spec-draft",
            "ci_checkpoint": "CI-checkpoint",
            "workstream_triage": "workstream-triage",
            "pre_planning_slice_reconcile": "pre-full-planning-convergence",
            "post_full_planning_reconcile": "post-full-planning-convergence",
        }[agent]

    def _latest_run_dir(self, feature_dir: Path, agent: str) -> Path:
        runs_dir = feature_dir / "logs" / self._step_dir_name(agent) / "runs"
        runs = sorted(path for path in runs_dir.iterdir() if path.is_dir())
        self.assertTrue(runs, msg=f"expected at least one run dir under {runs_dir}")
        return runs[-1]

    def _run(
        self,
        feature_dir: Path,
        *,
        agent: str,
        primary_source: Path,
        primary_dest_rel: str,
        secondary_source: Path | None = None,
        secondary_dest_rel: str | None = None,
        write_mode: str = "staged",
        orchestrated: bool = False,
        phase: str = "single",
        emit_tool_failure: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        bin_dir = feature_dir / "bin"
        codex_path = bin_dir / "codex"
        git_path = bin_dir / "git"
        self._write_codex_stub(codex_path)
        self._write_git_wrapper(git_path)

        env = os.environ.copy()
        env["PATH"] = f"{bin_dir}:{env['PATH']}"
        env["REAL_GIT"] = subprocess.check_output(["which", "git"], text=True, cwd=str(self.repo_root)).strip()
        env["TEST_RUNNER_PRIMARY_SOURCE"] = str(primary_source)
        env["TEST_RUNNER_PRIMARY_DEST"] = str(feature_dir / primary_dest_rel)
        env["TEST_RUNNER_WRITE_MODE"] = write_mode
        ls_files = [str(feature_dir / primary_dest_rel)]
        if secondary_dest_rel is not None:
            ls_files.append(str(feature_dir / secondary_dest_rel))
        env["TEST_GIT_LS_FILES"] = "\n".join(Path(p).relative_to(self.repo_root).as_posix() for p in ls_files)
        if secondary_source is not None and secondary_dest_rel is not None:
            env["TEST_RUNNER_SECONDARY_SOURCE"] = str(secondary_source)
            env["TEST_RUNNER_SECONDARY_DEST"] = str(feature_dir / secondary_dest_rel)
        if orchestrated:
            env["PM_PLANNING_ORCHESTRATED"] = "1"
        if emit_tool_failure:
            env["TEST_RUNNER_EMIT_TOOL_FAILURE"] = "1"

        cmd = ["bash", str(self.script), "--feature-dir", str(feature_dir), "--agent", agent]
        if phase != "single":
            cmd.extend(["--phase", phase])

        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )

    def test_workstream_triage_runner_accepts_staged_backticked_headings(self) -> None:
        feature_dir, _ = self._make_feature_dir("pass_staged_backticked_headings")
        artifact_source = feature_dir / "backticked_workstream_triage.md"
        _write_text(
            artifact_source,
            _triage_text(heading_tokens=["`WDRA-PWS-contract`", "`WDRA-PWS-tasks_checkpoints`"]),
        )

        res = self._run(
            feature_dir,
            agent="workstream_triage",
            primary_source=artifact_source,
            primary_dest_rel="logs/workstream-triage/staged/pre-planning/workstream_triage.md",
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue((feature_dir / "pre-planning" / "workstream_triage.md").is_file())
        self.assertIn("OK: tracked outputs within allowlist", res.stdout)

    def test_workstream_triage_runner_fails_true_heading_json_mismatch(self) -> None:
        feature_dir, _ = self._make_feature_dir("fail_heading_json_mismatch")
        artifact_source = feature_dir / "mismatch_workstream_triage.md"
        _write_text(
            artifact_source,
            _triage_text(heading_tokens=["`WDRA-PWS-contract_typo`", "`WDRA-PWS-tasks_checkpoints`"]),
        )

        res = self._run(
            feature_dir,
            agent="workstream_triage",
            primary_source=artifact_source,
            primary_dest_rel="logs/workstream-triage/staged/pre-planning/workstream_triage.md",
        )
        self.assertEqual(res.returncode, 1)
        self.assertIn("workstream_triage closeout validation failed", res.stderr)
        self.assertIn("normalized to 'WDRA-PWS-contract_typo'", res.stderr)

    def test_workstream_triage_runner_orchestrated_accepts_staged_candidate(self) -> None:
        feature_dir, _ = self._make_feature_dir("orchestrated_staged_candidate")
        artifact_source = feature_dir / "orchestrated_workstream_triage.md"
        _write_text(
            artifact_source,
            _triage_text(heading_tokens=["WDRA-PWS-contract", "WDRA-PWS-tasks_checkpoints"]),
        )

        res = self._run(
            feature_dir,
            agent="workstream_triage",
            primary_source=artifact_source,
            primary_dest_rel="logs/workstream-triage/staged/pre-planning/workstream_triage.md",
            orchestrated=True,
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertFalse((feature_dir / "pre-planning" / "workstream_triage.md").exists())
        self.assertFalse((feature_dir / "logs" / "workstream-triage" / "last_message.md").exists())
        self.assertIn("OK: staged outputs within allowlist", res.stdout)

        run_dir = self._latest_run_dir(feature_dir, "workstream_triage")
        run_state = json.loads((run_dir / "run_state.json").read_text(encoding="utf-8"))
        self.assertEqual(run_state["phase"], "single")
        self.assertEqual(run_state["agent"], "workstream_triage")
        self.assertEqual(run_state["exit_code"], 0)
        self.assertTrue(run_state["turn_completed"])
        self.assertTrue(run_state["assistant_message_present"])
        self.assertEqual(run_state["tool_error_count"], 0)
        self.assertEqual(run_state["thread_id"], "thread-test-123")
        self.assertEqual(
            run_state["events_path"],
            str((run_dir / "events.jsonl").relative_to(self.repo_root)).replace(os.sep, "/"),
        )
        self.assertEqual(
            run_state["last_message_run_path"],
            str((run_dir / "last_message.run.md").relative_to(self.repo_root)).replace(os.sep, "/"),
        )

    def test_workstream_triage_runner_orchestrated_rejects_direct_canonical_write(self) -> None:
        feature_dir, _ = self._make_feature_dir("orchestrated_direct_canonical_write")
        artifact_source = feature_dir / "direct_workstream_triage.md"
        _write_text(
            artifact_source,
            _triage_text(heading_tokens=["WDRA-PWS-contract", "WDRA-PWS-tasks_checkpoints"]),
        )

        res = self._run(
            feature_dir,
            agent="workstream_triage",
            primary_source=artifact_source,
            primary_dest_rel="pre-planning/workstream_triage.md",
            write_mode="canonical",
            orchestrated=True,
        )
        self.assertEqual(res.returncode, 2)
        self.assertIn("unexpected untracked", res.stderr)
        self.assertIn("pre-planning/workstream_triage.md", res.stderr)

    def test_ci_checkpoint_runner_accepts_staged_plan_only(self) -> None:
        feature_dir, _ = self._make_feature_dir("ci_checkpoint_plan_only")
        plan_source = feature_dir / "ci_checkpoint_plan.md"
        _write_text(plan_source, _checkpoint_plan_text())

        res = self._run(
            feature_dir,
            agent="ci_checkpoint",
            primary_source=plan_source,
            primary_dest_rel="logs/CI-checkpoint/staged/pre-planning/ci_checkpoint_plan.md",
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue((feature_dir / "pre-planning" / "ci_checkpoint_plan.md").is_file())

    def test_ci_checkpoint_runner_accepts_staged_plan_and_tasks_json(self) -> None:
        feature_dir, _ = self._make_feature_dir("ci_checkpoint_plan_and_tasks")
        plan_source = feature_dir / "ci_checkpoint_plan.md"
        tasks_source = feature_dir / "tasks.updated.json"
        _write_text(plan_source, _checkpoint_plan_text())
        _write_text(
            tasks_source,
            json.dumps(
                {
                    "meta": {
                        "schema_version": 4,
                        "slice_spec_version": 2,
                        "cross_platform": True,
                        "automation": {"enabled": True, "orchestration_branch": "feat/test"},
                        "checkpoint_boundaries": ["WDRA0"],
                    },
                    "tasks": [],
                },
                indent=2,
            ),
        )

        res = self._run(
            feature_dir,
            agent="ci_checkpoint",
            primary_source=plan_source,
            primary_dest_rel="logs/CI-checkpoint/staged/pre-planning/ci_checkpoint_plan.md",
            secondary_source=tasks_source,
            secondary_dest_rel="logs/CI-checkpoint/staged/tasks.json",
            write_mode="staged_with_tasks",
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue((feature_dir / "pre-planning" / "ci_checkpoint_plan.md").is_file())
        tasks = json.loads((feature_dir / "tasks.json").read_text(encoding="utf-8"))
        self.assertEqual(tasks["meta"]["checkpoint_boundaries"], ["WDRA0"])

    def test_impact_map_phase_a_requires_logs_only_outputs(self) -> None:
        feature_dir, _ = self._make_feature_dir("impact_map_phase_a_logs_only")
        scratch_source = feature_dir / "impact_map.scratch.md"
        handoff_source = feature_dir / "impact_map.handoff.md"
        _write_text(scratch_source, "# Scratch\n")
        _write_text(handoff_source, "# Handoff\n")

        res = self._run(
            feature_dir,
            agent="impact_map",
            primary_source=scratch_source,
            primary_dest_rel="logs/impact-map/scratch.md",
            secondary_source=handoff_source,
            secondary_dest_rel="logs/impact-map/handoff.md",
            write_mode="staged_with_tasks",
            phase="phase_a",
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertFalse((feature_dir / "pre-planning" / "impact_map.md").exists())
        self.assertFalse((feature_dir / "logs" / "impact-map" / "last_message.md").exists())
        self.assertIn("OK: phase_a log outputs within allowlist", res.stdout)

        run_dir = self._latest_run_dir(feature_dir, "impact_map")
        self.assertTrue((run_dir / "events.jsonl").is_file())
        run_state = json.loads((run_dir / "run_state.json").read_text(encoding="utf-8"))
        self.assertEqual(run_state["phase"], "phase_a")
        self.assertEqual(run_state["tool_error_count"], 0)
        prompt_text = (run_dir / "prompt.md").read_text(encoding="utf-8")
        self.assertIn("Closeout micro-lint bans ambiguous modal wording", prompt_text)
        self.assertIn("Do not use `should`, `could`, `might`, or `maybe`", prompt_text)

    def test_impact_map_phase_b_rejects_logs_only_output(self) -> None:
        feature_dir, _ = self._make_feature_dir("impact_map_phase_b_requires_staged")
        handoff_source = feature_dir / "impact_map.handoff.md"
        _write_text(handoff_source, "# Handoff\n")

        res = self._run(
            feature_dir,
            agent="impact_map",
            primary_source=handoff_source,
            primary_dest_rel="logs/impact-map/handoff.md",
            phase="phase_b",
        )
        self.assertEqual(res.returncode, 2)
        self.assertIn("unexpected untracked", res.stderr)
        self.assertFalse((feature_dir / "logs" / "impact-map" / "last_message.md").exists())

        run_dir = self._latest_run_dir(feature_dir, "impact_map")
        run_state = json.loads((run_dir / "run_state.json").read_text(encoding="utf-8"))
        self.assertEqual(run_state["phase"], "phase_b")

    def test_orchestrated_single_forces_json_and_captures_tool_failures(self) -> None:
        feature_dir, _ = self._make_feature_dir("orchestrated_single_json_run_state")
        artifact_source = feature_dir / "orchestrated_workstream_triage.md"
        _write_text(
            artifact_source,
            _triage_text(heading_tokens=["WDRA-PWS-contract", "WDRA-PWS-tasks_checkpoints"]),
        )

        res = self._run(
            feature_dir,
            agent="workstream_triage",
            primary_source=artifact_source,
            primary_dest_rel="logs/workstream-triage/staged/pre-planning/workstream_triage.md",
            orchestrated=True,
            emit_tool_failure=True,
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)

        run_dir = self._latest_run_dir(feature_dir, "workstream_triage")
        self.assertTrue((run_dir / "events.jsonl").is_file())
        run_state = json.loads((run_dir / "run_state.json").read_text(encoding="utf-8"))
        self.assertEqual(run_state["phase"], "single")
        self.assertEqual(run_state["tool_error_count"], 1)

    def test_impact_map_runner_rejects_invalid_staged_candidate(self) -> None:
        feature_dir, _ = self._make_feature_dir("impact_map_invalid_staged")
        artifact_source = feature_dir / "invalid_impact_map.md"
        _write_text(artifact_source, _impact_map_text(edit=["__impact_map_test__/missing.rs"]))

        res = self._run(
            feature_dir,
            agent="impact_map",
            primary_source=artifact_source,
            primary_dest_rel="logs/impact-map/staged/pre-planning/impact_map.md",
        )
        self.assertEqual(res.returncode, 1)
        self.assertIn("staged impact_map validation failed", res.stderr)
        self.assertFalse((feature_dir / "pre-planning" / "impact_map.md").exists())

    def test_impact_map_runner_orchestrated_rejects_invalid_staged_candidate(self) -> None:
        feature_dir, _ = self._make_feature_dir("impact_map_invalid_staged_orchestrated")
        artifact_source = feature_dir / "invalid_impact_map.md"
        _write_text(artifact_source, _impact_map_text(edit=["__impact_map_test__/missing.rs"]))

        res = self._run(
            feature_dir,
            agent="impact_map",
            primary_source=artifact_source,
            primary_dest_rel="logs/impact-map/staged/pre-planning/impact_map.md",
            orchestrated=True,
        )
        self.assertEqual(res.returncode, 1)
        self.assertIn("staged impact_map validation failed", res.stderr)
        self.assertFalse((feature_dir / "pre-planning" / "impact_map.md").exists())


if __name__ == "__main__":
    unittest.main()
