# strverify

Быстрая библиотека для проверки содержимого строк с ядром на Rust и Python API.
Фильтры неизменяемые и комбинируются логическими операторами `&`, `|`, `~`.

[English](../README.md) | **Русский** | [中文](README.zh.md)

> **Имена пакетов.** Дистрибутив на PyPI и импортируемый пакет называются
> одинаково — **`strverify`**.

## Установка

```bash
pip install strverify
```

Требуется Python 3.9+. Колёса собираются как `abi3`, поэтому один wheel
работает на всех поддерживаемых версиях CPython.

## Быстрый старт

```python
from strverify import CharSet, Filter

Filter.only_cyrillic()("Привет")          # True
Filter.only_cyrillic()("Привет!")         # False
Filter.only_latin()("Hello")              # True

# Либо целиком кириллица, либо целиком латиница:
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

Каждый фильтр можно вызвать как функцию (`filter_obj(text)`) или через
`.matches(text)`, поэтому он работает с `filter()` и любым кодом, ожидающим
предикат.

## Комбинирование фильтров

Предикаты комбинируются операторами:

| Оператор | Значение |
|----------|----------|
| `a & b`  | И: строка должна удовлетворять обоим фильтрам |
| `a \| b` | ИЛИ: достаточно одного |
| `~a`     | НЕ: инверсия фильтра |

```python
from strverify import Filter

f = Filter.only_cyrillic() & ~Filter.contains("!") & Filter.non_empty()

f("Привет")     # True
f("Привет!")    # False
f("")           # False
f("Hello")      # False
```

Есть функциональные аналоги для тех, кто предпочитает явные вызовы:

```python
Filter.all_of(Filter.only_cyrillic(), Filter.non_empty())
Filter.any_of(Filter.contains("!"), Filter.contains("?"))
```

`Filter.all_of()` без аргументов принимает всё, `Filter.any_of()` без
аргументов не принимает ничего.

> **Важно:** используйте `&`, `|`, `~`, а не питоновские `and`, `or`, `not`.
> У фильтров намеренно нет истинностного значения: `bool(Filter...)` выбрасывает
> `TypeError`, чтобы случайная запись `f1 and f2` не привела к тихой ошибке.

`~` — это логическое отрицание всего предиката, а не замена одной операции
другой:

```python
~Filter.only_digits()   # отрицание условия «все символы — цифры»
Filter.no_digits()      # «ни одной цифры»
```

### Разница между классами символов и предикатами

`CharSet` описывает **множество допустимых символов**, а `Filter` — **условие
над строкой целиком**. Это разные операции, и их нельзя путать:

```python
from strverify import CharSet, Filter

# Правильно: множество символов = цифры ИЛИ кириллица
f = Filter.only(CharSet.DIGITS | CharSet.CYRILLIC)
f("Привет123")   # True

# Это НЕ то же самое: "вся строка состоит только из цифр" ИЛИ
# "вся строка состоит только из кириллицы"
g = Filter.only_digits() | Filter.only_cyrillic()
g("Привет123")   # False
```

`&` двух `only`-фильтров тоже не объединяет классы: он требует, чтобы
выполнились оба условия. Для объединения классов используйте `CharSet` и один
`Filter.only`.

Для частых случаев есть готовые пресеты:
`Filter.only_digits_and_cyrillic()` и `Filter.only_digits_and(".,")`.

## Классы символов

`CharSet` — неизменяемое множество. Объединяется оператором `|`:

```python
allowed = CharSet.DIGITS | CharSet.of(".,_")
assert "5" in allowed
assert "_" in allowed
assert "a" not in allowed
```

| Класс | Что означает |
|-------|--------------|
| `CharSet.CYRILLIC` | Unicode-скрипт Cyrillic |
| `CharSet.LATIN` | Unicode-скрипт Latin |
| `CharSet.LETTERS` | любые Unicode-буквы (`L*`) |
| `CharSet.DIGITS` | любые десятичные цифры Unicode (`Nd`), включая `0-9` |
| `CharSet.ASCII_DIGITS` | только ASCII-цифры `0-9` |
| `CharSet.PUNCTUATION` | Unicode-пунктуация (`P*`) |
| `CharSet.SYMBOLS` | Unicode-символы (`S*`): `+`, `€`, `😀` |
| `CharSet.WHITESPACE` | Unicode-пробельные символы |
| `CharSet.ASCII` | все ASCII-символы `U+0000..U+007F` |

`CharSet.of("...")` добавляет конкретные символы. `CharSet.empty()` создаёт
пустое множество.

Например, кириллица с пробелами и дефисами:

```python
Filter.only(CharSet.CYRILLIC | CharSet.of(" -"))
```

`Filter.only_symbols()` — это пресет над `PUNCTUATION | SYMBOLS`; пробелы в него
не входят. Если нужны пробелы:

```python
Filter.only(CharSet.PUNCTUATION | CharSet.SYMBOLS | CharSet.WHITESPACE)
```

## Пустая строка

Фильтры классов символов формально принимают пустую строку: в ней нет символов,
нарушающих ограничение. `Filter.no_digits()`, `Filter.only_digits()` и
`Filter.only_symbols()` на `""` возвращают `True`, `Filter.contains("!")` —
`False`.

Если пустая строка недопустима, добавьте `Filter.non_empty()`:

```python
f = Filter.only_digits() & Filter.non_empty()
f("")     # False
f("123")  # True
```

## API

### `Filter`

Во всех методах с параметром `chars` можно передать либо `str` (множество
литеральных символов), либо `CharSet`.

Создание:

- `Filter.only(chars)` — все символы строки принадлежат `chars` (строка или `CharSet`).
- `Filter.none_of(chars)` — ни одного символа из `chars`.
- `Filter.contains(chars)` — есть хотя бы один символ из `chars`.
- `Filter.non_empty()` — строка не пустая.
- `Filter.always()` / `Filter.never()` — тавтология и противоречие.
- `Filter.only_cyrillic()`, `Filter.only_latin()`, `Filter.only_digits()`,
  `Filter.no_digits()`, `Filter.only_digits_and(chars)`,
  `Filter.only_digits_and_cyrillic()`, `Filter.only_digits_and_underscore()`,
  `Filter.only_symbols()`.
- `Filter.all_of(*filters)`, `Filter.any_of(*filters)`.

Проверка и комбинирование:

- `f.matches(text)` и `f(text)`.
- `f & g`, `f | g`, `~f`.
- `repr(f)` показывает дерево выражения.

### `CharSet`

- Константы из таблицы выше.
- `CharSet.of(chars)` — литеральные символы.
- `CharSet.empty()`.
- `set | set` — объединение.
- `"x" in set` — проверка одного символа.

## Разработка

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest

maturin develop --release
pytest
cargo test
```

Сборка wheel и sdist:

```bash
maturin build --release
maturin sdist
```

Готовые артефакты появятся в `target/wheels/`.

Публикация на PyPI (нужен токен в `MATURIN_PYPI_TOKEN` или интерактивный ввод):

```bash
maturin publish
```

Сначала полезно проверить пакет на TestPyPI:

```bash
maturin publish --repository testpypi
```

## Лицензия

MIT. См. [LICENSE](../LICENSE).
