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


def _triage_text(slice_prefix: str, accepted_slice_order: list[str], *, triage_version: int = 2) -> str:
    pws = [
        {
            "id": f"{slice_prefix}-PWS-contract",
            "role": "contract",
            "depends_on": [],
            "assumes": [],
            "owns": ["contract.md"],
        },
        {
            "id": f"{slice_prefix}-PWS-tasks_checkpoints",
            "role": "tasks_checkpoints",
            "depends_on": [f"{slice_prefix}-PWS-contract"],
            "assumes": [],
            "owns": [
                "tasks.json",
                "session_log.md",
                "kickoff_prompts/",
                *(f"slices/{sid}/kickoff_prompts/" for sid in accepted_slice_order),
            ],
        },
    ]
    idx = {
        "pws_index_version": triage_version,
        "slice_prefix": slice_prefix,
        "pws": pws,
    }
    if triage_version == 2:
        idx["accepted_slice_order"] = accepted_slice_order
        idx["draft_slice_order"] = [accepted_slice_order[0]]
    body = json.dumps(idx, indent=2, sort_keys=False)
    return (
        "# Workstream triage fixture\n\n"
        "### " + pws[0]["id"] + " — contract\n\n- Goal: fixture\n\n"
        "### " + pws[1]["id"] + " — tasks_checkpoints\n\n- Goal: fixture\n\n"
        + f"{BEGIN}\n```json\n{body}\n```\n{END}\n"
    )


def _checkpoint_plan_text(slice_ids: list[str]) -> str:
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
                        "slices": slice_ids,
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


def _tasks_json() -> dict:
    return {"meta": {"schema_version": 4, "cross_platform": True}, "tasks": []}


