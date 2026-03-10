import json
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


class TestValidateImpactMapEmitJsonContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; validate_impact_map.py uses git to find repo root.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "validate_impact_map" / "emit_json_contract"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.validator = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "validate_impact_map.py"
        )
        if not cls.validator.is_file():
            raise unittest.SkipTest("validate_impact_map.py not found at expected canonical path.")

    def _run(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.validator), *args]
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def _make_feature_dir(self, name: str, *, slice_spec_version: int, impact_map_text: str | None) -> Path:
        feature_dir = self.tmp_root / name
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_json(feature_dir / "tasks.json", {"meta": {"slice_spec_version": int(slice_spec_version)}})
        if impact_map_text is not None:
            _write_text(feature_dir / "impact_map.md", impact_map_text)
        return feature_dir

    def _assert_required_shape(self, out: dict) -> None:
        required_keys = ["create", "edit", "deprecate", "delete", "dir_prefixes"]
        for k in required_keys:
            self.assertIn(k, out)
            self.assertIsInstance(out[k], list)
            self.assertTrue(all(isinstance(x, str) for x in out[k]))
            self.assertEqual(out[k], sorted(set(out[k])))
        for p in out["dir_prefixes"]:
            self.assertTrue(p.endswith("/"))

    def test_strict_a_explicit_only(self) -> None:
        feature_dir = self._make_feature_dir(
            "strict_a",
            slice_spec_version=2,
            impact_map_text=_impact_map_strict(["__impact_map_test__/a.txt", "__impact_map_test__/b.txt"]),
        )
        res = self._run(["--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr on success for synthetic create-only fixture.")

        out = json.loads(res.stdout)
        self.assertIsInstance(out, dict)
        self._assert_required_shape(out)
        self.assertEqual(out["dir_prefixes"], [])

    def test_strict_b_prefix_present(self) -> None:
        prefix = "__impact_map_test__/no_such_prefix/"
        feature_dir = self._make_feature_dir(
            "strict_b",
            slice_spec_version=2,
            impact_map_text=_impact_map_strict(["__impact_map_test__/c.txt", prefix]),
        )
        res = self._run(["--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr on success for synthetic create-only fixture.")

        out = json.loads(res.stdout)
        self.assertIsInstance(out, dict)
        self._assert_required_shape(out)
        self.assertEqual(out["create"], ["__impact_map_test__/c.txt", prefix])
        self.assertEqual(out["dir_prefixes"], [prefix])

    def test_strict_c_normalizes_leading_dot_slash(self) -> None:
        feature_dir = self._make_feature_dir(
            "strict_c",
            slice_spec_version=2,
            impact_map_text=_impact_map_strict(["./__impact_map_test__/d.txt"]),
        )
        res = self._run(["--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr on success for synthetic create-only fixture.")

        out = json.loads(res.stdout)
        self.assertIsInstance(out, dict)
        self._assert_required_shape(out)
        self.assertEqual(out["create"], ["__impact_map_test__/d.txt"])
        self.assertEqual(out["dir_prefixes"], [])

    def test_strict_d_validates_alternate_impact_map_path(self) -> None:
        feature_dir = self._make_feature_dir("strict_alt_path", slice_spec_version=2, impact_map_text=None)
        staged = feature_dir / "logs" / "impact-map" / "staged" / "pre-planning" / "impact_map.md"
        _write_text(staged, _impact_map_strict(["__impact_map_test__/alt.txt"]))

        res = self._run(["--feature-dir", str(feature_dir), "--impact-map-path", str(staged), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        out = json.loads(res.stdout)
        self.assertEqual(out["create"], ["__impact_map_test__/alt.txt"])

    def test_strict_e_alternate_impact_map_path_rejects_missing_edit_path(self) -> None:
        feature_dir = self._make_feature_dir("strict_alt_wrong_section", slice_spec_version=2, impact_map_text=None)
        staged = feature_dir / "logs" / "impact-map" / "staged" / "pre-planning" / "impact_map.md"
        _write_text(
            staged,
            "# Impact Map Fixture\n\n"
            "## Touch set (explicit)\n\n"
            "### Create\n"
            "- None\n\n"
            "### Edit\n"
            "- `__impact_map_test__/missing.rs`\n\n"
            "### Deprecate\n"
            "- None\n\n"
            "### Delete\n"
            "- None\n",
        )

        res = self._run(["--feature-dir", str(feature_dir), "--impact-map-path", str(staged)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("declared path does not exist", res.stderr)

    def test_legacy_a_emits_full_shape_with_empty_arrays(self) -> None:
        feature_dir = self._make_feature_dir("legacy_a", slice_spec_version=1, impact_map_text=None)
        res = self._run(["--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr for legacy emit-json.")

        out = json.loads(res.stdout)
        self.assertIsInstance(out, dict)
        self._assert_required_shape(out)
        self.assertEqual(out["create"], [])
        self.assertEqual(out["edit"], [])
        self.assertEqual(out["deprecate"], [])
        self.assertEqual(out["delete"], [])
        self.assertEqual(out["dir_prefixes"], [])

    def test_usage_a_missing_args_exit_2_stdout_empty(self) -> None:
        res = self._run([])
        self.assertEqual(res.returncode, 2)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("usage:", res.stderr.lower())

    def test_strict_fail_a_missing_impact_map_exit_1_stdout_empty(self) -> None:
        feature_dir = self._make_feature_dir("strict_fail_missing_impact_map", slice_spec_version=2, impact_map_text=None)
        res = self._run(["--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("missing required path", res.stderr)

    def test_strict_fail_b_glob_token_exit_1_stdout_empty(self) -> None:
        feature_dir = self._make_feature_dir(
            "strict_fail_glob",
            slice_spec_version=2,
            impact_map_text=_impact_map_strict(["__impact_map_test__/*.txt"]),
        )
        res = self._run(["--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("glob tokens are not allowed", res.stderr)


if __name__ == "__main__":
    unittest.main()
