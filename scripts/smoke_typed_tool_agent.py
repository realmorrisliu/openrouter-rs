#!/usr/bin/env python3
"""Run the actual typed-tool example; require both tools and a completed answer."""

import argparse
import json
import os
from pathlib import Path
import subprocess


def validate_trace(text):
    records = [json.loads(line) for line in text.splitlines() if line.strip()]
    assert all(row["event"] in ("completion", "tool_result", "final") for row in records)
    positions = {row["step"]: index for index, row in enumerate(records) if row["event"] == "completion"}
    completions = [row for row in records if row["event"] == "completion"]
    results = [row for row in records if row["event"] == "tool_result"]
    finals = [row for row in records if row["event"] == "final"]
    assert 2 <= len(completions) <= 4, "A real tool round trip is required"
    assert len(finals) == 1 and records[-1] == finals[0], "One terminal final answer is required"
    assert [row["step"] for row in completions] == list(range(1, len(completions) + 1))
    expected_calls = {}
    generations = []
    for row in completions:
        response = row["response"]
        assert response["id"].startswith("gen-"), "Expected real OpenRouter generation IDs"
        assert response["model"]
        usage = response["usage"]
        for field in ("prompt_tokens", "completion_tokens", "total_tokens"):
            assert type(usage[field]) is int and usage[field] >= 0, f"Missing valid {field}"
        choice = response["choices"][0]
        calls = choice["message"].get("tool_calls") or []
        assert choice["finish_reason"] in ("stop", "tool_calls") and not choice.get("error")
        for call in calls:
            assert call["id"] and call["id"] not in expected_calls
            expected_calls[call["id"]] = (row["step"], call["function"]["name"])
        generations.append({"id": response["id"], "model": response["model"], "usage": usage})
    assert len(results) == len(expected_calls), "Every requested tool needs a result"
    seen = set()
    for index, row in enumerate(records):
        if row["event"] != "tool_result":
            continue
        assert positions[row["step"]] < index < positions.get(row["step"] + 1, len(records))
        assert row["tool_call_id"] not in seen
        seen.add(row["tool_call_id"])
        assert expected_calls[row["tool_call_id"]] == (row["step"], row["name"])
        assert row["result"].get("demo") is True and "error" not in row["result"]
        assert row["result"]["service"] == "checkout"
    assert {row["name"] for row in results} == {"get_deployment_status", "lookup_runbook"}
    assert completions[-1]["response"]["choices"][0]["finish_reason"] == "stop"
    assert not completions[-1]["response"]["choices"][0]["message"].get("tool_calls")
    final = finals[0]
    assert final["step"] == completions[-1]["step"]
    assert isinstance(final["content"], str) and final["content"].strip()
    assert final["content"] == completions[-1]["response"]["choices"][0]["message"]["content"]
    return {"requests": len(completions), "tool_results": len(results), "generations": generations,
            "final_answer": final["content"]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", default="target/debug/examples/typed_tool_agent")
    args = parser.parse_args()
    assert os.environ.get("OPENROUTER_API_KEY"), "OPENROUTER_API_KEY is required; no skipping"
    output = Path("typed-tool-smoke-artifacts")
    output.mkdir(exist_ok=True)
    with (output / "trace.jsonl").open("w") as trace, (output / "stderr.txt").open("w") as errors:
        result = subprocess.run([str(Path(args.binary).resolve())], stdout=trace, stderr=errors, text=True, timeout=90)
    assert result.returncode == 0, f"Agent failed with exit code {result.returncode}; inspect artifacts"
    report = validate_trace((output / "trace.jsonl").read_text())
    (output / "result.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
