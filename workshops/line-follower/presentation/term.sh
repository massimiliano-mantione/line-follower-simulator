#!/usr/bin/env bash
# Open a WezTerm window set up for presenting (big font, F11 for full screen),
# starting in this directory.
set -euo pipefail
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
exec wez --config-file "$DIR/wezterm.lua" start --cwd "$DIR"
