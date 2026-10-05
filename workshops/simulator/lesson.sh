#!/usr/bin/env bash
#
# Switch this checkout to a lesson of the "simulator" workshop.
#
#   ./workshops/simulator/lesson.sh              show the lessons and where you are
#   ./workshops/simulator/lesson.sh 02           open a lesson (02, 02-fuel or fuel)
#   ./workshops/simulator/lesson.sh --reset      start the current lesson over
#   ./workshops/simulator/lesson.sh --solution   back to main, the full application
#
# main is the complete working simulator and the reference solution for every
# lesson. Each lesson branch ws/simulator/NN is main minus one feature, in two
# commits: the removal, then the `// EXERCISE` hint comments.
#
# You work on your own branch, my/NN, which starts at the removal commit with the
# hint comments applied as uncommitted changes - so your IDE highlights every
# place you need to write code. Commit whenever you like; your commits stay on
# my/NN. --reset keeps a copy of your work in my/NN-before-reset-<time>.
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
WORK_PREFIX="my/"
BASE_BRANCH="main"
REMOTE="origin"
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

has_ref()  { git show-ref --verify --quiet "$1"; }
rev()      { git rev-parse --verify --quiet "$1^{commit}"; }
current()  { git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "?"; }

known_slug() {
  for entry in "${LESSONS[@]}"; do
    [ "$(slug_of "$entry")" = "$1" ] && return 0
  done
  return 1
}

