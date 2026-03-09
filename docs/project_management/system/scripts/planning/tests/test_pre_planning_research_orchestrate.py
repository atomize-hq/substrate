import json
import os
import shutil
import subprocess
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TestPrePlanningResearchOrchestrate(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pre_planning_research_orchestrate"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.script = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "pre_planning_research_orchestrate.sh"
        )
        if not cls.script.is_file():
            raise unittest.SkipTest("pre_planning_research_orchestrate.sh not found at expected canonical path.")

    def _make_feature_dir(self, name: str) -> Path:
        feature_dir = self.tmp_root / name
        if feature_dir.exists():
            shutil.rmtree(feature_dir)
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_text(
            feature_dir / "tasks.json",
            json.dumps(
                {
                    "meta": {
                        "schema_version": 4,
                        "slice_spec_version": 2,
                        "cross_platform": True,
                        "automation": {"enabled": True, "orchestration_branch": "feat/test"},
                        "adr_paths": [str(feature_dir / "ADR-0001-fixture.md")],
                    },
                    "tasks": [],
                },
                indent=2,
            ),
        )
        _write_text(feature_dir / "ADR-0001-fixture.md", "# ADR fixture\n")
        return feature_dir

    def _write_fake_runner(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail

feature_dir=""
agent=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --feature-dir)
      feature_dir="$2"
      shift 2
      ;;
    --agent)
      agent="$2"
      shift 2
      ;;
    --codex-profile|--codex-model)
      shift 2
      ;;
    --codex-jsonl)
      shift
      ;;
    *)
      shift
      ;;
  esac
done

write_doc() {
  local out="$1"
  shift
  mkdir -p "$(dirname "${out}")"
  cat > "${out}"
}

case "$agent" in
  spec_manifest) step="spec-manifest"; rel="pre-planning/spec_manifest.md" ;;
  impact_map)
    step="impact-map"
    rel="pre-planning/impact_map.md"
    sleep_s="3"
    ;;
  min_spec_draft) step="min-spec-draft"; rel="pre-planning/minimal_spec_draft.md" ;;
  ci_checkpoint) step="CI-checkpoint"; rel="pre-planning/ci_checkpoint_plan.md" ;;
  workstream_triage)
    step="workstream-triage"
    rel="pre-planning/workstream_triage.md"
    ;;
  *)
    echo "unknown agent: $agent" >&2
    exit 2
    ;;
esac

step_dir="${feature_dir}/logs/${step}"
run_dir="${step_dir}/runs/fixture-${agent}"
mkdir -p "${run_dir}"
printf 'runner summary for %s\\n' "$agent" > "${run_dir}/last_message.run.md"
printf 'handoff for %s\\n' "$agent" > "${step_dir}/handoff.md"
printf 'start %s\\n' "$agent" >> "${TRACE_PATH}"
staged="${step_dir}/staged/${rel}"
case "$agent" in
  spec_manifest)
    write_doc "${staged}" <<'EOF'
# spec manifest
EOF
    ;;
  impact_map)
    write_doc "${staged}" <<'EOF'
# impact map

## Touch set (explicit)

### Create
- `docs/project_management/packs/draft/fake-feature/pre-planning/ci_checkpoint_plan.md`

### Edit
- None

### Deprecate
- None

### Delete
- None
EOF
    ;;
  min_spec_draft)
    write_doc "${staged}" <<'EOF'
# minimal spec
EOF
    ;;
  ci_checkpoint)
    write_doc "${staged}" <<'EOF'
# checkpoint plan
EOF
    ;;
  workstream_triage)
    if [[ "${FAIL_WORKSTREAM:-0}" == "1" ]]; then
      write_doc "${staged}" <<'EOF'
bad triage
EOF
    else
      write_doc "${staged}" <<'EOF'
# Workstream triage fixture

### NASPP-PWS-contract — contract

- Goal: fixture

### NASPP-PWS-tasks_checkpoints — tasks_checkpoints

