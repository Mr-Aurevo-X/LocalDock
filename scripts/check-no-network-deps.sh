#!/usr/bin/env bash
# LocalDock security gate: direct network deps + URL grep scan.
# See docs/SECURITY.md for transitive Tauri exceptions.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FORBIDDEN=(
  reqwest hyper ureq curl surf isahc awc attohttpc minreq oauth2 hubcaps octocrab
)

LOCAL_TOMLS=(
  "$ROOT/crates/localdock-core/Cargo.toml"
  "$ROOT/src-tauri/Cargo.toml"
)

dep_failures=()

collect_direct_deps() {
  local toml="$1"
  awk '
    /^\[/ {
      section = $0
      gsub(/^\[|\]$/, "", section)
      next
    }
    section ~ /^(dependencies|dev-dependencies|build-dependencies|target\.[^.]+\.dependencies)$/ {
      if ($0 ~ /^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=/) {
        sub(/^[[:space:]]*/, "")
        sub(/[[:space:]]*=.*/, "")
        print tolower($0)
      }
    }
  ' "$toml"
}

echo "== LocalDock: direct dependency check =="
for toml in "${LOCAL_TOMLS[@]}"; do
  if [[ ! -f "$toml" ]]; then
    dep_failures+=("Missing Cargo.toml: $toml")
    continue
  fi
  mapfile -t direct < <(collect_direct_deps "$toml")
  for crate in "${FORBIDDEN[@]}"; do
    for dep in "${direct[@]}"; do
      if [[ "$dep" == "$crate" ]]; then
        dep_failures+=("Forbidden direct dependency '$crate' in ${toml#"$ROOT"/}")
      fi
    done
  done
done

if ((${#dep_failures[@]} > 0)); then
  printf '%s\n' "${dep_failures[@]}" >&2
  exit 1
fi

if command -v cargo >/dev/null 2>&1; then
  echo "== LocalDock: cargo tree depth-1 verification =="
  for pkg in localdock-core localdock; do
    mapfile -t lines < <(cargo tree --depth 1 -p "$pkg" --prefix none)
    direct_deps=()
    for ((i = 1; i < ${#lines[@]}; i++)); do
      if [[ "${lines[$i]}" =~ ^([A-Za-z0-9_-]+)[[:space:]]v ]]; then
        direct_deps+=("${BASH_REMATCH[1],,}")
      fi
    done
    for crate in "${FORBIDDEN[@]}"; do
      for dep in "${direct_deps[@]}"; do
        if [[ "$dep" == "$crate" ]]; then
          echo "Forbidden crate '$crate' is a direct dependency of $pkg" >&2
          exit 1
        fi
      done
    done
  done
else
  echo "WARN: cargo not found; skipped cargo tree verification" >&2
fi

echo "== LocalDock: URL grep gate =="
SCAN_PATHS=(crates src-tauri/src ui)
ALLOW_RE='http://127\.0\.0\.1|http://localhost|https://github\.com/Mr-Aurevo-X|https://api\.github\.com/repos/Mr-Aurevo-X/LocalDock|https://discord\.com/users/406891052516114442'
url_failures=()

if command -v rg >/dev/null 2>&1; then
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if ! echo "$line" | grep -Eq "$ALLOW_RE"; then
      url_failures+=("$line")
    fi
  done < <(rg -n 'https?://' "${SCAN_PATHS[@]}" || true)
else
  while IFS= read -r file; do
    line_no=0
    while IFS= read -r content || [[ -n "$content" ]]; do
      ((line_no++)) || true
      if echo "$content" | grep -q 'https\?://'; then
        if ! echo "$content" | grep -Eq "$ALLOW_RE"; then
          url_failures+=("${file#"$ROOT"/}:$line_no:$content")
        fi
      fi
    done < "$file"
  done < <(find "${SCAN_PATHS[@]}" -type f 2>/dev/null)
fi

if ((${#url_failures[@]} > 0)); then
  echo "Disallowed http(s):// literals:" >&2
  printf '%s\n' "${url_failures[@]}" >&2
  exit 1
fi

echo "== LocalDock: Semgrep security scan =="
if command -v semgrep >/dev/null 2>&1; then
  semgrep scan --config p/security-audit --error crates src-tauri ui
  echo "semgrep: no findings"
else
  echo "WARN: semgrep not installed; skipped (install for release — see docs/SECURITY.md)" >&2
fi

echo "== LocalDock: gitleaks secret scan =="
if command -v gitleaks >/dev/null 2>&1; then
  gitleaks detect --source "$ROOT" --no-banner
  echo "gitleaks: no findings"
else
  echo "WARN: gitleaks not installed; skipped (install for release — see docs/SECURITY.md)" >&2
fi

echo "OK: LocalDock network dependency and URL gates passed."
