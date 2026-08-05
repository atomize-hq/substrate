import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "validate_r3_native_evidence.py"


def valid_artifact() -> dict:
    return {
        "schema_owner": "substrate.r3-native-evidence",
        "schema_version": 1,
        "evidence_id": "EVIDENCE:R3-LINUX-IMP-01",
        "source": {
            "commit": "a" * 40,
            "tree": "b" * 40,
            "ref": "refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap",
            "live_remote": "a" * 40,
        },
        "platform": {
            "os": "linux",
            "architecture": "x86_64",
        },
        "product_project_id": "2ccb802f-301c-4af4-9bd5-51d22808f0a2",
        "tool_versions": {
            "cargo": "cargo 1.89.0",
            "rustc": "rustc 1.89.0",
        },
        "allowed_actions": ["publish-manifest", "issue-receipt"],
        "prohibited_actions": ["delete", "kill"],
        "commands": [
            {
                "index": 1,
                "argv": ["cargo", "test", "-p", "shell"],
                "status": 0,
                "stdout_sha256": "sha256:" + "1" * 64,
                "stderr_sha256": "sha256:" + "2" * 64,
            }
        ],
        "artifact_manifest": {
            "sha256": "sha256:" + "3" * 64,
            "entries": ["review-control/r3-native-linux-01-evidence.json"],
        },
        "restoration_manifest": {
            "sha256": "sha256:" + "4" * 64,
            "entries": ["baseline.json", "restored.json"],
        },
        "intent_publication_capability": {
            "state_root_identity": "/var/lib/substrate",
            "filesystem_identity": "dev:0:1",
            "same_filesystem": True,
            "otmpfile_supported": True,
            "linkat_empty_path_supported": True,
            "effective_root": True,
            "probe_commands": [
                {
                    "index": 1,
                    "argv": ["probe", "--intent-publication"],
                    "status": 0,
                    "stdout_sha256": "sha256:" + "5" * 64,
                    "stderr_sha256": "sha256:" + "6" * 64,
                }
            ],
            "probe_artifact_sha256": "sha256:" + "7" * 64,
        },
        "correlation": {
            "dispatch_nonce": "nonce-1",
            "orchestration_id": "runtime-refactor-r3-home-manifest-linux-20260805T032054Z",
            "source_task_thread_id": "019fd20d-8874-7e40-aa56-ff75b72da4ac",
            "source_task_host_id": "remote-ssh-discovered:spenser-linux-codex",
            "evidence_task_thread_id": "019fd20d-8874-7e40-aa56-ff75b72da4ac",
            "evidence_task_host_id": "remote-ssh-discovered:spenser-linux-codex",
            "return_thread_id": "019fcff5-16c3-7de1-9ae4-c7f184a94adf",
            "return_host_id": "remote-ssh-discovered:spenser-linux-codex",
        },
        "result": {
            "status": "EVIDENCE_CLEAN",
            "summary": "Baseline matched restoration after manifest publication proof.",
            "restoration_exact": True,
        },
        "gated_successor": "AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT",
    }


def run_validator(path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            str(path),
            "--expected-evidence-id",
            "EVIDENCE:R3-LINUX-IMP-01",
            "--expected-source-commit",
            "a" * 40,
            "--expected-source-tree",
            "b" * 40,
            "--expected-source-ref",
            "refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap",
            "--expected-gated-successor",
            "AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT",
        ],
        text=True,
        capture_output=True,
        check=False,
    )


class ValidateR3NativeEvidenceTests(unittest.TestCase):
    def write_artifact(self, artifact: dict) -> Path:
        handle = tempfile.NamedTemporaryFile("w", suffix=".json", delete=False)
        with handle:
            json.dump(artifact, handle)
        return Path(handle.name)

    def test_accepts_valid_artifact(self) -> None:
        artifact_path = self.write_artifact(valid_artifact())
        result = run_validator(artifact_path)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("VALID:", result.stdout)

    def test_rejects_mismatched_evidence_id(self) -> None:
        artifact = valid_artifact()
        artifact["evidence_id"] = "EVIDENCE:R3-MAC-IMP-01"
        artifact_path = self.write_artifact(artifact)
        result = run_validator(artifact_path)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("evidence_id does not match", result.stderr)

    def test_rejects_live_remote_not_equal_commit(self) -> None:
        artifact = valid_artifact()
        artifact["source"]["live_remote"] = "c" * 40
        artifact_path = self.write_artifact(artifact)
        result = run_validator(artifact_path)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("source.live_remote must equal source.commit", result.stderr)

    def test_rejects_missing_correlation_field(self) -> None:
        artifact = valid_artifact()
        artifact["correlation"].pop("dispatch_nonce")
        artifact_path = self.write_artifact(artifact)
        result = run_validator(artifact_path)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("correlation fields do not match the closed shape", result.stderr)

    def test_rejects_wrong_gated_successor(self) -> None:
        artifact = valid_artifact()
        artifact["gated_successor"] = "AUTHORITY_REQUIRED:A1.1d-5R3-MAC"
        artifact_path = self.write_artifact(artifact)
        result = run_validator(artifact_path)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("gated_successor does not match", result.stderr)

    def test_rejects_platform_mismatch_for_evidence_id(self) -> None:
        artifact = valid_artifact()
        artifact["platform"]["os"] = "windows"
        artifact_path = self.write_artifact(artifact)
        result = run_validator(artifact_path)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("platform.os does not match", result.stderr)

    def test_rejects_overlapping_allowed_and_prohibited_actions(self) -> None:
        artifact = valid_artifact()
        artifact["prohibited_actions"] = ["delete", "publish-manifest"]
        artifact_path = self.write_artifact(artifact)
        result = run_validator(artifact_path)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must be disjoint", result.stderr)

    def test_rejects_non_json_manifest_entry(self) -> None:
        artifact = valid_artifact()
        artifact["artifact_manifest"]["entries"] = ["review-control/r3-native-linux-01-evidence.txt"]
        artifact_path = self.write_artifact(artifact)
        result = run_validator(artifact_path)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must contain JSON artifact paths", result.stderr)


if __name__ == "__main__":
    unittest.main()
