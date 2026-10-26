# openrouter-rs adoption plan: October 2026

Status: proposed execution plan; implementation and distribution have not started.
Planning date: 2026-10-13. Target review date: 2026-11-10. Dates use Asia/Shanghai.

## Outcome and scope

Make openrouter-rs easier to discover, evaluate, install, and keep using. Deliver four workstreams: onboarding, useful release content, downstream adoption, and measurement. Regaining first place in the OpenRouter keyword list is an outcome to track, not a guaranteed delivery date.

Use the existing SDK, examples, personal website, and X account. Keep the positioning precise: a community-maintained Rust SDK with broad OpenRouter-specific API support and a companion CLI. Link support claims to the endpoint matrix, which distinguishes implementation from live validation.

Primary journey: X or community post → useful tutorial on realmorrisliu.com → runnable GitHub example → installation → successful application integration.

## Verified starting point

- SDK repository: `/Users/morris/Developer/openrouter-rs`, version 0.19.0 released.
- Website repository: `/Users/morris/Developer/github.com/realmorrisliu/realmorrisliu.com`.
- Website: Astro content collection under `src/content/blog/`, existing `/thoughts` routes, RSS, and a PostHog component included by `src/layouts/Layout.astro`.
- Website social links identify `https://x.com/realmorrisliu`. Account session, publishing access, and analytics access remain to be verified at execution time.
- The website checkout currently contains unrelated work. Use an isolated branch/worktree for website changes after checking its current state; preserve existing changes.
- `examples/axum_chat_gateway.rs` already implements a non-streaming gateway. Streaming and typed-tool examples also exist. Extend or explain these instead of creating a separate SDK or demo application.
- README puts multiple historical compatibility updates before installation and Quick Start.
- `docs/community/awesome-openrouter/README.md` still recommends 0.8.0 and contains an April traction snapshot. The snapshot is historical, but the installation guidance needs updating.
- OpenRouter already lists the project: <https://openrouter.ai/works-with-openrouter/openrouter-rs>. Inspect the live listing's destination before proposing any upstream change.

Baseline captured during the 2026-10-13 investigation:

| Metric | openrouter-rs | rai-sdk |
| --- | ---: | ---: |
| Recent downloads reported by crates.io | 10,179 | 10,252 |
| All-time downloads | 29,878 | 10,252 |
| Downloads, 2026-09-13 through 2026-10-12 | 4,235 | 6,454 |
| Downloads, 2026-10-06 through 2026-10-12 | 1,414 | 1,695 |
| GitHub stars | 97 | 1 |
| Published reverse dependencies reported by crates.io | 4 | 0 |

These are timestamped observations, not forecasts. Published reverse dependencies exclude unpublished applications; the four SDK dependents include its own CLI. Daily downloads aggregate `version_downloads` and `meta.extra_downloads`; recent-download totals and daily statistics may refresh at different times.