class TestPreFullPlanningConvergence(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pre_full_planning_convergence"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.helper = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pre_full_planning_convergence.py"
        cls.script = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pre_full_planning_converge.sh"
        if not cls.helper.is_file() or not cls.script.is_file():
            raise unittest.SkipTest("Convergence helper/scripts not found at expected canonical paths.")

    def _make_feature_dir(
        self,
        name: str,
        *,
        accepted_slice_order: list[str],
        checkpoint_slice_ids: list[str] | None = None,
        spec_manifest_slice_ids: list[str] | None = None,
        impact_map_slice_ids: list[str] | None = None,
        include_triage: bool = True,
        triage_version: int = 2,
    ) -> Path:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)
        slice_prefix = accepted_slice_order[0][:-1]
        checkpoint_slice_ids = checkpoint_slice_ids or list(accepted_slice_order)
        spec_manifest_slice_ids = spec_manifest_slice_ids or list(accepted_slice_order)
        impact_map_slice_ids = impact_map_slice_ids or list(accepted_slice_order)

        if include_triage:
            _write_text(
                feature_dir / "pre-planning" / "workstream_triage.md",
                _triage_text(slice_prefix, accepted_slice_order, triage_version=triage_version),
            )
        _write_text(
            feature_dir / "pre-planning" / "spec_manifest.md",
            "Canonical slice IDs selected for this feature:\n" + "\n".join(f"- `{sid}`" for sid in spec_manifest_slice_ids),
        )
        _write_text(
            feature_dir / "pre-planning" / "impact_map.md",
            "Slice touch set:\n" + "\n".join(f"- `{sid}`" for sid in impact_map_slice_ids),
        )
        _write_text(feature_dir / "pre-planning" / "ci_checkpoint_plan.md", _checkpoint_plan_text(checkpoint_slice_ids))
        _write_text(
            feature_dir / "pre-planning" / "minimal_spec_draft.md",
            "## Draft slice skeleton (pre-planning only)\n\n"
            f"Slice prefix (draft): `{slice_prefix}`\n\n"
            f"- slice_id: `{accepted_slice_order[0]}`\n",
        )
        _write_text(feature_dir / "tasks.json", json.dumps(_tasks_json(), indent=2, sort_keys=True))
        return feature_dir

    def _run_helper(self, feature_dir: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(self.helper), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def test_helper_classifies_pass(self) -> None:
        feature_dir = self._make_feature_dir("helper_pass", accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"])
        res = self._run_helper(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        data = json.loads(res.stdout)
        self.assertEqual(data["status"], "pass")
        self.assertEqual(data["accepted_slice_order"], ["WDAP0", "WDAP2", "WDAP1", "WDAP3"])

    def test_helper_classifies_needs_remediation(self) -> None:
        feature_dir = self._make_feature_dir(
            "helper_needs_remediation",
            accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"],
            checkpoint_slice_ids=["WDAP0", "WDAP1"],
        )
        res = self._run_helper(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        data = json.loads(res.stdout)
        self.assertEqual(data["status"], "needs_remediation")
        self.assertIn("ci_checkpoint_plan", data["stale_docs"])
        self.assertTrue(data["remediation_allowed"])

    def test_helper_classifies_hard_fail(self) -> None:
        feature_dir = self._make_feature_dir(
            "helper_hard_fail",
            accepted_slice_order=["WDAP0", "WDAP2"],
            include_triage=False,
        )
        res = self._run_helper(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        data = json.loads(res.stdout)
        self.assertEqual(data["status"], "hard_fail")
        self.assertFalse(data["remediation_allowed"])

    def test_helper_classifies_v1_triage_as_hard_fail(self) -> None:
        feature_dir = self._make_feature_dir(
            "helper_v1_triage_hard_fail",
            accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"],
            triage_version=1,
        )
        res = self._run_helper(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        data = json.loads(res.stdout)
        self.assertEqual(data["status"], "hard_fail")
        self.assertIn("requires PM_PWS_INDEX v2", data["issues"][0]["message"])

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

payload = json.loads((feature_dir / "logs" / "pre-full-planning-convergence" / "remediation_input.json").read_text(encoding="utf-8"))
accepted = payload["accepted_slice_order"]
if {str(mutate)}:
    (feature_dir / "pre-planning" / "spec_manifest.md").write_text(
        "Canonical slice IDs selected for this feature:\\n" + "\\n".join(f"- `{{sid}}`" for sid in accepted) + "\\n",
        encoding="utf-8",
    )
    (feature_dir / "pre-planning" / "impact_map.md").write_text(
        "Slice touch set:\\n" + "\\n".join(f"- `{{sid}}`" for sid in accepted) + "\\n",
        encoding="utf-8",
    )
    plan = {{
        "version": 1,
        "defaults": {{"min_triads_per_checkpoint": 1, "max_triads_per_checkpoint": 8}},
        "checkpoints": [{{
            "id": "CP1",
            "task_id": "CP1-ci-checkpoint",
            "slices": accepted,
            "gates": {{"compile_parity": True, "feature_smoke": False, "ci_testing": "quick"}},
            "rationale": "fixture"
        }}]
    }}
    (feature_dir / "pre-planning" / "ci_checkpoint_plan.md").write_text(
        "# Fixture checkpoint plan\\n\\n## Machine-readable plan (linted)\\n\\n```json\\n" + json.dumps(plan, indent=2, sort_keys=False) + "\\n```\\n",
        encoding="utf-8",
    )
raise SystemExit(0)
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_alignment_reporter(self, path: Path) -> None:
        script = """#!/usr/bin/env python3
import sys
from pathlib import Path

feature_dir = None
for idx, arg in enumerate(sys.argv):
    if arg == "--feature-dir":
        feature_dir = Path(sys.argv[idx + 1]).resolve()
        break
if feature_dir is None:
    raise SystemExit(2)
print("# Alignment report\\n\\n- generated")
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _run_script(self, feature_dir: Path, *, mutate_runner: bool) -> subprocess.CompletedProcess[str]:
        runner = feature_dir / "fake_reconcile_runner.py"
        reporter = feature_dir / "fake_alignment_reporter.py"
        self._write_reconcile_runner(runner, mutate=mutate_runner)
        self._write_alignment_reporter(reporter)
        env = os.environ.copy()
        env["PM_PRE_FULL_PLANNING_AGENT_RUNNER"] = str(runner)
        env["PM_PRE_FULL_PLANNING_ALIGNMENT_REPORTER"] = str(reporter)
        env["PM_PRE_FULL_PLANNING_SKIP_COMMIT"] = "1"
        env["PM_PRE_FULL_PLANNING_SKIP_CLEAN_CHECK"] = "1"
        return subprocess.run(
            ["bash", str(self.script), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )

    def test_script_noop_clean_pack(self) -> None:
        feature_dir = self._make_feature_dir("script_noop", accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"])
        res = self._run_script(feature_dir, mutate_runner=True)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue((feature_dir / "pre-planning" / "alignment_report.md").exists())

    def test_script_remediates_stale_docs(self) -> None:
        feature_dir = self._make_feature_dir(
            "script_remediate",
            accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"],
            checkpoint_slice_ids=["WDAP0", "WDAP1"],
            spec_manifest_slice_ids=["WDAP0", "WDAP1"],
            impact_map_slice_ids=["WDAP0", "WDAP1"],
        )
        res = self._run_script(feature_dir, mutate_runner=True)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        spec_manifest = (feature_dir / "pre-planning" / "spec_manifest.md").read_text(encoding="utf-8")
        self.assertIn("`WDAP3`", spec_manifest)

    def test_script_stops_after_bounded_failures(self) -> None:
        feature_dir = self._make_feature_dir(
            "script_exhausts",
            accepted_slice_order=["WDAP0", "WDAP2", "WDAP1", "WDAP3"],
            checkpoint_slice_ids=["WDAP0", "WDAP1"],
        )
        res = self._run_script(feature_dir, mutate_runner=False)
        self.assertEqual(res.returncode, 1)
        summaries = sorted((feature_dir / "logs" / "pre-full-planning-convergence").glob("*/summary.md"))
        self.assertTrue(summaries)
        self.assertIn("remediation attempts exhausted", summaries[-1].read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
