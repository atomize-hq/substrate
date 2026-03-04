import json
import subprocess
import sys
import unittest
from pathlib import Path


BEGIN = "<!-- PM_PWS_INDEX:BEGIN -->"
END = "<!-- PM_PWS_INDEX:END -->"


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _triage_text(*, slice_prefix: str, pws: list[dict]) -> str:
    headings = []
    for p in pws:
        headings.append(f"### {p['id']} — {p['role']}\n\n- Goal: fixture\n")

    idx = {"pws_index_version": 1, "slice_prefix": slice_prefix, "pws": pws}
    body = json.dumps(idx, indent=2, sort_keys=False)
    return (
        "# Workstream triage fixture\n\n"
        "## Planning workstreams (PWS)\n\n"
        + "\n".join(headings)
        + "\n"
        + f"{BEGIN}\n"
        + "```json\n"
        + body
        + "\n```\n"
        + f"{END}\n"
    )


def _parse_topo_ids(stdout: str) -> list[str]:
    lines = stdout.splitlines()
    try:
        start = lines.index("Topological order:") + 1
    except ValueError:
        raise AssertionError("missing 'Topological order:' section") from None

    topo: list[str] = []
    for line in lines[start:]:
        if not line.strip():
            break
        # Example: "1) WDRA-PWS-contract (role=..., depends_on=..., owns=...)"
        if ") " not in line:
            raise AssertionError(f"unexpected topo line: {line!r}")
        _, rest = line.split(") ", 1)
        pid = rest.split(" ", 1)[0].strip()
        topo.append(pid)
    return topo


def _parse_layers(stdout: str) -> dict[int, list[str]]:
    lines = stdout.splitlines()
    try:
        start = lines.index("Parallel layers:") + 1
    except ValueError:
        raise AssertionError("missing 'Parallel layers:' section") from None

    layers: dict[int, list[str]] = {}
    current: int | None = None
    for line in lines[start:]:
        if line == "Notes:":
            break
        if not line.strip():
            continue
        if line.startswith("Layer ") and line.endswith(":"):
            raw = line[len("Layer ") : -1]
            current = int(raw.strip())
            layers[current] = []
            continue
        if line.startswith("- "):
            if current is None:
                raise AssertionError(f"bullet before any layer header: {line!r}")
            pid = line[len("- ") :].split(" ", 1)[0].strip()
            layers[current].append(pid)
            continue
    return layers


