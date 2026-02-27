#!/usr/bin/env bash
set -euo pipefail

# ── Home Assistant add-on entrypoint ──
# Bridges HA's /data/options.json to the env vars the Rust binary expects.
# Also works standalone (no options.json) with sensible defaults.

OPTIONS="/data/options.json"

if [ -f "$OPTIONS" ]; then
    PORT=$(jq -r '.port // 3000' "$OPTIONS")
    LOG_LEVEL=$(jq -r '.log_level // "info"' "$OPTIONS")
else
    PORT="${PORT:-3000}"
    LOG_LEVEL="${LOG_LEVEL:-info}"
fi

# Persistent SQLite database in HA's /data volume
export DATABASE_URL="sqlite:///data/gym.db"
export HOST="0.0.0.0"
export PORT="$PORT"
export RUST_LOG="gym=${LOG_LEVEL},tower_http=${LOG_LEVEL}"

# Auto-generate and persist JWT secret on first run
JWT_SECRET_FILE="/data/.jwt_secret"
if [ ! -f "$JWT_SECRET_FILE" ]; then
    openssl rand -base64 32 > "$JWT_SECRET_FILE"
    chmod 600 "$JWT_SECRET_FILE"
fi
export JWT_SECRET
JWT_SECRET=$(cat "$JWT_SECRET_FILE")

echo "Starting Gym Tracker on port ${PORT}..."
exec /app/gym
