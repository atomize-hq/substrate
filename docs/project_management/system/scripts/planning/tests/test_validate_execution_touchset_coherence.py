import json
import shutil
import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _impact_map(*, create: list[str] | None = None, edit: list[str] | None = None) -> str:
    def _section(tokens: list[str] | None) -> str:
        if not tokens:
            return "- None"
        return "\n".join(f"- `{token}`" for token in tokens)

    return (
        "# Impact Map Fixture\n\n"
        "## Touch set (explicit)\n\n"
        "### Create\n"
        f"{_section(create)}\n\n"
        "### Edit\n"
        f"{_section(edit)}\n\n"
        "### Deprecate\n"
        "- None\n\n"
        "### Delete\n"
        "- None\n"
    )


class TestValidateExecutionTouchsetCoherence(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; validator expects repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "validate_execution_touchset_coherence"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.validator = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "validate_execution_touchset_coherence.py"
        )
        if not cls.validator.is_file():
            raise unittest.SkipTest("validate_execution_touchset_coherence.py not found at expected canonical path.")

        sys.path.insert(0, str(cls.validator.parent))
        import validate_execution_touchset_coherence as vetc  # type: ignore

        cls.module = vetc

    def _make_feature_dir(self, name: str, *, impact_map_text: str, plan_text: str = "# plan\n", manual_text: str = "# manual\n") -> Path:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)
        _write_text(feature_dir / "pre-planning" / "impact_map.md", impact_map_text)
        _write_text(feature_dir / "plan.md", plan_text)
        _write_text(feature_dir / "manual_testing_playbook.md", manual_text)
        _write_text(feature_dir / "kickoff_prompts" / "K0.md", "# kickoff\n")
        _write_text(
            feature_dir / "tasks.json",
            json.dumps(
                {
                    "meta": {"slice_spec_version": 2},
                    "tasks": [
                        {
                            "id": "K0",
                            "type": "ops",
                            "kickoff_prompt": f"{feature_dir.as_posix()}/kickoff_prompts/K0.md",
                            "references": [],
                            "start_checklist": [],
                            "end_checklist": [],
                        }
                    ],
                },
                indent=2,
            ),
        )
        return feature_dir

    def _run(self, feature_dir: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(self.validator), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def test_missing_exact_path_from_plan_fails(self) -> None:
        feature_dir = self._make_feature_dir(
            "missing_plan_path",
            impact_map_text=_impact_map(create=["__impact_map_test__/placeholder.txt"]),
            plan_text="# plan\n\n- Update `Cargo.toml` for the fixture.\n",
        )
        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 1)
        self.assertIn("not covered by impact_map.md", res.stderr)

    def test_path_covered_by_allowed_prefix_passes(self) -> None:
        feature_dir = self._make_feature_dir(
            "prefix_pass",
            impact_map_text=_impact_map(edit=["scripts/"]),
        )
        _write_text(feature_dir / "kickoff_prompts" / "K0.md", "# kickoff\n\n- Update `scripts/check-host-prereqs.sh`.\n")
        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)

    def test_nonexistent_referenced_path_requires_create(self) -> None:
        feature_dir = self._make_feature_dir(
            "nonexistent_requires_create",
            impact_map_text=_impact_map(edit=["scripts/"]),
            manual_text="# manual\n\n- Verify `scripts/new-generated-script.sh` exists after generation.\n",
        )
        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 1)
        self.assertIn("does not cover it under Create", res.stderr)

    def test_ambiguity_is_hard_fail(self) -> None:
        feature_dir = self._make_feature_dir(
            "ambiguity_hard_fail",
            impact_map_text=_impact_map(create=["scripts/"], edit=["scripts/check-host-prereqs.sh"]),
        )
        _write_text(feature_dir / "kickoff_prompts" / "K0.md", "# kickoff\n\n- Update `scripts/check-host-prereqs.sh`.\n")
        issues = self.module.inspect_feature_dir(feature_dir)
        self.assertTrue(issues)
        self.assertEqual(issues[0].remediation, "hard_fail")
        self.assertIn("ambiguously covered", issues[0].message)


if __name__ == "__main__":
    unittest.main()
