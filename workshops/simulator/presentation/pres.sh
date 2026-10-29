#!/usr/bin/env bash
# Show the slides. Needs presenterm 0.16 or later (the theme centers all text with
# `default.alignment`).
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
exec presenterm slides.md
