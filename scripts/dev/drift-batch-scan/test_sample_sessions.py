import dataclasses
import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).with_name("sample_sessions.py")
spec = importlib.util.spec_from_file_location("drift_batch_sample_sessions", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)


class FrozenInventoryTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temp_dir.name)

    def tearDown(self):
        self.temp_dir.cleanup()

    def write_rollout(
        self,
        relative_path,
        *,
        session_id,
        timestamp="2026-07-20T12:00:00Z",
        version="v2",
        second_version=None,
    ):
        path = self.root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        payload = {"id": session_id, "cwd": f"/workspace/{session_id}"}
        if version is not None:
            payload["multi_agent_version"] = version
        rows = [{"timestamp": timestamp, "type": "session_meta", "payload": payload}]
        if second_version is not None:
            rows.append(
                {
                    "timestamp": timestamp,
                    "type": "session_meta",
                    "payload": {
                        "id": session_id,
                        "cwd": f"/workspace/{session_id}",
                        "multi_agent_version": second_version,
                    },
                }
            )
        rows.append(
            {
                "timestamp": timestamp,
                "type": "event_msg",
                "payload": {"type": "task_started", "turn_id": f"turn-{session_id}"},
            }
        )
        path.write_text("".join(json.dumps(row) + "\n" for row in rows))
        return path

    def test_exact_current_native_route_and_as_of_cutoff(self):
        current = self.write_rollout(
            "2026/07/20/rollout-current.jsonl", session_id="session-current"
        )
        legacy = self.write_rollout(
            "2026/07/20/rollout-legacy.jsonl",
            session_id="session-legacy",
            version="v1",
        )
        missing = self.write_rollout(
            "2026/07/20/rollout-missing.jsonl",
            session_id="session-missing",
            version=None,
        )
        conflicting = self.write_rollout(
            "2026/07/20/rollout-conflict.jsonl",
            session_id="session-conflict",
            second_version="v1",
        )
        future = self.write_rollout(
            "2026/08/01/rollout-future.jsonl",
            session_id="session-future",
            timestamp="2026-08-01T00:00:00Z",
        )

        frozen = module.freeze_inventory(
            [future, legacy, current, conflicting, missing],
            sessions_root=self.root,
            as_of="2026-07-31T23:59:59Z",
            min_bytes=0,
            max_bytes=1024 * 1024,
        )

        self.assertEqual(
            [candidate.session_id for candidate in frozen.candidates],
            ["session-current"],
        )
        self.assertEqual(frozen.candidates[0].rollout_format, "CurrentNativeV2")
        self.assertEqual(frozen.as_of, "2026-07-31T23:59:59Z")

    def test_inventory_order_and_digest_are_enumeration_independent(self):
        first = self.write_rollout(
            "2026/07/20/rollout-b.jsonl", session_id="session-b"
        )
        second = self.write_rollout(
            "2026/07/19/rollout-a.jsonl", session_id="session-a"
        )
        kwargs = {
            "sessions_root": self.root,
            "as_of": "2026-07-31T23:59:59Z",
            "min_bytes": 0,
            "max_bytes": 1024 * 1024,
        }

        forward = module.freeze_inventory([first, second], **kwargs)
        reverse = module.freeze_inventory([second, first], **kwargs)

        self.assertEqual(forward.candidates, reverse.candidates)
        self.assertEqual(forward.digest, reverse.digest)
        self.assertEqual(
            [candidate.session_id for candidate in forward.candidates],
            ["session-a", "session-b"],
        )

    def test_inventory_content_change_changes_digest(self):
        first = self.write_rollout(
            "2026/07/20/rollout-a.jsonl", session_id="session-a"
        )
        kwargs = {
            "sessions_root": self.root,
            "as_of": "2026-07-31T23:59:59Z",
            "min_bytes": 0,
            "max_bytes": 1024 * 1024,
        }
        before = module.freeze_inventory([first], **kwargs)
        with first.open("a") as handle:
            handle.write(
                json.dumps(
                    {
                        "timestamp": "2026-07-20T12:00:01Z",
                        "type": "event_msg",
                        "payload": {"type": "task_complete"},
                    }
                )
                + "\n"
            )
        after = module.freeze_inventory([first], **kwargs)

        self.assertNotEqual(before.digest, after.digest)

    def test_quota_authority_is_exact_and_immutable(self):
        config = module.freeze_quota_config(seed=42)

        ordinary = [
            bucket for bucket in config.buckets if bucket.risk_class == "ordinary"
        ]
        high_risk = [
            bucket for bucket in config.buckets if bucket.risk_class == "high_risk"
        ]
        self.assertTrue(ordinary)
        self.assertTrue(high_risk)
        self.assertTrue(all(bucket.quota == 3 for bucket in ordinary))
        self.assertTrue(all(bucket.mandatory_population == 3 for bucket in ordinary))
        self.assertTrue(all(bucket.quota == 5 for bucket in high_risk))
        self.assertTrue(all(bucket.mandatory_population == 5 for bucket in high_risk))
        self.assertEqual(
            set(config.required_families),
            {"language_repo", "workflow", "tooling", "delegation"},
        )
        with self.assertRaises(dataclasses.FrozenInstanceError):
            config.seed = 7
        with self.assertRaises(AttributeError):
            config.buckets.append(config.buckets[0])


if __name__ == "__main__":
    unittest.main()
