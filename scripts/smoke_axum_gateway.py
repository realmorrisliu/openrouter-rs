#!/usr/bin/env python3
"""Run the documented curl request against the real Axum example and OpenRouter."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
from urllib.error import URLError
from urllib.request import urlopen


def main():
    if not os.environ.get("OPENROUTER_API_KEY"):
        raise RuntimeError("OPENROUTER_API_KEY is required; this live check must not silently skip")
    environment = os.environ.copy()
    environment["OPENROUTER_AXUM_BIND"] = "127.0.0.1:3000"
    environment["OPENROUTER_MODEL"] = "openai/gpt-4o-mini"
    output = Path("axum-smoke-artifacts")
    output.mkdir(exist_ok=True)
    with tempfile.TemporaryFile() as server_log:
        server = subprocess.Popen(
            ["target/debug/examples/axum_chat_gateway"], env=environment,
            stdout=server_log, stderr=subprocess.STDOUT,
        )
        try:
            for _ in range(50):
                if server.poll() is not None:
                    raise RuntimeError("Gateway exited during startup")
                try:
                    with urlopen("http://127.0.0.1:3000/healthz", timeout=1) as response:
                        assert json.load(response) == {"ok": True}
                    break
                except (URLError, TimeoutError):
                    time.sleep(0.1)
            else:
                raise RuntimeError("Gateway did not become healthy")

            command = [
                "curl", "--no-buffer", "--silent", "--show-error", "--fail-with-body",
                "--max-time", "75", "http://127.0.0.1:3000/chat/stream",
                "-H", "content-type: application/json", "-d",
                json.dumps({"prompt": "Write three short sentences about Rust.", "max_tokens": 128}),
            ]
            started = time.monotonic()
            first_event = None
            lines = []
            with subprocess.Popen(command, stdout=subprocess.PIPE, text=True, encoding="utf-8") as curl:
                for line in curl.stdout:
                    lines.append(line)
                    if line.startswith("event: chunk") and first_event is None:
                        first_event = time.monotonic() - started
                assert curl.wait() == 0, "curl request failed"
            transcript = "".join(lines)
            (output / "stream.txt").write_text(transcript)
            chunks = []
            events = []
            for frame in transcript.split("\n\n"):
                event = next((line[7:] for line in frame.splitlines() if line.startswith("event: ")), None)
                if event is None:
                    continue
                events.append(event)
                if event == "chunk":
                    data = "\n".join(line[6:] for line in frame.splitlines() if line.startswith("data: "))
                    chunks.append(json.loads(data))
            assert events and events[-1] == "done" and events.count("done") == 1, events
            assert "error" not in events, transcript
            assert len(chunks) > 1, "Expected multiple real streamed chunks"
            text = "".join(choice.get("delta", {}).get("content") or "" for chunk in chunks for choice in chunk["choices"])
            assert text.strip(), "Stream returned no text"
            assert all(chunk["id"] for chunk in chunks)
            usage = next((chunk["usage"] for chunk in reversed(chunks) if chunk.get("usage")), None)
            report = {
                "model": chunks[0]["model"], "generation_id": chunks[0]["id"],
                "chunks": len(chunks), "first_event_seconds": first_event,
                "elapsed_seconds": time.monotonic() - started, "text": text, "usage": usage,
            }
            (output / "result.json").write_text(json.dumps(report, indent=2) + "\n")
            print(json.dumps(report, indent=2))
        finally:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait()


if __name__ == "__main__":
    main()