- Goal: fixture

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "NASPP",
  "accepted_slice_order": ["NASPP0"],
  "pws": [
    {
      "id": "NASPP-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [],
      "owns": ["contract.md"]
    },
    {
      "id": "NASPP-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": ["NASPP-PWS-contract"],
      "assumes": [],
      "owns": ["tasks.json", "session_log.md", "kickoff_prompts/", "slices/NASPP0/kickoff_prompts/"]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->
EOF
    fi
    ;;
esac
if [[ "$agent" == "ci_checkpoint" && "${WRITE_TASKS_JSON:-0}" == "1" ]]; then
  mkdir -p "${step_dir}/staged"
  cat > "${step_dir}/staged/tasks.json" <<'EOF'
{
  "meta": {
    "schema_version": 4,
    "slice_spec_version": 2,
    "cross_platform": true,
    "automation": {"enabled": true, "orchestration_branch": "feat/test"},
    "checkpoint_boundaries": ["NASPP0"]
  },
  "tasks": []
}
EOF
fi
if [[ "${sleep_s:-0}" != "0" ]]; then
  sleep "${sleep_s}"
fi
printf 'end %s\\n' "$agent" >> "${TRACE_PATH}"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_fake_git(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "status" && "${2:-}" == "--porcelain=v1" ]]; then
  exit 0
fi
if [[ "${1:-}" == "diff" && "${2:-}" == "--name-only" ]]; then
  exit 0
fi
if [[ "${1:-}" == "diff" && "${2:-}" == "--cached" && "${3:-}" == "--quiet" ]]; then
  exit 0
fi
if [[ "${1:-}" == "add" ]]; then
  exit 0
fi
if [[ "${1:-}" == "commit" ]]; then
  exit 0
fi
exec "${REAL_GIT}" "$@"
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_fake_alignment_reporter(self, path: Path) -> None:
        script = """#!/usr/bin/env python3
print("# Alignment report\\n\\n- synthetic\\n")
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _write_fake_codex(self, path: Path) -> None:
        script = """#!/usr/bin/env bash
set -euo pipefail
exit 0
"""
        _write_text(path, script)
        os.chmod(path, 0o755)

    def _run(self, feature_dir: Path, *, fail_workstream: bool = False) -> tuple[subprocess.CompletedProcess[str], Path]:
        tools_dir = feature_dir / "tools"
        runner = tools_dir / "fake_runner.sh"
        fake_git = tools_dir / "git"
        fake_alignment = tools_dir / "fake_alignment_reporter.py"
        fake_codex = tools_dir / "codex"
        trace_path = feature_dir / "runner_trace.log"
        self._write_fake_runner(runner)
        self._write_fake_git(fake_git)
        self._write_fake_alignment_reporter(fake_alignment)
        self._write_fake_codex(fake_codex)

        env = os.environ.copy()
        env["PM_PRE_PLANNING_RUNNER"] = str(runner)
        env["PM_PRE_PLANNING_ALIGNMENT_REPORTER"] = str(fake_alignment)
        env["REAL_GIT"] = subprocess.check_output(["which", "git"], text=True, cwd=str(self.repo_root)).strip()
        env["PATH"] = str(tools_dir) + os.pathsep + env["PATH"]
        env["TRACE_PATH"] = str(trace_path)
        if fail_workstream:
            env["FAIL_WORKSTREAM"] = "1"

        res = subprocess.run(
            ["bash", str(self.script), "--feature-dir", str(feature_dir), "--poll-s", "1"],
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
            env=env,
        )
        return res, trace_path

    def test_overlap_wrapper_promotes_staged_outputs_and_publishes_stable_last_messages(self) -> None:
        feature_dir = self._make_feature_dir("success_overlap")
        res, trace_path = self._run(feature_dir)

        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertTrue((feature_dir / "pre-planning" / "spec_manifest.md").is_file())
        self.assertTrue((feature_dir / "pre-planning" / "impact_map.md").is_file())
        self.assertTrue((feature_dir / "pre-planning" / "minimal_spec_draft.md").is_file())
        self.assertTrue((feature_dir / "pre-planning" / "ci_checkpoint_plan.md").is_file())
        self.assertTrue((feature_dir / "pre-planning" / "workstream_triage.md").is_file())
        for step in ["spec-manifest", "impact-map", "min-spec-draft", "CI-checkpoint", "workstream-triage"]:
            self.assertTrue((feature_dir / "logs" / step / "last_message.md").is_file(), msg=step)

        trace = trace_path.read_text(encoding="utf-8")
        self.assertIn("start impact_map", trace)
        self.assertIn("start workstream_triage", trace)
        self.assertLess(trace.index("start workstream_triage"), trace.index("end impact_map"))

    def test_overlap_wrapper_restores_failed_workstream_promotion(self) -> None:
        feature_dir = self._make_feature_dir("failed_overlap")
        res, _ = self._run(feature_dir, fail_workstream=True)

        self.assertNotEqual(res.returncode, 0)
        self.assertFalse((feature_dir / "pre-planning" / "workstream_triage.md").exists())
        self.assertTrue((feature_dir / "logs" / "workstream-triage" / "staged" / "pre-planning" / "workstream_triage.md").is_file())
        self.assertFalse((feature_dir / "logs" / "workstream-triage" / "last_message.md").exists())


if __name__ == "__main__":
    unittest.main()
