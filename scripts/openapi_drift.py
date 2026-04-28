#!/usr/bin/env python3

from __future__ import annotations

import argparse
import datetime as dt
import difflib
import hashlib
import json
import sys
import urllib.request
from pathlib import Path
from typing import Any

UPSTREAM_OPENAPI_URL = "https://openrouter.ai/openapi.json"
HTTP_METHODS = ("get", "post", "put", "patch", "delete", "options", "head", "trace")
DOC_ONLY_FIELDS = {"description", "example", "examples", "externalDocs", "summary", "title"}
REPO_KNOWN_METADATA_PARAMETERS = frozenset(
    {
        ("header", "HTTP-Referer"),
        ("header", "X-Title"),
        ("header", "X-OpenRouter-Categories"),
        ("header", "X-OpenRouter-Title"),
    }
)
REPO_SUPPORTED_METADATA_PARAMETER_SHAPES = {
    ("header", "HTTP-Referer"): {
        "in": "header",
        "name": "HTTP-Referer",
        "schema": {
            "type": "string",
        },
    },
    ("header", "X-OpenRouter-Categories"): {
        "in": "header",
        "name": "X-OpenRouter-Categories",
        "schema": {
            "type": "string",
        },
        "x-speakeasy-name-override": "appCategories",
    },
    ("header", "X-Title"): {
        "in": "header",
        "name": "X-Title",
        "schema": {
            "type": "string",
        },
    },
    ("header", "X-OpenRouter-Title"): {
        "in": "header",
        "name": "X-OpenRouter-Title",
        "schema": {
            "type": "string",
        },
        "x-speakeasy-name-override": "appTitle",
    },
}
REPO_DYNAMIC_PROVIDER_NAME_MARKERS = frozenset({"Anthropic", "Google", "OpenAI"})
REPO_DYNAMIC_OUTPUT_MODALITY_MARKERS = frozenset({"image", "text", "video"})
REPO_FLEXIBLE_PROVIDER_OPTION_MARKERS = frozenset({"anthropic", "google-vertex", "openai"})
REPO_FLEXIBLE_PROVIDER_OPTION_VALUE_SCHEMA = {
    "additionalProperties": {
        "nullable": True,
    },
    "type": "object",
}
REPO_RESPONSES_FLEXIBLE_NULLABILITY_FIELDS = frozenset(
    {
        "instructions",
        "text",
        "top_logprobs",
    }
)
REPO_FLEXIBLE_PLUGIN_OPERATION_KEYS = frozenset(
    {
        "POST /chat/completions",
        "POST /messages",
        "POST /responses",
    }
)
BASELINE_TOP_LEVEL_FIELDS = (
    "components",
    "info",
    "jsonSchemaDialect",
    "openapi",
    "paths",
    "security",
    "servers",
    "tags",
)


def utc_now_iso() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def read_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: Path, payload: Any) -> None:
    ensure_parent(path)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def write_text(path: Path, payload: str) -> None:
    ensure_parent(path)
    path.write_text(payload, encoding="utf-8")


def fetch_spec(url: str) -> dict[str, Any]:
    with urllib.request.urlopen(url) as response:
        return json.load(response)


def strip_doc_only_fields(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: strip_doc_only_fields(item)
            for key, item in value.items()
            if key not in DOC_ONLY_FIELDS
        }

    if isinstance(value, list):
        return [strip_doc_only_fields(item) for item in value]

    return value


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def short_hash(value: Any) -> str:
    return hashlib.sha256(canonical_json(value).encode("utf-8")).hexdigest()[:16]


def validate_openapi_spec(spec: Any, source: str) -> dict[str, Any]:
    if not isinstance(spec, dict):
        raise ValueError(f"{source} did not contain a top-level JSON object.")

    openapi_version = spec.get("openapi")
    if not isinstance(openapi_version, str) or not openapi_version:
        raise ValueError(f"{source} is not an OpenAPI document: missing top-level `openapi` string.")

    paths = spec.get("paths")
    if not isinstance(paths, dict):
        raise ValueError(f"{source} is not an OpenAPI document: missing top-level `paths` object.")

    return spec


def decode_json_pointer_token(token: str) -> str:
    return token.replace("~1", "/").replace("~0", "~")


def resolve_json_pointer(document: dict[str, Any], pointer: str) -> Any:
    if pointer == "#":
        return document

    if not pointer.startswith("#/"):
        raise ValueError(f"Only local JSON pointers are supported, got: {pointer}")

    current: Any = document
    for token in pointer[2:].split("/"):
        decoded_token = decode_json_pointer_token(token)

        if isinstance(current, list):
            try:
                current = current[int(decoded_token)]
            except (ValueError, IndexError) as exc:
                raise KeyError(decoded_token) from exc
            continue

        if isinstance(current, dict):
            current = current[decoded_token]
            continue

        raise TypeError(f"JSON pointer segment {decoded_token!r} cannot be applied to {type(current).__name__}")
    return current


