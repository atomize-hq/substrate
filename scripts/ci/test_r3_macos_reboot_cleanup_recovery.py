#!/usr/bin/env python3
"""Focused effect-free regressions for the one-attempt reboot cleanup membrane."""

from __future__ import annotations

import copy
import importlib.util
import inspect
import pathlib
import unittest


SOURCE = (
    pathlib.Path(__file__).resolve().parents[1]
    / "mac/r3-macos-finalizer-reboot-cleanup.py"
)
SPEC = importlib.util.spec_from_file_location("r3_reboot_cleanup", SOURCE)
assert SPEC is not None and SPEC.loader is not None
RECOVERY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RECOVERY)


def valid_projection() -> dict:
    executable = {
        "role": "alternate_coordinator_executable",
        "path": (
            "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/"
            "substrate-r3-macos-evidence-coordinator-alternate-path"
        ),
        "file_type": "regular",
        "uid": 0,
        "gid": 0,
        "mode": 0o555,
        "link_count": 1,
        "size": 2_398_672,
        "sha256": "d56eae90f2a06f9f4e77d39560700e220e93c9d60060ec2d51bec5a9ee20d6aa",
        "code_identity": {
            "signing_identifier": (
                "com.atomize.substrate.r3-macos-evidence-coordinator.v2"
            ),
            "identity_sha256": "0" * 64,
        },
    }
    plist = {
        "role": "launchd_plist",
        "path": (
            "/Library/LaunchDaemons/"
            "com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist"
        ),
        "file_type": "regular",
        "uid": 0,
        "gid": 0,
        "mode": 0o644,
        "link_count": 1,
        "size": 948,
        "sha256": RECOVERY.LAUNCHD_PLIST_SHA256,
        "code_identity": None,
    }
    artifacts = [executable, plist]
    return {
        "experiment_id": RECOVERY.EXPERIMENT_ID,
        "repetition_scopes": RECOVERY.SCOPES,
        "ephemeral_claims_absent": True,
        "journal_root_absent": True,
        "cleanup_authority_source": (
            "durable_canonical_global_pre_effect_claim_copies"
        ),
        "durable_claim_bindings": {
            "root_install_claims_binding_sha256": (RECOVERY.ROOT_INSTALL_CLAIMS_SHA256),
            "root_install_preclaim_sha256": (RECOVERY.ROOT_INSTALL_PRECLAIM_SHA256),
            "root_install_completion_sha256": (RECOVERY.ROOT_INSTALL_COMPLETION_SHA256),
        },
        "preservation_finished_utc": RECOVERY.PRESERVATION_FINISHED_UTC,
        "observed_reboot_utc": RECOVERY.OBSERVED_REBOOT_UTC,
        "installed_artifacts": copy.deepcopy(artifacts),
        "frozen_artifact_bindings": copy.deepcopy(artifacts),
        "launchd_rehydration": {
            "classification": (
                "launchd_rehydrated_socket_from_exact_installed_plist_after_reboot"
            ),
            "label": "com.atomize.substrate.r3-macos-evidence-finalizer.v2",
            "plist_sha256": RECOVERY.LAUNCHD_PLIST_SHA256,
            "plist_path": plist["path"],
            "socket_path": (
                "/private/var/run/"
                "com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock"
            ),
            "socket_type": "socket",
            "socket_uid": 0,
            "socket_gid": 20,
            "socket_mode": 0o660,
            "socket_link_count": 1,
        },
        "conflicting_processes": [],
        "emergency_signer_deletion_receipt_sha256": (
            "ac6037f43205602cd8ccf465283ef680ef3b3fee8c8abf28bd96413adb47a49a"
        ),
        "unexplained_live_state_differences": [],
    }


