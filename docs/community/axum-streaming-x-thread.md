# Axum streaming tutorial — published X thread

Published on October 13, 2026: https://x.com/realmorrisliu/status/2109853608443609370

The five posts below record the published thread. Later tutorial model changes do not rewrite this historical validation.

## 1/5

I wrote a runnable Rust + Axum tutorial for streaming OpenRouter chat with openrouter-rs.

One shared client, a POST endpoint, and SSE chunks. The example includes tests and a real API smoke check.

## 2/5

The happy path is only part of streaming.

An upstream can fail after HTTP 200. The example sends an explicit error event and never follows it with done. It also bounds input, output tokens, and total upstream time.

## 3/5

Don't stop reading at the first finish_reason: the final chunk can carry usage and cost.

The response body owns the upstream stream, so a disconnected caller releases it. Whether the provider stops generation/billing still depends on provider support.

## 4/5

Verified on October 13 with openai/gpt-4o-mini:

curl → Axum → OpenRouter → 43 chunks, generated text, final usage, and done.

Six local HTTP tests cover incremental delivery, errors, input limits, timeouts, and disconnects. Published SDK 0.19.0 was tested too.

## 5/5

Copy the pinned checkout and curl commands, then run the included smoke check with your own API key.

Tutorial + complete source + verification run:
https://realmorrisliu.com/thoughts/streaming-openrouter-rust-axum?utm_source=x&utm_medium=social&utm_campaign=openrouter-rs-2026-10&utm_content=axum-thread

---

No benchmark or production-readiness claim. The live result is one verified request, not an availability promise. The thread was published after the production article was verified.