def resolve_local_refs(value: Any, document: dict[str, Any], active_refs: frozenset[str] = frozenset()) -> Any:
    if isinstance(value, dict):
        ref = value.get("$ref")
        if isinstance(ref, str) and ref.startswith("#/"):
            if ref in active_refs:
                return {"$ref": ref}

            resolved = resolve_local_refs(
                resolve_json_pointer(document, ref),
                document,
                active_refs | {ref},
            )
            siblings = {
                key: resolve_local_refs(item, document, active_refs)
                for key, item in value.items()
                if key != "$ref"
            }

            if siblings and isinstance(resolved, dict):
                merged = dict(resolved)
                merged.update(siblings)
                return merged

            if siblings:
                return {"allOf": [resolved], **siblings}

            return resolved

        return {
            key: resolve_local_refs(item, document, active_refs)
            for key, item in value.items()
        }

    if isinstance(value, list):
        return [resolve_local_refs(item, document, active_refs) for item in value]

    return value


def merge_parameter_lists(
    inherited_parameters: list[Any],
    operation_parameters: list[Any],
) -> list[Any]:
    merged: list[Any] = []
    parameter_index: dict[tuple[str, str], int] = {}

    for parameter in inherited_parameters + operation_parameters:
        if not isinstance(parameter, dict):
            merged.append(parameter)
            continue

        name = parameter.get("name")
        location = parameter.get("in")
        if not isinstance(name, str) or not isinstance(location, str):
            merged.append(parameter)
            continue

        parameter_key = (name, location)
        existing_index = parameter_index.get(parameter_key)
        if existing_index is None:
            parameter_index[parameter_key] = len(merged)
            merged.append(parameter)
            continue

        merged[existing_index] = parameter

    return merged


def normalize_parameter_order(parameters: Any) -> Any:
    if not isinstance(parameters, list):
        return parameters

    def parameter_sort_key(parameter: Any) -> tuple[int, str, str, str]:
        if isinstance(parameter, dict):
            name = parameter.get("name")
            location = parameter.get("in")
            if isinstance(name, str) and isinstance(location, str):
                return (0, location, name, canonical_json(parameter))

        return (1, "", "", canonical_json(parameter))

    return sorted(parameters, key=parameter_sort_key)


def repo_known_metadata_parameter_key(parameter: Any) -> tuple[str, str] | None:
    if not isinstance(parameter, dict):
        return None

    name = parameter.get("name")
    location = parameter.get("in")
    if not isinstance(name, str) or not isinstance(location, str):
        return None

    parameter_key = (location, name)
    if parameter_key not in REPO_KNOWN_METADATA_PARAMETERS:
        return None

    return parameter_key


def is_repo_supported_metadata_parameter(parameter: Any) -> bool:
    parameter_key = repo_known_metadata_parameter_key(parameter)
    if parameter_key is None:
        return False

    return parameter == REPO_SUPPORTED_METADATA_PARAMETER_SHAPES[parameter_key]


def collect_repo_supported_metadata_parameters(operation: Any) -> list[str]:
    if not isinstance(operation, dict):
        return []

    parameters = operation.get("parameters")
    if not isinstance(parameters, list):
        return []

    supported_parameters = {
        f"{parameter['in']} {parameter['name']}"
        for parameter in parameters
        if repo_known_metadata_parameter_key(parameter) is not None
    }
    return sorted(supported_parameters)


def collect_exact_repo_supported_metadata_parameters(operation: Any) -> list[str]:
    if not isinstance(operation, dict):
        return []

    parameters = operation.get("parameters")
    if not isinstance(parameters, list):
        return []

    exact_supported_parameters = {
        f"{parameter['in']} {parameter['name']}"
        for parameter in parameters
        if is_repo_supported_metadata_parameter(parameter)
    }
    return sorted(exact_supported_parameters)


def strip_repo_supported_metadata_parameters(operation: Any) -> Any:
    if not isinstance(operation, dict):
        return operation

    stripped_operation = dict(operation)
    parameters = stripped_operation.get("parameters")
    if not isinstance(parameters, list):
        return stripped_operation

    filtered_parameters = [
        parameter
        for parameter in parameters
        if not is_repo_supported_metadata_parameter(parameter)
    ]

    if filtered_parameters:
        stripped_operation["parameters"] = normalize_parameter_order(filtered_parameters)
    else:
        stripped_operation.pop("parameters", None)

    return stripped_operation


def scalar_enum_values(value: Any) -> set[Any]:
    enum_values = value.get("enum") if isinstance(value, dict) else None
    if not isinstance(enum_values, list):
        return set()

    return {
        item
        for item in enum_values
        if item is None or isinstance(item, (str, int, float, bool))
    }


def is_repo_supported_dynamic_provider_name_enum(value: Any) -> bool:
    if not isinstance(value, dict):
        return False

    enum_values = scalar_enum_values(value)
    string_values = {item for item in enum_values if isinstance(item, str)}
    return (
        value.get("type") == "string"
        and value.get("x-speakeasy-unknown-values") == "allow"
        and REPO_DYNAMIC_PROVIDER_NAME_MARKERS.issubset(string_values)
    )