class TestPmPwsPlan(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; pm_pws_plan tests expect repo-root cwd.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_pws_plan"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.planner = (
            cls.repo_root
            / "docs"
            / "project_management"
            / "system"
            / "scripts"
            / "planning"
            / "pm_pws_plan.py"
        )
        if not cls.planner.is_file():
            raise unittest.SkipTest("pm_pws_plan.py not found at expected canonical path.")

    def _run(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.planner), *args]
        return subprocess.run(cmd, text=True, capture_output=True, check=False, cwd=str(self.repo_root))

    def _make_feature_dir(self, name: str, triage_text: str) -> Path:
        feature_dir = self.tmp_root / name
        feature_dir.mkdir(parents=True, exist_ok=True)
        _write_text(feature_dir / "pre-planning" / "workstream_triage.md", triage_text)
        return feature_dir

    def test_pass_simple_dag_and_disjoint_owns(self) -> None:
        prefix = "WDRA"
        contract = f"{prefix}-PWS-contract"
        seams = f"{prefix}-PWS-implementation_seams"
        schema = f"{prefix}-PWS-schema_inventory"
        slice_spec = f"{prefix}-PWS-slice_spec_wdra0"
        tasks = f"{prefix}-PWS-tasks_checkpoints"

        pws = [
            {
                "id": contract,
                "role": "contract",
                "depends_on": [],
                "assumes": ["fixture"],
                "owns": ["contract.md"],
            },
            {
                "id": seams,
                "role": "implementation_seams",
                "depends_on": [],
                "assumes": ["fixture"],
                "owns": ["pre-planning/impact_map.md"],
            },
            {
                "id": schema,
                "role": "schema_inventory",
                "depends_on": [contract],
                "assumes": ["fixture"],
                "owns": ["telemetry-spec.md"],
            },
            {
                "id": slice_spec,
                "role": "slice_spec",
                "depends_on": [contract],
                "assumes": ["fixture"],
                "owns": ["slices/WDRA0/WDRA0-spec.md"],
            },
            {
                "id": tasks,
                "role": "tasks_checkpoints",
                "depends_on": [schema, slice_spec],
                "assumes": ["fixture"],
                "owns": [
                    "tasks.json",
                    "plan.md",
                    "session_log.md",
                    "kickoff_prompts/",
                    "slices/WDRA0/kickoff_prompts/",
                ],
            },
        ]

        feature_dir = self._make_feature_dir("pass_simple_dag", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "")

        topo = _parse_topo_ids(res.stdout)
        self.assertEqual(topo, [contract, seams, schema, slice_spec, tasks])

        layers = _parse_layers(res.stdout)
        self.assertEqual(sorted(layers.get(0, [])), [contract, seams])
        self.assertEqual(sorted(layers.get(1, [])), [schema, slice_spec])
        self.assertEqual(sorted(layers.get(2, [])), [tasks])

    def test_pass_directory_prefix_overlap_sequentializes_layer(self) -> None:
        prefix = "WDRA"
        contract = f"{prefix}-PWS-contract"
        slices_prefix = f"{prefix}-PWS-slices_prefix"
        slice_spec = f"{prefix}-PWS-slice_spec_wdra0"
        tasks = f"{prefix}-PWS-tasks_checkpoints"

        pws = [
            {
                "id": contract,
                "role": "contract",
                "depends_on": [],
                "assumes": ["fixture"],
                "owns": ["contract.md"],
            },
            {
                "id": slices_prefix,
                "role": "implementation",
                "depends_on": [contract],
                "assumes": ["fixture"],
                "owns": ["slices/"],
            },
            {
                "id": slice_spec,
                "role": "slice_spec",
                "depends_on": [contract],
                "assumes": ["fixture"],
                "owns": ["slices/WDRA0/WDRA0-spec.md"],
            },
            {
                "id": tasks,
                "role": "tasks_checkpoints",
                "depends_on": [slice_spec],
                "assumes": ["fixture"],
                "owns": [
                    "tasks.json",
                    "plan.md",
                    "session_log.md",
                    "kickoff_prompts/",
                    "slices/WDRA0/kickoff_prompts/",
                ],
            },
        ]

        feature_dir = self._make_feature_dir("pass_prefix_overlap", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 0, msg=res.stderr)

        layers = _parse_layers(res.stdout)
        layer_of_slices_prefix = next((k for k, v in layers.items() if slices_prefix in v), None)
        layer_of_slice_spec = next((k for k, v in layers.items() if slice_spec in v), None)
        self.assertIsNotNone(layer_of_slices_prefix)
        self.assertIsNotNone(layer_of_slice_spec)
        self.assertNotEqual(layer_of_slices_prefix, layer_of_slice_spec)

        self.assertIn("WARN: owns conflict prevented full parallelism", res.stdout)
        self.assertIn("path=slices/", res.stdout)

    def test_fail_invalid_index_exits_nonzero(self) -> None:
        prefix = "WDRA"
        pws = [
            {
                "id": f"{prefix}-PWS-contract",
                "role": "contract",
                "depends_on": [],
                "assumes": ["fixture"],
                "owns": ["contract.md"],
            }
        ]
        feature_dir = self._make_feature_dir("fail_invalid_index", _triage_text(slice_prefix=prefix, pws=pws))
        res = self._run(["--feature-dir", str(feature_dir)])
        self.assertEqual(res.returncode, 1)
        self.assertIn("missing required PWS id", res.stderr)


if __name__ == "__main__":
    unittest.main()
