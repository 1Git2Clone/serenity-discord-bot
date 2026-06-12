#!/usr/bin/env bash
set -euo pipefail

FEATURES="${FEATURES:-"--all-features"}"
LCOV_PATH="${LCOV_PATH:-"lcov.info"}"

# shellcheck disable=SC2086
cargo llvm-cov $FEATURES \
    --fail-under-lines 70 \
    --lcov \
    --output-path "$LCOV_PATH"

# Check that lines covered for files matching PATTERN meet THRESHOLD percent.
# Aggregates across all matching files (e.g. mod.rs + lib.rs for one module).
check_coverage() {
    local pattern="$1"
    local threshold="$2"
    local lf=0 lh=0 in_file=0

    while IFS= read -r line; do
        case "$line" in
            SF:*)
                [[ "${line#SF:}" =~ $pattern ]] && in_file=1 || in_file=0
                ;;
            LF:*)
                [[ $in_file -eq 1 ]] && lf=$(( lf + ${line#LF:} ))
                ;;
            LH:*)
                [[ $in_file -eq 1 ]] && lh=$(( lh + ${line#LH:} ))
                ;;
        esac
    done < "$LCOV_PATH"

    if [[ $lf -eq 0 ]]; then
        echo "ERROR: no lines found for pattern '$pattern'" >&2
        return 1
    fi

    local pct=$(( lh * 100 / lf ))
    echo "coverage[$pattern]: ${lh}/${lf} lines (${pct}%) — threshold ${threshold}%"
    if [[ $pct -lt $threshold ]]; then
        echo "ERROR: ${pct}% < ${threshold}%" >&2
        return 1
    fi
}

# Pure logic — no I/O, every branch reachable from unit tests.
check_coverage "src/utils/string_manipulation" 100
check_coverage "src/commands/level_logic"       100

# Authorization gate for /ai-review.
check_coverage "src/data/ai/review/guilds"      90
