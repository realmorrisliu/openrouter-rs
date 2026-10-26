# Community distribution experiment — October 15, 2026

Status: the single authorized Reddit comment was published on October 15, then found removed on its logged-out public permalink. This placement is stopped. Discord remains a draft. Do not count preparation, link checks, previews, or maintainer visits as campaign conversion.

Publication: https://www.reddit.com/r/openrouter/comments/1wuvfzj/comment/pgujbmz/

Account: u/realMorrisLiu. Reddit displayed a successful-comment confirmation; reopening the permanent link in the signed-in account retained the comment. Publication was confirmed by 14:24:30 Asia/Shanghai (06:24:30 UTC); the exact server timestamp was not exposed in the inspected UI. The initial attempt to verify visibility without signing in encountered Reddit's human-verification page. No challenge was solved or bypassed. A subsequent logged-out retrieval of the exact comment permalink displayed `Comment removed by moderator` with the body absent; this was reconfirmed at 14:33 Asia/Shanghai. The page establishes public removal, but does not identify whether a human moderator or automated filter acted, the reason, or any account-wide ban. Signed-in author visibility is not evidence of public visibility. The actual article link target was inspected using the native link context menu and retained all four intended UTM parameters.

Immediately before posting, a fresh PostHog query over the production typed-tools article since October 13, excluding the known QA browser device, returned: 0 pageviews, 0 browser IDs, 0 GitHub click events, 0 matching Reddit landing pageviews. The temporary SQL was discarded without changing the saved Axum insight. No post-publication conversion is claimed.

Follow-up automation `openrouter` is PAUSED following the removal finding; its planned October 16, 17, and 22 observations are suspended. Do not repost, edit and resubmit, switch accounts, contact moderators, or move the same placement to another community without further authorization. The earlier signed-in screenshot `openrouter-reddit-monthly-published.png` in the task visualization directory records publication, not continued public availability.

## Channel selection

