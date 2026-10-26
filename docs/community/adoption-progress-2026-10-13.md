# openrouter-rs adoption progress: October 13, 2026

Status checked on October 13, 2026, Asia/Shanghai. This records execution against
[the campaign plan](adoption-plan-2026-10.md); preparation, publication, and adoption
remain separate milestones.

## Delivered

| Workstream | Verified delivery | Remaining work |
| --- | --- | --- |
| Onboarding | README and community setup page updated; SDK PR #267 merged | Changed README has not been included in a subsequent crate release |
| Tutorials and distribution | Axum SSE example, real API validation, article, and five-post X thread published; SDK #268/#269 and website #10/#11/#12 merged | Typed-tools article/thread, release announcement, reader-driven follow-up, and community submission |
| Downstream adoption | Existing consumers identified in the plan | Verify consumers, qualify five external candidates, prepare two integration proposals, and record accepted adoption |
| Measurement | Collector, tests, daily workflow, and 90-day snapshot retention merged; main manual run verified today | Verify website analytics delivery and useful outbound/copy events, obtain available X analytics, and establish weekly review records |

Delivery links:

- [Onboarding and metrics PR #267](https://github.com/realmorrisliu/openrouter-rs/pull/267)
- [Streaming example PR #268](https://github.com/realmorrisliu/openrouter-rs/pull/268)
- [Configurable reasoning correction PR #269](https://github.com/realmorrisliu/openrouter-rs/pull/269)
- [Initial article PR #10](https://github.com/realmorrisliu/realmorrisliu.com/pull/10),
  [DeepSeek update #11](https://github.com/realmorrisliu/realmorrisliu.com/pull/11),
  [configuration and editorial update #12](https://github.com/realmorrisliu/realmorrisliu.com/pull/12)
- [Live Axum tutorial](https://realmorrisliu.com/thoughts/streaming-openrouter-rust-axum/)
- [Published X thread](https://x.com/realmorrisliu/status/2109853608443609370)
- [Real API run for both DeepSeek configurations](https://github.com/realmorrisliu/openrouter-rs/actions/runs/38344606219)

The published X thread records its original GPT-4o mini validation. The updated
article records the later DeepSeek validation; these are distinct historical runs.
The article's latest prose, eight code blocks, and source/evidence links were
verified in production after #12 merged. The completed PR-monitor heartbeat is
paused.

## Current metrics

[Main-branch collection run](https://github.com/realmorrisliu/openrouter-rs/actions/runs/38353932519)
succeeded. Snapshot collected at 13:55:55 Asia/Shanghai; daily sums include only
complete UTC dates through October 12. The OpenRouter keyword list contains 52
crates.

| Metric | openrouter-rs | rai-sdk |
| --- | ---: | ---: |
| Keyword rank by recent downloads | 2 | 1 |
| Registry recent downloads | 10,179 | 10,252 |
| Complete 7-day downloads | 1,414 | 1,695 |
| Complete 30-day downloads | 4,235 | 6,454 |
| All-time downloads | 29,878 | 10,252 |
| Published dependents | 4 | 0 |

These values do not establish campaign impact. Publication began today, and the
complete-day sums exclude today. Dependents include the companion CLI; downloads
are requests, not unique users. There is not yet a daily sequence proving sustained
ranking improvement. The workflow is configured for 10:23 Asia/Shanghai daily;
the first scheduled run after merge still needs confirmation.

## Next execution order

1. Finish measurement acceptance. Confirm production PostHog event delivery and
   available analytics access, inspect existing autocapture before adding events,
   and record the X thread's available impressions/link activity. Review after 48
   hours on October 15 and after seven days on October 20.
2. Prepare Article B from `examples/typed_tool_agent.rs`. First verify tool-result
   messages, argument errors, unknown tools, empty responses, and stopping limits.
   Preserve configurable model selection and validate a bounded real tool-calling
   run before drafting claims or expected output. Apply the same problem-first,
   concise editorial style as the revised Axum article.
3. Build the downstream candidate ledger in parallel with content preparation:
   verify existing consumers first, then qualify five active external projects
   with an actual OpenRouter use case. Choose two proposals from evidence of fit.
4. Distribute verified work: prepare the typed-tools thread and one specific
   Axum follow-up, keep primary posts at least 48 hours apart, and inspect current
   This Week in Rust submission instructions before preparing a submission.
5. At the October 20 review, compare downloads and rank with the baseline, record
   qualified reader feedback and proposal outcomes, and prioritize observed
   integration failures over additional promotion.

A future justified release should include the new onboarding documentation.
Do not release solely to change ranking. Website analytics and independently
verified adoption remain unmeasured; third-party acceptance is still external.