def is_repo_supported_dynamic_output_modality_enum(value: Any) -> bool:
    if not isinstance(value, dict):
        return False

    enum_values = scalar_enum_values(value)
    string_values = {item for item in enum_values if isinstance(item, str)}
    return (
        value.get("type") == "string"
        and value.get("x-speakeasy-unknown-values") == "allow"
        and REPO_DYNAMIC_OUTPUT_MODALITY_MARKERS.issubset(string_values)
    )


def is_repo_supported_provider_options_map(value: Any) -> bool:
    if not isinstance(value, dict):
        return False

    properties = value.get("properties")
    if not isinstance(properties, dict):
        return False

    property_names = set(properties)
    return (
        value.get("type") == "object"
        and REPO_FLEXIBLE_PROVIDER_OPTION_MARKERS.issubset(property_names)
        and all(
            property_schema == REPO_FLEXIBLE_PROVIDER_OPTION_VALUE_SCHEMA
            for property_schema in properties.values()
        )
    )


def is_responses_response_payload_path(operation_key: str, path: tuple[Any, ...]) -> bool:
    return operation_key == "POST /responses" and bool(path) and path[0] == "responses"


def is_request_schema_property_path(path: tuple[Any, ...], property_name: str) -> bool:
    return (
        "requestBody" in path
        and len(path) >= 2
        and path[-2:] == ("properties", property_name)
    )


def is_response_schema_property_path(path: tuple[Any, ...], property_name: str) -> bool:
    return (
        "responses" in path
        and len(path) >= 2
        and path[-2:] == ("properties", property_name)
    )


def is_repo_supported_flexible_plugin_payload_path(
    operation_key: str,
    path: tuple[Any, ...],
) -> bool:
    return (
        operation_key in REPO_FLEXIBLE_PLUGIN_OPERATION_KEYS
        and is_request_schema_property_path(path, "plugins")
    )


def is_repo_supported_messages_tool_payload_path(
    operation_key: str,
    path: tuple[Any, ...],
) -> bool:
    return operation_key == "POST /messages" and is_request_schema_property_path(path, "tools")


def is_repo_supported_responses_tool_payload_path(
    operation_key: str,
    path: tuple[Any, ...],
) -> bool:
    return operation_key == "POST /responses" and is_request_schema_property_path(path, "tools")


def is_repo_supported_responses_output_payload_path(
    operation_key: str,
    path: tuple[Any, ...],
) -> bool:
    return operation_key == "POST /responses" and is_response_schema_property_path(path, "output")


def strip_repo_supported_schema_details(
    operation_key: str,
    value: Any,
    path: tuple[Any, ...] = (),
) -> Any:
    if isinstance(value, dict):
        if is_repo_supported_flexible_plugin_payload_path(operation_key, path):
            return {"<repo-supported-flexible-plugin-payload>": True}

        if is_repo_supported_messages_tool_payload_path(operation_key, path):
            return {"<repo-supported-messages-tool-payload>": True}

        if is_repo_supported_responses_tool_payload_path(operation_key, path):
            return {"<repo-supported-responses-tool-payload>": True}

        if is_repo_supported_responses_output_payload_path(operation_key, path):
            return {"<repo-supported-responses-output-payload>": True}

        stripped = {
            key: strip_repo_supported_schema_details(operation_key, item, path + (key,))
            for key, item in value.items()
        }

        if (
            is_repo_supported_dynamic_provider_name_enum(stripped)
            or is_repo_supported_dynamic_output_modality_enum(stripped)
        ):
            stripped["enum"] = ["<repo-supported-dynamic-enum>"]

        if is_repo_supported_provider_options_map(stripped):
            stripped["properties"] = {
                "<repo-supported-provider-options>": REPO_FLEXIBLE_PROVIDER_OPTION_VALUE_SCHEMA
            }

        if is_responses_response_payload_path(operation_key, path):
            properties = stripped.get("properties")
            if isinstance(properties, dict):
                for field_name in REPO_RESPONSES_FLEXIBLE_NULLABILITY_FIELDS:
                    field_schema = properties.get(field_name)
                    if isinstance(field_schema, dict):
                        field_schema.pop("nullable", None)

        return stripped

    if isinstance(value, list):
        return [
            strip_repo_supported_schema_details(operation_key, item, path + (index,))
            for index, item in enumerate(value)
        ]

    return value


