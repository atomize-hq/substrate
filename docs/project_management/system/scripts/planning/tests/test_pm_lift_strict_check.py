import json
import os
import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _write_json(path: Path, obj: object) -> None:
    _write_text(path, json.dumps(obj, indent=2, sort_keys=True) + "\n")


def _impact_map_strict(create_tokens: list[str]) -> str:
    create_lines = "\n".join(f"- `{t}`" for t in create_tokens) if create_tokens else "- None"
    return (
        "# Impact Map Fixture\n\n"
        "## Touch set (explicit)\n\n"
        "### Create\n"
        f"{create_lines}\n\n"
        "### Edit\n"
        "- None\n\n"
        "### Deprecate\n"
        "- None\n\n"
        "### Delete\n"
        "- None\n"
    )


class TestPmLiftStrictCheck(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; strict checker requires repo root discovery via git.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_lift_strict_check"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.strict = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pm_lift_strict_check.py"
        if not cls.strict.is_file():
            raise unittest.SkipTest("pm_lift_strict_check.py not found at expected canonical path.")

    def _run(self, args: list[str], *, strict_enabled: bool) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.strict), *args]
        env = os.environ.copy()
        if strict_enabled:
            env["PM_LIFT_STRICT"] = "1"
        else:
            env.pop("PM_LIFT_STRICT", None)
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )

    def test_strict_disabled_skip(self) -> None:
        fixture = self.tmp_root / "intake_any.md"
        _write_text(fixture, "# Anything\n")
        res = self._run(["--intake", str(fixture)], strict_enabled=False)
        self.assertEqual(res.returncode, 0)
        self.assertTrue(res.stdout.startswith("SKIP:"), msg=res.stdout)

    def test_intake_strict_pass(self) -> None:
        fixture = self.tmp_root / "intake_strict_pass.md"
        text = (
            "# Intake Fixture\n\n"
            "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
            "```json\n"
            "{\n"
            '  "model_version": 1,\n'
            '  "touch": {\n'
            '    "create_files": 0,\n'
            '    "edit_files": 0,\n'
            '    "delete_files": 0,\n'
            '    "deprecate_files": 0,\n'
            '    "crates_touched": 0,\n'
            '    "boundary_crossings": 0\n'
            "  },\n"
            '  "contract": {\n'
            '    "cli_flags": 0,\n'
            '    "config_keys": 0,\n'
            '    "exit_codes": 0,\n'
            '    "file_formats": 0,\n'
            '    "behavior_deltas": 1\n'
            "  },\n"
            '  "qa": { "new_test_files": 0, "new_test_cases": 0 },\n'
            '  "docs": { "new_docs_files": 0 },\n'
            '  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },\n'
            '  "risk": {\n'
            '    "cross_platform": false,\n'
            '    "security_sensitive": false,\n'
            '    "concurrency_or_ordering": false,\n'
            '    "migration_or_backfill": false,\n'
            '    "unknowns_high": 0\n'
            "  },\n"
            '  "notes": ""\n'
            "}\n"
            "```\n"
            "<!-- PM_LIFT_VECTOR:END -->\n"
        )
        _write_text(fixture, text)
        res = self._run(["--intake", str(fixture)], strict_enabled=True)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("OK: strict lift check passed (intake)", res.stdout)

    def test_intake_strict_fail_on_missing_inputs(self) -> None:
        fixture = self.tmp_root / "intake_strict_fail.md"
        text = (
            "# Intake Fixture\n\n"
            "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
            "```json\n"
            "{\n"
            '  "touch": {},\n'
            '  "contract": {},\n'
            '  "qa": {},\n'
            '  "docs": {},\n'
            '  "ops": {},\n'
            '  "risk": {},\n'
            '  "notes": ""\n'
            "}\n"
            "```\n"
            "<!-- PM_LIFT_VECTOR:END -->\n"
        )
        _write_text(fixture, text)
        res = self._run(["--intake", str(fixture)], strict_enabled=True)
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("intake strict invariants failed", res.stderr)

    def test_pack_legacy_not_eligible(self) -> None:
        feature_dir = self.tmp_root / "pack_legacy"
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_json(feature_dir / "tasks.json", {"meta": {"slice_spec_version": 1}})
        _write_text(feature_dir / "impact_map.md", _impact_map_strict(["__pm_lift_test__/a.txt"]))
        res = self._run(["--feature-dir", str(feature_dir)], strict_enabled=True)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue(res.stdout.startswith("NOT ELIGIBLE:"), msg=res.stdout)

    def test_pack_strict_fail_on_prefix_entries(self) -> None:
        feature_dir = self.tmp_root / "pack_prefix_fail"
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_json(feature_dir / "tasks.json", {"meta": {"slice_spec_version": 2}})
        _write_text(feature_dir / "impact_map.md", _impact_map_strict(["__pm_lift_test__/prefix/"]))
        res = self._run(["--feature-dir", str(feature_dir)], strict_enabled=True)
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("forbids prefix entries", res.stderr)

    def test_pack_strict_pass_explicit_only(self) -> None:
        feature_dir = self.tmp_root / "pack_strict_pass"
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_json(feature_dir / "tasks.json", {"meta": {"slice_spec_version": 2}})
        _write_text(feature_dir / "impact_map.md", _impact_map_strict(["__pm_lift_test__/a.txt", "__pm_lift_test__/b.txt"]))
        res = self._run(["--feature-dir", str(feature_dir)], strict_enabled=True)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("OK: strict lift check passed (pack)", res.stdout)


if __name__ == "__main__":
    unittest.main()

