#!/usr/bin/env python3
"""Heuristic Epitech C style audit for src/, include/ and tests/."""
import re
import sys
from pathlib import Path

HEADER_RE = re.compile(
    r"\A/\*\n\*\* Charles Le Maux, \d{4}\n\*\* .+\n"
    r"\*\* File description:\n(\*\* .+\n)+\*/\n")
FUNC_RE = re.compile(r"^[a-z]", re.M)

def file_errors(path):
    errors = []
    raw = path.read_bytes()
    if b"\r" in raw:
        errors.append("CRLF line ending")
    text = raw.decode("utf-8", errors="replace")
    if path.suffix in (".c", ".h") and not HEADER_RE.match(text):
        errors.append("missing or malformed file header")
    for i, line in enumerate(text.split("\n"), 1):
        if len(line) > 80:
            errors.append(f"line {i}: over 80 columns")
        if line != line.rstrip():
            errors.append(f"line {i}: trailing whitespace")
        if "\t" in line and path.suffix in (".c", ".h"):
            errors.append(f"line {i}: tab character")
    if path.suffix == ".c":
        errors += function_errors(text)
    return errors

def function_errors(text):
    errors = []
    lines = text.split("\n")
    exported = total = 0
    i = 0
    while i < len(lines):
        if lines[i] == "{" and i > 0 and "(" in lines[i - 1] \
                and not lines[i - 1].startswith(" "):
            total += 1
            sig = " ".join(lines[max(0, i - 3):i])
            if "static" not in sig.split("(")[0]:
                exported += 1
            depth, body = 1, 0
            j = i + 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count("{") - lines[j].count("}")
                body += 1
                j += 1
            if body - 1 > 20:
                errors.append(f"function ending line {j}: body > 20 lines")
            i = j
        else:
            i += 1
    if exported > 5:
        errors.append(f"{exported} exported functions (max 5)")
    if total > 10:
        errors.append(f"{total} total functions (max 10)")
    return errors

def main():
    bad = 0
    for folder in ("src", "include", "tests"):
        for path in sorted(Path(folder).glob("**/*")):
            if path.suffix not in (".c", ".h") \
                    or "test_config" in str(path):
                pass
            if path.suffix not in (".c", ".h"):
                continue
            errors = file_errors(path)
            for err in errors:
                print(f"{path}: {err}")
            bad += len(errors)
    print("style: OK" if bad == 0 else f"style: {bad} issue(s)")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main())
