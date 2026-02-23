import json
import shutil
import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _mk_intake_markdown(vector_block: str) -> str:
    return (
        "# Intake Fixture\n\n"
        "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
        f"{vector_block}\n"
        "<!-- PM_LIFT_VECTOR:END -->\n"
    )


class TestPmLiftNegativeCases(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; pm_lift.py requires repo root discovery via git.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_lift" / "negative_cases"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.pm_lift = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pm_lift.py"
        if not cls.pm_lift.is_file():
            raise unittest.SkipTest("pm_lift.py not found at expected canonical path.")

    def _run_repo(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.pm_lift), *args]
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def _init_tmp_git_repo(self, name: str) -> Path:
        root = self.tmp_root / "tmp_repo" / name
        if root.exists():
            shutil.rmtree(root)
        root.mkdir(parents=True, exist_ok=True)

        subprocess.run(["git", "init"], cwd=str(root), capture_output=True, text=True, check=True)
        return root

    def _write_tmp_repo_script(self, tmp_repo_root: Path) -> Path:
        dst = tmp_repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pm_lift.py"
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(self.pm_lift, dst)
        return dst

    def _run_tmp_repo(self, tmp_repo_root: Path, pm_lift_path: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(pm_lift_path), *args]
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(tmp_repo_root),
        )

    def test_markers_present_but_missing_json_fence(self) -> None:
        fixture = self.tmp_root / "intake_missing_json_fence.md"
        _write_text(fixture, _mk_intake_markdown("not a fenced json block"))

        res = self._run_repo(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("lift markers present", res.stderr)
        self.assertIn("```json", res.stderr)

    def test_invalid_json_in_fence(self) -> None:
        fixture = self.tmp_root / "intake_invalid_json.md"
        block = "```json\n{\n  \"touch\": {,\n}\n```\n"
        _write_text(fixture, _mk_intake_markdown(block))

        res = self._run_repo(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("invalid lift JSON block", res.stderr)

    def test_model_version_mismatch_errors(self) -> None:
        fixture = self.tmp_root / "intake_model_version_2.md"
        vector = {
            "model_version": 2,
            "touch": {},
            "contract": {},
            "qa": {},
            "docs": {},
            "ops": {},
            "risk": {},
            "notes": "",
        }
        block = "```json\n" + json.dumps(vector, indent=2, sort_keys=True) + "\n```\n"
        _write_text(fixture, _mk_intake_markdown(block))

        res = self._run_repo(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertTrue(("model_version" in res.stderr) or ("unsupported model_version" in res.stderr), msg=res.stderr)

    def test_tmp_repo_invalid_model_config_json_fails(self) -> None:
        tmp_repo = self._init_tmp_git_repo("invalid_model_config_json")
        pm_lift = self._write_tmp_repo_script(tmp_repo)

        (tmp_repo / "docs" / "project_management" / "system" / "schemas").mkdir(parents=True, exist_ok=True)
        _write_text(
            tmp_repo / "docs" / "project_management" / "system" / "schemas" / "work_lift_model.v1.json",
            "{\n",
        )

        intake = tmp_repo / "intake.md"
        vector = {
            "touch": {},
            "contract": {},
            "qa": {},
            "docs": {},
            "ops": {},
            "risk": {},
            "notes": "",
        }
        block = "```json\n" + json.dumps(vector, indent=2, sort_keys=True) + "\n```\n"
        _write_text(intake, _mk_intake_markdown(block))

        res = self._run_tmp_repo(tmp_repo, pm_lift, ["from-intake", "--intake", str(intake), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("failed to parse model config JSON", res.stderr)
        self.assertIn("work_lift_model.v1.json", res.stderr)

    def test_tmp_repo_invalid_vector_schema_json_fails(self) -> None:
        tmp_repo = self._init_tmp_git_repo("invalid_vector_schema_json")
        pm_lift = self._write_tmp_repo_script(tmp_repo)

        (tmp_repo / "docs" / "project_management" / "system" / "schemas").mkdir(parents=True, exist_ok=True)
        _write_text(
            tmp_repo / "docs" / "project_management" / "system" / "schemas" / "work_lift_vector.schema.json",
            "{\n",
        )

        intake = tmp_repo / "intake.md"
        vector = {
            "touch": {},
            "contract": {},
            "qa": {},
            "docs": {},
            "ops": {},
            "risk": {},
            "notes": "",
        }
        block = "```json\n" + json.dumps(vector, indent=2, sort_keys=True) + "\n```\n"
        _write_text(intake, _mk_intake_markdown(block))

        res = self._run_tmp_repo(tmp_repo, pm_lift, ["from-intake", "--intake", str(intake), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("failed to parse lift vector schema JSON", res.stderr)
        self.assertIn("work_lift_vector.schema.json", res.stderr)

    def test_tmp_repo_missing_schema_allows_conservative_success(self) -> None:
        tmp_repo = self._init_tmp_git_repo("missing_schema_success")
        pm_lift = self._write_tmp_repo_script(tmp_repo)

        intake = tmp_repo / "intake.md"
        vector = {
            "touch": {},
            "contract": {},
            "qa": {},
            "docs": {},
            "ops": {},
            "risk": {},
            "notes": "",
        }
        block = "```json\n" + json.dumps(vector, indent=2, sort_keys=True) + "\n```\n"
        _write_text(intake, _mk_intake_markdown(block))

        res = self._run_tmp_repo(tmp_repo, pm_lift, ["from-intake", "--intake", str(intake), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "")

        out = json.loads(res.stdout)
        self.assertIsInstance(out, dict)
        self.assertIn("missing_inputs", out)
        self.assertIn("confidence", out)
        self.assertEqual(out["confidence"], "low")

    def test_tmp_repo_missing_schema_unknown_top_level_key_fails_actionably(self) -> None:
        tmp_repo = self._init_tmp_git_repo("missing_schema_unknown_key")
        pm_lift = self._write_tmp_repo_script(tmp_repo)

        intake = tmp_repo / "intake.md"
        vector = {
            "touch": {},
            "contract": {},
            "qa": {},
            "docs": {},
            "ops": {},
            "risk": {},
            "notes": "",
            "bogus": 123,
        }
        block = "```json\n" + json.dumps(vector, indent=2, sort_keys=True) + "\n```\n"
        _write_text(intake, _mk_intake_markdown(block))

        res = self._run_tmp_repo(tmp_repo, pm_lift, ["from-intake", "--intake", str(intake), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("unknown top-level keys", res.stderr)
        self.assertIn("schema missing", res.stderr)


if __name__ == "__main__":
    unittest.main()

