import importlib.util
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = REPO_ROOT / "scripts" / "openapi_drift.py"
SPEC = importlib.util.spec_from_file_location("openapi_drift", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
openapi_drift = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(openapi_drift)


def build_spec(
    *,
    method="get",
    path="/models",
    operation_id="getModels",
    parameters=None,
    request_properties=None,
    response_properties=None,
):
    operation = {
        "operationId": operation_id,
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
    if request_properties is not None:
        operation["requestBody"] = {
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": request_properties,
                    }
                }
            }
        }
    if parameters is not None:
        operation["parameters"] = parameters

    return {
        "openapi": "3.1.0",
        "info": {"title": "Test", "version": "1.0.0"},
        "paths": {
            path: {
                method: operation,
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
                    "x-speakeasy-name-override": "appTitle",
                },
                {
                    "in": "header",
                    "name": "X-Title",
                    "schema": {"type": "string"},
                },
                {
                    "in": "header",
                    "name": "X-OpenRouter-Categories",
                    "schema": {"type": "string"},
                    "x-speakeasy-name-override": "appCategories",
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(
            report["changed"][0]["repo_impact"]["category"],
            "already_supported",
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")

    def test_required_metadata_header_change_remains_actionable(self):
        baseline = build_spec(
            parameters=[
                {
                    "in": "header",
                    "name": "X-OpenRouter-Title",
                    "schema": {"type": "string"},
                    "x-speakeasy-name-override": "appTitle",
                }
            ]
        )
        candidate = build_spec(
            parameters=[
                {
                    "in": "header",
                    "name": "X-OpenRouter-Title",
                    "required": True,
                    "schema": {"type": "string"},
                    "x-speakeasy-name-override": "appTitle",
                }
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
        self.assertTrue(report["has_actionable_drift"])
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["supported_parameters"],
            ["header X-OpenRouter-Title"],
        )

    def test_dynamic_provider_enum_drift_is_classified_as_already_supported(self):
        baseline = build_spec(
            response_properties={
                "provider_name": {
                    "enum": ["Anthropic", "Google", "OpenAI"],
                    "type": "string",
                    "x-speakeasy-unknown-values": "allow",
                }
            }
        )
        candidate = build_spec(
            response_properties={
                "provider_name": {
                    "enum": ["Anthropic", "Google", "Nex AGI", "OpenAI"],
                    "type": "string",
                    "x-speakeasy-unknown-values": "allow",
                }
            }
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["dynamic provider name enum"],
        )

    def test_dynamic_enum_with_object_members_does_not_crash(self):
        baseline = build_spec(
            response_properties={
                "provider_name": {
                    "enum": ["Anthropic", "Google", "OpenAI", {"custom": True}],
                    "type": "string",
                    "x-speakeasy-unknown-values": "allow",
                }
            }
        )
        candidate = build_spec(
            response_properties={
                "provider_name": {
                    "enum": ["Anthropic", "Google", "Nex AGI", "OpenAI", {"custom": True}],
                    "type": "string",
                    "x-speakeasy-unknown-values": "allow",
                }
            }
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")

    def test_dynamic_modality_enum_drift_is_classified_as_already_supported(self):
        baseline = build_spec(
            response_properties={
                "output_modalities": {
                    "items": {
                        "enum": ["audio", "image", "text", "tts", "video"],
                        "type": "string",
                        "x-speakeasy-unknown-values": "allow",
                    },
                    "type": "array",
                }
            }
        )
        candidate = build_spec(
            response_properties={
                "output_modalities": {
                    "items": {
                        "enum": ["audio", "image", "speech", "text", "video"],
                        "type": "string",
                        "x-speakeasy-unknown-values": "allow",
                    },
                    "type": "array",
                }
            }
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["dynamic output modality enum"],
        )

    def test_provider_options_map_drift_is_classified_as_already_supported(self):
        provider_option = {
            "additionalProperties": {"nullable": True},
            "type": "object",
        }
        baseline = build_spec(
            response_properties={
                "provider": {
                    "properties": {
                        "anthropic": provider_option,
                        "google-vertex": provider_option,
                        "openai": provider_option,
                    },
                    "type": "object",
                }
            }
        )
        candidate = build_spec(
            response_properties={
                "provider": {
                    "properties": {
                        "anthropic": provider_option,
                        "google-vertex": provider_option,
                        "nex-agi": provider_option,
                        "openai": provider_option,
                    },
                    "type": "object",
                }
            }
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["provider-specific options map"],
        )

    def test_responses_nullable_drift_is_classified_as_already_supported(self):
        baseline = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            response_properties={
                "top_logprobs": {"nullable": True, "type": "integer"},
            },
        )
        candidate = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            response_properties={
                "top_logprobs": {"type": "integer"},
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
        self.assertFalse(report["has_actionable_drift"])
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["Responses flexible nullable fields"],
        )

    def test_non_responses_nullable_drift_remains_actionable(self):
        baseline = build_spec(
            response_properties={
                "top_logprobs": {"nullable": True, "type": "integer"},
            },
        )
        candidate = build_spec(
            response_properties={
                "top_logprobs": {"type": "integer"},
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")

    def test_responses_request_nullable_drift_remains_actionable(self):
        baseline = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            request_properties={
                "top_logprobs": {"nullable": True, "type": "integer"},
            },
        )
        candidate = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            request_properties={
                "top_logprobs": {"type": "integer"},
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")

    def test_flexible_plugin_property_drift_is_classified_as_already_supported(self):
        baseline = build_spec(
            method="post",
            path="/chat/completions",
            operation_id="sendChatCompletionRequest",
            request_properties={
                "plugins": {
                    "items": {
                        "oneOf": [
                            {
                                "properties": {
                                    "id": {"enum": ["web"], "type": "string"},
                                    "max_results": {"type": "integer"},
                                },
                                "required": ["id"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
        )
        candidate = build_spec(
            method="post",
            path="/chat/completions",
            operation_id="sendChatCompletionRequest",
            request_properties={
                "plugins": {
                    "items": {
                        "oneOf": [
                            {
                                "properties": {
                                    "id": {"enum": ["web"], "type": "string"},
                                    "max_results": {"type": "integer"},
                                    "max_uses": {"type": "integer"},
                                },
                                "required": ["id"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
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
        self.assertFalse(report["has_actionable_drift"])
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["flexible plugin payload"],
        )

    def test_flexible_plugin_container_type_change_remains_actionable(self):
        baseline = build_spec(
            method="post",
            path="/chat/completions",
            operation_id="sendChatCompletionRequest",
            request_properties={
                "plugins": {
                    "items": {
                        "oneOf": [
                            {
                                "properties": {
                                    "id": {"enum": ["web"], "type": "string"},
                                },
                                "required": ["id"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
        )
        candidate = build_spec(
            method="post",
            path="/chat/completions",
            operation_id="sendChatCompletionRequest",
            request_properties={
                "plugins": {
                    "properties": {
                        "id": {"enum": ["web"], "type": "string"},
                    },
                    "type": "object",
                }
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["flexible plugin payload"],
        )

    def test_responses_value_tool_and_output_drift_is_classified_as_already_supported(self):
        baseline = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            request_properties={
                "tools": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "name": {"type": "string"},
                                    "type": {"enum": ["function"], "type": "string"},
                                },
                                "required": ["type", "name"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
            response_properties={
                "output": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "type": {"enum": ["message"], "type": "string"},
                                },
                                "required": ["type"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
        )
        candidate = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            request_properties={
                "tools": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "name": {"type": "string"},
                                    "type": {"enum": ["function"], "type": "string"},
                                },
                                "required": ["type", "name"],
                                "type": "object",
                            },
                            {
                                "properties": {
                                    "type": {"enum": ["web_search_preview"], "type": "string"},
                                    "max_results": {"type": "integer"},
                                },
                                "required": ["type"],
                                "type": "object",
                            },
                        ]
                    },
                    "type": "array",
                }
            },
            response_properties={
                "output": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "type": {"enum": ["message"], "type": "string"},
                                },
                                "required": ["type"],
                                "type": "object",
                            },
                            {
                                "properties": {
                                    "type": {"enum": ["code_interpreter_call"], "type": "string"},
                                    "outputs": {
                                        "items": {"type": "object"},
                                        "type": "array",
                                    },
                                },
                                "required": ["type"],
                                "type": "object",
                            },
                        ]
                    },
                    "type": "array",
                }
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
        self.assertFalse(report["has_actionable_drift"])
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["Responses flexible output payload", "Responses flexible tool payload"],
        )

    def test_responses_flexible_container_type_changes_remain_actionable(self):
        baseline = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            request_properties={
                "tools": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "name": {"type": "string"},
                                    "type": {"enum": ["function"], "type": "string"},
                                },
                                "required": ["type", "name"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
            response_properties={
                "output": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "type": {"enum": ["message"], "type": "string"},
                                },
                                "required": ["type"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
        )
        candidate = build_spec(
            method="post",
            path="/responses",
            operation_id="createResponses",
            request_properties={
                "tools": {
                    "properties": {
                        "type": {"enum": ["function"], "type": "string"},
                    },
                    "type": "object",
                }
            },
            response_properties={
                "output": {
                    "properties": {
                        "type": {"enum": ["message"], "type": "string"},
                    },
                    "type": "object",
                }
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["Responses flexible output payload", "Responses flexible tool payload"],
        )

    def test_messages_flexible_tool_option_drift_is_classified_as_already_supported(self):
        baseline = build_spec(
            method="post",
            path="/messages",
            operation_id="createMessages",
            request_properties={
                "tools": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "name": {"enum": ["web_search"], "type": "string"},
                                    "type": {
                                        "enum": ["web_search_20250305"],
                                        "type": "string",
                                    },
                                },
                                "required": ["type", "name"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
        )
        candidate = build_spec(
            method="post",
            path="/messages",
            operation_id="createMessages",
            request_properties={
                "tools": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "max_uses": {"nullable": True, "type": "integer"},
                                    "name": {"enum": ["web_search"], "type": "string"},
                                    "type": {
                                        "enum": ["web_search_20250305"],
                                        "type": "string",
                                    },
                                    "user_location": {
                                        "properties": {
                                            "type": {
                                                "enum": ["approximate"],
                                                "type": "string",
                                            }
                                        },
                                        "type": "object",
                                    },
                                },
                                "required": ["type", "name"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
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
        self.assertFalse(report["has_actionable_drift"])
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 1)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 0)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "already_supported")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["Messages flexible tool payload"],
        )

    def test_messages_flexible_tool_container_type_change_remains_actionable(self):
        baseline = build_spec(
            method="post",
            path="/messages",
            operation_id="createMessages",
            request_properties={
                "tools": {
                    "items": {
                        "anyOf": [
                            {
                                "properties": {
                                    "name": {"enum": ["web_search"], "type": "string"},
                                    "type": {
                                        "enum": ["web_search_20250305"],
                                        "type": "string",
                                    },
                                },
                                "required": ["type", "name"],
                                "type": "object",
                            }
                        ]
                    },
                    "type": "array",
                }
            },
        )
        candidate = build_spec(
            method="post",
            path="/messages",
            operation_id="createMessages",
            request_properties={
                "tools": {
                    "properties": {
                        "name": {"enum": ["web_search"], "type": "string"},
                        "type": {
                            "enum": ["web_search_20250305"],
                            "type": "string",
                        },
                    },
                    "type": "object",
                }
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
        self.assertEqual(report["repo_summary"]["already_supported_changed"], 0)
        self.assertEqual(report["repo_summary"]["actionable_changed"], 1)
        self.assertEqual(report["changed"][0]["repo_impact"]["category"], "actionable")
        self.assertEqual(
            report["changed"][0]["repo_impact"]["schema_rules"],
            ["Messages flexible tool payload"],
        )


if __name__ == "__main__":
    unittest.main()