def collect_repo_supported_schema_rules(operation_key: str, value: Any) -> list[str]:
    rules: set[str] = set()

    def collect(item: Any, path: tuple[Any, ...] = ()) -> None:
        if isinstance(item, dict):
            if is_repo_supported_dynamic_provider_name_enum(item):
                rules.add("dynamic provider name enum")
            if is_repo_supported_dynamic_output_modality_enum(item):
                rules.add("dynamic output modality enum")
            if is_repo_supported_provider_options_map(item):
                rules.add("provider-specific options map")
            if is_repo_supported_flexible_plugin_payload_path(operation_key, path):
                rules.add("flexible plugin payload")
            if is_repo_supported_messages_tool_payload_path(operation_key, path):
                rules.add("Messages flexible tool payload")
            if is_repo_supported_responses_tool_payload_path(operation_key, path):
                rules.add("Responses flexible tool payload")
            if is_repo_supported_responses_output_payload_path(operation_key, path):
                rules.add("Responses flexible output payload")
            if is_responses_response_payload_path(operation_key, path):
                properties = item.get("properties")
                if isinstance(properties, dict):
                    for field_name in REPO_RESPONSES_FLEXIBLE_NULLABILITY_FIELDS:
                        field_schema = properties.get(field_name)
                        if isinstance(field_schema, dict) and field_schema.get("nullable") is True:
                            rules.add("Responses flexible nullable fields")

            for key, child in item.items():
                collect(child, path + (key,))
            return

        if isinstance(item, list):
            for index, child in enumerate(item):
                collect(child, path + (index,))

    collect(value)
    return sorted(rules)


def classify_repo_impact_for_changed_operation(
    operation_key: str,
    baseline_operation: dict[str, Any],
    candidate_operation: dict[str, Any],
) -> dict[str, Any]:
    baseline_normalized = baseline_operation["normalized"]
    candidate_normalized = candidate_operation["normalized"]
    supported_parameters = sorted(
        {
            *collect_repo_supported_metadata_parameters(baseline_normalized),
            *collect_repo_supported_metadata_parameters(candidate_normalized),
        }
    )
    exact_supported_parameters = sorted(
        {
            *collect_exact_repo_supported_metadata_parameters(baseline_normalized),
            *collect_exact_repo_supported_metadata_parameters(candidate_normalized),
        }
    )

    baseline_without_supported = strip_repo_supported_metadata_parameters(
        baseline_normalized
    )
    candidate_without_supported = strip_repo_supported_metadata_parameters(
        candidate_normalized
    )
    schema_rules = sorted(
        {
            *collect_repo_supported_schema_rules(operation_key, baseline_without_supported),
            *collect_repo_supported_schema_rules(operation_key, candidate_without_supported),
        }
    )

    baseline_without_supported = strip_repo_supported_schema_details(
        operation_key,
        baseline_without_supported,
    )
    candidate_without_supported = strip_repo_supported_schema_details(
        operation_key,
        candidate_without_supported,
    )

    if (
        (exact_supported_parameters or schema_rules)
        and baseline_without_supported == candidate_without_supported
    ):
        return {
            "category": "already_supported",
            "schema_rules": schema_rules,
            "supported_parameters": supported_parameters,
        }

    return {
        "category": "actionable",
        "schema_rules": schema_rules,
        "supported_parameters": supported_parameters,
    }


def normalize_security_order(security: Any) -> Any:
    if not isinstance(security, list):
        return security

    normalized_requirements: list[Any] = []
    for requirement in security:
        if not isinstance(requirement, dict):
            normalized_requirements.append(requirement)
            continue

        normalized_requirement: dict[str, Any] = {}
        for scheme_name in sorted(requirement):
            scopes = requirement[scheme_name]
            if isinstance(scopes, list):
                normalized_requirement[scheme_name] = sorted(scopes, key=canonical_json)
            else:
                normalized_requirement[scheme_name] = scopes

        normalized_requirements.append(normalized_requirement)

    return sorted(normalized_requirements, key=canonical_json)


def canonicalize_unordered_schema_collections(value: Any, key: str | None = None) -> Any:
    if isinstance(value, dict):
        normalized: dict[str, Any] = {}
        for child_key, child_value in value.items():
            normalized[child_key] = canonicalize_unordered_schema_collections(
                child_value,
                child_key,
            )

        dependent_required = normalized.get("dependentRequired")
        if isinstance(dependent_required, dict):
            normalized["dependentRequired"] = {
                dependency_key: canonicalize_unordered_schema_collections(
                    dependency_value,
                    "required",
                )
                for dependency_key, dependency_value in dependent_required.items()
            }

        return normalized

    if isinstance(value, list):
        normalized_items = [
            canonicalize_unordered_schema_collections(item)
            for item in value
        ]

        if key in {"required", "enum", "type"}:
            return sorted(normalized_items, key=canonical_json)

        if key in {"allOf", "anyOf", "oneOf"}:
            return sorted(normalized_items, key=canonical_json)

        return normalized_items

    return value


def collect_effective_security_schemes(
    effective_security: Any,
    spec: dict[str, Any],
) -> dict[str, Any] | None:
    if not isinstance(effective_security, list):
        return None

    security_schemes = spec.get("components", {}).get("securitySchemes", {})
    if not isinstance(security_schemes, dict):
        return None

    resolved_schemes: dict[str, Any] = {}
    for requirement in effective_security:
        if not isinstance(requirement, dict):
            continue

        for scheme_name in sorted(requirement):
            scheme_definition = security_schemes.get(scheme_name)
            if scheme_definition is None:
                continue
            resolved_schemes[scheme_name] = resolve_local_refs(scheme_definition, spec)

    return resolved_schemes or None


