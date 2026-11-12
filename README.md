# strverify

A fast string-content validation library with a Rust core and a Python API.
Filters are immutable and compose with the logical operators `&`, `|`, and `~`.

**English** | [Русский](docs/README.ru.md) | [中文](docs/README.zh.md)

> **Package names.** The PyPI distribution and the importable package share the
> same name: **`strverify`**.

## Installation

```bash
pip install strverify
```

Python 3.9+ is required. Wheels are built as `abi3`, so a single wheel works
on every supported CPython version.

## Quick start

```python
from strverify import CharSet, Filter

Filter.only_cyrillic()("Привет")          # True
Filter.only_cyrillic()("Привет!")         # False
Filter.only_latin()("Hello")              # True

# Either entirely Cyrillic or entirely Latin:
(Filter.only_cyrillic() | Filter.only_latin())("Привет")  # True

Filter.only_digits()("12345")             # True
Filter.only_digits()("12.3")              # False

Filter.no_digits()("Привет")              # True
Filter.no_digits()("Привет1")             # False

Filter.only_digits_and(".,")("1 234,50")  # False
Filter.only_digits_and(".,")("1234,50")   # True

Filter.contains("!")("Привет!")           # True
Filter.contains("!")("Привет")            # False

Filter.only_digits_and_cyrillic()("Привет123")   # True
Filter.only_digits_and_cyrillic()("Hello123")    # False

Filter.only_symbols()("!@#$%^&*")         # True
Filter.only_symbols()("Hello!")           # False

Filter.only_digits_and_underscore()("12_34")     # True
Filter.only_digits_and_underscore()("12.34")     # False
```

Every filter can be called like a function (`filter_obj(text)`) or via
`.matches(text)`, so it works with `filter()` and any code that expects a
predicate.

## Combining filters

Predicates compose with operators:

| Operator | Meaning |
|----------|---------|
| `a & b`  | AND: the string must satisfy both filters |
| `a \| b` | OR: satisfying either one is enough |
| `~a`     | NOT: inverts the filter |

```python
from strverify import Filter

f = Filter.only_cyrillic() & ~Filter.contains("!") & Filter.non_empty()

f("Привет")     # True
f("Привет!")    # False
f("")           # False
f("Hello")      # False
```

Function-style equivalents are available for those who prefer explicit calls:

```python
Filter.all_of(Filter.only_cyrillic(), Filter.non_empty())
Filter.any_of(Filter.contains("!"), Filter.contains("?"))
```

`Filter.all_of()` with no arguments accepts everything, while
`Filter.any_of()` with no arguments accepts nothing.

> **Important:** use `&`, `|`, and `~`, not Python's `and`, `or`, and `not`.
> Filters intentionally have no truth value: `bool(Filter...)` raises
> `TypeError`, so an accidental `f1 and f2` cannot fail silently.

`~` is the logical negation of the entire predicate, not a swapped operation:

```python
~Filter.only_digits()   # negation of "all characters are digits"
Filter.no_digits()      # "no digits at all"
```

### Character classes vs. predicates

`CharSet` describes a **set of allowed characters**, while `Filter` describes a
**condition on the whole string**. These are different operations and must not
be confused:

```python
from strverify import CharSet, Filter

# Correct: the character set is digits OR Cyrillic
f = Filter.only(CharSet.DIGITS | CharSet.CYRILLIC)
f("Привет123")   # True

# This is NOT the same: "the whole string consists only of digits" OR
# "the whole string consists only of Cyrillic"
g = Filter.only_digits() | Filter.only_cyrillic()
g("Привет123")   # False
```

Combining two `only` filters with `&` does not merge their classes either: it
requires both conditions to hold. To combine character classes, use `CharSet`
and a single `Filter.only`.

Ready-made presets cover common cases:
`Filter.only_digits_and_cyrillic()` and `Filter.only_digits_and(".,")`.

## Character sets

`CharSet` is an immutable set. Sets are combined with the `|` operator:

```python
allowed = CharSet.DIGITS | CharSet.of(".,_")
assert "5" in allowed
assert "_" in allowed
assert "a" not in allowed
```

| Class | Meaning |
|-------|---------|
| `CharSet.CYRILLIC` | Unicode Cyrillic script |
| `CharSet.LATIN` | Unicode Latin script |
| `CharSet.LETTERS` | any Unicode letters (`L*`) |
| `CharSet.DIGITS` | any Unicode decimal digits (`Nd`), including `0-9` |
| `CharSet.ASCII_DIGITS` | ASCII digits `0-9` only |
| `CharSet.PUNCTUATION` | Unicode punctuation (`P*`) |
| `CharSet.SYMBOLS` | Unicode symbols (`S*`): `+`, `€`, `😀` |
| `CharSet.WHITESPACE` | Unicode whitespace characters |
| `CharSet.ASCII` | all ASCII characters `U+0000..U+007F` |

`CharSet.of("...")` adds specific characters. `CharSet.empty()` creates an
empty set.

For example, Cyrillic with spaces and hyphens:

```python
Filter.only(CharSet.CYRILLIC | CharSet.of(" -"))
```

`Filter.only_symbols()` is a preset over `PUNCTUATION | SYMBOLS`; it does not
include whitespace. If you need spaces:

```python
Filter.only(CharSet.PUNCTUATION | CharSet.SYMBOLS | CharSet.WHITESPACE)
```

## Empty strings

Character-class filters formally accept an empty string: it contains no
characters that violate the constraint. `Filter.no_digits()`,
`Filter.only_digits()`, and `Filter.only_symbols()` return `True` for `""`,
while `Filter.contains("!")` returns `False`.

If an empty string is not acceptable, add `Filter.non_empty()`:

```python
f = Filter.only_digits() & Filter.non_empty()
f("")     # False
f("123")  # True
```

## API

### `Filter`

In every method with a `chars` parameter, you can pass either a `str` (a set of
literal characters) or a `CharSet`.

Construction:

- `Filter.only(chars)` — every character in the string belongs to `chars`
  (a string or a `CharSet`).
- `Filter.none_of(chars)` — no characters from `chars`.
- `Filter.contains(chars)` — at least one character from `chars`.
- `Filter.non_empty()` — the string is not empty.
- `Filter.always()` / `Filter.never()` — tautology and contradiction.
- `Filter.only_cyrillic()`, `Filter.only_latin()`, `Filter.only_digits()`,
  `Filter.no_digits()`, `Filter.only_digits_and(chars)`,
  `Filter.only_digits_and_cyrillic()`, `Filter.only_digits_and_underscore()`,
  `Filter.only_symbols()`.
- `Filter.all_of(*filters)`, `Filter.any_of(*filters)`.

Matching and combining:

- `f.matches(text)` and `f(text)`.
- `f & g`, `f | g`, `~f`.
- `repr(f)` shows the expression tree.

### `CharSet`

- The constants from the table above.
- `CharSet.of(chars)` — literal characters.
- `CharSet.empty()`.
- `set | set` — union.
- `"x" in set` — membership test for a single character.

## Development

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest

maturin develop --release
pytest
cargo test
```

Building a wheel and an sdist:

```bash
maturin build --release
maturin sdist
```

The artifacts appear in `target/wheels/`.

Publishing to PyPI (requires a token in `MATURIN_PYPI_TOKEN` or interactive
input):

```bash
maturin publish
```

It is a good idea to test the package on TestPyPI first:

```bash
maturin publish --repository testpypi
```

## License

MIT. See [LICENSE](LICENSE).
