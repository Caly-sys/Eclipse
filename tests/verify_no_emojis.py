#!/usr/bin/env python3
"""
Zero-Emoji Compliance Verification Script for Umbra / Eclipse.
Scans all Rust source files in src/ to ensure NO Unicode emojis are present.
All iconography must use vector / egui painted primitives (ui::icons), not emojis.
"""

import os
import sys

EMOJI_RANGES = [
    (0x1F600, 0x1F64F),  # Emoticons
    (0x1F300, 0x1F5FF),  # Misc Symbols and Pictographs
    (0x1F680, 0x1F6FF),  # Transport and Map
    (0x1F1E0, 0x1F1FF),  # Regional Indicator Symbols
    (0x1F900, 0x1F9FF),  # Supplemental Symbols and Pictographs
    (0x1FA70, 0x1FAFF),  # Symbols and Pictographs Extended-A
    (0x2600, 0x26FF),    # Misc symbols
    (0x2700, 0x27BF),    # Dingbats
    (0xFE00, 0xFE0F),    # Variation Selectors
    (0x1F000, 0x1F02F),  # Mahjong Tiles
    (0x1F0A0, 0x1F0FF),  # Playing Cards
    (0x2300, 0x23FF),    # Misc Technical
    (0x2B50, 0x2B50),    # Star
]

def is_emoji(cp: int) -> bool:
    for start, end in EMOJI_RANGES:
        if start <= cp <= end:
            return True
    return False

def main():
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    src_dir = os.path.join(repo_root, "src")
    if not os.path.exists(src_dir):
        print(f"Error: src directory not found at {src_dir}")
        sys.exit(1)

    violations = []
    scanned_files = 0

    for root, _, files in os.walk(src_dir):
        for f in files:
            if f.endswith(".rs"):
                scanned_files += 1
                path = os.path.join(root, f)
                rel_path = os.path.relpath(path, repo_root)
                with open(path, "r", encoding="utf-8", errors="replace") as fp:
                    for line_no, line in enumerate(fp, 1):
                        for col_no, ch in enumerate(line, 1):
                            cp = ord(ch)
                            if is_emoji(cp):
                                violations.append((rel_path, line_no, col_no, ch, hex(cp)))

    print(f"Scanned {scanned_files} Rust files in {src_dir}.")
    if violations:
        print(f"FAIL: Found {len(violations)} emoji violation(s):")
        for v in violations:
            print(f"  {v[0]}:{v[1]}:{v[2]}: '{v[3]}' (Unicode {v[4]})")
        sys.exit(1)
    else:
        print("PASS: Verified 0 Unicode emojis across all .rs files in src/.")
        sys.exit(0)

if __name__ == "__main__":
    main()