Sources: [keyword ranking API](https://crates.io/api/v1/crates?keyword=openrouter&per_page=100&sort=recent-downloads), [SDK daily downloads](https://crates.io/api/v1/crates/openrouter-rs/downloads), [rai daily downloads](https://crates.io/api/v1/crates/rai-sdk/downloads), [SDK dependents](https://crates.io/api/v1/crates/openrouter-rs/reverse_dependencies?page=1&per_page=100), [ranking policy](https://rust-lang.github.io/rfcs/1824-crates.io-default-ranking.html).

## Workstream 1: installation and first success

Deliver one focused SDK documentation PR.

- [ ] Reorder README: concise positioning → installation → smallest working example → three practical workflows → supported API reference and project policies.
- [ ] Move dated compatibility narratives out of the introduction. Keep their durable details in existing CHANGELOG/MIGRATION entries and link to them; verify information is preserved before deleting duplicate prose.
- [ ] Provide visible routes to chat/streaming, typed tools, and management/cost workflows, reusing canonical domain clients.
- [ ] Repair current installation guidance in the community setup page. Preserve its stable URL and separate historical submission material from current setup instructions.
- [ ] Link version-sensitive guidance to maintained sources; remove duplicated current-version claims where possible. Cover remaining release-version pins in the existing release synchronization check if needed.
- [ ] Check README, docs map, crate docs, community setup page, and CLI links as one newcomer journey. Link CLI version support accurately; a workspace dependency edit does not publish a CLI release.

Acceptance: from a clean temporary Rust project, the documented published dependency compiles; with a valid key, the quick start returns a response. Commands, environment variables, model selection, and failure guidance are explicit. All internal links resolve. A real newcomer should be able to attempt this without reading the API matrix first; record observed usability feedback rather than claiming an unmeasured five-minute conversion rate.

GitHub README changes become visible on merge. The crates.io README is a published-package surface: verify its behavior and include changed package documentation in the next justified release using the release skill. Do not mark crates.io onboarding updated merely because main changed.

## Workstream 2: website tutorials and X distribution

Deliver two practical articles in the existing website and a small reusable publication pack in the SDK's community docs. English is the initial language for Rust ecosystem reach; prioritize a Chinese companion only if reader feedback supports it. Keep code in SDK examples and pin article code links to a release or commit that was actually tested.

### Article A: streaming chat with Rust, Axum, and OpenRouter

- [ ] Reuse the existing Axum gateway and streaming primitives. Add the smallest streaming route/example needed for a complete tutorial.
- [ ] Cover setup, a curl request, streamed output, completion, upstream failure, and client disconnect. Test cancellation before promising that dropping a downstream connection cancels upstream work.
- [ ] Keep the key server-side and the demo bound to localhost by default. Bound input/output and timeouts; explain the authentication and quota work required before public deployment.
- [ ] Describe available usage/cost fields without inventing zero values when usage is absent. Link existing generation lookup guidance where relevant.
- [ ] End with one clear action: run the example, followed by a link to report a concrete integration problem.

### Article B: typed tools in a bounded Rust agent loop

- [ ] Reuse `typed_tool_agent.rs` and typed-tool examples, including the existing maximum-step bound.
- [ ] Explain typed arguments, tool-result messages, invalid arguments, unknown tools, and stopping conditions. Use deterministic demonstration tools.
- [ ] Explain when the direct OpenRouter SDK is useful, using verified features rather than unsupported superiority claims.
- [ ] Include the exact tested version, prerequisites, complete runnable code link, and expected output shape.

### Website delivery

- [ ] Add articles under `src/content/blog/` using its existing schema, tags, draft state, and layouts.
- [ ] Cross-link from relevant existing Rust content and the existing openrouter-rs introduction. Reuse site styling.
- [ ] Validate title, description, canonical URL, RSS/sitemap inclusion, social preview, mobile code blocks, and all installation links.
- [ ] Use the website's normal PR/main deployment path through Cloudflare Workers Builds, followed by live URL verification.

### X publication pack and cadence

- [ ] Prepare one short 0.19.0 announcement, one 4–6-post thread for each article, and one follow-up showing a useful result or answering a real reader question.
- [ ] Each article thread starts with the problem and a short working demo, explains 2–3 implementation decisions, and links to the full tutorial and runnable source. Use real captured output, with credentials removed.
- [ ] Use the actual publication date. Describe 0.19.0 as the current release if publication is no longer close to release day.
- [ ] Space primary posts by at least 48 hours initially. Choose publication time from available account analytics; otherwise test two time slots and record results.
- [ ] Record each published URL and inspect results after 48 hours and seven days. Respond to concrete technical questions; turn repeated friction into targeted SDK/docs fixes.
- [ ] Consider This Week in Rust after Article A is live and reproducible. Verify current submission instructions and submit one relevant tutorial item, not duplicate promotions.

Use stable campaign parameters, for example `utm_source=x&utm_medium=social&utm_campaign=openrouter-rs-2026-10&utm_content=axum-thread`. Keep the article's canonical URL free of campaign parameters. Website article body is the primary long-form source; X carries excerpts and GitHub carries executable code.

Acceptance: both articles have passing builds and verified examples; publication drafts contain working destinations and readable previews. A publication is complete only after its live URL is checked. Material preparation, publication, community acceptance, and resulting adoption are separate statuses.

## Workstream 3: downstream adoption

Deliver a small candidate ledger with project, use case, existing implementation, integration fit, last activity, proposed change, evidence URL, and status.

- [ ] Inspect existing published SDK consumers first: git-acm, kiku, zo-cli, and the companion CLI. Verify current repository and dependency state before proposing upgrades; an older SDK version alone is not a bug.
- [ ] Inspect the user's Alan and other relevant projects as candidates, then choose only where an actual OpenRouter workflow benefits. Do not add a dependency just to manufacture a dependent.
- [ ] Research five additional active Rust CLI/agent/backend projects. Prioritize a requested or documented OpenRouter use case and maintainers open to integrations.
- [ ] Select two high-fit external opportunities; produce one minimal working patch/demo per selected opportunity. Prefer direct usage and an existing extension interface over introducing a new adapter crate.
- [ ] Prepare a specific maintainer-facing proposal: current problem, before/after behavior, test evidence, maintenance burden, and link to the runnable example. Follow each project's contribution rules.
- [ ] When accepted, record the merge/release or maintainer-confirmed usage. Add a factual downstream showcase only with public evidence and suitable attribution.

Execution target: a ledger of at least five qualified external candidates, two complete integration proposals, and at least one adoption pursued through to a verified outcome. Two new independently maintained adopters is a stretch outcome; third-party acceptance is not controlled by this project.

## Workstream 4: measurement and weekly decisions

Start measurement before publishing content. Reuse the website's existing PostHog installation; first verify production event delivery and authorized dashboard access.

- [ ] Establish a daily crates.io snapshot with timestamp, SDK and competitor recent/all-time downloads, complete-day 7/30-day totals, keyword rank, and published reverse dependencies. Read enough pages to compute rank correctly if the keyword grows beyond one API page.
- [ ] Use a small standard-library script and one scheduled/manual GitHub Actions workflow if existing infrastructure cannot already capture it. Persist snapshots in retained artifacts or another existing durable store, with retention covering this campaign; avoid a daily source-code commit.
- [ ] Treat network failure, throttling, and missing fields as missing data, never as zero. Limit requests and retries, identify the client, deduplicate same-date snapshots, and exclude the unfinished UTC day from trailing daily sums.
- [ ] Verify existing PostHog autocapture first. Add only missing article-to-GitHub/crates click and install-command-copy events, with article/campaign identifiers and no prompt, API key, or user-entered content.
- [ ] Track X impressions and link clicks when the account exposes them. Record unavailable analytics as unavailable; no paid tool is a prerequisite.
- [ ] Keep one weekly table: content/post links, article visits, available outbound clicks, SDK downloads, qualified feedback, downstream proposals, verified adoption, and next action.

Measurement boundaries: website clicks are not installs; crates.io cannot attribute a download to an X post or individual reader. Compare periods and campaign observations as directional evidence, not causal attribution. Separate independent downstream adopters from this repository's CLI. Stars remain a secondary signal.

Suggested outcome targets, to calibrate at the first weekly review:

| Target | Definition |
| --- | --- |
| Delivery | Revised onboarding, two validated articles, two X threads plus announcement/follow-up, two integration proposals, operational measurement |
| Download growth | Reach at least 6,000 complete-day trailing-30-day downloads by the final review, about 42% over the 4,235 baseline; an experiment target, not a forecast |
| Adoption | Seek two new independently maintained applications; count merged/released or maintainer-confirmed usage, including unpublished applications when publicly evidenced |
| Ranking stretch | First place in recent-download sorting for 14 consecutive daily snapshots; compare all matching crates, not only rai-sdk |

Decision rules:

- Low article visits: improve the opening/demo and distribution before adding SDK features.
- Visits without useful outbound activity: simplify the article's first-success path and action links; inspect tracking validity first.
- Reported install/runtime failures: prioritize those fixes over the next promotional post.
- Higher downloads without independently verified adoption: report the increase without claiming equivalent user growth.
- Little response after two distribution attempts and seven days: revise the content/use case; do not repeat the same post indefinitely.

## Sequence, ownership, and review

Dates are a proposed cadence starting on 2026-10-13; shift them together if execution starts later.

| Window | Deliverables and dependency |
| --- | --- |
| Oct 13–15 | SDK onboarding PR, baseline capture, analytics/access verification; these precede distribution |
| Oct 16–20 | Article A + tested streaming example, website PR/deployment, release announcement and first X thread; first weekly review Oct 20 |
| Oct 21–27 | Article B + second thread, first integration proposal, tutorial/community submission where suitable; review Oct 27 |
| Oct 28–Nov 3 | Second integration proposal, respond to feedback, fix observed onboarding problems; review Nov 3 |
| Nov 4–10 | Follow through on adoption, check sustained rank and download trends, decide next month's focus; final review Nov 10 |

The assistant can prepare repository changes, tests, articles, publication drafts, candidate research, and reports when execution is requested. Morris owns the personal voice and the publisher/maintainer account context. The user's website and X channel suggestion is incorporated here; this planning turn itself does not publish posts, contact downstream maintainers, activate schedules, merge changes, or deploy the site. At execution time, use the authorization then in scope and request only genuinely missing account access or concrete external-action approval, after preparing the reviewable material.

Suggested PR boundaries: SDK onboarding; metrics collection; SDK tutorial example changes; website articles/instrumentation. Keep dependencies explicit: a published tutorial points to code that is already accessible and validated. Batch small related changes where that reduces coordination without mixing repositories or obscuring review.

## Validation and completion evidence

- SDK: `just quality` before opening a PR. Run `just quality-ci` for public SDK types/surfaces, CLI behavior, migration, or CI-aligned release/test changes. Run the explicit migration checks for canonical naming or migration guidance changes. Inspect current-commit GitHub checks after opening/updating each PR.
- Tutorial runtime logic: focused local mock coverage for streaming completion/error/disconnect and tool-loop failure paths; a bounded live smoke with a valid API key for the published example. Do not equate mock success or missing-key skips with live success.
- Metrics script: small fixture/assert tests for all-version daily aggregation, incomplete-day exclusion, pagination/rank, missing data, and duplicate snapshots. Workflow smoke through its manual trigger before enabling its schedule.
- Website: `pnpm check`, `pnpm build`, preview smoke of the new articles plus `/`, `/thoughts`, `/so-far`, `/now`; inspect desktop/mobile rendering and production URLs after deployment. Follow the website repository's current AGENTS.md.
- Delivery report: PR/commit links, test results, article and X URLs, campaign parameters, proposal/adoption evidence, dated metric snapshots, and remaining dependencies. Mark planned, drafted, published, accepted, and measured separately.

Scope intentionally excludes a new marketing site, paid advertising, a new analytics stack, artificial downloads, an unrelated SDK API expansion, and a release performed solely to move the ranking. The main uncertainty is external response: download growth, editorial selection, and downstream acceptance cannot be guaranteed by completing the implementation checklist.