| Venue | Evidence checked October 15 | Decision |
| --- | --- | --- |
| r/OpenRouter monthly project thread | [Latest search result](https://www.reddit.com/r/openrouter/search/?q=MONTHLY%20MEGATHREAD&restrict_sr=1&sort=new), [current thread](https://www.reddit.com/r/openrouter/comments/1wuvfzj/monthly_megathread_what_are_you_working_on_with/). [Community rule 2](https://www.reddit.com/r/openrouter/) restricts promotion and links to one's own projects to the monthly thread; rule 8 forbids reposts. | One comment was published and subsequently found removed. Placement stopped; do not repost. Do not put promotional replies into unrelated support discussions. |
| Official OpenRouter Discord | [Official support page](https://openrouter.ai/support) links the [server](https://openrouter.ai/discord) and directs help requests to its help forum. Its public page does not establish permission to promote in a particular channel. | Second candidate after reading the actual server rules and finding a channel explicitly allowing projects/resources. No assumed channel name; no promotion in help merely because it is visible. |
| Rust Users Forum | [Guidelines](https://users.rust-lang.org/guidelines) ask for relevant discussion and prohibit spam/cross-posting. In [September's moderator discussion](https://users.rust-lang.org/t/announcement-topic-category/142378), announcements were muted by default because of low engagement. A related [agent-harness announcement](https://users.rust-lang.org/t/i-built-a-rust-learning-project-for-implementing-a-claude-code-style-agent-harness-from-scratch/139834) is closed after 90 days. Search found no current OpenRouter/tool-calling help question suited to a direct reply. | Defer. Do not revive old threads or disguise an announcement as a request for help. Only answer a live, relevant technical question when there is a concrete contribution. |
| r/rust | [Moderator announcement](https://www.reddit.com/r/rust/comments/1wkmzun/no_more_code_dumps/) and [current sidebar](https://www.reddit.com/r/rust/) reject code dumps and AI-generated text/articles, even in part. | Exclude these AI-assisted drafts/articles. Do not rewrite them merely to evade that rule. |

## Published placement: r/OpenRouter monthly thread

Target: https://www.reddit.com/r/openrouter/comments/1wuvfzj/monthly_megathread_what_are_you_working_on_with/

Original published comment, retained verbatim for the record (public body now removed):

> I maintain openrouter-rs. I fixed a misleading success case in its typed-tool example: an empty answer or an exhausted four-request budget could still exit successfully.
>
> The updated example fails those paths, checks tool arguments with Serde plus service-specific validation, and returns each tool result with the original call ID. It also keeps the full modeled assistant message, including reasoning details, for the next request.
>
> I wrote up the runnable example and the failure cases here:
> https://realmorrisliu.com/thoughts/typed-tools-openrouter-rust-agent/?utm_source=reddit&utm_medium=community&utm_campaign=openrouter-rs-2026-10&utm_content=openrouter-monthly-typed-tools
>
> The October 13 live smoke run with DeepSeek V4.1 Flash completed two model requests, both tool results, and a final answer. The tools read fixed demo data; they don't access real deployment systems. The model and reasoning settings are configurable.
>
> If you're writing a Rust tool loop, I'd be interested in the failure cases your harness checks before accepting a final answer.

The original copy makes no speed, savings, production-readiness, or competitor claims. On editorial review, it reads too much like a validation report: several implementation details, a full smoke-run summary, and a generic invitation to discuss failure cases. Do not reuse this wording as a posting template. Removal alone does not establish that writing style caused moderation. Do not ask for stars, votes, downloads, or repeat installs.

## Revised draft: OpenRouter Discord

Draft only, rewritten October 15 for the user's review. Use only in a channel that explicitly permits project/resource sharing, after checking its current rules. No publication is authorized by this revision.

> I maintain openrouter-rs. I've been working on a Rust example where a model uses two tools to look into a simulated checkout incident.
>
> While writing it up, I changed how the loop finishes. It used to exit successfully even when the model returned no answer or ran out of turns. It now returns an error in both cases.
>
> The walkthrough and complete example are here:
> https://realmorrisliu.com/thoughts/typed-tools-openrouter-rust-agent/?utm_source=discord&utm_medium=community&utm_campaign=openrouter-rs-2026-10&utm_content=openrouter-discord-typed-tools
>
> I tested the round trip with DeepSeek V4.1 Flash. It makes real OpenRouter requests, but the tools only read fixed demo data.

The last paragraph preserves the boundary of the live verification without turning the post into a test report. The source of that claim is the October 13 run linked in the article; this editorial revision did not rerun it. The shorter wording does not imply that every model/configuration has been tested.

If the channel does not permit this message, skip it. Do not mass-message members, tag moderators, or send unsolicited DMs.

## Revised draft: reply to a relevant technical question

Draft only. No current discussion has been selected. This wording fits a question about a loop exiting successfully without a final answer; adapt it to the actual code and question before any use. It is not a standalone announcement or a Reddit repost.

> I'd check the path after your last allowed model request. If it returns `Ok(())`, the program can report success without ever getting an answer.
>
> In openrouter-rs, which I maintain, I changed the typed-tool example so an empty final answer or an exhausted turn limit returns an error. Invalid tool arguments go back to the model as a tool result, giving it a chance to retry within the same limit.

This reply deliberately has no article link. The useful answer is already in the comment; add a direct source/example link only if the actual question needs it and local rules allow it. An eventual linked reply needs its own placement tag and ledger entry; do not reuse the Discord tag.

Answer in place only when there is a concrete contribution to a live question. Disclose SDK maintenance when recommending it. No generic prepared reply should be pasted into unrelated discussions. The removed Reddit placement remains stopped; Rust Users Forum remains deferred, and r/rust remains excluded as recorded above.

## Measurement

Use existing PostHog `$pageview` and `$autocapture`; no new production event or SDK code is needed. Each actual placement gets a unique `utm_content`, a permanent publication URL, and an actual publication timestamp. The prepared links above are reserved for exactly those placements.

1. Immediately before publishing, record the target article's current baseline, verify the link resolves with all four UTM parameters intact, and verify the known QA-device exclusion. If a different browser is used for QA, exclude it before evaluating results. Avoid clicking the campaign link from the maintainer's own post.
2. Count landing pageviews only when the **current URL** has the exact campaign's source, medium, campaign, and content parameters, with host exactly `realmorrisliu.com` and either slash form of the exact article path. Persisted UTM properties alone do not establish a landing.
3. Report pageviews, distinct browser IDs, and distinct sessions separately. They are not unique people. Treat a tagged landing as a measured visit, not a proven human: previews/crawlers or a shared link may generate traffic. Check available referrer/session evidence before calling a visit qualified; do not publish visitor identifiers.
4. For downstream clicks, require `$autocapture`, `$event_type = click`, the article path, and an external URL equal to `https://github.com/realmorrisliu/openrouter-rs` or beginning with that exact repository plus `/` or `#`. To attribute that click to a placement, require the same `distinct_id` and nonempty session ID as a matching tagged landing, with the click after the landing in that session. Exclude maintainer/QA traffic on both sides. Report direct source-file/repository clicks separately from Actions-run evidence lookups.
5. Look for substantive replies that discuss running, adapting, or debugging the example. Preserve their public URLs. A star, post view, download request, or self-authored thread reply does not prove adoption.
6. Observe at 24 hours, 48 hours, and seven days after **actual publication**, using the same elapsed window for channel comparisons. Start with one placement; a second placement must have its own tag and time. These are small observational experiments, not randomized A/B tests.

The October 15 pre-placement observation found 0 typed-tools article pageviews and 0 GitHub clicks after excluding the known QA browser. Axum had 2 pageviews and 0 GitHub clicks since October 13; remaining visits could include the maintainer. Re-query immediately before posting rather than treating these numbers as a permanent baseline.

| Placement | Published URL / time (Asia/Shanghai) | Elapsed window | Tagged landings / browsers / sessions | Same-session GitHub clicks | Qualified replies | Decision |
| --- | --- | --- | --- | --- | --- | --- |
| Reddit monthly thread | [pgujbmz](https://www.reddit.com/r/openrouter/comments/1wuvfzj/comment/pgujbmz/), publication confirmed by October 15 14:24:30 | Public removal reconfirmed October 15 14:33 | Baseline 0; post-publication not measured | Baseline 0; post-publication not measured | Not measured | Stopped; public body removed, automation paused; no repost |
| Discord | Not published | Not started | Not observed | Not observed | Not observed | Verify actual channel and rules first |

Interpretation: no tagged landings after 48 hours means the placement has not demonstrated measured website traffic; it does not prove zero readers or broken tracking. Landings without downstream clicks suggest reviewing the entry paragraph and runnable instructions. At least one non-QA, plausibly human same-session GitHub click is an initial conversion signal, not proof of installation. Treat independent run/adaptation feedback as stronger evidence. If neither placement shows qualified traffic after seven days, revisit audience/problem fit rather than repeating the same message in more communities.

## Before publication

The user authorized a conservative placement on October 15 after reviewing the drafts and risks. That authorization was used for the single Reddit monthly-thread comment recorded above. Keep Discord and further placements as drafts. Do not repost a removed or filtered comment, mass-message members, or automate engagement. No additional posting is part of the measurement follow-up.


## Voice for future campaign content

The user explicitly requested on October 15 that future operational content avoid an AI-like voice. Apply this to articles, X posts, community comments, and replies in this campaign.

- Before drafting, read relevant existing posts by the author and the actual discussion. Match the author's vocabulary, paragraph rhythm, and degree of informality. Do not assume that generic casual English is their voice.
- Choose one concrete point useful to that audience. Explain the actual problem and the change in ordinary language. A community comment should make sense without opening the article; it need not compress every technical detail into the post.
- Keep verification evidence available in the article or linked source. In the post, include only the result and limitations needed to assess its claim. Preserve material limitations such as fixture-backed tools; never turn a demo into a production claim.
- Remove staged hooks, inflated significance, repeated conclusions, forced three-part lists, emoji decoration, and generic engagement endings. Ask a question only when there is a specific answer the author actually needs.
- Use first-person experiences, feelings, dates, and outcomes only when supported by the author's account or documented work. Do not invent late-night debugging, customer stories, excitement, slang, or deliberate mistakes to simulate a person.
- Review facts separately from prose: first verify the claims, then read the draft aloud and remove sentences that add nothing. Prefer a shorter useful post when there is only one point to make. No fixed word count or mechanical punctuation ban.
- Follow each community's current rules, including AI-content restrictions. Editorial cleanup does not change the provenance of AI-assisted content or permit evading those restrictions. A removed post stays stopped unless the user authorizes a next step.

Research references inspected October 15 (read only; no third-party skill installed):

| Reference | Useful parts | Limits for this campaign |
| --- | --- | --- |
| [coreyhaines31/marketingskills: social](https://github.com/coreyhaines31/marketingskills/blob/main/skills/social/SKILL.md) | Audience and voice context; platform-specific adaptation; concrete standalone posts; removing engagement bait and manufactured vulnerability. | Do not adopt generic posting frequencies, unsolicited DM routines, hook templates, or reach claims as evidence-backed instructions for this account. |
| [blader/humanizer](https://github.com/blader/humanizer/blob/main/SKILL.md) | Preserve supported facts; use the author's writing samples; remove staging, repetition, inflation, and excessive formatting. | An editorial reference, not a detector-evasion tool or proof of human authorship. Do not add reactions or experiences on the author's behalf. |
| [kostja94/marketing-skills: reddit-posts](https://github.com/kostja94/marketing-skills/blob/main/skills/platforms/reddit/SKILL.md) | Specific accurate titles, local rule/flair checks, helpful context, transparent affiliation. | Its fixed promotional ratios, recommended karma range, and ranking assertions are not established here as Reddit rules or reliable thresholds. Do not farm karma or follow posting quotas. |
| [resciencelab/opc-skills: reddit](https://github.com/resciencelab/opc-skills/blob/main/skills/reddit/SKILL.md) | Retrieval/search capability reference. | An API utility rather than an editorial or moderation playbook. No scripts were executed and its access/rate-limit claims were not validated. |

These references inform editing choices. The user's voice, verified facts, current community rules, and explicit publication authorization govern the actual work.
