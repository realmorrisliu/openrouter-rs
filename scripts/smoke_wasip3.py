"""Exercise the SDK in Wasmtime against a credential-free loopback HTTP server."""

import email.parser
import email.policy
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit


def main():
    acknowledged = threading.Event()
    cancelled = threading.Event()
    failures = []
    seen = set()

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *_):
            pass

        def reply(self, body, status=200, content_type="application/json"):
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            self.wfile.flush()

        def body(self):
            if self.headers.get("Transfer-Encoding") != "chunked":
                return self.rfile.read(int(self.headers.get("Content-Length", "0")))
            chunks = []
            while True:
                size = int(self.rfile.readline().split(b";", 1)[0], 16)
                if not size:
                    assert self.rfile.readline() == b"\r\n"
                    return b"".join(chunks)
                chunks.append(self.rfile.read(size))
                assert self.rfile.read(2) == b"\r\n"

        def do_GET(self):
            self.handle_request()

        def do_POST(self):
            self.handle_request()

        def handle_request(self):
            try:
                self.connection.settimeout(10)
                assert self.headers["Authorization"] == "Bearer smoke-test-only"
                url = urlsplit(self.path)
                seen.add((self.command, url.path))
                if self.command == "GET" and url.path == "/files":
                    assert parse_qs(url.query) == {"limit": ["1"], "cursor": ["a b&c"]}
                    self.reply(b'{"data":[],"has_more":false}')
                elif self.command == "POST" and url.path == "/files":
                    raw = (
                        "Content-Type: " + self.headers["Content-Type"] + "\r\n\r\n"
                    ).encode() + self.body()
                    form = email.parser.BytesParser(policy=email.policy.default).parsebytes(raw)
                    parts = list(form.iter_parts())
                    assert len(parts) == 1
                    assert parts[0].get_param("name", header="content-disposition") == "file"
                    assert parts[0].get_filename() == "smoke.txt"
                    assert parts[0].get_content_type() == "text/plain"
                    assert parts[0].get_payload(decode=True) == b"upload body"
                    self.reply(json.dumps({
                        "id": "uploaded", "type": "file", "filename": "smoke.txt",
                        "mime_type": "text/plain", "size_bytes": 11,
                        "created_at": "2026-01-01", "downloadable": True,
                    }).encode())
                elif url.path == "/files/binary/content":
                    self.reply(b"a\0b", content_type="application/octet-stream")
                elif url.path == "/files/error/content":
                    self.reply(b'{"error":{"code":429,"message":"test rate limit"}}', 429)
                elif url.path == "/files/truncated/content":
                    self.send_response(200)
                    self.send_header("Content-Length", "100")
                    self.end_headers()
                    self.wfile.write(b"short")
                    self.wfile.flush()
                    self.close_connection = True
                elif url.path == "/files/ack/content":
                    acknowledged.set()
                    self.reply(b"ok")
                elif url.path == "/files/cancelled/content":
                    assert cancelled.wait(10), "dropping the stream did not close its HTTP body"
                    self.reply(b"ok")
                elif self.command == "POST" and url.path == "/chat/completions":
                    request = json.loads(self.body())
                    assert request["stream"] is True
                    assert request["model"] in ("stream", "cancel")
                    self.send_response(200)
                    self.send_header("Content-Type", "text/event-stream")
                    self.send_header("Transfer-Encoding", "chunked")
                    self.end_headers()

                    def chunk(data):
                        self.wfile.write(f"{len(data):x}\r\n".encode() + data + b"\r\n")
                        self.wfile.flush()

                    def event(text):
                        data = {
                            "id": "test", "created": 0, "model": "test",
                            "object": "chat.completion.chunk",
                            "choices": [{"index": 0, "delta": {"content": text}}],
                        }
                        return ("data: " + json.dumps(data) + "\n\n").encode()

                    chunk(event("first"))
                    if request["model"] == "cancel":
                        try:
                            assert self.connection.recv(1) == b"", "expected stream cancellation"
                        except ConnectionResetError:
                            pass
                        cancelled.set()
                        self.close_connection = True
                    else:
                        assert acknowledged.wait(10), "SSE was buffered until completion"
                        chunk(event("second"))
                        chunk(b"data: [DONE]\n\n")
                        self.wfile.write(b"0\r\n\r\n")
                        self.wfile.flush()
                else:
                    raise AssertionError(f"unexpected request: {self.command} {self.path}")
            except Exception as error:
                failures.append(repr(error))
                self.close_connection = True

    with ThreadingHTTPServer(("127.0.0.1", 0), Handler) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            command = [
                os.environ.get("WASMTIME", "wasmtime"), "run", "-S", "p3,http",
                "--env", f"OPENROUTER_TEST_URL=http://127.0.0.1:{server.server_port}",
                str(Path(sys.argv[1]).resolve()),
            ]
            result = subprocess.run(command, timeout=60, check=False)
            assert not failures, failures
            assert result.returncode == 0, f"WASI component exited {result.returncode}"
            expected = {("GET", "/files"), ("POST", "/files"), ("POST", "/chat/completions")}
            expected.update(
                ("GET", f"/files/{name}/content")
                for name in ("binary", "error", "truncated", "ack", "cancelled")
            )
            assert seen == expected, seen
        finally:
            server.shutdown()
            thread.join()
    print("WASI HTTP, multipart, binary, errors, incremental SSE and cancellation passed")


if __name__ == "__main__":
    main()
