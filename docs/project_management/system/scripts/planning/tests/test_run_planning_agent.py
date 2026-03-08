import json
import os
import shutil
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


class TestRunPlanningAgent(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "run_planning_agent"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.script = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "run_planning_agent.sh"
        )
        if not cls.script.is_file():
            raise unittest.SkipTest("run_planning_agent.sh not found at expected canonical path.")

    def _write_codex_stub(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail

output_last=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-last-message)
      output_last="$2"
      shift 2
      ;;
    --cd|--config|--profile|--model)
      shift 2
      ;;
    exec|--dangerously-bypass-approvals-and-sandbox|--json|-)
      shift
      ;;
    *)
      shift
      ;;
  esac
done

cat >/dev/null
mkdir -p "${TEST_RUNNER_FEATURE_DIR}/pre-planning"
cp "${TEST_RUNNER_ARTIFACT_SOURCE}" "${TEST_RUNNER_FEATURE_DIR}/pre-planning/workstream_triage.md"
printf 'runner stub summary\n' >"${output_last}"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _make_feature_dir(self, name: str) -> tuple[Path, Path]:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)
        feature_dir.mkdir(parents=True, exist_ok=True)

        adr_path = feature_dir / "ADR-0001-fixture.md"
        _write_text(adr_path, "# ADR fixture\n")
        _write_text(
            feature_dir / "tasks.json",
            json.dumps(
                {
                    "meta": {
                        "schema_version": 4,
                        "slice_spec_version": 2,
                        "adr_paths": [str(adr_path)],
                    },
                    "tasks": [],
                },
                indent=2,
            ),
        )
        return feature_dir, adr_path

    def _run(self, feature_dir: Path, artifact_source: Path) -> subprocess.CompletedProcess[str]:
        bin_dir = feature_dir / "bin"
        codex_path = bin_dir / "codex"
        self._write_codex_stub(codex_path)

        env = os.environ.copy()
        env["PATH"] = f"{bin_dir}:{env['PATH']}"
        env["TEST_RUNNER_FEATURE_DIR"] = str(feature_dir)
        env["TEST_RUNNER_ARTIFACT_SOURCE"] = str(artifact_source)

        return subprocess.run(
            ["bash", str(self.script), "--feature-dir", str(feature_dir), "--agent", "workstream_triage"],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )

    def test_workstream_triage_runner_accepts_backticked_headings(self) -> None:
        feature_dir, _ = self._make_feature_dir("pass_backticked_headings")
        artifact_source = feature_dir / "backticked_workstream_triage.md"
        _write_text(
            artifact_source,
            _triage_text(heading_tokens=["`WDRA-PWS-contract`", "`WDRA-PWS-tasks_checkpoints`"]),
        )

        res = self._run(feature_dir, artifact_source)
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertIn("OK: tracked outputs within allowlist", res.stdout)

    def test_workstream_triage_runner_fails_true_heading_json_mismatch(self) -> None:
        feature_dir, _ = self._make_feature_dir("fail_heading_json_mismatch")
        artifact_source = feature_dir / "mismatch_workstream_triage.md"
        _write_text(
            artifact_source,
            _triage_text(heading_tokens=["`WDRA-PWS-contract_typo`", "`WDRA-PWS-tasks_checkpoints`"]),
        )

        res = self._run(feature_dir, artifact_source)
        self.assertEqual(res.returncode, 1)
        self.assertIn("workstream_triage closeout validation failed", res.stderr)
        self.assertIn("normalized to 'WDRA-PWS-contract_typo'", res.stderr)


if __name__ == "__main__":
    unittest.main()
