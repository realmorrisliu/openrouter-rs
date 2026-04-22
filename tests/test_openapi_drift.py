import importlib.util
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = REPO_ROOT / "scripts" / "openapi_drift.py"
SPEC = importlib.util.spec_from_file_location("openapi_drift", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
openapi_drift = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(openapi_drift)


def build_spec(*, parameters=None, response_properties=None):
    operation = {
        "operationId": "getModels",
        "responses": {
            "200": {
                "description": "ok",
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object",
                            "properties": response_properties or {
                                "id": {"type": "string"},
                            },
                        }
                    }
                },
            }
        },
    }
    if parameters is not None:
        operation["parameters"] = parameters

    return {
        "openapi": "3.1.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {
            "/models": {
                "get": operation,
            }
        },
    }


class OpenApiDriftReportTests(unittest.TestCase):
    def test_metadata_only_header_drift_is_classified_as_already_supported(self):
        baseline = build_spec()
        candidate = build_spec(
            parameters=[
                {
                    "in": "header",
                    "name": "HTTP-Referer",
                    "schema": {"type": "string"},
                },
                {
                    "in": "header",
                    "name": "X-OpenRouter-Title",
                    "schema": {"type": "string"},
                },
                {
                    "in": "header",
                    "name": "X-OpenRouter-Categories",
                    "schema": {"type": "string"},
                },
            ]
        )

        report = openapi_drift.build_report(
            baseline_spec=baseline,
            candidate_spec=candidate,
            baseline_label="baseline",
            candidate_label="candidate",
            source_url="https://example.com/openapi.json",
            max_diff_lines=20,
        )

        self.assertTrue(report["has_drift"])
        self.assertFalse(report["has_actionable_drift"])
        self.assertEqual(report["summary"]["changed"], 1)
        self.assertEqual(report["repo_summary"]["metadata_only_already_supported"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(
            report["changed"][0]["repo_impact"]["category"],
            "metadata_only_already_supported",
        )

    def test_non_metadata_schema_change_remains_actionable(self):
        baseline = build_spec(
            parameters=[
                {
                    "in": "header",
                    "name": "HTTP-Referer",
                    "schema": {"type": "string"},
                }
            ]
        )
        candidate = build_spec(
            parameters=[
                {
                    "in": "header",
                    "name": "HTTP-Referer",
                    "schema": {"type": "string"},
                }
            ],
            response_properties={
                "id": {"type": "string"},
                "num_fetches": {"type": "integer", "nullable": True},
            },
        )

        report = openapi_drift.build_report(
            baseline_spec=baseline,
            candidate_spec=candidate,
            baseline_label="baseline",
            candidate_label="candidate",
            source_url="https://example.com/openapi.json",
            max_diff_lines=20,
        )

        self.assertTrue(report["has_drift"])
        self.assertTrue(report["has_actionable_drift"])
        self.assertEqual(report["summary"]["changed"], 1)
        self.assertEqual(report["repo_summary"]["metadata_only_already_supported"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")


if __name__ == "__main__":
    unittest.main()