def inherit_effective_operation_fields(
    raw_operation: dict[str, Any],
    path_item: dict[str, Any],
    spec: dict[str, Any],
) -> dict[str, Any]:
    inherited_operation = dict(raw_operation)

    path_parameters = path_item.get("parameters", [])
    operation_parameters = raw_operation.get("parameters", [])
    if path_parameters or operation_parameters:
        inherited_operation["parameters"] = merge_parameter_lists(
            path_parameters if isinstance(path_parameters, list) else [],
            operation_parameters if isinstance(operation_parameters, list) else [],
        )
        inherited_operation["parameters"] = normalize_parameter_order(
            inherited_operation["parameters"]
        )

    if "servers" not in inherited_operation:
        if "servers" in path_item:
            inherited_operation["servers"] = path_item["servers"]
        elif "servers" in spec:
            inherited_operation["servers"] = spec["servers"]

    if "security" not in inherited_operation and "security" in spec:
        inherited_operation["security"] = spec["security"]
    if "security" in inherited_operation:
        inherited_operation["security"] = normalize_security_order(
            inherited_operation["security"]
        )

    resolved_security_schemes = collect_effective_security_schemes(
        inherited_operation.get("security"),
        spec,
    )
    if resolved_security_schemes:
        inherited_operation["_effective_security_schemes"] = resolved_security_schemes

    return inherited_operation


def normalize_path_item(path_item: dict[str, Any], spec: dict[str, Any]) -> dict[str, Any]:
    resolved_path_item = resolve_local_refs(path_item, spec)
    if not isinstance(resolved_path_item, dict):
        raise TypeError("Resolved Path Item must be an object")
    return resolved_path_item


def normalize_operation(raw_operation: dict[str, Any], path_item: dict[str, Any], spec: dict[str, Any]) -> Any:
    inherited_operation = inherit_effective_operation_fields(raw_operation, path_item, spec)
    normalized_operation = strip_doc_only_fields(inherited_operation)
    return canonicalize_unordered_schema_collections(normalized_operation)


def collect_operations(spec: dict[str, Any]) -> dict[str, dict[str, Any]]:
    operations: dict[str, dict[str, Any]] = {}

    for path, raw_path_item in sorted(spec.get("paths", {}).items()):
        path_item = normalize_path_item(raw_path_item, spec)
        for method in HTTP_METHODS:
            if method not in path_item:
                continue

            raw_operation = path_item[method]
            if not isinstance(raw_operation, dict):
                continue

            normalized = normalize_operation(raw_operation, path_item, spec)
            operation_key = f"{method.upper()} {path}"
            operations[operation_key] = {
                "id": operation_key,
                "method": method.upper(),
                "path": path,
                "operation_id": raw_operation.get("operationId"),
                "tags": raw_operation.get("tags", []),
                "deprecated": raw_operation.get("deprecated", False),
                "fingerprint": short_hash(normalized),
                "normalized": normalized,
            }

    return operations


def reduce_spec_for_baseline(spec: dict[str, Any]) -> dict[str, Any]:
    reduced = {field: spec[field] for field in BASELINE_TOP_LEVEL_FIELDS if field in spec}
    reduced["paths"] = spec.get("paths", {})
    return reduced


def build_snapshot(spec: dict[str, Any], source_url: str) -> dict[str, Any]:
    operations = collect_operations(spec)
    return {
        "captured_at": utc_now_iso(),
        "info": spec.get("info", {}),
        "operation_count": len(operations),
        "operations": [
            {
                "deprecated": operation["deprecated"],
                "fingerprint": operation["fingerprint"],
                "id": operation["id"],
                "method": operation["method"],
                "operation_id": operation["operation_id"],
                "path": operation["path"],
                "tags": operation["tags"],
            }
            for _, operation in sorted(operations.items())
        ],
        "source_url": source_url,
    }


def diff_preview(before: Any, after: Any, max_diff_lines: int) -> list[str]:
    diff_lines = list(
        difflib.unified_diff(
            json.dumps(before, indent=2, sort_keys=True).splitlines(),
            json.dumps(after, indent=2, sort_keys=True).splitlines(),
            fromfile="baseline",
            tofile="candidate",
            lineterm="",
        )
    )

    if len(diff_lines) <= max_diff_lines:
        return diff_lines

    truncated = diff_lines[:max_diff_lines]
    truncated.append(f"... truncated {len(diff_lines) - max_diff_lines} additional diff line(s) ...")
    return truncated


