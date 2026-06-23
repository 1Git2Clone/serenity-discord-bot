# Single source of truth for the cargo feature set production ships.
# Sourceable snippet (no shebang): `source scripts/deploy-features.sh`.
# bg-deploy.sh sources this for its default; the CI build/clippy matrix
# mirrors the same string with a comment pointing here.
# shellcheck disable=SC2034
DEPLOY_FEATURES="opentelemetry ai-openrouter util-download"
