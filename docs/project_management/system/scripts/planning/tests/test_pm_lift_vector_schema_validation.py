import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TestPmLiftVectorSchemaValidation(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; pm_lift.py requires repo root discovery via git.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_lift" / "schema_validation"
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

    def test_schema_violation_minimum_behavior_deltas(self) -> None:
        fixture = self.tmp_root / "intake_behavior_deltas_zero.md"
        text = (
            "# Intake Fixture\n\n"
            "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
            "```json\n"
            "{\n"
            '  "model_version": 1,\n'
            '  "touch": { "create_files": 0, "edit_files": 0, "delete_files": 0, "deprecate_files": 0, "crates_touched": 0, "boundary_crossings": 0 },\n'
            '  "contract": { "cli_flags": 0, "config_keys": 0, "exit_codes": 0, "file_formats": 0, "behavior_deltas": 0 },\n'
            '  "qa": { "new_test_files": 0, "new_test_cases": 0 },\n'
            '  "docs": { "new_docs_files": 0 },\n'
            '  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },\n'
            '  "risk": { "cross_platform": false, "security_sensitive": false, "concurrency_or_ordering": false, "migration_or_backfill": false, "unknowns_high": 0 },\n'
            '  "notes": ""\n'
            "}\n"
            "```\n"
            "<!-- PM_LIFT_VECTOR:END -->\n"
        )
        _write_text(fixture, text)

        res = self._run(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertTrue(
            ("/contract/behavior_deltas" in res.stderr) or ("contract.behavior_deltas" in res.stderr),
            msg=res.stderr,
        )

    def test_schema_violation_type_touch_create_files_bool(self) -> None:
        fixture = self.tmp_root / "intake_touch_create_files_bool.md"
        text = (
            "# Intake Fixture\n\n"
            "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
            "```json\n"
            "{\n"
            '  "model_version": 1,\n'
            '  "touch": { "create_files": true, "edit_files": 0, "delete_files": 0, "deprecate_files": 0, "crates_touched": 0, "boundary_crossings": 0 },\n'
            '  "contract": { "cli_flags": 0, "config_keys": 0, "exit_codes": 0, "file_formats": 0, "behavior_deltas": 1 },\n'
            '  "qa": { "new_test_files": 0, "new_test_cases": 0 },\n'
            '  "docs": { "new_docs_files": 0 },\n'
            '  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },\n'
            '  "risk": { "cross_platform": false, "security_sensitive": false, "concurrency_or_ordering": false, "migration_or_backfill": false, "unknowns_high": 0 },\n'
            '  "notes": ""\n'
            "}\n"
            "```\n"
            "<!-- PM_LIFT_VECTOR:END -->\n"
        )
        _write_text(fixture, text)

        res = self._run(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertTrue(
            ("/touch/create_files" in res.stderr) or ("touch.create_files" in res.stderr),
            msg=res.stderr,
        )


if __name__ == "__main__":
    unittest.main()

