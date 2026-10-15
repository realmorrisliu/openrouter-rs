# Typed tools tutorial — published X thread

Published on October 15, 2026 at 11:47 Asia/Shanghai:
https://x.com/realmorrisliu/status/2110578158118605288

The five posts below record the published thread. The production article, pinned
source, and live verification run were checked before publication. All five posts
were verified in the public conversation after X sent the scheduled thread.

Publication links, in order:

1. https://x.com/realmorrisliu/status/2110578158118605288
2. https://x.com/realmorrisliu/status/2110578158370164877
3. https://x.com/realmorrisliu/status/2110578158638616715
4. https://x.com/realmorrisliu/status/2110578158919651563
5. https://x.com/realmorrisliu/status/2110578159209046311

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
Inspect results after 48 hours and seven days; exclude QA visits from PostHog.
The screenshots/transcripts must use the labeled demo fixtures and contain no credentials.
