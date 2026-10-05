#!/usr/bin/env python3
"""Split each one-commit lesson branch into <removal> + <hint comments>.

Usage: workshops/simulator/split-hints.py BASE BRANCH [--dry-run]

See MAINTAINING.md, section 7.

The tip tree is kept byte-identical; the new first commit is the tip minus every
hint block, where a hint block is a run of consecutive lines *added* by the branch
that are comments and whose first line starts with `// EXERCISE n.m` (or `///`).
"""
import os
import re
import subprocess
import sys
import tempfile

START = re.compile(r"^\s*//+\s*EXERCISE \d+\.\d+\b")
COMMENT = re.compile(r"^\s*//")
HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@")


def git(*args, env=None, input=None):
    return subprocess.run(
        ["git", *args], check=True, capture_output=True, text=True, env=env, input=input
    ).stdout


def added_lines(base, tip, path):
    """1-based line numbers in tip's version of path that the branch added."""
    out = set()
    for line in git("diff", "-U0", base, tip, "--", path).splitlines():
        m = HUNK.match(line)
        if m:
            start, count = int(m.group(1)), int(m.group(2) or "1")
            out.update(range(start, start + count))
    return out


def strip_hints(text, added):
    lines = text.split("\n")
    keep, removed, i = [], 0, 0
    while i < len(lines):
        n = i + 1
        # A hint starts a comment run; a match in the middle of one is the
        # wrapped tail of another comment ("... used by\n// EXERCISE 4.1 ...").
        mid_run = i > 0 and n - 1 in added and COMMENT.match(lines[i - 1])
        if n in added and START.match(lines[i]) and not mid_run:
            j = i
            while j < len(lines) and (j + 1) in added and COMMENT.match(lines[j]):
                j += 1
            removed += j - i
            i = j
            continue
        keep.append(lines[i])
        i += 1
    return "\n".join(keep), removed


def main():
    base, branch = sys.argv[1], sys.argv[2]
    dry = "--dry-run" in sys.argv
    tip = git("rev-parse", branch).strip()
    if git("rev-parse", tip + "^").strip() != git("rev-parse", base).strip():
        sys.exit(f"{branch}: not exactly one commit on top of {base}")

    files = git("diff", "--name-only", "--diff-filter=AM", base, tip).split()
    with tempfile.NamedTemporaryFile(delete=False) as f:
        index = f.name
    env = dict(os.environ, GIT_INDEX_FILE=index)
    try:
        git("read-tree", tip, env=env)
        total = 0
        for path in files:
            text = git("show", f"{tip}:{path}")
            stripped, removed = strip_hints(text, added_lines(base, tip, path))
            if not removed:
                continue
            total += removed
            mode = git("ls-tree", tip, "--", path).split()[0]
            blob = git("hash-object", "-w", "--stdin", input=stripped).strip()
            git("update-index", "--cacheinfo", f"{mode},{blob},{path}", env=env)
            print(f"  {path}: {removed} hint lines")
        tree1 = git("write-tree", env=env).strip()
    finally:
        os.unlink(index)

    if dry:
        print(f"{branch}: {total} hint lines (dry run)")
        return

    # Commit 1: the removal, with the original author, date and message.
    fmt = git("log", "-1", "--format=%an%x00%ae%x00%ad%x00%B", tip)
    an, ae, ad, msg = fmt.split("\x00", 3)
    env1 = dict(os.environ, GIT_AUTHOR_NAME=an, GIT_AUTHOR_EMAIL=ae, GIT_AUTHOR_DATE=ad)
    c1 = git("commit-tree", tree1, "-p", base, "-F", "-", env=env1, input=msg).strip()

    # Commit 2: the hint comments; its tree is exactly the old tip's tree.
    num = re.search(r"Lesson (\d+)", msg)
    title = f"Lesson {num.group(1)}: add the hint comments" if num else "Add the lesson hint comments"
    msg2 = (
        f"{title}\n\n"
        "Only the `// EXERCISE n.m ...` comment blocks. lesson.sh checks out the\n"
        "commit below and applies this one as uncommitted changes, so the hints\n"
        "stand out in the participant's IDE.\n"
    )
    c2 = git("commit-tree", git("rev-parse", tip + "^{tree}").strip(), "-p", c1, "-F", "-", input=msg2).strip()
    git("update-ref", f"refs/heads/{branch}", c2, tip)
    print(f"{branch}: {total} hint lines, {tip[:7]} -> {c1[:7]} + {c2[:7]}")


if __name__ == "__main__":
    main()
