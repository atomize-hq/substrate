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


def valid_global_exchange_authority() -> dict:
    archived = {
        "path": str(RECOVERY.GLOBAL_EXCHANGE_PATH),
        "device": 16777230,
        "inode": 531753102,
        "uid": 501,
        "gid": 0,
        "mode": 0o040700,
    }
    rebound = {
        "path": str(RECOVERY.GLOBAL_EXCHANGE_PATH),
        "file_type": "directory",
        "uid": 501,
        "gid": 0,
        "mode": 0o700,
        "post_reboot_physical_identity": copy.deepcopy(
            RECOVERY.SEALED_POST_REBOOT_GLOBAL_EXCHANGE_IDENTITY
        ),
    }
    state = {"global_exchange": rebound}
    return {
        "root_install_claims": {"completion": {"installed_directories": [archived]}},
        "post_reboot_rebind": state,
        "post_reboot_rebind_sha256": RECOVERY.document_sha256(state),
    }


def valid_receipt_only_projection() -> dict:
    return {
        "experiment_id": RECOVERY.EXPERIMENT_ID,
        "repetition_scopes": RECOVERY.SCOPES,
        "cleanup_commits": [
            RECOVERY.PRIOR_CLEANUP_COMMIT,
            RECOVERY.COMPLETED_CLEANUP_COMMIT,
            "a" * 40,
        ],
        "terminal_route_sha256": RECOVERY.TERMINAL_CLEANUP_ROUTE_SHA256,
        "terminal_authority_file_sha256": (
            RECOVERY.TERMINAL_CLEANUP_AUTHORITY_FILE_SHA256
        ),
        "terminal_partial_rebind_sha256": (RECOVERY.TERMINAL_PARTIAL_REBIND_SHA256),
        "cleanup_target_absences": [
            {
                "path": "/Library/PrivilegedHelperTools/example",
                "lstat_return": -1,
                "raw_errno": 2,
            }
        ],
        "launchd_label": "com.atomize.substrate.r3-macos-evidence-finalizer.v2",
        "launchctl_not_found_exit_status": 113,
        "socket_path": RECOVERY.PRIOR_SOCKET_PHYSICAL_IDENTITY["path"],
        "socket_absent": True,
        "matching_processes": [],
        "global_exchange_path": str(RECOVERY.GLOBAL_EXCHANGE_PATH),
        "global_exchange_children": [],
        "archived_global_exchange_device": 16777230,
        "sealed_rebound_global_exchange_identity": copy.deepcopy(
            RECOVERY.SEALED_POST_REBOOT_GLOBAL_EXCHANGE_IDENTITY
        ),
        "restoration_receipt_path": str(RECOVERY.RESTORATION_PATH),
        "restoration_receipt_absent": True,
        "cleanup_mutations_authorized": False,
        "keychain_queries_authorized": False,
        "native_experiment_authorized": False,
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

    def test_final_observation_uses_sealed_rebound_global_exchange_identity(
        self,
    ) -> None:
        authority = valid_global_exchange_authority()
        expected_digest = authority["post_reboot_rebind_sha256"]
        rebound = RECOVERY.sealed_rebound_global_exchange_identity(
            authority, expected_digest
        )
        archived = authority["root_install_claims"]["completion"][
            "installed_directories"
        ][0]
        self.assertEqual(archived["device"], 16777230)
        self.assertEqual(rebound["device"], 16777229)

        class FakeObserver:
            @staticmethod
            def observe(expected: dict) -> dict:
                if expected != RECOVERY.SEALED_POST_REBOOT_GLOBAL_EXCHANGE_IDENTITY:
                    raise RECOVERY.Stop(
                        "global evidence exchange changed before admin restoration receipt"
                    )
                return {"global_exchange_stable_identity": expected}

        self.assertEqual(
            FakeObserver.observe(rebound)["global_exchange_stable_identity"],
            rebound,
        )
        with self.assertRaisesRegex(
            RECOVERY.Stop,
            "global evidence exchange changed before admin restoration receipt",
        ):
            FakeObserver.observe(archived)

        for field, replacement in (
            ("path", "/tmp/substituted"),
            ("device", 16777230),
            ("inode", 531753103),
            ("uid", 0),
            ("gid", 20),
            ("mode", 0o040755),
        ):
            with self.subTest(field=field):
                changed = valid_global_exchange_authority()
                changed["post_reboot_rebind"]["global_exchange"][
                    "post_reboot_physical_identity"
                ][field] = replacement
                changed["post_reboot_rebind_sha256"] = RECOVERY.document_sha256(
                    changed["post_reboot_rebind"]
                )
                with self.assertRaises(RECOVERY.Stop):
                    RECOVERY.sealed_rebound_global_exchange_identity(
                        changed, changed["post_reboot_rebind_sha256"]
                    )

    def test_receipt_only_resumption_rejects_incomplete_or_drifted_cleanup(
        self,
    ) -> None:
        value = valid_receipt_only_projection()
        self.assertIs(RECOVERY.validate_receipt_only_projection(value), value)

        variants = []
        target_remains = copy.deepcopy(value)
        target_remains["cleanup_target_absences"][0].update(
            {"lstat_return": 0, "raw_errno": 0}
        )
        variants.append(target_remains)

        loaded_launchd = copy.deepcopy(value)
        loaded_launchd["launchctl_not_found_exit_status"] = 0
        variants.append(loaded_launchd)

        socket_exists = copy.deepcopy(value)
        socket_exists["socket_absent"] = False
        variants.append(socket_exists)

        process_exists = copy.deepcopy(value)
        process_exists["matching_processes"] = [{"pid": 42}]
        variants.append(process_exists)

        nonempty_global = copy.deepcopy(value)
        nonempty_global["global_exchange_children"] = ["unexpected"]
        variants.append(nonempty_global)

        wrong_route = copy.deepcopy(value)
        wrong_route["terminal_route_sha256"] = "b" * 64
        variants.append(wrong_route)

        wrong_authority = copy.deepcopy(value)
        wrong_authority["terminal_authority_file_sha256"] = "c" * 64
        variants.append(wrong_authority)

        drifted_rebind = copy.deepcopy(value)
        drifted_rebind["sealed_rebound_global_exchange_identity"]["inode"] += 1
        variants.append(drifted_rebind)

        receipt_exists = copy.deepcopy(value)
        receipt_exists["restoration_receipt_absent"] = False
        variants.append(receipt_exists)

        for changed in variants:
            with self.subTest(changed=changed):
                with self.assertRaises(RECOVERY.Stop):
                    RECOVERY.validate_receipt_only_projection(changed)

        source = inspect.getsource(RECOVERY.execute_receipt_only)
        for prohibited in (
            "launchctl",
            "unlink_bound",
            "unlink_persistent_orphaned_socket",
            "remove_inventory_root",
            "remove_empty_directory",
            "os.unlink",
            "os.rmdir",
            "SecItem",
            "Security.framework",
        ):
            self.assertNotIn(prohibited, source)


if __name__ == "__main__":
    unittest.main()
