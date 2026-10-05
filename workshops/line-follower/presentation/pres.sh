#!/usr/bin/env bash
# Show the slides. Uses `prt` (a local presenterm build) when present, else presenterm.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
if command -v prt >/dev/null; then exec prt slides.md; fi
exec presenterm slides.md
