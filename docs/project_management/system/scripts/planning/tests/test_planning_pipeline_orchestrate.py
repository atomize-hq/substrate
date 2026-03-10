import json
import os
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
                "id": "WDRA-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": ["WDRA-PWS-contract"],
                "assumes": [],
                "owns": ["tasks.json", "session_log.md", "kickoff_prompts/", "slices/WDRA0/kickoff_prompts/"],
            },
        ],
    }
    return (
        "# Workstream triage\n\n"
        "### WDRA-PWS-contract — contract\n\n- Goal: fixture\n\n"
        "### WDRA-PWS-tasks_checkpoints — tasks_checkpoints\n\n- Goal: fixture\n\n"
        f"{BEGIN}\n```json\n{json.dumps(idx, indent=2, sort_keys=False)}\n```\n{END}\n"
    )


class TestPlanningPipelineOrchestrate(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "planning_pipeline"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.pipeline_script = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "planning_pipeline_orchestrate.sh"
        cls.full_planning_script = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "full_planning_orchestrate.sh"
        if not cls.pipeline_script.is_file() or not cls.full_planning_script.is_file():
            raise unittest.SkipTest("Pipeline/full-planning scripts not found at expected canonical paths.")

    def _write_stub(self, path: Path, label: str, *, extra_body: str = "") -> None:
        script = f"""#!/usr/bin/env bash
set -euo pipefail
echo "{label}" >>"${{PIPELINE_TRACE_FILE}}"
{extra_body}
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _make_feature_dir(self, name: str) -> Path:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)
        _write_text(feature_dir / "pre-planning" / "workstream_triage.md", _triage_text())
        _write_text(feature_dir / "tasks.json", json.dumps({"meta": {"schema_version": 4, "cross_platform": True}, "tasks": []}, indent=2))
        return feature_dir

    def test_pipeline_runs_pre_planning_then_convergence_then_full_planning(self) -> None:
        feature_dir = self._make_feature_dir("pipeline_order")
        trace_file = feature_dir / "pipeline_trace.log"
        pre_planning = feature_dir / "stub_pre_planning.sh"
        convergence = feature_dir / "stub_convergence.sh"
        full_planning = feature_dir / "stub_full_planning.sh"
        self._write_stub(pre_planning, "pre_planning")
        self._write_stub(convergence, "convergence")
        self._write_stub(full_planning, "full_planning")

        env = os.environ.copy()
        env["PIPELINE_TRACE_FILE"] = str(trace_file)
        env["PM_PRE_PLANNING_ORCHESTRATOR"] = str(pre_planning)
        env["PM_PRE_FULL_PLANNING_CONVERGE_SCRIPT"] = str(convergence)
        env["PM_FULL_PLANNING_ORCHESTRATOR"] = str(full_planning)
        res = subprocess.run(
            ["bash", str(self.pipeline_script), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(trace_file.read_text(encoding="utf-8").splitlines(), ["pre_planning", "convergence", "full_planning"])

    def test_full_planning_dry_run_invokes_convergence_first(self) -> None:
        feature_dir = self._make_feature_dir("full_planning_convergence")
        trace_file = feature_dir / "convergence_trace.log"
        convergence = feature_dir / "stub_convergence.sh"
        extra_body = f"""mkdir -p "{feature_dir / "pre-planning"}"
cat >"{feature_dir / "pre-planning" / "alignment_report.md"}" <<'EOF'
# Alignment report
EOF
"""
        self._write_stub(convergence, "convergence", extra_body=extra_body)

        env = os.environ.copy()
        env["PIPELINE_TRACE_FILE"] = str(trace_file)
        env["PM_FULL_PLANNING_CONVERGE_SCRIPT"] = str(convergence)
        res = subprocess.run(
            ["bash", str(self.full_planning_script), "--feature-dir", str(feature_dir), "--dry-run"],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(trace_file.read_text(encoding="utf-8").splitlines(), ["convergence"])
        self.assertTrue((feature_dir / "pre-planning" / "alignment_report.md").exists())

    def test_pipeline_picks_up_post_full_convergence_via_full_planning(self) -> None:
        feature_dir = self._make_feature_dir("pipeline_post_full")
        trace_file = feature_dir / "pipeline_post_full_trace.log"
        tools_dir = feature_dir / "tools"
        pre_planning = tools_dir / "stub_pre_planning.sh"
        pipeline_convergence = tools_dir / "stub_pipeline_convergence.sh"
        full_runner = tools_dir / "fake_runner.sh"
        make_stub = tools_dir / "make"
        git_stub = tools_dir / "git"
        inner_convergence = tools_dir / "stub_inner_convergence.sh"
        post_full = tools_dir / "stub_post_full.sh"

        self._write_stub(pre_planning, "pre_planning")
        self._write_stub(pipeline_convergence, "pipeline_pre_full")
        self._write_stub(
            inner_convergence,
            "full_planning_pre_full",
            extra_body=f"""mkdir -p "{feature_dir / "pre-planning"}"
cat >"{feature_dir / "pre-planning" / "alignment_report.md"}" <<'EOF'
# Alignment report
EOF
""",
        )
        self._write_stub(post_full, "post_full")
        self._write_stub(
            full_runner,
            "full_runner",
            extra_body="""pws_id=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --pws-id)
      pws_id="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
mkdir -p "$(dirname "$PIPELINE_TRACE_FILE")/logs/pws/${pws_id}"
: >"$(dirname "$PIPELINE_TRACE_FILE")/logs/pws/${pws_id}/stderr.log"
printf '%s-thread\n' "${pws_id}" >"$(dirname "$PIPELINE_TRACE_FILE")/logs/pws/${pws_id}/last_thread_id.txt"
""",
        )
        self._write_stub(make_stub, "micro_lint")
        _write_text(
            git_stub,
            "#!/usr/bin/env bash\n"
            "set -euo pipefail\n"
            'if [[ "${1:-}" == "status" && "${2:-}" == "--porcelain=v1" ]]; then exit 0; fi\n'
            'exec "${REAL_GIT}" "$@"\n',
        )
        os.chmod(git_stub, 0o755)

        env = os.environ.copy()
        env["PIPELINE_TRACE_FILE"] = str(trace_file)
        env["PM_PRE_PLANNING_ORCHESTRATOR"] = str(pre_planning)
        env["PM_PRE_FULL_PLANNING_CONVERGE_SCRIPT"] = str(pipeline_convergence)
        env["PM_FULL_PLANNING_ORCHESTRATOR"] = str(self.full_planning_script)
        env["PM_FULL_PLANNING_CONVERGE_SCRIPT"] = str(inner_convergence)
        env["PM_FULL_PLANNING_POST_CONVERGE_SCRIPT"] = str(post_full)
        env["PM_FULL_PLANNING_RUNNER"] = str(full_runner)
        env["REAL_GIT"] = subprocess.check_output(["which", "git"], text=True, cwd=str(self.repo_root)).strip()
        env["PATH"] = str(tools_dir) + os.pathsep + env["PATH"]

        res = subprocess.run(
            ["bash", str(self.pipeline_script), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        trace = trace_file.read_text(encoding="utf-8").splitlines()
        self.assertIn("post_full", trace)
        self.assertLess(trace.index("pre_planning"), trace.index("pipeline_pre_full"))
        self.assertLess(trace.index("pipeline_pre_full"), trace.index("full_planning_pre_full"))
        self.assertLess(trace.index("full_runner"), trace.index("post_full"))


if __name__ == "__main__":
    unittest.main()
