import io
import json
import os
import shutil
import subprocess
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TestPostFullPlanningConvergence(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "post_full_planning_convergence"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.helper_script = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "post_full_planning_convergence.py"
        cls.shell_script = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "post_full_planning_converge.sh"
        if not cls.helper_script.is_file() or not cls.shell_script.is_file():
            raise unittest.SkipTest("Post-full convergence scripts not found at expected canonical paths.")

        sys.path.insert(0, str(cls.helper_script.parent))
        import post_full_planning_convergence as pfpc  # type: ignore

        cls.module = pfpc

    def _make_feature_dir(self, name: str) -> Path:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)

        _write_text(feature_dir / "pre-planning" / "impact_map.md", "# impact\n")
        _write_text(feature_dir / "plan.md", "# plan\n")
        _write_text(feature_dir / "manual_testing_playbook.md", "# manual\n")
        _write_text(feature_dir / "kickoff_prompts" / "K0.md", "# kickoff\n")
        _write_text(
            feature_dir / "tasks.json",
            json.dumps(
                {
                    "meta": {"slice_spec_version": 2},
                    "tasks": [
                        {
                            "id": "K0",
                            "type": "ops",
                            "kickoff_prompt": f"{feature_dir.as_posix()}/kickoff_prompts/K0.md",
                        }
                    ],
                },
                indent=2,
            ),
        )
        return feature_dir

    def _run_main(self, feature_dir: Path) -> dict:
        stdout = io.StringIO()
        with redirect_stdout(stdout):
            rc = self.module.main(["--feature-dir", str(feature_dir)])
        self.assertEqual(rc, 0)
        return json.loads(stdout.getvalue())

    def test_classifier_pass(self) -> None:
        feature_dir = self._make_feature_dir("classifier_pass")
        with patch.object(self.module, "inspect_post_full_planning", return_value=[]):
            data = self._run_main(feature_dir)
        self.assertEqual(data["status"], "pass")
        self.assertEqual(data["stale_docs"], [])

    def test_classifier_needs_remediation(self) -> None:
        feature_dir = self._make_feature_dir("classifier_safe")
        issue = self.module.ConvergenceIssue(
            validator="validate_execution_touchset_coherence.py",
            source_name="kickoff_prompt",
            path="kickoff_prompts/K0.md",
            message="fixture safe drift",
            remediation="safe",
        )
        with patch.object(self.module, "inspect_post_full_planning", return_value=[issue]):
            data = self._run_main(feature_dir)
        self.assertEqual(data["status"], "needs_remediation")
        self.assertIn("kickoff_prompts/K0.md", data["stale_docs"])
        self.assertIn("pre-planning/impact_map.md", data["stale_docs"])

    def test_classifier_hard_fail(self) -> None:
        feature_dir = self._make_feature_dir("classifier_hard_fail")
        issue = self.module.ConvergenceIssue(
            validator="validate_slice_specs.py",
            source_name="validate_slice_specs.py",
            path="slices/WDRA0/WDRA0-spec.md",
            message="fixture hard fail",
            remediation="hard_fail",
        )
        with patch.object(self.module, "inspect_post_full_planning", return_value=[issue]):
            data = self._run_main(feature_dir)
        self.assertEqual(data["status"], "hard_fail")

    def _write_alignment_reporter(self, path: Path) -> None:
        _write_text(
            path,
            "#!/usr/bin/env python3\n"
            "print('# Alignment report\\n\\n- generated')\n",
        )
        os.chmod(path, 0o755)

    def _write_reconcile_runner(self, path: Path, *, mutate: bool) -> None:
        script = f"""#!/usr/bin/env python3
import json
import sys
from pathlib import Path

feature_dir = None
for idx, arg in enumerate(sys.argv):
    if arg == "--feature-dir":
        feature_dir = Path(sys.argv[idx + 1]).resolve()
        break
if feature_dir is None:
    raise SystemExit(2)

payload = json.loads((feature_dir / "logs" / "post-full-planning-convergence" / "remediation_input.json").read_text(encoding="utf-8"))
if {str(mutate)}:
    (feature_dir / "post-full-fixed").write_text("done\\n", encoding="utf-8")
    for rel in payload.get("stale_docs", []):
        target = feature_dir / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.name == "impact_map.md":
            target.write_text("# impact map\\n\\nfixed\\n", encoding="utf-8")
        else:
            target.write_text("# fixed\\n", encoding="utf-8")
raise SystemExit(0)
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_helper(self, path: Path, *, mode: str) -> None:
        script = f"""#!/usr/bin/env python3
