#!/usr/bin/env python3
"""Normalize text file line endings to LF in a staged IPK tree.

Windows checkouts can introduce CRLF endings into shell scripts, Python files,
and config files. Those break on Linux/RT targets unless converted before the
IPK is packaged.

Usage:
    normalize_ipk_line_endings.py <ipk_root>
    normalize_ipk_line_endings.py <ipk_root> --verify

In normal mode, text files are converted to LF and the build fails if any
text file still contains carriage returns afterward. In verify mode, no files
are modified; the command exits with an error if CRLF is detected.
"""

from __future__ import annotations

import sys
from pathlib import Path

# File extensions treated as text and normalized when present in the IPK tree.
TEXT_SUFFIXES = {
    ".conf",
    ".control",
    ".h",
    ".info",
    ".ini",
    ".json",
    ".log",
    ".md",
    ".proto",
    ".py",
    ".pth",
    ".pyi",
    ".sh",
    ".txt",
}

# Extensionless Linux/deployed files that must remain text with LF endings.
TEXT_BASENAMES = {
    ".gitkeep",
    "control",
    "debian-binary",
    "postinst",
    "prerm",
}

# Never modify known binary artifacts even if they contain byte patterns
# that resemble CRLF sequences.
BINARY_SUFFIXES = {
    ".deb",
    ".dll",
    ".exe",
    ".gif",
    ".gz",
    ".ico",
    ".ipk",
    ".jpeg",
    ".jpg",
    ".lvlibp",
    ".pdf",
    ".png",
    ".pyc",
    ".rtexe",
    ".so",
    ".tar",
    ".zip",
}


def should_normalize(path: Path) -> bool:
    """Return True when a file should be treated as text for LF normalization."""
    suffix = path.suffix.lower()
    if suffix in BINARY_SUFFIXES:
        return False
    if path.name in TEXT_BASENAMES:
        return True
    if suffix in TEXT_SUFFIXES:
        return True
    if path.parent.name in {"control", "CONTROL"}:
        return True
    return False


def normalize_bytes(data: bytes) -> bytes | None:
    """Convert CRLF/CR to LF. Return None when no change is needed or file is binary."""
    if b"\x00" in data:
        return None

    normalized = data.replace(b"\r\n", b"\n").replace(b"\r", b"\n")
    if normalized == data:
        return None
    return normalized


def normalize_tree(root: Path) -> list[str]:
    """Normalize all eligible text files under ipk_root. Return changed relative paths."""
    changed: list[str] = []

    for path in sorted(root.rglob("*")):
        if not path.is_file() or not should_normalize(path):
            continue

        original = path.read_bytes()
        normalized = normalize_bytes(original)
        if normalized is None:
            continue

        path.write_bytes(normalized)
        changed.append(str(path.relative_to(root)))

    return changed


def find_crlf_text_files(root: Path) -> list[str]:
    """Return text files that still contain carriage returns."""
    offenders: list[str] = []

    for path in sorted(root.rglob("*")):
        if not path.is_file() or not should_normalize(path):
            continue
        if b"\r" in path.read_bytes():
            offenders.append(str(path.relative_to(root)))

    return offenders


def main() -> int:
    if len(sys.argv) not in {2, 3}:
        print(
            "Usage: normalize_ipk_line_endings.py <ipk_root> [--verify]",
            file=sys.stderr,
        )
        return 1

    root = Path(sys.argv[1]).resolve()
    verify_only = len(sys.argv) == 3 and sys.argv[2] == "--verify"

    if not root.is_dir():
        print(f"Error: IPK root not found: {root}", file=sys.stderr)
        return 1

    if verify_only:
        offenders = find_crlf_text_files(root)
        if offenders:
            print("Error: CRLF line endings found in text files:", file=sys.stderr)
            for path in offenders:
                print(f"  {path}", file=sys.stderr)
            return 1
        print("All text files use LF line endings.")
        return 0

    changed = normalize_tree(root)
    if changed:
        print("Normalized line endings to LF:")
        for path in changed:
            print(f"  {path}")
    else:
        print("All text files already use LF line endings.")

    offenders = find_crlf_text_files(root)
    if offenders:
        print("Error: CRLF remains after normalization:", file=sys.stderr)
        for path in offenders:
            print(f"  {path}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