def build_report(
    baseline_spec: dict[str, Any],
    candidate_spec: dict[str, Any],
    baseline_label: str,
    candidate_label: str,
    source_url: str,
    max_diff_lines: int,
) -> dict[str, Any]:
    baseline_operations = collect_operations(baseline_spec)
    candidate_operations = collect_operations(candidate_spec)

    baseline_keys = set(baseline_operations)
    candidate_keys = set(candidate_operations)

    added = sorted(candidate_keys - baseline_keys)
    removed = sorted(baseline_keys - candidate_keys)
    changed = []
    already_supported_count = 0
    actionable_changed_count = 0

    for operation_key in sorted(baseline_keys & candidate_keys):
        baseline_operation = baseline_operations[operation_key]
        candidate_operation = candidate_operations[operation_key]
        if baseline_operation["fingerprint"] == candidate_operation["fingerprint"]:
            continue

        repo_impact = classify_repo_impact_for_changed_operation(
            operation_key,
            baseline_operation,
            candidate_operation,
        )
        if repo_impact["category"] == "already_supported":
            already_supported_count += 1
        else:
            actionable_changed_count += 1

        changed.append(
            {
                "id": operation_key,
                "baseline_fingerprint": baseline_operation["fingerprint"],
                "candidate_fingerprint": candidate_operation["fingerprint"],
                "repo_impact": repo_impact,
                "diff_preview": diff_preview(
                    baseline_operation["normalized"],
                    candidate_operation["normalized"],
                    max_diff_lines=max_diff_lines,
                ),
            }
        )

    return {
        "baseline": {
            "info": baseline_spec.get("info", {}),
            "label": baseline_label,
            "operation_count": len(baseline_operations),
        },
        "candidate": {
            "info": candidate_spec.get("info", {}),
            "label": candidate_label,
            "operation_count": len(candidate_operations),
        },
        "compared_at": utc_now_iso(),
        "source_url": source_url,
        "has_drift": bool(added or removed or changed),
        "has_actionable_drift": bool(added or removed or actionable_changed_count),
        "summary": {
            "added": len(added),
            "removed": len(removed),
            "changed": len(changed),
        },
        "repo_summary": {
            "already_supported_changed": already_supported_count,
            "actionable_added": len(added),
            "actionable_removed": len(removed),
            "actionable_changed": actionable_changed_count,
        },
        "added": [
            {
                "fingerprint": candidate_operations[operation_key]["fingerprint"],
                "id": operation_key,
                "operation_id": candidate_operations[operation_key]["operation_id"],
                "tags": candidate_operations[operation_key]["tags"],
            }
            for operation_key in added
        ],
        "removed": [
            {
                "fingerprint": baseline_operations[operation_key]["fingerprint"],
                "id": operation_key,
                "operation_id": baseline_operations[operation_key]["operation_id"],
                "tags": baseline_operations[operation_key]["tags"],
            }
            for operation_key in removed
        ],
        "changed": changed,
    }


def markdown_list(title: str, items: list[str]) -> list[str]:
    if not items:
        return [f"## {title}", "", "- None", ""]

    return [f"## {title}", "", *[f"- `{item}`" for item in items], ""]


