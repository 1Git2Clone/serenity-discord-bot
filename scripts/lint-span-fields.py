#!/usr/bin/env python3
"""Reject unsigned integer casts inside `#[tracing::instrument]` field lists.

tracing-opentelemetry's span visitor implements `record_i64` and no
`record_u64`, so a `u64` field falls through the `Visit` trait's default to
`record_debug` and reaches the trace backend as a STRING. Nothing fails: the
attribute is present, looks right in a trace view, and every `sum_over_time()`
over it silently returns an empty series.

That is a bad bug to find by hand twice, so it is a lint. `as i64` is the fix.

Run: python3 scripts/lint-span-fields.py [paths...]   (default: src/)
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ATTR = "#[tracing::instrument("
BAD = re.compile(r"\bas\s+(u8|u16|u32|u64|u128|usize)\b")


def attribute_spans(text: str):
    """Yield (start, end) for each instrument attribute, paren-balanced.

    Regex cannot do this: field lists nest parens (`map(GuildId::get)`) and
    routinely span a dozen lines.
    """
    i = 0
    while (i := text.find(ATTR, i)) != -1:
        depth, j = 0, i + len(ATTR) - 1
        while j < len(text):
            if text[j] == "(":
                depth += 1
            elif text[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        yield i, min(j + 1, len(text))
        i = j + 1


def check(path: Path) -> list[str]:
    text = path.read_text()
    problems = []
    for start, end in attribute_spans(text):
        for m in BAD.finditer(text, start, end):
            line = text.count("\n", 0, m.start()) + 1
            problems.append(
                f"{path}:{line}: `{m.group(0)}` in a span field — "
                f"use `as i64` (tracing-opentelemetry has no record_u64, so an "
                f"unsigned value is recorded as a string)"
            )
    return problems


def self_test() -> None:
    import tempfile

    bad = 'x\n#[tracing::instrument(\n    fields(n = v.len() as u64, g = a.map(G::get))\n)]\nfn f() {}\n'
    good = bad.replace("as u64", "as i64")
    # a cast OUTSIDE any attribute is none of this lint's business
    outside = 'fn f() { let n = v.len() as u64; }\n'
    with tempfile.TemporaryDirectory() as d:
        for name, src, want in (("bad", bad, 1), ("good", good, 0), ("out", outside, 0)):
            p = Path(d) / f"{name}.rs"
            p.write_text(src)
            got = len(check(p))
            assert got == want, f"{name}: expected {want} problem(s), got {got}"
    print("self-test ok")


def main() -> int:
    args = sys.argv[1:]
    if args == ["--self-test"]:
        self_test()
        return 0

    roots = [Path(a) for a in args] or [Path("src")]
    files = sorted(p for r in roots for p in ([r] if r.is_file() else r.rglob("*.rs")))

    problems = [p for f in files for p in check(f)]
    for p in problems:
        print(p, file=sys.stderr)
    if problems:
        print(f"\n{len(problems)} problem(s) in {len(files)} file(s)", file=sys.stderr)
        return 1
    print(f"span fields ok ({len(files)} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
