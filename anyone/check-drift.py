#!/usr/bin/env python3
"""Check that this fork differs from upstream Arti only where anyone/README.md says it does.

Fails when a file differs from the latest upstream release tag (arti-v*) in the history of HEAD
without being listed in a table of anyone/README.md, or when a commit that no upstream release
tag contains has no signature.
"""

import fnmatch
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OURS = ["anyone/*", ".github/*", "Dockerfile", ".dockerignore"]
LISTED = re.compile(r"^\| `([^`]+)` \|", re.M)


def git(*args):
    return subprocess.run(
        ["git", *args], cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout


def main():
    tags = git("tag", "--list", "arti-v*", "--merged", "HEAD").split()
    if not tags:
        sys.exit("no upstream arti-v* tag in the history of HEAD; fetch the upstream tags first")
    base = git("describe", "--tags", "--abbrev=0", "--match", "arti-v*", "HEAD").strip()
    patterns = OURS + LISTED.findall((ROOT / "anyone" / "README.md").read_text())

    changed = [f for f in git("diff", "--name-only", "-z", base, "HEAD").split("\0") if f]
    unlisted = [f for f in changed if not any(fnmatch.fnmatchcase(f, p) for p in patterns)]

    ours = git("rev-list", "HEAD", "--not", *tags).split()
    unsigned = [c for c in ours if git("log", "-1", "--format=%G?", c).strip() == "N"]

    print(f"{len(changed)} files differ from {base}, {len(ours)} commits are not from an upstream tag")
    for f in unlisted:
        print(f"not listed in anyone/README.md: {f}", file=sys.stderr)
    for c in unsigned:
        print(f"unsigned commit: {git('log', '-1', '--format=%h %s', c).strip()}", file=sys.stderr)
    return 1 if unlisted or unsigned else 0


if __name__ == "__main__":
    sys.exit(main())
