#!/usr/bin/env python3
"""Fail if a comment sits between an attribute and the item it annotates.

Comments and doc comments belong *above* an item's attributes, so the
attributes read directly against the item. Rust's own style guide says the same
("put doc comments before attributes"), but neither rustfmt nor clippy enforce
it, and it says nothing about plain `//` comments. This check covers both.

    Bad                          Good
    #[test]                      /// Frobnicate the widget.
    /// Frobnicate the widget.   #[test]
    fn frob() {}                 fn frob() {}

`#[doc = "..."]` attributes (what `///` desugars to) count as documentation, so
a doc comment following one is fine. Inner attributes (`#![...]`) and inner doc
comments (`//!`) at crate or module level keep their own ordering and are left
alone.

Usage:
    check.py [PATH ...]   # files or directories; defaults to `.`
Exits non-zero and prints `path:line` for every misplaced comment.
"""
import sys
from pathlib import Path


def violations(lines):
    after_attr = False   # the last thing seen was a real (non-doc) attribute
    depth = 0            # unbalanced brackets of a multi-line attribute
    doc = False          # ...and whether that attribute is a doc attribute
    for i, line in enumerate(lines):
        s = line.strip()
        if depth:                                  # inside a multi-line attribute
            depth += s.count("[") - s.count("]")
            if depth == 0:
                after_attr = not doc
        elif s.startswith("#["):                   # outer attribute (`#![...]` is left alone)
            doc = s.startswith("#[doc")
            depth = s.count("[") - s.count("]")
            if depth == 0:
                after_attr = not doc
        elif s.startswith("//"):                   # comment, doc comment, or inner doc comment
            if after_attr:
                yield i + 1, s
                after_attr = False                 # one report per comment block
        elif s:                                    # any other code ends the run
            after_attr = False


def rs_files(paths):
    for p in paths:
        p = Path(p)
        if p.is_dir():
            yield from (q for q in p.rglob("*.rs") if "target" not in q.parts)
        elif p.suffix == ".rs":
            yield p


def main(argv):
    total = 0
    for path in rs_files(argv[1:] or ["."]):
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        for lineno, text in violations(lines):
            print(f"{path}:{lineno}: comment must go above the attribute, not between it and the item: {text}")
            total += 1
    if total:
        print(f"\n{total} misplaced comment(s); move each above its attribute.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
