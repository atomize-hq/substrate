import json
import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TestPmLiftGoldensIntake(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; pm_lift.py requires repo root discovery via git.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_lift" / "goldens_intake"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.pm_lift = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pm_lift.py"
        if not cls.pm_lift.is_file():
            raise unittest.SkipTest("pm_lift.py not found at expected canonical path.")

    def _run(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.pm_lift), *args]
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def _run_intake_emit_json(self, vector: dict) -> dict:
        fixture = self.tmp_root / "intake_fixture.md"
        text = (
            "# Intake Fixture\n\n"
            "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
            "```json\n"
            + json.dumps(vector, indent=2, sort_keys=True)
            + "\n```\n"
            "<!-- PM_LIFT_VECTOR:END -->\n"
        )
        _write_text(fixture, text)

        res = self._run(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr on success.")
        return json.loads(res.stdout)

    def _assert_contract_3_minimum(self, out: dict) -> None:
        required = [
            "model_version",
            "lift_score",
            "estimated_slices",
            "confidence",
            "triggers",
            "missing_inputs",
            "vector",
            "derived",
        ]
        for k in required:
            self.assertIn(k, out)

        self.assertIsInstance(out["model_version"], int)
        self.assertIsInstance(out["lift_score"], int)
        self.assertIsInstance(out["estimated_slices"], int)
        self.assertIn(out["confidence"], {"high", "low"})

        self.assertIsInstance(out["triggers"], list)
        self.assertTrue(all(isinstance(x, str) for x in out["triggers"]))
        self.assertIsInstance(out["missing_inputs"], list)
        self.assertTrue(all(isinstance(x, str) for x in out["missing_inputs"]))

        self.assertEqual(out["triggers"], sorted(set(out["triggers"])))
        self.assertEqual(out["missing_inputs"], sorted(set(out["missing_inputs"])))

        self.assertIsInstance(out["vector"], dict)
        self.assertIsInstance(out["derived"], dict)

    def test_golden_a_fully_specified_high_confidence_no_triggers(self) -> None:
        vector = {
            "model_version": 1,
            "touch": {
                "create_files": 1,
                "edit_files": 2,
                "delete_files": 0,
                "deprecate_files": 0,
                "crates_touched": 1,
                "boundary_crossings": 0,
            },
            "contract": {
                "cli_flags": 0,
                "config_keys": 0,
                "exit_codes": 0,
                "file_formats": 0,
                "behavior_deltas": 1,
            },
            "qa": {"new_test_files": 0, "new_test_cases": 0},
            "docs": {"new_docs_files": 0},
            "ops": {"new_smoke_steps": 0, "ci_changes": 0},
            "risk": {
                "cross_platform": False,
                "security_sensitive": False,
                "concurrency_or_ordering": False,
                "migration_or_backfill": False,
                "unknowns_high": 0,
            },
            "notes": "GOLDEN-A: fully specified; high confidence; no triggers.",
        }
        out = self._run_intake_emit_json(vector)
        self._assert_contract_3_minimum(out)

        self.assertEqual(out["lift_score"], 11)
        self.assertEqual(out["estimated_slices"], 1)
        self.assertEqual(out["confidence"], "high")
        self.assertEqual(out["missing_inputs"], [])
        self.assertEqual(out["triggers"], [])

    def test_golden_b_missing_input_crates_touched(self) -> None:
        vector = {
            "model_version": 1,
            "touch": {
                "create_files": 0,
                "edit_files": 0,
                "delete_files": 0,
                "deprecate_files": 0,
                "crates_touched": None,
                "boundary_crossings": 0,
            },
            "contract": {
                "cli_flags": 0,
                "config_keys": 0,
                "exit_codes": 0,
                "file_formats": 0,
                "behavior_deltas": 1,
            },
            "qa": {"new_test_files": 0, "new_test_cases": 0},
            "docs": {"new_docs_files": 0},
            "ops": {"new_smoke_steps": 0, "ci_changes": 0},
            "risk": {
                "cross_platform": False,
                "security_sensitive": False,
                "concurrency_or_ordering": False,
                "migration_or_backfill": False,
                "unknowns_high": 0,
            },
            "notes": "GOLDEN-B: minimal missing-input downgrade; matches CONTRACT-3 example shape.",
        }
        out = self._run_intake_emit_json(vector)
        self._assert_contract_3_minimum(out)

        self.assertEqual(out["lift_score"], 0)
        self.assertEqual(out["estimated_slices"], 1)
        self.assertEqual(out["confidence"], "low")
        self.assertEqual(out["missing_inputs"], ["touch.crates_touched"])
        self.assertEqual(set(out["triggers"]), {"missing_inputs:touch.crates_touched"})

    def test_golden_1_from_work_lift_model_v1_goldens_doc(self) -> None:
        vector = {
            "model_version": 1,
            "touch": {
                "create_files": 2,
                "edit_files": 3,
                "delete_files": 1,
                "deprecate_files": 0,
                "crates_touched": None,
                "boundary_crossings": 1,
            },
            "contract": {
                "cli_flags": 1,
                "config_keys": 1,
                "exit_codes": 0,
                "file_formats": 1,
                "behavior_deltas": 2,
            },
            "qa": {"new_test_files": 1, "new_test_cases": 4},
            "docs": {"new_docs_files": 1},
            "ops": {"new_smoke_steps": 1, "ci_changes": 1},
            "risk": {
                "cross_platform": False,
                "security_sensitive": False,
                "concurrency_or_ordering": True,
                "migration_or_backfill": True,
                "unknowns_high": 2,
            },
            "notes": "GOLDEN-1: copied from docs/project_management/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md",
        }
        out = self._run_intake_emit_json(vector)
        self._assert_contract_3_minimum(out)

        self.assertEqual(out["lift_score"], 78)
        self.assertEqual(out["estimated_slices"], 7)
        self.assertEqual(out["confidence"], "low")
        self.assertEqual(out["missing_inputs"], ["touch.crates_touched"])

        expected_triggers = {
            "missing_inputs:touch.crates_touched",
            "split_required:behavior_deltas>1",
            "likely_split:lift_score>24",
            "split_required:estimated_slices>3",
        }
        self.assertEqual(set(out["triggers"]), expected_triggers)


if __name__ == "__main__":
    unittest.main()

