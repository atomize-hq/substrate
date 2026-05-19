from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class PrepareProvingRunCloseoutTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve().parents[6]
        cls.script = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "execution" / "prepare_proving_run_closeout.py"
        if not cls.script.is_file():
            raise unittest.SkipTest("prepare_proving_run_closeout.py not found at expected canonical path")

    def setUp(self) -> None:
        self.temp_dir = Path(tempfile.mkdtemp(prefix="prepare_proving_run_closeout_"))

    def _write_json(self, relative_path: str, payload: dict) -> Path:
        path = self.temp_dir / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        return path

    def _run(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(self.script), *args],
            cwd=self.repo_root,
            text=True,
            capture_output=True,
            check=False,
        )

    def _baseline_facts(self) -> dict:
        return {
            "schema_version": 1,
            "run_id": "create-run-001",
            "mode": "create",
            "lifecycle": {
                "current_state": "published_baseline",
            },
            "publication": {
                "status": "green",
                "published_at": "2026-05-04T12:00:00Z",
                "artifact_path": "artifacts/publication.json",
                "evidence_refs": [
                    "artifacts/publication.json",
                    "artifacts/summary.md",
                ],
            },
        }

    def test_generates_scaffold_when_human_inputs_are_missing(self) -> None:
        facts_path = self._write_json("facts.json", self._baseline_facts())
        output_path = self.temp_dir / "proving-run-closeout.json"

        result = self._run("--facts", str(facts_path), "--output", str(output_path))

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual(payload["machine_owned"]["run_id"], "create-run-001")
        self.assertEqual(payload["machine_owned"]["lifecycle"]["source_state"], "published_baseline")
        self.assertEqual(payload["machine_owned"]["lifecycle"]["target_state"], "closed_baseline")
        self.assertEqual(payload["handoff"]["status"], "awaiting_human_inputs")
        self.assertEqual(
            payload["handoff"]["required_human_fields"],
            ["human_owned.residual_friction", "human_owned.manual_edits"],
        )
        self.assertIsNone(payload["human_owned"]["residual_friction"])
        self.assertIsNone(payload["human_owned"]["manual_edits"])
        self.assertEqual(len(payload["preparation"]["inputs"]), 1)

    def test_marks_closeout_ready_when_required_human_inputs_are_present(self) -> None:
        facts_path = self._write_json("facts.json", self._baseline_facts())
        human_inputs_path = self._write_json(
            "human-inputs.json",
            {
                "residual_friction": "low: one manual review pass",
                "manual_edits": [],
                "operator_notes": "publication diff already reconciled",
            },
        )
        output_path = self.temp_dir / "proving-run-closeout.json"

        result = self._run(
            "--facts",
            str(facts_path),
            "--human-inputs",
            str(human_inputs_path),
            "--output",
            str(output_path),
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual(payload["handoff"]["status"], "ready_to_close")
        self.assertEqual(payload["handoff"]["required_human_fields"], [])
        self.assertEqual(payload["human_owned"]["manual_edits"], [])
        self.assertEqual(len(payload["preparation"]["inputs"]), 2)

    def test_rejects_non_published_baseline_source_state(self) -> None:
        facts = self._baseline_facts()
        facts["lifecycle"]["current_state"] = "draft_baseline"
        facts_path = self._write_json("facts.json", facts)

        result = self._run("--facts", str(facts_path), "--output", str(self.temp_dir / "closeout.json"))

        self.assertEqual(result.returncode, 2)
        self.assertIn("facts.lifecycle.current_state must be 'published_baseline'", result.stderr)

    def test_rejects_non_green_publication(self) -> None:
        facts = self._baseline_facts()
        facts["publication"]["status"] = "failed"
        facts_path = self._write_json("facts.json", facts)

        result = self._run("--facts", str(facts_path), "--output", str(self.temp_dir / "closeout.json"))

        self.assertEqual(result.returncode, 2)
        self.assertIn("facts.publication.status must be 'green'", result.stderr)


if __name__ == "__main__":
    unittest.main()
