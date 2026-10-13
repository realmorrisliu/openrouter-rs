#!/usr/bin/env python3
"""Capture public crates.io metrics; downloads are not unique users."""

import argparse
from collections import Counter
from datetime import date, datetime, timedelta, timezone
import json
import os
from pathlib import Path
import sys
import tempfile
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


API = "https://crates.io/api/v1"
CRATES = ("openrouter-rs", "rai-sdk")
USER_AGENT = "openrouter-rs-adoption-metrics (https://github.com/realmorrisliu/openrouter-rs)"


def fetch_json(path):
    request = Request(API + path, headers={"User-Agent": USER_AGENT, "Accept": "application/json"})
    for attempt in range(3):
        # Keep requests sequential and below one request per second.
        time.sleep(1 + attempt * 2)
        try:
            with urlopen(request, timeout=30) as response:
                return json.load(response)
        except HTTPError as error:
            if error.code not in (429, 500, 502, 503, 504) or attempt == 2:
                raise
            # Do not retry earlier than a numeric Retry-After permits.
            retry_after = error.headers.get("Retry-After")
            if retry_after:
                if not retry_after.isdigit() or int(retry_after) > 60:
                    raise
                time.sleep(int(retry_after))
        except (URLError, TimeoutError):
            if attempt == 2:
                raise


def count(value):
    if type(value) is not int or value < 0:
        raise ValueError(f"Expected a nonnegative integer, got {value!r}")
    return value


def keyword_ranks(fetch):
    ranks = {}
    page = 1
    total = None
    while True:
        payload = fetch(f"/crates?keyword=openrouter&sort=recent-downloads&per_page=100&page={page}")
        current_total = count(payload["meta"]["total"])
        if total is not None and current_total != total:
            raise ValueError("Keyword membership changed during pagination; retry the collection")
        total = current_total
        rows = payload["crates"]
        if not isinstance(rows, list) or (not rows and len(ranks) < total):
            raise ValueError("Incomplete keyword page")
        for row in rows:
            name = row["id"]
            if not isinstance(name, str) or not name or name in ranks:
                raise ValueError("Invalid or repeated crate in keyword pages")
            ranks[name] = len(ranks) + 1
        if len(ranks) == total:
            return ranks
        if len(ranks) > total:
            raise ValueError("Keyword pagination exceeded reported total")
        page += 1


def download_totals(payload, today):
    daily = Counter()
    seen = set()
    for group, rows in (
        ("version", payload["version_downloads"]),
        ("extra", payload["meta"]["extra_downloads"]),
    ):
        if not isinstance(rows, list):
            raise ValueError("Download rows must be lists")
        for row in rows:
            day = date.fromisoformat(row["date"])
            key = (group, count(row["version"]) if group == "version" else None, day)
            if key in seen:
                raise ValueError("Repeated daily download row")
            seen.add(key)
            daily[day] += count(row["downloads"])
    return {
        f"downloads_{days}d": sum(
            value for day, value in daily.items() if today - timedelta(days=days) <= day < today
        )
        for days in (7, 30)
    }


def collect(fetch, now):
    today = now.astimezone(timezone.utc).date()
    ranks = keyword_ranks(fetch)
    metrics = {}
    for name in CRATES:
        crate = fetch(f"/crates/{name}")["crate"]
        if crate["id"] != name:
            raise ValueError("Crate response does not match requested name")
        totals = download_totals(fetch(f"/crates/{name}/downloads"), today)
        dependents = fetch(f"/crates/{name}/reverse_dependencies?page=1&per_page=1")
        metrics[name] = {
            "downloads_all_time": count(crate["downloads"]),
            "downloads_recent": count(crate["recent_downloads"]),
            **totals,
            "keyword_rank": ranks.get(name),
            "published_reverse_dependencies": count(dependents["meta"]["total"]),
        }
    return {
        "schema_version": 1,
        "collected_at": now.astimezone(timezone.utc).isoformat(),
        "snapshot_date": today.isoformat(),
        "complete_days_through": (today - timedelta(days=1)).isoformat(),
        "keyword": "openrouter",
        "keyword_crate_count": len(ranks),
        "source": API,
        "crates": metrics,
    }


def save_snapshot(directory, snapshot):
    directory.mkdir(parents=True, exist_ok=True)
    destination = directory / f"{snapshot['snapshot_date']}.json"
    # A successful same-day rerun replaces the snapshot atomically.
    with tempfile.NamedTemporaryFile(mode="w", dir=directory, suffix=".tmp", delete=False) as handle:
        temporary = Path(handle.name)
        try:
            json.dump(snapshot, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.close()
            temporary.replace(destination)
        finally:
            temporary.unlink(missing_ok=True)
    return destination


def summary(snapshot):
    rows = [
        f"## Adoption snapshot: {snapshot['snapshot_date']} (UTC)",
        "",
        f"Complete daily totals through {snapshot['complete_days_through']}; "
        f"{snapshot['keyword_crate_count']} crates in the OpenRouter keyword list.",
        "",
        "| Crate | Rank | Recent (90d) | Complete 7d | Complete 30d | All time | Published dependents |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for name, values in snapshot["crates"].items():
        fields = ("keyword_rank", "downloads_recent", "downloads_7d", "downloads_30d",
                  "downloads_all_time", "published_reverse_dependencies")
        cells = [str(values[field]) if values[field] is not None else "not listed" for field in fields]
        rows.append(f"| {name} | " + " | ".join(cells) + " |")
    rows.extend(["", "Downloads are requests, not unique users. Recent totals and daily counts may refresh "
                 "at different times. Dependents include the project's own CLI; unpublished apps are absent."])
    return "\n".join(rows) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    try:
        snapshot = collect(fetch_json, datetime.now(timezone.utc))
        output = save_snapshot(args.output_dir, snapshot)
        report = summary(snapshot)
        print(report)
        print(f"Snapshot: {output}")
        if os.environ.get("GITHUB_STEP_SUMMARY"):
            with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as handle:
                handle.write(report)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Adoption collection failed; no missing metric is recorded as zero: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
