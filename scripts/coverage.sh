#!/usr/bin/env bash
set -euo pipefail

FEATURES="${FEATURES:-"--all-features"}"
LCOV_PATH="${LCOV_PATH:-"lcov.info"}"

# The full report (including Discord-bound code) still goes to Codecov;
# the thresholds below are checked against filtered subsets of it.
# shellcheck disable=SC2086
cargo llvm-cov $FEATURES \
    --lcov \
    --output-path "$LCOV_PATH"

# Check that lines covered for files matching PATTERN meet THRESHOLD percent.
# Aggregates across all matching files (e.g. mod.rs + lib.rs for one module).
# An optional third EXCLUDE pattern drops matching files from the aggregate.
check_coverage() {
    local pattern="$1"
    local threshold="$2"
    local exclude="${3:-}"
    local lf=0 lh=0 in_file=0

    while IFS= read -r line; do
        case "$line" in
            SF:*)
                in_file=0
                if [[ "${line#SF:}" =~ $pattern ]]; then
                    if [[ -z "$exclude" || ! "${line#SF:}" =~ $exclude ]]; then
                        in_file=1
                    fi
                fi
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

# Global gate over unit-testable code. Files that need a live Discord shard
# or an external service (Redis, the LLM API, GitHub) are excluded — they can
# only be exercised by an integration test suite with a live bot.
DISCORD_BOUND='src/main\.rs'
DISCORD_BOUND+='|src/event_handler/'
DISCORD_BOUND+='|src/commands/(general_commands|level_cmds|embed_commands|cmd_utils)'
DISCORD_BOUND+='|src/utils/replies\.rs'
DISCORD_BOUND+='|src/database/level_system\.rs'
DISCORD_BOUND+='|src/data/(reminders|user_data|database|cache)\.rs'
DISCORD_BOUND+='|src/data/ai/(handler|channels)\.rs'
DISCORD_BOUND+='|src/data/ai/review/(agent|client|github)\.rs'

check_coverage "src/" 70 "$DISCORD_BOUND"
