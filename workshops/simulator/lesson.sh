#!/usr/bin/env bash
#
# Switch this checkout to a lesson of the "simulator" workshop.
#
#   ./workshops/simulator/lesson.sh              show the lessons and where you are
#   ./workshops/simulator/lesson.sh 02           switch to a lesson
#   ./workshops/simulator/lesson.sh --solution   back to main, the full application
#
# main is the complete working simulator and the reference solution for every
# lesson. Each lesson branch is main minus exactly one feature.
#
# The briefs and the speaker notes are plain files under workshops/simulator/, so
# they are readable from any branch without switching.
#
# One checkout, one build cache: switching lesson recompiles only the three
# workshop crates, never Bevy, Rapier or Wasmtime.

set -euo pipefail

# Lessons, in workshop order. "slug:label"
LESSONS=(
  "01-wasmtime:embedding Wasmtime (live demo)"
  "02-fuel:SLOT 1 - fuel is the clock"
  "03-stepping:SLOT 2 - on-demand physics ticks"
  "04-physics:SLOT 3 - Rapier bodies, joints and motors"
  "05-sensors:SLOT 4 - ray-cast light sensors"
  "06-replay:SLOT 5 - record and replay"
  "07-async-host:optional - host-side device futures"
  "08-bot-async:optional - the robot async runtime"
  "09-telemetry:optional - telemetry collection"
  "10-track:optional - declarative track segments"
  "11-ui:optional - the record-player UI"
  "12-new-device:capstone - add a device end to end"
)

BRANCH_PREFIX="ws/simulator/"
BASE_BRANCH="main"
DOCS_DIR="workshops/simulator"

if [ -t 1 ]; then
  B=$'\033[1m'; DIM=$'\033[2m'; G=$'\033[32m'; Y=$'\033[33m'; R=$'\033[31m'; N=$'\033[0m'
else
  B=""; DIM=""; G=""; Y=""; R=""; N=""
fi

die() { printf '%serror:%s %s\n' "$R" "$N" "$*" >&2; exit 1; }

SELF=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")

command -v git >/dev/null || die "git not found"
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || die "not inside a git repository"
cd "$REPO_ROOT"

slug_of()  { printf '%s' "${1%%:*}"; }
label_of() { printf '%s' "${1#*:}"; }

known_slug() {
  for entry in "${LESSONS[@]}"; do
    [ "$(slug_of "$entry")" = "$1" ] && return 0
  done
  return 1
}

show_list() {
  local current
  current=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "?")
  printf '%sSimulator workshop%s\n\n' "$B" "$N"
  if [ "$current" = "$BASE_BRANCH" ]; then
    printf '  %s>%s %-15s %sthe full application (reference solution)%s\n' "$G" "$N" "$BASE_BRANCH" "$DIM" "$N"
  else
    printf '    %-15s %sthe full application (reference solution)%s\n' "$BASE_BRANCH" "$DIM" "$N"
  fi
  for entry in "${LESSONS[@]}"; do
    local slug branch marker
    slug=$(slug_of "$entry")
    branch="${BRANCH_PREFIX}${slug}"
    if [ "$current" = "$branch" ]; then
      marker="${G}>${N}"
    elif git show-ref --verify --quiet "refs/heads/$branch"; then
      marker="  "
    else
      marker="${Y}?${N}"
    fi
    printf '  %s %-15s %s%s%s\n' "$marker" "$slug" "$DIM" "$(label_of "$entry")" "$N"
  done
  printf '\n  currently on %s%s%s\n' "$B" "$current" "$N"
  printf '  %sbriefs and notes: %s/%s\n' "$DIM" "$DOCS_DIR" "$N"
}

TARGET=""

while [ $# -gt 0 ]; do
  case "$1" in
    --solution|--main) TARGET="$BASE_BRANCH" ;;
    --list)            show_list; exit 0 ;;
    -h|--help)         sed -n '2,17p' "$SELF" | sed 's/^# \{0,1\}//'; exit 0 ;;
    -*)                die "unknown option: $1 (try --help)" ;;
    *)                 TARGET="$1" ;;
  esac
  shift
done

if [ -z "$TARGET" ]; then
  show_list
  exit 0
fi

BRANCH="$BASE_BRANCH"
SLUG=""
if [ "$TARGET" != "$BASE_BRANCH" ]; then
  # Accept "2", "02", "02-fuel" and "fuel".
  if ! known_slug "$TARGET"; then
    MATCHES=()
    for entry in "${LESSONS[@]}"; do
      slug=$(slug_of "$entry")
      case "$slug" in
        "$TARGET"*|*"-$TARGET") MATCHES+=("$slug") ;;
      esac
    done
    [ ${#MATCHES[@]} -eq 0 ] && die "no lesson matches '$TARGET' (try --list)"
    [ ${#MATCHES[@]} -gt 1 ] && die "'$TARGET' is ambiguous: ${MATCHES[*]}"
    TARGET="${MATCHES[0]}"
  fi
  SLUG="$TARGET"
  BRANCH="${BRANCH_PREFIX}${TARGET}"
fi

git show-ref --verify --quiet "refs/heads/$BRANCH" \
  || die "branch $BRANCH does not exist"

# Refuse to throw away work. Untracked files are fine - git carries them across.
if ! git diff --quiet || ! git diff --cached --quiet; then
  printf '%sYou have uncommitted changes.%s\n\n' "$Y" "$N"
  git status --short | sed 's/^/  /'
  cat <<MSG

Switching lesson would carry them along or refuse outright. Keep your work first:

  ${B}git commit -am "my work on $(git rev-parse --abbrev-ref HEAD)"${N}

then run this again. (Each lesson is its own branch, so a commit here stays here.)
MSG
  exit 1
fi

git switch "$BRANCH"

printf '\n%sNow on %s%s\n' "$B" "$BRANCH" "$N"
if [ -n "$SLUG" ]; then
  printf '  brief: %s%s/lesson-%s.md%s\n' "$B" "$DOCS_DIR" "$SLUG" "$N"
fi
cat <<NEXT

  ${DIM}cd sim && cargo test${NC:-$N}                     the lesson's tests
  ${DIM}cd sim && cargo run --release -p sim -- test${N}   drive the robot
  ${DIM}$0 --solution${N}      back to the full application
NEXT
