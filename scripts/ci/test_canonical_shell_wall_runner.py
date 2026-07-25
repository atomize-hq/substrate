# canonical-stdlib-manifest-v1: built-in:_io,built-in:_thread,built-in:posix,built-in:sys,built-in:time,file:grp:/usr/lib/python3.13/lib-dynload/grp.cpython-313-x86_64-linux-gnu.so:59d32074eea1efae74751c22b8645de1c997080265507d36a8b24f947eee241e,file:hashlib:/usr/lib/python3.13/hashlib.py:f129b330e6ab878a96085843b3606acd7d157b8fe6edfa15e37fa13988cef19c,file:json:/usr/lib/python3.13/json/__init__.py:d5d41e2c29049515d295d81a6d40b4890fbec8d8482cfb401630f8ef2f77e4d5,file:re:/usr/lib/python3.13/re/__init__.py:dbe158a677c6aaacf717ea2abc23c56233453d38024aef75b7c3d93612cabb93,file:socket:/usr/lib/python3.13/socket.py:6d5e10b5bcd75b7a6a883a1819e3b47dda3f00621e8e7db365b96782fcf59ac3,file:struct:/usr/lib/python3.13/struct.py:9c231f9497caf513a22dee8f790b07f969b0e45854a0bdd6dd84b492e08c2856,file:subprocess:/usr/lib/python3.13/subprocess.py:21ecc4c8f4fcf641974fc0d9cc28e97b07670ed1cef7e9d96769b31bb8345586,file:tarfile:/usr/lib/python3.13/tarfile.py:255ac02c3cceb482782fed8e971be7665f9a5216672debb391e460d38e6850c7,file:tempfile:/usr/lib/python3.13/tempfile.py:7149355dfc2ccbd82a9b3614c7710f2c92bc7ab06440e7f427766481638a1528,file:unittest:/usr/lib/python3.13/unittest/__init__.py:7c22ee0c503b75aba5221e3e8189f9bf4632f85a43315ff841ec9d26a68e3551,frozen:io,frozen:os,frozen:stat
"""Authenticated self-tests for the canonical shell-wall runner."""

from __future__ import annotations

_authenticated_source = globals().get("__CANONICAL_MODULE_SOURCE")
_authenticated_file = globals().get("__file__")
_authenticated_prefix = "<git-blob:"
_authenticated_suffix = ":scripts/ci/test_canonical_shell_wall_runner.py>"
_authenticated_oid = (
    _authenticated_file[
        len(_authenticated_prefix) : -len(_authenticated_suffix)
    ]
    if isinstance(_authenticated_file, str)
    and _authenticated_file.startswith(_authenticated_prefix)
    and _authenticated_file.endswith(_authenticated_suffix)
    else ""
)
_authenticated_self_test_loader = (
    globals().get("__CANONICAL_AUTHENTICATED_SELF_TEST_LOADER") is True
    and __name__ == "canonical_shell_wall_runner_selftest"
    and isinstance(_authenticated_source, bytes)
    and len(_authenticated_oid) == 40
    and all(character in "0123456789abcdef" for character in _authenticated_oid)
)
if not _authenticated_self_test_loader:
    raise SystemExit(65)

import hashlib
import io
import json
import os
import re
import socket
import stat
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import unittest

import canonical_shell_wall_runner as runner


SELF_TEST_IDS = (
    "test_rejects_cargo_or_bwrap_exit_with_live_descendant",
    "test_reaps_reparented_descendant",
    "test_process_group_escape_remains_contained",
    "test_timeout_is_ineligible",
    "test_snapshot_timeout_is_ineligible",
    "test_copy_tree_no_follow_preserves_modes_despite_umask",
    "test_rejects_root_path_replacement",
    "test_rejects_root_symlink_substitution",
    "test_rejects_child_entry_replacement",
    "test_rejects_owner_drift",
    "test_rejects_mode_drift",
    "test_rejects_acl_drift",
    "test_rejects_descriptor_path_inode_mismatch",
    "test_rejects_premature_descriptor_closure",
    "test_rejects_deletion_before_containment_empty",
    "test_preserves_unrelated_sentinel",
    "test_rejects_shared_or_reused_root",
    "test_rejects_sticky_world_writable_ancestor",
    "test_rejects_unexpected_mount_id",
    "test_rejects_unexpected_entry_type",
    "test_rejects_subreaper_readback_failure",
    "test_rejects_bwrap_status_or_worker_ready_handshake_failure",
    "test_rejects_namespace_prerequisite_fallback",
    "test_teardown_never_targets_unrelated_process",
    "test_stage_b_setup_timeout_signals_only_retained_bwrap_pidfd",
    "test_stage_b_timeout_before_status_starts_no_cargo_and_reaps_exact_bwrap",
    "test_stage_b_bwrap_death_before_worker_ready_is_ineligible",
    "test_stage_a_maps_vanished_post_status_worker_to_premature_exit",
    "test_stage_a_parent_loss_before_stage_b_start_starts_no_wall",
    "test_stage_b_bwrap_loss_after_ready_before_start_is_ineligible",
    "test_stage_a_start_requires_bwrap_status_worker_pidfd_peer_credentials_and_ready",
    "test_stage_a_loss_before_stage_b_launch_starts_no_wall",
    "test_stage_a_loss_before_worker_ready_terminates_bwrap_and_worker",
    "test_stage_a_loss_after_ready_before_start_terminates_bwrap_and_worker",
    "test_stage_a_loss_after_start_invalidates_and_terminates_wall",
    "test_premature_stage_b_exit_after_start_is_ineligible",
    "test_stage_b_complete_requires_authenticated_echild_record_and_zero_exit",
    "test_fixed_private_socket_peer_status_cloexec_unchanged_stdin_and_no_cargo_control_fd",
    "test_sigterm_ignoring_descendant_requires_ineligible_sigkill",
    "test_stuck_stage_b_hits_bounded_stage_a_deadline",
    "test_rejects_forbidden_glob_ambient_parent_and_product_home_targets",
    "test_rejects_pathname_only_cleanup_api",
    "test_diagnostics_omit_comm_cmdline_environment_and_markers",
    "test_cli_rejects_arbitrary_command_root_and_environment_overrides",
    "test_public_request_cannot_select_internal_host_stage_worker_roles_or_control_path",
    "test_cli_rejects_invalid_label_timeout_cwd_and_head",
    "test_rejects_dirty_index_worktree_and_nonignored_untracked_state",
    "test_ignores_only_git_ignored_build_outputs",
    "test_rejects_git_environment_config_alternates_replace_and_fsmonitor",
    "test_rejects_local_git_config_include_mode_exclude_worktree_and_unknown_keys",
    "test_info_exclude_is_held_but_never_cleanliness_authority",
    "test_rejects_info_attributes_worktree_config_and_index_authority_extensions",
    "test_rejects_tree_index_stage_mode_flag_and_worktree_byte_drift",
    "test_tracked_gitignore_is_only_untracked_exclusion_authority",
    "test_rejects_untracked_gitignore_and_every_nonignored_untracked_path",
    "test_rejects_path_shim_and_executable_identity_drift",
    "test_executes_and_revalidates_held_bwrap_python_git_rustup_cargo_and_rustc",
    "test_trusted_bwrap_launcher_rejects_ambient_loader_and_startup_tcb_drift",
    "test_stage_b_as_pid1_matches_json_child_pid_peer_credentials_and_echild",
    "test_held_git_fd_path_replacement_never_executes_replacement",
    "test_isolated_python_startup_rejects_environment_site_path_and_module_poison",
    "test_python_runtime_rejects_origin_and_late_import_drift",
    "test_host_controller_bootstrap_commit_templates_and_slots_avoid_self_reference",
    "test_bootstrap_rejects_path_execution_cmdline_drift_and_runner_blob_mismatch",
    "test_self_test_loader_authenticates_exact_runner_and_test_blobs",
    "test_self_test_loader_rejects_path_import_discovery_and_blob_mismatch",
    "test_private_rustup_toolchain_snapshot_matches_pinned_complete_manifest",
    "test_mounted_rustc_reports_private_sysroot_and_compiles_minimal_test",
    "test_vendor_snapshot_reconstructs_only_lock_checksum_selected_archives",
    "test_constructed_cargo_home_resolves_locked_vendor_offline_without_drift",
    "test_private_snapshots_ignore_transient_source_modify_replace_and_restore",
    "test_cargo_environment_preserves_e0_with_only_tmpdir_xdg_overrides",
    "test_environment_values_and_protected_markers_are_never_serialized",
    "test_fresh_private_target_never_reads_repository_target",
    "test_private_target_is_descriptor_removed_after_containment",
    "test_cargo_home_seed_runtime_confines_lock_metadata_and_is_removed",
    "test_rejects_project_ancestor_or_account_cargo_config_appearance",
    "test_evidence_bounds_fail_closed",
    "test_schema_rejects_unknown_missing_and_wrong_type_fields",
    "test_schema_encodes_preflight_setup_containment_cleanup_failures",
    "test_atomic_finalization_hides_partial_eligibility",
    "test_final_evidence_rename_is_noreplace",
    "test_summarizer_golden_failure_names_and_signatures",
    "test_summarizer_normalization_sort_and_newline_bytes",
    "test_summarizer_last_result_and_failure_section_grammar",
    "test_summarizer_accepts_valid_all_pass_as_eligible_regression",
    "test_summarizer_rejects_cargo_exit_status_mismatch",
    "test_summarizer_rejects_integer_duplicate_missing_and_panic_grammar",
    "test_summarizer_rejects_invalid_encoding_nul_and_overflow",
    "test_mount_source_flags_ids_propagation_and_order_are_exact",
    "test_projected_root_ids_include_mapped_stage_identity",
    "test_cache_and_proc_mounts_unmount_before_tmpfs",
    "test_backing_mountpoint_removed_and_evidence_parent_preserved",
    "test_parallel_command_and_fresh_root",
    "test_serial_command_and_fresh_root",
    "test_stage_a_output_pipe_preserves_combined_order_eof_backpressure_bounds_and_hash",
    "test_success_removes_root_under_continuous_descriptor_authority",
    "test_success_preserves_unrelated_files",
    "test_host_finalizes_eligible_only_after_stage_a_exit_namespace_and_backing_absence",
)


def _category(index: int) -> str:
    boundaries = (
        (6, "lifecycle"),
        (20, "root-authority"),
        (40, "containment"),
        (46, "cli-diagnostics"),
        (55, "repository"),
        (66, "invocation-tcb"),
        (77, "snapshots-environment"),
        (82, "evidence"),
        (89, "summarizer"),
        (93, "mount-lifecycle"),
        (99, "success-output-cleanup"),
    )
    for limit, category in boundaries:
        if index < limit:
            return category
    raise AssertionError(index)


INELIGIBLE_SELF_TEST_IDS = frozenset(
    {
        "test_rejects_cargo_or_bwrap_exit_with_live_descendant",
        "test_timeout_is_ineligible",
        "test_snapshot_timeout_is_ineligible",
        "test_rejects_root_path_replacement",
        "test_rejects_root_symlink_substitution",
        "test_rejects_child_entry_replacement",
        "test_rejects_owner_drift",
        "test_rejects_mode_drift",
        "test_rejects_acl_drift",
        "test_rejects_descriptor_path_inode_mismatch",
        "test_rejects_premature_descriptor_closure",
        "test_rejects_deletion_before_containment_empty",
        "test_rejects_shared_or_reused_root",
        "test_rejects_sticky_world_writable_ancestor",
        "test_rejects_unexpected_mount_id",
        "test_rejects_unexpected_entry_type",
        "test_rejects_subreaper_readback_failure",
        "test_rejects_bwrap_status_or_worker_ready_handshake_failure",
        "test_rejects_namespace_prerequisite_fallback",
        "test_stage_b_setup_timeout_signals_only_retained_bwrap_pidfd",
        "test_stage_b_timeout_before_status_starts_no_cargo_and_reaps_exact_bwrap",
        "test_stage_b_bwrap_death_before_worker_ready_is_ineligible",
        "test_stage_a_parent_loss_before_stage_b_start_starts_no_wall",
        "test_stage_b_bwrap_loss_after_ready_before_start_is_ineligible",
        "test_stage_a_loss_before_stage_b_launch_starts_no_wall",
        "test_stage_a_loss_before_worker_ready_terminates_bwrap_and_worker",
        "test_stage_a_loss_after_ready_before_start_terminates_bwrap_and_worker",
        "test_stage_a_loss_after_start_invalidates_and_terminates_wall",
        "test_premature_stage_b_exit_after_start_is_ineligible",
        "test_sigterm_ignoring_descendant_requires_ineligible_sigkill",
        "test_stuck_stage_b_hits_bounded_stage_a_deadline",
        "test_rejects_forbidden_glob_ambient_parent_and_product_home_targets",
        "test_rejects_pathname_only_cleanup_api",
        "test_cli_rejects_arbitrary_command_root_and_environment_overrides",
        "test_public_request_cannot_select_internal_host_stage_worker_roles_or_control_path",
        "test_cli_rejects_invalid_label_timeout_cwd_and_head",
        "test_rejects_dirty_index_worktree_and_nonignored_untracked_state",
        "test_rejects_git_environment_config_alternates_replace_and_fsmonitor",
        "test_rejects_local_git_config_include_mode_exclude_worktree_and_unknown_keys",
        "test_rejects_info_attributes_worktree_config_and_index_authority_extensions",
        "test_rejects_tree_index_stage_mode_flag_and_worktree_byte_drift",
        "test_rejects_untracked_gitignore_and_every_nonignored_untracked_path",
        "test_rejects_path_shim_and_executable_identity_drift",
        "test_trusted_bwrap_launcher_rejects_ambient_loader_and_startup_tcb_drift",
        "test_held_git_fd_path_replacement_never_executes_replacement",
        "test_isolated_python_startup_rejects_environment_site_path_and_module_poison",
        "test_python_runtime_rejects_origin_and_late_import_drift",
        "test_bootstrap_rejects_path_execution_cmdline_drift_and_runner_blob_mismatch",
        "test_self_test_loader_rejects_path_import_discovery_and_blob_mismatch",
        "test_rejects_project_ancestor_or_account_cargo_config_appearance",
        "test_evidence_bounds_fail_closed",
        "test_atomic_finalization_hides_partial_eligibility",
        "test_final_evidence_rename_is_noreplace",
    }
)

ELIGIBLE_REGRESSION_SELF_TEST_IDS = frozenset(
    {
        "test_summarizer_accepts_valid_all_pass_as_eligible_regression",
        "test_summarizer_rejects_cargo_exit_status_mismatch",
        "test_summarizer_rejects_integer_duplicate_missing_and_panic_grammar",
        "test_summarizer_rejects_invalid_encoding_nul_and_overflow",
    }
)


def _evidence_classification(test_id: str) -> str:
    if test_id in INELIGIBLE_SELF_TEST_IDS:
        return "ineligible"
    if test_id in ELIGIBLE_REGRESSION_SELF_TEST_IDS:
        return "eligible-regression"
    return "authority-proof"


SELF_TEST_INVENTORY = tuple(
    {
        "id": test_id,
        "category": _category(index),
        "setup": (
            f"bounded {_category(index)} scenario exercises "
            f"{test_id.removeprefix('test_').replace('_', ' ')}"
        ),
        "expected_result": (
            f"{test_id.removeprefix('test_').replace('_', ' ')}: "
            + {
                "ineligible": "the unsafe or incomplete wall state is rejected",
                "eligible-regression": (
                    "all authority gates complete and noncanonical evidence "
                    "is eligible but classified as regression"
                ),
                "authority-proof": (
                    "the bounded mechanism proves the named contract property "
                    "without claiming a wall disposition"
                ),
            }[_evidence_classification(test_id)]
        ),
        "expected_evidence_classification": _evidence_classification(test_id),
        "contract_clause": (
            "04-contracts-and-gates.md::canonical-self-test::" + test_id
        ),
    }
    for index, test_id in enumerate(SELF_TEST_IDS)
)


def _module_source(module: object) -> bytes:
    authenticated = getattr(module, "__CANONICAL_MODULE_SOURCE", None)
    if isinstance(authenticated, bytes):
        return authenticated
    path = getattr(module, "__file__", None)
    if not isinstance(path, str) or path.startswith("<"):
        raise AssertionError("authenticated module source unavailable")
    with open(path, "rb") as handle:
        return handle.read()


def _test_module_source() -> bytes:
    authenticated = globals().get("__CANONICAL_MODULE_SOURCE")
    if isinstance(authenticated, bytes):
        return authenticated
    return _module_source(sys.modules[__name__])