import json
import sys
from pathlib import Path

feature_dir = None
for idx, arg in enumerate(sys.argv):
    if arg == "--feature-dir":
        feature_dir = Path(sys.argv[idx + 1]).resolve()
        break
if feature_dir is None:
    raise SystemExit(2)

mode = {mode!r}
fixed = (feature_dir / "post-full-fixed").exists()
if mode == "pass" or fixed:
    payload = {{"status": "pass", "stale_docs": [], "remediation_allowed": False, "issues": []}}
elif mode == "needs_remediation":
    payload = {{
        "status": "needs_remediation",
        "stale_docs": ["pre-planning/impact_map.md", "kickoff_prompts/K0.md"],
        "remediation_allowed": True,
        "issues": [
            {{
                "validator": "validate_execution_touchset_coherence.py",
                "source": "kickoff_prompt",
                "path": "kickoff_prompts/K0.md",
                "message": "fixture safe drift",
                "remediation": "safe"
            }}
        ]
    }}
else:
    payload = {{
        "status": "hard_fail",
        "stale_docs": [],
        "remediation_allowed": False,
        "issues": [
            {{
                "validator": "validate_slice_specs.py",
                "source": "validate_slice_specs.py",
                "path": "slices/W0/W0-spec.md",
                "message": "fixture hard fail",
                "remediation": "hard_fail"
            }}
        ]
    }}

print(json.dumps(payload, sort_keys=True))
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _run_shell(self, feature_dir: Path, *, helper_mode: str, mutate_runner: bool) -> subprocess.CompletedProcess[str]:
        helper = feature_dir / "fake_post_full_helper.py"
        runner = feature_dir / "fake_post_full_runner.py"
        reporter = feature_dir / "fake_alignment_reporter.py"
        self._write_helper(helper, mode=helper_mode)
        self._write_reconcile_runner(runner, mutate=mutate_runner)
        self._write_alignment_reporter(reporter)

        env = os.environ.copy()
        env["PM_POST_FULL_PLANNING_CONVERGENCE_SCRIPT"] = str(helper)
        env["PM_POST_FULL_PLANNING_AGENT_RUNNER"] = str(runner)
        env["PM_POST_FULL_PLANNING_ALIGNMENT_REPORTER"] = str(reporter)
        env["PM_POST_FULL_PLANNING_SKIP_COMMIT"] = "1"
        env["PM_POST_FULL_PLANNING_SKIP_CLEAN_CHECK"] = "1"
        return subprocess.run(
            ["bash", str(self.shell_script), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )

    def test_script_passes_clean_pack(self) -> None:
        feature_dir = self._make_feature_dir("script_pass")
        res = self._run_shell(feature_dir, helper_mode="pass", mutate_runner=True)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue((feature_dir / "pre-planning" / "alignment_report.md").exists())

    def test_script_remediates_safe_drift(self) -> None:
        feature_dir = self._make_feature_dir("script_remediate")
        res = self._run_shell(feature_dir, helper_mode="needs_remediation", mutate_runner=True)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue((feature_dir / "post-full-fixed").exists())

    def test_script_hard_fails(self) -> None:
        feature_dir = self._make_feature_dir("script_hard_fail")
        res = self._run_shell(feature_dir, helper_mode="hard_fail", mutate_runner=False)
        self.assertEqual(res.returncode, 1)
        summaries = sorted((feature_dir / "logs" / "post-full-planning-convergence").glob("*/summary.md"))
        self.assertTrue(summaries)
        self.assertIn("fixture hard fail", summaries[-1].read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
