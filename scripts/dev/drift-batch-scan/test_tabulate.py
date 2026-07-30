import importlib.util
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

MODULE_PATH = pathlib.Path(__file__).with_name('tabulate.py')
spec = importlib.util.spec_from_file_location('drift_batch_tabulate', MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

RUN_BATCH_PATH = pathlib.Path(__file__).with_name('run_batch.py')
run_batch_spec = importlib.util.spec_from_file_location('drift_batch_run_batch', RUN_BATCH_PATH)
run_batch = importlib.util.module_from_spec(run_batch_spec)
assert run_batch_spec.loader is not None
run_batch_spec.loader.exec_module(run_batch)


class TabulateStrataInferenceTests(unittest.TestCase):
    def test_infer_language_repo_type_detects_rust(self):
        cp = {
            'task_frame': {
                'working_set_paths': ['crates/agent-drift-analyzer/src/lib.rs'],
                'verification_commands': ['cargo test -p agent-drift-analyzer'],
                'command_families': ['cargo'],
                'tools': [],
            },
            'structured_objective': {
                'target': {
                    'paths': ['Cargo.toml'],
                    'named_artifacts': [],
                    'workspace_refs': [],
                    'symbols': [],
                    'display': 'agent-drift-analyzer',
                },
                'primary_intent': 'implement',
            },
            'session_archetype': {'label': 'autonomous_implementation'},
            'session_progress': {'dimension': 'implementation_verification_wall'},
        }
        self.assertEqual(module.infer_language_repo_type(cp), 'rust')

    def test_infer_language_repo_type_detects_docs_only(self):
        cp = {
            'task_frame': {
                'working_set_paths': ['docs/specs/r6/MAP.md'],
                'verification_commands': [],
                'command_families': [],
                'tools': [],
            },
            'structured_objective': {
                'target': {
                    'paths': ['docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md'],
                    'named_artifacts': [],
                    'workspace_refs': [],
                    'symbols': [],
                    'display': 'findings update',
                },
                'primary_intent': 'docs',
            },
        }
        self.assertEqual(module.infer_language_repo_type(cp), 'docs_only')
        self.assertEqual(module.infer_tooling_type(cp), 'generic_filesystem_doc')

    def test_infer_language_repo_type_detects_mixed(self):
        cp = {
            'task_frame': {
                'working_set_paths': ['src/main.rs', 'tests/test_semantic_goal_drift.py'],
                'verification_commands': [],
                'command_families': [],
                'tools': [],
            },
            'structured_objective': {
                'target': {
                    'paths': [],
                    'named_artifacts': [],
                    'workspace_refs': [],
                    'symbols': [],
                    'display': 'mixed repo touch',
                },
                'primary_intent': 'implement',
            },
        }
        self.assertEqual(module.infer_language_repo_type(cp), 'mixed')

    def test_infer_workflow_type_uses_planning_and_verification_signals(self):
        planning_cp = {
            'task_frame': {'working_set_paths': [], 'verification_commands': [], 'command_families': [], 'tools': []},
            'structured_objective': {'primary_intent': 'plan', 'target': None},
            'session_archetype': {'label': 'planning'},
            'session_progress': {'dimension': 'planning_convergence'},
        }
        verification_cp = {
            'task_frame': {'working_set_paths': [], 'verification_commands': ['cargo test'], 'command_families': ['cargo'], 'tools': []},
            'structured_objective': {'primary_intent': 'validate', 'target': None},
            'session_archetype': {'label': 'verification_closeout'},
            'session_progress': {'dimension': 'verification_closeout_narrowing'},
        }
        self.assertEqual(module.infer_workflow_type(planning_cp), 'docs_planning')
        self.assertEqual(module.infer_workflow_type(verification_cp), 'verification')

    def test_infer_workflow_type_detects_review_fix(self):
        cp = {
            'task_frame': {'working_set_paths': ['crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs'], 'verification_commands': [], 'command_families': [], 'tools': []},
            'structured_objective': {'primary_intent': 'debug', 'target': None},
            'session_archetype': {'label': 'troubleshooting'},
            'session_progress': {'dimension': 'troubleshooting_frontier'},
        }
        self.assertEqual(module.infer_workflow_type(cp), 'review_fix')

    def test_infer_tooling_type_detects_python_pytest(self):
        cp = {
            'task_frame': {
                'working_set_paths': ['tests/test_semantic_goal_drift.py'],
                'verification_commands': ['python -m pytest tests/test_semantic_goal_drift.py'],
                'command_families': ['python', 'pytest'],
                'tools': ['python'],
            },
            'structured_objective': {
                'target': {
                    'paths': ['tests/test_semantic_goal_drift.py'],
                    'named_artifacts': [],
                    'workspace_refs': [],
                    'symbols': [],
                    'display': 'pytest target',
                },
                'primary_intent': 'validate',
            },
        }
        self.assertEqual(module.infer_tooling_type(cp), 'python_pytest')


class PrivacySafeReceiptTests(unittest.TestCase):
    @staticmethod
    def selection_receipt():
        return {
            'schema_version': 'p7-private-selection-v1',
            'inventory': {
                'route': 'CurrentNativeV2',
                'as_of': '2026-07-31T23:59:59Z',
                'digest': 'a' * 64,
                'candidate_count': 4,
            },
            'quota_config': {
                'schema_version': 1,
                'seed': 42,
                'required_families': ['language_repo'],
                'buckets': [{
                    'family': 'language_repo',
                    'label': 'rust',
                    'quota': 3,
                    'mandatory_population': 3,
                    'risk_class': 'ordinary',
                }],
                'digest': 'b' * 64,
            },
            'selection': {'digest': 'c' * 64, 'selected_count': 3},
            'coverage': [{
                'family': 'language_repo',
                'label': 'rust',
                'quota': 3,
                'mandatory_population': 3,
                'eligible': 4,
                'selected': 3,
                'underfill': 0,
                'status': 'filled',
            }],
            'privacy': {'raw_private_fields_included': False},
        }

    def test_batch_receipt_counts_outcomes_without_private_status_rows(self):
        status = [
            {
                'session_id': 'private-session-a',
                'repo': 'private-repo',
                'ok': True,
                'error': None,
            },
            {
                'session_id': 'private-session-b',
                'repo': 'private-repo',
                'ok': False,
                'error': 'compact:/private/path leaked',
            },
        ]

        checkpoint_set = {
            'schema_version': 1,
            'file_count': 1,
            'checkpoint_count': 2,
            'digest': 'd' * 64,
        }
        receipt = run_batch.build_batch_receipt(
            self.selection_receipt(), status, checkpoint_set
        )
        serialized = json.dumps(receipt, sort_keys=True)

        self.assertEqual(receipt['batch'], {
            'attempted_count': 2,
            'succeeded_count': 1,
            'failed_count': 1,
            'failure_kinds': {'compact': 1},
        })
        self.assertEqual(receipt['checkpoint_set'], checkpoint_set)
        self.assertNotIn('private-session', serialized)
        self.assertNotIn('private-repo', serialized)
        self.assertNotIn('/private/path', serialized)
        run_batch.validate_safe_receipt(receipt)

    def test_tabulation_receipt_contains_only_aggregate_observable_fields(self):
        batch_receipt = run_batch.build_batch_receipt(
            self.selection_receipt(),
            [{'ok': True, 'error': None}],
            {
                'schema_version': 1,
                'file_count': 1,
                'checkpoint_count': 3,
                'digest': 'd' * 64,
            },
        )
        receipt = module.build_sanitized_receipt(
            batch_receipt,
            analysis_coverage={
                'sessions_analyzed': 1,
                'total_checkpoints': 3,
                'semantic_goal_drift_fires': 0,
            },
            heuristic_strata={
                'language_repo': {
                    'rust': {'checkpoints': 3, 'semantic_goal_drift_fires': 0}
                }
            },
        )
        serialized = json.dumps(receipt, sort_keys=True)

        self.assertEqual(receipt['selection']['inventory']['digest'], 'a' * 64)
        self.assertEqual(receipt['analysis_coverage']['sessions_analyzed'], 1)
        self.assertIn('scorer_internal_claims', receipt['limitations'])
        for forbidden in (
            '"session_id"',
            '"repo"',
            '"path"',
            '"message"',
            '/Users/',
            '/home/',
        ):
            self.assertNotIn(forbidden, serialized)
        module.validate_sanitized_receipt(receipt)

    def test_same_sized_selected_manifest_must_match_receipt_digest(self):
        records = [
            {
                'timestamp': '2026-07-20T12:00:00Z',
                'relative_path': '2026/07/20/rollout-a.jsonl',
                'session_id': 'session-a',
                'path': '/scratch/rollout-a.jsonl',
                'cwd': '/workspace/a',
                'repo': 'repo-a',
                'month': '2026-07',
                'size': 2048,
                'source_digest': 'a' * 64,
                'rollout_format': 'CurrentNativeV2',
                'strata': {
                    'language_repo': ['rust'],
                    'workflow': ['verification'],
                },
            }
        ]
        receipt = self.selection_receipt()
        receipt['selection']['selected_count'] = 1
        receipt['selection']['digest'] = run_batch.selected_manifest_digest(
            records, receipt['quota_config']['digest']
        )

        run_batch.validate_selected_manifest(receipt, records)

        changed = [dict(records[0], source_digest='e' * 64)]
        with self.assertRaisesRegex(ValueError, 'selected-set digest'):
            run_batch.validate_selected_manifest(receipt, changed)

    def test_batch_directory_must_be_fresh(self):
        with tempfile.TemporaryDirectory() as root:
            batch_dir = pathlib.Path(root) / 'batch'
            checkpoints, work = run_batch.prepare_batch_directories(batch_dir)
            self.assertTrue(checkpoints.is_dir())
            self.assertTrue(work.is_dir())
            with self.assertRaisesRegex(ValueError, 'must not already exist'):
                run_batch.prepare_batch_directories(batch_dir)

    def test_failed_batch_cannot_produce_aggregate_receipt(self):
        batch_receipt = run_batch.build_batch_receipt(
            self.selection_receipt(),
            [{'ok': False, 'error': 'compact:failed'}],
            {
                'schema_version': 1,
                'file_count': 0,
                'checkpoint_count': 0,
                'digest': 'd' * 64,
            },
        )
        with self.assertRaisesRegex(ValueError, 'failed sessions'):
            module.build_sanitized_receipt(
                batch_receipt,
                analysis_coverage={'sessions_analyzed': 0},
                heuristic_strata={},
            )

    def test_batch_runner_records_failure_and_exits_nonzero(self):
        with tempfile.TemporaryDirectory() as root:
            root_path = pathlib.Path(root)
            repo = root_path / 'repo'
            binary_dir = repo / 'target' / 'debug'
            binary_dir.mkdir(parents=True)
            for name in ('agent-session-compactor', 'agent-drift-analyzer'):
                binary = binary_dir / name
                binary.write_text('#!/bin/sh\nexit 1\n')
                binary.chmod(0o755)

            source = root_path / 'rollout.jsonl'
            source.write_text('{}\n')
            records = [
                {
                    'timestamp': '2026-07-20T12:00:00Z',
                    'relative_path': '2026/07/20/rollout-a.jsonl',
                    'session_id': 'session-a',
                    'path': str(source),
                    'cwd': '/workspace/a',
                    'repo': 'repo-a',
                    'month': '2026-07',
                    'size': source.stat().st_size,
                    'source_digest': 'a' * 64,
                    'rollout_format': 'CurrentNativeV2',
                    'strata': {},
                }
            ]
            receipt = self.selection_receipt()
            receipt['selection']['selected_count'] = 1
            receipt['selection']['digest'] = run_batch.selected_manifest_digest(
                records, receipt['quota_config']['digest']
            )
            selected = root_path / 'selected.jsonl'
            selected.write_text(json.dumps(records[0]) + '\n')
            receipt_path = root_path / 'selection_receipt.json'
            receipt_path.write_text(json.dumps(receipt))
            batch_dir = root_path / 'batch'

            result = subprocess.run(
                [
                    sys.executable,
                    str(pathlib.Path(__file__).with_name('run_batch.py')),
                    '--repo',
                    str(repo),
                    '--selected',
                    str(selected),
                    '--selection-receipt',
                    str(receipt_path),
                    '--batch-dir',
                    str(batch_dir),
                ],
                capture_output=True,
                text=True,
                check=False,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn('aggregate receipt is not eligible', result.stderr)
            batch_receipt = json.loads(
                (batch_dir / 'batch_receipt.json').read_text()
            )
            self.assertEqual(batch_receipt['batch']['failed_count'], 1)

    def test_tabulation_rejects_stale_checkpoint_files(self):
        with tempfile.TemporaryDirectory() as root:
            checkpoint_dir = pathlib.Path(root) / 'checkpoints'
            checkpoint_dir.mkdir()
            (checkpoint_dir / 'current.jsonl').write_text(
                json.dumps({'_session_file_id': 'current', 'ordinal': 1}) + '\n'
            )
            checkpoint_set = run_batch.checkpoint_set_summary(checkpoint_dir)
            batch_receipt = run_batch.build_batch_receipt(
                self.selection_receipt(),
                [{'ok': True, 'error': None}],
                checkpoint_set,
            )
            (checkpoint_dir / 'stale.jsonl').write_text(
                json.dumps({'_session_file_id': 'stale', 'ordinal': 1}) + '\n'
            )

            with self.assertRaisesRegex(ValueError, 'checkpoint set'):
                module.validate_batch_inputs(batch_receipt, checkpoint_dir)

    def test_receipt_validation_rejects_private_keys_and_paths(self):
        with self.assertRaisesRegex(ValueError, 'forbidden private key'):
            run_batch.validate_safe_receipt({'session_id': 'private-session'})
        with self.assertRaisesRegex(ValueError, 'absolute private path'):
            module.validate_sanitized_receipt({'detail': '/Users/private/session.jsonl'})


if __name__ == '__main__':
    unittest.main()
