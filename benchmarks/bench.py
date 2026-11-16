"""
Benchmarks for strverify, emitted in the github-action-benchmark
"customSmallerIsBetter" format (time per call, lower is better).

Local usage:
    maturin develop --release
    python benchmarks/bench.py            # human-readable table
    python benchmarks/bench.py --json     # JSON for CI

CI usage (see .github/workflows/benchmark.yml):
    python benchmarks/bench.py --json > bench.json
"""
from __future__ import annotations

import argparse
import json
import sys
import timeit
from typing import Callable

from strverify import CharSet, Filter

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

    print(f"{'benchmark':<22}{'time (us)':>12}")
    print("-" * 34)
    for r in results:
        print(f"{r['name']:<22}{r['value']:>12.3f}")


if __name__ == "__main__":
    main()
