# Typed tools tutorial — X thread draft

Status: prepared, not published. Publish after the production article is verified.
Keep at least 48 hours between primary threads; the Axum thread was published on
October 13 at 11:47 Asia/Shanghai, so October 15 at 11:47 is the earliest slot.
Choose the actual time from account analytics when available.

## 1/5

I added a runnable typed-tools tutorial for openrouter-rs.

A model requests deployment context, Rust parses the arguments and returns tool results, then the model answers. The tools use fixed demo data; they don't touch real infrastructure.

## 2/5

Rust types don't make model arguments trustworthy.

An environment enum rejects "prod" when the schema expects "production". The dispatcher also checks the service. Bad arguments become JSON tool feedback; network failures and exhausted limits stop the run.

## 3/5

Keep the assistant message that requested the tool, then append a tool result with its call ID.

The example preserves modeled reasoning details too. Each request carries both tool schemas, with a four-request limit and one total deadline.

## 4/5

Verified with DeepSeek V4.1 Flash: two model requests, two tool results, a final answer, and usage from both completions.

The first response contained both tool calls. The program executes them sequentially and checks the returned count locally.

## 5/5

Run the pinned example and its live smoke check:

https://realmorrisliu.com/thoughts/typed-tools-openrouter-rust-agent?utm_source=x&utm_medium=social&utm_campaign=openrouter-rs-2026-10&utm_content=typed-tools-thread

---

Evidence: https://github.com/realmorrisliu/openrouter-rs/actions/runs/38362237401
Source: https://github.com/realmorrisliu/openrouter-rs/blob/f6d4a06832e891c51f00e178e122fa7ffbe0f906/examples/typed_tool_agent.rs

Seven local tests and a separate project using published SDK 0.19.0 passed.
Retain the actual publication URLs and date when the thread is published.
Inspect results after 48 hours and seven days; exclude QA visits from PostHog.
The screenshots/transcripts must use the labeled demo fixtures and contain no credentials.
