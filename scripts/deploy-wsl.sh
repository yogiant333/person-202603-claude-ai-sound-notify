#!/bin/bash
# Deploy/update ai-sound-notify to WSL local filesystem and restart service
set -euo pipefail

SRC_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEST_DIR="$HOME/services/ai-sound-notify"
NODE_BIN="$HOME/local/bin"

echo "Syncing from $SRC_DIR to $DEST_DIR ..."
rsync -a "$SRC_DIR/" "$DEST_DIR/" --exclude node_modules --exclude .git --delete

echo "Installing dependencies..."
export PATH="$NODE_BIN:$PATH"
cd "$DEST_DIR/server" && npm install --production

echo "Restarting service..."
systemctl --user restart ai-sound-notify.service
systemctl --user status ai-sound-notify.service --no-pager

echo "Done! Service running on port 9800"
