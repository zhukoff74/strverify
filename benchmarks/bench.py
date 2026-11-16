"""
Benchmarks for strverify, emitted in the github-action-benchmark
"customSmallerIsBetter" format (time per call, lower is better).

The first group measures the original end-to-end cases: build a filter and
match a string. The comparison groups use prebuilt strverify filters,
compiled ``re`` patterns and (when installed) compiled ``regex`` patterns, so
they measure the matching path itself.

Local usage:
    maturin develop --release
    python benchmarks/bench.py            # human-readable table
    python benchmarks/bench.py --json     # JSON for CI

The third-party ``regex`` comparison is optional:
    python -m pip install regex           # or: maturin develop --release --extras bench

CI usage (see .github/workflows/benchmark.yml):
    python benchmarks/bench.py --json > bench.json
"""
from __future__ import annotations

import argparse
import json
import re
import sys
import timeit
from typing import Callable

from strverify import CharSet, Filter

try:
    import regex as _regex
except ImportError:  # optional benchmark dependency
    _regex = None

# (case_name, callable) — each callable performs one unit of work.
CASES: list[tuple[str, Callable[[], object]]] = [
    ("only_cyrillic",      lambda: Filter.only_cyrillic()("Привет мир")),
    ("only_latin",         lambda: Filter.only_latin()("Hello world")),
    ("only_digits",        lambda: Filter.only_digits()("1234567890")),
    ("only_digits_and",    lambda: Filter.only_digits_and(".,")("1234,50")),
    ("contains",           lambda: Filter.contains("!")("Привет!")),
    ("compose_and_not",    lambda: (
        Filter.only_cyrillic() & ~Filter.contains("!") & Filter.non_empty()
    )("Привет")),
    ("long_text_cyrillic", lambda: Filter.only_cyrillic()("Привет " * 1000)),
    ("long_text_no_match", lambda: Filter.only_cyrillic()("Hello " * 1000)),
]


# ---------------------------------------------------------------------------
# Comparison cases: prebuilt filters and precompiled regular expressions.
# ---------------------------------------------------------------------------
_SHORT_CYRILLIC = "Привет мир"
_SHORT_LATIN = "Hello world"
_SHORT_DIGITS = "1234567890"
_SHORT_DIGITS_AND = "1234,50"
_SHORT_BANG = "Привет!"
_SHORT_COMPOSE = "Привет"
_LONG_CYRILLIC = "Привет " * 1000
_LONG_NOT_CYRILLIC = "Hello " * 1000

# strverify filters, built once.
_F_ONLY_CYRILLIC = Filter.only_cyrillic()
_F_ONLY_LATIN = Filter.only_latin()
_F_ONLY_DIGITS = Filter.only_digits()
_F_ONLY_DIGITS_AND = Filter.only_digits_and(".,")
_F_CONTAINS = Filter.contains("!")
_F_COMPOSE = Filter.only_cyrillic() & ~Filter.contains("!") & Filter.non_empty()

# Python's `re` has no Unicode-script properties, so approximate the scripts
# with their main Unicode blocks. The optional `regex` module below uses the
# real `\p{...}` properties.
_RE_CYRILLIC = re.compile(r"[\u0400-\u052F\u2DE0-\u2DFF\uA640-\uA69F]*")
_RE_LATIN = re.compile(r"[A-Za-z\u00C0-\u024F\u1E00-\u1EFF]*")
_RE_DIGITS = re.compile(r"\d*")
_RE_DIGITS_AND = re.compile(r"[\d.,]*")
_RE_BANG = re.compile(r"!")


def _re_compose(text: str) -> bool:
    return (
        _RE_CYRILLIC.fullmatch(text) is not None
        and _RE_BANG.search(text) is None
        and bool(text)
    )


