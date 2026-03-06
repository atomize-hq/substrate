import json
import subprocess
import sys
import unittest
from pathlib import Path


def _write_json(path: Path, obj: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True), encoding="utf-8")


class TestParseAllowlistRequest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "parse_allowlist_request"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.script = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "parse_allowlist_request.py"
        )
        if not cls.script.is_file():
            raise unittest.SkipTest("parse_allowlist_request.py not found at expected canonical path.")

    def _run(self, request_path: Path, expected_pws_id: str = "") -> dict:
        cmd = [sys.executable, str(self.script), "--request", str(request_path)]
        if expected_pws_id:
            cmd.extend(["--expected-pws-id", expected_pws_id])
        res = subprocess.run(cmd, text=True, capture_output=True, check=False, cwd=str(self.repo_root))
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        return json.loads(res.stdout)

    def test_accepts_canonical_schema(self) -> None:
        request_path = self.tmp_root / "canonical.json"
        _write_json(
            request_path,
            {
                "pws_id": "BEDPM-PWS-tasks_checkpoints",
                "requested_tracked_paths": ["pre-planning/ci_checkpoint_plan.md"],
                "reason": "checkpoint plan drift",
                "required_updates": ["expand checkpoint slice list"],
            },
        )

        result = self._run(request_path, expected_pws_id="BEDPM-PWS-tasks_checkpoints")
        self.assertEqual(result["status"], "ok")
        self.assertEqual(result["alias_used"], None)
        self.assertEqual(result["requested_tracked_paths"], ["pre-planning/ci_checkpoint_plan.md"])
        self.assertEqual(result["extra_keys"], ["required_updates"])

    def test_accepts_legacy_requested_paths_alias(self) -> None:
        request_path = self.tmp_root / "legacy_alias.json"
        _write_json(
            request_path,
            {
                "pws_id": "BEDPM-PWS-tasks_checkpoints",
                "requested_paths": ["docs/project_management/packs/draft/foo/pre-planning/ci_checkpoint_plan.md"],
                "reason": "legacy agent emitted old field",
            },
        )

        result = self._run(request_path, expected_pws_id="BEDPM-PWS-tasks_checkpoints")
        self.assertEqual(result["status"], "ok")
        self.assertEqual(result["alias_used"], "requested_paths")
        self.assertEqual(
            result["requested_tracked_paths"],
            ["docs/project_management/packs/draft/foo/pre-planning/ci_checkpoint_plan.md"],
        )

    def test_reports_malformed_missing_reason(self) -> None:
        request_path = self.tmp_root / "missing_reason.json"
        _write_json(
            request_path,
            {
                "pws_id": "BEDPM-PWS-tasks_checkpoints",
                "requested_tracked_paths": ["pre-planning/ci_checkpoint_plan.md"],
            },
        )

        result = self._run(request_path, expected_pws_id="BEDPM-PWS-tasks_checkpoints")
        self.assertEqual(result["status"], "malformed")
        self.assertIn("missing or invalid 'reason'", "\n".join(result["errors"]))

    def test_reports_pws_id_mismatch(self) -> None:
        request_path = self.tmp_root / "pws_mismatch.json"
        _write_json(
            request_path,
            {
                "pws_id": "BEDPM-PWS-contract",
                "requested_tracked_paths": ["pre-planning/ci_checkpoint_plan.md"],
                "reason": "wrong pws id",
            },
        )

        result = self._run(request_path, expected_pws_id="BEDPM-PWS-tasks_checkpoints")
        self.assertEqual(result["status"], "malformed")
        self.assertIn("pws_id mismatch", "\n".join(result["errors"]))