# Accept "2", "02", "02-fuel" and "fuel".
resolve_slug() {
  local target=$1 slug
  known_slug "$target" && { printf '%s' "$target"; return; }
  local matches=()
  if [[ "$target" =~ ^[0-9]+$ ]]; then
    target=$(printf '%02d' "$((10#$target))")
    for entry in "${LESSONS[@]}"; do
      slug=$(slug_of "$entry")
      case "$slug" in "$target"-*) matches+=("$slug") ;; esac
    done
  else
    for entry in "${LESSONS[@]}"; do
      slug=$(slug_of "$entry")
      case "$slug" in
        "$target"*|*"-$target"*) matches+=("$slug") ;;
      esac
    done
  fi
  [ ${#matches[@]} -eq 0 ] && die "no lesson matches '$target' (try --list)"
  [ ${#matches[@]} -gt 1 ] && die "'$target' is ambiguous: ${matches[*]}"
  printf '%s' "${matches[0]}"
}

# The slug of the lesson HEAD is on (my/NN or ws/simulator/NN), or nothing.
current_slug() {
  local cur slug
  cur=$(current)
  case "$cur" in
    "$WORK_PREFIX"*)   slug=${cur#"$WORK_PREFIX"} ;;
    "$BRANCH_PREFIX"*) slug=${cur#"$BRANCH_PREFIX"} ;;
    *) return 0 ;;
  esac
  known_slug "$slug" && printf '%s' "$slug"
  return 0
}

# Make sure the local lesson branch exists and is current with the remote.
#
# A fresh clone only has origin/ws/simulator/NN. We create the local branch from
# it and mark it as a mirror; mirrors are refreshed from the remote on every use,
# so a `git fetch` is enough to pick up a lesson fixed during the day. A local
# branch without the mark (a maintainer's own) is never touched.
ensure_lesson_branch() {
  local branch="$BRANCH_PREFIX$1" remote_ref="refs/remotes/$REMOTE/$BRANCH_PREFIX$1"
  if ! has_ref "refs/heads/$branch"; then
    has_ref "$remote_ref" || die "lesson branch $branch not found (try: git fetch $REMOTE)"
    git branch -q --no-track "$branch" "$remote_ref"
    git config "branch.$branch.lessonMirror" true
  elif [ "$(git config --get "branch.$branch.lessonMirror" || true)" = true ] \
       && has_ref "$remote_ref" \
       && [ "$(rev "$branch")" != "$(rev "$remote_ref")" ] \
       && [ "$(current)" != "$branch" ]; then
    git branch -q -f "$branch" "$remote_ref"
    printf '%slesson %s updated from %s%s\n' "$DIM" "$1" "$REMOTE" "$N"
  fi
}

lesson_base() { git config --get "branch.$WORK_PREFIX$1.lessonBase" || true; }

# True if my/NN exists and holds none of the participant's commits.
work_untouched() {
  local work="$WORK_PREFIX$1"
  has_ref "refs/heads/$work" && [ "$(rev "$work")" = "$(lesson_base "$1")" ]
}

# True if the working tree holds exactly the lesson's hints and nothing else:
# untouched branch, nothing staged, working tree identical to the lesson tip.
only_hints() {
  work_untouched "$1" \
    && git diff --cached --quiet \
    && git diff --quiet "$BRANCH_PREFIX$1" --
}

# (Re)create my/NN at the lesson's removal commit and apply the hint comments
# as uncommitted changes.
start_lesson() {
  local slug=$1 branch="$BRANCH_PREFIX$1" work="$WORK_PREFIX$1" base files
  base=$(rev "$branch^") || die "$branch has no parent commit"
  git switch -q -C "$work" "$base"
  git config "branch.$work.lessonBase" "$base"
  files=$(git diff --name-only "$base" "$branch")
  if [ -n "$files" ]; then
    # shellcheck disable=SC2086 # lesson paths contain no spaces
    git restore --source="$branch" --worktree -- $files
  fi
}

# Before HEAD moves, make sure nothing of the participant's is lost.
leave_current() {
  local slug
  slug=$(current_slug)
  if git diff --quiet && git diff --cached --quiet; then
    return 0
  fi
  if [ -n "$slug" ] && [ "$(current)" = "$WORK_PREFIX$slug" ] && only_hints "$slug"; then
    # Only the hints: they come back when the lesson is opened again.
    git reset -q --hard
    return 0
  fi
  printf '%sYou have uncommitted changes.%s\n\n' "$Y" "$N"
  git status --short | sed 's/^/  /'
  cat <<MSG

Keep your work first:

  ${B}git commit -am "my work on $(current)"${N}

then run this again. (Your commit stays on this branch, and opening the lesson
again brings you back to it.)
MSG
  exit 1
}

show_list() {
  local cur slug branch marker state
  cur=$(current)
  printf '%sSimulator workshop%s\n\n' "$B" "$N"
  if [ "$cur" = "$BASE_BRANCH" ]; then
    printf '  %s>%s %-15s %sthe full application (reference solution)%s\n' "$G" "$N" "$BASE_BRANCH" "$DIM" "$N"
  else
    printf '    %-15s %sthe full application (reference solution)%s\n' "$BASE_BRANCH" "$DIM" "$N"
  fi
  for entry in "${LESSONS[@]}"; do
    slug=$(slug_of "$entry")
    branch="$BRANCH_PREFIX$slug"
    state=""
    if [ "$cur" = "$WORK_PREFIX$slug" ] || [ "$cur" = "$branch" ]; then
      marker="${G}>${N}"
    elif has_ref "refs/heads/$branch" || has_ref "refs/remotes/$REMOTE/$branch"; then
      marker=" "
    else
      marker="${Y}?${N}"
    fi
    if has_ref "refs/heads/$WORK_PREFIX$slug" && ! work_untouched "$slug"; then
      state=" ${Y}(your work is on $WORK_PREFIX$slug)${N}"
    fi
    printf '  %s %-15s %s%s%s%s\n' "$marker" "$slug" "$DIM" "$(label_of "$entry")" "$N" "$state"
  done
  printf '\n  currently on %s%s%s\n' "$B" "$cur" "$N"
  printf '  %sbriefs and notes: %s/%s\n' "$DIM" "$DOCS_DIR" "$N"
}

ACTION="switch"
TARGET=""
FRESH=""

while [ $# -gt 0 ]; do
  case "$1" in
    --solution|--main) TARGET="$BASE_BRANCH" ;;
    --reset)           ACTION="reset" ;;
    --list)            show_list; exit 0 ;;
    -h|--help)         sed -n '2,21p' "$SELF" | sed 's/^# \{0,1\}//'; exit 0 ;;
    -*)                die "unknown option: $1 (try --help)" ;;
    *)                 TARGET="$1" ;;
  esac
  shift
