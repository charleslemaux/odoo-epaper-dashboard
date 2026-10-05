#!/usr/bin/env python3
"""Assert the Rust generated glyph data equals the C generated data."""
import re
import sys


def triples(text, name):
    block = re.search(name + r"[^=]*=\s*\{(.*?)\};", text, re.S) \
        or re.search(name + r"[^=]*=\s*\[(.*?)\];", text, re.S)
    body = block.group(1)
    c = re.findall(r"\{(\d+),\s*(\d+),\s*(\d+)\}", body)
    r = re.findall(r"offset:\s*(\d+),\s*width:\s*(\d+),\s*advance:\s*(\d+)",
                   body)
    return c or r


def hexbytes(text, name):
    block = re.search(name + r"[^=]*=\s*[\{\[](.*?)[\}\]];", text, re.S)
    return re.findall(r"0x[0-9a-fA-F]{2}", block.group(1))


def main():
    cf = open("src/gfx/font_data.c", encoding="utf-8").read()
    rf = open("rust/dashboard-core/src/font_data.rs", encoding="utf-8").read()
    ci = open("src/gfx/icon_data.c", encoding="utf-8").read()
    ri = open("rust/dashboard-core/src/icon_data.rs", encoding="utf-8").read()
    pairs = [(cf, rf, "Q32_GLYPHS"), (cf, rf, "Q48_GLYPHS"),
             (ci, ri, "ICON_GLYPHS")]
    for c, r, name in pairs:
        if triples(c, name) != triples(r, name):
            sys.exit("glyph mismatch: " + name)
    for c, r, name in [(cf, rf, "Q32_BITMAP"), (cf, rf, "Q48_BITMAP"),
                       (ci, ri, "ICON_BITMAP")]:
        if hexbytes(c, name) != hexbytes(r, name):
            sys.exit("bitmap mismatch: " + name)
    print("glyph parity: OK")


if __name__ == "__main__":
    main()
