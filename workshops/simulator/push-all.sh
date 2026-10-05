#!/usr/bin/env bash
#
# Publish main and every lesson branch, then drop the local backup refs.
#
#   ./workshops/simulator/push-all.sh
#
# The lesson branches are rewritten by every rebase (MAINTAINING.md, section 7), so
# they need a force push. --force-with-lease refuses to overwrite a branch whose
# GitHub version differs from our remote-tracking ref, i.e. one that changed since
# we last pushed it. Do not `git fetch` the lesson branches right before running
# this: that would update the tracking refs to whatever is on GitHub, and the lease
# would no longer protect anything.
#
# Every branch rejected as "stale info" means the tracking refs do not match GitHub
# at all (a machine that has never pushed the lesson branches, for instance). Look
# at GitHub first; if nothing there is worth keeping, `git fetch origin` and rerun.
#
# The refs/backup/* refs made before rebasing are deleted only after both pushes
# have succeeded.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

git push origin main
git push --force-with-lease origin 'refs/heads/ws/simulator/*:refs/heads/ws/simulator/*'

git for-each-ref --format='%(refname)' refs/backup | xargs -r -n1 git update-ref -d
echo "Pushed main and the lesson branches; local backup refs removed."
