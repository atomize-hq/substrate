import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TestValidateSpecManifest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "validate_spec_manifest"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.validator = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "validate_spec_manifest.py"
        )
        if not cls.validator.is_file():
            raise unittest.SkipTest("validate_spec_manifest.py not found at expected canonical path.")

    def _run(self, feature_dir: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(self.validator), "--feature-dir", str(feature_dir)],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def _make_feature_dir(self, name: str, spec_manifest_text: str) -> Path:
        feature_dir = self.tmp_root / name
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_text(feature_dir / "pre-planning" / "spec_manifest.md", spec_manifest_text)
        return feature_dir

    def test_ignores_non_path_backticked_literals(self) -> None:
        feature_dir = self._make_feature_dir(
            "ignores_non_path_literals",
            "# Fixture\n\n"
            "## Required spec documents (authoritative)\n\n"
            "- `fixtures/example.md`\n"
            "- `fixtures/example.sh`\n"
            "- exact flag name `--pkg-manager`\n"
            "- exact env var `PKG_MANAGER`\n"
            "- exact decision line `Detected distro: {id}`\n",
        )
        _write_text(feature_dir / "fixtures" / "example.md", "# example\n")
        _write_text(feature_dir / "fixtures" / "example.sh", "#!/usr/bin/env bash\n")

        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)

    def test_rejects_placeholder_path_token(self) -> None:
        feature_dir = self._make_feature_dir(
            "rejects_placeholder_path",
            "# Fixture\n\n"
            "## Required spec documents (authoritative)\n\n"
            "- `docs/project_management/packs/draft/example/slices/<SLICE_ID>/<SLICE_ID>-spec.md`\n",
        )

        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 1)
        self.assertIn("placeholder token", res.stderr)

    def test_repo_root_relative_paths_resolve_from_repo_root(self) -> None:
        feature_dir = self._make_feature_dir(
            "repo_root_relative_paths",
            "# Fixture\n\n"
            "## Required spec documents (authoritative)\n\n"
            "- `scripts/substrate/install.sh`\n"
            "- `tests/installers/`\n",
        )

        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)

    def test_ignores_glob_like_directory_references(self) -> None:
        feature_dir = self._make_feature_dir(
            "ignores_glob_references",
            "# Fixture\n\n"
            "## Required spec documents (authoritative)\n\n"
            "- `fixtures/example.md`\n"
            "- references under `slices/BEDPM*/`\n",
        )
        _write_text(feature_dir / "fixtures" / "example.md", "# example\n")

        res = self._run(feature_dir)
        self.assertEqual(res.returncode, 0, msg=res.stderr)


if __name__ == "__main__":
    unittest.main()
