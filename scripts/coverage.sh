#!/usr/bin/env bash
set -euo pipefail

FEATURES="${FEATURES:-"--all-features"}"
LCOV_PATH="${LCOV_PATH:-"lcov.info"}"

# shellcheck disable=SC2086
cargo llvm-cov $FEATURES \
    --include-pattern "src/utils/string_manipulation|src/commands/level_logic" \
    --fail-under-lines 100

# shellcheck disable=SC2086
cargo llvm-cov $FEATURES \
    --include-pattern "src/data/ai/review/guilds" \
    --fail-under-lines 90

# shellcheck disable=SC2086
cargo llvm-cov $FEATURES \
    --fail-under-lines 70 \
    --lcov \
    --output-path "$LCOV_PATH"