def valid_persistent_orphan() -> dict:
    empty = RECOVERY.raw_stream(b"")
    not_found = RECOVERY.raw_stream(
        (
            'Bad request.\nCould not find service "'
            "com.atomize.substrate.r3-macos-evidence-finalizer.v2"
            '" in domain for system\n'
        ).encode()
    )
    socket_path = RECOVERY.PRIOR_SOCKET_PHYSICAL_IDENTITY["path"]
    return {
        "classification": "persistent_orphaned_socket",
        "launchd_label": "com.atomize.substrate.r3-macos-evidence-finalizer.v2",
        "launchd_label_absence_observations": [
            {
                "phase": phase,
                "exit_status": 113,
                "stdout": empty,
                "stderr": not_found,
            }
            for phase in ("before_socket_observation", "after_socket_observation")
        ],
        "installed_plist": {
            "path": (
                "/Library/LaunchDaemons/"
                "com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist"
            ),
            "sha256": RECOVERY.LAUNCHD_PLIST_SHA256,
            "physical_identity": {"inode": 531753115},
        },
        "socket_path": socket_path,
        "socket_type": "socket",
        "socket_uid": 0,
        "socket_gid": 20,
        "socket_mode": 0o660,
        "socket_link_count": 1,
        "socket_size": 0,
        "socket_physical_identity": copy.deepcopy(
            RECOVERY.PRIOR_SOCKET_PHYSICAL_IDENTITY
        ),
        "prior_socket_physical_identity": copy.deepcopy(
            RECOVERY.PRIOR_SOCKET_PHYSICAL_IDENTITY
        ),
        "platform_parent_path": "/private/var/run",
        "platform_parent_physical_identity": {
            "path": "/private/var/run",
            "device": 16777229,
            "inode": 531870206,
            "uid": 0,
            "gid": 1,
            "mode": 0o040775,
        },
        "root_owned_non_symlink_parent_traversal": True,
        "open_descriptor_observation": {
            "command": ["/usr/sbin/lsof", "-nP", "--", socket_path],
            "exit_status": 1,
            "stdout": empty,
            "stderr": empty,
            "matching_open_descriptors": [],
        },
        "listener_observation": {
            "command": ["/usr/sbin/netstat", "-anv", "-f", "unix"],
            "exit_status": 0,
            "exact_socket_path_matches": [],
        },
        "conflicting_processes": [],
        "unexplained_live_state_differences": [],
    }


