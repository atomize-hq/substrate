import json
import subprocess
import sys
import unittest
from pathlib import Path


BEGIN = "<!-- PM_PWS_INDEX:BEGIN -->"
END = "<!-- PM_PWS_INDEX:END -->"


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _triage_text(*, slice_prefix: str, pws: list[dict]) -> str:
    headings = []
    for p in pws:
        headings.append(f"### {p['id']} — {p['role']}\n\n- Goal: fixture\n")

    idx = {"pws_index_version": 1, "slice_prefix": slice_prefix, "pws": pws}
    body = json.dumps(idx, indent=2, sort_keys=False)
    return (
        "# Workstream triage fixture\n\n"
        "## Planning workstreams (PWS)\n\n"
        + "\n".join(headings)
        + "\n"
        + f"{BEGIN}\n"
        + "```json\n"
        + body
        + "\n```\n"
        + f"{END}\n"
    )


class TestValidatePwsIndex(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; validator tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "validate_pws_index"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.validator = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "validate_pws_index.py"
        )
        if not cls.validator.is_file():
            raise unittest.SkipTest("validate_pws_index.py not found at expected canonical path.")

    def _run(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.validator), *args]
        return subprocess.run(cmd, text=True, capture_output=True, check=False, cwd=str(self.repo_root))

    def _make_feature_dir(self, name: str, triage_text: str | None) -> Path:
        feature_dir = self.tmp_root / name
        feature_dir.mkdir(parents=True, exist_ok=True)
        if triage_text is not None:
            _write_text(feature_dir / "pre-planning" / "workstream_triage.md", triage_text)
        return feature_dir

    def test_pass_valid_contract_and_tasks_checkpoints(self) -> None:
        prefix = "WDRA"
        pws = [
            {
                "id": f"{prefix}-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": ["ADR-0000 is authoritative"],
                "owns": ["contract.md"],
            },
            {
                "id": f"{prefix}-PWS-schema_inventory",
                "role": "schema_inventory",
                "depends_on": [f"{prefix}-PWS-contract"],
                "assumes": ["Tokens reused verbatim"],
                "owns": ["telemetry-spec.md"],
            },
            {
                "id": f"{prefix}-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": [f"{prefix}-PWS-contract"],
                "assumes": [],
                "owns": ["tasks.json", "plan.md"],
            },
        ]
        feature_dir = self._make_feature_dir("pass_valid", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "")

    def test_fail_assumes_mentions_pws_id(self) -> None:
        prefix = "WDRA"
        contract_id = f"{prefix}-PWS-contract"
        pws = [
            {
                "id": contract_id,
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": f"{prefix}-PWS-schema_inventory",
                "role": "schema_inventory",
                "depends_on": [contract_id],
                "assumes": [f"{contract_id} drafts wording first"],
                "owns": ["telemetry-spec.md"],
            },
            {
                "id": f"{prefix}-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": [contract_id],
                "assumes": [],
                "owns": ["tasks.json", "plan.md"],
            },
        ]
        feature_dir = self._make_feature_dir("fail_assumes_mentions_id", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("assumes must not contain PWS id strings", res.stderr)

    def test_fail_overlapping_owns(self) -> None:
        prefix = "WDRA"
        pws = [
            {
                "id": f"{prefix}-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": f"{prefix}-PWS-schema_inventory",
                "role": "schema_inventory",
                "depends_on": [f"{prefix}-PWS-contract"],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": f"{prefix}-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": [f"{prefix}-PWS-contract"],
                "assumes": [],
                "owns": ["tasks.json", "plan.md"],
            },
        ]
        feature_dir = self._make_feature_dir("fail_overlapping_owns", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("owns path overlap is not allowed", res.stderr)

    def test_fail_missing_required_pws(self) -> None:
        prefix = "WDRA"
        pws = [
            {
                "id": f"{prefix}-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            }
        ]
        feature_dir = self._make_feature_dir("fail_missing_required", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("missing required PWS id", res.stderr)

    def test_fail_tasks_json_wrong_owner(self) -> None:
        prefix = "WDRA"
        pws = [
            {
                "id": f"{prefix}-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md", "tasks.json"],
            },
            {
                "id": f"{prefix}-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": [f"{prefix}-PWS-contract"],
                "assumes": [],
                "owns": ["plan.md"],
            },
        ]
        feature_dir = self._make_feature_dir("fail_tasks_json_owner", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("tasks.json must be owned", res.stderr)

    def test_fail_depends_on_unknown_id(self) -> None:
        prefix = "WDRA"
        pws = [
            {
                "id": f"{prefix}-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": f"{prefix}-PWS-tasks_checkpoints",
                "role": "tasks_checkpoints",
                "depends_on": [f"{prefix}-PWS-nope"],
                "assumes": [],
                "owns": ["tasks.json", "plan.md"],
            },
        ]
        feature_dir = self._make_feature_dir("fail_unknown_dep", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("references unknown PWS id", res.stderr)

    def test_fail_cycle(self) -> None:
        prefix = "WDRA"
        contract = f"{prefix}-PWS-contract"
        tasks = f"{prefix}-PWS-tasks_checkpoints"
        pws = [
            {
                "id": contract,
                "role": "contract",
                "depends_on": [tasks],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": tasks,
                "role": "tasks_checkpoints",
                "depends_on": [contract],
                "assumes": [],
                "owns": ["tasks.json", "plan.md"],
            },
        ]
        feature_dir = self._make_feature_dir("fail_cycle", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("contains a cycle", res.stderr)

    def test_advisory_missing_triage_is_warn_only(self) -> None:
        feature_dir = self._make_feature_dir("advisory_missing_triage", triage_text=None)
        res = self._run(["--feature-dir", str(feature_dir), "--advisory"])
        self.assertEqual(res.returncode, 0)
        self.assertIn("WARN:", res.stderr)


if __name__ == "__main__":
    unittest.main()

