#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if command -v codecontext &>/dev/null; then
  exec codecontext "$@"
fi

if [ -d "$SCRIPT_DIR/venv" ]; then
  source "$SCRIPT_DIR/venv/bin/activate"
elif [ -d "$SCRIPT_DIR/.venv" ]; then
  source "$SCRIPT_DIR/.venv/bin/activate"
fi

exec python "$SCRIPT_DIR/codecontext.py" "$@"
