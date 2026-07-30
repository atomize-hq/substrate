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

    def test_inventory_ignores_non_object_json_rows(self):
        current = self.write_rollout(
            "2026/07/20/rollout-current.jsonl", session_id="session-current"
        )
        original = current.read_text()
        current.write_text(json.dumps("non-object diagnostic") + "\n" + original)

        frozen = module.freeze_inventory(
            [current],
            sessions_root=self.root,
            as_of="2026-07-31T23:59:59Z",
            min_bytes=0,
            max_bytes=1024 * 1024,
        )

        self.assertEqual(
            [candidate.session_id for candidate in frozen.candidates],
            ["session-current"],
        )

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


class OverlappingQuotaSelectionTests(unittest.TestCase):
    @staticmethod
    def candidate(session_id):
        return module.Candidate(
            timestamp="2026-07-20T12:00:00Z",
            relative_path=f"2026/07/20/rollout-{session_id}.jsonl",
            session_id=session_id,
            path=f"/private/sessions/rollout-{session_id}.jsonl",
            cwd=f"/workspace/{session_id}",
            repo=f"repo-{session_id}",
            month="2026-07",
            size=2048,
            source_digest=(session_id.encode().hex() + "0" * 64)[:64],
        )

    @classmethod
    def labeled(cls, session_id, *labels):
        return module.LabeledCandidate(cls.candidate(session_id), frozenset(labels))

    @staticmethod
    def config(seed, *buckets):
        return module.QuotaConfig(
            schema_version=1,
            seed=seed,
            required_families=tuple(sorted({bucket.family for bucket in buckets})),
            buckets=tuple(sorted(buckets)),
        )

    def test_multi_label_candidate_credits_every_genuine_bucket(self):
        config = self.config(
            42,
            module.QuotaBucket("language_repo", "rust", 1, 1, "ordinary"),
            module.QuotaBucket("workflow", "verification", 1, 1, "ordinary"),
        )
        candidates = [
            self.labeled("overlap", "language_repo:rust", "workflow:verification"),
            self.labeled("rust-only", "language_repo:rust"),
            self.labeled("verification-only", "workflow:verification"),
        ]

        result = module.select_overlapping_quotas(candidates, config)

        self.assertEqual(
            [candidate.candidate.session_id for candidate in result.selected],
            ["overlap"],
        )
        self.assertTrue(all(bucket.selected == 1 for bucket in result.coverage))

    def test_seed_is_only_a_stable_tie_breaker(self):
        bucket = module.QuotaBucket("language_repo", "rust", 1, 1, "ordinary")
        candidates = [
            self.labeled("a", "language_repo:rust"),
            self.labeled("b", "language_repo:rust"),
            self.labeled("c", "language_repo:rust"),
        ]
        first = module.select_overlapping_quotas(
            list(reversed(candidates)), self.config(42, bucket)
        )
        repeat = module.select_overlapping_quotas(
            candidates, self.config(42, bucket)
        )
        self.assertEqual(first.selected, repeat.selected)
        self.assertEqual(first.selected_digest, repeat.selected_digest)

        choices = {
            module.select_overlapping_quotas(candidates, self.config(seed, bucket))
            .selected[0]
            .candidate.session_id
            for seed in range(1, 20)
        }
        self.assertGreater(len(choices), 1)

    def test_unknown_never_substitutes_for_named_bucket_or_nonempty_selection(self):
        config = self.config(
            42, module.QuotaBucket("language_repo", "rust", 3, 3, "ordinary")
        )
        with self.assertRaisesRegex(ValueError, "selected set must be non-empty"):
            module.select_overlapping_quotas(
                [self.labeled("unknown", "language_repo:unknown")], config
            )

    def test_sufficiently_populated_underfill_fails_validation(self):
        config = self.config(
            42, module.QuotaBucket("language_repo", "rust", 3, 3, "ordinary")
        )
        candidates = [
            self.labeled("a", "language_repo:rust"),
            self.labeled("b", "language_repo:rust"),
            self.labeled("c", "language_repo:rust"),
        ]
        with self.assertRaisesRegex(ValueError, "sufficiently populated"):
            module.validate_selection(config, candidates, candidates[:2])

    def test_sparse_inventory_underfill_selects_every_eligible_candidate(self):
        config = self.config(
            42, module.QuotaBucket("language_repo", "rust", 3, 3, "ordinary")
        )
        candidates = [
            self.labeled("a", "language_repo:rust"),
            self.labeled("b", "language_repo:rust"),
        ]

        result = module.select_overlapping_quotas(candidates, config)

        self.assertEqual(len(result.selected), 2)
        self.assertEqual(result.coverage[0].eligible, 2)
        self.assertEqual(result.coverage[0].selected, 2)
        self.assertEqual(result.coverage[0].underfill, 1)
        self.assertEqual(
            result.coverage[0].status, "permitted_inventory_scarcity"
        )

    def test_observable_labeling_assigns_all_four_stratum_families(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            path = pathlib.Path(temp_dir) / "rollout-labeled.jsonl"
            rows = [
                {
                    "timestamp": "2026-07-20T12:00:00Z",
                    "type": "session_meta",
                    "payload": {
                        "id": "session-labeled",
                        "cwd": "/workspace/rust-repo",
                        "multi_agent_version": "v2",
                        "base_instructions": {
                            "text": "Generic examples mention python, pytest, and npm."
                        },
                    },
                },
                {
                    "timestamp": "2026-07-20T12:00:01Z",
                    "type": "response_item",
                    "payload": {
                        "type": "message",
                        "role": "user",
                        "content": [
                            {
                                "type": "input_text",
                                "text": "Implement and review src/lib.rs, then run cargo test.",
                            }
                        ],
                    },
                },
                {
                    "timestamp": "2026-07-20T12:00:02Z",
                    "type": "response_item",
                    "payload": {
                        "type": "custom_tool_call",
                        "name": "spawn_agent",
                        "call_id": "call-labeled",
                    },
                },
            ]
            path.write_text("".join(json.dumps(row) + "\n" for row in rows))
            candidate = self.candidate("labeled")
            candidate = dataclasses.replace(candidate, path=str(path), cwd="/workspace/rust-repo")

            labeled = module.assign_observable_labels(candidate)

        self.assertIn("language_repo:rust", labeled.labels)
        self.assertIn("workflow:implementation", labeled.labels)
        self.assertIn("workflow:verification", labeled.labels)
        self.assertIn("workflow:review_fix", labeled.labels)
        self.assertIn("workflow:mixed", labeled.labels)
        self.assertIn("tooling:cargo_rust", labeled.labels)
        self.assertIn("delegation:delegated_parent_opaque", labeled.labels)
        self.assertNotIn("language_repo:python", labeled.labels)
        self.assertNotIn("tooling:python_pytest", labeled.labels)

    def test_selection_receipt_contains_only_sanitized_authority(self):
        config = self.config(
            42, module.QuotaBucket("language_repo", "rust", 1, 1, "ordinary")
        )
        labeled = [self.labeled("private-a", "language_repo:rust")]
        inventory = module.FrozenInventory(
            "2026-07-31T23:59:59Z",
            tuple(candidate.candidate for candidate in labeled),
            "a" * 64,
        )
        selection = module.select_overlapping_quotas(labeled, config)

        receipt = module.build_selection_receipt(inventory, config, selection)
        serialized = json.dumps(receipt, sort_keys=True)

        self.assertEqual(receipt["inventory"]["route"], "CurrentNativeV2")
        self.assertEqual(receipt["selection"]["selected_count"], 1)
        self.assertEqual(receipt["coverage"][0]["eligible"], 1)
        self.assertEqual(receipt["coverage"][0]["selected"], 1)
        self.assertFalse(receipt["privacy"]["raw_private_fields_included"])
        for private_value in (
            "private-a",
            "/private/sessions",
            "/workspace",
            "repo-private-a",
        ):
            self.assertNotIn(private_value, serialized)


if __name__ == "__main__":
    unittest.main()