done

if [ "$ACTION" = "switch" ] && [ -z "$TARGET" ]; then
  show_list
  exit 0
fi

if [ "$TARGET" = "$BASE_BRANCH" ]; then
  [ "$ACTION" = "reset" ] && die "--reset applies to a lesson, not to $BASE_BRANCH"
  leave_current
  git switch -q "$BASE_BRANCH"
  printf '%sNow on %s%s, the full application.\n' "$B" "$BASE_BRANCH" "$N"
  exit 0
fi

if [ -n "$TARGET" ]; then
  SLUG=$(resolve_slug "$TARGET")
else
  SLUG=$(current_slug)
  [ -n "$SLUG" ] || die "--reset: not on a lesson; name one, e.g. --reset 04"
fi
BRANCH="$BRANCH_PREFIX$SLUG"
WORK="$WORK_PREFIX$SLUG"

if [ "$ACTION" = "reset" ]; then
  # Keep anything worth keeping, then start over from the published lesson.
  if [ "$(current)" = "$WORK" ] && ! only_hints "$SLUG" \
     && { ! git diff --quiet || ! git diff --cached --quiet; }; then
    git commit -q -am "lesson.sh: work in progress before reset"
  fi
  if has_ref "refs/heads/$WORK" && ! work_untouched "$SLUG"; then
    SAVED="$WORK-before-reset-$(date +%Y%m%d-%H%M%S)"
    git branch -q "$SAVED" "$WORK"
    printf '%sYour work is saved on %s%s\n' "$Y" "$SAVED" "$N"
  fi
  if [ "$(current)" = "$WORK" ]; then
    git reset -q --hard
  else
    leave_current
  fi
  ensure_lesson_branch "$SLUG"
  start_lesson "$SLUG"
  FRESH=1
else
  [ "$(current)" = "$WORK" ] && { printf 'Already on %s.\n' "$WORK"; exit 0; }
  leave_current
  ensure_lesson_branch "$SLUG"
  if ! has_ref "refs/heads/$WORK" || work_untouched "$SLUG"; then
    start_lesson "$SLUG"
    FRESH=1
  else
    git switch -q "$WORK"
    if [ "$(lesson_base "$SLUG")" != "$(rev "$BRANCH^")" ]; then
      printf '%sThis lesson has been updated since you started it.%s\n' "$Y" "$N"
      printf '  %s --reset%s starts over from the new version (your work is kept).\n' "$0" "$N"
    fi
  fi
fi

printf '\n%sNow on %s%s (lesson %s)\n' "$B" "$WORK" "$N" "$SLUG"
printf '  brief: %s%s/lesson-%s.md%s\n' "$B" "$DOCS_DIR" "$SLUG" "$N"
if [ -n "$FRESH" ]; then
  cat <<HINTS

  The ${B}// EXERCISE${N} comments are uncommitted changes: your IDE marks every
  place to write code. ${DIM}git diff --stat${N} lists the files.
HINTS
else
  printf '\n  Back to your work. %sgit log %s..%s%s shows your commits.\n' \
    "$DIM" "$(lesson_base "$SLUG" | cut -c1-7)" "$WORK" "$N"
fi
cat <<NEXT

  ${DIM}cd sim && cargo test${N}                     the lesson's tests
  ${DIM}cd sim && cargo run --release -p sim -- test${N}   drive the robot
  ${DIM}git commit -am "wip"${N}                     keep your work (it stays on $WORK)
  ${DIM}$0 --reset${N}         start this lesson over
  ${DIM}$0 --solution${N}      back to the full application
NEXT
