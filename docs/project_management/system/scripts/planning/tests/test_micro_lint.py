import json
import subprocess
import unittest
from pathlib import Path


BEGIN = "<!-- PM_PWS_INDEX:BEGIN -->"
END = "<!-- PM_PWS_INDEX:END -->"


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _triage_text(*, heading_tokens: list[str]) -> str:
    pws = [
        {
            "id": "WDRA-PWS-contract",
            "role": "contract",
            "depends_on": [],
            "assumes": [],
            "owns": ["contract.md"],
        },
        {
            "id": "WDRA-PWS-tasks_checkpoints",
            "role": "tasks_checkpoints",
            "depends_on": ["WDRA-PWS-contract"],
            "assumes": [],
            "owns": [
                "tasks.json",
                "session_log.md",
                "kickoff_prompts/",
                "slices/WDRA0/kickoff_prompts/",
            ],
        },
    ]
    headings = []
    for p, heading_token in zip(pws, heading_tokens):
        headings.append(f"### {heading_token} — {p['role']}\n\n- Goal: fixture\n")
    idx = {"pws_index_version": 1, "slice_prefix": "WDRA", "pws": pws}
    return (
        "# Workstream triage fixture\n\n"
        + "\n".join(headings)
        + "\n"
        + f"{BEGIN}\n```json\n{json.dumps(idx, indent=2, sort_keys=False)}\n```\n{END}\n"
    )


class TestMicroLint(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "micro_lint"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.script = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "micro_lint.sh"
        )
        if not cls.script.is_file():
            raise unittest.SkipTest("micro_lint.sh not found at expected canonical path.")

    def _make_feature_dir(self, name: str, triage_text: str) -> Path:
        feature_dir = self.tmp_root / name
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_text(feature_dir / "pre-planning" / "workstream_triage.md", triage_text)
        return feature_dir

    def _run(self, feature_dir: Path, *, agent: str | None) -> subprocess.CompletedProcess[str]:
        cmd = ["bash", str(self.script), "--feature-dir", str(feature_dir)]
        if agent is not None:
            cmd.extend(["--agent", agent])
        cmd.extend(["--", "pre-planning/workstream_triage.md"])
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def test_workstream_triage_agent_mode_passes_backticked_headings(self) -> None:
        feature_dir = self._make_feature_dir(
            "pass_backticked_headings",
            _triage_text(heading_tokens=["`WDRA-PWS-contract`", "`WDRA-PWS-tasks_checkpoints`"]),
        )
        res = self._run(feature_dir, agent="workstream_triage")
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("Structural validation (workstream_triage)", res.stdout)

    def test_workstream_triage_agent_mode_fails_structural_mismatch(self) -> None:
        feature_dir = self._make_feature_dir(
            "fail_structural_mismatch",
            _triage_text(heading_tokens=["`WDRA-PWS-contract_typo`", "`WDRA-PWS-tasks_checkpoints`"]),
        )
        res = self._run(feature_dir, agent="workstream_triage")
        self.assertEqual(res.returncode, 1)
        self.assertIn("heading PWS id missing from PM_PWS_INDEX JSON", res.stderr)

    def test_without_agent_preserves_text_only_behavior(self) -> None:
        feature_dir = self._make_feature_dir(
            "pass_without_agent_on_structural_mismatch",
            _triage_text(heading_tokens=["`WDRA-PWS-contract_typo`", "`WDRA-PWS-tasks_checkpoints`"]),
        )
        res = self._run(feature_dir, agent=None)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertNotIn("Structural validation", res.stdout)


if __name__ == "__main__":
    unittest.main()
