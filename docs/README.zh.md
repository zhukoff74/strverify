# strverify

一个快速校验字符串内容的库，核心用 Rust 实现，并提供 Python API。
过滤器是不可变的，可以用逻辑运算符 `&`、`|` 和 `~` 组合。

[English](../README.md) | [Русский](README.ru.md) | **中文**

> **包名。** PyPI 上的发行版和可导入的包同名——**`strverify`**。

## 安装

```bash
pip install strverify
```

需要 Python 3.9+。wheel 以 `abi3` 方式构建，因此同一个 wheel
可用于所有受支持的 CPython 版本。

## 快速开始

```python
from strverify import CharSet, Filter

Filter.only_cyrillic()("Привет")          # True
Filter.only_cyrillic()("Привет!")         # False
Filter.only_latin()("Hello")              # True

# 要么整串都是西里尔字母，要么整串都是拉丁字母：
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

每个过滤器既可以像函数一样调用（`filter_obj(text)`），也可以通过
`.matches(text)` 调用，因此可用于 `filter()` 以及任何接受谓词的代码。

## 组合过滤器

谓词可以用运算符组合：

| 运算符 | 含义 |
|--------|------|
| `a & b`  | 与：字符串必须同时满足两个过滤器 |
| `a \| b` | 或：满足其中一个即可 |
| `~a`     | 非：对过滤器取反 |

```python
from strverify import Filter

f = Filter.only_cyrillic() & ~Filter.contains("!") & Filter.non_empty()

f("Привет")     # True
f("Привет!")    # False
f("")           # False
f("Hello")      # False
```

对于偏好显式调用的用户，也提供了函数形式的等价写法：

```python
Filter.all_of(Filter.only_cyrillic(), Filter.non_empty())
Filter.any_of(Filter.contains("!"), Filter.contains("?"))
```

不带参数的 `Filter.all_of()` 接受一切，不带参数的 `Filter.any_of()`
不接受任何内容。

> **重要：** 请使用 `&`、`|`、`~`，而不是 Python 的 `and`、`or`、`not`。
> 过滤器有意不定义真值：`bool(Filter...)` 会抛出 `TypeError`，
> 这样偶然写成 `f1 and f2` 时不会悄无声息地出错。

`~` 是对整个谓词做逻辑取反，而不是把一种操作换成另一种：

```python
~Filter.only_digits()   # 对「所有字符都是数字」取反
Filter.no_digits()      # 「一个数字都没有」
```

### 字符类与谓词的区别

`CharSet` 描述的是**允许字符的集合**，而 `Filter` 描述的是**针对整个字符串的条件**。
这是两种不同的操作，不能混为一谈：

```python
from strverify import CharSet, Filter

# 正确：字符集为数字 或 西里尔字母
f = Filter.only(CharSet.DIGITS | CharSet.CYRILLIC)
f("Привет123")   # True

# 这与上面并不相同：「整串只由数字组成」 或
# 「整串只由西里尔字母组成」
g = Filter.only_digits() | Filter.only_cyrillic()
g("Привет123")   # False
```

两个 `only` 过滤器用 `&` 组合也不会合并字符类：它要求两个条件同时成立。
要合并字符类，请使用 `CharSet` 和一个 `Filter.only`。

常见场景有现成的预设：
`Filter.only_digits_and_cyrillic()` 和 `Filter.only_digits_and(".,")`。

## 字符类

`CharSet` 是不可变集合，用 `|` 运算符求并集：

```python
allowed = CharSet.DIGITS | CharSet.of(".,_")
assert "5" in allowed
assert "_" in allowed
assert "a" not in allowed
```

| 字符类 | 含义 |
|--------|------|
| `CharSet.CYRILLIC` | Unicode Cyrillic 文字系统 |
| `CharSet.LATIN` | Unicode Latin 文字系统 |
| `CharSet.LETTERS` | 任意 Unicode 字母（`L*`） |
| `CharSet.DIGITS` | 任意 Unicode 十进制数字（`Nd`），包括 `0-9` |
| `CharSet.ASCII_DIGITS` | 仅 ASCII 数字 `0-9` |
| `CharSet.PUNCTUATION` | Unicode 标点（`P*`） |
| `CharSet.SYMBOLS` | Unicode 符号（`S*`）：`+`、`€`、`😀` |
| `CharSet.WHITESPACE` | Unicode 空白字符 |
| `CharSet.ASCII` | 所有 ASCII 字符 `U+0000..U+007F` |

`CharSet.of("...")` 添加具体的字符。`CharSet.empty()` 创建空集合。

例如，包含空格和连字符的西里尔字母：

```python
Filter.only(CharSet.CYRILLIC | CharSet.of(" -"))
```

`Filter.only_symbols()` 是 `PUNCTUATION | SYMBOLS` 的预设，不包含空白。
如果需要空格：

```python
Filter.only(CharSet.PUNCTUATION | CharSet.SYMBOLS | CharSet.WHITESPACE)
```

## 空字符串

字符类过滤器形式上接受空字符串：其中没有任何违反限制的字符。
`Filter.no_digits()`、`Filter.only_digits()` 和 `Filter.only_symbols()`
对 `""` 返回 `True`，而 `Filter.contains("!")` 返回 `False`。

如果不允许空字符串，请加上 `Filter.non_empty()`：

```python
f = Filter.only_digits() & Filter.non_empty()
f("")     # False
f("123")  # True
```

## API

### `Filter`

在所有带 `chars` 参数的方法中，既可以传入 `str`（字面字符集合），
也可以传入 `CharSet`。

创建：

- `Filter.only(chars)` — 字符串的所有字符都属于 `chars`（字符串或 `CharSet`）。
- `Filter.none_of(chars)` — 不含 `chars` 中的任何字符。
- `Filter.contains(chars)` — 至少包含 `chars` 中的一个字符。
- `Filter.non_empty()` — 字符串非空。
- `Filter.always()` / `Filter.never()` — 恒真与永假。
- `Filter.only_cyrillic()`、`Filter.only_latin()`、`Filter.only_digits()`、
  `Filter.no_digits()`、`Filter.only_digits_and(chars)`、
  `Filter.only_digits_and_cyrillic()`、`Filter.only_digits_and_underscore()`、
  `Filter.only_symbols()`。
- `Filter.all_of(*filters)`、`Filter.any_of(*filters)`。

匹配与组合：

- `f.matches(text)` 和 `f(text)`。
- `f & g`、`f | g`、`~f`。
- `repr(f)` 显示表达式树。

### `CharSet`

- 上表中的常量。
- `CharSet.of(chars)` — 字面字符。
- `CharSet.empty()`。
- `set | set` — 并集。
- `"x" in set` — 检查单个字符。

## 开发

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest

maturin develop --release
pytest
cargo test
```

构建 wheel 和 sdist：

```bash
maturin build --release
maturin sdist
```

构建产物会出现在 `target/wheels/`。

发布到 PyPI（需要在 `MATURIN_PYPI_TOKEN` 中提供令牌，或交互式输入）：

```bash
maturin publish
```

建议先在 TestPyPI 上验证包：

```bash
maturin publish --repository testpypi
```

## 许可证

MIT。参见 [LICENSE](../LICENSE)。
