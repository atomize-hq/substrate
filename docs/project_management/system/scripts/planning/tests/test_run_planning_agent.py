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
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-last-message)
      output_last="$2"
      shift 2
      ;;
    --cd|--config|--profile|--model)
      shift 2
      ;;
    exec|--dangerously-bypass-approvals-and-sandbox|--json|-)
      shift
      ;;
    *)
      shift
      ;;
  esac
done

cat >/dev/null
write_mode="${TEST_RUNNER_WRITE_MODE:-staged}"

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

        return subprocess.run(
            ["bash", str(self.script), "--feature-dir", str(feature_dir), "--agent", agent],
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


if __name__ == "__main__":
    unittest.main()
