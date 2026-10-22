# Adoption Metrics

The [Adoption Metrics workflow](../../.github/workflows/adoption-metrics.yml) records public crates.io statistics for `openrouter-rs` and `rai-sdk`. It runs daily at 02:23 UTC (10:23 Asia/Shanghai), supports manual dispatch, and runs on collector-related PRs as a live smoke check. GitHub schedules run from the default branch after merge and may be delayed.

## Collect Locally

Requires Python 3.10+ and network access; no credentials or third-party Python packages are needed.

```bash
just test-adoption-metrics
python3 scripts/collect_adoption_metrics.py --output-dir /tmp/openrouter-adoption
```

The collector writes `<UTC-date>.json` only after all requests and validation succeed. Repeating a collection into the same directory replaces that date atomically. Collection failure returns a nonzero status, leaves any prior snapshot intact, and does not substitute zero for missing metrics. A valid empty download history has zero downloads. An absent crate in a successfully fetched keyword list has a `null` rank, meaning “not listed.”

Requests are sequential, identify the project in User-Agent, and have a 30-second timeout and at most three attempts for transient failures. Missing or malformed data fails collection. Pagination must complete without duplicate crates or changing membership counts.

## Snapshot Fields

| Field | Meaning |
| --- | --- |
| `collected_at`, `snapshot_date` | UTC collection start and snapshot date |
| `complete_days_through` | Yesterday in UTC; today's partial download counts are excluded |
| `keyword_crate_count` | Number of entries fetched across all pages for keyword `openrouter` |
| `downloads_recent` | crates.io's reported recent-download total, used for its default keyword ordering (90 days) |
| `downloads_all_time` | Registry-reported lifetime count |
| `downloads_7d`, `downloads_30d` | Sums over complete UTC calendar dates, including both `version_downloads` and `meta.extra_downloads` for older versions |
| `keyword_rank` | One-based position in the API's `recent-downloads` ordering across every matching crate |
| `published_reverse_dependencies` | Registry-reported number of published dependents; includes the companion CLI |

Source endpoints: `/api/v1/crates?keyword=openrouter&sort=recent-downloads`, `/api/v1/crates/<name>`, `/api/v1/crates/<name>/downloads`, and `/api/v1/crates/<name>/reverse_dependencies` on crates.io.

Daily counts and headline totals can update at different times. “Complete day” describes the date boundary, not a guarantee that the registry has finished ingesting every request. Pagination and individual crate metrics are separate requests, not a transaction; a near tie can change between requests. Do not infer an inconsistent collector from that alone. A concurrent membership change that prevents complete pagination fails the run instead of recording an unreliable rank.

## Retrieve And Retain Evidence

Every successful run writes a table to the Actions job summary and uploads its JSON snapshot. Artifacts are named `adoption-metrics-<run-id>-<attempt>` and retained for 90 days, subject to repository retention limits. No daily source commit or issue is created.

```bash
gh run list --workflow adoption-metrics.yml --branch main --status success --limit 40
gh run view <run-id>
gh run download <run-id> --dir /tmp/openrouter-adoption-run-<run-id>
```

For an on-demand snapshot after the workflow has landed on main:

```bash
gh workflow run adoption-metrics.yml --ref main
```

For the weekly record, use successful main-branch runs, excluding PR smoke snapshots. If a UTC date has several runs, select the JSON with the latest `collected_at` for that date, not the sum of those snapshots. A failed day remains missing; it does not count as zero downloads or a confirmed first-place day. Download and archive evidence before artifact expiration if it is needed for a longer campaign. The four-week campaign fits within the configured retention.

## Weekly Review

Review on Tuesdays in Asia/Shanghai, using available completed UTC dates:

| Week ending | 7d downloads | 30d downloads | Recent downloads | Keyword rank | Published dependents | Verified independent adopters | Content/proposal URLs and next action |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |

Compare the same date window for both crates. Read daily ranks to assess sustained leadership; a missing date interrupts a claim of consecutive verified days. Record independent adoption from a merged/released integration or public maintainer confirmation, including application users that are not published crates. Keep these observations separate from the dependent count, which includes this project's CLI.

Website and X analytics can supplement the weekly record when available. They are not collected by this workflow. Downloads measure requests, not unique users or source-attributed installations; website clicks and command copies are interest signals, not proof of installation.

## Website And X Measurement Acceptance

The website already uses PostHog. Reuse its `$pageview` and `$autocapture` events before adding custom tracking: an article link click carries `$event_type = "click"`, `$external_click_url`, `$pathname`, and campaign properties. Do not sum an autocapture event and a custom event for the same click.

Accept the collection path only after both checks succeed:

1. In the browser, load the published article with the campaign parameters and click its runnable-source link. Confirm the script loads, the destination opens, and the collection request succeeds.
2. In the authenticated PostHog project's Activity view, find the resulting pageview and click. Check the article path, exact GitHub target, timestamp, and `utm_source`, `utm_medium`, `utm_campaign`, and `utm_content`. HTTP 200 from the collector alone does not prove an event is queryable.

For the Axum thread, the campaign is `utm_source=x&utm_medium=social&utm_campaign=openrouter-rs-2026-10&utm_content=axum-thread`. Accept both `/thoughts/streaming-openrouter-rust-axum` and its trailing-slash form, but filter the production host exactly. Count GitHub clicks only when the target is this SDK repository, a path inside it, or an anchor on it; clicks on API-run links are evidence lookups, not necessarily attempts to use the example.

UTM properties can persist between pages. For a thread landing-page count, also require the current URL to contain the matching campaign parameters; a persisted `utm_source = "x"` by itself is insufficient. The canonical article URL remains free of those parameters.

### Exclude Verification Traffic

Before reporting campaign results, configure PostHog's internal/test-user filters to exclude localhost and known QA browser device IDs. A controlled production click proves tracking works; it is not reader conversion. Keep the device IDs in the private analytics configuration, not in this repository. Verify the rule persists after a reload, then confirm a filtered query excludes the known QA click while retaining unrelated traffic. Check the filter on each existing insight; a default for new insights is not proof that older insights or raw SQL are filtered.

The saved PostHog insight **OpenRouter Axum: daily visits and GitHub clicks (QA excluded)** provides daily UTC counts of pageviews, distinct browser IDs, GitHub clicks, and X-tagged landing pageviews from the campaign's start. Its explicit QA exclusion must be updated if the verification browser changes. Distinct browser IDs are not unique people, and remaining visits may still include the author. Zero rows for a day do not by themselves prove the collector was working that day.

### Record Distribution Results

Read each post's available X analytics, especially the article-link post's impressions and link clicks. Keep unavailable fields as unavailable. Do not add the five posts' view counts and describe the sum as unique thread reach. Replies used to connect the author's own thread are not reader feedback.

Use the same observation window and record QA exclusions:

| Observed at (Asia/Shanghai) | Article pageviews / browser IDs | GitHub click events | X-tagged landing pageviews | X post URL | Impressions | X link clicks | Qualified feedback / next action |
| --- | --- | ---: | ---: | --- | ---: | ---: | --- |

Review after 48 hours and seven days. Website clicks, X link clicks, and crates.io downloads measure different actions and need not match. Preserve the existing collection events and fix query/filter errors before changing the website instrumentation. Record private dashboard references and screenshots locally; do not commit visitor identifiers or raw event payloads.

The [official ranking policy](https://rust-lang.github.io/rfcs/1824-crates.io-default-ranking.html) explains recent-download sorting. The [download accounting announcement](https://blog.rust-lang.org/2024/03/11/crates-io-download-changes/) explains CDN-based download statistics.
