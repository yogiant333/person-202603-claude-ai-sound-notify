#!/bin/bash
# AI Sound Notify - Hook notification script
# Reads server_url from .claude/ai-sound-notify.local.md and sends notification

SOURCE="$1"
EVENT="$2"
MESSAGE="$3"

# Find project-level config first, then user-level
CONFIG_FILE=""
if [ -f ".claude/ai-sound-notify.local.md" ]; then
  CONFIG_FILE=".claude/ai-sound-notify.local.md"
elif [ -f "$HOME/.claude/ai-sound-notify.local.md" ]; then
  CONFIG_FILE="$HOME/.claude/ai-sound-notify.local.md"
fi

# No config = silently exit
if [ -z "$CONFIG_FILE" ]; then
  exit 0
fi

# Extract server_url from YAML frontmatter
SERVER_URL=$(sed -n '/^---$/,/^---$/{ /^server_url:/{ s/^server_url: *//; s/ *$//; p; } }' "$CONFIG_FILE")

# No server_url configured = silently exit
if [ -z "$SERVER_URL" ]; then
  exit 0
fi

# Send notification
curl -s -X POST "${SERVER_URL}/notify" \
  -H 'Content-Type: application/json' \
  -d "{\"source\":\"${SOURCE}\",\"event\":\"${EVENT}\",\"message\":\"${MESSAGE}\"}" \
  --connect-timeout 3 \
  --max-time 5 \
  > /dev/null 2>&1 || true

exit 0
