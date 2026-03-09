import json
import os
import shlex
import shutil
import subprocess
import sys
import unittest
from pathlib import Path


BEGIN = "<!-- PM_PWS_INDEX:BEGIN -->"
END = "<!-- PM_PWS_INDEX:END -->"


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _triage_text() -> str:
    idx = {
        "pws_index_version": 2,
        "slice_prefix": "WDRA",
        "accepted_slice_order": ["WDRA0"],
        "pws": [
            {
                "id": "WDRA-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": "WDRA-PWS-docs_validation",
                "role": "docs_validation",
                "depends_on": ["WDRA-PWS-contract"],
                "assumes": [],
                "owns": ["plan.md"],
            },
            {
                "id": "WDRA-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": ["WDRA-PWS-docs_validation"],
                "assumes": [],
                "owns": [
                    "tasks.json",
                    "session_log.md",
                    "kickoff_prompts/",
                    "slices/WDRA0/kickoff_prompts/",
                ],
            },
        ],
    }
    return (
        "# Workstream triage fixture\n\n"
        "### WDRA-PWS-contract — contract\n\n- Goal: fixture\n\n"
        "### WDRA-PWS-docs_validation — docs_validation\n\n- Goal: fixture\n\n"
        "### WDRA-PWS-tasks_checkpoints — tasks_checkpoints\n\n- Goal: fixture\n\n"
        f"{BEGIN}\n```json\n{json.dumps(idx, indent=2, sort_keys=False)}\n```\n{END}\n"
    )


def _checkpoint_plan_text() -> str:
    return (
        "# Fixture checkpoint plan\n\n"
        "## Machine-readable plan (linted)\n\n"
        "```json\n"
        + json.dumps(
            {
                "version": 1,
                "defaults": {"min_triads_per_checkpoint": 1, "max_triads_per_checkpoint": 8},
                "checkpoints": [
                    {
                        "id": "CP1",
                        "task_id": "CP1-ci-checkpoint",
                        "slices": ["WDRA0"],
                        "gates": {"compile_parity": True, "feature_smoke": False, "ci_testing": "quick"},
                        "rationale": "fixture",
                    }
                ],
            },
            indent=2,
            sort_keys=False,
        )
        + "\n```\n"
    )


class TestFullPlanningOrchestrate(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "full_planning_orchestrate"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.script = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "full_planning_orchestrate.sh"
        if not cls.script.is_file():
            raise unittest.SkipTest("full_planning_orchestrate.sh not found at expected canonical path.")

    def _make_feature_dir(self, name: str) -> Path:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)

        _write_text(feature_dir / "pre-planning" / "workstream_triage.md", _triage_text())
        _write_text(feature_dir / "pre-planning" / "minimal_spec_draft.md", "## Draft slice skeleton (pre-planning only)\n\n- slice_id: `WDRA0`\n")
        _write_text(feature_dir / "pre-planning" / "spec_manifest.md", "Canonical slice IDs:\n- `WDRA0`\n")
        _write_text(feature_dir / "pre-planning" / "impact_map.md", "Slice touch set:\n- `WDRA0`\n")
        _write_text(feature_dir / "pre-planning" / "ci_checkpoint_plan.md", _checkpoint_plan_text())
        _write_text(feature_dir / "plan.md", "# plan\n\n## Slices (sequencing)\n\n### WDRA0 — Fixture slice\n")
        _write_text(feature_dir / "contract.md", "# contract\n")
        _write_text(feature_dir / "tasks.json", json.dumps({"meta": {"schema_version": 4, "cross_platform": True, "slice_spec_version": 2}, "tasks": []}, indent=2))
        return feature_dir

    def _write_convergence_stub(self, path: Path, *, alignment_body: str = "# Alignment report\n") -> None:
        script = f"""#!/usr/bin/env bash
set -euo pipefail
feature_dir=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --feature-dir)
      feature_dir="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
mkdir -p "${{feature_dir}}/pre-planning"
cat >"${{feature_dir}}/pre-planning/alignment_report.md" <<'EOF'
{alignment_body.rstrip()}
EOF
echo "OK: pre-full-planning convergence passed"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_alignment_reporter(self, path: Path, *, body: str) -> None:
        script = f"""#!/usr/bin/env python3
print({body!r})
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_make_stub(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail
exit 0
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_git_wrapper(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "status" && "${2:-}" == "--porcelain=v1" ]]; then
  exit 0
