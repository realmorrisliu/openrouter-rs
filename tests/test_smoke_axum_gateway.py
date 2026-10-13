import importlib.util
import json
from pathlib import Path
import unittest


SPEC = importlib.util.spec_from_file_location(
    "smoke", Path(__file__).resolve().parents[1] / "scripts/smoke_axum_gateway.py"
)
smoke = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(smoke)


def transcript(usage):
    chunks = [
        {"id": "gen-test", "choices": [{"delta": {"content": "Hello"}}]},
        {"id": "gen-test", "choices": [], "usage": usage},
    ]
    return "".join(f"event: chunk\ndata: {json.dumps(chunk)}\n\n" for chunk in chunks) + "event: done\ndata: {}\n\n"


class TutorialSmokeTests(unittest.TestCase):
    def test_final_usage_is_required_even_with_text_and_done(self):
        for usage in (None, {}, [], "usage"):
            with self.subTest(usage=usage), self.assertRaises(AssertionError):
                smoke.validate_transcript(transcript(usage))

    def test_earlier_usage_does_not_replace_final_usage(self):
        early = {"id": "gen-test", "choices": [], "usage": {
            "prompt_tokens": 3, "completion_tokens": 0, "total_tokens": 3,
        }}
        with self.assertRaises(AssertionError):
            smoke.validate_transcript(
                f"event: chunk\ndata: {json.dumps(early)}\n\n" + transcript(None)
            )

    def test_each_token_field_is_required_and_valid(self):
        for field in ("prompt_tokens", "completion_tokens", "total_tokens"):
            for invalid in (None, -1, "2", True, 1.5):
                usage = {"prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5}
                usage[field] = invalid
                with self.subTest(field=field, invalid=invalid), self.assertRaises(AssertionError):
                    smoke.validate_transcript(transcript(usage))

    def test_cost_remains_optional_and_zero_is_preserved(self):
        for cost in (None, 0, 0.001):
            usage = {"prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5}
            if cost is not None:
                usage["cost"] = cost
            chunks, text, actual = smoke.validate_transcript(transcript(usage))
            self.assertEqual(len(chunks), 2)
            self.assertEqual(text, "Hello")
            self.assertEqual(actual, usage)

    def test_usage_does_not_hide_error_or_missing_done(self):
        valid = transcript({"prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5})
        for invalid in (
            valid.replace("event: done\ndata: {}\n\n", ""),
            valid.replace("event: done", "event: error\ndata: upstream_timeout\n\nevent: done"),
        ):
            with self.subTest(transcript=invalid), self.assertRaises(AssertionError):
                smoke.validate_transcript(invalid)


if __name__ == "__main__":
    unittest.main()
