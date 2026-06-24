#!/usr/bin/env bash

set -euo pipefail

# Configuration
PATTERN="serenity-bot"
SLEEP_DELAY=5 # Seconds to wait for the new service to stabilize
SUPERVISOR_CONF="/etc/supervisor/conf.d/serenity-bot.conf"

# supervisorctl needs root to reach its unix socket; route every call through
# sudo so the script works when invoked as a normal user.
SUPERVISORCTL="sudo supervisorctl"

# Default feature set comes from the shared single source of truth; the $1
# override still wins when given.
# shellcheck source=scripts/deploy-features.sh
. "$(dirname "$0")/deploy-features.sh"
FEATURES="${1:-$DEPLOY_FEATURES}"

echo "=== Starting Blue-Green Restart for ${PATTERN}* ==="

# 1. Pull latest changes and rebuild only if there's something new
REPO_DIR="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
echo "--> Fetching latest changes in $REPO_DIR..."
PULL_OUTPUT=$(git -C "$REPO_DIR" pull 2>&1)
echo "$PULL_OUTPUT"

if ! echo "$PULL_OUTPUT" | grep -q "Already up to date"; then
  echo "--> Changes detected. Building release binary with features $FEATURES..."
  cargo build --release --manifest-path "$REPO_DIR/Cargo.toml" --features="$FEATURES"
else
  echo "--> Already up to date. Skipping build."
fi

# 2. Keep the live supervisor config symlinked to the repo and load any
#    changes. ln -sf is idempotent; reread/update are no-ops when nothing
#    changed, so this is safe to run on every deploy.
REPO_CONF="$REPO_DIR/deploy/supervisor/serenity-bot.conf"
echo "--> Syncing supervisor config from $REPO_CONF..."
sudo ln -sf "$REPO_CONF" "$SUPERVISOR_CONF"
${SUPERVISORCTL} reread
${SUPERVISORCTL} update

# 3. Get all matching services from supervisorctl
SERVICES=$(${SUPERVISORCTL} status | awk '{print $1}' | grep "^${PATTERN}" || true)

if [ -z "$SERVICES" ]; then
  echo "No services found matching pattern: ${PATTERN}*"
  exit 0
fi

# 4. Track processed pairs so we don't restart the same app twice
PROCESSED_APPS=()

for SERVICE in $SERVICES; do
  # Match the service name and extract the base app name. Names arrive in
  # "group:program" form with a blue/green slot suffix, e.g.
  # serenity-bot:serenity-bot-a-blue. Each shard range (serenity-bot-a,
  # serenity-bot-b) is its own base app with its own blue/green pair.
  if [[ "$SERVICE" =~ ^(.*)-(blue|green)$ ]]; then
    BASE_APP="${BASH_REMATCH[1]}"
  else
    echo "Skipping $SERVICE (does not end in -blue or -green)"
    continue
  fi

  # Check if we already handled this app pair
  ALREADY_PROCESSED=0
  for APP in "${PROCESSED_APPS[@]}"; do
    if [ "$APP" = "$BASE_APP" ]; then
      ALREADY_PROCESSED=1
      break
    fi
  done

  if [ "$ALREADY_PROCESSED" -eq 1 ]; then
    continue
  fi

  echo "------------------------------------------------"
  echo "Processing Blue-Green deployment for: $BASE_APP"

  BLUE_SERVICE="${BASE_APP}-blue"
  GREEN_SERVICE="${BASE_APP}-green"

  # 5. Determine which service is currently running
  BLUE_STATUS=$(${SUPERVISORCTL} status "$BLUE_SERVICE" | awk '{print $2}' || echo "STOPPED")
  GREEN_STATUS=$(${SUPERVISORCTL} status "$GREEN_SERVICE" | awk '{print $2}' || echo "STOPPED")

  ACTIVE_SERVICE=""
  IDLE_SERVICE=""

  if [ "$BLUE_STATUS" == "RUNNING" ]; then
    ACTIVE_SERVICE="$BLUE_SERVICE"
    IDLE_SERVICE="$GREEN_SERVICE"
  elif [ "$GREEN_STATUS" == "RUNNING" ]; then
    ACTIVE_SERVICE="$GREEN_SERVICE"
    IDLE_SERVICE="$BLUE_SERVICE"
  else
    echo "Warning: Neither slot is running for $BASE_APP. Defaulting to start blue."
    ACTIVE_SERVICE=""
    IDLE_SERVICE="$BLUE_SERVICE"
  fi

  # 6. Perform the zero-downtime swap
  if [ -n "$ACTIVE_SERVICE" ]; then
    echo "Active service: $ACTIVE_SERVICE"
    echo "Idle service:   $IDLE_SERVICE"

    echo "--> Starting $IDLE_SERVICE..."
    ${SUPERVISORCTL} start "$IDLE_SERVICE"

    echo "--> Waiting $SLEEP_DELAY seconds for $IDLE_SERVICE to stabilize..."
    sleep "$SLEEP_DELAY"

    # Double check if the new service stayed up
    NEW_STATUS=$(${SUPERVISORCTL} status "$IDLE_SERVICE" | awk '{print $2}')
    if [ "$NEW_STATUS" == "RUNNING" ]; then
      echo "--> $IDLE_SERVICE is healthy. Stopping $ACTIVE_SERVICE..."
      ${SUPERVISORCTL} stop "$ACTIVE_SERVICE"
      echo "Successfully swapped to $IDLE_SERVICE"
    else
      echo "ERROR: $IDLE_SERVICE failed to stay RUNNING. Aborting swap to protect live traffic."
      exit 1
    fi
  else
    # Fallback if both were dead
    echo "--> Starting $IDLE_SERVICE fresh..."
    ${SUPERVISORCTL} start "$IDLE_SERVICE"
  fi

  # Append this app pair to our processed list
  PROCESSED_APPS+=("$BASE_APP")
done

echo "------------------------------------------------"
echo "=== Blue-Green Restart Completed Successfully ==="
