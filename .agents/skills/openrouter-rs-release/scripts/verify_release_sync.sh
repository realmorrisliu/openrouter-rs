#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <version>"
  exit 1
fi

version="$1"

if [[ ! -f Cargo.toml || ! -f README.md || ! -f CHANGELOG.md ]]; then
  echo "Run this script from the repository root (Cargo.toml/README.md/CHANGELOG.md not found)."
  exit 1
fi

fail=0

echo "Checking Cargo.toml version..."
if ! rg -n "^version\\s*=\\s*\"${version}\"$" Cargo.toml >/dev/null; then
  echo "[FAIL] Cargo.toml package version is not ${version}"
  fail=1
else
  echo "[OK] Cargo.toml version matches ${version}"
fi

echo "Checking README installation snippet..."
if ! rg -n "openrouter-rs\\s*=\\s*\"${version}\"" README.md >/dev/null; then
  echo "[FAIL] README installation snippet does not reference ${version}"
  fail=1
else
  echo "[OK] README installation snippet matches ${version}"
fi

echo "Checking CHANGELOG top released section..."
first_release_line="$(rg -n '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' CHANGELOG.md | head -n1 || true)"
if [[ -z "${first_release_line}" ]]; then
  echo "[FAIL] No released version section found in CHANGELOG.md"
  fail=1
elif [[ "${first_release_line}" != *"[${version}]"* ]]; then
  echo "[FAIL] Latest released changelog section is not [${version}]"
  echo "       Found: ${first_release_line}"
  fail=1
else
  echo "[OK] CHANGELOG latest section is [${version}]"
fi

echo "Checking README Release History latest marker..."
latest_line="$(rg -n '^### Version .*\*\(Latest\)\*' README.md | head -n1 || true)"
if [[ -z "${latest_line}" ]]; then
  echo "[FAIL] README Release History missing '(Latest)' marker"
  fail=1
elif [[ "${latest_line}" != *"Version ${version} "* ]]; then
  echo "[FAIL] README latest release marker is not Version ${version}"
  echo "       Found: ${latest_line}"
  fail=1
else
  echo "[OK] README latest release marker matches ${version}"
fi

if [[ ${fail} -ne 0 ]]; then
  echo "Release sync checks failed."
  exit 1
fi

echo "All release sync checks passed for version ${version}."
