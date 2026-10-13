import importlib.util
import json
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "smoke", Path(__file__).resolve().parents[1] / "scripts/smoke_typed_tool_agent.py"
)
smoke = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(smoke)


def successful_trace():
    calls = [{"id": name, "function": {"name": name}} for name in ("get_deployment_status", "lookup_runbook")]
    def completion(step, content, tools, finish):
        return {"event": "completion", "step": step, "response": {
            "id": f"gen-test-{step}", "model": "model",
            "usage": {"prompt_tokens": 10, "completion_tokens": 10, "total_tokens": 20},
            "choices": [{"message": {"content": content, "tool_calls": tools}, "finish_reason": finish}]}}
    return [completion(1, None, calls, "tool_calls"),
            *[{"event": "tool_result", "step": 1, "tool_call_id": call["id"], "name": call["function"]["name"],
               "result": {"demo": True, "service": "checkout"}} for call in calls],
            completion(2, "Answer", [], "stop"), {"event": "final", "step": 2, "content": "Answer"}]


def encode(rows):
    return "\n".join(json.dumps(row) for row in rows)


class TypedToolSmokeTests(unittest.TestCase):
    def test_accepts_complete_correlated_tool_round_trip(self):
        self.assertEqual(smoke.validate_trace(encode(successful_trace()))["tool_results"], 2)

    def test_rejects_missing_tool_result_final_answer_and_usage(self):
        for kind in ("tool", "final", "usage", "empty", "truncated", "mismatched", "error", "reordered"):
            rows = successful_trace()
            if kind == "tool":
                rows.pop(1)
            elif kind == "final":
                rows.pop()
            elif kind == "usage":
                rows[0]["response"]["usage"] = {}
            elif kind == "empty":
                rows[-1]["content"] = " "
            elif kind == "truncated":
                rows[-2]["response"]["choices"][0]["finish_reason"] = "length"
            elif kind == "mismatched":
                rows[1]["tool_call_id"] = "unknown"
            elif kind == "error":
                rows[1]["result"]["error"] = "invalid_arguments"
            elif kind == "reordered":
                rows[1], rows[-2] = rows[-2], rows[1]
            with self.subTest(kind=kind), self.assertRaises((AssertionError, KeyError)):
                smoke.validate_trace(encode(rows))


if __name__ == "__main__":
    unittest.main()