fi
exec "${REAL_GIT}" "$@"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_runner_stub(
        self,
        path: Path,
        trace_path: Path,
        *,
        fail_tasks: bool = True,
        expected_alignment_text: str | None = None,
    ) -> None:
        expected_alignment = shlex.quote(expected_alignment_text or "")
        tasks_should_fail = "1" if fail_tasks else "0"
        script = f"""#!/usr/bin/env bash
set -euo pipefail

feature_dir=""
pws_id=""
resume_message=""
expected_alignment={expected_alignment}
tasks_should_fail="{tasks_should_fail}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --feature-dir)
      feature_dir="$2"
      shift 2
      ;;
    --pws-id)
      pws_id="$2"
      shift 2
      ;;
    --resume-message)
      resume_message="$2"
      shift 2
      ;;
    --resume-thread-id|--codex-profile|--codex-model)
      shift 2
      ;;
    --codex-jsonl)
      shift
      ;;
    *)
      shift
      ;;
  esac
done

step_dir="${{feature_dir}}/logs/pws/${{pws_id}}"
mkdir -p "${{step_dir}}"
printf '%s-thread\n' "${{pws_id}}" >"${{step_dir}}/last_thread_id.txt"

if [[ -n "${{resume_message}}" ]]; then
  printf '%s\t%s\n' "${{pws_id}}" "${{resume_message}}" >>"{trace_path}"
  if [[ ! -f "${{resume_message}}" ]]; then
    printf 'ERROR: resume message file not found: %s\n' "${{resume_message}}" >"${{step_dir}}/stderr.log"
    exit 2
  fi
  : >"${{step_dir}}/stderr.log"
  exit 0
fi

if [[ "${{pws_id}}" == "WDRA-PWS-tasks_checkpoints" && -n "${{expected_alignment}}" ]]; then
  alignment_report="${{feature_dir}}/pre-planning/alignment_report.md"
  if [[ ! -f "${{alignment_report}}" ]]; then
    printf 'ERROR: alignment report missing: %s\n' "${{alignment_report}}" >"${{step_dir}}/stderr.log"
    exit 2
  fi
  if ! grep -Fq -- "${{expected_alignment}}" "${{alignment_report}}"; then
    printf 'ERROR: alignment report was not refreshed before tasks gate\n' >"${{step_dir}}/stderr.log"
    exit 2
  fi
fi

if [[ "${{pws_id}}" == "WDRA-PWS-tasks_checkpoints" && "${{tasks_should_fail}}" == "1" ]]; then
  cat >"${{step_dir}}/allowlist_request.json" <<'EOF'
{{
  "pws_id": "WDRA-PWS-tasks_checkpoints",
  "requested_tracked_paths": [
    "plan.md",
    "pre-planning/ci_checkpoint_plan.md"
  ],
  "reason": "fixture checkpoint drift"
}}
EOF
  printf 'FAIL: validate_ci_checkpoint_plan.py fixture drift\n' >"${{step_dir}}/stderr.log"
  exit 2
fi

: >"${{step_dir}}/stderr.log"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def test_allowlist_auto_heal_uses_real_resume_message_path(self) -> None:
        feature_dir = self._make_feature_dir("allowlist_resume_path")
        tools_dir = feature_dir / "tools"
        trace_path = feature_dir / "resume_trace.tsv"
        runner = tools_dir / "fake_runner.sh"
        convergence = tools_dir / "fake_convergence.sh"
        reporter = tools_dir / "fake_alignment_reporter.py"
        fake_make = tools_dir / "make"
        fake_git = tools_dir / "git"
        self._write_runner_stub(runner, trace_path)
        self._write_convergence_stub(convergence)
        self._write_alignment_reporter(reporter, body="# Alignment report\n\n- generated\n")
        self._write_make_stub(fake_make)
        self._write_git_wrapper(fake_git)

        env = os.environ.copy()
        env["PM_FULL_PLANNING_RUNNER"] = str(runner)
        env["PM_FULL_PLANNING_CONVERGE_SCRIPT"] = str(convergence)
        env["PM_FULL_PLANNING_ALIGNMENT_REPORTER"] = str(reporter)
        env["PM_FULL_PLANNING_MAX_RESUMES"] = "2"
        env["REAL_GIT"] = subprocess.check_output(["which", "git"], text=True, cwd=str(self.repo_root)).strip()
        env["PATH"] = str(tools_dir) + os.pathsep + env["PATH"]

        res = subprocess.run(
            ["bash", str(self.script), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )

        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertNotIn("resume message file not found", res.stderr)
        self.assertTrue(trace_path.exists(), msg="runner stub did not record any resume-message usage")

        rows = [line.split("\t", 1) for line in trace_path.read_text(encoding="utf-8").splitlines() if line.strip()]
        self.assertTrue(rows)
        task_rows = [row for row in rows if row[0] == "WDRA-PWS-tasks_checkpoints"]
        self.assertEqual(len(task_rows), 1)

        resume_path = Path(task_rows[0][1])
        self.assertTrue(resume_path.is_file(), msg=f"resume path was not a real file: {resume_path}")
        self.assertIn("after_allowlist", resume_path.name)

    def test_refreshes_alignment_report_before_tasks_gate(self) -> None:
        feature_dir = self._make_feature_dir("alignment_report_refresh")
        tools_dir = feature_dir / "tools"
        trace_path = feature_dir / "runner_trace.tsv"
        runner = tools_dir / "fake_runner.sh"
        convergence = tools_dir / "fake_convergence.sh"
        reporter = tools_dir / "fake_alignment_reporter.py"
        fake_make = tools_dir / "make"
        fake_git = tools_dir / "git"
        self._write_runner_stub(runner, trace_path, fail_tasks=False, expected_alignment_text="fresh-marker")
        self._write_convergence_stub(convergence, alignment_body="# Alignment report\n\nstale-marker\n")
        self._write_alignment_reporter(reporter, body="# Alignment report\n\nfresh-marker\n")
        self._write_make_stub(fake_make)
        self._write_git_wrapper(fake_git)

        env = os.environ.copy()
        env["PM_FULL_PLANNING_RUNNER"] = str(runner)
        env["PM_FULL_PLANNING_CONVERGE_SCRIPT"] = str(convergence)
        env["PM_FULL_PLANNING_ALIGNMENT_REPORTER"] = str(reporter)
        env["REAL_GIT"] = subprocess.check_output(["which", "git"], text=True, cwd=str(self.repo_root)).strip()
        env["PATH"] = str(tools_dir) + os.pathsep + env["PATH"]

        res = subprocess.run(
            ["bash", str(self.script), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )

        self.assertEqual(res.returncode, 0, msg=res.stderr)
        alignment_report = (feature_dir / "pre-planning" / "alignment_report.md").read_text(encoding="utf-8")
        self.assertIn("fresh-marker", alignment_report)
        self.assertNotIn("stale-marker", alignment_report)


if __name__ == "__main__":
    unittest.main()