def render_markdown_report(report: dict[str, Any]) -> str:
    already_supported_changes = [
        entry
        for entry in report["changed"]
        if entry["repo_impact"]["category"] == "already_supported"
    ]
    actionable_changes = [
        entry
        for entry in report["changed"]
        if entry["repo_impact"]["category"] == "actionable"
    ]

    lines = [
        "# OpenRouter OpenAPI Drift Report",
        "",
        f"Compared at: `{report['compared_at']}`",
        f"Upstream source: `{report['source_url']}`",
        "",
        "## Summary",
        "",
        f"- Baseline: `{report['baseline']['label']}` with `{report['baseline']['operation_count']}` method+path entries",
        f"- Candidate: `{report['candidate']['label']}` with `{report['candidate']['operation_count']}` method+path entries",
        f"- Added operations: `{report['summary']['added']}`",
        f"- Removed operations: `{report['summary']['removed']}`",
        f"- Changed operations: `{report['summary']['changed']}`",
        "",
    ]

    if not report["has_drift"]:
        lines.extend(
            [
                "No operation-level drift detected after resolving local component refs and removing",
                "docs-only OpenAPI fields",
                "(`summary`, `description`, `title`, `example`, `examples`, `externalDocs`).",
                "",
            ]
        )
    else:
        lines.extend(
            [
                "Operation-level drift detected after resolving local component refs and removing",
                "docs-only OpenAPI fields",
                "(`summary`, `description`, `title`, `example`, `examples`, `externalDocs`).",
                "",
            ]
        )

    lines.extend(
        [
            "## Repo-Aware Classification",
            "",
            f"- Actionable added operations: `{report['repo_summary']['actionable_added']}`",
            f"- Actionable removed operations: `{report['repo_summary']['actionable_removed']}`",
            f"- Actionable changed operations: `{report['repo_summary']['actionable_changed']}`",
            (
                "- Changed operations already supported by repo handling: "
                f"`{report['repo_summary']['already_supported_changed']}`"
            ),
            "",
        ]
    )

    if report["has_drift"] and not report["has_actionable_drift"]:
        lines.extend(
            [
                "No actionable repo drift detected after repo-aware classification.",
                "The tracked baseline is stale, but the changed operations are already covered by",
                "the repository's global request-metadata or flexible schema handling.",
                "",
            ]
        )

    lines.extend(markdown_list("Added Operations", [entry["id"] for entry in report["added"]]))
    lines.extend(markdown_list("Removed Operations", [entry["id"] for entry in report["removed"]]))

    lines.append("## Changes Already Supported By Repo")
    lines.append("")
    if not already_supported_changes:
        lines.append("- None")
        lines.append("")
    else:
        for entry in already_supported_changes:
            support_notes = []
            if entry["repo_impact"]["supported_parameters"]:
                support_notes.extend(
                    f"`{parameter}`"
                    for parameter in entry["repo_impact"]["supported_parameters"]
                )
            if entry["repo_impact"].get("schema_rules"):
                support_notes.extend(
                    f"`{rule}`"
                    for rule in entry["repo_impact"]["schema_rules"]
                )
            support_note = ", ".join(support_notes)
            lines.append(f"- `{entry['id']}` ({support_note})")
        lines.append("")

    lines.append("## Actionable Changed Operations")
    lines.append("")
    if not actionable_changes:
        lines.append("- None")
        lines.append("")
    else:
        for entry in actionable_changes:
            lines.append(
                f"- `{entry['id']}` "
                f"(`{entry['baseline_fingerprint']}` -> `{entry['candidate_fingerprint']}`)"
            )
            support_notes = []
            if entry["repo_impact"]["supported_parameters"]:
                support_notes.extend(
                    f"`{parameter}`"
                    for parameter in entry["repo_impact"]["supported_parameters"]
                )
            if entry["repo_impact"].get("schema_rules"):
                support_notes.extend(
                    f"`{rule}`"
                    for rule in entry["repo_impact"]["schema_rules"]
                )
            if support_notes:
                lines.append(f"  Repo already covers: {', '.join(support_notes)}")
            lines.append("")
            lines.append("```diff")
            lines.extend(entry["diff_preview"] or ["# normalized operation diff was empty"])
            lines.append("```")
            lines.append("")

    lines.extend(
        [
            "## Follow-up",
            "",
            "- Review the upstream spec change against `docs/operations/official-endpoint-test-matrix.md`.",
            "- If the upstream change is accepted, refresh the tracked baseline with `just openapi-refresh-baseline`.",
            "- Update docs, tests, or endpoint coverage notes before closing the follow-up issue.",
            "",
        ]
    )

    return "\n".join(lines)


def write_github_output(
    path: Path,
    *,
    has_drift: bool,
    has_actionable_drift: bool,
    report_md: Path,
    report_json: Path,
) -> None:
    ensure_parent(path)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(f"has_drift={'true' if has_drift else 'false'}\n")
        handle.write(
            "has_actionable_drift="
            f"{'true' if has_actionable_drift else 'false'}\n"
        )
        handle.write(f"report_markdown={report_md}\n")
        handle.write(f"report_json={report_json}\n")


def command_refresh_baseline(args: argparse.Namespace) -> int:
    source_url = args.source_url or UPSTREAM_OPENAPI_URL
    source_label = str(args.source_file) if args.source_file else source_url
    raw_spec = read_json(args.source_file) if args.source_file else fetch_spec(source_url)
    spec = validate_openapi_spec(raw_spec, source_label)
    reduced_spec = reduce_spec_for_baseline(spec)
    snapshot = build_snapshot(reduced_spec, source_url)
    write_json(args.baseline_json, reduced_spec)
    write_json(args.operations_json, snapshot)
    print(
        f"Refreshed baseline from {source_url} with "
        f"{snapshot['operation_count']} method+path entries."
    )
    print(f"- raw baseline: {args.baseline_json}")
    print(f"- normalized snapshot: {args.operations_json}")
    return 0


def command_refresh_source(args: argparse.Namespace) -> int:
    source_url = args.source_url or UPSTREAM_OPENAPI_URL
    source_label = str(args.source_file) if args.source_file else source_url
    raw_spec = read_json(args.source_file) if args.source_file else fetch_spec(source_url)
    spec = validate_openapi_spec(raw_spec, source_label)
    write_json(args.source_json, spec)
    print(f"Refreshed generation source snapshot from {source_url}.")
    print(f"- source snapshot: {args.source_json}")
    return 0


