import importlib.util
import json
from datetime import datetime, timezone
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from urllib.error import HTTPError, URLError


SPEC = importlib.util.spec_from_file_location(
    "adoption", Path(__file__).resolve().parents[1] / "scripts/collect_adoption_metrics.py"
)
adoption = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(adoption)
NOW = datetime(2026, 10, 13, 2, 23, tzinfo=timezone.utc)


def daily_payload():
    return {
        "version_downloads": [
            {"version": 1, "date": "2026-10-13", "downloads": 999},
            {"version": 1, "date": "2026-10-12", "downloads": 10},
            {"version": 2, "date": "2026-10-12", "downloads": 5},
            {"version": 1, "date": "2026-10-06", "downloads": 3},
            {"version": 1, "date": "2026-10-05", "downloads": 2},
            {"version": 1, "date": "2026-09-13", "downloads": 4},
            {"version": 1, "date": "2026-09-12", "downloads": 500},
        ],
        "meta": {"extra_downloads": [{"date": "2026-10-12", "downloads": 7}]},
    }


def fixture_fetch(path):
    if path.startswith("/crates?"):
        return {"meta": {"total": 2}, "crates": [{"id": name} for name in adoption.CRATES]}
    if path.endswith("/downloads"):
        return daily_payload()
    if "reverse_dependencies" in path:
        return {"meta": {"total": 4}}
    return {"crate": {"id": path.split("/")[-1], "downloads": 1000, "recent_downloads": 100}}


class AdoptionMetricsTests(unittest.TestCase):
    def test_all_versions_and_complete_window_boundaries(self):
        self.assertEqual(adoption.download_totals(daily_payload(), NOW.date()),
                         {"downloads_7d": 25, "downloads_30d": 31})

    def test_keyword_pagination(self):
        paths = []

        def fetch(path):
            paths.append(path)
            names = [f"crate-{i}" for i in range(100)] if path.endswith("&page=1") else ["openrouter-rs"]
            return {"meta": {"total": 101}, "crates": [{"id": name} for name in names]}

        self.assertEqual(adoption.keyword_ranks(fetch)["openrouter-rs"], 101)
        self.assertEqual(len(paths), 2)

    def test_repeated_or_incomplete_keyword_page_fails(self):
        for rows in ([], [{"id": "repeat"}, {"id": "repeat"}]):
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                adoption.keyword_ranks(lambda _: {"meta": {"total": 2}, "crates": rows})

    def test_collection_metadata_and_not_listed_rank(self):
        def fetch(path):
            if path.startswith("/crates?"):
                return {"meta": {"total": 0}, "crates": []}
            return fixture_fetch(path)

        result = adoption.collect(fetch, NOW)
        self.assertEqual(result["complete_days_through"], "2026-10-12")
        self.assertIsNone(result["crates"]["openrouter-rs"]["keyword_rank"])
        self.assertEqual(result["crates"]["rai-sdk"]["published_reverse_dependencies"], 4)

    def test_missing_or_invalid_metrics_fail_instead_of_becoming_zero(self):
        for value in (None, -1, "100", True):
            with self.subTest(value=value), self.assertRaises(ValueError):
                adoption.count(value)
        payload = daily_payload()
        del payload["meta"]["extra_downloads"]
        with self.assertRaises(KeyError):
            adoption.download_totals(payload, NOW.date())
        payload = daily_payload()
        payload["version_downloads"].append(payload["version_downloads"][0])
        with self.assertRaises(ValueError):
            adoption.download_totals(payload, NOW.date())

    def test_same_date_replaces_one_snapshot(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = adoption.collect(fixture_fetch, NOW)
            output = adoption.save_snapshot(Path(tmp), result)
            result["crates"]["openrouter-rs"]["downloads_recent"] = 101
            adoption.save_snapshot(Path(tmp), result)
            self.assertEqual(list(Path(tmp).iterdir()), [output])
            self.assertEqual(json.loads(output.read_text()), result)

    def test_failed_collection_preserves_existing_snapshot(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = adoption.save_snapshot(Path(tmp), adoption.collect(fixture_fetch, NOW))
            original = output.read_text()
            with patch.object(adoption, "fetch_json", side_effect=URLError("offline")), \
                 patch("sys.argv", ["collect", "--output-dir", tmp]):
                self.assertEqual(adoption.main(), 1)
            self.assertEqual(output.read_text(), original)

    def test_network_retries_are_bounded_and_client_is_identified(self):
        with patch.object(adoption, "urlopen", side_effect=URLError("offline")) as request, \
             patch.object(adoption.time, "sleep"):
            with self.assertRaises(URLError):
                adoption.fetch_json("/crates/openrouter-rs")
            self.assertEqual(request.call_count, 3)
            self.assertIn("openrouter-rs", request.call_args.args[0].get_header("User-agent"))

    def test_nontransient_error_is_not_retried(self):
        error = HTTPError("url", 404, "not found", {}, None)
        with patch.object(adoption, "urlopen", side_effect=error) as request, \
             patch.object(adoption.time, "sleep"):
            with self.assertRaises(HTTPError):
                adoption.fetch_json("/crates/missing")
            self.assertEqual(request.call_count, 1)


if __name__ == "__main__":
    unittest.main()
