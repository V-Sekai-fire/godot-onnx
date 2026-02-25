#!/usr/bin/env python3
# Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
"""
Check that tracked source files contain an SPDX license identifier.
Run with --add to insert the header into files that lack it.
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

COPYRIGHT_LINE = "Copyright (c) 2026-present K. S. Ernest (iFire) Lee & godot-onnx contributors"
SPDX_LINE = "SPDX-License-Identifier: Apache-2.0 OR MIT"
ROOT = Path(__file__).resolve().parent.parent

# Extensions we care about and the comment prefix for the SPDX line
LICENSED_EXTENSIONS = {
    ".rs": "//",
    ".gd": "#",
    ".py": "#",
    ".ps1": "#",
    ".sh": "#",
}


def get_tracked_files() -> list[Path]:
    out = subprocess.run(
        ["git", "ls-files", "--cached"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    paths = [ROOT / p for p in out.stdout.strip().splitlines() if p]
    return [p for p in paths if p.suffix in LICENSED_EXTENSIONS and "target" not in p.parts]


def has_spdx(path: Path) -> bool:
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            for _ in range(25):
                line = f.readline()
                if not line:
                    break
                if "SPDX-License-Identifier" in line and "Apache-2.0 OR MIT" in line:
                    return True
    except OSError:
        pass
    return False


def add_header(path: Path) -> bool:
    prefix = LICENSED_EXTENSIONS[path.suffix]
    header_block = f"{prefix} {COPYRIGHT_LINE}\n{prefix} {SPDX_LINE}\n"
    try:
        raw = path.read_bytes()
        text = raw.decode("utf-8", errors="replace")
    except OSError:
        return False

    if "SPDX-License-Identifier" in text[:2000]:
        return False  # Already present

    lines = text.splitlines(keepends=True)
    insert_at = 0
    # Put header after shebang and optional encoding line
    if lines and re.match(r"^#!.*\n", lines[0]):
        insert_at = 1
    if insert_at < len(lines) and re.match(r"^#.*coding[:=]", lines[insert_at], re.I):
        insert_at += 1

    if insert_at == 0:
        new_content = header_block + text
    else:
        before = "".join(lines[:insert_at])
        after = "".join(lines[insert_at:])
        new_content = before + header_block + after

    path.write_text(new_content, encoding="utf-8", newline="")
    return True


def main() -> int:
    ap = argparse.ArgumentParser(description="Check or add SPDX license headers.")
    ap.add_argument("--add", action="store_true", help="Add header to files that lack it.")
    args = ap.parse_args()

    files = get_tracked_files()
    missing = [p for p in files if not has_spdx(p)]

    if not missing:
        return 0

    if args.add:
        for p in missing:
            if add_header(p):
                print(f"Added license header: {p.relative_to(ROOT)}")
        return 0

    print("The following files are missing an SPDX license header (Apache-2.0 OR MIT):", file=sys.stderr)
    for p in sorted(missing):
        print(f"  {p.relative_to(ROOT)}", file=sys.stderr)
    print("Run: python misc/check-license-headers.py --add", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