class RebootCleanupContractTests(unittest.TestCase):
    def assert_rejected(self, value: dict) -> None:
        with self.assertRaises(RECOVERY.Stop):
            RECOVERY.validate_fixture_projection(value)

    def test_exact_reboot_rebind_contract_accepts_only_complete_authority(self) -> None:
        value = valid_projection()
        self.assertIs(RECOVERY.validate_fixture_projection(value), value)

        missing_claims = copy.deepcopy(value)
        del missing_claims["durable_claim_bindings"]["root_install_completion_sha256"]
        self.assert_rejected(missing_claims)

        ephemeral_claims_present = copy.deepcopy(value)
        ephemeral_claims_present["ephemeral_claims_absent"] = False
        self.assert_rejected(ephemeral_claims_present)

        journal_recreated_for_old_validator = copy.deepcopy(value)
        journal_recreated_for_old_validator["journal_root_absent"] = False
        self.assert_rejected(journal_recreated_for_old_validator)

        fabricated_authority = copy.deepcopy(value)
        fabricated_authority["cleanup_authority_source"] = "absence_inference"
        self.assert_rejected(fabricated_authority)

        preservation_after_reboot = copy.deepcopy(value)
        preservation_after_reboot["preservation_finished_utc"] = (
            RECOVERY.OBSERVED_REBOOT_UTC
        )
        self.assert_rejected(preservation_after_reboot)

    def test_installed_artifact_rebind_rejects_content_code_and_posture_drift(
        self,
    ) -> None:
        for field, replacement in (
            ("sha256", "1" * 64),
            ("uid", 501),
            ("mode", 0o755),
            ("file_type", "symlink"),
            ("link_count", 2),
        ):
            with self.subTest(field=field):
                value = valid_projection()
                value["installed_artifacts"][0][field] = replacement
                self.assert_rejected(value)

        code_drift = valid_projection()
        code_drift["installed_artifacts"][0]["code_identity"][
            "signing_identifier"
        ] = "com.example.unrelated"
        self.assert_rejected(code_drift)

        oversize = valid_projection()
        oversize["installed_artifacts"][0]["size"] = RECOVERY.MAX_MANAGED_FILE_BYTES + 1
        self.assert_rejected(oversize)

    def test_launchd_rehydration_rejects_unrelated_plist_label_and_socket(self) -> None:
        for field, replacement in (
            ("label", "com.example.unrelated"),
            ("plist_sha256", "2" * 64),
            ("socket_path", "/private/var/run/unrelated.sock"),
            ("socket_type", "regular"),
            ("socket_uid", 501),
            ("socket_mode", 0o666),
            ("socket_link_count", 2),
        ):
            with self.subTest(field=field):
                value = valid_projection()
                value["launchd_rehydration"][field] = replacement
                self.assert_rejected(value)

    def test_process_and_unexplained_live_state_are_fail_closed(self) -> None:
        process = valid_projection()
        process["conflicting_processes"] = [
            {
                "pid": 42,
                "path": process["installed_artifacts"][0]["path"],
            }
        ]
        self.assert_rejected(process)

        unexplained = valid_projection()
        unexplained["unexplained_live_state_differences"] = ["alternate socket"]
        self.assert_rejected(unexplained)

    def test_route_is_cleanup_only_and_does_not_recreate_deleted_authority(
        self,
    ) -> None:
        source = SOURCE.read_text()
        self.assertIn("journal_root_was_exactly_removed", source)
        self.assertIn('native_experiment_execution_authorized": False', source)
        self.assertIn(
            'security_framework_or_keychain_queries_authorized": False', source
        )
        self.assertNotIn("os.mkdir(membrane.JOURNAL_ROOT", source)
        self.assertNotIn("os.mkdir(membrane.CLAIM_ROOT", source)
        self.assertNotIn("SecItem", source)

    def test_persistent_orphaned_socket_is_exact_and_never_repeats_bootout(
        self,
    ) -> None:
        value = valid_persistent_orphan()
        self.assertIs(RECOVERY.validate_persistent_orphaned_socket(value), value)

        variants = []
        for field, replacement in (
            ("socket_type", "regular"),
            ("socket_type", "symlink"),
            ("socket_uid", 501),
            ("socket_gid", 0),
            ("socket_mode", 0o666),
            ("socket_link_count", 2),
            ("root_owned_non_symlink_parent_traversal", False),
        ):
            changed = copy.deepcopy(value)
            changed[field] = replacement
            variants.append(changed)

        identity_drift = copy.deepcopy(value)
        identity_drift["socket_physical_identity"]["inode"] += 1
        variants.append(identity_drift)

        loaded_job = copy.deepcopy(value)
        loaded_job["launchd_label_absence_observations"][0]["exit_status"] = 0
        variants.append(loaded_job)

        open_descriptor = copy.deepcopy(value)
        open_descriptor["open_descriptor_observation"]["exit_status"] = 0
        open_descriptor["open_descriptor_observation"]["matching_open_descriptors"] = [
            {"pid": 42}
        ]
        variants.append(open_descriptor)

        listener = copy.deepcopy(value)
        listener["listener_observation"]["exact_socket_path_matches"] = [
            value["socket_path"]
        ]
        variants.append(listener)

        process = copy.deepcopy(value)
        process["conflicting_processes"] = [{"pid": 42}]
        variants.append(process)

        for changed in variants:
            with self.subTest(changed=changed):
                with self.assertRaises(RECOVERY.Stop):
                    RECOVERY.validate_persistent_orphaned_socket(changed)

        execute_source = inspect.getsource(RECOVERY.execute)
        self.assertIn("unlink_persistent_orphaned_socket", execute_source)
        self.assertNotIn('"bootout"', execute_source)


if __name__ == "__main__":
    unittest.main()