def command_compare(args: argparse.Namespace) -> int:
    baseline_spec = validate_openapi_spec(read_json(args.baseline), str(args.baseline))
    candidate_label = args.candidate_url or str(args.candidate)
    raw_candidate_spec = fetch_spec(args.candidate_url) if args.candidate_url else read_json(args.candidate)
    candidate_spec = validate_openapi_spec(raw_candidate_spec, candidate_label)
    report = build_report(
        baseline_spec=baseline_spec,
        candidate_spec=candidate_spec,
        baseline_label=args.baseline_label,
        candidate_label=args.candidate_label,
        source_url=args.source_url,
        max_diff_lines=args.max_diff_lines,
    )

    report_markdown = render_markdown_report(report)
    write_json(args.report_json, report)
    write_text(args.report_md, report_markdown)

    if args.candidate_operations:
        write_json(args.candidate_operations, build_snapshot(candidate_spec, args.source_url))

    if args.github_output:
        write_github_output(
            args.github_output,
            has_drift=report["has_drift"],
            has_actionable_drift=report["has_actionable_drift"],
            report_md=args.report_md,
            report_json=args.report_json,
        )

    if args.step_summary:
        write_text(args.step_summary, report_markdown)

    print(
        f"Compared baseline `{args.baseline_label}` to candidate `{args.candidate_label}`: "
        f"added={report['summary']['added']}, "
        f"removed={report['summary']['removed']}, "
        f"changed={report['summary']['changed']}, "
        f"actionable_changed={report['repo_summary']['actionable_changed']}, "
        "already_supported_changed="
        f"{report['repo_summary']['already_supported_changed']}"
    )
    print(f"- markdown report: {args.report_md}")
    print(f"- json report: {args.report_json}")

    if report["has_drift"] and args.fail_on_drift:
        return 2

    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="OpenRouter OpenAPI drift tooling")
    subparsers = parser.add_subparsers(dest="command", required=True)

    refresh = subparsers.add_parser(
        "refresh-baseline",
        help="Fetch the latest upstream spec and refresh the tracked baseline artifacts.",
    )
    refresh_source = refresh.add_mutually_exclusive_group()
    refresh_source.add_argument(
        "--source-url",
        default=UPSTREAM_OPENAPI_URL,
        help="OpenAPI URL used to refresh the tracked baseline.",
    )
    refresh_source.add_argument(
        "--source-file",
        type=Path,
        help="Local OpenAPI JSON file used to refresh the tracked baseline.",
    )
    refresh.add_argument(
        "--baseline-json",
        type=Path,
        required=True,
        help="Path where the raw tracked baseline JSON should be written.",
    )
    refresh.add_argument(
        "--operations-json",
        type=Path,
        required=True,
        help="Path where the normalized operations snapshot should be written.",
    )
    refresh.set_defaults(func=command_refresh_baseline)

    refresh_source = subparsers.add_parser(
        "refresh-source",
        help="Fetch and validate a full accepted source snapshot for future generation work.",
    )
    refresh_source_input = refresh_source.add_mutually_exclusive_group()
    refresh_source_input.add_argument(
        "--source-url",
        default=UPSTREAM_OPENAPI_URL,
        help="OpenAPI URL used to refresh the accepted source snapshot.",
    )
    refresh_source_input.add_argument(
        "--source-file",
        type=Path,
        help="Local OpenAPI JSON file used to refresh the accepted source snapshot.",
    )
    refresh_source.add_argument(
        "--source-json",
        type=Path,
        required=True,
        help="Path where the validated full source snapshot should be written.",
    )
    refresh_source.set_defaults(func=command_refresh_source)

    compare = subparsers.add_parser(
        "compare",
        help="Compare the tracked baseline against a candidate OpenAPI spec and emit reports.",
    )
    compare.add_argument(
        "--baseline",
        type=Path,
        required=True,
        help="Tracked baseline OpenAPI JSON file.",
    )
    compare_source = compare.add_mutually_exclusive_group(required=True)
    compare_source.add_argument(
        "--candidate",
        type=Path,
        help="Candidate OpenAPI JSON file to compare against the tracked baseline.",
    )
    compare_source.add_argument(
        "--candidate-url",
        help="Candidate OpenAPI URL to compare against the tracked baseline.",
    )
    compare.add_argument(
        "--source-url",
        default=UPSTREAM_OPENAPI_URL,
        help="Source URL associated with the compared candidate spec.",
    )
    compare.add_argument(
        "--baseline-label",
        default="tracked baseline",
        help="Human-readable label for the tracked baseline in reports.",
    )
    compare.add_argument(
        "--candidate-label",
        default="latest upstream",
        help="Human-readable label for the candidate spec in reports.",
    )
    compare.add_argument(
        "--report-md",
        type=Path,
        required=True,
        help="Markdown report output path.",
    )
    compare.add_argument(
        "--report-json",
        type=Path,
        required=True,
        help="JSON report output path.",
    )
    compare.add_argument(
        "--candidate-operations",
        type=Path,
        help="Optional output path for the candidate normalized operations snapshot.",
    )
    compare.add_argument(
        "--github-output",
        type=Path,
        help="Optional GitHub Actions output file path.",
    )
    compare.add_argument(
        "--step-summary",
        type=Path,
        help="Optional GitHub Actions step summary output path.",
    )
    compare.add_argument(
        "--max-diff-lines",
        type=int,
        default=60,
        help="Maximum diff lines to include for each changed operation in the markdown report.",
    )
    compare.add_argument(
        "--fail-on-drift",
        action="store_true",
        help="Exit with code 2 when drift is detected.",
    )
    compare.set_defaults(func=command_compare)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
