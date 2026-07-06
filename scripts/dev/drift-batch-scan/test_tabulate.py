import importlib.util
import pathlib
import unittest

MODULE_PATH = pathlib.Path(__file__).with_name('tabulate.py')
spec = importlib.util.spec_from_file_location('drift_batch_tabulate', MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)


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


if __name__ == '__main__':
    unittest.main()
