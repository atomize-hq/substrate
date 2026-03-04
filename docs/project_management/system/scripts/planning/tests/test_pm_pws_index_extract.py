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


class TestPmPwsIndexExtract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_pws_index_extract"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.extractor = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "pm_pws_index_extract.py"
        )
        if not cls.extractor.is_file():
            raise unittest.SkipTest("pm_pws_index_extract.py not found at expected canonical path.")

    def _run(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.extractor), *args]
        return subprocess.run(cmd, text=True, capture_output=True, check=False, cwd=str(self.repo_root))

    def _make_feature_dir(self, name: str, triage_text: str | None) -> Path:
        feature_dir = self.tmp_root / name
        feature_dir.mkdir(parents=True, exist_ok=True)
        if triage_text is not None:
            _write_text(feature_dir / "pre-planning" / "workstream_triage.md", triage_text)
        return feature_dir

    def test_pass_extracts_role_and_normalizes_owns(self) -> None:
        prefix = "WDRA"
        contract_id = f"{prefix}-PWS-contract"
        tasks_id = f"{prefix}-PWS-tasks_checkpoints"
        pws = [
            {
                "id": contract_id,
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["./contract.md", "slices//WDRA0//WDRA0-spec.md"],
            },
            {
                "id": f"{prefix}-PWS-schema_inventory",
                "role": "schema_inventory",
                "depends_on": [contract_id],
                "assumes": [],
                "owns": ["telemetry-spec.md"],
            },
            {
                "id": tasks_id,
                "role": "tasks_checkpoints",
                "depends_on": [contract_id],
                "assumes": [],
                "owns": [
                    "tasks.json",
                    "session_log.md",
                    "kickoff_prompts/",
                    "slices/WDRA0/kickoff_prompts/",
                ],
            },
        ]
        feature_dir = self._make_feature_dir("pass_normalize", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir), "--pws-id", contract_id])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        data = json.loads(res.stdout)
        self.assertEqual(data["slice_prefix"], prefix)
        self.assertEqual(data["pws_id"], contract_id)
        self.assertEqual(data["role"], "contract")
        self.assertEqual(data["owns_norm"], ["contract.md", "slices/WDRA0/WDRA0-spec.md"])

    def test_pass_classifies_prefix_vs_exact(self) -> None:
        prefix = "WDRA"
        contract_id = f"{prefix}-PWS-contract"
        tasks_id = f"{prefix}-PWS-tasks_checkpoints"
        pws = [
            {
                "id": contract_id,
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["slices/", "contract.md"],
            },
            {
                "id": f"{prefix}-PWS-slice_spec_wdra0",
                "role": "slice_spec",
                "depends_on": [contract_id],
                "assumes": [],
                "owns": ["slices/WDRA0/WDRA0-spec.md"],
            },
            {
                "id": tasks_id,
                "role": "tasks_checkpoints",
                "depends_on": [contract_id],
                "assumes": [],
                "owns": [
                    "tasks.json",
                    "session_log.md",
                    "kickoff_prompts/",
                    "slices/WDRA0/kickoff_prompts/",
                ],
            },
        ]
        feature_dir = self._make_feature_dir("pass_prefix_exact", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir), "--pws-id", contract_id])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        data = json.loads(res.stdout)
        self.assertEqual(data["owns_prefix_norm"], ["slices/"])
        self.assertEqual(data["owns_exact_norm"], ["contract.md"])

    def test_fail_unknown_pws_id(self) -> None:
        prefix = "WDRA"
        contract_id = f"{prefix}-PWS-contract"
        tasks_id = f"{prefix}-PWS-tasks_checkpoints"
        pws = [
            {
                "id": contract_id,
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            },
            {
                "id": tasks_id,
                "role": "tasks_checkpoints",
                "depends_on": [contract_id],
                "assumes": [],
                "owns": ["tasks.json", "session_log.md", "kickoff_prompts/"],
            },
        ]
        feature_dir = self._make_feature_dir("fail_unknown_id", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir), "--pws-id", f"{prefix}-PWS-nope"])
        self.assertEqual(res.returncode, 2)
        self.assertIn("ERROR: unknown PWS_ID", res.stderr)

    def test_fail_invalid_index_surfaces_validator_failures(self) -> None:
        prefix = "WDRA"
        contract_id = f"{prefix}-PWS-contract"
        # Missing required WDRA-PWS-tasks_checkpoints.
        pws = [
            {
                "id": contract_id,
                "role": "contract",
                "depends_on": [],
                "assumes": [],
                "owns": ["contract.md"],
            }
        ]
        feature_dir = self._make_feature_dir("fail_invalid_index", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir), "--pws-id", contract_id])
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("missing required PWS id", res.stderr)
