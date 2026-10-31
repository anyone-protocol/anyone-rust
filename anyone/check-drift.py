#!/usr/bin/env python3
"""Check that this fork differs from upstream Arti only where anyone/README.md says it does.

Fails when a file differs from the latest upstream release tag (arti-v*) in the history of HEAD
without being listed in a table of anyone/README.md, or when a commit that no upstream release
tag contains has no signature or a bad one. Upstream tags are read from refs/upstream-tags/, so
a tag in this repository cannot stand in for one.
"""

import fnmatch
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OURS = ["anyone/*", ".github/*", "Dockerfile", ".dockerignore"]
LISTED = re.compile(r"^\| `([^`]+)` \|", re.M)
TAGS = "refs/upstream-tags/"
FETCH = (
    "git fetch --no-tags https://gitlab.torproject.org/tpo/core/arti.git"
    ' "+refs/tags/arti-v*:refs/upstream-tags/arti-v*"'
)


def git(*args):
    return subprocess.run(
        ["git", *args], cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout


def signed(commit):
    header = git("cat-file", "commit", commit).split("\n\n", 1)[0]
    return re.search(r"^gpgsig(-sha256)? ", header, re.M) is not None


def main():
    tags = git(
        "for-each-ref", "--merged", "HEAD", "--sort=-v:refname", "--format=%(refname)", TAGS + "arti-v*"
    ).split()
    if not tags:
        sys.exit(f"no upstream arti-v* tag in the history of HEAD; fetch them first:\n{FETCH}")
    base = tags[0]
    patterns = OURS + LISTED.findall((ROOT / "anyone" / "README.md").read_text())

    diff = git("diff", "--name-only", "--no-renames", "-z", base, "HEAD")
    changed = [f for f in diff.split("\0") if f]
    unlisted = [f for f in changed if not any(fnmatch.fnmatchcase(f, p) for p in patterns)]

    ours = git("rev-list", "HEAD", "--not", *tags).split()
    unsigned = [c for c in ours if not signed(c)]
    bad = [c for c in ours if git("log", "-1", "--format=%G?", c).strip() == "B"]

    print(f"{len(changed)} files differ from {base}, {len(ours)} commits are not from an upstream tag")
    for f in unlisted:
        print(f"not listed in anyone/README.md: {f}", file=sys.stderr)
    for kind, commits in (("unsigned", unsigned), ("bad signature on", bad)):
        for c in commits:
            print(f"{kind} commit: {git('log', '-1', '--format=%h %s', c).strip()}", file=sys.stderr)
    return 1 if unlisted or unsigned or bad else 0


if __name__ == "__main__":
    sys.exit(main())
