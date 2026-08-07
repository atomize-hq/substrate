from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import jsonschema


SKILL = Path(__file__).resolve().parents[1]
SCRIPTS = SKILL / "scripts"
ASSETS = SKILL / "assets"
REFERENCES = SKILL / "references"
ROOT = SKILL.parents[2]


def run(*args: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    return subprocess.run(
        list(args),
        cwd=cwd,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )


def git(repo: Path, *args: str) -> str:
    result = run("git", *args, cwd=repo)
    if result.returncode != 0:
        raise AssertionError(result.stderr or result.stdout)
    return result.stdout.strip()


class ReceiptProtocolTests(unittest.TestCase):
    def test_existing_landed_receipt_remains_valid(self) -> None:
        result = run(
            sys.executable,
            str(SCRIPTS / "validate_receipt.py"),
            str(ASSETS / "landing-receipt.example.json"),
            "--expected-next",
            "EVIDENCE:EXAMPLE-2",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("status=LANDED_CLEAN", result.stdout)

    def test_closed_receipt_is_structurally_valid(self) -> None:
        result = run(
            sys.executable,
            str(SCRIPTS / "validate_receipt.py"),
            str(ASSETS / "closed-receipt.example.json"),
            "--expected-next",
            "EXAMPLE-2",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("status=CLOSED_CLEAN", result.stdout)

    def test_closed_receipt_rejects_remote_mutation(self) -> None:
        receipt = json.loads((ASSETS / "closed-receipt.example.json").read_text())
        receipt["publication"]["remote_mutation"] = True
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "receipt.json"
            path.write_text(json.dumps(receipt))
            result = run(
                sys.executable,
                str(SCRIPTS / "validate_receipt.py"),
                str(path),
                "--expected-next",
                "EXAMPLE-2",
            )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("remote_mutation must be false", result.stderr)

    def test_local_closeout_verifies_exact_git_truth(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            repo.mkdir()
            git(repo, "init", "-b", "main")
            git(repo, "config", "user.name", "Protocol Test")
            git(repo, "config", "user.email", "protocol@example.invalid")

            (repo / "base.txt").write_text("base\n")
            git(repo, "add", "base.txt")
            git(repo, "commit", "-m", "base")
            base = git(repo, "rev-parse", "HEAD")
            base_tree = git(repo, "rev-parse", "HEAD^{tree}")

            git(repo, "switch", "-c", "feat/example")
            (repo / "src").mkdir()
            (repo / "src/example.py").write_text("VALUE = 1\n")
            git(repo, "add", "src/example.py")
            git(repo, "commit", "-m", "work")
            work = git(repo, "rev-parse", "HEAD")
            work_tree = git(repo, "rev-parse", "HEAD^{tree}")

            (repo / "control").mkdir()
            (repo / "control/example.json").write_text("{}\n")
            git(repo, "add", "control/example.json")
            git(repo, "commit", "-m", "control")
            control = git(repo, "rev-parse", "HEAD")
            control_tree = git(repo, "rev-parse", "HEAD^{tree}")

            review = root / "review.json"
            review.write_text('{"terminal":"CLEAN"}\n')
            review_digest = "sha256:" + hashlib.sha256(review.read_bytes()).hexdigest()

            receipt = json.loads((ASSETS / "closed-receipt.example.json").read_text())
            receipt["expected_base"] = {"commit": base, "tree": base_tree}
            receipt["closed"].update(
                {
                    "worktree": str(repo),
                    "work_commit": work,
                    "work_tree": work_tree,
                    "control_commit": control,
                    "control_tree": control_tree,
                    "local_ref": control,
                }
            )
            receipt["review"]["record_path"] = str(review)
            receipt["review"]["record_sha256"] = review_digest
            receipt_path = root / "receipt.json"
            receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
            state = json.loads((ASSETS / "state.example.json").read_text())
            state["publication_mode"] = "local"
            state["remote"] = None
            state["remote_observation"] = {
                "name": None,
                "target_ref": "refs/heads/example",
                "before": None,
            }
            state["expected_base"] = {"commit": base, "tree": base_tree}
            state_path = root / "state.json"
            state_path.write_text(json.dumps(state, indent=2) + "\n")

            result = run(
                sys.executable,
                str(SCRIPTS / "verify_local_closeout.py"),
                str(receipt_path),
                "--state",
                str(state_path),
                "--expected-next",
                "EXAMPLE-2",
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("VERIFIED local closeout", result.stdout)

            state["remote_observation"]["target_ref"] = "refs/heads/other"
            state_path.write_text(json.dumps(state, indent=2) + "\n")
            mismatched_observation = run(
                sys.executable,
                str(SCRIPTS / "verify_local_closeout.py"),
                str(receipt_path),
                "--state",
                str(state_path),
                "--expected-next",
                "EXAMPLE-2",
            )
            self.assertNotEqual(mismatched_observation.returncode, 0)
            self.assertIn(
                "remote_observation.target_ref",
                mismatched_observation.stderr,
            )
            state["remote_observation"]["target_ref"] = "refs/heads/example"
            state_path.write_text(json.dumps(state, indent=2) + "\n")

            receipt["closed"]["control_commit"] = work
            receipt["closed"]["control_tree"] = work_tree
            receipt["closed"]["local_ref"] = work
            receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
            invalid = run(
                sys.executable,
                str(SCRIPTS / "verify_local_closeout.py"),
                str(receipt_path),
                "--state",
                str(state_path),
                "--expected-next",
                "EXAMPLE-2",
            )
            self.assertNotEqual(invalid.returncode, 0)
            self.assertIn("live local branch ref", invalid.stderr)


class StateProtocolTests(unittest.TestCase):
    def test_existing_remote_state_remains_valid(self) -> None:
        result = run(
            sys.executable,
            str(SCRIPTS / "validate_state.py"),
            str(ASSETS / "state.example.json"),
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_local_state_allows_no_remote(self) -> None:
        state = json.loads((ASSETS / "state.example.json").read_text())
        state["publication_mode"] = "local"
        state["remote"] = None
        state["remote_observation"] = {
            "name": None,
            "target_ref": "refs/heads/example",
            "before": None,
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "state.json"
            path.write_text(json.dumps(state))
            result = run(
                sys.executable,
                str(SCRIPTS / "validate_state.py"),
                str(path),
            )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_remote_state_still_requires_remote_name(self) -> None:
        state = json.loads((ASSETS / "state.example.json").read_text())
        state["remote"] = None
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "state.json"
            path.write_text(json.dumps(state))
            result = run(
                sys.executable,
                str(SCRIPTS / "validate_state.py"),
                str(path),
            )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("remote must be a non-empty string", result.stderr)


class EvidenceProtocolTests(unittest.TestCase):
    def test_remote_and_local_evidence_receipts_are_valid(self) -> None:
        for name in (
            "evidence-receipt.example.json",
            "local-evidence-receipt.example.json",
        ):
            with self.subTest(name=name):
                result = run(
                    sys.executable,
                    str(SCRIPTS / "validate_evidence_receipt.py"),
                    str(ASSETS / name),
                )
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_legacy_remote_evidence_defaults_to_remote_mode(self) -> None:
        receipt = json.loads((ASSETS / "evidence-receipt.example.json").read_text())
        del receipt["source"]["publication_mode"]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "receipt.json"
            path.write_text(json.dumps(receipt))
            result = run(
                sys.executable,
                str(SCRIPTS / "validate_evidence_receipt.py"),
                str(path),
            )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_local_evidence_rejects_live_remote_substitution(self) -> None:
        receipt = json.loads(
            (ASSETS / "local-evidence-receipt.example.json").read_text()
        )
        receipt["source"]["live_remote"] = receipt["source"]["commit"]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "receipt.json"
            path.write_text(json.dumps(receipt))
            result = run(
                sys.executable,
                str(SCRIPTS / "validate_evidence_receipt.py"),
                str(path),
            )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must not contain live_remote", result.stderr)

    def test_local_evidence_verifies_exact_checkout_and_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            repo.mkdir()
            git(repo, "init", "-b", "feat/evidence")
            git(repo, "config", "user.name", "Protocol Test")
            git(repo, "config", "user.email", "protocol@example.invalid")
            (repo / "product.txt").write_text("ready\n")
            git(repo, "add", "product.txt")
            git(repo, "commit", "-m", "checkpoint")
            commit = git(repo, "rev-parse", "HEAD")
            tree = git(repo, "rev-parse", "HEAD^{tree}")

            artifact = root / "evidence.json"
            artifact.write_text('{"visible":true}\n')
            digest = "sha256:" + hashlib.sha256(artifact.read_bytes()).hexdigest()
            receipt = json.loads(
                (ASSETS / "local-evidence-receipt.example.json").read_text()
            )
            receipt["source"].update(
                {
                    "commit": commit,
                    "tree": tree,
                    "local_ref": commit,
                    "worktree": str(repo),
                    "target_ref": "refs/heads/feat/evidence",
                }
            )
            receipt["environment"]["project_path"] = str(repo)
            receipt["evidence"]["artifact_path"] = str(artifact)
            receipt["evidence"]["artifact_sha256"] = digest
            path = root / "receipt.json"
            path.write_text(json.dumps(receipt, indent=2) + "\n")

            result = run(
                sys.executable,
                str(SCRIPTS / "verify_local_evidence.py"),
                str(path),
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("VERIFIED local evidence", result.stdout)


class JsonSchemaTests(unittest.TestCase):
    def test_receipt_examples_match_schema(self) -> None:
        schema = json.loads((REFERENCES / "task-receipt.schema.json").read_text())
        validator = jsonschema.Draft202012Validator(schema)
        for name in (
            "landing-receipt.example.json",
            "closed-receipt.example.json",
            "blocked-receipt.example.json",
        ):
            with self.subTest(name=name):
                instance = json.loads((ASSETS / name).read_text())
                errors = sorted(validator.iter_errors(instance), key=lambda item: list(item.path))
                self.assertEqual(errors, [], "\n".join(error.message for error in errors))

    def test_state_example_matches_schema(self) -> None:
        schema = json.loads(
            (REFERENCES / "orchestration-state.schema.json").read_text()
        )
        instance = json.loads((ASSETS / "state.example.json").read_text())
        validator = jsonschema.Draft202012Validator(schema)
        validator.validate(instance)
        instance["publication_mode"] = "local"
        instance["remote"] = None
        instance["remote_observation"] = {
            "name": None,
            "target_ref": "refs/heads/example",
            "before": None,
        }
        validator.validate(instance)

    def test_evidence_examples_match_schema(self) -> None:
        schema = json.loads(
            (REFERENCES / "evidence-receipt.schema.json").read_text()
        )
        validator = jsonschema.Draft202012Validator(schema)
        for name in (
            "evidence-receipt.example.json",
            "local-evidence-receipt.example.json",
        ):
            with self.subTest(name=name):
                instance = json.loads((ASSETS / name).read_text())
                validator.validate(instance)


class LifecycleSafetyTests(unittest.TestCase):
    def test_archive_safety_is_explicit_everywhere(self) -> None:
        skill = (SKILL / "SKILL.md").read_text()
        protocol = (REFERENCES / "protocol.md").read_text()
        meta_prompt = (ASSETS / "meta-orchestrator-prompt-template.md").read_text()
        increment_prompt = (
            ASSETS / "increment-orchestrator-prompt-template.md"
        ).read_text()

        self.assertIn("Never automatically archive a top-level task", skill)
        self.assertIn("Never archive a blocked", skill)
        self.assertIn("Never archive automatically", protocol)
        self.assertIn("Never automatically archive a task", meta_prompt)
        self.assertIn("Preserve the task-assigned worktree exactly", increment_prompt)

    def test_repo_local_skill_suite_has_worktree_hydration_contract(self) -> None:
        skill = (SKILL / "SKILL.md").read_text()
        protocol = (REFERENCES / "protocol.md").read_text()
        self.assertIn("repository-local skill", skill)
        self.assertIn("never install or register any of it as a global Codex skill", skill)
        self.assertIn("Git worktree creation materializes tracked repository content only", protocol)
        self.assertNotIn("install_global_skill.py", skill)
        self.assertNotIn("Install this skill under the global Codex skill root", protocol)

        with tempfile.TemporaryDirectory() as directory:
            worktree = Path(directory) / "task-worktree"
            worktree.mkdir()
            (worktree / ".git").write_text("gitdir: /tmp/example\n")

            hydrate = run(
                sys.executable,
                str(SCRIPTS / "hydrate_worktree_skill.py"),
                str(worktree),
            )
            self.assertEqual(hydrate.returncode, 0, hydrate.stderr)
            target = worktree / ".agents" / "skills"
            orchestration = target / "orchestrate-top-level-tasks"
            self.assertTrue((orchestration / "SKILL.md").is_file())
            self.assertTrue((target / "spec-driven-development" / "SKILL.md").is_file())
            self.assertFalse(target.is_symlink())
            self.assertNotIn("__pycache__", {part for path in target.rglob("*") for part in path.parts})
            hydration_report = json.loads(hydrate.stdout)
            self.assertGreater(hydration_report["skill_count"], 1)
            self.assertIn("spec-driven-development", hydration_report["skills"])

            check = run(
                sys.executable,
                str(SCRIPTS / "hydrate_worktree_skill.py"),
                str(worktree),
                "--check",
            )
            self.assertEqual(check.returncode, 0, check.stderr)
            self.assertIn('"status": "verified"', check.stdout)

            (target / "spec-driven-development" / "SKILL.md").write_text("different\n")
            mismatch = run(
                sys.executable,
                str(SCRIPTS / "hydrate_worktree_skill.py"),
                str(worktree),
                "--check",
            )
            self.assertNotEqual(mismatch.returncode, 0)
            self.assertIn("hydrated skill suite differs", mismatch.stderr)

    def test_verified_legacy_single_skill_copy_is_upgraded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            worktree = Path(directory) / "task-worktree"
            worktree.mkdir()
            (worktree / ".git").write_text("gitdir: /tmp/example\n")
            legacy = worktree / ".agents" / "skills" / "orchestrate-top-level-tasks"
            shutil.copytree(SKILL, legacy, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))

            hydrate = run(
                sys.executable,
                str(SCRIPTS / "hydrate_worktree_skill.py"),
                str(worktree),
            )
            self.assertEqual(hydrate.returncode, 0, hydrate.stderr)
            self.assertIn('"status": "upgraded"', hydrate.stdout)
            self.assertTrue(
                (worktree / ".agents" / "skills" / "spec-driven-development" / "SKILL.md").is_file()
            )


if __name__ == "__main__":
    unittest.main()