CASES += [
    ("strverify_prebuilt:only_cyrillic", lambda: _F_ONLY_CYRILLIC(_SHORT_CYRILLIC)),
    ("strverify_prebuilt:only_latin", lambda: _F_ONLY_LATIN(_SHORT_LATIN)),
    ("strverify_prebuilt:only_digits", lambda: _F_ONLY_DIGITS(_SHORT_DIGITS)),
    ("strverify_prebuilt:only_digits_and", lambda: _F_ONLY_DIGITS_AND(_SHORT_DIGITS_AND)),
    ("strverify_prebuilt:contains", lambda: _F_CONTAINS(_SHORT_BANG)),
    ("strverify_prebuilt:compose_and_not", lambda: _F_COMPOSE(_SHORT_COMPOSE)),
    ("strverify_prebuilt:long_text_cyrillic", lambda: _F_ONLY_CYRILLIC(_LONG_CYRILLIC)),
    ("strverify_prebuilt:long_text_no_match", lambda: _F_ONLY_CYRILLIC(_LONG_NOT_CYRILLIC)),
    ("re:only_cyrillic", lambda: _RE_CYRILLIC.fullmatch(_SHORT_CYRILLIC) is not None),
    ("re:only_latin", lambda: _RE_LATIN.fullmatch(_SHORT_LATIN) is not None),
    ("re:only_digits", lambda: _RE_DIGITS.fullmatch(_SHORT_DIGITS) is not None),
    ("re:only_digits_and", lambda: _RE_DIGITS_AND.fullmatch(_SHORT_DIGITS_AND) is not None),
    ("re:contains", lambda: _RE_BANG.search(_SHORT_BANG) is not None),
    ("re:compose_and_not", lambda: _re_compose(_SHORT_COMPOSE)),
    ("re:long_text_cyrillic", lambda: _RE_CYRILLIC.fullmatch(_LONG_CYRILLIC) is not None),
    ("re:long_text_no_match", lambda: _RE_CYRILLIC.fullmatch(_LONG_NOT_CYRILLIC) is not None),
]

if _regex is not None:
    _REGEX_CYRILLIC = _regex.compile(r"\p{Cyrillic}*")
    _REGEX_LATIN = _regex.compile(r"\p{Latin}*")
    _REGEX_DIGITS = _regex.compile(r"\p{Nd}*")
    _REGEX_DIGITS_AND = _regex.compile(r"[\p{Nd}.,]*")
    _REGEX_BANG = _regex.compile(r"!")

    def _regex_compose(text: str) -> bool:
        return (
            _REGEX_CYRILLIC.fullmatch(text) is not None
            and _REGEX_BANG.search(text) is None
            and bool(text)
        )

    CASES += [
        ("regex:only_cyrillic", lambda: _REGEX_CYRILLIC.fullmatch(_SHORT_CYRILLIC) is not None),
        ("regex:only_latin", lambda: _REGEX_LATIN.fullmatch(_SHORT_LATIN) is not None),
        ("regex:only_digits", lambda: _REGEX_DIGITS.fullmatch(_SHORT_DIGITS) is not None),
        ("regex:only_digits_and", lambda: _REGEX_DIGITS_AND.fullmatch(_SHORT_DIGITS_AND) is not None),
        ("regex:contains", lambda: _REGEX_BANG.search(_SHORT_BANG) is not None),
        ("regex:compose_and_not", lambda: _regex_compose(_SHORT_COMPOSE)),
        ("regex:long_text_cyrillic", lambda: _REGEX_CYRILLIC.fullmatch(_LONG_CYRILLIC) is not None),
        ("regex:long_text_no_match", lambda: _REGEX_CYRILLIC.fullmatch(_LONG_NOT_CYRILLIC) is not None),
    ]

# How many times each callable is invoked per single timing sample.
NUMBER = 20_000
# How many independent samples to take; we keep the best (least noisy) one.
REPEAT = 5


def bench(fn: Callable[[], object]) -> float:
    """Return the best mean time of one call, in microseconds."""
    best = min(timeit.repeat(fn, number=NUMBER, repeat=REPEAT))
    return best / NUMBER * 1e6


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--json",
        action="store_true",
        help="emit JSON in the github-action-benchmark customSmallerIsBetter format",
    )
    args = parser.parse_args()

    results = [
        {"name": name, "unit": "us", "value": round(bench(fn), 3)}
        for name, fn in CASES
    ]

    if args.json:
        json.dump(results, sys.stdout, ensure_ascii=False, indent=2)
        sys.stdout.write("\n")
        return

    name_width = max(len("benchmark"), *(len(r["name"]) for r in results))
    print(f"{'benchmark':<{name_width}}  {'time (us)':>12}")
    print("-" * (name_width + 14))
    for r in results:
        print(f"{r['name']:<{name_width}}  {r['value']:>12.3f}")


if __name__ == "__main__":
    main()
