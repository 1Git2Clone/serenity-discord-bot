#!/usr/bin/env sh
# Enable the project's git hooks (.githooks/) for this clone.
set -e
cd "$(git rev-parse --show-toplevel)"
git config core.hooksPath .githooks
echo "Git hooks enabled (core.hooksPath = .githooks)."
echo "  pre-commit: cargo fmt --check"