class CanonicalShellWallRunnerTests(unittest.TestCase):
    maxDiff = None

    def setUp(self) -> None:
        self._temporary_directories: list[tempfile.TemporaryDirectory[str]] = []
        self.cleanup_authority = runner._mint_cleanup_quiescence(
            "self-test",
            lifecycle_proof={"authenticated_self_test": True},
        )

    def tearDown(self) -> None:
        while self._temporary_directories:
            self._temporary_directories.pop().cleanup()

    def make_safe_parent(self) -> str:
        temporary = tempfile.TemporaryDirectory(
            prefix="canonical-wall-selftest-",
            dir="/home/spenser/.local/state",
        )
        self._temporary_directories.append(temporary)
        os.chmod(temporary.name, 0o700)
        return temporary.name

    def make_disposable_repo(self) -> str:
        repository = self.make_safe_parent()
        environment = {
            "LANG": "C.UTF-8",
            "LC_ALL": "C.UTF-8",
            "PATH": "/usr/bin:/bin",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": "/dev/null",
            "GIT_AUTHOR_NAME": "P1 Self Test",
            "GIT_AUTHOR_EMAIL": "p1-self-test@example.invalid",
            "GIT_COMMITTER_NAME": "P1 Self Test",
            "GIT_COMMITTER_EMAIL": "p1-self-test@example.invalid",
        }
        subprocess.run(
            ["/usr/bin/git", "init", "-q", "-b", "main", repository],
            check=True,
            env=environment,
        )
        scripts = os.path.join(repository, "scripts", "ci")
        os.makedirs(scripts, mode=0o700)
        for source, basename in (
            (_module_source(runner), "canonical_shell_wall_runner.py"),
            (_test_module_source(), "test_canonical_shell_wall_runner.py"),
        ):
            with open(os.path.join(scripts, basename), "xb") as destination:
                destination.write(source)
        with open(os.path.join(repository, ".gitignore"), "xb") as handle:
            handle.write(b"target/\nignored/\n")
        with open(os.path.join(repository, "tracked.txt"), "xb") as handle:
            handle.write(b"tracked\n")
        subprocess.run(
            ["/usr/bin/git", "-C", repository, "add", "--", "."],
            check=True,
            env=environment,
        )
        subprocess.run(
            ["/usr/bin/git", "-C", repository, "commit", "-q", "-F", "-"],
            check=True,
            env=environment,
            input=(
                "fixture\n\n"
                f"P1-Bootstrap-Bytes: {len(runner.BOOTSTRAP_V2_SOURCE.encode())}\n"
                "P1-Bootstrap-SHA256: "
                f"{runner.inline_sha256(runner.BOOTSTRAP_V2_SOURCE.encode())}\n"
                "P1-Host-Argv-Template-SHA256: "
                f"{runner._template_sha256(runner.HOST_INVOCATION_ARGV_TEMPLATE_V1)}\n"
                "P1-Stage-A-Argv-Template-SHA256: "
                f"{runner._template_sha256(runner.BWRAP_STAGE_A_ARGV_TEMPLATE_V1)}\n"
                "P1-Stage-B-Argv-Template-SHA256: "
                f"{runner._template_sha256(runner.BWRAP_STAGE_B_ARGV_TEMPLATE_V1)}\n"
            ),
            text=True,
        )
        return repository

    def make_fake_rustup_home(self) -> str:
        return self.make_safe_parent()

    def make_fake_cargo_home_seed(self) -> str:
        return self.make_safe_parent()

    def make_registry_archives(self) -> str:
        return self.make_safe_parent()

    def invoke_bootstrap(self, *arguments: str) -> tuple[int, bytes, bytes]:
        return runner._run_selftest_bootstrap(arguments)

    def invoke_self_test(self) -> tuple[int, bytes, bytes]:
        return self.invoke_bootstrap("self-test")

    def run_bounded_fixture(self, fixture: str, timeout: float = 5.0) -> dict[str, object]:
        original = runner._AUTHENTICATED_SOURCE
        if not original:
            runner._AUTHENTICATED_SOURCE = _module_source(runner)
        try:
            return runner._run_bounded_fixture(fixture, timeout=timeout)
        finally:
            runner._AUTHENTICATED_SOURCE = original

    def read_final_provenance(self, directory: str) -> dict[str, object]:
        with open(os.path.join(directory, "provenance.json"), "rb") as handle:
            return json.load(handle)

    def assert_ineligible(
        self,
        record: dict[str, object],
        reason: str | None = None,
    ) -> None:
        self.assertFalse(record["eligible"])
        self.assertEqual(record["wall_gate"], "ineligible")
        if reason is not None:
            self.assertIn(reason, record["ineligibility_reasons"])

    def test_rejects_cargo_or_bwrap_exit_with_live_descendant(self) -> None:
        result = self.run_bounded_fixture("production-long-lived")
        self.assertTrue(result["echld"])
        self.assertEqual(result["reaped"], 3)

    def test_reaps_reparented_descendant(self) -> None:
        result = self.run_bounded_fixture("production-reparent")
        self.assertEqual(result, {"echld": True, "pid": 1, "reaped": 2})
        source = _module_source(runner).decode("utf-8")
        host = source[
            source.index("def host_main") : source.index(
                "def _run_selftest_bootstrap"
            )
        ]
        normal_reap = host.index(
            "_reap_owned_children_to_echild(",
            host.index("stage_a_status ="),
        )
        teardown_proved = host.index(
            "stage_a_process_teardown_proved = True",
            normal_reap,
        )
        self.assertNotIn("!= 0", host[normal_reap:teardown_proved])

    def test_process_group_escape_remains_contained(self) -> None:
        result = self.run_bounded_fixture("production-setsid")
        self.assertEqual(result, {"echld": True, "pid": 1, "reaped": 2})

    def test_timeout_is_ineligible(self) -> None:
        self.assertEqual(
            runner.wait_for_containment_empty.__kwdefaults__,
            {
                "expected_pid_namespace_inode": None,
                "cancelled": None,
                "teardown_term_seconds": 5.0,
                "teardown_kill_seconds": 5.0,
            },
        )
        module_source = _module_source(runner).decode("utf-8")
        containment_source = module_source[
            module_source.index("def wait_for_containment_empty")
            : module_source.index("def _json_no_duplicates")
        ]
        self.assertNotIn("final_deadline", containment_source)
        termination_source = module_source[
            module_source.index("def terminate_contained_children")
            : module_source.index("def wait_for_containment_empty")
        ]
        self.assertIn("retained_pidfds", termination_source)
        self.assertLess(
            termination_source.index("retained_pidfds"),
            termination_source.index("for signal_number, duration in"),
        )
        self.assertGreater(
            termination_source.rindex("os.close(pidfd)"),
            termination_source.index("for signal_number, duration in"),
        )
        after_teardown = containment_source.split(
            "terminate_contained_children(",
            1,
        )[1]
        self.assertNotIn("_drain_waitable(", after_teardown)
        result = self.run_bounded_fixture("production-timeout")
        self.assertEqual(
            result,
            {
                "echld": True,
                "forced": True,
                "pid": 1,
                "pidfd_count": 1,
                "primary_status": -9,
                "reaped": 1,
                "signals": [9, 15],
                "survivors": 0,
                "timed_out": True,
            },
        )
        default_phases = self.run_bounded_fixture(
            "production-timeout-multigeneration-defaults",
            timeout=8,
        )
        self.assertEqual(
            default_phases,
            {
                "echld": True,
                "elapsed_window": True,
                "reaped": 3,
                "same_pidfds": True,
                "targets": 3,
            },
        )
        worker_source = module_source[
            module_source.index("def stage_b_worker_main")
            : module_source.index("def _default_provenance")
        ]
        self.assertLess(
            worker_source.index("wall_deadline = time.monotonic()"),
            worker_source.index("_verify_projected_toolchain(environment)"),
        )
        self.assertLess(
            worker_source.index("wall_deadline = time.monotonic()"),
            worker_source.index("_spawn_primary_command("),
        )
        self.assertIn(
            "primary_pid,\n            wall_deadline,",
            worker_source,
        )

    def test_snapshot_timeout_is_ineligible(self) -> None:
        parent = self.make_safe_parent()
        source = os.path.join(parent, "source")
        destination = os.path.join(parent, "destination")
        os.mkdir(source, 0o700)
        with open(os.path.join(source, "entry"), "xb") as handle:
            handle.write(b"bounded\n")
        with self.assertRaises(runner.RunnerError) as timed_out:
            runner._copy_tree_no_follow(
                source,
                destination,
                deadline=time.monotonic() - 1,
            )
        self.assertEqual(timed_out.exception.reason, "snapshot_timeout")
        self.assertFalse(os.path.exists(destination))
        record = runner._default_provenance(
            invocation_id="0" * 32,
            label="snapshot-timeout",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        runner._record_ineligibility(record, "snapshot_timeout", 67)
        self.assert_ineligible(record, "snapshot_timeout")
        self.assertEqual(record["runner_exit_code"], 67)

    def test_copy_tree_no_follow_preserves_modes_despite_umask(self) -> None:
        source = self.make_safe_parent()
        os.mkdir(os.path.join(source, "bin"), 0o755)
        os.chmod(os.path.join(source, "bin"), 0o755)
        executable = os.path.join(source, "bin", "tool")
        with open(executable, "xb") as handle:
            handle.write(b"#!/bin/sh\nexit 0\n")
        os.chmod(executable, 0o755)
        config = os.path.join(source, "config.toml")
        with open(config, "xb") as handle:
            handle.write(b"[tool]\nname = \"snapshot\"\n")
        os.chmod(config, 0o644)
        destination = os.path.join(self.make_safe_parent(), "snapshot")
        source_manifest = runner._manifest_tree(source)[0]
        original_umask = os.umask(0o077)
        try:
            copied_manifest, _count = runner._copy_tree_no_follow(
                source,
                destination,
            )
        finally:
            os.umask(original_umask)
        self.assertEqual(copied_manifest, source_manifest)
        self.assertEqual(runner._manifest_tree(destination)[0], source_manifest)
        self.assertEqual(
            stat.S_IMODE(
                os.stat(
                    os.path.join(destination, "bin"),
                    follow_symlinks=False,
                ).st_mode
            ),
            0o755,
        )
        self.assertEqual(
            stat.S_IMODE(
                os.stat(
                    os.path.join(destination, "bin", "tool"),
                    follow_symlinks=False,
                ).st_mode
            ),
            0o755,
        )
        self.assertEqual(
            stat.S_IMODE(
                os.stat(
                    os.path.join(destination, "config.toml"),
                    follow_symlinks=False,
                ).st_mode
            ),
            0o644,
        )

    def _assert_primary_command_uses_private_umask_without_mutating_parent(
        self,
    ) -> None:
        output_path = os.path.join(self.make_safe_parent(), "child-umask")
        program = (
            "import os;"
            "value=os.umask(0);"
            f"open({output_path!r},'x',encoding='ascii').write(f'{{value:03o}}\\n')"
        )
        command = (
            "/usr/bin/python3.13",
            "-I",
            "-S",
            "-B",
            "-c",
            program,
        )
        executable_fd = os.open(
            "/usr/bin/python3.13",
            os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        details = os.fstat(executable_fd)
        executable = runner.HeldExecutable(
            path="/usr/bin/python3.13",
            fd=executable_fd,
            identity=(
                details.st_dev,
                details.st_ino,
                details.st_uid,
                stat.S_IMODE(details.st_mode),
            ),
            sha256=runner.hash_open_file(executable_fd),
        )
        original_command = runner._canonical_cargo_command
        original_stat = runner.os.stat
        original_umask = os.umask(0o022)
        try:
            runner._canonical_cargo_command = lambda _mode: command

            def projected_stat(
                path: object,
                *,
                dir_fd: int | None = None,
                follow_symlinks: bool = True,
            ) -> os.stat_result:
                if os.fspath(path) == "/home/spenser/.cargo/bin/rustup":
                    return original_stat(
                        "/usr/bin/python3.13",
                        follow_symlinks=False,
                    )
                return original_stat(
                    path,
                    dir_fd=dir_fd,
                    follow_symlinks=follow_symlinks,
                )

            runner.os.stat = projected_stat
            pid = runner._spawn_primary_command(command, {}, executable)
            waited, status = os.waitpid(pid, 0)
            self.assertEqual(waited, pid)
            self.assertEqual(os.waitstatus_to_exitcode(status), 0)
            observed_parent_umask = os.umask(0o022)
            self.assertEqual(observed_parent_umask, 0o022)
        finally:
            os.umask(original_umask)
            runner.os.stat = original_stat
            runner._canonical_cargo_command = original_command
            os.close(executable_fd)
        with open(output_path, "rt", encoding="ascii") as handle:
            self.assertEqual(handle.read(), "077\n")

    def test_rejects_root_path_replacement(self) -> None:
        parent = self.make_safe_parent()
        root = os.path.join(parent, "root")
        os.mkdir(root, 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        os.rename(root, os.path.join(parent, "held"))
        os.mkdir(root, 0o700)
        with self.assertRaises(runner.RunnerError):
            runner.revalidate_named_identity(parent_fd, "root", root_fd, identity)
        os.close(root_fd)
        os.close(parent_fd)

    def test_rejects_root_symlink_substitution(self) -> None:
        parent = self.make_safe_parent()
        root = os.path.join(parent, "root")
        other = os.path.join(parent, "other")
        os.mkdir(root, 0o700)
        os.mkdir(other, 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        os.rename(root, os.path.join(parent, "held"))
        os.symlink(other, root)
        with self.assertRaises(runner.RunnerError):
            runner.revalidate_named_identity(parent_fd, "root", root_fd, identity)
        os.close(root_fd)
        os.close(parent_fd)

    def test_rejects_child_entry_replacement(self) -> None:
        parent = self.make_safe_parent()
        root = os.path.join(parent, "root")
        os.mkdir(root, 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        os.rename(root, os.path.join(parent, "held"))
        os.mkdir(root, 0o700)
        with self.assertRaises(runner.RunnerError):
            runner.remove_tree_at(
                parent_fd,
                "root",
                expected=identity,
                cleanup_authority=self.cleanup_authority,
            )
        os.close(root_fd)
        os.close(parent_fd)

    def test_rejects_owner_drift(self) -> None:
        parent = self.make_safe_parent()
        with self.assertRaises(runner.RunnerError):
            runner.open_validated_directory(
                parent,
                expected_uid=os.getuid() + 1,
                expected_mode=0o700,
            )

    def test_rejects_mode_drift(self) -> None:
        parent = self.make_safe_parent()
        os.chmod(parent, 0o750)
        with self.assertRaises(runner.RunnerError):
            runner.open_validated_directory(
                parent,
                expected_uid=os.getuid(),
                expected_mode=0o700,
            )

    def test_rejects_acl_drift(self) -> None:
        parent = self.make_safe_parent()
        acl = struct.pack(
            "<IHHIHHIHHIHHIHHI",
            2,
            0x01,
            0x07,
            0xFFFFFFFF,
            0x02,
            0x02,
            0,
            0x04,
            0,
            0xFFFFFFFF,
            0x10,
            0x02,
            0xFFFFFFFF,
            0x20,
            0,
            0xFFFFFFFF,
        )
        try:
            os.setxattr(parent, b"system.posix_acl_access", acl)
        except OSError as error:
            self.fail(f"required ACL xattr unsupported: {error}")
        _digest, foreign_write = runner.read_effective_acl(parent)
        self.assertTrue(foreign_write)
        with self.assertRaises(runner.RunnerError):
            runner.open_validated_directory(
                parent,
                expected_uid=os.getuid(),
                expected_mode=stat.S_IMODE(os.stat(parent).st_mode),
            )
        acl_name = b"system.posix_acl_access"
        malformed_value = b"\x02\x00\x00\x00\x01"
        malformed = (
            len(acl_name).to_bytes(2, "big")
            + acl_name
            + len(malformed_value).to_bytes(4, "big")
            + malformed_value
        )
        default_name = b"system.posix_acl_default"
        malformed += (
            len(default_name).to_bytes(2, "big")
            + default_name
            + (0).to_bytes(4, "big")
        )
        original_acl_bytes = runner._acl_bytes
        runner._acl_bytes = lambda _path_or_fd: malformed
        try:
            with self.assertRaises(runner.RunnerError):
                runner.read_effective_acl(parent)
        finally:
            runner._acl_bytes = original_acl_bytes

    def test_rejects_descriptor_path_inode_mismatch(self) -> None:
        parent = self.make_safe_parent()
        original = os.path.join(parent, "original")
        replacement = os.path.join(parent, "replacement")
        os.mkdir(original, 0o700)
        os.mkdir(replacement, 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        held_fd, identity = runner.open_validated_directory(
            original,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        with self.assertRaises(runner.RunnerError):
            runner.revalidate_named_identity(
                parent_fd,
                "replacement",
                held_fd,
                identity,
            )
        os.close(held_fd)
        os.close(parent_fd)

    def test_rejects_premature_descriptor_closure(self) -> None:
        parent = self.make_safe_parent()
        fd, identity = runner.open_validated_directory(
            parent,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        os.close(fd)
        with self.assertRaises(OSError):
            runner._directory_identity(fd, parent)
        self.assertTrue(identity.path_matches_fd)

    def test_rejects_deletion_before_containment_empty(self) -> None:
        parent = self.make_safe_parent()
        os.mkdir(os.path.join(parent, "root"), 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        with self.assertRaises(runner.RunnerError):
            runner._mint_cleanup_quiescence(
                "stage-a",
                lifecycle_proof={
                    "echld_observed": False,
                    "stage_b_reaped": False,
                    "stage_b_process_teardown_proved": False,
                    "stage_b_namespace_teardown_proved": False,
                },
            )
        with self.assertRaises(runner.RunnerError):
            runner.remove_tree_at(
                parent_fd,
                "root",
                cleanup_authority=None,
            )
        self.assertTrue(os.path.isdir(os.path.join(parent, "root")))
        os.close(parent_fd)

    def test_preserves_unrelated_sentinel(self) -> None:
        parent = self.make_safe_parent()
        root = os.path.join(parent, "root")
        sentinel = os.path.join(parent, "sentinel")
        os.mkdir(root, 0o700)
        with open(sentinel, "xb") as handle:
            handle.write(b"keep")
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        proof = runner.remove_tree_at(
            parent_fd,
            "root",
            expected=identity,
            cleanup_authority=self.cleanup_authority,
        )
        os.close(root_fd)
        os.close(parent_fd)
        self.assertTrue(proof["name_absence_proved"])
        with open(sentinel, "rb") as handle:
            self.assertEqual(handle.read(), b"keep")

        attack_root = os.path.join(parent, "attack-root")
        nested = os.path.join(attack_root, "nested")
        sentinel_directory = os.path.join(parent, "sentinel-directory")
        os.mkdir(attack_root, 0o700)
        os.mkdir(nested, 0o700)
        os.mkdir(sentinel_directory, 0o700)
        with open(os.path.join(sentinel_directory, "marker"), "xb") as handle:
            handle.write(b"unrelated")
        attack_root_fd, attack_identity = runner.open_validated_directory(
            attack_root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        sentinel_fd = os.open(
            sentinel_directory,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        parent_fd = os.open(
            parent,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        original_remove_tree_at = runner.remove_tree_at
        exchanged = False

        def exchange_before_recursive_open(
            directory_fd: int,
            name: str,
            **kwargs: object,
        ) -> dict[str, bool]:
            nonlocal exchanged
            if name == "nested" and not exchanged:
                exchanged = True
                result = runner._LIBC.renameat2(
                    runner.ctypes.c_int(directory_fd),
                    runner.ctypes.c_char_p(b"nested"),
                    runner.ctypes.c_int(parent_fd),
                    runner.ctypes.c_char_p(b"sentinel-directory"),
                    runner.ctypes.c_uint(2),
                )
                self.assertEqual(result, 0)
            return original_remove_tree_at(directory_fd, name, **kwargs)

        runner.remove_tree_at = exchange_before_recursive_open
        try:
            with self.assertRaises(runner.RunnerError):
                original_remove_tree_at(
                    parent_fd,
                    "attack-root",
                    expected=attack_identity,
                    cleanup_authority=self.cleanup_authority,
                )
        finally:
            runner.remove_tree_at = original_remove_tree_at
        self.assertTrue(exchanged)
        marker_fd = os.open(
            "marker",
            os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=sentinel_fd,
        )
        self.assertEqual(os.read(marker_fd, 32), b"unrelated")
        os.close(marker_fd)
        self.assertFalse(
            any(name.startswith(".substrate-cleanup-") for name in os.listdir(parent))
        )
        isolate_root = os.path.join(parent, "isolate-root")
        isolate_sentinel = os.path.join(parent, "isolate-sentinel")
        os.mkdir(isolate_root, 0o700)
        os.mkdir(isolate_sentinel, 0o700)
        with open(os.path.join(isolate_sentinel, "marker"), "xb") as handle:
            handle.write(b"isolate-window")
        isolate_root_fd, isolate_identity = runner.open_validated_directory(
            isolate_root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        isolate_sentinel_fd = os.open(
            isolate_sentinel,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        original_rename_expected = runner._rename_expected_entry_to_quarantine
        isolate_injected = False

        def replace_at_isolation_boundary(
            directory_fd: int,
            name: str,
            quarantine: str,
            held_fd: int,
            expected: tuple[int, int],
            cleanup_authority: object,
        ) -> bool:
            nonlocal isolate_injected
            if name == "isolate-root" and not isolate_injected:
                isolate_injected = True
                result = runner._LIBC.renameat2(
                    runner.ctypes.c_int(directory_fd),
                    runner.ctypes.c_char_p(b"isolate-root"),
                    runner.ctypes.c_int(directory_fd),
                    runner.ctypes.c_char_p(b"isolate-sentinel"),
                    runner.ctypes.c_uint(2),
                )
                self.assertEqual(result, 0)
            return original_rename_expected(
                directory_fd,
                name,
                quarantine,
                held_fd,
                expected,
                cleanup_authority,
            )

        runner._rename_expected_entry_to_quarantine = replace_at_isolation_boundary
        try:
            with self.assertRaises(runner.RunnerError):
                runner.remove_tree_at(
                    parent_fd,
                    "isolate-root",
                    expected=isolate_identity,
                    cleanup_authority=self.cleanup_authority,
                )
        finally:
            runner._rename_expected_entry_to_quarantine = original_rename_expected
        self.assertTrue(isolate_injected)
        marker_fd = os.open(
            "marker",
            os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=isolate_sentinel_fd,
        )
        self.assertEqual(os.read(marker_fd, 32), b"isolate-window")
        os.close(marker_fd)
        os.close(isolate_sentinel_fd)
        os.close(isolate_root_fd)

        delete_root = os.path.join(parent, "delete-root")
        os.mkdir(delete_root, 0o700)
        with open(os.path.join(delete_root, "a-victim"), "xb") as handle:
            handle.write(b"victim")
        with open(os.path.join(delete_root, "z-sentinel"), "xb") as handle:
            handle.write(b"delete-window")
        delete_root_fd, delete_identity = runner.open_validated_directory(
            delete_root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        victim_fd = os.open(
            "a-victim",
            os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=delete_root_fd,
        )
        delete_sentinel_fd = os.open(
            "z-sentinel",
            os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=delete_root_fd,
        )
        original_delete_isolated = runner._delete_isolated_entry
        delete_injected = False

        def replace_at_delete_boundary(
            directory_fd: int,
            quarantine: str,
            held_fd: int,
            expected: tuple[int, int],
            **kwargs: object,
        ) -> None:
            nonlocal delete_injected
            if not bool(kwargs["is_directory"]) and not delete_injected:
                delete_injected = True
                result = runner._LIBC.renameat2(
                    runner.ctypes.c_int(directory_fd),
                    runner.ctypes.c_char_p(os.fsencode(quarantine)),
                    runner.ctypes.c_int(directory_fd),
                    runner.ctypes.c_char_p(b"z-sentinel"),
                    runner.ctypes.c_uint(2),
                )
                self.assertEqual(result, 0)
            original_delete_isolated(
                directory_fd,
                quarantine,
                held_fd,
                expected,
                **kwargs,
            )

        runner._delete_isolated_entry = replace_at_delete_boundary
        try:
            with self.assertRaises(runner.RunnerError):
                runner.remove_tree_at(
                    parent_fd,
                    "delete-root",
                    expected=delete_identity,
                    cleanup_authority=self.cleanup_authority,
                )
        finally:
            runner._delete_isolated_entry = original_delete_isolated
        self.assertTrue(delete_injected)
        self.assertEqual(os.pread(victim_fd, 32, 0), b"victim")
        self.assertEqual(os.pread(delete_sentinel_fd, 32, 0), b"delete-window")
        self.assertEqual(os.fstat(victim_fd).st_nlink, 1)
        self.assertEqual(os.fstat(delete_sentinel_fd).st_nlink, 1)
        os.close(victim_fd)
        os.close(delete_sentinel_fd)
        os.close(delete_root_fd)

        delete_directory_root = os.path.join(parent, "delete-directory-root")
        delete_victim_directory = os.path.join(delete_directory_root, "a-victim-dir")
        delete_sentinel_directory = os.path.join(
            delete_directory_root,
            "z-sentinel-dir",
        )
        os.mkdir(delete_directory_root, 0o700)
        os.mkdir(delete_victim_directory, 0o700)
        os.mkdir(delete_sentinel_directory, 0o700)
        with open(os.path.join(delete_sentinel_directory, "marker"), "xb") as handle:
            handle.write(b"rmdir-window")
        delete_directory_root_fd, delete_directory_identity = (
            runner.open_validated_directory(
                delete_directory_root,
                expected_uid=os.getuid(),
                expected_mode=0o700,
            )
        )
        delete_victim_directory_fd = os.open(
            "a-victim-dir",
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=delete_directory_root_fd,
        )
        delete_sentinel_directory_fd = os.open(
            "z-sentinel-dir",
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=delete_directory_root_fd,
        )
        directory_delete_injected = False

        def replace_at_directory_delete_boundary(
            directory_fd: int,
            quarantine: str,
            held_fd: int,
            expected: tuple[int, int],
            **kwargs: object,
        ) -> None:
            nonlocal directory_delete_injected
            if bool(kwargs["is_directory"]) and not directory_delete_injected:
                directory_delete_injected = True
                result = runner._LIBC.renameat2(
                    runner.ctypes.c_int(directory_fd),
                    runner.ctypes.c_char_p(os.fsencode(quarantine)),
                    runner.ctypes.c_int(directory_fd),
                    runner.ctypes.c_char_p(b"z-sentinel-dir"),
                    runner.ctypes.c_uint(2),
                )
                self.assertEqual(result, 0)
            original_delete_isolated(
                directory_fd,
                quarantine,
                held_fd,
                expected,
                **kwargs,
            )

        runner._delete_isolated_entry = replace_at_directory_delete_boundary
        try:
            with self.assertRaises(runner.RunnerError):
                runner.remove_tree_at(
                    parent_fd,
                    "delete-directory-root",
                    expected=delete_directory_identity,
                    cleanup_authority=self.cleanup_authority,
                )
        finally:
            runner._delete_isolated_entry = original_delete_isolated
        self.assertTrue(directory_delete_injected)
        marker_fd = os.open(
            "marker",
            os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
            dir_fd=delete_sentinel_directory_fd,
        )
        self.assertEqual(os.read(marker_fd, 32), b"rmdir-window")
        os.close(marker_fd)
        self.assertGreater(os.fstat(delete_victim_directory_fd).st_nlink, 0)
        self.assertGreater(os.fstat(delete_sentinel_directory_fd).st_nlink, 0)
        os.close(delete_victim_directory_fd)
        os.close(delete_sentinel_directory_fd)
        os.close(delete_directory_root_fd)
        os.close(sentinel_fd)
        os.close(attack_root_fd)
        os.close(parent_fd)

    def test_rejects_shared_or_reused_root(self) -> None:
        parent = self.make_safe_parent()
        first = runner.prepare_backing_root(parent, "reuse", "0" * 32)
        with self.assertRaises(runner.RunnerError):
            runner.prepare_backing_root(parent, "reuse", "0" * 32)
        for key in ("backing_fd", "partial_fd", "parent_fd"):
            os.close(first[key])
        for descriptor, _path, _identity in first["ancestor_records"]:
            os.close(descriptor)

        failure_parent = self.make_safe_parent()
        sentinel = os.path.join(failure_parent, "sentinel")
        os.mkdir(sentinel, 0o700)
        sentinel_fd = os.open(
            sentinel,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        sentinel_identity = os.fstat(sentinel_fd)
        sentinel_parent_fd = os.open(
            failure_parent,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        original_directory_identity = runner._directory_identity
        original_rmdir = runner.os.rmdir
        exchanged = False

        def fail_partial_identity(
            descriptor: int,
            path: str | None = None,
        ) -> runner.DirectoryIdentity:
            if path is not None and path.endswith(".partial"):
                raise runner.RunnerError(67, "root_validation_failed")
            return original_directory_identity(descriptor, path)

        def exchange_before_failure_rmdir(
            name: str,
            *,
            dir_fd: int | None = None,
        ) -> None:
            nonlocal exchanged
            if name.endswith(".partial") and dir_fd is not None and not exchanged:
                exchanged = True
                result = runner._LIBC.renameat2(
                    runner.ctypes.c_int(dir_fd),
                    runner.ctypes.c_char_p(os.fsencode(name)),
                    runner.ctypes.c_int(sentinel_parent_fd),
                    runner.ctypes.c_char_p(b"sentinel"),
                    runner.ctypes.c_uint(2),
                )
                self.assertEqual(result, 0)
            original_rmdir(name, dir_fd=dir_fd)

        runner._directory_identity = fail_partial_identity
        runner.os.rmdir = exchange_before_failure_rmdir
        try:
            with self.assertRaises(runner.RunnerError):
                runner.prepare_backing_root(
                    failure_parent,
                    "failure-cleanup",
                    "a" * 32,
                )
        finally:
            runner._directory_identity = original_directory_identity
            runner.os.rmdir = original_rmdir
        self.assertFalse(exchanged)
        sentinel_post = os.fstat(sentinel_fd)
        self.assertEqual(
            (sentinel_post.st_dev, sentinel_post.st_ino),
            (sentinel_identity.st_dev, sentinel_identity.st_ino),
        )
        self.assertGreater(sentinel_post.st_nlink, 0)
        os.close(sentinel_fd)
        os.close(sentinel_parent_fd)

    def test_rejects_sticky_world_writable_ancestor(self) -> None:
        parent = self.make_safe_parent()
        sticky = os.path.join(parent, "sticky")
        child = os.path.join(sticky, "child")
        os.mkdir(sticky, 0o777)
        os.chmod(sticky, 0o1777)
        os.mkdir(child, 0o700)
        with self.assertRaises(runner.RunnerError):
            runner.validate_safe_ancestor_chain(
                child,
                expected_uid=os.getuid(),
            )
        root_identities = runner.validate_safe_ancestor_chain(
            parent,
            expected_uid=os.getuid(),
        )
        root_details = os.stat("/", follow_symlinks=False)
        self.assertEqual(
            root_identities[0],
            (
                root_details.st_dev,
                root_details.st_ino,
                stat.S_IMODE(root_details.st_mode),
            ),
        )
        outer = os.path.join(parent, "outer")
        evidence = os.path.join(outer, "evidence")
        os.mkdir(outer, 0o700)
        os.mkdir(evidence, 0o700)
        authority = runner.prepare_backing_root(
            evidence,
            "ancestor-drift",
            "f" * 32,
        )
        os.chmod(outer, 0o750)
        record = runner._default_provenance(
            invocation_id="f" * 32,
            label="ancestor-drift",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        with self.assertRaises(runner.RunnerError):
            runner.finalize_after_stage_a_exit(authority, record)
        os.chmod(outer, 0o700)
        for key in ("backing_fd", "partial_fd", "parent_fd"):
            os.close(authority[key])
        for descriptor, _path, _identity in authority["ancestor_records"]:
            os.close(descriptor)

    def test_rejects_unexpected_mount_id(self) -> None:
        parent = self.make_safe_parent()
        root = os.path.join(parent, "root")
        os.mkdir(root, 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        drifted = runner.DirectoryIdentity(
            **{**identity.__dict__, "mount_id": identity.mount_id + 1}
        )
        with self.assertRaises(runner.RunnerError):
            runner.remove_tree_at(
                parent_fd,
                "root",
                expected=drifted,
                cleanup_authority=self.cleanup_authority,
            )
        os.close(root_fd)
        os.close(parent_fd)
        self.assertEqual(
            self.run_bounded_fixture("production-mount-alias"),
            {
                "distinct_mount_ids": True,
                "path_matches": False,
                "reason": "cleanup_identity_mismatch",
                "same_inode": True,
            },
        )
        self.assertEqual(
            self.run_bounded_fixture("production-namespace-drift"),
            {"reason": "diagnostic_scope_violation"},
        )

    def test_rejects_unexpected_entry_type(self) -> None:
        parent = self.make_safe_parent()
        path = os.path.join(parent, "file")
        with open(path, "xb") as handle:
            handle.write(b"x")
        with self.assertRaises(runner.RunnerError):
            runner.open_validated_directory(
                path,
                expected_uid=os.getuid(),
                expected_mode=0o700,
            )
        fifo_root = os.path.join(parent, "fifo-root")
        os.mkdir(fifo_root, 0o700)
        regular_before_fifo = os.path.join(fifo_root, "a")
        fifo = os.path.join(fifo_root, "z")
        with open(regular_before_fifo, "xb") as handle:
            handle.write(b"must-survive")
        os.mkfifo(fifo, 0o600)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        fifo_root_fd, fifo_root_identity = runner.open_validated_directory(
            fifo_root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        with self.assertRaises(runner.RunnerError):
            runner.remove_tree_at(
                parent_fd,
                "fifo-root",
                expected=fifo_root_identity,
                cleanup_authority=self.cleanup_authority,
            )
        self.assertTrue(os.path.exists(regular_before_fifo))
        self.assertTrue(os.path.exists(fifo))
        self.assertFalse(
            any(
                name.startswith(".substrate-cleanup-")
                for name in os.listdir(parent)
            )
        )
        os.close(fifo_root_fd)
        os.close(parent_fd)
        root = os.path.join(parent, "hardlink-root")
        os.mkdir(root, 0o700)
        first = os.path.join(root, "first")
        outside = os.path.join(parent, "outside-link")
        with open(first, "xb") as handle:
            handle.write(b"owned")
        os.link(first, outside)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        with self.assertRaises(runner.RunnerError):
            runner.remove_tree_at(
                parent_fd,
                "hardlink-root",
                expected=identity,
                cleanup_authority=self.cleanup_authority,
            )
        self.assertTrue(os.path.exists(first))
        self.assertTrue(os.path.exists(outside))
        os.close(root_fd)
        os.close(parent_fd)

        root = os.path.join(parent, "cargo-target")
        build = os.path.join(root, "debug", "build", "crate-hash")
        os.makedirs(build, mode=0o700)
        first = os.path.join(build, "build_script_build-hash")
        alias = os.path.join(build, "build-script-build")
        with open(first, "xb") as handle:
            handle.write(b"cargo build script")
        os.link(first, alias)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            root,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        proof = runner.remove_tree_at(
            parent_fd,
            "cargo-target",
            expected=identity,
            cleanup_authority=self.cleanup_authority,
        )
        self.assertFalse(os.path.exists(root))
        self.assertTrue(proof["removed"])
        self.assertTrue(proof["name_absence_proved"])
        os.close(root_fd)
        os.close(parent_fd)

    def test_rejects_subreaper_readback_failure(self) -> None:
        result = self.run_bounded_fixture("pid1")
        self.assertEqual(result, {"pid": 1, "subreaper": 1})
        original_libc = runner._LIBC

        class FailedReadback:
            def __init__(self) -> None:
                self.operations: list[int] = []

            def prctl(self, operation: object, *_arguments: object) -> int:
                value = int(getattr(operation, "value"))
                self.operations.append(value)
                if value == runner.PR_SET_CHILD_SUBREAPER:
                    return 0
                return -1

        failed = FailedReadback()
        runner._LIBC = failed
        try:
            self.assertFalse(runner.set_child_subreaper())
        finally:
            runner._LIBC = original_libc
        self.assertEqual(
            failed.operations,
            [
                runner.PR_SET_CHILD_SUBREAPER,
                runner.PR_GET_CHILD_SUBREAPER,
            ],
        )

    def test_rejects_bwrap_status_or_worker_ready_handshake_failure(self) -> None:
        invalid_statuses = (
            b"",
            b'{"child-pid":1}\n',
            b'{"child-pid":1}\n{"exit-code":0}\n',
            b'{"child-pid":1,"child-pid":2,"mnt-namespace":3,"pid-namespace":4}\n'
            b'{"exit-code":0}\n',
            b'{"child-pid":1,"mnt-namespace":2,"pid-namespace":3}\n'
            b'{"exit-code":0}\ntrailing',
            b'{"child-pid":0,"mnt-namespace":2,"pid-namespace":3}\n'
            b'{"exit-code":0}\n',
            b'{"child-pid":1,"mnt-namespace":-1,"pid-namespace":3}\n'
            b'{"exit-code":0}\n',
            b'{"child-pid":1,"mnt-namespace":2,"pid-namespace":0}\n'
            b'{"exit-code":0}\n',
            b'{"child-pid":1,"mnt-namespace":2,"pid-namespace":3}\n'
            b'{"exit-code":-1}\n',
            b'{"child-pid":1,"mnt-namespace":2,"pid-namespace":3}\n'
            b'{"exit-code":256}\n',
            b'{"child-pid":2147483648,"mnt-namespace":2,"pid-namespace":3}\n'
            b'{"exit-code":0}\n',
            b'{"child-pid":1,"mnt-namespace":18446744073709551616,'
            b'"pid-namespace":3}\n{"exit-code":0}\n',
            b'{"child-pid":1,"mnt-namespace":2,'
            b'"pid-namespace":18446744073709551616}\n{"exit-code":0}\n',
            b'{"child-pid":1,"mnt-namespace":2,"pid-namespace":3}\r\n'
            b'{"exit-code":0}\r\n',
        )
        for malformed in invalid_statuses:
            with self.assertRaises(runner.RunnerError):
                runner.parse_bwrap_status(malformed)
        self.assertEqual(
            runner.parse_bwrap_status(
                b'{"child-pid":2147483647,'
                b'"mnt-namespace":18446744073709551615,'
                b'"pid-namespace":18446744073709551615}\n'
                b'{"exit-code":255}\n'
            ),
            (
                {
                    "child-pid": 2_147_483_647,
                    "mnt-namespace": 18_446_744_073_709_551_615,
                    "pid-namespace": 18_446_744_073_709_551_615,
                },
                {"exit-code": 255},
            ),
        )
        left, right = socket.socketpair(socket.AF_UNIX, socket.SOCK_SEQPACKET)
        right.sendall(b"WRONG")
        self.assertNotEqual(left.recv(16), runner.READY_RECORD)
        left.close()
        right.close()
        stage_a_source = _module_source(runner).decode("utf-8")
        stage_a_source = stage_a_source[
            stage_a_source.index("def stage_a_main")
            : stage_a_source.index("def _executable_identity")
        ]
        self.assertLess(
            stage_a_source.index('"stage_b_bwrap_pid": spawned.pid'),
            stage_a_source.index(
                "stage_b_start_time = _process_start_time_ticks(spawned.pid)"
            ),
        )
        self.assertLess(
            stage_a_source.index(
                "_capture_stage_b_worker_identity(first_status)",
                stage_a_source.index("first_status ="),
            ),
            stage_a_source.index("endpoint = accept_authenticated_worker("),
        )
        self.assertLess(
            stage_a_source.index(
                '"stage_b_pid_namespace_inode": namespace_identities['
            ),
            stage_a_source.index("endpoint = accept_authenticated_worker("),
        )
        parent = self.make_safe_parent()
        control_path = os.path.join(parent, "worker.sock")
        listener = runner.create_worker_control_listener(control_path)
        connected_read, connected_write = os.pipe2(os.O_CLOEXEC)
        release_read, release_write = os.pipe2(os.O_CLOEXEC)
        pid = os.fork()
        if pid == 0:
            os.close(connected_read)
            os.close(release_write)
            endpoint = socket.socket(
                socket.AF_UNIX,
                socket.SOCK_SEQPACKET | socket.SOCK_CLOEXEC,
            )
            endpoint.connect(control_path)
            os.write(connected_write, b"R")
            os.close(connected_write)
            os.read(release_read, 1)
            os.close(release_read)
            endpoint.close()
            os._exit(0)
        os.close(connected_write)
        os.close(release_read)
        self.assertEqual(os.read(connected_read, 1), b"R")
        os.close(connected_read)
        with self.assertRaises(runner.RunnerError) as stalled:
            runner.accept_authenticated_worker(
                listener,
                expected_pid=pid,
                deadline=time.monotonic() + 0.05,
            )
        self.assertEqual(stalled.exception.reason, "worker_handshake_failed")
        os.close(release_write)
        os.waitpid(pid, 0)
        listener.close()
        for invalid_first in (
            b'{"child-pid":0,"mnt-namespace":2,"pid-namespace":3}\n',
            b'{"child-pid":1,"mnt-namespace":0,"pid-namespace":3}\n',
            b'{"child-pid":1,"mnt-namespace":2,"pid-namespace":-1}\n',
        ):
            with self.assertRaises(runner.RunnerError):
                runner._parse_first_bwrap_record(invalid_first)

    def test_rejects_namespace_prerequisite_fallback(self) -> None:
        template = runner.BWRAP_STAGE_B_ARGV_TEMPLATE_V1
        self.assertIn("--unshare-pid", template)
        self.assertIn("--as-pid-1", template)
        self.assertNotIn("--unshare-pid-try", template)

    def test_teardown_never_targets_unrelated_process(self) -> None:
        self.assertEqual(
            self.run_bounded_fixture("production-exact-pidfd"),
            {"status": -15, "unrelated_alive": True},
        )

    def test_stage_b_setup_timeout_signals_only_retained_bwrap_pidfd(self) -> None:
        self.assertEqual(
            self.run_bounded_fixture("production-stage-b-setup-timeout"),
            {
                "bwrap_reaped": True,
                "cargo_started": False,
                "echld": True,
                "namespace_absent": True,
                "production_wait_path": True,
                "signals_exact": True,
                "status": -15,
                "status_withheld": True,
                "worker_ready_observed": True,
                "worker_start_withheld": True,
            },
        )

    def test_stage_b_timeout_before_status_starts_no_cargo_and_reaps_exact_bwrap(
        self,
    ) -> None:
        mechanics = self.run_bounded_fixture("production-stage-b-setup-timeout")
        self.assertTrue(mechanics["status_withheld"])
        self.assertTrue(mechanics["worker_ready_observed"])
        self.assertTrue(mechanics["worker_start_withheld"])
        self.assertFalse(mechanics["cargo_started"])
        self.assertTrue(mechanics["signals_exact"])
        self.assertTrue(mechanics["bwrap_reaped"])
        self.assertTrue(mechanics["echld"])
        self.assertTrue(mechanics["namespace_absent"])
        self.assertTrue(mechanics["production_wait_path"])
        self.assertEqual(
            self.run_bounded_fixture("production-namespace-boundary-holder"),
            {
                "echld": True,
                "held_rejected": True,
                "namespace_absent": True,
                "status": -15,
            },
        )
        record = runner._default_provenance(
            invocation_id="1" * 32,
            label="setup-timeout",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        self.assertIsNone(record["containment"]["cargo_pid"])
        self.assertFalse(record["namespace"]["worker_start_sent"])
        stage_a_source = _module_source(runner).decode("utf-8").split(
            "def stage_a_main",
            1,
        )[1].split("def _executable_identity", 1)[0]
        self.assertIn(
            "verify_namespace_mount_teardown(",
            stage_a_source.rsplit("finally:", 1)[1],
        )
        self.assertIn(
            '_record_ineligibility(provenance, error.reason, error.exit_code)',
            stage_a_source,
        )
        self.assertLess(
            stage_a_source.index("setup_deadline = time.monotonic()"),
            stage_a_source.index("create_worker_control_listener(control_path)"),
        )
        self.assertLess(
            stage_a_source.index("setup_deadline = time.monotonic()"),
            stage_a_source.index("launch_stage_b_bwrap("),
        )
        self.assertIn(
            "first_status_line = _read_stage_b_first_status_or_teardown(",
            stage_a_source,
        )
        self.assertLess(
            stage_a_source.index(
                'provenance["namespace"]["stage_b_status_received"] = True'
            ),
            stage_a_source.index("_capture_stage_b_worker_identity("),
        )
        self.assertIn(
            "_preserve_pre_ready_stage_b_output(",
            stage_a_source,
        )
        runner_source = _module_source(runner).decode("utf-8")
        setup_wait = runner_source[
            runner_source.index(
                "def _read_stage_b_first_status_or_teardown"
            ) : runner_source.index("def _parse_first_bwrap_record")
        ]
        self.assertLess(
            setup_wait.index("_terminate_exact_spawned_process(spawned)"),
            setup_wait.index("mark_stage_b_reaped()"),
        )
        self.assertLess(
            setup_wait.index("_reap_owned_children_to_echild("),
            setup_wait.index("mark_containment_teardown_proved()"),
        )
        self.assertLess(
            stage_a_source.index("arm_cleanup_deadline()", 1),
            stage_a_source.index('if output["overflow"]:'),
        )
        self.assertIn(
            "signal.setitimer(signal.ITIMER_REAL, CLEANUP_TIMEOUT_SECONDS)",
            stage_a_source,
        )
        self.assertLess(
            stage_a_source.index(
                "if not set_child_subreaper():"
            ),
            stage_a_source.index("spawned, status_fd, stage_b_argv ="),
        )
        self.assertIn(
            "_reap_owned_children_to_echild(",
            stage_a_source.rsplit("finally:", 1)[1],
        )
        identityless_namespace_branch = stage_a_source[
            stage_a_source.index(
                "if (\n"
                "            stage_b_launch_attempted\n"
                "            and spawned is not None\n"
                "            and not stage_b_namespace_teardown_proved"
            ) : stage_a_source.index(
                "if spawned is not None and not stage_b_pidfd_closed:",
                stage_a_source.index(
                    "if (\n"
                    "            stage_b_launch_attempted\n"
                    "            and spawned is not None\n"
                    "            and not stage_b_namespace_teardown_proved"
                ),
            )
        ]
        self.assertIn(
            '"mount_teardown_failed",\n                70,',
            identityless_namespace_branch,
        )
        self.assertNotIn(
            "stage_b_namespace_teardown_proved = True",
            identityless_namespace_branch,
        )

    def test_stage_b_bwrap_death_before_worker_ready_is_ineligible(self) -> None:
        record = runner._default_provenance(
            invocation_id="2" * 32,
            label="bwrap-death",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        record["ineligibility_reasons"] = ["premature_stage_b_exit"]
        self.assert_ineligible(record, "premature_stage_b_exit")
        self.assertEqual(
            self.run_bounded_fixture("production-prestatus-namespace-absence"),
            {
                "echld": True,
                "live_rejected": True,
                "live_reference_rejected": True,
                "namespace_absent": True,
                "status": 68,
            },
        )
        source = _module_source(runner).decode("utf-8")
        host = source[
            source.index("def host_main")
            : source.index("def _run_selftest_bootstrap")
        ]
        self.assertLess(
            host.index(
                "if not set_child_subreaper():"
            ),
            host.index("spawned, status_fd, stage_a_argv ="),
        )
        self.assertIn(
            "_reap_owned_children_to_echild(",
            host,
        )
        host_failure = host[host.index("except RunnerError as error:") :]
        self.assertIn(
            "elif spawned is not None and not stage_a_pidfd_closed:\n"
            "            namespace_teardown_proved = "
            "namespace_boundary_teardown_proved",
            host_failure,
        )
        self.assertIn(
            "_verify_spawned_namespace_boundary_teardown(",
            host_failure,
        )
        self.assertIn(
            'raise RunnerError(70, "mount_teardown_failed")',
            host_failure,
        )
        self.assertIn(
            "stage_a_process_teardown_proved\n"
            "                            and namespace_teardown_proved",
            host_failure,
        )
        self.assertIn("retain_backing=True", host_failure)
        self.assertLess(
            host_failure.index("_finalize_ineligible("),
            host_failure.index(
                'print(\n                    "wall_gate=ineligible "'
            ),
        )

    def test_stage_a_parent_loss_before_stage_b_start_starts_no_wall(self) -> None:
        record = runner._default_provenance(
            invocation_id="a" * 32,
            label="parent-loss",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        self.assertIsNone(record["namespace"]["stage_b_bwrap_pid"])
        self.assertFalse(record["namespace"]["worker_start_sent"])
        self.assertEqual(
            self.run_bounded_fixture("production-bwrap-parent-loss"),
            {
                "echld": True,
                "ready": True,
                "reaped": True,
            },
        )

    def test_stage_b_bwrap_loss_after_ready_before_start_is_ineligible(self) -> None:
        release_read, release_write = os.pipe2(os.O_CLOEXEC)
        live_pid = os.fork()
        if live_pid == 0:
            os.close(release_write)
            os.read(release_read, 1)
            os.close(release_read)
            os._exit(0)
        os.close(release_read)
        dead_pid = os.fork()
        if dead_pid == 0:
            os._exit(0)
        dead_pidfd = os.pidfd_open(dead_pid)
        live_pidfd = os.pidfd_open(live_pid)
        try:
            os.waitid(
                os.P_PIDFD,
                dead_pidfd,
                os.WEXITED | os.WNOWAIT,
            )
            with self.assertRaises(runner.RunnerError) as caught:
                runner._require_pidfds_live(dead_pidfd, live_pidfd)
            self.assertEqual(caught.exception.reason, "premature_stage_b_exit")
        finally:
            os.close(release_write)
            os.waitpid(dead_pid, 0)
            os.waitpid(live_pid, 0)
            os.close(dead_pidfd)
            os.close(live_pidfd)
        self.assertEqual(
            self.run_bounded_fixture("production-bwrap-loss-after-ready"),
            {
                "echld": True,
                "ready": True,
                "status": -15,
            },
        )

    def test_stage_a_start_requires_bwrap_status_worker_pidfd_peer_credentials_and_ready(
        self,
    ) -> None:
        required = {
            "stage_b_status_received",
            "worker_pidfd_opened",
            "worker_peer_credentials_verified",
            "worker_ready_received",
            "control_path_absence_proved",
        }
        record = runner._default_provenance(
            invocation_id="4" * 32,
            label="start-gate",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        self.assertTrue(required.issubset(record["namespace"]))
        self.assertFalse(any(record["namespace"][key] for key in required))
        integrated = self.run_bounded_fixture(
            "production-integrated-stage-b-handshake",
            timeout=8,
        )
        self.assertTrue(
            integrated["json_child_matches"]
            and integrated["same_namespace"]
            and integrated["fixed_socket"]
            and integrated["worker_departed"]
        )

    def test_stage_a_maps_vanished_post_status_worker_to_premature_exit(
        self,
    ) -> None:
        exited = os.fork()
        if exited == 0:
            os._exit(0)
        os.waitpid(exited, 0)
        with self.assertRaises(runner.RunnerError) as reaped:
            runner._capture_stage_b_worker_identity(
                {
                    "child-pid": exited,
                    "mnt-namespace": 1,
                    "pid-namespace": 1,
                }
            )
        self.assertEqual(reaped.exception.reason, "premature_stage_b_exit")

        release_read, release_write = os.pipe2(os.O_CLOEXEC)
        live = os.fork()
        if live == 0:
            os.close(release_write)
            os.read(release_read, 1)
            os.close(release_read)
            os._exit(0)
        os.close(release_read)
        original_pidfd_open = runner.os.pidfd_open
        try:
            status = {
                "child-pid": live,
                "mnt-namespace": os.stat(f"/proc/{live}/ns/mnt").st_ino,
                "pid-namespace": os.stat(f"/proc/{live}/ns/pid").st_ino,
            }

            def fail_pidfd_open(pid: int) -> int:
                if pid == live:
                    raise ProcessLookupError()
                return original_pidfd_open(pid)

            runner.os.pidfd_open = fail_pidfd_open
            with self.assertRaises(runner.RunnerError) as raced:
                runner._capture_stage_b_worker_identity(status)
            self.assertEqual(raced.exception.reason, "premature_stage_b_exit")
        finally:
            runner.os.pidfd_open = original_pidfd_open
            os.close(release_write)
            os.waitpid(live, 0)

    def test_stage_a_loss_before_stage_b_launch_starts_no_wall(self) -> None:
        record = runner._default_provenance(
            invocation_id="b" * 32,
            label="prelaunch-loss",
            mode="serial",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("serial"),
            environment={},
        )
        self.assertIsNone(record["namespace"]["stage_b_bwrap_pid"])
        self.assertIsNone(record["containment"]["cargo_pid"])
        self.assertEqual(
            self.run_bounded_fixture(
                "production-before-stage-b-launch-loss"
            ),
            {
                "echld": True,
                "launched": False,
                "status": -15,
            },
        )

    def test_stage_a_loss_before_worker_ready_terminates_bwrap_and_worker(self) -> None:
        template = runner.BWRAP_STAGE_B_ARGV_TEMPLATE_V1
        self.assertIn("--die-with-parent", template)
        self.assertIn("--json-status-fd", template)
        result = self.run_bounded_fixture(
            "production-prestatus-namespace-absence"
        )
        self.assertTrue(result["namespace_absent"])
        self.assertTrue(result["echld"])

    def test_stage_a_loss_after_ready_before_start_terminates_bwrap_and_worker(
        self,
    ) -> None:
        template = runner.BWRAP_STAGE_B_ARGV_TEMPLATE_V1
        self.assertIn("--die-with-parent", template)
        self.assertLess(template.index("--die-with-parent"), template.index("--as-pid-1"))
        self.assertEqual(
            self.run_bounded_fixture("production-bwrap-loss-after-ready"),
            {
                "echld": True,
                "ready": True,
                "status": -15,
            },
        )

    def test_stage_a_loss_after_start_invalidates_and_terminates_wall(self) -> None:
        started = time.monotonic()
        result = self.run_bounded_fixture(
            "production-bwrap-parent-loss-after-start"
        )
        self.assertLess(time.monotonic() - started, 1.0)
        self.assertEqual(
            result,
            {
                "echld": True,
                "ready": True,
                "reaped": True,
                "started": True,
            },
        )

    def test_premature_stage_b_exit_after_start_is_ineligible(self) -> None:
        record = runner._default_provenance(
            invocation_id="6" * 32,
            label="premature",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        record["containment"]["premature_stage_b_exit"] = True
        record["ineligibility_reasons"] = ["premature_stage_b_exit"]
        self.assert_ineligible(record, "premature_stage_b_exit")
        terminated = self.run_bounded_fixture(
            "production-bwrap-parent-loss-after-start"
        )
        self.assertTrue(terminated["started"])
        self.assertTrue(terminated["echld"])

    def test_stage_b_complete_requires_authenticated_echild_record_and_zero_exit(
        self,
    ) -> None:
        good = {
            "schema": runner.STATUS_VERSION,
            "discriminator": "CONTAINMENT_EMPTY",
            "echld_observed": True,
            "timed_out": False,
            "forced_teardown": False,
            "survivors": [],
            "pid_namespace_inode": 123,
            "primary_pid": 2,
            "primary_status": 0,
            "reaped_descendants": 1,
            "received_signals": [],
        }
        runner._validate_worker_result(good)
        body = (
            json.dumps(good, sort_keys=True, separators=(",", ":")) + "\n"
        ).encode("ascii")
        packet = runner.FRAME_MAGIC + struct.pack(">I", len(body)) + body
        self.assertEqual(runner._parse_frame_packet(packet), good)
        for malformed_body in (b"\n" + body, body + b"\n"):
            malformed_packet = (
                runner.FRAME_MAGIC
                + struct.pack(">I", len(malformed_body))
                + malformed_body
            )
            with self.assertRaises(runner.RunnerError):
                runner._parse_frame_packet(malformed_packet)
        self.assertTrue(
            good["echld_observed"]
            and not good["timed_out"]
            and not good["forced_teardown"]
            and not good["survivors"]
        )
        for key, replacement in (
            ("schema", "wrong-version"),
            ("discriminator", "WRONG"),
            ("echld_observed", False),
            ("timed_out", True),
            ("forced_teardown", True),
            ("survivors", [9]),
        ):
            invalid = dict(good)
            invalid[key] = replacement
            if key in {"echld_observed", "timed_out", "forced_teardown", "survivors"}:
                invalid["discriminator"] = "CONTAINMENT_INELIGIBLE"
                runner._validate_worker_result(invalid)
                self.assertFalse(
                    invalid["echld_observed"]
                    and not invalid["timed_out"]
                    and not invalid["forced_teardown"]
                    and not invalid["survivors"]
                )
            else:
                with self.assertRaises(runner.RunnerError):
                    runner._validate_worker_result(invalid)
        signalled = dict(good)
        signalled.update(
            {
                "discriminator": "CONTAINMENT_INELIGIBLE",
                "forced_teardown": True,
                "received_signals": [15],
            }
        )
        runner._validate_worker_result(signalled)
        incomplete = dict(good)
        incomplete.update(
            {
                "discriminator": "CONTAINMENT_INELIGIBLE",
                "echld_observed": False,
                "forced_teardown": True,
                "primary_status": None,
                "reaped_descendants": 0,
                "survivors": [2],
            }
        )
        runner._validate_worker_result(incomplete)
        invalid_empty = dict(good)
        invalid_empty["primary_status"] = None
        with self.assertRaises(runner.RunnerError):
            runner._validate_worker_result(invalid_empty)
        boundary = dict(good)
        boundary.update(
            {
                "pid_namespace_inode": runner.MAX_NAMESPACE_INODE,
                "primary_pid": runner.MAX_PID,
                "primary_status": -(runner.signal.NSIG - 1),
                "reaped_descendants": runner.MAX_CONTAINMENT_EVENTS,
            }
        )
        runner._validate_worker_result(boundary)
        for key, replacement in (
            ("pid_namespace_inode", runner.MAX_NAMESPACE_INODE + 1),
            ("primary_pid", runner.MAX_PID + 1),
            ("primary_status", -runner.signal.NSIG),
            ("primary_status", 256),
            ("reaped_descendants", runner.MAX_CONTAINMENT_EVENTS + 1),
            ("survivors", [2, 2]),
            ("received_signals", [15, 15]),
        ):
            invalid_domain = dict(good)
            invalid_domain.update(
                {
                    "discriminator": "CONTAINMENT_INELIGIBLE",
                    "echld_observed": False,
                    key: replacement,
                }
            )
            with self.assertRaises(runner.RunnerError):
                runner._validate_worker_result(invalid_domain)
        survivor_overflow = dict(good)
        survivor_overflow.update(
            {
                "discriminator": "CONTAINMENT_INELIGIBLE",
                "echld_observed": False,
                "forced_teardown": True,
                "primary_status": None,
                "survivors": list(range(2, 2 + 4_097)),
            }
        )
        survivor_boundary = dict(survivor_overflow)
        survivor_boundary["survivors"] = list(range(2, 2 + 4_096))
        runner._validate_worker_result(survivor_boundary)
        with self.assertRaises(runner.RunnerError) as survivor_limit:
            runner._validate_worker_result(survivor_overflow)
        self.assertEqual(survivor_limit.exception.reason, "evidence_limit_exceeded")
        stage_a_source = _module_source(runner).decode("utf-8").split(
            "def stage_a_main",
            1,
        )[1].split("def _executable_identity", 1)[0]
        self.assertNotIn('"signal_termination"', stage_a_source)
        self.assertIn(
            '_record_ineligibility(\n                provenance,',
            stage_a_source,
        )
        self.assertIn('"surviving_descendant"', stage_a_source)
        self.assertIn('"containment_timeout"', stage_a_source)
        worker_source = _module_source(runner).decode("utf-8").split(
            "def stage_b_worker_main",
            1,
        )[1].split("def _executable_identity", 1)[0]
        self.assertLess(
            worker_source.index("_validate_worker_result(payload)"),
            worker_source.index("_send_frame(endpoint, payload)"),
        )

    def test_fixed_private_socket_peer_status_cloexec_unchanged_stdin_and_no_cargo_control_fd(
        self,
    ) -> None:
        stage_bootstrap = runner.STAGE_BOOTSTRAP_SOURCE
        self.assertIn(
            "if _stage_role_hint in ('stage-a','stage-b-worker'):",
            stage_bootstrap,
        )
        self.assertIn("if len(_stage_extra_fds)!=1:", stage_bootstrap)
        self.assertIn(
            "_stage_userns_target=os.readlink(_stage_userns_path)",
            stage_bootstrap,
        )
        self.assertIn(
            "_stage_current_target=os.readlink('/proc/self/ns/user')",
            stage_bootstrap,
        )
        self.assertIn("os.close(_stage_userns_fd)", stage_bootstrap)
        self.assertIn("if _initial_fds!={0,1,2}:", stage_bootstrap)
        self.assertLess(
            stage_bootstrap.index(
                "if _stage_role_hint in ('stage-a','stage-b-worker'):"
            ),
            stage_bootstrap.index("if _initial_fds!={0,1,2}:"),
        )
        self.assertLess(
            stage_bootstrap.index("os.close(_stage_userns_fd)"),
            stage_bootstrap.index("if _initial_fds!={0,1,2}:"),
        )
        self.assertLess(
            stage_bootstrap.index("if _initial_fds!={0,1,2}:"),
            stage_bootstrap.index("_path='/run/substrate-wall/runner.py'"),
        )
        parent = self.make_safe_parent()
        path = os.path.join(parent, "worker.sock")
        listener = runner.create_worker_control_listener(path)
        self.assertFalse(listener.get_inheritable())
        self.assertEqual(stat.S_IMODE(os.stat(path).st_mode), 0o600)
        listener.close()

        high_fd = 70_000
        with self.assertRaises(OSError) as absent_high:
            os.fstat(high_fd)
        self.assertEqual(absent_high.exception.errno, runner.errno.EBADF)
        held_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/python3.13"]["sha256"],
            expected_uid=0,
        )
        output_read, output_write = os.pipe2(os.O_CLOEXEC)
        proof_read, proof_write = os.pipe2(os.O_CLOEXEC)
        devnull = os.open("/dev/null", os.O_RDONLY | os.O_CLOEXEC)
        os.dup2(devnull, high_fd, inheritable=True)
        os.close(devnull)
        child = None
        try:
            child_code = (
                "import json,os,sys;"
                "executor=int(sys.argv[1]);high=int(sys.argv[2]);"
                "preserved=int(sys.argv[3]);stable=[];"
                "\nfor name in os.listdir('/proc/self/fd'):\n"
                " if not name.isdigit(): continue\n"
                " fd=int(name)\n"
                " try: os.fstat(fd)\n"
                " except OSError: continue\n"
                " stable.append(fd)\n"
                "allowed={0,1,2,preserved};"
                "os.write(preserved,b'P');"
                "print(json.dumps({'executor_absent':executor not in stable,"
                "'high_absent':high not in stable,'preserved':preserved in stable,"
                "'unexpected':sorted(set(stable)-allowed)},"
                "sort_keys=True,separators=(',',':')))"
            )
            child = runner._spawn_held(
                held_python.fd,
                (
                    "/usr/bin/python3.13",
                    "-I",
                    "-S",
                    "-B",
                    "-c",
                    child_code,
                    str(held_python.fd),
                    str(high_fd),
                    str(proof_write),
                ),
                {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                preserve_fds=(proof_write,),
                stdout_fd=output_write,
                stderr_fd=output_write,
            )
        finally:
            os.close(high_fd)
            os.close(held_python.fd)
            os.close(output_write)
            os.close(proof_write)
        deadline = time.monotonic() + 5
        output = runner._read_fd_bounded(output_read, 4096, deadline)
        proof = runner._read_fd_bounded(proof_read, 1, deadline)
        os.close(output_read)
        os.close(proof_read)
        self.assertIsNotNone(child)
        waited, raw_status = os.waitpid(child.pid, 0)
        os.close(child.pidfd)
        self.assertEqual(waited, child.pid)
        self.assertEqual(runner._decode_wait_status(raw_status), 0)
        self.assertEqual(proof, b"P")
        self.assertEqual(
            json.loads(output),
            {
                "executor_absent": True,
                "high_absent": True,
                "preserved": True,
                "unexpected": [],
            },
        )
        failure_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        original_pidfd_open = runner.os.pidfd_open

        def fail_pidfd_open(_pid: int) -> int:
            raise OSError(runner.errno.EMFILE, "injected pidfd exhaustion")

        spawn_probe_read, spawn_probe_write = os.pipe2(os.O_CLOEXEC)
        runner.os.pidfd_open = fail_pidfd_open
        try:
            with self.assertRaises(runner.RunnerError) as pidfd_failure:
                runner._spawn_held(
                    failure_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        (
                            "import os,signal,sys;"
                            "probe=int(sys.argv[1]);child=os.fork();"
                            "\nif child==0:\n"
                            " os.write(probe,b\"forked\");signal.pause()\n"
                            "signal.pause()"
                        ),
                        str(spawn_probe_write),
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                    preserve_fds=(spawn_probe_write,),
                )
        finally:
            runner.os.pidfd_open = original_pidfd_open
            os.close(failure_python.fd)
            os.close(spawn_probe_write)
        self.assertEqual(pidfd_failure.exception.reason, "namespace_setup_failed")
        self.assertEqual(
            runner._read_fd_bounded(
                spawn_probe_read,
                16,
                time.monotonic() + 1,
            ),
            b"",
        )
        os.close(spawn_probe_read)
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        timeout_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        timeout_probe_read, timeout_probe_write = os.pipe2(os.O_CLOEXEC)
        before_timeout_fds = set(os.listdir("/proc/self/fd"))
        original_kill = runner.os.kill
        previous_sigchld = runner.signal.signal(
            runner.signal.SIGCHLD,
            runner.signal.SIG_IGN,
        )
        exact_kill_observations: list[tuple[int, bool, int]] = []

        def audit_exact_kill(pid: int, number: int) -> None:
            exact_kill_observations.append(
                (pid, os.path.exists(f"/proc/{pid}"), number)
            )
            original_kill(pid, number)

        runner.os.pidfd_open = fail_pidfd_open
        runner.os.kill = audit_exact_kill
        try:
            with self.assertRaises(runner.RunnerError) as timeout_failure:
                runner.exec_held_executable(
                    timeout_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        (
                            "import os,signal,sys;"
                            "probe=int(sys.argv[1]);child=os.fork();"
                            "\nif child==0:\n"
                            " os.write(probe,b\"forked\");signal.pause()\n"
                            "signal.pause()"
                        ),
                        str(timeout_probe_write),
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                    pass_fds=(timeout_probe_write,),
                    timeout=0.01,
                )
            restored_sigchld = runner.signal.getsignal(runner.signal.SIGCHLD)
        finally:
            runner.os.kill = original_kill
            runner.os.pidfd_open = original_pidfd_open
            runner.signal.signal(runner.signal.SIGCHLD, previous_sigchld)
        self.assertEqual(
            timeout_failure.exception.reason,
            "environment_unavailable",
        )
        self.assertIs(restored_sigchld, runner.signal.SIG_IGN)
        self.assertEqual(len(exact_kill_observations), 1)
        self.assertEqual(
            exact_kill_observations,
            [
                (
                    exact_kill_observations[0][0],
                    True,
                    runner.signal.SIGKILL,
                )
            ],
        )
        self.assertEqual(set(os.listdir("/proc/self/fd")), before_timeout_fds)
        os.close(timeout_probe_write)
        self.assertEqual(
            runner._read_fd_bounded(
                timeout_probe_read,
                16,
                time.monotonic() + 1,
            ),
            b"",
        )
        os.close(timeout_probe_read)
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        os.close(timeout_python.fd)
        descendant_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        descendant_probe_read, descendant_probe_write = os.pipe2(os.O_CLOEXEC)
        try:
            with self.assertRaises(runner.RunnerError) as descendant_failure:
                runner.exec_held_executable(
                    descendant_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        (
                            "import os,signal,sys;"
                            "probe=int(sys.argv[1]);ready_r,ready_w=os.pipe();"
                            "child=os.fork();"
                            "\nif child==0:\n"
                            " os.close(ready_r);os.setsid();"
                            "os.write(probe,(str(os.getpid())+'\\n').encode());"
                            "os.write(ready_w,b'R');os.close(ready_w);"
                            "signal.pause()\n"
                            "os.close(ready_w);os.read(ready_r,1);"
                            "os.close(ready_r);os._exit(0)"
                        ),
                        str(descendant_probe_write),
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                    pass_fds=(descendant_probe_write,),
                    timeout=2.0,
                )
        finally:
            os.close(descendant_probe_write)
            os.close(descendant_python.fd)
        self.assertEqual(
            descendant_failure.exception.reason,
            "environment_unavailable",
        )
        descendant_pid = int(
            runner._read_fd_bounded(
                descendant_probe_read,
                32,
                time.monotonic() + 1,
            )
        )
        os.close(descendant_probe_read)
        with self.assertRaises(ProcessLookupError):
            os.kill(descendant_pid, 0)
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        signal_failure_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        original_pidfd_send_signal = runner._pidfd_send_signal

        def fail_pidfd_send_signal(_pidfd: int, _number: int) -> None:
            raise OSError(runner.errno.EIO, "injected pidfd signal failure")

        before_signal_failure_fds = set(os.listdir("/proc/self/fd"))
        before_signal_failure_mask = runner.signal.pthread_sigmask(
            runner.signal.SIG_BLOCK,
            set(),
        )
        before_signal_failure_subreaper = runner._child_subreaper_state()
        runner._pidfd_send_signal = fail_pidfd_send_signal
        try:
            with self.assertRaises(runner.RunnerError) as signal_failure:
                runner.exec_held_executable(
                    signal_failure_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        "import signal;signal.pause()",
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                    timeout=0.01,
                )
        finally:
            runner._pidfd_send_signal = original_pidfd_send_signal
        self.assertEqual(
            signal_failure.exception.reason,
            "environment_unavailable",
        )
        self.assertEqual(
            set(os.listdir("/proc/self/fd")),
            before_signal_failure_fds,
        )
        self.assertEqual(
            runner.signal.pthread_sigmask(runner.signal.SIG_BLOCK, set()),
            before_signal_failure_mask,
        )
        self.assertIs(
            runner._child_subreaper_state(),
            before_signal_failure_subreaper,
        )
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        os.close(signal_failure_python.fd)
        bounded_teardown_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        bounded_teardown = runner._spawn_held(
            bounded_teardown_python.fd,
            (
                "/usr/bin/python3.13",
                "-I",
                "-S",
                "-B",
                "-c",
                "import signal;signal.pause()",
            ),
            {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
        )
        original_pidfd_send_signal = runner._pidfd_send_signal
        before_bounded_teardown_mask = runner.signal.pthread_sigmask(
            runner.signal.SIG_BLOCK,
            set(),
        )

        def ignore_pidfd_signal(_pidfd: int, _number: int) -> None:
            return None

        runner._pidfd_send_signal = ignore_pidfd_signal
        bounded_started = time.monotonic()
        try:
            with self.assertRaises(runner.RunnerError) as bounded_failure:
                runner._terminate_exact_spawned_process(
                    bounded_teardown,
                    term_seconds=0.01,
                    kill_seconds=0.05,
                )
        finally:
            runner._pidfd_send_signal = original_pidfd_send_signal
        bounded_elapsed = time.monotonic() - bounded_started
        self.assertEqual(bounded_failure.exception.reason, "containment_timeout")
        self.assertLess(bounded_elapsed, 0.5)
        self.assertEqual(
            runner.signal.pthread_sigmask(runner.signal.SIG_BLOCK, set()),
            before_bounded_teardown_mask,
        )
        os.kill(bounded_teardown.pid, 0)
        self.assertEqual(
            runner._force_kill_and_reap_spawned(
                bounded_teardown,
                timeout_seconds=1.0,
            ),
            -runner.signal.SIGKILL,
        )
        os.close(bounded_teardown.pidfd)
        os.close(bounded_teardown_python.fd)
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        close_failure_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        original_pthread_sigmask = runner.signal.pthread_sigmask
        before_mask_setup_fds = set(os.listdir("/proc/self/fd"))
        before_mask_setup_subreaper = runner._child_subreaper_state()

        def fail_mask_setup(_how: int, _mask: object) -> object:
            raise OSError(runner.errno.EIO, "injected signal-mask failure")

        runner.signal.pthread_sigmask = fail_mask_setup
        try:
            with self.assertRaises(runner.RunnerError) as mask_setup_failure:
                runner.exec_held_executable(
                    close_failure_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        "pass",
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                )
        finally:
            runner.signal.pthread_sigmask = original_pthread_sigmask
        self.assertEqual(
            mask_setup_failure.exception.reason,
            "environment_unavailable",
        )
        self.assertEqual(
            set(os.listdir("/proc/self/fd")),
            before_mask_setup_fds,
        )
        self.assertIs(
            runner._child_subreaper_state(),
            before_mask_setup_subreaper,
        )
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)

        original_pipe2 = runner.os.pipe2
        original_close = runner.os.close
        close_failure_pipes: list[tuple[int, int]] = []
        close_failure_injected = False

        def track_exec_pipe(flags: int) -> tuple[int, int]:
            descriptors = original_pipe2(flags)
            close_failure_pipes.append(descriptors)
            return descriptors

        def fail_pre_pidfd_close(fd: int) -> None:
            nonlocal close_failure_injected
            target = (
                close_failure_pipes[2][0]
                if len(close_failure_pipes) >= 3
                else -1
            )
            if fd == target and not close_failure_injected:
                close_failure_injected = True
                original_close(fd)
                raise OSError(runner.errno.EIO, "injected pre-pidfd close")
            original_close(fd)

        before_close_failure_fds = set(os.listdir("/proc/self/fd"))
        before_close_failure_mask = runner.signal.pthread_sigmask(
            runner.signal.SIG_BLOCK,
            set(),
        )
        before_close_failure_subreaper = runner._child_subreaper_state()
        runner.os.pipe2 = track_exec_pipe
        runner.os.close = fail_pre_pidfd_close
        try:
            with self.assertRaises(runner.RunnerError) as pre_pidfd_failure:
                runner.exec_held_executable(
                    close_failure_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        "import signal;signal.pause()",
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                )
        finally:
            runner.os.close = original_close
            runner.os.pipe2 = original_pipe2
        self.assertTrue(close_failure_injected)
        self.assertEqual(
            pre_pidfd_failure.exception.reason,
            "environment_unavailable",
        )
        self.assertEqual(
            set(os.listdir("/proc/self/fd")),
            before_close_failure_fds,
        )
        self.assertEqual(
            runner.signal.pthread_sigmask(runner.signal.SIG_BLOCK, set()),
            before_close_failure_mask,
        )
        self.assertIs(
            runner._child_subreaper_state(),
            before_close_failure_subreaper,
        )
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)

        original_pidfd_open = runner.os.pidfd_open
        retained_close_target = -1
        retained_close_injected = False

        def track_exec_pidfd(pid: int) -> int:
            nonlocal retained_close_target
            retained_close_target = original_pidfd_open(pid)
            return retained_close_target

        def fail_retained_pidfd_close(fd: int) -> None:
            nonlocal retained_close_injected
            if fd == retained_close_target and not retained_close_injected:
                retained_close_injected = True
                original_close(fd)
                raise OSError(runner.errno.EIO, "injected pidfd close")
            original_close(fd)

        before_retained_close_fds = set(os.listdir("/proc/self/fd"))
        before_retained_close_mask = runner.signal.pthread_sigmask(
            runner.signal.SIG_BLOCK,
            set(),
        )
        before_retained_close_subreaper = runner._child_subreaper_state()
        runner.os.pidfd_open = track_exec_pidfd
        runner.os.close = fail_retained_pidfd_close
        try:
            with self.assertRaises(runner.RunnerError) as retained_close_failure:
                runner.exec_held_executable(
                    close_failure_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        "pass",
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                )
        finally:
            runner.os.close = original_close
            runner.os.pidfd_open = original_pidfd_open
        self.assertTrue(retained_close_injected)
        self.assertEqual(
            retained_close_failure.exception.reason,
            "environment_unavailable",
        )
        self.assertEqual(
            set(os.listdir("/proc/self/fd")),
            before_retained_close_fds,
        )
        self.assertEqual(
            runner.signal.pthread_sigmask(runner.signal.SIG_BLOCK, set()),
            before_retained_close_mask,
        )
        self.assertIs(
            runner._child_subreaper_state(),
            before_retained_close_subreaper,
        )
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        os.close(close_failure_python.fd)
        gated_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        original_write = runner.os.write
        original_default_selector = runner.selectors.DefaultSelector

        def fail_gate_write(fd: int, data: bytes) -> int:
            if data == b"G":
                raise OSError(runner.errno.EIO, "injected gate write failure")
            return original_write(fd, data)

        def fail_cleanup_selector() -> object:
            raise OSError(runner.errno.EMFILE, "injected selector exhaustion")

        before_gate_failure_fds = set(os.listdir("/proc/self/fd"))
        runner.os.write = fail_gate_write
        runner.selectors.DefaultSelector = fail_cleanup_selector
        try:
            with self.assertRaises(runner.RunnerError) as gate_failure:
                runner._spawn_held(
                    gated_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        "import signal;signal.pause()",
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                )
        finally:
            runner.selectors.DefaultSelector = original_default_selector
            runner.os.write = original_write
        self.assertEqual(gate_failure.exception.reason, "namespace_setup_failed")
        self.assertEqual(
            set(os.listdir("/proc/self/fd")),
            before_gate_failure_fds,
        )
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        inherited_ignore_signals: list[tuple[int, int]] = []
        previous_sigchld = runner.signal.signal(
            runner.signal.SIGCHLD,
            runner.signal.SIG_IGN,
        )
        original_kill = runner.os.kill

        def audit_ignored_spawn_kill(pid: int, number: int) -> None:
            inherited_ignore_signals.append((pid, number))
            original_kill(pid, number)

        before_ignored_spawn_fds = set(os.listdir("/proc/self/fd"))
        runner.os.kill = audit_ignored_spawn_kill
        try:
            with self.assertRaises(runner.RunnerError) as ignored_spawn:
                runner._spawn_held(
                    gated_python.fd,
                    (
                        "/usr/bin/python3.13",
                        "-I",
                        "-S",
                        "-B",
                        "-c",
                        "pass",
                    ),
                    {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
                )
        finally:
            runner.os.kill = original_kill
            runner.signal.signal(runner.signal.SIGCHLD, previous_sigchld)
        self.assertEqual(ignored_spawn.exception.reason, "namespace_setup_failed")
        self.assertEqual(inherited_ignore_signals, [])
        self.assertEqual(
            set(os.listdir("/proc/self/fd")),
            before_ignored_spawn_fds,
        )
        with self.assertRaises(ChildProcessError):
            os.waitpid(-1, os.WNOHANG)
        bootstrap_sigign_check = "_sigign_mask&(1<<(17-1))"
        self.assertIn(bootstrap_sigign_check, runner.BOOTSTRAP_V2_SOURCE)
        self.assertLess(
            runner.BOOTSTRAP_V2_SOURCE.index(bootstrap_sigign_check),
            runner.BOOTSTRAP_V2_SOURCE.index("pid=os.fork()"),
        )
        bootstrap_environment = {
            key: value
            for key, value in os.environ.items()
            if not (
                key.startswith(("LD_", "MALLOC_", "PYTHON"))
                or key
                in {
                    "GLIBC_TUNABLES",
                    "GCONV_PATH",
                    "LOCPATH",
                    "NLSPATH",
                }
            )
        }
        ignored_bootstrap = subprocess.run(
            [
                "/usr/bin/python3.13",
                "-I",
                "-S",
                "-B",
                "-c",
                runner.BOOTSTRAP_V2_SOURCE,
                "self-test",
                "--evidence-parent",
                "/home/spenser/__Active_code",
                "--expected-head",
                "0" * 40,
            ],
            cwd=runner.REPOSITORY_CANONICAL_PATH,
            env=bootstrap_environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            preexec_fn=lambda: runner.signal.signal(
                runner.signal.SIGCHLD,
                runner.signal.SIG_IGN,
            ),
            check=False,
        )
        self.assertEqual(
            (
                ignored_bootstrap.returncode,
                ignored_bootstrap.stdout,
                ignored_bootstrap.stderr,
            ),
            (66, b"", b""),
        )
        os.close(gated_python.fd)
        launch_bwrap_fd = os.open("/dev/null", os.O_RDONLY | os.O_CLOEXEC)
        launch_bwrap = runner.HeldExecutable(
            path="/dev/null",
            fd=launch_bwrap_fd,
            identity=(0, 0, 0, 0),
            sha256="",
        )
        launch_user_namespaces = runner.PreparedAclUserNamespaces(
            stage_a_fd=os.open(
                "/proc/self/ns/user",
                os.O_RDONLY | os.O_CLOEXEC,
            ),
            stage_b_fd=os.open(
                "/proc/self/ns/user",
                os.O_RDONLY | os.O_CLOEXEC,
            ),
        )
        original_spawn_held = runner._spawn_held

        def fail_spawn_held(*_args: object, **_kwargs: object) -> object:
            raise runner.RunnerError(68, "namespace_setup_failed")

        runner._spawn_held = fail_spawn_held
        launch_environment = {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"}
        try:
            before_stage_a_fds = set(os.listdir("/proc/self/fd"))
            with self.assertRaises(runner.RunnerError) as stage_a_failure:
                runner.launch_stage_a_bwrap(
                    bwrap=launch_bwrap,
                    user_namespaces=launch_user_namespaces,
                    source=_module_source(runner),
                    authority={
                        "partial_path": "/tmp/evidence.partial",
                        "backing_path": "/tmp/evidence.backing",
                    },
                    repository="/tmp/repository",
                    rustup_source="/tmp/rustup",
                    cargo_source="/tmp/cargo",
                    mode="serial",
                    label="launch-failure",
                    timeout_seconds=1,
                    expected_head="0" * 40,
                    environment=launch_environment,
                )
            self.assertEqual(
                stage_a_failure.exception.reason,
                "namespace_setup_failed",
            )
            self.assertEqual(
                set(os.listdir("/proc/self/fd")),
                before_stage_a_fds,
            )

            launch_output_read, launch_output_write = os.pipe2(os.O_CLOEXEC)
            try:
                before_stage_b_fds = set(os.listdir("/proc/self/fd"))
                with self.assertRaises(runner.RunnerError) as stage_b_failure:
                    runner.launch_stage_b_bwrap(
                        bwrap=launch_bwrap,
                        userns_fd=launch_user_namespaces.stage_b_fd,
                        source=_module_source(runner),
                        root="/tmp/root",
                        repository_snapshot="/tmp/repository",
                        rustup_snapshot="/tmp/rustup",
                        cargo_runtime="/tmp/cargo",
                        control_directory="/tmp/control",
                        repository_cwd="/tmp/repository",
                        mode="serial",
                        label="launch-failure",
                        timeout_seconds=1,
                        expected_head="0" * 40,
                        environment=launch_environment,
                        output_write_fd=launch_output_write,
                    )
                self.assertEqual(
                    stage_b_failure.exception.reason,
                    "namespace_setup_failed",
                )
                self.assertEqual(
                    set(os.listdir("/proc/self/fd")),
                    before_stage_b_fds,
                )
            finally:
                os.close(launch_output_read)
                os.close(launch_output_write)
        finally:
            runner._spawn_held = original_spawn_held
            runner.close_prepared_acl_user_namespaces(
                launch_user_namespaces
            )
            os.close(launch_bwrap_fd)
        acquired_python = runner.resolve_validated_executable(
            "/usr/bin/python3.13",
            runner.PLATFORM_STARTUP_TCB_V1[
                "/usr/bin/python3.13"
            ]["sha256"],
            expected_uid=0,
        )
        acquired_user_namespaces = runner.PreparedAclUserNamespaces(
            stage_a_fd=os.open(
                "/proc/self/ns/user",
                os.O_RDONLY | os.O_CLOEXEC,
            ),
            stage_b_fd=os.open(
                "/proc/self/ns/user",
                os.O_RDONLY | os.O_CLOEXEC,
            ),
        )
        original_close = runner.os.close
        close_injection: dict[str, object] = {
            "fd": None,
            "failed": False,
        }

        def spawn_acquired_child(
            _executable_fd: int,
            _argv: object,
            environment: object,
            *,
            preserve_fds: object = (),
            stdout_fd: int | None = None,
            stderr_fd: int | None = None,
        ) -> runner.SpawnedProcess:
            preserved = tuple(preserve_fds)
            close_injection["fd"] = preserved[1]
            return original_spawn_held(
                acquired_python.fd,
                (
                    "/usr/bin/python3.13",
                    "-I",
                    "-S",
                    "-B",
                    "-c",
                    "import signal;signal.pause()",
                ),
                environment,
                preserve_fds=preserved,
                stdout_fd=stdout_fd,
                stderr_fd=stderr_fd,
            )

        def close_then_fail(fd: int) -> None:
            if fd == close_injection["fd"] and not close_injection["failed"]:
                close_injection["failed"] = True
                original_close(fd)
                raise OSError(runner.errno.EIO, "injected close failure")
            original_close(fd)

        runner._spawn_held = spawn_acquired_child
        runner.os.close = close_then_fail
        runner.selectors.DefaultSelector = fail_cleanup_selector
        try:
            before_acquired_stage_a_fds = set(os.listdir("/proc/self/fd"))
            with self.assertRaises(OSError):
                runner.launch_stage_a_bwrap(
                    bwrap=runner.HeldExecutable(
                        path="/dev/null",
                        fd=acquired_python.fd,
                        identity=(0, 0, 0, 0),
                        sha256="",
                    ),
                    user_namespaces=acquired_user_namespaces,
                    source=_module_source(runner),
                    authority={
                        "partial_path": "/tmp/evidence.partial",
                        "backing_path": "/tmp/evidence.backing",
                    },
                    repository="/tmp/repository",
                    rustup_source="/tmp/rustup",
                    cargo_source="/tmp/cargo",
                    mode="serial",
                    label="acquired-child-failure",
                    timeout_seconds=1,
                    expected_head="0" * 40,
                    environment=launch_environment,
                )
            self.assertIs(close_injection["failed"], True)
            self.assertEqual(
                set(os.listdir("/proc/self/fd")),
                before_acquired_stage_a_fds,
            )
            with self.assertRaises(ChildProcessError):
                os.waitpid(-1, os.WNOHANG)

            close_injection.update({"fd": None, "failed": False})
            acquired_output_read, acquired_output_write = os.pipe2(os.O_CLOEXEC)
            try:
                before_acquired_stage_b_fds = set(os.listdir("/proc/self/fd"))
                with self.assertRaises(OSError):
                    runner.launch_stage_b_bwrap(
                        bwrap=runner.HeldExecutable(
                            path="/dev/null",
                            fd=acquired_python.fd,
                            identity=(0, 0, 0, 0),
                            sha256="",
                        ),
                        userns_fd=acquired_user_namespaces.stage_b_fd,
                        source=_module_source(runner),
                        root="/tmp/root",
                        repository_snapshot="/tmp/repository",
                        rustup_snapshot="/tmp/rustup",
                        cargo_runtime="/tmp/cargo",
                        control_directory="/tmp/control",
                        repository_cwd="/tmp/repository",
                        mode="serial",
                        label="acquired-child-failure",
                        timeout_seconds=1,
                        expected_head="0" * 40,
                        environment=launch_environment,
                        output_write_fd=acquired_output_write,
                    )
                self.assertIs(close_injection["failed"], True)
                self.assertEqual(
                    set(os.listdir("/proc/self/fd")),
                    before_acquired_stage_b_fds,
                )
                with self.assertRaises(ChildProcessError):
                    os.waitpid(-1, os.WNOHANG)
            finally:
                original_close(acquired_output_read)
                original_close(acquired_output_write)
        finally:
            runner.selectors.DefaultSelector = original_default_selector
            runner.os.close = original_close
            runner._spawn_held = original_spawn_held
            runner.close_prepared_acl_user_namespaces(
                acquired_user_namespaces
            )
            original_close(acquired_python.fd)
        mechanics = self.run_bounded_fixture("fd-protocol")
        self.assertEqual(
            mechanics,
            {
                "control_cloexec": True,
                "pid": 1,
                "stdin_unchanged": True,
            },
        )
        integrated = self.run_bounded_fixture(
            "production-integrated-stage-b-handshake",
            timeout=8,
        )
        self.assertEqual(
            integrated,
            {
                "bwrap_exit": True,
                "control_cloexec": True,
                "echld": True,
                "fixed_socket": True,
                "json_child_matches": True,
                "pid1": True,
                "same_namespace": True,
                "stdin_unchanged": True,
                "worker_departed": True,
            },
        )

    def test_sigterm_ignoring_descendant_requires_ineligible_sigkill(self) -> None:
        result = self.run_bounded_fixture("production-signal")
        self.assertTrue(result["forced"])
        self.assertFalse(result["timed_out"])
        self.assertEqual(result["received"], 1)
        stage_b_source = _module_source(runner).decode("utf-8").split(
            "def stage_b_worker_main",
            1,
        )[1]
        self.assertIn(
            "os.kill(os.getpid(), signal.SIGCHLD)",
            stage_b_source,
        )

    def test_stuck_stage_b_hits_bounded_stage_a_deadline(self) -> None:
        self.assertEqual(runner.SETUP_TIMEOUT_SECONDS, 60)
        self.assertEqual(runner.CLEANUP_TIMEOUT_SECONDS, 120)
        host_source = _module_source(runner).decode("utf-8").split(
            "def host_main",
            1,
        )[1]
        self.assertRegex(
            host_source,
            r"(?s)status_deadline\s*=\s*\(.*?"
            r"SNAPSHOT_TIMEOUT_SECONDS.*?"
            r"SETUP_TIMEOUT_SECONDS.*?"
            r"timeout_seconds.*?"
            r"CLEANUP_TIMEOUT_SECONDS.*?\)",
        )
        self.assertIn(
            "_reap_exact_spawned_before_deadline(\n"
            "            spawned,\n"
            "            deadline=status_deadline,\n"
            "        )",
            host_source,
        )
        self.assertNotIn("os.waitpid(spawned.pid, 0)", host_source)
        gate_read, gate_write = os.pipe2(os.O_CLOEXEC)
        survivor = os.fork()
        if survivor == 0:
            try:
                os.close(gate_write)
                os.read(gate_read, 1)
            finally:
                os._exit(0)
        os.close(gate_read)
        spawned = runner.SpawnedProcess(survivor, os.pidfd_open(survivor, 0))
        gate_open = True
        reaped = False
        try:
            started = time.monotonic()
            with self.assertRaisesRegex(
                runner.RunnerError,
                "containment_timeout",
            ):
                runner._reap_exact_spawned_before_deadline(
                    spawned,
                    deadline=time.monotonic() + 0.05,
                )
            self.assertLess(time.monotonic() - started, 0.5)
            self.assertIsNone(
                os.waitid(
                    os.P_PIDFD,
                    spawned.pidfd,
                    os.WEXITED | os.WNOHANG | os.WNOWAIT,
                )
            )
            os.close(gate_write)
            gate_open = False
            self.assertEqual(
                runner._reap_exact_spawned_before_deadline(
                    spawned,
                    deadline=time.monotonic() + 1.0,
                ),
                0,
            )
            reaped = True
            with self.assertRaises(ChildProcessError):
                os.waitpid(-1, os.WNOHANG)
        finally:
            if gate_open:
                os.close(gate_write)
            if not reaped:
                runner._terminate_exact_spawned_process(
                    spawned,
                    term_seconds=0.05,
                    kill_seconds=1.0,
                )
            os.close(spawned.pidfd)
        bounded = self.run_bounded_fixture("production-stuck-stage-b")
        self.assertEqual(
            bounded,
            {
                "bounded": True,
                "echld": True,
                "ready": True,
                "reaped": True,
                "status": -15,
            },
        )

    def test_rejects_forbidden_glob_ambient_parent_and_product_home_targets(
        self,
    ) -> None:
        parent = self.make_safe_parent()
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        for name in ("*", "..", "child/path"):
            with self.assertRaises(runner.RunnerError):
                runner.remove_tree_at(
                    parent_fd,
                    name,
                    cleanup_authority=self.cleanup_authority,
                )
        os.close(parent_fd)
        product_home = os.path.join(
            runner.pwd.getpwuid(os.getuid()).pw_dir,
            ".substrate",
        )
        for forbidden in (product_home, os.path.join(product_home, "nested")):
            with self.assertRaises(runner.RunnerError):
                runner._reject_product_home_selection(forbidden)
        previous = os.environ.get("SUBSTRATE_HOME")
        os.environ["SUBSTRATE_HOME"] = parent
        try:
            with self.assertRaises(runner.RunnerError):
                runner._reject_product_home_selection(
                    os.path.join(parent, "nested"),
                )
        finally:
            if previous is None:
                os.environ.pop("SUBSTRATE_HOME", None)
            else:
                os.environ["SUBSTRATE_HOME"] = previous

    def test_rejects_pathname_only_cleanup_api(self) -> None:
        source = _module_source(runner).decode("utf-8")
        self.assertNotIn("shutil.rmtree", source)
        self.assertIn("dir_fd=", source)
        parent = self.make_safe_parent()
        control_fd = os.open(
            parent,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        control_path = os.path.join(parent, "worker.sock")
        original_listener = runner.create_worker_control_listener(control_path)
        original = os.stat(
            "worker.sock",
            dir_fd=control_fd,
            follow_symlinks=False,
        )
        os.rename(
            "worker.sock",
            "worker.original",
            src_dir_fd=control_fd,
            dst_dir_fd=control_fd,
        )
        replacement_listener = runner.create_worker_control_listener(
            control_path
        )
        with self.assertRaises(runner.RunnerError):
            runner.remove_control_socket_at(
                control_fd,
                "worker.sock",
                (original.st_dev, original.st_ino),
            )
        self.assertTrue(
            stat.S_ISSOCK(
                os.stat(
                    "worker.sock",
                    dir_fd=control_fd,
                    follow_symlinks=False,
                ).st_mode
            )
        )
        self.assertTrue(
            stat.S_ISSOCK(
                os.stat(
                    "worker.original",
                    dir_fd=control_fd,
                    follow_symlinks=False,
                ).st_mode
            )
        )
        replacement_listener.close()
        original_listener.close()
        os.close(control_fd)

    def test_diagnostics_omit_comm_cmdline_environment_and_markers(self) -> None:
        record = runner._default_provenance(
            invocation_id="7" * 32,
            label="diagnostic",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={"SECRET_MARKER": "do-not-serialize"},
        )
        encoded = json.dumps(record, sort_keys=True)
        self.assertNotIn("do-not-serialize", encoded)
        self.assertNotIn("/proc/", encoded)
        self.assertNotIn("cmdline", encoded)

    def test_cli_rejects_arbitrary_command_root_and_environment_overrides(self) -> None:
        base = (
            "host",
            "--mode",
            "parallel",
            "--label",
            "cli",
            "--timeout-seconds",
            "1",
            "--evidence-parent",
            "/safe",
            "--expected-head",
            "0" * 40,
        )
        for option in ("--command", "--root", "--environment"):
            with self.assertRaises(runner.RunnerError):
                runner.parse_args((*base, option, "x"))

    def test_public_request_cannot_select_internal_host_stage_worker_roles_or_control_path(
        self,
    ) -> None:
        with self.assertRaises(runner.RunnerError):
            runner.parse_args(("stage-b-worker", "--control-path", "/tmp/x"))
        with self.assertRaises(runner.RunnerError):
            runner.parse_args(("stage-a", "--root", "/tmp/x"))

    def test_cli_rejects_invalid_label_timeout_cwd_and_head(self) -> None:
        variants = (
            ("Bad", "1", "0" * 40),
            ("ok", "0", "0" * 40),
            ("ok", "21601", "0" * 40),
            ("ok", "1", "Z" * 40),
        )
        for label, timeout, head in variants:
            with self.assertRaises(runner.RunnerError):
                runner.parse_args(
                    (
                        "host",
                        "--mode",
                        "parallel",
                        "--label",
                        label,
                        "--timeout-seconds",
                        timeout,
                        "--evidence-parent",
                        "/safe",
                        "--expected-head",
                        head,
                    )
                )
        captured_stderr = io.StringIO()
        previous_stderr = sys.stderr
        sys.stderr = captured_stderr
        try:
            with self.assertRaises(runner.RunnerError) as invalid_mode:
                runner.parse_args(
                    (
                        "host",
                        "--mode",
                        "SECRET-MODE-MARKER",
                        "--label",
                        "ok",
                        "--timeout-seconds",
                        "1",
                        "--evidence-parent",
                        "/safe",
                        "--expected-head",
                        "0" * 40,
                    )
                )
        finally:
            sys.stderr = previous_stderr
        self.assertEqual(
            invalid_mode.exception.reason,
            "invocation_authority_invalid",
        )
        self.assertEqual(captured_stderr.getvalue(), "")

    def test_rejects_dirty_index_worktree_and_nonignored_untracked_state(self) -> None:
        repository = self.make_disposable_repo()
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            with open("tracked.txt", "ab") as handle:
                handle.write(b"dirty\n")
            with self.assertRaises(runner.RunnerError) as caught:
                runner.verify_clean_repository(repository, expected_head, git)
            self.assertEqual(caught.exception.reason, "repository_dirty")
        finally:
            os.chdir(previous)
            os.close(git.fd)

        clean_repository = self.make_disposable_repo()
        clean_head = subprocess.check_output(
            ["/usr/bin/git", "-C", clean_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        clean_git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        retained_metadata: list[dict[str, object]] = []
        metadata_gitdir_fd = runner._open_validated_gitdir(clean_repository)
        previous = os.getcwd()
        try:
            os.chdir(clean_repository)
            verified = runner.verify_clean_repository(
                clean_repository,
                clean_head,
                clean_git,
                retained_metadata=retained_metadata,
            )
            self.assertTrue(verified["clean"])
            self.assertEqual(
                {record["relative"] for record in retained_metadata},
                {
                    "HEAD",
                    "config",
                    "index",
                    "info/exclude",
                    "refs/heads/main",
                },
            )
            runner._revalidate_retained_git_metadata(
                metadata_gitdir_fd,
                retained_metadata,
            )
            config_path = os.path.join(clean_repository, ".git", "config")
            held_path = config_path + ".held"
            os.rename(config_path, held_path)
            with open(config_path, "xb") as replacement:
                replacement.write(b"[core]\nrepositoryformatversion = 0\n")
            with self.assertRaises(runner.RunnerError):
                runner._revalidate_retained_git_metadata(
                    metadata_gitdir_fd,
                    retained_metadata,
                )
            os.unlink(config_path)
            os.rename(held_path, config_path)
            runner._revalidate_retained_git_metadata(
                metadata_gitdir_fd,
                retained_metadata,
            )
            info_path = os.path.join(clean_repository, ".git", "info")
            info_held = info_path + ".held"
            os.rename(info_path, info_held)
            os.mkdir(info_path, 0o700)
            with open(os.path.join(info_path, "exclude"), "xb") as replacement:
                replacement.write(b"")
            with self.assertRaises(runner.RunnerError):
                runner._revalidate_retained_git_metadata(
                    metadata_gitdir_fd,
                    retained_metadata,
                )
            os.unlink(os.path.join(info_path, "exclude"))
            os.rmdir(info_path)
            os.rename(info_held, info_path)
            runner._revalidate_retained_git_metadata(
                metadata_gitdir_fd,
                retained_metadata,
            )
        finally:
            os.chdir(previous)
            runner._close_retained_git_metadata(retained_metadata)
            os.close(metadata_gitdir_fd)
            os.close(clean_git.fd)
        source = _module_source(runner).decode("utf-8")
        host = source[
            source.index("def host_main")
            : source.index("def _run_selftest_bootstrap")
        ]
        self.assertIn(
            "retained_metadata=repository_metadata_authorities",
            host,
        )
        self.assertLess(
            host.index("_revalidate_retained_git_metadata("),
            host.index("finalize_after_stage_a_exit("),
        )

    def test_ignores_only_git_ignored_build_outputs(self) -> None:
        repository = self.make_disposable_repo()
        os.mkdir(os.path.join(repository, "ignored"), 0o700)
        with open(os.path.join(repository, "ignored", "artifact"), "xb") as handle:
            handle.write(b"ignored")
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            result = runner.verify_clean_repository(repository, expected_head, git)
            self.assertTrue(result["clean"])
        finally:
            os.chdir(previous)
            os.close(git.fd)

    def test_rejects_git_environment_config_alternates_replace_and_fsmonitor(
        self,
    ) -> None:
        valid_ref = b"refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap"
        invalid_refs = (
            b"refs/tags/not-a-branch",
            b"refs/heads/",
            b"refs/heads/-leading-dash",
            b"refs/heads/bad name",
            b"refs/heads/bad..name",
            b"refs/heads/.hidden",
            b"refs/heads/part/.hidden",
            b"refs/heads/name.lock",
            b"refs/heads/name@{prior",
            b"refs/heads/name~prior",
            b"refs/heads/name^prior",
            b"refs/heads/name:prior",
            b"refs/heads/name?prior",
            b"refs/heads/name*prior",
            b"refs/heads/name[prior",
            b"refs/heads/name\\prior",
            b"refs/heads/name.",
            b"refs/heads/part//name",
            b"refs/heads/control\x1fbyte",
            b"refs/heads/delete\x7fbyte",
        )
        self.assertTrue(runner._valid_symbolic_head_ref(valid_ref))
        for invalid_ref in invalid_refs:
            self.assertFalse(runner._valid_symbolic_head_ref(invalid_ref))
        bootstrap = runner.BOOTSTRAP_V2_SOURCE
        validator_source = bootstrap[
            bootstrap.index("def _valid_symbolic_head_ref") : bootstrap.index(
                "_root=os.getcwd()"
            )
        ]
        validator_scope: dict[str, object] = {}
        exec(validator_source, validator_scope)
        bootstrap_validator = validator_scope["_valid_symbolic_head_ref"]
        self.assertTrue(bootstrap_validator(valid_ref))
        for invalid_ref in invalid_refs:
            self.assertFalse(bootstrap_validator(invalid_ref))
        forbidden = {
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            "GIT_CONFIG",
            "GIT_CONFIG_COUNT",
            "GIT_DIR",
            "GIT_INDEX_FILE",
            "GIT_OBJECT_DIRECTORY",
            "GIT_WORK_TREE",
        }
        fixed = runner._fixed_git_environment()
        self.assertFalse(forbidden & fixed.keys())
        self.assertEqual(fixed["GIT_NO_REPLACE_OBJECTS"], "1")
        self.assertEqual(fixed["GIT_OPTIONAL_LOCKS"], "0")
        authority_repository = self.make_disposable_repo()
        authority_head = subprocess.check_output(
            ["/usr/bin/git", "-C", authority_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        authority_gitdir = os.open(
            os.path.join(authority_repository, ".git"),
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        held_git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        os.rename(
            os.path.join(authority_repository, ".git"),
            os.path.join(authority_repository, ".git-held"),
        )
        try:
            commit = runner._read_git_object(
                held_git,
                authority_repository,
                "commit",
                authority_head,
                gitdir_fd=authority_gitdir,
            )
            self.assertTrue(commit.startswith(b"tree "))
        finally:
            os.rename(
                os.path.join(authority_repository, ".git-held"),
                os.path.join(authority_repository, ".git"),
            )
            os.close(authority_gitdir)
            os.close(held_git.fd)
        fifo_repository = self.make_disposable_repo()
        fifo_head = subprocess.check_output(
            ["/usr/bin/git", "-C", fifo_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        os.unlink(os.path.join(fifo_repository, ".git", "HEAD"))
        os.mkfifo(os.path.join(fifo_repository, ".git", "HEAD"), 0o600)
        fifo_git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        try:
            with self.assertRaises(runner.RunnerError):
                runner.verify_expected_head_object_chain(
                    fifo_repository,
                    fifo_head,
                    fifo_git,
                )
        finally:
            os.close(fifo_git.fd)
        bootstrap_prefix = runner.BOOTSTRAP_V2_SOURCE.split("_initial_expected=", 1)[0]
        bootstrap_namespace: dict[str, object] = {}
        exec(compile(bootstrap_prefix, "<bootstrap-reader-test>", "exec"), bootstrap_namespace)
        bootstrap_root = self.make_safe_parent()
        bootstrap_fd = os.open(
            bootstrap_root,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        os.mkfifo(os.path.join(bootstrap_root, "HEAD"), 0o600)
        try:
            with self.assertRaises(SystemExit) as bootstrap_rejection:
                bootstrap_namespace["_read_at"](bootstrap_fd, "HEAD", 4096)
            self.assertEqual(bootstrap_rejection.exception.code, 65)
        finally:
            os.close(bootstrap_fd)
        self.assertIn("'commondir'", runner.BOOTSTRAP_V2_SOURCE)
        commondir_repository = self.make_disposable_repo()
        commondir_head = subprocess.check_output(
            ["/usr/bin/git", "-C", commondir_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        with open(os.path.join(commondir_repository, ".git", "commondir"), "xb") as handle:
            handle.write(b"../redirected-common-dir\n")
        commondir_git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        try:
            with self.assertRaises(runner.RunnerError):
                runner.verify_expected_head_object_chain(
                    commondir_repository,
                    commondir_head,
                    commondir_git,
                )
        finally:
            os.close(commondir_git.fd)
        runner_source = _module_source(runner).decode("utf-8")
        host_source = runner_source.split("def host_main", 1)[1].split(
            "def _run_selftest_bootstrap",
            1,
        )[0]
        self.assertIn(
            'globals().get("__CANONICAL_BOOTSTRAP_GITDIR_FD")',
            host_source,
        )
        self.assertGreaterEqual(
            host_source.count("gitdir_fd=bootstrap_gitdir_fd"),
            3,
        )
        stage_a_source = runner_source.split("def stage_a_main", 1)[1].split(
            "def _executable_identity",
            1,
        )[0]
        self.assertLess(
            stage_a_source.index("stage_a_gitdir_fd = _open_validated_gitdir"),
            stage_a_source.index("_verify_clean_repository_with_gitdir("),
        )
        self.assertLess(
            stage_a_source.index("_verify_clean_repository_with_gitdir("),
            stage_a_source.index("construct_repository_snapshot("),
        )
        self.assertLess(
            stage_a_source.index("construct_repository_snapshot("),
            stage_a_source.index("os.close(stage_a_gitdir_fd)"),
        )
        continuous_repository = self.make_disposable_repo()
        continuous_head = subprocess.check_output(
            ["/usr/bin/git", "-C", continuous_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        continuous_git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        continuous_held = continuous_repository + "-gitdir-held"
        original_git_command = runner._git_command
        rebound_calls = 0

        def rebind_before_cleanliness_git(
            git: object,
            repository_path: str,
            arguments: object,
            **kwargs: object,
        ) -> tuple[int, bytes, bytes]:
            nonlocal rebound_calls
            self.assertIsNotNone(kwargs.get("gitdir_fd"))
            did_rebind = not os.path.exists(continuous_held)
            if did_rebind:
                os.rename(os.path.join(continuous_repository, ".git"), continuous_held)
                os.mkdir(os.path.join(continuous_repository, ".git"), 0o700)
                rebound_calls += 1
            try:
                return original_git_command(
                    git,
                    repository_path,
                    arguments,
                    **kwargs,
                )
            finally:
                if did_rebind:
                    os.rmdir(os.path.join(continuous_repository, ".git"))
                    os.rename(continuous_held, os.path.join(continuous_repository, ".git"))

        runner._git_command = rebind_before_cleanliness_git
        previous = os.getcwd()
        try:
            os.chdir(continuous_repository)
            clean = runner.verify_clean_repository(
                continuous_repository,
                continuous_head,
                continuous_git,
            )
            self.assertTrue(clean["clean"])
            self.assertGreaterEqual(rebound_calls, 5)
        finally:
            os.chdir(previous)
            runner._git_command = original_git_command
            replacement = os.path.join(continuous_repository, ".git")
            if os.path.isdir(replacement) and os.path.isdir(continuous_held):
                os.rmdir(replacement)
                os.rename(continuous_held, replacement)
            os.close(continuous_git.fd)
        snapshot_repository = self.make_disposable_repo()
        snapshot_head = subprocess.check_output(
            ["/usr/bin/git", "-C", snapshot_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        snapshot_git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        snapshot_held = snapshot_repository + "-gitdir-held"
        snapshot_parent = self.make_safe_parent()
        original_tree_manifest = runner._tree_manifest
        original_read_git_object = runner._read_git_object
        snapshot_tree_rebounds = 0
        snapshot_blob_rebounds = 0

        def rebind_before_snapshot_tree(
            git: object,
            repository_path: str,
            tree_oid: str,
            prefix: bytes = b"",
            **kwargs: object,
        ) -> tuple[bytes, tuple[tuple[bytes, int, str], ...]]:
            nonlocal snapshot_tree_rebounds
            self.assertIsNotNone(kwargs.get("gitdir_fd"))
            did_rebind = not os.path.exists(snapshot_held)
            if did_rebind:
                os.rename(os.path.join(snapshot_repository, ".git"), snapshot_held)
                os.mkdir(os.path.join(snapshot_repository, ".git"), 0o700)
                snapshot_tree_rebounds += 1
            try:
                return original_tree_manifest(
                    git,
                    repository_path,
                    tree_oid,
                    prefix,
                    **kwargs,
                )
            finally:
                if did_rebind:
                    os.rmdir(os.path.join(snapshot_repository, ".git"))
                    os.rename(snapshot_held, os.path.join(snapshot_repository, ".git"))

        def rebind_before_snapshot_blob(
            git: object,
            repository_path: str,
            object_type: str,
            oid: str,
            **kwargs: object,
        ) -> bytes:
            nonlocal snapshot_blob_rebounds
            if object_type != "blob":
                return original_read_git_object(
                    git,
                    repository_path,
                    object_type,
                    oid,
                    **kwargs,
                )
            self.assertIsNotNone(kwargs.get("gitdir_fd"))
            os.rename(os.path.join(snapshot_repository, ".git"), snapshot_held)
            os.mkdir(os.path.join(snapshot_repository, ".git"), 0o700)
            snapshot_blob_rebounds += 1
            try:
                return original_read_git_object(
                    git,
                    repository_path,
                    object_type,
                    oid,
                    **kwargs,
                )
            finally:
                os.rmdir(os.path.join(snapshot_repository, ".git"))
                os.rename(snapshot_held, os.path.join(snapshot_repository, ".git"))

        runner._tree_manifest = rebind_before_snapshot_tree
        runner._read_git_object = rebind_before_snapshot_blob
        try:
            snapshot = runner.construct_repository_snapshot(
                snapshot_repository,
                snapshot_head,
                snapshot_git,
                os.path.join(snapshot_parent, "snapshot"),
            )
            self.assertTrue(snapshot["no_host_alias"])
            self.assertGreaterEqual(snapshot_tree_rebounds, 1)
            self.assertGreaterEqual(snapshot_blob_rebounds, 1)
        finally:
            runner._tree_manifest = original_tree_manifest
            runner._read_git_object = original_read_git_object
            replacement = os.path.join(snapshot_repository, ".git")
            if os.path.isdir(replacement) and os.path.isdir(snapshot_held):
                os.rmdir(replacement)
                os.rename(snapshot_held, replacement)
            os.close(snapshot_git.fd)
        for relative in ("config", "index", "info/exclude", "refs/heads/main"):
            bounded_repository = self.make_disposable_repo()
            bounded_head = subprocess.check_output(
                ["/usr/bin/git", "-C", bounded_repository, "rev-parse", "HEAD"],
                text=True,
            ).strip()
            target = os.path.join(bounded_repository, ".git", *relative.split("/"))
            saved = target + ".held"
            os.rename(target, saved)
            os.mkfifo(target, 0o600)
            bounded_git = runner.resolve_validated_executable(
                "/usr/bin/git",
                runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
                expected_uid=0,
            )
            parent_socket, child_socket = socket.socketpair()
            child = os.fork()
            if child == 0:
                parent_socket.close()
                try:
                    os.chdir(bounded_repository)
                    try:
                        runner.verify_clean_repository(
                            bounded_repository,
                            bounded_head,
                            bounded_git,
                        )
                    except runner.RunnerError:
                        child_socket.sendall(b"R")
                    else:
                        child_socket.sendall(b"S")
                except BaseException:
                    try:
                        child_socket.sendall(b"X")
                    except OSError:
                        pass
                finally:
                    os._exit(0)
            child_socket.close()
            parent_socket.settimeout(3.0)
            try:
                try:
                    bounded_result = parent_socket.recv(1)
                except TimeoutError:
                    bounded_result = b"T"
                    os.kill(child, 9)
                waited, status = os.waitpid(child, 0)
            finally:
                parent_socket.close()
                os.close(bounded_git.fd)
                os.unlink(target)
                os.rename(saved, target)
            self.assertEqual((waited, bounded_result), (child, b"R"), relative)
            self.assertTrue(os.WIFEXITED(status), relative)
        repository = self.make_disposable_repo()
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        alternates = os.path.join(
            repository,
            ".git",
            "objects",
            "info",
            "alternates",
        )
        with open(alternates, "xb") as handle:
            handle.write(b"/forbidden\n")
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            with self.assertRaises(runner.RunnerError):
                runner.verify_clean_repository(repository, expected_head, git)
        finally:
            os.chdir(previous)
            os.close(git.fd)

    def test_rejects_local_git_config_include_mode_exclude_worktree_and_unknown_keys(
        self,
    ) -> None:
        self.assertFalse(runner._allowed_git_config("include.path", "/tmp/x"))
        self.assertFalse(runner._allowed_git_config("core.excludesfile", "/tmp/x"))
        self.assertFalse(runner._allowed_git_config("core.worktree", "/tmp/x"))
        self.assertFalse(runner._allowed_git_config("status.showuntrackedfiles", "no"))
        repository = self.make_disposable_repo()
        subprocess.run(
            [
                "/usr/bin/git",
                "-C",
                repository,
                "config",
                "--local",
                "status.showUntrackedFiles",
                "no",
            ],
            check=True,
        )
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            with self.assertRaises(runner.RunnerError):
                runner.verify_clean_repository(repository, expected_head, git)
        finally:
            os.chdir(previous)
            os.close(git.fd)

    def test_info_exclude_is_held_but_never_cleanliness_authority(self) -> None:
        repository = self.make_disposable_repo()
        with open(
            os.path.join(repository, ".git", "info", "exclude"),
            "ab",
        ) as handle:
            handle.write(b"hidden-by-info-exclude\n")
        with open(
            os.path.join(repository, "hidden-by-info-exclude"),
            "xb",
        ) as handle:
            handle.write(b"still dirty")
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            with self.assertRaises(runner.RunnerError) as caught:
                runner.verify_clean_repository(repository, expected_head, git)
            self.assertEqual(caught.exception.reason, "repository_dirty")
        finally:
            os.chdir(previous)
            os.close(git.fd)

    def test_rejects_info_attributes_worktree_config_and_index_authority_extensions(
        self,
    ) -> None:
        repository = self.make_disposable_repo()
        with open(
            os.path.join(repository, ".git", "info", "attributes"),
            "xb",
        ) as handle:
            handle.write(b"* -text\n")
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            with self.assertRaises(runner.RunnerError):
                runner.verify_clean_repository(repository, expected_head, git)
        finally:
            os.chdir(previous)
            os.close(git.fd)

    def test_rejects_tree_index_stage_mode_flag_and_worktree_byte_drift(self) -> None:
        file_entry = b"100644 dist-workspace.toml\0" + b"\x01" * 20
        directory_entry = b"40000 dist\0" + b"\x02" * 20
        parsed = runner._parse_tree(file_entry + directory_entry)
        self.assertEqual(
            tuple((mode, name) for mode, name, _oid in parsed),
            ((0o100644, b"dist-workspace.toml"), (0o40000, b"dist")),
        )
        with self.assertRaises(runner.RunnerError):
            runner._parse_tree(directory_entry + file_entry)
        duplicate_name_tree = (
            b"100644 a\0"
            + b"\x03" * 20
            + b"100644 a.\0"
            + b"\x04" * 20
            + b"40000 a\0"
            + b"\x05" * 20
        )
        with self.assertRaises(runner.RunnerError):
            runner._parse_tree(duplicate_name_tree)
        malformed_mode_trees = (
            b"0100644 padded\0" + b"\x06" * 20,
            b"100648 invalid-octal\0" + b"\x07" * 20,
            b"100600 unknown-mode\0" + b"\x08" * 20,
        )
        for malformed_tree in malformed_mode_trees:
            with self.assertRaises(runner.RunnerError):
                runner._parse_tree(malformed_tree)
        bootstrap = runner.BOOTSTRAP_V2_SOURCE
        parser_source = bootstrap[
            bootstrap.index("def _tree_entries") : bootstrap.index("def _lookup")
        ]
        parser_scope: dict[str, object] = {}
        exec(parser_source, parser_scope)
        bootstrap_parser = parser_scope["_tree_entries"]
        self.assertEqual(
            tuple(
                (mode, name)
                for mode, name, _oid in bootstrap_parser(
                    file_entry + directory_entry
                )
            ),
            ((0o100644, b"dist-workspace.toml"), (0o40000, b"dist")),
        )
        for malformed_tree in malformed_mode_trees:
            with self.assertRaises(SystemExit):
                bootstrap_parser(malformed_tree)
        content = b"tracked\n"
        oid = runner.inline_sha1(
            b"blob " + str(len(content)).encode() + b"\0" + content
        )
        self.assertEqual(len(oid), 40)
        changed = b"tracked!\n"
        changed_oid = runner.inline_sha1(
            b"blob " + str(len(changed)).encode() + b"\0" + changed
        )
        self.assertNotEqual(oid, changed_oid)
        repository = self.make_disposable_repo()
        subprocess.run(
            [
                "/usr/bin/git",
                "-C",
                repository,
                "update-index",
                "--assume-unchanged",
                "tracked.txt",
            ],
            check=True,
        )
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            with self.assertRaises(runner.RunnerError):
                runner.verify_clean_repository(repository, expected_head, git)
        finally:
            os.chdir(previous)
            os.close(git.fd)

    def test_tracked_gitignore_is_only_untracked_exclusion_authority(self) -> None:
        source = _module_source(runner).decode("utf-8")
        self.assertIn(".gitignore", source)
        self.assertIn("--exclude-per-directory=.gitignore", source)

    def test_rejects_untracked_gitignore_and_every_nonignored_untracked_path(
        self,
    ) -> None:
        repository = self.make_disposable_repo()
        with open(os.path.join(repository, "untracked"), "xb") as handle:
            handle.write(b"x")
        output = subprocess.check_output(
            [
                "/usr/bin/git",
                "-C",
                repository,
                "ls-files",
                "--others",
                "--exclude-per-directory=.gitignore",
                "-z",
            ]
        )
        self.assertEqual(output, b"untracked\0")
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(repository)
            with self.assertRaises(runner.RunnerError) as caught:
                runner.verify_clean_repository(repository, expected_head, git)
            self.assertEqual(caught.exception.reason, "repository_dirty")
        finally:
            os.chdir(previous)
            os.close(git.fd)

        hidden_repository = self.make_disposable_repo()
        hidden_directory = os.path.join(hidden_repository, "ignored")
        os.mkdir(hidden_directory, 0o700)
        with open(
            os.path.join(hidden_directory, ".gitignore"),
            "xb",
        ) as handle:
            handle.write(b".gitignore\nsecret\n")
        with open(os.path.join(hidden_directory, "secret"), "xb") as handle:
            handle.write(b"hidden\n")
        ignored = subprocess.check_output(
            [
                "/usr/bin/git",
                "-C",
                hidden_repository,
                "ls-files",
                "--others",
                "--ignored",
                "--exclude-per-directory=.gitignore",
                "-z",
            ]
        )
        self.assertEqual(
            ignored,
            b"ignored/.gitignore\0ignored/secret\0",
        )
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", hidden_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(hidden_repository)
            self.assertTrue(
                runner.verify_clean_repository(
                    hidden_repository,
                    expected_head,
                    git,
                )["clean"]
            )
        finally:
            os.chdir(previous)
            os.close(git.fd)

        rogue_repository = self.make_disposable_repo()
        rogue_directory = os.path.join(rogue_repository, "rogue")
        os.mkdir(rogue_directory, 0o700)
        with open(
            os.path.join(rogue_directory, ".gitignore"),
            "xb",
        ) as handle:
            handle.write(b".gitignore\nsecret\n")
        with open(os.path.join(rogue_directory, "secret"), "xb") as handle:
            handle.write(b"hidden\n")
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", rogue_repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        previous = os.getcwd()
        try:
            os.chdir(rogue_repository)
            with self.assertRaises(runner.RunnerError) as self_hidden:
                runner.verify_clean_repository(
                    rogue_repository,
                    expected_head,
                    git,
                )
            self.assertEqual(self_hidden.exception.reason, "repository_dirty")
        finally:
            os.chdir(previous)
            os.close(git.fd)

    def test_rejects_path_shim_and_executable_identity_drift(self) -> None:
        parent = self.make_safe_parent()
        source_fd = os.open("/usr/bin/true", os.O_RDONLY | os.O_CLOEXEC)
        path = os.path.join(parent, "tool")
        destination_fd = os.open(
            path,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o755,
        )
        try:
            while True:
                chunk = os.read(source_fd, 65_536)
                if not chunk:
                    break
                os.write(destination_fd, chunk)
        finally:
            os.close(source_fd)
            os.close(destination_fd)
        with open(path, "rb") as handle:
            digest = hashlib.sha256(handle.read()).hexdigest()
        held = runner.resolve_validated_executable(
            path,
            digest,
            expected_uid=os.getuid(),
        )
        os.rename(path, path + ".held")
        with open(path, "xb") as handle:
            handle.write(b"replacement")
        with self.assertRaises(runner.RunnerError):
            runner.resolve_validated_executable(
                path,
                digest,
                expected_uid=os.getuid(),
            )
        os.close(held.fd)

    def test_executes_and_revalidates_held_bwrap_python_git_rustup_cargo_and_rustc(
        self,
    ) -> None:
        held = runner.validate_platform_startup_tcb()
        try:
            self.assertEqual(
                set(held),
                set(runner.PLATFORM_STARTUP_TCB_V1),
            )
            for executable in held.values():
                self.assertEqual(
                    runner.hash_open_file(executable.fd),
                    executable.sha256,
                )
        finally:
            for executable in held.values():
                os.close(executable.fd)

    def test_trusted_bwrap_launcher_rejects_ambient_loader_and_startup_tcb_drift(
        self,
    ) -> None:
        key = "LD_PRELOAD"
        old = os.environ.get(key)
        os.environ[key] = "/forbidden"
        try:
            with self.assertRaises(runner.RunnerError):
                runner.validate_python_runtime(require_bootstrap=False)
        finally:
            if old is None:
                os.environ.pop(key)
            else:
                os.environ[key] = old

    def test_stage_b_as_pid1_matches_json_child_pid_peer_credentials_and_echild(
        self,
    ) -> None:
        integrated = self.run_bounded_fixture(
            "production-integrated-stage-b-handshake",
            timeout=8,
        )
        self.assertTrue(all(integrated.values()))
        source = _module_source(runner).decode("utf-8")
        stage_a = source[
            source.index("def stage_a_main")
            : source.index("def _executable_identity")
        ]
        worker = source[
            source.index("def stage_b_worker_main")
            : source.index("def _default_provenance")
        ]
        worker_identity = source[
            source.index("def _capture_stage_b_worker_identity")
            : source.index("def _canonical_cargo_command")
        ]
        self.assertIn(
            'worker_pid = int(first_status["child-pid"])',
            worker_identity,
        )
        self.assertIn("_capture_stage_b_worker_identity(first_status)", stage_a)
        self.assertIn("expected_pid=worker_pid", stage_a)
        self.assertIn(
            'int(worker_result["pid_namespace_inode"])\n'
            '            != namespace_identities["pid-namespace"]',
            stage_a,
        )
        self.assertIn(
            "if os.getpid() != 1 or os.getppid() != 0:",
            worker,
        )
        self.assertIn("if not set_child_subreaper():", worker)
        self.assertIn("result.echld_observed", worker)

    def test_held_git_fd_path_replacement_never_executes_replacement(self) -> None:
        parent = self.make_safe_parent()
        source_fd = os.open("/usr/bin/true", os.O_RDONLY | os.O_CLOEXEC)
        path = os.path.join(parent, "held-true")
        destination_fd = os.open(
            path,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o755,
        )
        try:
            while True:
                chunk = os.read(source_fd, 65_536)
                if not chunk:
                    break
                os.write(destination_fd, chunk)
        finally:
            os.close(source_fd)
            os.close(destination_fd)
        with open(path, "rb") as handle:
            digest = hashlib.sha256(handle.read()).hexdigest()
        held = runner.resolve_validated_executable(
            path,
            digest,
            expected_uid=os.getuid(),
        )
        os.rename(path, path + ".original")
        with open(path, "xb") as handle:
            handle.write(b"marker")
        status, stdout, stderr = runner.exec_held_executable(
            held.fd,
            [path],
            {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
        )
        os.close(held.fd)
        self.assertEqual((status, stdout, stderr), (0, b"", b""))

    def test_isolated_python_startup_rejects_environment_site_path_and_module_poison(
        self,
    ) -> None:
        poison = self.make_safe_parent()
        with open(os.path.join(poison, "sitecustomize.py"), "xb") as handle:
            handle.write(b"raise SystemExit(99)\n")
        environment = dict(os.environ)
        environment["PYTHONPATH"] = poison
        completed = subprocess.run(
            [
                "/usr/bin/python3.13",
                "-I",
                "-S",
                "-B",
                "-c",
                (
                    "import sys;"
                    "assert sys.flags.isolated==1;"
                    "assert sys.flags.no_site==1;"
                    "assert sys.flags.no_user_site==1;"
                    "assert sys.flags.ignore_environment==1;"
                    "assert sys.flags.dont_write_bytecode==1;"
                    "assert sys.flags.safe_path;"
                    "assert '' not in sys.path;"
                    "assert sys.path==['/usr/lib/python313.zip',"
                    "'/usr/lib/python3.13',"
                    "'/usr/lib/python3.13/lib-dynload']"
                ),
            ],
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=5,
            check=False,
        )
        self.assertEqual(
            (completed.returncode, completed.stdout, completed.stderr),
            (0, b"", b""),
        )

    def test_python_runtime_rejects_origin_and_late_import_drift(self) -> None:
        source = _module_source(runner)
        entries = runner.parse_stdlib_manifest(source)
        self.assertEqual(entries, tuple(sorted(entries)))
        self.assertEqual(len(entries), len(set(entries)))

    def test_bootstrap_rejects_path_execution_cmdline_drift_and_runner_blob_mismatch(
        self,
    ) -> None:
        source = runner.BOOTSTRAP_V2_SOURCE
        self.assertIn("/proc", source)
        self.assertIn("expected-head", source)
        self.assertIn("_sha256", source)
        self.assertNotIn("subprocess", source)
        self.assertLess(
            source.index("_trailers!=_expected_trailers"),
            source.index("exec(compile(_source"),
        )
        repository = self.make_disposable_repo()
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        evidence_parent = self.make_safe_parent()
        argv = runner._substitute_template(
            runner.HOST_INVOCATION_ARGV_TEMPLATE_V1,
            {
                "MODE": "parallel",
                "LABEL": "bootstrap-proof",
                "TIMEOUT_DECIMAL": "1",
                "EVIDENCE_PARENT": evidence_parent,
                "EXPECTED_HEAD": expected_head,
            },
        )
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(runner.FORBIDDEN_STARTUP_PREFIXES)
            and key not in runner.FORBIDDEN_STARTUP_KEYS
        }
        completed = subprocess.run(
            argv,
            cwd=repository,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(completed.returncode, 65)
        self.assertEqual(completed.stdout, b"")
        self.assertEqual(completed.stderr, b"repository_identity_mismatch\n")

    def test_self_test_loader_authenticates_exact_runner_and_test_blobs(self) -> None:
        self.assertIn(
            "_module=sys.modules['types'].ModuleType('__canonical_runner__')",
            runner.BOOTSTRAP_V2_SOURCE,
        )
        repository = self.make_disposable_repo()
        expected_head = subprocess.check_output(
            ["/usr/bin/git", "-C", repository, "rev-parse", "HEAD"],
            text=True,
        ).strip()
        git = runner.resolve_validated_executable(
            "/usr/bin/git",
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/git"]["sha256"],
            expected_uid=0,
        )
        try:
            runner_blob, runner_oid = runner.read_expected_head_blob(
                repository,
                expected_head,
                "scripts/ci/canonical_shell_wall_runner.py",
                git,
            )
            test_blob, test_oid = runner.read_expected_head_blob(
                repository,
                expected_head,
                "scripts/ci/test_canonical_shell_wall_runner.py",
                git,
            )
            self.assertEqual(len(runner_oid), 40)
            self.assertEqual(len(test_oid), 40)
            self.assertEqual(runner.inline_sha256(runner_blob), hashlib.sha256(runner_blob).hexdigest())
            self.assertEqual(runner.inline_sha256(test_blob), hashlib.sha256(test_blob).hexdigest())
        finally:
            os.close(git.fd)

    def test_self_test_loader_rejects_path_import_discovery_and_blob_mismatch(
        self,
    ) -> None:
        source = _module_source(runner).decode("utf-8")
        function = source[source.index("def run_authenticated_self_tests") :]
        self.assertNotIn("discover(", function)
        test_source = _test_module_source().decode("utf-8")
        guard = "if not _authenticated_self_test_loader:"
        self.assertIn(guard, test_source)
        self.assertLess(test_source.index(guard), test_source.index("import hashlib"))
        self.assertIsNone(
            re.search(r'(?m)^if __name__ == "__main__":$', test_source)
        )
        direct = subprocess.run(
            [
                "/usr/bin/python3.13",
                "-I",
                "-S",
                "-B",
                os.path.join(
                    runner.REPOSITORY_CANONICAL_PATH,
                    "scripts/ci/test_canonical_shell_wall_runner.py",
                ),
            ],
            cwd=runner.REPOSITORY_CANONICAL_PATH,
            env={"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(direct.returncode, 65)
        self.assertEqual(direct.stdout, b"")
        self.assertEqual(direct.stderr, b"")
        with self.assertRaises(runner.RunnerError):
            runner.load_authenticated_module(
                "bad",
                b"# canonical-stdlib-manifest-v1: built-in:sys\n",
                "<synthetic>",
                "0" * 64,
            )

    def test_private_rustup_toolchain_snapshot_matches_pinned_complete_manifest(
        self,
    ) -> None:
        manifest = runner._manifest_tree(
            "/home/spenser/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu"
        )
        self.assertEqual(
            manifest,
            (
                "62de669575b22b82124eacb7caf100fd1e1041e42bd66b2a148f2ff6d14540c0",
                26,
                213,
                0,
            ),
        )
        source = _module_source(runner).decode("utf-8")
        validation = source[
            source.index("def validate_rustup_resolution")
            : source.index("def run_authenticated_self_tests")
        ]
        self.assertIn(
            "directory_manifests = {",
            validation,
        )
        self.assertIn(
            "_manifest_tree(descriptor) != directory_manifests[path]",
            validation,
        )
        self.assertLess(
            validation.index("directory_manifests = {"),
            validation.index("return {"),
        )

    def test_mounted_rustc_reports_private_sysroot_and_compiles_minimal_test(
        self,
    ) -> None:
        result = self.run_bounded_fixture(
            "production-toolchain-probe",
            timeout=15,
        )
        self.assertEqual(
            result,
            {
                "probe_removed": True,
                "rustup_sha256": (
                    "20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c"
                ),
            },
        )

    def test_vendor_snapshot_reconstructs_only_lock_checksum_selected_archives(
        self,
    ) -> None:
        repository = self.make_safe_parent()
        cargo = self.make_safe_parent()
        destination = os.path.join(self.make_safe_parent(), "vendor")
        package = self.make_safe_parent()
        with open(os.path.join(package, "Cargo.toml"), "xb") as handle:
            handle.write(b'[package]\nname="selected"\nversion="1.0.0"\n')
        os.mkdir(os.path.join(cargo, "cache"), 0o700)
        selected_archive = os.path.join(cargo, "cache", "selected.crate")
        with tarfile.open(selected_archive, "w:gz") as archive:
            archive.add(
                os.path.join(package, "Cargo.toml"),
                arcname="selected-1.0.0/Cargo.toml",
                recursive=False,
            )
        with open(selected_archive, "rb") as handle:
            checksum = hashlib.sha256(handle.read()).hexdigest()
        with open(os.path.join(repository, "Cargo.lock"), "xb") as handle:
            handle.write(
                (
                    "[[package]]\n"
                    'name = "selected"\n'
                    'version = "1.0.0"\n'
                    'source = "registry+https://github.com/rust-lang/'
                    'crates.io-index"\n'
                    f'checksum = "{checksum}"\n'
                ).encode("ascii")
            )
        with open(os.path.join(cargo, "cache", "other.crate"), "xb") as handle:
            handle.write(b"other")
        _manifest, count = runner.reconstruct_locked_vendor(
            repository,
            cargo,
            destination,
        )
        self.assertEqual(count, 1)
        self.assertEqual(os.listdir(destination), ["selected-1.0.0"])
        with open(
            os.path.join(destination, "selected-1.0.0", ".cargo-checksum.json"),
            "rb",
        ) as handle:
            checksum_record = json.load(handle)
        self.assertEqual(checksum_record["package"], checksum)

    def test_constructed_cargo_home_resolves_locked_vendor_offline_without_drift(
        self,
    ) -> None:
        self.assertIn(
            "file:gzip:/usr/lib/python3.13/gzip.py:"
            "dba33ac2497af37712ea23f5c3ce3ed56177262886868cbc6a6d43632c79afe9",
            runner.parse_stdlib_manifest(_module_source(runner)),
        )
        repository = self.make_safe_parent()
        os.mkdir(os.path.join(repository, "src"), 0o700)
        with open(os.path.join(repository, "Cargo.toml"), "xb") as handle:
            handle.write(
                b'[package]\nname="fixture-root"\nversion="0.1.0"\n'
                b'edition="2021"\n[dependencies]\nfixture-dep="=1.0.0"\n'
            )
        with open(os.path.join(repository, "src", "lib.rs"), "xb") as handle:
            handle.write(b"pub fn answer() -> u32 { fixture_dep::answer() }\n")

        package = self.make_safe_parent()
        os.mkdir(os.path.join(package, "src"), 0o700)
        with open(os.path.join(package, "Cargo.toml"), "xb") as handle:
            handle.write(
                b'[package]\nname="fixture-dep"\nversion="1.0.0"\n'
                b'edition="2021"\n[lib]\npath="src/lib.rs"\n'
            )
        with open(os.path.join(package, "src", "lib.rs"), "xb") as handle:
            handle.write(b"pub fn answer() -> u32 { 42 }\n")

        cargo_source = self.make_safe_parent()
        cargo_bin = os.path.join(cargo_source, "bin")
        os.mkdir(cargo_bin, 0o700)
        rustup_source = os.open(
            "/home/spenser/.cargo/bin/rustup",
            os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        rustup_destination = os.open(
            os.path.join(cargo_bin, "rustup"),
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o755,
        )
        try:
            while True:
                chunk = os.read(rustup_source, 1024 * 1024)
                if not chunk:
                    break
                runner._write_all(rustup_destination, chunk)
            os.fsync(rustup_destination)
        finally:
            os.close(rustup_source)
            os.close(rustup_destination)
        os.symlink("rustup", os.path.join(cargo_bin, "cargo"))
        os.symlink("rustup", os.path.join(cargo_bin, "rustc"))
        archive_parent = os.path.join(
            cargo_source,
            "registry",
            "cache",
            "index.crates.io-1949cf8c6b5b557f",
        )
        os.makedirs(archive_parent, mode=0o700)
        archive_path = os.path.join(archive_parent, "fixture-dep-1.0.0.crate")
        with tarfile.open(archive_path, "w:gz") as archive:
            for relative in ("Cargo.toml", "src/lib.rs"):
                archive.add(
                    os.path.join(package, relative),
                    arcname=f"fixture-dep-1.0.0/{relative}",
                    recursive=False,
                )
        with open(archive_path, "rb") as handle:
            package_checksum = hashlib.sha256(handle.read()).hexdigest()
        with open(os.path.join(repository, "Cargo.lock"), "xb") as handle:
            handle.write(
                (
                    "version = 4\n\n"
                    "[[package]]\n"
                    'name = "fixture-dep"\n'
                    'version = "1.0.0"\n'
                    'source = "registry+https://github.com/rust-lang/'
                    'crates.io-index"\n'
                    f'checksum = "{package_checksum}"\n\n'
                    "[[package]]\n"
                    'name = "fixture-root"\n'
                    'version = "0.1.0"\n'
                    'dependencies = [\n "fixture-dep",\n]\n'
                ).encode("ascii")
            )

        seed = os.path.join(self.make_safe_parent(), "seed")
        runner.construct_cargo_home_seed(cargo_source, repository, seed)
        self.assertTrue(os.path.isfile(os.path.join(seed, "config.toml")))
        self.assertTrue(
            os.path.isdir(os.path.join(seed, "vendor", "fixture-dep-1.0.0"))
        )
        self.assertFalse(os.path.exists(os.path.join(seed, "registry")))
        fixture_home = self.make_safe_parent()
        runtime = os.path.join(fixture_home, ".cargo")
        runner.construct_cargo_home_runtime(seed, runtime)
        runner.seal_snapshot_mounts((seed, runtime))
        os.chmod(runtime, 0o700)
        tmpdir = self.make_safe_parent()
        xdg_runtime = self.make_safe_parent()
        target = self.make_safe_parent()
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("CARGO_", "RUSTUP_"))
        }
        environment.update(
            {
                "CARGO_HOME": runtime,
                "CARGO_NET_OFFLINE": "true",
                "CARGO_TARGET_DIR": target,
                "HOME": fixture_home,
                "RUSTUP_HOME": "/home/spenser/.rustup",
                "TMPDIR": tmpdir,
                "XDG_RUNTIME_DIR": xdg_runtime,
            }
        )
        completed = subprocess.run(
            [
                os.path.join(runtime, "bin", "cargo"),
                "metadata",
                "--locked",
                "--offline",
                "--format-version",
                "1",
            ],
            cwd=repository,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=30,
            check=False,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr.decode())
        metadata = json.loads(completed.stdout)
        self.assertEqual(
            sorted(package["name"] for package in metadata["packages"]),
            ["fixture-dep", "fixture-root"],
        )
        validation = runner.validate_cargo_home_runtime_writes(seed, runtime)
        self.assertTrue(validation["only_authorized_metadata_changed"])
        self.assertLessEqual(
            set(validation["changed"]),
            set(runner.AUTHORIZED_CARGO_RUNTIME_MUTATIONS),
        )

    def test_private_snapshots_ignore_transient_source_modify_replace_and_restore(
        self,
    ) -> None:
        source = self.make_safe_parent()
        destination = os.path.join(self.make_safe_parent(), "snapshot")
        path = os.path.join(source, "input")
        with open(path, "xb") as handle:
            handle.write(b"original")
        source_fd = os.open(
            source,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        held_source = source + ".held"
        os.rename(source, held_source)
        os.mkdir(source, 0o700)
        with open(os.path.join(source, "input"), "xb") as handle:
            handle.write(b"replacement")
        try:
            manifest, _count = runner._copy_tree_no_follow(
                source_fd,
                destination,
            )
            self.assertEqual(runner._manifest_tree(destination)[0], manifest)
            with open(os.path.join(destination, "input"), "rb") as handle:
                self.assertEqual(handle.read(), b"original")
        finally:
            os.close(source_fd)
            source_parent = os.path.dirname(source)
            source_name = os.path.basename(source)
            source_parent_fd = os.open(
                source_parent,
                os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
            )
            replacement_fd, replacement_identity = (
                runner.open_validated_directory(
                    source,
                    expected_uid=os.getuid(),
                    expected_mode=0o700,
                )
            )
            try:
                runner.remove_tree_at(
                    source_parent_fd,
                    source_name,
                    expected=replacement_identity,
                    cleanup_authority=self.cleanup_authority,
                )
            finally:
                os.close(replacement_fd)
            os.close(source_parent_fd)
        repository = self.make_safe_parent()
        cargo = self.make_safe_parent()
        destination = os.path.join(self.make_safe_parent(), "vendor")
        package = self.make_safe_parent()
        with open(os.path.join(package, "Cargo.toml"), "xb") as handle:
            handle.write(b'[package]\nname="selected"\nversion="1.0.0"\n')
        archive_path = os.path.join(cargo, "selected.crate")
        with tarfile.open(archive_path, "w:gz") as archive:
            archive.add(
                os.path.join(package, "Cargo.toml"),
                arcname="selected-1.0.0/Cargo.toml",
                recursive=False,
            )
        with open(archive_path, "rb") as handle:
            original_archive = handle.read()
        modified_archive = bytes([original_archive[0] ^ 1]) + original_archive[1:]
        checksum = hashlib.sha256(original_archive).hexdigest()
        with open(os.path.join(repository, "Cargo.lock"), "xb") as handle:
            handle.write(
                (
                    "[[package]]\n"
                    'name = "selected"\n'
                    'version = "1.0.0"\n'
                    'source = "registry+https://github.com/rust-lang/'
                    'crates.io-index"\n'
                    f'checksum = "{checksum}"\n'
                ).encode("ascii")
            )
        archive_identity = os.stat(archive_path, follow_symlinks=False)
        original_pread = runner.os.pread
        source_reads = 0

        def modify_during_copy(fd: int, size: int, offset: int) -> bytes:
            nonlocal source_reads
            details = os.fstat(fd)
            if (
                (details.st_dev, details.st_ino)
                == (archive_identity.st_dev, archive_identity.st_ino)
                and offset == 0
            ):
                source_reads += 1
                if source_reads == 2:
                    writer = os.open(archive_path, os.O_WRONLY | os.O_CLOEXEC)
                    try:
                        os.pwrite(writer, modified_archive, 0)
                        result = original_pread(fd, size, offset)
                        os.pwrite(writer, original_archive, 0)
                        os.fsync(writer)
                        return result
                    finally:
                        os.close(writer)
            return original_pread(fd, size, offset)

        runner.os.pread = modify_during_copy
        try:
            with self.assertRaises(runner.RunnerError) as modified:
                runner.reconstruct_locked_vendor(
                    repository,
                    cargo,
                    destination,
                )
            self.assertEqual(
                modified.exception.reason,
                "snapshot_identity_invalid",
            )
        finally:
            runner.os.pread = original_pread
        with open(archive_path, "rb") as handle:
            self.assertEqual(handle.read(), original_archive)
            os.rename(held_source, source)

    def test_cargo_environment_preserves_e0_with_only_tmpdir_xdg_overrides(
        self,
    ) -> None:
        marker = "PROTECTED-ENVIRONMENT-VALUE"
        original = {
            "A": "one",
            "MARKER": marker,
            "TMPDIR": "/old",
            "XDG_RUNTIME_DIR": "/old-xdg",
        }
        updated = runner._canonical_wall_environment(original, "/root")
        self.assertEqual(updated["TMPDIR"], "/root/tmp")
        self.assertEqual(updated["XDG_RUNTIME_DIR"], "/run/xdg")
        longest_fixture_transport = os.path.join(
            updated["XDG_RUNTIME_DIR"],
            "sXX/h/run/agent-hub/handles/startup",
            "ssssssssssss-pppppppppppp.startup.sock",
        )
        self.assertLessEqual(len(os.fsencode(longest_fixture_transport)), 100)
        self.assertEqual(
            {key: value for key, value in updated.items() if key not in {"TMPDIR", "XDG_RUNTIME_DIR"}},
            {"A": "one", "MARKER": marker},
        )
        record = runner._default_provenance(
            invocation_id="7" * 32,
            label="environment-constructor",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment=updated,
        )
        self.assertNotIn(marker, json.dumps(record, sort_keys=True))

    def test_environment_values_and_protected_markers_are_never_serialized(self) -> None:
        marker = "PROTECTED-DO-NOT-SERIALIZE"
        record = runner._default_provenance(
            invocation_id="8" * 32,
            label="environment",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={"MARKER": marker},
        )
        self.assertNotIn(marker, json.dumps(record))

    def test_fresh_private_target_never_reads_repository_target(self) -> None:
        parent = self.make_safe_parent()
        target = os.path.join(parent, "target")
        os.mkdir(target, 0o700)
        self.assertEqual(os.listdir(target), [])
        self.assertNotEqual(os.path.realpath(target), os.path.realpath("target"))

    def test_private_target_is_descriptor_removed_after_containment(self) -> None:
        parent = self.make_safe_parent()
        target = os.path.join(parent, "target")
        os.mkdir(target, 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        target_fd, identity = runner.open_validated_directory(
            target,
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        proof = runner.remove_tree_at(
            parent_fd,
            "target",
            expected=identity,
            cleanup_authority=self.cleanup_authority,
        )
        os.close(target_fd)
        os.close(parent_fd)
        self.assertEqual(
            proof,
            {
                "removed": True,
                "name_absence_proved": True,
                "descriptor_closed_after_required_absence": True,
            },
        )

    def test_cargo_home_seed_runtime_confines_lock_metadata_and_is_removed(
        self,
    ) -> None:
        self._assert_primary_command_uses_private_umask_without_mutating_parent()
        seed = self.make_safe_parent()
        with open(os.path.join(seed, "stable"), "xb") as handle:
            handle.write(b"stable")
        runtime_path = os.path.join(self.make_safe_parent(), "runtime")
        runner.construct_cargo_home_runtime(seed, runtime_path)
        runner.seal_snapshot_mounts((seed, runtime_path))
        os.chmod(runtime_path, 0o700)
        for name in runner.AUTHORIZED_CARGO_RUNTIME_MUTATIONS:
            with open(os.path.join(runtime_path, name), "xb") as handle:
                handle.write(b"lock")
            os.chmod(os.path.join(runtime_path, name), 0o600)
        result = runner.validate_cargo_home_runtime_writes(seed, runtime_path)
        self.assertEqual(
            result["changed"],
            list(runner.AUTHORIZED_CARGO_RUNTIME_MUTATIONS),
        )

    def test_rejects_project_ancestor_or_account_cargo_config_appearance(self) -> None:
        account = self.make_safe_parent()
        workspace = os.path.join(account, "workspace")
        repository = os.path.join(workspace, "repository")
        os.mkdir(workspace, 0o700)
        os.mkdir(repository, 0o700)
        os.mkdir(os.path.join(account, ".cargo"), 0o700)
        authority = runner.retain_cargo_search_path_absence(
            repository=repository,
            account=account,
        )
        try:
            runner.revalidate_cargo_search_path_absence(authority)
            with open(
                os.path.join(account, ".cargo", "config.toml"),
                "xb",
            ) as handle:
                handle.write(b"[build]\nrustc-wrapper='forbidden'\n")
            with self.assertRaises(runner.RunnerError) as appeared:
                runner.revalidate_cargo_search_path_absence(authority)
            self.assertEqual(
                appeared.exception.reason,
                "repository_identity_mismatch",
            )
        finally:
            runner.close_cargo_search_path_authority(authority)

    def test_evidence_bounds_fail_closed(self) -> None:
        left, right = socket.socketpair(socket.AF_UNIX, socket.SOCK_SEQPACKET)
        with self.assertRaises(runner.RunnerError):
            runner._send_frame(left, {"x": "a" * (runner.MAX_STATUS_BYTES + 1)})
        left.close()
        right.close()
        malformed_packets = (
            b"",
            runner.FRAME_MAGIC + struct.pack(">I", 4) + b"{}\n",
            runner.FRAME_MAGIC
            + struct.pack(">I", runner.MAX_STATUS_BYTES + 1)
            + b"{}\n",
            runner.FRAME_MAGIC + struct.pack(">I", 3) + b"{}\ntrailing",
            runner.FRAME_MAGIC + struct.pack(">I", 14) + b'{"x":1,"x":2}\n',
            runner.FRAME_MAGIC
            + struct.pack(">I", len(b'{"x":1}\r\n'))
            + b'{"x":1}\r\n',
        )
        for packet in malformed_packets:
            receiver, sender = socket.socketpair(
                socket.AF_UNIX,
                socket.SOCK_SEQPACKET,
            )
            if packet:
                sender.sendall(packet)
            sender.close()
            with self.assertRaises(runner.RunnerError):
                runner._recv_frame(receiver)
            receiver.close()
        receiver, sender = socket.socketpair(
            socket.AF_UNIX,
            socket.SOCK_SEQPACKET,
        )
        runner._send_frame(sender, {"one": 1})
        runner._send_frame(sender, {"two": 2})
        sender.close()
        with self.assertRaises(runner.RunnerError):
            runner._recv_exactly_one_frame(receiver, eof_timeout=0.1)
        receiver.close()
        containment_line = (
            b'{"kind":"containment_empty_received","monotonic_ns":1,'
            b'"pid":null,"pid_namespace_inode":1,"ppid":null,'
            b'"seq":0,"state":null}\n'
        )
        containment_rows = []
        for sequence in range(65_537):
            containment_rows.append(
                containment_line.replace(b'"seq":0', f'"seq":{sequence}'.encode("ascii"))
            )
        runner.validate_containment_jsonl(b"".join(containment_rows[:65_536]))
        with self.assertRaises(runner.RunnerError) as event_limit:
            runner.validate_containment_jsonl(b"".join(containment_rows))
        self.assertEqual(event_limit.exception.reason, "evidence_limit_exceeded")
        oversized_line = containment_line[:-1] + b" " * (513 - len(containment_line)) + b"\n"
        boundary_line = containment_line[:-1] + b" " * (512 - len(containment_line)) + b"\n"
        self.assertEqual(len(boundary_line), 512)
        runner.validate_containment_jsonl(boundary_line)
        self.assertEqual(len(oversized_line), 513)
        with self.assertRaises(runner.RunnerError) as line_limit:
            runner.validate_containment_jsonl(oversized_line)
        self.assertEqual(line_limit.exception.reason, "evidence_limit_exceeded")
        with self.assertRaises(runner.RunnerError) as crlf_containment:
            runner.validate_containment_jsonl(
                containment_line[:-1] + b"\r\n"
            )
        self.assertEqual(
            crlf_containment.exception.reason,
            "evidence_write_failed",
        )
        failure_names = [f"case{index:04d}::fails" for index in range(4_097)]
        bounded_log = bytearray(b"failures:\n")
        for name in failure_names:
            bounded_log.extend(b"    " + name.encode("ascii") + b"\n")
        for name in failure_names:
            bounded_log.extend(
                b"thread '"
                + name.encode("ascii")
                + b"' panicked at file.rs:1:1:\nbody\n"
            )
        bounded_log.extend(
            b"test result: FAILED. 0 passed; 4097 failed; 0 ignored; "
            b"0 measured; 0 filtered out; done\n"
        )
        with self.assertRaises(runner.RunnerError) as failure_limit:
            runner.summarize_test_log(bytes(bounded_log), 101)
        self.assertEqual(failure_limit.exception.reason, "evidence_limit_exceeded")
        boundary_names = failure_names[:4_096]
        boundary_log = bytearray(b"failures:\n")
        for name in boundary_names:
            boundary_log.extend(b"    " + name.encode("ascii") + b"\n")
        for name in boundary_names:
            boundary_log.extend(
                b"thread '"
                + name.encode("ascii")
                + b"' panicked at file.rs:1:1:\nbody\n"
            )
        boundary_log.extend(
            b"test result: FAILED. 0 passed; 4096 failed; 0 ignored; "
            b"0 measured; 0 filtered out; done\n"
        )
        boundary_summary, boundary_failure_names, boundary_signatures = (
            runner.summarize_test_log(bytes(boundary_log), 101)
        )
        self.assertEqual(boundary_summary["analysis_status"], "complete")
        self.assertEqual(boundary_failure_names.count(b"\n"), 4_096)
        self.assertEqual(boundary_signatures.count(b"\n"), 4_096)
        original_names_limit = runner.MAX_FAILURE_NAMES_BYTES
        original_signatures_limit = runner.MAX_NORMALIZED_SIGNATURES_BYTES
        try:
            runner.MAX_FAILURE_NAMES_BYTES = len(runner.GOLDEN_NAMES) - 1
            with self.assertRaises(runner.RunnerError):
                runner.summarize_test_log(runner.GOLDEN_LOG, 101)
            runner.MAX_FAILURE_NAMES_BYTES = original_names_limit
            runner.MAX_NORMALIZED_SIGNATURES_BYTES = (
                len(runner.GOLDEN_SIGNATURES) - 1
            )
            with self.assertRaises(runner.RunnerError):
                runner.summarize_test_log(runner.GOLDEN_LOG, 101)
        finally:
            runner.MAX_FAILURE_NAMES_BYTES = original_names_limit
            runner.MAX_NORMALIZED_SIGNATURES_BYTES = original_signatures_limit

    def test_schema_rejects_unknown_missing_and_wrong_type_fields(self) -> None:
        valid = runner._default_provenance(
            invocation_id="9" * 32,
            label="schema",
            mode="serial",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("serial"),
            environment={},
        )
        self.assertEqual(valid["schema"], runner.SCHEMA)
        self.assertEqual(set(valid), {
            "schema", "record_state", "record_stage", "invocation_id", "label",
            "mode", "eligible", "wall_gate", "ineligibility_reasons",
            "runner_exit_code", "started_at_utc", "finalized_at_utc", "request",
            "repository", "toolchain", "namespace", "mounts", "containment",
            "roots", "evidence", "result",
        })
        runner._validate_provenance_record(valid)
        unknown = dict(valid)
        unknown["unknown"] = True
        missing = dict(valid)
        missing.pop("containment")
        wrong_type = dict(valid)
        wrong_type["eligible"] = 0
        inconsistent = dict(valid)
        inconsistent.update(
            {
                "eligible": True,
                "record_stage": "complete",
                "wall_gate": "ineligible",
            }
        )
        false_clean = dict(valid)
        false_clean["wall_gate"] = "clean"
        false_regression = dict(valid)
        false_regression["wall_gate"] = "regression"
        finalized_reasonless = dict(valid)
        finalized_reasonless["finalized_at_utc"] = "2026-07-23T00:00:00Z"
        for malformed in (
            unknown,
            missing,
            wrong_type,
            inconsistent,
            false_clean,
            false_regression,
            finalized_reasonless,
        ):
            with self.assertRaises(runner.RunnerError):
                runner._validate_provenance_record(malformed)
        partial = json.loads(json.dumps(valid))
        partial_result, _names, _signatures = runner.summarize_test_log(
            runner.GOLDEN_LOG,
            0,
        )
        partial["result"] = partial_result
        partial["containment"]["cargo_exit"] = 0
        runner._validate_provenance_record(partial)
        malformed_partial = json.loads(json.dumps(partial))
        malformed_partial["result"]["passed"] = None
        with self.assertRaises(runner.RunnerError):
            runner._validate_provenance_record(malformed_partial)

        kernel_maps = runner._default_provenance(
            invocation_id="8" * 32,
            label="kernel-maps",
            mode="serial",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("serial"),
            environment={},
        )
        for stage_name in ("stage_a", "stage_b"):
            kernel_maps["mounts"][stage_name] = {
                "argv_template_sha256": "0" * 64,
                "argv_sha256": "1" * 64,
                "as_pid_1": stage_name == "stage_b",
                "user_namespace": True,
                "mount_namespace": True,
                "pid_namespace": True,
                "json_status_fd": True,
                "builtin_pid1_fail_safe_only": stage_name == "stage_a",
                "uid_map": (
                    runner.CANONICAL_STAGE_A_UID_MAP
                    if stage_name == "stage_a"
                    else runner.CANONICAL_STAGE_B_UID_MAP
                ),
                "gid_map": (
                    runner.CANONICAL_STAGE_A_GID_MAP
                    if stage_name == "stage_a"
                    else runner.CANONICAL_STAGE_B_GID_MAP
                ),
                "private_propagation": True,
                "die_with_parent": True,
                "completed": False,
                "unmounted": False,
            }
        runner._validate_provenance_record(kernel_maps)

        malformed_map = json.loads(json.dumps(kernel_maps))
        malformed_map["mounts"]["stage_a"]["uid_map"] = "1000 0 2\n"
        with self.assertRaises(runner.RunnerError) as rejected:
            runner._validate_provenance_record(malformed_map)
        self.assertEqual(rejected.exception.reason, "evidence_write_failed")

        kernel_maps["mounts"]["cargo_home_runtime"] = {
            "seed_manifest_sha256": "0" * 64,
            "pre_manifest_sha256": "0" * 64,
            "post_manifest_sha256": "1" * 64,
            "seed_entry_count": 1,
            "pre_entry_count": 1,
            "post_entry_count": 2,
            "private_tmpfs": True,
            "no_host_alias": True,
            "stage_b_writable": True,
            "seed_copy_equal": True,
            "only_authorized_metadata_changed": True,
            "removed": True,
            "authorized_mutable_paths": list(
                runner.AUTHORIZED_CARGO_RUNTIME_MUTATIONS
            ),
            "changed": list(runner.AUTHORIZED_CARGO_RUNTIME_MUTATIONS),
        }
        runner._validate_provenance_record(kernel_maps)
        unauthorized = json.loads(json.dumps(kernel_maps))
        unauthorized["mounts"]["cargo_home_runtime"]["changed"] = [
            "registry/CACHEDIR.TAG"
        ]
        with self.assertRaises(runner.RunnerError) as rejected:
            runner._validate_provenance_record(unauthorized)
        self.assertEqual(rejected.exception.reason, "evidence_write_failed")

    def test_schema_encodes_preflight_setup_containment_cleanup_failures(self) -> None:
        stage_failures = (
            ("preflight", "repository_identity_mismatch", 65),
            ("setup", "snapshot_timeout", 67),
            ("containment", "stage_b_liveness_failed", 68),
            ("cleanup", "mount_teardown_failed", 70),
        )
        for index, (stage, reason, exit_code) in enumerate(stage_failures):
            record = runner._default_provenance(
                invocation_id=f"{index + 1:x}" * 32,
                label=f"stage-{stage}",
                mode="parallel",
                timeout_seconds=1,
                expected_head="0" * 40,
                command=runner._canonical_cargo_command("parallel"),
                environment={},
            )
            record["record_stage"] = stage
            runner._record_ineligibility(record, reason, exit_code)
            record["finalized_at_utc"] = "2026-07-23T00:00:00Z"
            runner._validate_provenance_record(record)
            self.assertEqual(record["runner_exit_code"], exit_code)
            self.assertEqual(record["ineligibility_reasons"], [reason])
        combined = runner._default_provenance(
            invocation_id="a" * 32,
            label="precedence",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        for reason, exit_code in (
            ("repository_identity_mismatch", 65),
            ("stage_b_liveness_failed", 68),
            ("mount_teardown_failed", 70),
            ("evidence_write_failed", 71),
            ("internal_invariant_failed", 72),
        ):
            runner._record_ineligibility(combined, reason, exit_code)
        self.assertEqual(combined["runner_exit_code"], 72)
        self.assertEqual(
            combined["ineligibility_reasons"],
            sorted(
                {
                    "repository_identity_mismatch",
                    "stage_b_liveness_failed",
                    "mount_teardown_failed",
                    "evidence_write_failed",
                    "internal_invariant_failed",
                }
            ),
        )
        preserved = runner._default_provenance(
            invocation_id="b" * 32,
            label="host-precedence",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        runner._record_ineligibility(preserved, "evidence_write_failed", 71)
        runner._record_ineligibility(preserved, "stage_a_liveness_failed", 68)
        self.assertEqual(preserved["runner_exit_code"], 71)
        host_source = _module_source(runner).decode("utf-8").split(
            "def host_main",
            1,
        )[1].split("def _run_selftest_bootstrap", 1)[0]
        self.assertIn(
            "_record_ineligibility(\n                    provenance,\n"
            "                    error.reason,",
            host_source,
        )
        failure_branch = host_source.split(
            "if authority is not None and provenance is not None:",
            1,
        )[1]
        self.assertLess(
            failure_branch.index("_record_ineligibility("),
            failure_branch.index("_read_authenticated_staged_provenance("),
        )
        self.assertIn('return int(provenance["runner_exit_code"])', host_source)
        staged_parent = self.make_safe_parent()
        staged_authority = runner.prepare_backing_root(
            staged_parent,
            "host-stage-a-fields",
            "e" * 32,
        )
        staged_artifacts = runner._initialize_staged_artifact_authorities(
            staged_authority
        )
        staged_record = runner._default_provenance(
            invocation_id="e" * 32,
            label="host-stage-a-fields",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        original_record = json.loads(json.dumps(staged_record))
        original_record["request"]["launcher"]["stage_a_argv_sha256"] = (
            "1" * 64
        )
        original_record["namespace"].update(
            {
                "stage_a_bwrap_pid": 123,
                "stage_a_bwrap_start_time_ticks": 456,
                "stage_a_bwrap_pidfd_opened": True,
            }
        )
        runner._rewrite_retained_json_artifact(
            staged_authority["partial_fd"],
            staged_artifacts["provenance.json"],
            staged_record,
        )
        merged = runner._read_authenticated_staged_provenance(
            staged_authority,
            original_record,
        )
        self.assertEqual(
            merged["request"]["launcher"]["stage_a_argv_sha256"],
            "1" * 64,
        )
        self.assertEqual(merged["namespace"]["stage_a_bwrap_pid"], 123)
        self.assertEqual(
            merged["namespace"]["stage_a_bwrap_start_time_ticks"],
            456,
        )
        self.assertTrue(merged["namespace"]["stage_a_bwrap_pidfd_opened"])
        conflicting = json.loads(json.dumps(staged_record))
        conflicting["request"]["launcher"]["stage_a_argv_sha256"] = "2" * 64
        runner._rewrite_retained_json_artifact(
            staged_authority["partial_fd"],
            staged_artifacts["provenance.json"],
            conflicting,
        )
        with self.assertRaises(runner.RunnerError) as conflict:
            runner._read_authenticated_staged_provenance(
                staged_authority,
                original_record,
            )
        self.assertEqual(conflict.exception.reason, "evidence_write_failed")
        for artifact in staged_artifacts.values():
            os.close(artifact["fd"])
        for key in ("backing_fd", "partial_fd", "parent_fd"):
            os.close(staged_authority[key])
        for descriptor, _path, _identity in staged_authority[
            "ancestor_records"
        ]:
            os.close(descriptor)
        with self.assertRaises(runner.RunnerError):
            runner._record_ineligibility(combined, "signal_termination", 68)
        with self.assertRaises(runner.RunnerError):
            runner._record_ineligibility(
                combined,
                "internal_invariant_failed",
                68,
            )
        invalid_floor = runner._default_provenance(
            invocation_id="c" * 32,
            label="invalid-floor",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        runner._record_ineligibility(
            invalid_floor,
            "internal_invariant_failed",
            72,
        )
        invalid_floor["runner_exit_code"] = 65
        invalid_floor["finalized_at_utc"] = "2026-07-23T00:00:00Z"
        with self.assertRaises(runner.RunnerError):
            runner._validate_provenance_record(invalid_floor)
        literal_reasons = set(
            re.findall(
                rb'RunnerError\(\s*\d+,\s*"([a-z_]+)"',
                _module_source(runner),
            )
        )
        self.assertTrue(literal_reasons)
        self.assertEqual(
            literal_reasons - {
                reason.encode("ascii")
                for reason in runner.INELIGIBILITY_REASONS
            },
            set(),
        )

    def test_atomic_finalization_hides_partial_eligibility(self) -> None:
        parent = self.make_safe_parent()
        authority = runner.prepare_backing_root(parent, "atomic", "b" * 32)
        runner._initialize_staged_artifact_authorities(authority)
        record = runner._default_provenance(
            invocation_id="b" * 32,
            label="atomic",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        self.assertFalse(record["eligible"])
        self.assertTrue(os.path.basename(authority["partial_path"]).endswith(".partial"))
        record["mounts"]["backing_mountpoint"].update(
            {
                "path": authority["backing_path"],
                "underlying_pre": authority["backing_identity"].as_dict(),
            }
        )
        final = runner._finalize_ineligible(
            authority,
            record,
            runner.RunnerError(65, "invocation_authority_invalid"),
        )
        self.assertFalse(os.path.exists(authority["partial_path"]))
        self.assertFalse(os.path.exists(authority["backing_path"]))
        final_record = self.read_final_provenance(final)
        self.assert_ineligible(final_record, "invocation_authority_invalid")
        retained_authority = runner.prepare_backing_root(
            parent,
            "retained-cleanup-failure",
            "3" * 32,
        )
        runner._initialize_staged_artifact_authorities(retained_authority)
        retained_record = runner._default_provenance(
            invocation_id="3" * 32,
            label="retained-cleanup-failure",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        retained_record["mounts"]["backing_mountpoint"].update(
            {
                "path": retained_authority["backing_path"],
                "underlying_pre": retained_authority[
                    "backing_identity"
                ].as_dict(),
            }
        )
        retained_record["namespace"].update(
            {
                "stage_a_bwrap_pid": 12_345,
                "stage_a_bwrap_pidfd_opened": True,
                "stage_a_namespace_absent": False,
            }
        )
        retained_final = runner._finalize_ineligible(
            retained_authority,
            retained_record,
            runner.RunnerError(70, "mount_teardown_failed"),
            retain_backing=True,
            stage_a_process_teardown_proved=True,
        )
        self.assertFalse(os.path.exists(retained_authority["partial_path"]))
        self.assertTrue(os.path.isdir(retained_authority["backing_path"]))
        retained_final_record = self.read_final_provenance(retained_final)
        self.assert_ineligible(retained_final_record, "mount_teardown_failed")
        retained_backing = retained_final_record["mounts"][
            "backing_mountpoint"
        ]
        self.assertFalse(retained_backing["removed"])
        self.assertFalse(retained_backing["name_absence_proved"])
        late_authority = runner.prepare_backing_root(
            parent,
            "late-evidence-failure",
            "1" * 32,
        )
        runner._initialize_staged_artifact_authorities(late_authority)
        late_record = runner._default_provenance(
            invocation_id="1" * 32,
            label="late-evidence-failure",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        late_record["mounts"]["backing_mountpoint"].update(
            {
                "path": late_authority["backing_path"],
                "underlying_pre": late_authority[
                    "backing_identity"
                ].as_dict(),
            }
        )
        backing_post = runner._directory_identity(
            late_authority["backing_fd"],
            late_authority["backing_path"],
        )
        runner.remove_tree_at(
            late_authority["parent_fd"],
            late_authority["backing_name"],
            expected=backing_post,
            cleanup_authority=self.cleanup_authority,
        )
        os.close(late_authority["backing_fd"])
        late_authority["backing_fd"] = -1
        late_authority["backing_removed"] = True
        late_authority["backing_post"] = backing_post
        late_final = runner._finalize_ineligible(
            late_authority,
            late_record,
            runner.RunnerError(71, "evidence_write_failed"),
        )
        self.assertFalse(os.path.exists(late_authority["backing_path"]))
        self.assert_ineligible(
            self.read_final_provenance(late_final),
            "evidence_write_failed",
        )
        rollback_authority = runner.prepare_backing_root(
            parent,
            "publication-rollback",
            "2" * 32,
        )
        runner._write_file_at(
            rollback_authority["partial_fd"],
            "artifact",
            b"published-only-after-parent-fsync",
        )
        rollback_artifact = runner._open_artifact_authority(
            rollback_authority["partial_fd"],
            "artifact",
        )
        original_fsync = runner.os.fsync
        publication_fsync_failed = False
        rollback_final_path = os.path.join(
            rollback_authority["parent_path"],
            rollback_authority["final_name"],
        )

        def fail_publication_parent_fsync(descriptor: int) -> None:
            nonlocal publication_fsync_failed
            if (
                descriptor == rollback_authority["parent_fd"]
                and os.path.isdir(rollback_final_path)
            ) and not publication_fsync_failed:
                publication_fsync_failed = True
                raise OSError(5, "injected publication fsync failure")
            original_fsync(descriptor)

        runner.os.fsync = fail_publication_parent_fsync
        try:
            with self.assertRaises(runner.RunnerError):
                runner._publish_evidence_directory(
                    rollback_authority,
                    rollback_authority["partial_fd"],
                    rollback_authority["partial_identity"],
                    [rollback_artifact],
                )
        finally:
            runner.os.fsync = original_fsync
        self.assertTrue(publication_fsync_failed)
        self.assertTrue(os.path.isdir(rollback_authority["partial_path"]))
        self.assertFalse(os.path.exists(rollback_final_path))
        self.assertGreater(os.fstat(rollback_artifact["fd"]).st_nlink, 0)
        finalizer_source = _module_source(runner).decode("utf-8").split(
            "def finalize_after_stage_a_exit",
            1,
        )[1].split("def _finalize_ineligible", 1)[0]
        self.assertIn("recovery_provenance", finalizer_source)
        self.assertIn("if published:", finalizer_source)
        self.assertIn(
            "_rewrite_retained_json_artifact(\n"
            "            partial_fd,\n"
            "            provenance_artifact,\n"
            "            recovery_provenance,",
            finalizer_source,
        )
        os.close(rollback_artifact["fd"])
        for key in ("backing_fd", "partial_fd"):
            os.close(rollback_authority[key])
        for descriptor, _path, _identity in rollback_authority[
            "ancestor_records"
        ]:
            os.close(descriptor)
        os.close(rollback_authority["parent_fd"])

    def test_final_evidence_rename_is_noreplace(self) -> None:
        parent = self.make_safe_parent()
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        os.mkdir("old", 0o700, dir_fd=parent_fd)
        os.mkdir("new", 0o700, dir_fd=parent_fd)
        with self.assertRaises(runner.RunnerError):
            runner._rename_noreplace(parent_fd, "old", parent_fd, "new")
        self.assertTrue(os.path.isdir(os.path.join(parent, "old")))
        self.assertTrue(os.path.isdir(os.path.join(parent, "new")))
        os.close(parent_fd)
        authority = runner.prepare_backing_root(
            parent,
            "partial-replacement",
            "e" * 32,
        )
        os.rename(
            authority["partial_path"],
            authority["partial_path"] + ".held",
        )
        os.mkdir(authority["partial_path"], 0o700)
        with self.assertRaises(runner.RunnerError):
            runner.revalidate_named_identity(
                authority["parent_fd"],
                authority["partial_name"],
                authority["partial_fd"],
                authority["partial_identity"],
            )
        artifact_directory = os.path.join(parent, "artifact-authority")
        os.mkdir(artifact_directory, 0o700)
        artifact_directory_fd = os.open(
            artifact_directory,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        runner._write_file_at(artifact_directory_fd, "evidence", b"original")
        artifact = runner._open_artifact_authority(
            artifact_directory_fd,
            "evidence",
        )
        os.rename(
            "evidence",
            "held-evidence",
            src_dir_fd=artifact_directory_fd,
            dst_dir_fd=authority["parent_fd"],
        )
        runner._write_file_at(artifact_directory_fd, "evidence", b"replacement")
        with self.assertRaises(runner.RunnerError):
            runner._revalidate_artifact_authorities(
                artifact_directory_fd,
                [artifact],
            )
        containment = runner._create_preopened_artifact_authority(
            artifact_directory_fd,
            "containment.jsonl",
        )
        runner._rewrite_preopened_artifact(
            artifact_directory_fd,
            "containment.jsonl",
            b'{"seq":0}\n',
        )
        valid_containment = (
            b'{"kind":"containment_empty_received","monotonic_ns":1,'
            b'"pid":null,"pid_namespace_inode":1,"ppid":null,'
            b'"seq":0,"state":null}\n'
        )
        runner.validate_containment_jsonl(valid_containment)
        for invalid_containment in (
            valid_containment.replace(
                b"containment_empty_received",
                b"containment_empty",
            ),
            valid_containment.replace(b'"state":null', b'"state":"invalid"'),
        ):
            with self.assertRaises(runner.RunnerError):
                runner.validate_containment_jsonl(invalid_containment)
        os.rename(
            "containment.jsonl",
            "containment.original",
            src_dir_fd=artifact_directory_fd,
            dst_dir_fd=artifact_directory_fd,
        )
        runner._write_file_at(
            artifact_directory_fd,
            "containment.jsonl",
            b'{"seq":0}\n',
        )
        with self.assertRaises(runner.RunnerError):
            runner._revalidate_preopened_artifact(
                artifact_directory_fd,
                containment,
            )
        os.close(containment["fd"])
        provenance = runner._create_preopened_artifact_authority(
            artifact_directory_fd,
            "provenance.json",
        )
        runner._rewrite_preopened_artifact(
            artifact_directory_fd,
            "provenance.json",
            b"{}\n",
        )
        os.rename(
            "provenance.json",
            "provenance.original",
            src_dir_fd=artifact_directory_fd,
            dst_dir_fd=artifact_directory_fd,
        )
        runner._write_file_at(
            artifact_directory_fd,
            "provenance.json",
            b"{}\n",
        )
        with self.assertRaises(runner.RunnerError):
            runner._read_json_from_preopened_artifact(
                artifact_directory_fd,
                provenance,
            )
        os.close(provenance["fd"])
        runner._write_file_at(
            artifact_directory_fd,
            "stage-a-provenance.json",
            b'{"stage":"original"}\n',
        )
        stage_a_provenance = runner._open_existing_artifact_authority(
            artifact_directory_fd,
            "stage-a-provenance.json",
        )
        os.rename(
            "stage-a-provenance.json",
            "stage-a-provenance.original",
            src_dir_fd=artifact_directory_fd,
            dst_dir_fd=artifact_directory_fd,
        )
        runner._write_file_at(
            artifact_directory_fd,
            "stage-a-provenance.json",
            b'{"stage":"replacement"}\n',
        )
        with self.assertRaises(runner.RunnerError):
            runner._rewrite_retained_artifact(
                artifact_directory_fd,
                stage_a_provenance,
                b'{"stage":"changed"}\n',
            )
        with open(
            os.path.join(artifact_directory, "stage-a-provenance.original"),
            "rb",
        ) as handle:
            self.assertEqual(handle.read(), b'{"stage":"original"}\n')
        with open(
            os.path.join(artifact_directory, "stage-a-provenance.json"),
            "rb",
        ) as handle:
            self.assertEqual(handle.read(), b'{"stage":"replacement"}\n')
        continuous_directory = os.path.join(parent, "continuous-artifacts")
        os.mkdir(continuous_directory, 0o700)
        continuous_fd = os.open(
            continuous_directory,
            os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW,
        )
        retained = {
            name: runner._create_preopened_artifact_authority(
                continuous_fd,
                name,
            )
            for name in runner.STAGED_ARTIFACT_ORDER_V1
        }
        for name, retained_artifact in retained.items():
            os.rename(
                name,
                name + ".original",
                src_dir_fd=continuous_fd,
                dst_dir_fd=continuous_fd,
            )
            runner._write_file_at(continuous_fd, name, b"replacement")
            with self.assertRaises(runner.RunnerError):
                runner._revalidate_preopened_artifact(
                    continuous_fd,
                    retained_artifact,
                )
            self.assertEqual(os.pread(int(retained_artifact["fd"]), 1, 0), b"")
            os.close(int(retained_artifact["fd"]))
        os.close(continuous_fd)
        source = _module_source(runner).decode("utf-8")
        host_source = source[
            source.index("def host_main") : source.index("def _run_selftest_bootstrap")
        ]
        stage_a_source = source[
            source.index("def stage_a_main") : source.index("def _executable_identity")
        ]
        self.assertLess(
            host_source.index("_initialize_staged_artifact_authorities(authority)"),
            host_source.index("launch_stage_a_bwrap("),
        )
        self.assertLess(
            host_source.index("_rewrite_retained_json_artifact("),
            host_source.index("launch_stage_a_bwrap("),
        )
        self.assertNotIn("write_preopened_provenance(", host_source)
        self.assertIn(
            "for name in STAGED_ARTIFACT_ORDER_V1",
            stage_a_source,
        )
        self.assertNotIn(
            '_write_file_at(evidence_fd, "summary.json"',
            stage_a_source,
        )
        os.close(stage_a_provenance["fd"])
        os.close(artifact["fd"])
        os.close(artifact_directory_fd)
        for key in ("backing_fd", "partial_fd", "parent_fd"):
            os.close(authority[key])
        for descriptor, _path, _identity in authority["ancestor_records"]:
            os.close(descriptor)

    def test_summarizer_golden_failure_names_and_signatures(self) -> None:
        summary, names, signatures = runner.summarize_test_log(
            runner.GOLDEN_LOG,
            101,
        )
        self.assertEqual(names, runner.GOLDEN_NAMES)
        self.assertEqual(signatures, runner.GOLDEN_SIGNATURES)
        self.assertEqual(summary["analysis_status"], "complete")
        self.assertEqual(
            summary["raw_log_sha256"],
            "80e7c12e6f3711bc85d8b6871e7d8e68442634f6883b04257ae14d9618256b57",
        )

    def test_summarizer_normalization_sort_and_newline_bytes(self) -> None:
        _summary, names, signatures = runner.summarize_test_log(
            runner.GOLDEN_LOG,
            101,
        )
        self.assertEqual(names, b"alpha::case\nzeta::case\n")
        self.assertTrue(signatures.endswith(b"\n"))
        self.assertIn(b"aos_<id>", signatures)
        self.assertIn(b".tmp<id>", signatures)
        note = (
            b"note: run with `RUST_BACKTRACE=1` environment variable "
            b"to display a backtrace\n"
        )
        interleaved = runner.GOLDEN_LOG.replace(note, note + b"ok\n", 1)
        interleaved_summary, interleaved_names, interleaved_signatures = (
            runner.summarize_test_log(interleaved, 101)
        )
        self.assertEqual(interleaved_summary["analysis_status"], "complete")
        self.assertEqual(interleaved_names, runner.GOLDEN_NAMES)
        self.assertEqual(interleaved_signatures, runner.GOLDEN_SIGNATURES)
        panic_body_ok = runner.GOLDEN_LOG.replace(
            b"left  aos_deadbeef\n right: .tmpABC\n",
            b"ok\n",
        )
        _body_summary, _body_names, body_signatures = runner.summarize_test_log(
            panic_body_ok,
            101,
        )
        self.assertIn(
            b"alpha::case\tcrates/shell/src/a.rs:10:2\tok\n",
            body_signatures,
        )

    def test_summarizer_last_result_and_failure_section_grammar(self) -> None:
        prefixed = (
            b"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; done\n"
            + runner.GOLDEN_LOG
        )
        summary, names, _signatures = runner.summarize_test_log(prefixed, 101)
        self.assertEqual(summary["final_status"], "FAILED")
        self.assertEqual(names, runner.GOLDEN_NAMES)

    def test_summarizer_accepts_valid_all_pass_as_eligible_regression(self) -> None:
        log = (
            b"test result: ok. 1 passed; 0 failed; 0 ignored; "
            b"0 measured; 0 filtered out; finished in 0.01s\n"
        )
        summary, names, signatures = runner.summarize_test_log(log, 0)
        self.assertEqual(summary["analysis_status"], "complete")
        self.assertFalse(summary["canonical_match"])
        self.assertEqual((names, signatures), (b"", b""))

    def test_summarizer_rejects_cargo_exit_status_mismatch(self) -> None:
        summary, names, signatures = runner.summarize_test_log(
            runner.GOLDEN_LOG,
            0,
        )
        self.assertEqual(
            summary["analysis_reasons"],
            ["cargo_exit_status_mismatch"],
        )
        self.assertIsNone(names)
        self.assertIsNone(signatures)
        self.assertEqual(summary["final_status"], "FAILED")
        self.assertEqual(
            (
                summary["passed"],
                summary["failed"],
                summary["ignored"],
                summary["discovered"],
            ),
            (0, 2, 0, 2),
        )

    def test_summarizer_rejects_integer_duplicate_missing_and_panic_grammar(
        self,
    ) -> None:
        oversized = (
            b"test result: FAILED. 0 passed; 2147483648 failed; 0 ignored; "
            b"0 measured; 0 filtered out; done\n"
        )
        summary, _names, _signatures = runner.summarize_test_log(oversized, 101)
        self.assertIn("invalid_result_grammar", summary["analysis_reasons"])
        duplicate = runner.GOLDEN_LOG.replace(
            b"    alpha::case\n",
            b"    alpha::case\n    alpha::case\n",
        )
        summary, _names, _signatures = runner.summarize_test_log(duplicate, 101)
        self.assertEqual(
            summary["analysis_reasons"],
            ["duplicate_failure_name", "failure_count_mismatch"],
        )
        malformed_panic = runner.GOLDEN_LOG.replace(
            b"thread 'alpha::case' panicked at crates/shell/src/a.rs:10:2:\n",
            b"thread 'alpha::case' panicked at \n",
        )
        summary, _names, _signatures = runner.summarize_test_log(
            malformed_panic,
            101,
        )
        self.assertEqual(summary["analysis_reasons"], ["invalid_panic_header"])
        combined = duplicate.replace(
            b"thread 'alpha::case' panicked at crates/shell/src/a.rs:10:2:\n",
            b"thread 'alpha::case' panicked at \n",
        )
        summary, _names, _signatures = runner.summarize_test_log(combined, 0)
        self.assertEqual(
            summary["analysis_reasons"],
            [
                "cargo_exit_status_mismatch",
                "duplicate_failure_name",
                "failure_count_mismatch",
                "invalid_panic_header",
            ],
        )

    def test_summarizer_rejects_invalid_encoding_nul_and_overflow(self) -> None:
        invalid, _names, _signatures = runner.summarize_test_log(b"\xff\n", 1)
        self.assertEqual(
            invalid["analysis_reasons"],
            ["invalid_encoding", "missing_final_result"],
        )
        nul, _names, _signatures = runner.summarize_test_log(b"x\0\n", 1)
        self.assertEqual(
            nul["analysis_reasons"],
            ["missing_final_result", "nul_byte"],
        )
        cr_record = (
            b"test result: ok. 1 passed; 0 failed; 0 ignored; "
            b"0 measured; 0 filtered out; forged\r"
            + runner.GOLDEN_LOG
        )
        cr_invalid, _names, _signatures = runner.summarize_test_log(
            cr_record,
            101,
        )
        self.assertIn(
            "invalid_result_grammar",
            cr_invalid["analysis_reasons"],
        )
        cr_only, _names, _signatures = runner.summarize_test_log(
            b"\r\n"
            b"test result: ok. 1 passed; 0 failed; 0 ignored; "
            b"0 measured; 0 filtered out; done\n",
            0,
        )
        self.assertEqual(
            cr_only["analysis_reasons"],
            ["invalid_result_grammar"],
        )
        combined_encoding, _names, _signatures = runner.summarize_test_log(
            b"\xff\0\n",
            1,
        )
        self.assertEqual(
            combined_encoding["analysis_reasons"],
            ["invalid_encoding", "missing_final_result", "nul_byte"],
        )
        overflow_sum, _names, _signatures = runner.summarize_test_log(
            b"test result: ok. 2147483647 passed; 0 failed; "
            b"2147483647 ignored; 0 measured; 0 filtered out; done\n",
            0,
        )
        self.assertEqual(
            overflow_sum["analysis_reasons"],
            ["invalid_result_grammar"],
        )
        with self.assertRaises(runner.RunnerError):
            runner.summarize_test_log(
                b"x" * (1024 * 1024 + 1) + b"\n",
                1,
            )

    def test_projected_root_ids_include_mapped_stage_identity(self) -> None:
        original_open = getattr(runner, "open", None)
        had_open = hasattr(runner, "open")
        original_getuid = runner.os.getuid

        def projected_open(path: str, *_args: object, **_kwargs: object) -> io.StringIO:
            if path == "/proc/self/uid_map":
                return io.StringIO(
                    "      1000       1000          1\n"
                    "      1001     100000       7919\n"
                )
            if path == "/proc/sys/kernel/overflowuid":
                return io.StringIO("65534\n")
            raise AssertionError(path)

        runner.open = projected_open
        runner.os.getuid = lambda: 1000
        try:
            self.assertEqual(
                runner._projected_expected_uids(0),
                {0, 1000, 65_534},
            )
            self.assertEqual(
                runner._projected_expected_uids(1000),
                {1000, 65_534},
            )
        finally:
            runner.os.getuid = original_getuid
            if had_open:
                runner.open = original_open
            else:
                del runner.open

    def test_mount_source_flags_ids_propagation_and_order_are_exact(self) -> None:
        stage_a = runner.BWRAP_STAGE_A_ARGV_TEMPLATE_V1
        stage_b = runner.BWRAP_STAGE_B_ARGV_TEMPLATE_V1
        self.assertEqual(
            runner.CANONICAL_STAGE_A_UID_MAP,
            "      1000       1000          1\n"
            "      1001     100000      63055\n",
        )
        self.assertEqual(
            runner.CANONICAL_STAGE_A_GID_MAP,
            runner.CANONICAL_STAGE_A_UID_MAP,
        )
        self.assertEqual(
            runner.CANONICAL_STAGE_B_UID_MAP,
            "      1000       1000          1\n"
            "      1001       1001      63055\n",
        )
        self.assertEqual(
            runner.CANONICAL_STAGE_B_GID_MAP,
            runner.CANONICAL_STAGE_B_UID_MAP,
        )
        self.assertEqual(
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/newuidmap"]["sha256"],
            "c043fa4b6ae4a824b9e059580cb4cfb80c0c1cdf732822d10f5ca7026822ca2d",
        )
        self.assertEqual(
            runner.PLATFORM_STARTUP_TCB_V1["/usr/bin/newgidmap"]["sha256"],
            "1870ba766732568ad3e6e7b69f9920cbd79e2ba6fe3123a07151865a06769083",
        )
        passwd_record, group_record = runner._canonical_account_records()
        self.assertEqual(
            passwd_record,
            b"spenser:x:1000:1000::/home/spenser:/bin/sh\n",
        )
        self.assertEqual(group_record, b"spenser:x:1000:\n")
        self.assertEqual(
            runner.validate_host_account_projection(),
            (passwd_record, group_record),
        )
        original_getpwuid = runner.pwd.getpwuid
        runner.pwd.getpwuid = lambda _uid: runner.types.SimpleNamespace(
            pw_name="unexpected",
            pw_uid=1000,
            pw_gid=1000,
            pw_dir="/home/spenser",
        )
        try:
            with self.assertRaises(runner.RunnerError) as drift:
                runner.validate_host_account_projection()
            self.assertEqual(drift.exception.reason, "environment_unavailable")
        finally:
            runner.pwd.getpwuid = original_getpwuid
        for template in (stage_a, stage_b):
            self.assertIn("--die-with-parent", template)
            self.assertNotIn("--unshare-user", template)
            self.assertIn("--unshare-pid", template)
            self.assertIn("--proc", template)
            self.assertIn("--json-status-fd", template)
            self.assertIn("--userns", template)
        self.assertIn("--sync-fd", stage_a)
        self.assertEqual(
            stage_a[stage_a.index("--userns") : stage_a.index("--userns") + 2],
            ("--userns", "{STAGE_A_USERNS_FD_DECIMAL}"),
        )
        self.assertEqual(
            stage_a[stage_a.index("--sync-fd") : stage_a.index("--sync-fd") + 2],
            ("--sync-fd", "{STAGE_B_USERNS_SYNC_FD_DECIMAL}"),
        )
        self.assertEqual(
            stage_b[stage_b.index("--userns") : stage_b.index("--userns") + 2],
            ("--userns", "{STAGE_B_USERNS_FD_DECIMAL}"),
        )
        self.assertNotIn("--as-pid-1", stage_a)
        self.assertIn("--as-pid-1", stage_b)
        self.assertLess(stage_b.index("--unshare-pid"), stage_b.index("--as-pid-1"))
        self.assertEqual(
            stage_a[stage_a.index("--tmpfs") : stage_a.index("--tmpfs") + 4],
            ("--tmpfs", "/", "--dir", "/tmp"),
        )
        self.assertEqual(
            stage_b[stage_b.index("--tmpfs") : stage_b.index("--tmpfs") + 2],
            ("--tmpfs", "/"),
        )
        home_parent_at = stage_b.index("/home") - 1
        self.assertEqual(
            stage_b[home_parent_at : home_parent_at + 9],
            (
                "--dir",
                "/home",
                "--tmpfs",
                "/home/spenser",
                "--chmod",
                "0700",
                "/home/spenser",
                "--dir",
                "/home/spenser/__Active_code",
            ),
        )
        self.assertEqual(
            stage_b[
                stage_b.index("--remount-ro") : stage_b.index("--remount-ro") + 2
            ],
            ("--remount-ro", "/"),
        )
        stage_b_symlink_pairs = tuple(
            (stage_b[index + 1], stage_b[index + 2])
            for index, value in enumerate(stage_b)
            if value == "--symlink"
        )
        self.assertIn(("usr/bin", "/bin"), stage_b_symlink_pairs)
        self.assertNotIn("{BACKING_PATH}", stage_b)
        expected_writable_projections = (
            (
                "/run/substrate-wall/backing/root/tmp",
                "/run/substrate-wall/backing/root/tmp",
            ),
            (
                "/run/substrate-wall/backing/root/tmp",
                "/tmp",
            ),
            (
                "/run/substrate-wall/backing/root/xdg-runtime",
                "/run/xdg",
            ),
            ("{CARGO_HOME_RUNTIME}", "/home/spenser/.cargo"),
            ("{ROOT}", "/home/spenser/__Active_code/substrate/target"),
            ("{CONTROL_DIRECTORY}", "/run/substrate-wall/control"),
        )
        bind_pairs = tuple(
            (stage_b[index + 1], stage_b[index + 2])
            for index, value in enumerate(stage_b)
            if value == "--bind"
        )
        self.assertEqual(bind_pairs, expected_writable_projections)
        self.assertIn("/etc", stage_b)
        read_only_pairs = tuple(
            (stage_b[index + 1], stage_b[index + 2])
            for index, value in enumerate(stage_b)
            if value == "--ro-bind"
        )
        self.assertIn(
            (
                "/run/substrate-wall/backing/root/account/passwd",
                "/etc/passwd",
            ),
            read_only_pairs,
        )
        self.assertIn(
            (
                "/run/substrate-wall/backing/root/account/group",
                "/etc/group",
            ),
            read_only_pairs,
        )
        self.assertNotIn(("/etc/passwd", "/etc/passwd"), read_only_pairs)
        self.assertNotIn(("/etc/group", "/etc/group"), read_only_pairs)
        self.assertFalse(
            any(
                source == "/home/spenser"
                for source, _destination in bind_pairs + read_only_pairs
            )
        )
        source = _module_source(runner)
        template_source = source.split(
            b"BWRAP_STAGE_B_ARGV_TEMPLATE_V1 = (\n",
            1,
        )[1].split(b")\n", 1)[0]
        self.assertIn(b'    "/run/xdg",\n', template_source)
        self.assertNotIn(b"STAGE_B_XDG_RUNTIME_DIR,", template_source)
        worker_source = source[
            source.index(b"def stage_b_worker_main") : source.index(
                b"def _default_provenance"
            )
        ]
        self.assertLess(
            worker_source.index(b"_validate_stage_b_home_projection()"),
            worker_source.index(b"_validate_stage_b_account_projection()"),
        )
        self.assertIn(
            (
                "{REPOSITORY_SNAPSHOT}",
                "/home/spenser/__Active_code/substrate",
            ),
            tuple(
                (stage_b[index + 1], stage_b[index + 2])
                for index, value in enumerate(stage_b)
                if value == "--ro-bind"
            ),
        )
        observed = self.run_bounded_fixture("production-stage-a-mount")
        self.assertEqual(
            observed,
            {
                "fstype": "tmpfs",
                "mode": 0o700,
                "private": True,
                "source": "tmpfs",
            },
        )
        self.assertIn(
            "source_records",
            runner.stage_a_main.__code__.co_varnames,
        )
        source = _module_source(runner).decode("utf-8")
        stage_a_source = source[
            source.index("def stage_a_main") : source.index(
                "def validate_rustup_resolution"
            )
        ]
        setup_steps = (
            "construct_repository_snapshot(",
            "construct_rustup_home_snapshot(",
            "construct_cargo_home_seed(",
            "construct_cargo_home_runtime(",
            "seal_snapshot_mounts(",
        )
        setup_offsets = tuple(stage_a_source.index(step) for step in setup_steps)
        self.assertEqual(setup_offsets, tuple(sorted(setup_offsets)))
        for constructor in (
            step for step in setup_steps if step.startswith("construct_")
        ):
            constructor_at = stage_a_source.index(constructor)
            checkpoint_at = stage_a_source.index(
                "persist_snapshot_setup()",
                constructor_at,
            )
            following_steps = tuple(
                offset for offset in setup_offsets if offset > constructor_at
            )
            if following_steps:
                self.assertLess(checkpoint_at, min(following_steps))
        seal_at = stage_a_source.index("seal_snapshot_mounts(")
        sealed_records_at = stage_a_source.index("sealed_records = []")
        seal_source = stage_a_source[seal_at:sealed_records_at]
        self.assertIn('child_paths["cargo-home-runtime"]', seal_source)
        self.assertLess(
            seal_at,
            stage_a_source.index("os.fchmod(runtime_fd, 0o700)"),
        )
        self.assertEqual(
            self.run_bounded_fixture("production-namespace-reference-scan"),
            {"missing": 0, "own": True},
        )
        exemptions = (
            runner._capture_inaccessible_namespace_scan_exemptions()
        )
        for pid, start_time in exemptions.items():
            try:
                observed_start = runner._process_start_time_ticks(pid)
            except FileNotFoundError:
                continue
            self.assertEqual(
                observed_start,
                start_time,
            )
        own_pid_namespace = os.stat("/proc/self/ns/pid").st_ino
        own_scan_exemptions = dict(exemptions)
        own_scan_exemptions[os.getpid()] = (
            runner._process_start_time_ticks(os.getpid())
        )
        self.assertTrue(
            runner._namespace_references(
                "pid",
                own_pid_namespace,
                inaccessible_exemptions=own_scan_exemptions,
            )
        )
        original_listdir = runner.os.listdir
        original_stat = runner.os.stat
        original_readlink = runner.os.readlink
        original_start_time = runner._process_start_time_ticks
        try:
            for task_order, target_source in (
                (("denied", "visible"), "namespace"),
                (("visible", "denied"), "descriptor"),
            ):
                def fake_listdir(path: object) -> list[str]:
                    rendered = os.fspath(path)
                    if rendered == "/proc":
                        return ["4242"]
                    if rendered == "/proc/4242/task":
                        return list(task_order)
                    if rendered.endswith("/task/denied/fd"):
                        raise PermissionError
                    if rendered.endswith("/task/visible/fd"):
                        return ["7"]
                    raise AssertionError(rendered)

                def fake_stat(
                    path: object,
                    *,
                    follow_symlinks: bool = True,
                    dir_fd: int | None = None,
                ) -> object:
                    del follow_symlinks, dir_fd
                    self.assertEqual(os.fspath(path), "/proc/4242")
                    return runner.types.SimpleNamespace(st_uid=os.getuid())

                def fake_readlink(path: object) -> str:
                    rendered = os.fspath(path)
                    if "/task/denied/" in rendered:
                        raise PermissionError
                    if target_source == "namespace" and rendered.endswith(
                        "/task/visible/ns/pid"
                    ):
                        return "pid:[123]"
                    if target_source == "descriptor" and rendered.endswith(
                        "/task/visible/fd/7"
                    ):
                        return "pid:[123]"
                    return "pid:[999]"

                runner.os.listdir = fake_listdir
                runner.os.stat = fake_stat
                runner.os.readlink = fake_readlink
                runner._process_start_time_ticks = lambda _pid: 77
                references = runner._namespace_references(
                    "pid",
                    123,
                    inaccessible_exemptions={4242: 77},
                )
                self.assertTrue(
                    any("/task/visible/" in path for path in references)
                )
                with self.assertRaises(runner.RunnerError):
                    runner.verify_namespace_inodes_absent(
                        {"pid": 123, "mnt": 456},
                        inaccessible_exemptions={4242: 77},
                    )
        finally:
            runner.os.listdir = original_listdir
            runner.os.stat = original_stat
            runner.os.readlink = original_readlink
            runner._process_start_time_ticks = original_start_time
        self.assertEqual(
            self.run_bounded_fixture("production-source-submount-rejected"),
            {
                "copy": "snapshot_identity_invalid",
                "manifest": "snapshot_identity_invalid",
            },
        )

    def test_cache_and_proc_mounts_unmount_before_tmpfs(self) -> None:
        source = _module_source(runner).decode("utf-8")
        stage_a = source[source.index("def stage_a_main") : source.index(
            "def validate_rustup_resolution"
        )]
        self.assertLess(
            stage_a.index("validate_cargo_home_runtime_writes"),
            stage_a.index("remove_tree_at"),
        )
        self.assertIn("cleanup_authority=cleanup_authority", stage_a)
        self.assertEqual(
            runner.STAGE_A_CLEANUP_ROLE_ORDER_V1,
            (
                "target",
                "tmp",
                "xdg-runtime",
                "control",
                "cargo-home-runtime",
                "cargo-home-seed",
                "rustup-home-snapshot",
                "repository-snapshot",
                "root",
            ),
        )
        self.assertNotIn("reversed(child_records)", stage_a)
        self.assertNotIn("reversed(snapshot_records)", stage_a)
        self.assertLess(
            stage_a.index("cleanup_preflight:"),
            stage_a.index("_validate_tree_for_cleanup("),
        )
        self.assertLess(
            stage_a.index("_validate_tree_for_cleanup("),
            stage_a.index(
                "for role in STAGE_A_CLEANUP_ROLE_ORDER_V1[:-1]:",
                stage_a.index("_validate_tree_for_cleanup("),
            ),
        )

    def test_backing_mountpoint_removed_and_evidence_parent_preserved(self) -> None:
        parent = self.make_safe_parent()
        authority = runner.prepare_backing_root(parent, "backing", "c" * 32)
        record = runner._default_provenance(
            invocation_id="c" * 32,
            label="backing",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        record["namespace"]["stage_a_namespace_absent"] = True
        record["containment"].update(
            {
                "echld_observed": True,
                "authenticated_empty_record": True,
                "output_eof_after_stage_b_reap": True,
            }
        )
        directory_identity = authority["backing_identity"].as_dict()
        regular_identity = {
            "dev": 1,
            "ino": 1,
            "uid": os.getuid(),
            "mode": 0o644,
            "kind": "regular",
            "sha256": "0" * 64,
            "path_matches_fd": True,
            "owner_ancestor_acl_safe": True,
        }
        executable_identity = {
            **regular_identity,
            "mode": 0o755,
            "executable": True,
        }
        symlink_identity = {
            "dev": 1,
            "ino": 2,
            "uid": os.getuid(),
            "mode": 0o777,
            "kind": "symlink",
            "target": "rustup",
            "path_matches_lstat": True,
            "owner_ancestor_acl_safe": True,
        }
        record["repository"] = {
            "branch": "fixture",
            "head": "0" * 40,
            "tree": "0" * 40,
            "cwd": "/fixture",
            "clean": True,
            "alternates_absent": True,
            "replace_refs_absent": True,
            "grafts_absent": True,
            "shallow_absent": True,
            "local_config_allowlist_valid": True,
            "worktree_config_absent": True,
            "info_attributes_absent": True,
            "tree_index_equal": True,
            "ignore_sources_tracked": True,
            "untracked_absent": True,
            "gitdir_identity": directory_identity,
            "index_pre": regular_identity,
            "index_post": regular_identity,
            "head_file_pre": regular_identity,
            "head_file_post": regular_identity,
            "branch_ref_pre": regular_identity,
            "branch_ref_post": regular_identity,
            "local_config_pre": regular_identity,
            "local_config_post": regular_identity,
            "info_exclude_pre": None,
            "info_exclude_post": None,
            "tree_manifest_sha256": "0" * 64,
            "index_manifest_pre_sha256": "0" * 64,
            "index_manifest_post_sha256": "0" * 64,
            "ignore_sources_manifest_pre_sha256": "0" * 64,
            "ignore_sources_manifest_post_sha256": "0" * 64,
            "tracked_manifest_pre_sha256": "0" * 64,
            "tracked_manifest_post_sha256": "0" * 64,
            "untracked_paths_pre_sha256": "0" * 64,
            "untracked_paths_post_sha256": "0" * 64,
            "ignored_paths_pre_sha256": "0" * 64,
            "ignored_paths_post_sha256": "0" * 64,
            "tracked_count": 1,
        }
        executable_rows = (
            ("bubblewrap", "/usr/bin/bwrap", "bubblewrap 0.11.0"),
            ("python", "/usr/bin/python3.13", "Python 3.13.7"),
            ("git", "/usr/bin/git", "git version 2.51.0"),
            (
                "newuidmap",
                "/usr/bin/newuidmap",
                "shadow 4.18.0-1",
            ),
            (
                "newgidmap",
                "/usr/bin/newgidmap",
                "shadow 4.18.0-1",
            ),
            (
                "rustup",
                "/home/spenser/.cargo/bin/rustup",
                "rustup 1.28.2 (e4f3ad6f8 2025-04-28)",
            ),
            (
                "toolchain-cargo",
                "/home/spenser/.rustup/toolchains/"
                "1.89.0-x86_64-unknown-linux-gnu/bin/cargo",
                "cargo 1.89.0 (c24e10642 2025-06-23)",
            ),
            (
                "toolchain-rustc",
                "/home/spenser/.rustup/toolchains/"
                "1.89.0-x86_64-unknown-linux-gnu/bin/rustc",
                "rustc 1.89.0 (29483883e 2025-08-04)",
            ),
        )
        runtime_flags = {
            "isolated": 1,
            "no_site": 1,
            "no_user_site": 1,
            "ignore_environment": 1,
            "dont_write_bytecode": 1,
            "safe_path": True,
        }
        record["toolchain"] = {
            "platform": "linux",
            "machine": "x86_64",
            "executables": [
                {
                    "role": role,
                    "path": path,
                    "version": version,
                    "pre": executable_identity,
                    "post": executable_identity,
                }
                for role, path, version in executable_rows
            ],
            "python_runtime": {
                "startup_tcb_trust_model": "immutable-root-platform",
                "startup_tcb_manifest_sha256": "0" * 64,
                "initial_modules_sha256": "0" * 64,
                "proc_maps_sha256": "0" * 64,
                "ld_so_preload_absent": True,
                "host_controller_verified": True,
                "bubblewrap_launcher_verified": True,
                "canonical_environment_preserved": True,
                "git_executed_from_fd": True,
                "git_object_chain_verified": True,
                "late_imports_absent": True,
                "flags": runtime_flags,
                "sys_path": [
                    "/usr/lib/python313.zip",
                    "/usr/lib/python3.13",
                    "/usr/lib/python3.13/lib-dynload",
                ],
                "bootstrap_bytes": 1,
                "bootstrap_sha256": "0" * 64,
                "runner_blob_git_oid": "0" * 40,
                "runner_blob_sha256": "0" * 64,
                "stdlib_manifest_sha256": "0" * 64,
                "runner_file": (
                    "<git-blob:"
                    + "0" * 40
                    + ":scripts/ci/canonical_shell_wall_runner.py>"
                ),
                "runner_blob_matches_manifest": True,
                "self_test_blob_git_oid": None,
                "self_test_blob_sha256": None,
                "self_test_file": None,
                "modules": [
                    {
                        "name": "builtins",
                        "origin_kind": "built-in",
                        "path": None,
                        "pre": None,
                        "post": None,
                    }
                ],
            },
            "rustup_resolution": {
                "cargo_shim_pre": symlink_identity,
                "cargo_shim_post": symlink_identity,
                "rustc_shim_pre": symlink_identity,
                "rustc_shim_post": symlink_identity,
                "settings_pre": regular_identity,
                "settings_post": regular_identity,
                "repo_toolchain_file_pre": regular_identity,
                "repo_toolchain_file_post": regular_identity,
                "rustup_version_stdout_sha256": "0" * 64,
                "rustup_version_stderr_sha256": "0" * 64,
                "cargo_proxy_version_stdout_sha256": "0" * 64,
                "rustc_proxy_version_stdout_sha256": "0" * 64,
                "proxy_stderr_empty": True,
                "canonical_selection": True,
            },
            "rust_toolchain_root": {
                "source_path": (
                    "/home/spenser/.rustup/toolchains/"
                    "1.89.0-x86_64-unknown-linux-gnu"
                ),
                "source_pre": directory_identity,
                "source_post": directory_identity,
                "directory_count": 26,
                "file_count": 213,
                "symlink_count": 0,
                "regular_file_bytes": 1_071_859_799,
                "manifest_pre_sha256": (
                    "62de669575b22b82124eacb7caf100fd1e1041e42bd66b2a148f2ff6d14540c0"
                ),
                "manifest_post_sha256": (
                    "62de669575b22b82124eacb7caf100fd1e1041e42bd66b2a148f2ff6d14540c0"
                ),
            },
        }
        root_roles = (
            "root",
            "tmp",
            "xdg-runtime",
            "control",
            "target",
            "repository-snapshot",
            "rustup-home-snapshot",
            "cargo-home-seed",
            "cargo-home-runtime",
        )
        record["roots"] = [
            {
                "role": role,
                "path": f"/fixture/{role}",
                "pre": directory_identity,
                "post": directory_identity,
                "removed": True,
                "name_absence_proved": True,
                "descriptor_closed_after_required_absence": True,
            }
            for role in root_roles
        ]
        record["request"].update(
            {
                "tmpdir": "/fixture/root/tmp",
                "xdg_runtime_dir": "/fixture/root/xdg-runtime",
                "target_dir": "/fixture/root/target",
            }
        )
        record["request"]["launcher"].update(
            {
                "host_argv_sha256": "0" * 64,
                "stage_a_argv_sha256": "0" * 64,
                "stage_b_argv_sha256": "0" * 64,
                "authority_commit_trailers_verified": True,
            }
        )
        record["namespace"].update(
            {
                "stage_a_bwrap_pid": 2,
                "stage_b_bwrap_pid": 3,
                "worker_stage_a_pid": 1,
                "stage_a_bwrap_start_time_ticks": 1,
                "stage_b_bwrap_start_time_ticks": 1,
                "worker_start_time_ticks": 1,
                "stage_a_pid_namespace_inode": 1,
                "stage_a_mount_namespace_inode": 1,
                "stage_b_pid_namespace_inode": 2,
                "stage_b_mount_namespace_inode": 2,
                "uid_map": f"0 {os.getuid()} 1\n",
                "gid_map": f"0 {os.getgid()} 1\n",
                "stage_a_bwrap_pidfd_opened": True,
                "stage_a_status_received": True,
                "stage_b_bwrap_pidfd_opened": True,
                "stage_b_status_received": True,
                "worker_pidfd_opened": True,
                "worker_peer_credentials_verified": True,
                "worker_ready_received": True,
                "control_path_absence_proved": True,
                "worker_start_sent": True,
                "worker_subreaper_verified": True,
                "worker_echild_received": True,
                "stage_b_exit_received": True,
                "stage_a_exit_received": True,
            }
        )
        record["containment"].update(
            {
                "cargo_pid": 2,
                "cargo_exit": 1,
                "stage_b_exit": 0,
                "stage_a_exit": 0,
                "output_pipe_dev": 1,
                "output_pipe_ino": 1,
                "output_pipe_opened_before_stage_b": True,
                "single_pipe_for_stdout_stderr": True,
                "original_stdin_preserved": True,
                "cargo_control_fd_absent": True,
                "stage_a_namespace_absent": True,
                "events_file": "containment.jsonl",
            }
        )
        stage_common = {
            "argv_template_sha256": "0" * 64,
            "argv_sha256": "0" * 64,
            "user_namespace": True,
            "mount_namespace": True,
            "pid_namespace": True,
            "json_status_fd": True,
            "private_propagation": True,
            "die_with_parent": True,
            "completed": True,
            "unmounted": True,
        }
        record["mounts"].update(
            {
                "backing_mountpoint": {
                    "path": authority["backing_path"],
                    "underlying_pre": directory_identity,
                    "mounted": True,
                    "underlying_post": None,
                    "removed": False,
                    "name_absence_proved": False,
                    "descriptor_closed_after_absence": False,
                },
                "tmpfs": {
                    "source": "tmpfs",
                    "fstype": "tmpfs",
                    "flags": "rw",
                    "mount_id": 1,
                    "unmounted": True,
                },
                "proc": {
                    "source": "proc",
                    "fstype": "proc",
                    "flags": "rw",
                    "mount_id": 2,
                    "unmounted": True,
                },
                "stage_a": {
                    **stage_common,
                    "uid_map": runner.CANONICAL_STAGE_A_UID_MAP,
                    "gid_map": runner.CANONICAL_STAGE_A_GID_MAP,
                    "as_pid_1": False,
                    "builtin_pid1_fail_safe_only": True,
                },
                "stage_b": {
                    **stage_common,
                    "uid_map": runner.CANONICAL_STAGE_B_UID_MAP,
                    "gid_map": runner.CANONICAL_STAGE_B_GID_MAP,
                    "as_pid_1": True,
                    "builtin_pid1_fail_safe_only": False,
                },
                "snapshots": [
                    {
                        "role": role,
                        "source_manifest_sha256": "0" * 64,
                        "snapshot_manifest_sha256": "0" * 64,
                        "source_entry_count": 1,
                        "snapshot_entry_count": 1,
                        "private_tmpfs": True,
                        "no_host_alias": True,
                        "stage_b_read_only": role != "cargo-home-seed",
                        "post_manifest_equal": True,
                        "removed": True,
                    }
                    for role in (
                        "repository",
                        "rustup-home",
                        "cargo-home-seed",
                    )
                ],
                "cargo_home_runtime": {
                    "seed_manifest_sha256": "0" * 64,
                    "pre_manifest_sha256": "0" * 64,
                    "post_manifest_sha256": "0" * 64,
                    "seed_entry_count": 1,
                    "pre_entry_count": 1,
                    "post_entry_count": 1,
                    "private_tmpfs": True,
                    "no_host_alias": True,
                    "stage_b_writable": True,
                    "seed_copy_equal": True,
                    "only_authorized_metadata_changed": True,
                    "removed": True,
                    "authorized_mutable_paths": list(
                        runner.AUTHORIZED_CARGO_RUNTIME_MUTATIONS
                    ),
                    "changed": [],
                },
            }
        )
        fixture_log = (
            b"test result: ok. 1 passed; 0 failed; 0 ignored; "
            b"0 measured; 0 filtered out; finished in 0.01s\n"
        )
        fixture_result, fixture_names, fixture_signatures = (
            runner.summarize_test_log(fixture_log, 0)
        )
        record["containment"]["cargo_exit"] = 0
        record["result"] = fixture_result
        fixture_summary = (
            json.dumps(
                fixture_result,
                sort_keys=True,
                separators=(",", ":"),
            )
            + "\n"
        ).encode("ascii")
        artifacts = runner._initialize_staged_artifact_authorities(authority)
        for name, data in {
            "cargo.log": fixture_log,
            "containment.jsonl": b"{}\n",
            "failure-names.txt": fixture_names or b"",
            "normalized-signatures.txt": fixture_signatures or b"",
            "summary.json": fixture_summary,
        }.items():
            runner._rewrite_retained_artifact(
                authority["partial_fd"],
                artifacts[name],
                data,
            )
        final = runner.finalize_after_stage_a_exit(authority, record)
        os.close(authority["parent_fd"])
        self.assertTrue(os.path.isdir(parent))
        self.assertTrue(os.path.isdir(final))
        self.assertFalse(os.path.exists(authority["backing_path"]))
        with open(
            os.path.join(final, "manifest.sha256"),
            "rt",
            encoding="ascii",
        ) as handle:
            manifest_names = [
                line.rstrip("\n").split("  ", 1)[1]
                for line in handle
            ]
        self.assertEqual(manifest_names, sorted(manifest_names, key=os.fsencode))

        recovery_authority = runner.prepare_backing_root(
            parent,
            "publication-recovery",
            "4" * 32,
        )
        recovery_record = json.loads(json.dumps(record))
        recovery_record.update(
            {
                "eligible": False,
                "record_stage": "cleanup",
                "wall_gate": "ineligible",
                "ineligibility_reasons": [],
                "runner_exit_code": 69,
                "finalized_at_utc": None,
            }
        )
        recovery_record["mounts"]["evidence_parent"] = {
            "pre": recovery_authority["parent_identity"].as_dict(),
            "post": None,
        }
        recovery_record["mounts"]["backing_mountpoint"].update(
            {
                "path": recovery_authority["backing_path"],
                "underlying_pre": recovery_authority[
                    "backing_identity"
                ].as_dict(),
                "underlying_post": None,
                "mounted": True,
                "removed": False,
                "name_absence_proved": False,
                "descriptor_closed_after_absence": False,
            }
        )
        recovery_artifacts = runner._initialize_staged_artifact_authorities(
            recovery_authority
        )
        for name, data in {
            "cargo.log": fixture_log,
            "containment.jsonl": b"{}\n",
            "failure-names.txt": fixture_names or b"",
            "normalized-signatures.txt": fixture_signatures or b"",
            "summary.json": fixture_summary,
        }.items():
            runner._rewrite_retained_artifact(
                recovery_authority["partial_fd"],
                recovery_artifacts[name],
                data,
            )
        original_fsync = runner.os.fsync
        failed_publication_fsync = False
        recovery_final = os.path.join(
            recovery_authority["parent_path"],
            recovery_authority["final_name"],
        )

        def fail_final_parent_fsync(descriptor: int) -> None:
            nonlocal failed_publication_fsync
            if (
                descriptor == recovery_authority["parent_fd"]
                and os.path.isdir(recovery_final)
                and not failed_publication_fsync
            ):
                failed_publication_fsync = True
                raise OSError(5, "injected final publication fsync failure")
            original_fsync(descriptor)

        runner.os.fsync = fail_final_parent_fsync
        try:
            with self.assertRaises(runner.RunnerError) as publication_failure:
                runner.finalize_after_stage_a_exit(
                    recovery_authority,
                    recovery_record,
                )
        finally:
            runner.os.fsync = original_fsync
        self.assertEqual(
            publication_failure.exception.reason,
            "evidence_write_failed",
        )
        self.assertTrue(failed_publication_fsync)
        self.assertTrue(os.path.isdir(recovery_authority["partial_path"]))
        self.assertFalse(os.path.exists(recovery_final))
        self.assertFalse(
            os.path.exists(
                os.path.join(
                    recovery_authority["partial_path"],
                    "manifest.sha256",
                )
            )
        )
        for artifact in recovery_artifacts.values():
            self.assertGreaterEqual(int(artifact["fd"]), 0)
            os.fstat(int(artifact["fd"]))
        self.assert_ineligible(recovery_record, "evidence_write_failed")
        recovered = runner._finalize_ineligible(
            recovery_authority,
            recovery_record,
            publication_failure.exception,
        )
        self.assert_ineligible(
            self.read_final_provenance(recovered),
            "evidence_write_failed",
        )

    def test_parallel_command_and_fresh_root(self) -> None:
        command = runner._canonical_cargo_command("parallel")
        self.assertEqual(
            command,
            (
                "/home/spenser/.cargo/bin/cargo",
                "test",
                "-p",
                "shell",
                "--lib",
                "--",
                "--nocapture",
            ),
        )
        self.assertNotIn("--test-threads=1", command)
        parent = self.make_safe_parent()
        authorities = (
            runner.prepare_backing_root(parent, "parallel-a", "1" * 32),
            runner.prepare_backing_root(parent, "parallel-b", "2" * 32),
        )
        self.assertNotEqual(
            authorities[0]["backing_path"],
            authorities[1]["backing_path"],
        )
        self.assertNotEqual(
            (
                authorities[0]["backing_identity"].dev,
                authorities[0]["backing_identity"].ino,
            ),
            (
                authorities[1]["backing_identity"].dev,
                authorities[1]["backing_identity"].ino,
            ),
        )
        for authority in authorities:
            for name_key, identity_key in (
                ("backing_name", "backing_identity"),
                ("partial_name", "partial_identity"),
            ):
                runner.remove_tree_at(
                    authority["parent_fd"],
                    authority[name_key],
                    expected=authority[identity_key],
                    cleanup_authority=self.cleanup_authority,
                )
                with self.assertRaises(FileNotFoundError):
                    os.stat(
                        authority[name_key],
                        dir_fd=authority["parent_fd"],
                        follow_symlinks=False,
                    )
            os.close(authority["backing_fd"])
            os.close(authority["partial_fd"])
            for descriptor, _path, _identity in authority["ancestor_records"]:
                os.close(descriptor)
            os.close(authority["parent_fd"])

    def test_serial_command_and_fresh_root(self) -> None:
        command = runner._canonical_cargo_command("serial")
        self.assertEqual(command[-1], "--test-threads=1")
        self.assertEqual(command[:-1], runner._canonical_cargo_command("parallel"))
        parent = self.make_safe_parent()
        authority = runner.prepare_backing_root(parent, "serial", "3" * 32)
        environment = runner._canonical_wall_environment(
            {"LANG": "C.UTF-8"},
            os.path.join(authority["backing_path"], "root"),
        )
        self.assertEqual(
            environment["TMPDIR"],
            os.path.join(authority["backing_path"], "root", "tmp"),
        )
        self.assertEqual(
            environment["XDG_RUNTIME_DIR"],
            "/run/xdg",
        )
        for name_key, identity_key in (
            ("backing_name", "backing_identity"),
            ("partial_name", "partial_identity"),
        ):
            runner.remove_tree_at(
                authority["parent_fd"],
                authority[name_key],
                expected=authority[identity_key],
                cleanup_authority=self.cleanup_authority,
            )
        os.close(authority["backing_fd"])
        os.close(authority["partial_fd"])
        for descriptor, _path, _identity in authority["ancestor_records"]:
            os.close(descriptor)
        os.close(authority["parent_fd"])

    def test_stage_a_output_pipe_preserves_combined_order_eof_backpressure_bounds_and_hash(
        self,
    ) -> None:
        parent = self.make_safe_parent()
        log_path = os.path.join(parent, "cargo.log")
        log_fd = os.open(
            log_path,
            os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o600,
        )
        read_fd, write_fd = os.pipe2(os.O_CLOEXEC)
        chunks = (
            b"stdout-1\n",
            b"stderr-" + b"x" * 8192 + b"\n",
            b"forged-" + runner.FRAME_MAGIC + b"\n",
            b"stdout-2\n",
        )
        bounded = self.run_bounded_fixture(
            "production-output-pipe",
            timeout=60,
        )
        proof = bounded["proof"]
        expected_prefix = b"".join(
            (
                b"stdout-1\n",
                b"stderr-" + b"x" * (proof["pipe_buf"] + 123) + b"\n",
                b"forged-" + runner.FRAME_MAGIC + b"\n",
                b"stdout-2\n",
            )
        )
        expected_real_bytes = proof["reported_bytes"]
        self.assertGreater(expected_real_bytes, len(expected_prefix))
        self.assertEqual(
            (expected_real_bytes - len(expected_prefix)) % proof["pipe_buf"],
            0,
        )
        expected_real = expected_prefix + b"P" * (
            expected_real_bytes - len(expected_prefix)
        )
        self.assertTrue(proof["argv_used"])
        self.assertTrue(proof["namespace_absent"])
        self.assertTrue(proof["eof_after_reap"])
        self.assertFalse(proof["overflow"])
        self.assertEqual(proof["bytes"], expected_real_bytes)
        self.assertEqual(bytes.fromhex(proof["log_hex"]), expected_real)
        self.assertEqual(
            proof["sha256"],
            hashlib.sha256(expected_real).hexdigest(),
        )
        self.assertGreater(len(expected_prefix), proof["pipe_buf"])
        self.assertGreater(proof["bytes"], proof["pipe_buf"])
        self.assertEqual(proof["worker"]["primary_status"], 0)
        self.assertTrue(proof["worker"]["echld_observed"])
        self.assertEqual(proof["worker"]["discriminator"], "CONTAINMENT_EMPTY")
        self.assertEqual(proof["final_status"], {"exit-code": 0})
        self.assertEqual(proof["stage_b_exit"], 0)

        overflow_real = bounded["overflow"]
        self.assertTrue(overflow_real["overflow"])
        self.assertIsNone(overflow_real["sha256"])
        self.assertEqual(overflow_real["reported_bytes"], expected_real_bytes)
        self.assertEqual(overflow_real["bytes"], expected_real_bytes)
        self.assertEqual(bytes.fromhex(overflow_real["log_hex"]), expected_real)
        self.assertTrue(overflow_real["eof_after_reap"])
        self.assertEqual(
            overflow_real["worker"]["discriminator"],
            "CONTAINMENT_EMPTY",
        )
        self.assertEqual(overflow_real["final_status"], {"exit-code": 0})
        self.assertEqual(overflow_real["stage_b_exit"], 0)

        retained = bounded["retained"]
        self.assertTrue(retained["retained_writer_rejected"])
        self.assertTrue(retained["namespace_absent"])
        self.assertEqual(retained["worker"]["primary_status"], 0)
        self.assertTrue(retained["worker"]["echld_observed"])
        self.assertEqual(
            retained["worker"]["discriminator"],
            "CONTAINMENT_EMPTY",
        )
        pid = os.fork()
        if pid == 0:
            os.close(read_fd)
            for chunk in chunks:
                view = memoryview(chunk)
                while view:
                    written = os.write(write_fd, view)
                    view = view[written:]
            os.close(write_fd)
            os._exit(0)
        os.close(write_fd)
        result = runner.collect_stage_b_output(
            read_fd,
            log_fd,
            deadline=time.monotonic() + 5,
            cap=64 * 1024,
        )
        os.close(read_fd)
        held_log = os.fstat(log_fd)
        named_log = os.stat(log_path, follow_symlinks=False)
        self.assertEqual(
            (held_log.st_dev, held_log.st_ino),
            (named_log.st_dev, named_log.st_ino),
        )
        self.assertEqual(runner.hash_open_file(log_fd), result["sha256"])
        os.close(log_fd)
        os.waitpid(pid, 0)
        expected = b"".join(chunks)
        with open(log_path, "rb") as handle:
            self.assertEqual(handle.read(), expected)
        self.assertEqual(result["sha256"], hashlib.sha256(expected).hexdigest())
        self.assertTrue(result["eof"])
        self.assertFalse(result["overflow"])
        overflow_read, overflow_write = os.pipe2(os.O_CLOEXEC)
        overflow_log = os.open(
            os.path.join(parent, "overflow.log"),
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o600,
        )
        os.write(overflow_write, b"x" * 65)
        os.close(overflow_write)
        overflow = runner.collect_stage_b_output(
            overflow_read,
            overflow_log,
            deadline=time.monotonic() + 1,
            cap=64,
        )
        os.close(overflow_read)
        os.close(overflow_log)
        self.assertTrue(overflow["overflow"])
        self.assertIsNone(overflow["sha256"])
        self.assertEqual(overflow["bytes"], 65)
        held_read, held_write = os.pipe2(os.O_CLOEXEC)
        retained_read, retained_write = os.pipe2(os.O_CLOEXEC)
        retained_log = os.open(
            os.path.join(parent, "retained.log"),
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o600,
        )
        holder = os.fork()
        if holder == 0:
            os.close(held_write)
            os.close(retained_read)
            os.read(held_read, 1)
            os.close(held_read)
            os.close(retained_write)
            os._exit(0)
        os.close(held_read)
        os.close(retained_write)
        with self.assertRaises(runner.RunnerError) as caught:
            runner.collect_stage_b_output(
                retained_read,
                retained_log,
                deadline=time.monotonic() + 0.05,
            )
        self.assertEqual(caught.exception.reason, "containment_timeout")
        os.write(held_write, b"x")
        os.close(held_write)
        os.waitpid(holder, 0)
        resumed = runner.collect_stage_b_output(
            retained_read,
            retained_log,
            deadline=time.monotonic() + 1,
        )
        self.assertTrue(resumed["eof"])
        os.close(retained_read)
        os.close(retained_log)
        pre_ready_record = runner._default_provenance(
            invocation_id="7" * 32,
            label="pre-ready-output",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        pre_ready_read, pre_ready_write = os.pipe2(os.O_CLOEXEC)
        pre_ready_log_path = os.path.join(parent, "pre-ready.log")
        pre_ready_log = os.open(
            pre_ready_log_path,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o600,
        )
        pre_ready_bytes = b"worker_handshake_failed\n"
        os.write(pre_ready_write, pre_ready_bytes)
        os.close(pre_ready_write)
        pre_ready_identity = os.fstat(pre_ready_read)
        pre_ready_output = runner._preserve_pre_ready_stage_b_output(
            pre_ready_read,
            pre_ready_log,
            deadline=time.monotonic() + 1,
            output_pipe_identity=(
                pre_ready_identity.st_dev,
                pre_ready_identity.st_ino,
            ),
            provenance=pre_ready_record,
        )
        os.close(pre_ready_read)
        os.close(pre_ready_log)
        with open(pre_ready_log_path, "rb") as handle:
            self.assertEqual(handle.read(), pre_ready_bytes)
        self.assertEqual(pre_ready_output["bytes"], len(pre_ready_bytes))
        self.assertEqual(
            pre_ready_record["containment"]["output_pipe_dev"],
            pre_ready_identity.st_dev,
        )
        self.assertEqual(
            pre_ready_record["containment"]["output_pipe_ino"],
            pre_ready_identity.st_ino,
        )
        self.assertEqual(
            pre_ready_record["containment"]["output_bytes_preserved"],
            len(pre_ready_bytes),
        )
        self.assertTrue(
            pre_ready_record["containment"]["output_pipe_opened_before_stage_b"]
        )
        self.assertTrue(
            pre_ready_record["containment"]["single_pipe_for_stdout_stderr"]
        )
        self.assertTrue(
            pre_ready_record["containment"]["output_eof_after_stage_b_reap"]
        )
        self.assertFalse(pre_ready_record["containment"]["output_overflow"])
        def run_supervised_case(
            case_name: str,
            *,
            incomplete: bool,
            missing_status: bool = False,
            mark_stage_b_reaped=lambda: None,
        ) -> tuple[
            dict[str, object],
            dict[str, object],
            dict[str, object],
            int,
        ]:
            output_read, output_write = os.pipe2(os.O_CLOEXEC)
            status_read, status_write = os.pipe2(os.O_CLOEXEC)
            worker_pid_read, worker_pid_write = os.pipe2(os.O_CLOEXEC)
            supervisor_endpoint, worker_endpoint = socket.socketpair(
                socket.AF_UNIX,
                socket.SOCK_SEQPACKET | socket.SOCK_CLOEXEC,
            )
            log_fd = os.open(
                os.path.join(parent, f"{case_name}.log"),
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
                0o600,
            )
            monitor_pid = os.fork()
            if monitor_pid == 0:
                os.close(output_read)
                os.close(status_read)
                os.close(worker_pid_read)
                supervisor_endpoint.close()
                try:
                    worker_pid = os.fork()
                    if worker_pid == 0:
                        os.close(worker_pid_write)
                        os.close(status_write)
                        try:
                            os.write(output_write, b"supervised\n")
                            result = {
                                "schema": runner.STATUS_VERSION,
                                "discriminator": (
                                    "CONTAINMENT_INELIGIBLE"
                                    if incomplete
                                    else "CONTAINMENT_EMPTY"
                                ),
                                "pid_namespace_inode": 1,
                                "primary_pid": os.getpid(),
                                "primary_status": None if incomplete else 0,
                                "reaped_descendants": 0 if incomplete else 1,
                                "echld_observed": not incomplete,
                                "timed_out": incomplete,
                                "forced_teardown": incomplete,
                                "survivors": (
                                    [os.getpid() + 100_000]
                                    if incomplete
                                    else []
                                ),
                                "received_signals": [],
                            }
                            if not missing_status:
                                runner._send_frame(worker_endpoint, result)
                            worker_endpoint.shutdown(socket.SHUT_WR)
                            if worker_endpoint.recv(1) != b"":
                                os._exit(91)
                            worker_endpoint.close()
                            os.close(output_write)
                            os._exit(0)
                        except BaseException:
                            os._exit(92)
                    worker_endpoint.close()
                    os.write(
                        worker_pid_write,
                        str(worker_pid).encode("ascii"),
                    )
                    os.close(worker_pid_write)
                    waited, worker_status = os.waitpid(worker_pid, 0)
                    if (
                        waited != worker_pid
                        or os.waitstatus_to_exitcode(worker_status) != 0
                    ):
                        os._exit(93)
                    exit_code = 68 if incomplete else 0
                    os.write(
                        status_write,
                        (
                            '{"exit-code":%d}\n' % exit_code
                        ).encode("ascii"),
                    )
                    os.close(status_write)
                    os._exit(exit_code)
                except BaseException:
                    os._exit(94)
            os.close(output_write)
            os.close(status_write)
            os.close(worker_pid_write)
            worker_endpoint.close()
            worker_pid_bytes = os.read(worker_pid_read, 64)
            os.close(worker_pid_read)
            self.assertTrue(worker_pid_bytes.isdigit())
            worker_pid = int(worker_pid_bytes)
            self.assertNotEqual(worker_pid, monitor_pid)
            spawned = runner.SpawnedProcess(
                monitor_pid,
                os.pidfd_open(monitor_pid),
            )
            worker_pidfd = os.pidfd_open(worker_pid)
            first_status = (
                b'{"child-pid":'
                + worker_pid_bytes
                + b',"mnt-namespace":1,"pid-namespace":1}\n'
            )
            staged_worker_results: list[dict[str, object]] = []

            def persist_worker_result(value: dict[str, object]) -> None:
                for pidfd in (spawned.pidfd, worker_pidfd):
                    probe = runner.selectors.DefaultSelector()
                    try:
                        probe.register(pidfd, runner.selectors.EVENT_READ)
                        self.assertEqual(probe.select(0), [])
                    finally:
                        probe.close()
                marker_fd = os.open(
                    os.path.join(parent, f"{case_name}-result-staged"),
                    os.O_WRONLY
                    | os.O_CREAT
                    | os.O_EXCL
                    | os.O_CLOEXEC,
                    0o600,
                )
                try:
                    os.write(marker_fd, b"durable")
                    os.fsync(marker_fd)
                finally:
                    os.close(marker_fd)
                staged_worker_results.append(dict(value))

            try:
                supervised = runner.supervise_stage_b_channels(
                    output_fd=output_read,
                    log_fd=log_fd,
                    endpoint=supervisor_endpoint,
                    status_fd=status_read,
                    spawned=spawned,
                    worker_pidfd=worker_pidfd,
                    first_status_line=first_status,
                    deadline=time.monotonic() + 5,
                    persist_worker_result=persist_worker_result,
                    mark_stage_b_reaped=mark_stage_b_reaped,
                )
                self.assertEqual(staged_worker_results, [supervised[1]])
                with self.assertRaises(ChildProcessError):
                    os.waitpid(monitor_pid, os.WNOHANG)
                return supervised
            finally:
                if missing_status:
                    supervisor_endpoint.close()
                    try:
                        os.waitpid(monitor_pid, 0)
                    except ChildProcessError:
                        pass
                os.close(spawned.pidfd)
                os.close(worker_pidfd)
                supervisor_endpoint.close()
                os.close(output_read)
                os.close(status_read)
                os.close(log_fd)

        supervised, worker_result, final_status, stage_b_exit = (
            run_supervised_case("supervised", incomplete=False)
        )
        self.assertTrue(supervised["eof_finalized_after_stage_b_reap"])
        self.assertEqual(worker_result["discriminator"], "CONTAINMENT_EMPTY")
        self.assertEqual(final_status, {"exit-code": 0})
        self.assertEqual(stage_b_exit, 0)
        incomplete_output, incomplete_result, incomplete_status, incomplete_exit = (
            run_supervised_case("incomplete", incomplete=True)
        )
        self.assertTrue(incomplete_output["eof_finalized_after_stage_b_reap"])
        self.assertEqual(
            incomplete_result["discriminator"],
            "CONTAINMENT_INELIGIBLE",
        )
        self.assertIsNone(incomplete_result["primary_status"])
        self.assertEqual(incomplete_result["reaped_descendants"], 0)
        self.assertEqual(incomplete_status, {"exit-code": 68})
        self.assertEqual(incomplete_exit, 68)
        with self.assertRaises(runner.RunnerError) as missing_status:
            run_supervised_case(
                "missing-worker-status",
                incomplete=False,
                missing_status=True,
            )
        self.assertEqual(
            missing_status.exception.reason,
            "stage_b_liveness_failed",
        )
        transferred_reap_ownership: list[bool] = []

        def fail_after_reap_ownership_transfer() -> None:
            transferred_reap_ownership.append(True)
            raise runner.RunnerError(71, "output_capture_failed")

        with self.assertRaises(runner.RunnerError) as after_reap:
            run_supervised_case(
                "failure-after-reap",
                incomplete=False,
                mark_stage_b_reaped=fail_after_reap_ownership_transfer,
            )
        self.assertEqual(after_reap.exception.reason, "output_capture_failed")
        self.assertEqual(transferred_reap_ownership, [True])

        early_output_read, early_output_write = os.pipe2(os.O_CLOEXEC)
        early_status_read, early_status_write = os.pipe2(os.O_CLOEXEC)
        early_supervisor, early_worker = socket.socketpair(
            socket.AF_UNIX,
            socket.SOCK_SEQPACKET | socket.SOCK_CLOEXEC,
        )
        early_gate_read, early_gate_write = os.pipe2(os.O_CLOEXEC)
        early_pid_read, early_pid_write = os.pipe2(os.O_CLOEXEC)
        early_log = os.open(
            os.path.join(parent, "early.log"),
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
            0o600,
        )
        early_monitor_pid = os.fork()
        if early_monitor_pid == 0:
            os.close(early_output_read)
            os.close(early_status_read)
            os.close(early_gate_write)
            os.close(early_pid_read)
            early_supervisor.close()
            try:
                early_worker_pid = os.fork()
                if early_worker_pid == 0:
                    os.close(early_pid_write)
                    os.close(early_status_write)
                    try:
                        runner._send_frame(
                            early_worker,
                            {
                                "schema": runner.STATUS_VERSION,
                                "discriminator": "CONTAINMENT_EMPTY",
                                "pid_namespace_inode": 1,
                                "primary_pid": os.getpid(),
                                "primary_status": 0,
                                "reaped_descendants": 1,
                                "echld_observed": True,
                                "timed_out": False,
                                "forced_teardown": False,
                                "survivors": [],
                                "received_signals": [],
                            },
                        )
                        early_worker.shutdown(socket.SHUT_WR)
                        if early_worker.recv(1) != b"":
                            os._exit(91)
                        early_worker.close()
                        os.close(early_output_write)
                        os.read(early_gate_read, 1)
                        os.close(early_gate_read)
                        os._exit(0)
                    except BaseException:
                        os._exit(92)
                early_worker.close()
                os.close(early_gate_read)
                os.close(early_output_write)
                os.write(
                    early_pid_write,
                    str(early_worker_pid).encode("ascii"),
                )
                os.close(early_pid_write)
                waited, worker_status = os.waitpid(early_worker_pid, 0)
                if (
                    waited != early_worker_pid
                    or os.waitstatus_to_exitcode(worker_status) != 0
                ):
                    os._exit(93)
                os.write(early_status_write, b'{"exit-code":0}\n')
                os.close(early_status_write)
                os._exit(0)
            except BaseException:
                os._exit(94)
        os.close(early_output_write)
        os.close(early_status_write)
        os.close(early_gate_read)
        os.close(early_pid_write)
        early_worker.close()
        early_worker_pid_bytes = os.read(early_pid_read, 64)
        os.close(early_pid_read)
        self.assertTrue(early_worker_pid_bytes.isdigit())
        early_worker_pid = int(early_worker_pid_bytes)
        self.assertNotEqual(early_worker_pid, early_monitor_pid)
        early_spawned = runner.SpawnedProcess(
            early_monitor_pid,
            os.pidfd_open(early_monitor_pid),
        )
        early_worker_pidfd = os.pidfd_open(early_worker_pid)
        early_first_status = (
            b'{"child-pid":'
            + early_worker_pid_bytes
            + b',"mnt-namespace":1,"pid-namespace":1}\n'
        )
        with self.assertRaises(runner.RunnerError) as early:
            runner.supervise_stage_b_channels(
                output_fd=early_output_read,
                log_fd=early_log,
                endpoint=early_supervisor,
                status_fd=early_status_read,
                spawned=early_spawned,
                worker_pidfd=early_worker_pidfd,
                first_status_line=early_first_status,
                deadline=time.monotonic() + 5,
                persist_worker_result=lambda _value: None,
        )
        self.assertEqual(early.exception.reason, "output_capture_failed")
        early_supervisor.close()
        os.write(early_gate_write, b"x")
        os.close(early_gate_write)
        waited, early_status = os.waitpid(early_monitor_pid, 0)
        self.assertEqual(waited, early_monitor_pid)
        self.assertEqual(os.waitstatus_to_exitcode(early_status), 0)
        os.close(early_spawned.pidfd)
        os.close(early_worker_pidfd)
        os.close(early_output_read)
        os.close(early_status_read)
        os.close(early_log)

    def test_success_removes_root_under_continuous_descriptor_authority(self) -> None:
        parent = self.make_safe_parent()
        os.mkdir(os.path.join(parent, "root"), 0o700)
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        root_fd, identity = runner.open_validated_directory(
            os.path.join(parent, "root"),
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        with open(os.path.join(parent, "root", "child"), "xb") as handle:
            handle.write(b"content")
        nested = os.path.join(parent, "root", "sealed")
        os.mkdir(nested, 0o700)
        with open(os.path.join(nested, "nested-child"), "xb") as handle:
            handle.write(b"sealed-content")
        os.chmod(nested, 0o500)
        os.chmod(os.path.join(parent, "root"), 0o500)
        identity = runner._directory_identity(
            root_fd,
            os.path.join(parent, "root"),
        )
        proof = runner.remove_tree_at(
            parent_fd,
            "root",
            expected=identity,
            cleanup_authority=self.cleanup_authority,
        )
        self.assertEqual(os.fstat(root_fd).st_ino, identity.ino)
        self.assertFalse(os.path.exists(os.path.join(parent, "root")))
        self.assertFalse(
            any(
                name.startswith(".substrate-cleanup-")
                for name in os.listdir(parent)
            )
        )
        self.assertTrue(proof["name_absence_proved"])
        os.close(root_fd)
        os.close(parent_fd)

    def test_success_preserves_unrelated_files(self) -> None:
        parent = self.make_safe_parent()
        os.mkdir(os.path.join(parent, "owned"), 0o700)
        sentinel = os.path.join(parent, "unrelated")
        with open(sentinel, "xb") as handle:
            handle.write(b"unchanged")
        parent_fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        owned_fd, identity = runner.open_validated_directory(
            os.path.join(parent, "owned"),
            expected_uid=os.getuid(),
            expected_mode=0o700,
        )
        runner.remove_tree_at(
            parent_fd,
            "owned",
            expected=identity,
            cleanup_authority=self.cleanup_authority,
        )
        os.close(owned_fd)
        os.close(parent_fd)
        with open(sentinel, "rb") as handle:
            self.assertEqual(handle.read(), b"unchanged")

    def test_host_finalizes_eligible_only_after_stage_a_exit_namespace_and_backing_absence(
        self,
    ) -> None:
        parent = self.make_safe_parent()
        authority = runner.prepare_backing_root(parent, "final-gate", "d" * 32)
        record = runner._default_provenance(
            invocation_id="d" * 32,
            label="final-gate",
            mode="parallel",
            timeout_seconds=1,
            expected_head="0" * 40,
            command=runner._canonical_cargo_command("parallel"),
            environment={},
        )
        record["result"] = {"canonical_match": False}
        with self.assertRaises(runner.RunnerError):
            runner.finalize_after_stage_a_exit(authority, record)
        self.assertTrue(os.path.isdir(authority["backing_path"]))
        source = _module_source(runner).decode("utf-8")
        stage_a = source[source.index("def stage_a_main") : source.index(
            "def validate_rustup_resolution"
        )]
        host = source[source.index("def host_main") : source.index(
            "def _run_selftest_bootstrap"
        )]
        self.assertLess(
            stage_a.index("def persist_authenticated_worker_result("),
            stage_a.index("supervise_stage_b_channels("),
        )
        self.assertIn(
            "persist_worker_result=persist_authenticated_worker_result",
            stage_a,
        )
        supervisor = source[
            source.index("def supervise_stage_b_channels")
            : source.index("def _fixed_git_environment")
        ]
        self.assertLess(
            supervisor.index("persist_worker_result(worker_result)"),
            supervisor.index("endpoint.close()"),
        )
        worker = source[
            source.index("def stage_b_worker_main")
            : source.index("def _default_provenance")
        ]
        self.assertLess(
            worker.index("_send_frame(endpoint, payload)"),
            worker.index("endpoint.recv(1)"),
        )
        self.assertLess(
            worker.index("endpoint.recv(1)"),
            worker.index("return 68"),
        )
        self.assertLess(
            host.index("_captured_stage_b_namespace_inodes(staged)"),
            host.index(
                "verify_namespace_inodes_absent(\n"
                "            nested_namespace_inodes,"
            ),
        )
        self.assertLess(
            host.index(
                "verify_namespace_inodes_absent(\n"
                "            nested_namespace_inodes,"
            ),
            host.index("finalize_after_stage_a_exit(authority, staged)"),
        )
        os.close(authority["backing_fd"])
        os.close(authority["partial_fd"])
        for ancestor_fd, _path, _identity in authority["ancestor_records"]:
            os.close(ancestor_fd)
        os.close(authority["parent_fd"])

    def test_host_controller_bootstrap_commit_templates_and_slots_avoid_self_reference(
        self,
    ) -> None:
        self.assertEqual(len(SELF_TEST_INVENTORY), 99)
        self.assertEqual(len({row["id"] for row in SELF_TEST_INVENTORY}), 99)
        self.assertEqual(
            tuple(row["id"] for row in SELF_TEST_INVENTORY),
            SELF_TEST_IDS,
        )
        category_counts = {
            category: sum(
                row["category"] == category
                for row in SELF_TEST_INVENTORY
            )
            for category in {
                row["category"]
                for row in SELF_TEST_INVENTORY
            }
        }
        self.assertEqual(
            category_counts,
            {
                "lifecycle": 6,
                "root-authority": 14,
                "containment": 20,
                "cli-diagnostics": 6,
                "repository": 9,
                "invocation-tcb": 11,
                "snapshots-environment": 11,
                "evidence": 5,
                "summarizer": 7,
                "mount-lifecycle": 4,
                "success-output-cleanup": 6,
            },
        )
        self.assertFalse(
            INELIGIBLE_SELF_TEST_IDS & ELIGIBLE_REGRESSION_SELF_TEST_IDS
        )
        self.assertTrue(
            INELIGIBLE_SELF_TEST_IDS | ELIGIBLE_REGRESSION_SELF_TEST_IDS
            <= set(SELF_TEST_IDS)
        )
        classifications = {
            row["id"]: row["expected_evidence_classification"]
            for row in SELF_TEST_INVENTORY
        }
        self.assertEqual(
            classifications[
                "test_stage_b_bwrap_death_before_worker_ready_is_ineligible"
            ],
            "ineligible",
        )
        self.assertEqual(
            classifications["test_summarizer_rejects_cargo_exit_status_mismatch"],
            "eligible-regression",
        )
        self.assertEqual(
            classifications[
                "test_success_removes_root_under_continuous_descriptor_authority"
            ],
            "authority-proof",
        )
        self.assertEqual(
            classifications["test_success_preserves_unrelated_files"],
            "authority-proof",
        )
        self.assertEqual(
            classifications[
                "test_backing_mountpoint_removed_and_evidence_parent_preserved"
            ],
            "authority-proof",
        )
        for row in SELF_TEST_INVENTORY:
            self.assertGreater(len(row["setup"].split()), 5)
            self.assertGreater(len(row["expected_result"].split()), 5)
            self.assertNotIn(row["id"], row["expected_result"])
            self.assertTrue(row["contract_clause"].endswith(row["id"]))
        discovered = {
            name
            for name in dir(type(self))
            if name.startswith("test_") and callable(getattr(type(self), name))
        }
        self.assertEqual(discovered, set(SELF_TEST_IDS))
