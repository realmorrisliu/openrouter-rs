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
BASELINE_TOP_LEVEL_FIELDS = (
    "components",
    "info",
    "jsonSchemaDialect",
    "openapi",
    "paths",
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


def inherit_path_item_fields(raw_operation: dict[str, Any], path_item: dict[str, Any]) -> dict[str, Any]:
    inherited_operation = dict(raw_operation)

    path_parameters = path_item.get("parameters", [])
    operation_parameters = raw_operation.get("parameters", [])
    if path_parameters or operation_parameters:
        inherited_operation["parameters"] = merge_parameter_lists(
            path_parameters if isinstance(path_parameters, list) else [],
            operation_parameters if isinstance(operation_parameters, list) else [],
        )

    if "servers" not in inherited_operation and "servers" in path_item:
        inherited_operation["servers"] = path_item["servers"]

    return inherited_operation


def normalize_path_item(path_item: dict[str, Any], spec: dict[str, Any]) -> dict[str, Any]:
    resolved_path_item = resolve_local_refs(path_item, spec)
    if not isinstance(resolved_path_item, dict):
        raise TypeError("Resolved Path Item must be an object")
    return resolved_path_item


def normalize_operation(raw_operation: dict[str, Any], path_item: dict[str, Any]) -> Any:
    inherited_operation = inherit_path_item_fields(raw_operation, path_item)
    return strip_doc_only_fields(inherited_operation)


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

            normalized = normalize_operation(raw_operation, path_item)
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

    for operation_key in sorted(baseline_keys & candidate_keys):
        baseline_operation = baseline_operations[operation_key]
        candidate_operation = candidate_operations[operation_key]
        if baseline_operation["fingerprint"] == candidate_operation["fingerprint"]:
            continue

        changed.append(
            {
                "id": operation_key,
                "baseline_fingerprint": baseline_operation["fingerprint"],
                "candidate_fingerprint": candidate_operation["fingerprint"],
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
        "summary": {
            "added": len(added),
            "removed": len(removed),
            "changed": len(changed),
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

    lines.extend(markdown_list("Added Operations", [entry["id"] for entry in report["added"]]))
    lines.extend(markdown_list("Removed Operations", [entry["id"] for entry in report["removed"]]))

    lines.append("## Changed Operations")
    lines.append("")
    if not report["changed"]:
        lines.append("- None")
        lines.append("")
    else:
        for entry in report["changed"]:
            lines.append(
                f"- `{entry['id']}` "
                f"(`{entry['baseline_fingerprint']}` -> `{entry['candidate_fingerprint']}`)"
            )
            lines.append("")
            lines.append("```diff")
            lines.extend(entry["diff_preview"] or ["# normalized operation diff was empty"])
            lines.append("```")
            lines.append("")

    lines.extend(
        [
            "## Follow-up",
            "",
            "- Review the upstream spec change against `docs/official-endpoint-test-matrix.md`.",
            "- If the upstream change is accepted, refresh the tracked baseline with `just openapi-refresh-baseline`.",
            "- Update docs, tests, or endpoint coverage notes before closing the follow-up issue.",
            "",
        ]
    )

    return "\n".join(lines)


def write_github_output(path: Path, *, has_drift: bool, report_md: Path, report_json: Path) -> None:
    ensure_parent(path)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(f"has_drift={'true' if has_drift else 'false'}\n")
        handle.write(f"report_markdown={report_md}\n")
        handle.write(f"report_json={report_json}\n")


def command_refresh_baseline(args: argparse.Namespace) -> int:
    source_url = args.source_url or UPSTREAM_OPENAPI_URL
    spec = read_json(args.source_file) if args.source_file else fetch_spec(source_url)
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


def command_compare(args: argparse.Namespace) -> int:
    baseline_spec = read_json(args.baseline)
    candidate_spec = fetch_spec(args.candidate_url) if args.candidate_url else read_json(args.candidate)
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
            report_md=args.report_md,
            report_json=args.report_json,
        )

    if args.step_summary:
        write_text(args.step_summary, report_markdown)

    print(
        f"Compared baseline `{args.baseline_label}` to candidate `{args.candidate_label}`: "
        f"added={report['summary']['added']}, "
        f"removed={report['summary']['removed']}, "
        f"changed={report['summary']['changed']}"
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
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
