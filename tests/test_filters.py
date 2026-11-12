import pytest

import strverify
from strverify import CharSet, Filter


def test_package_version():
    assert isinstance(strverify.__version__, str)
    assert strverify.__version__


class TestExamplesFromTheBrief:
    def test_only_cyrillic(self):
        f = Filter.only_cyrillic()
        assert f("Привет")
        assert f("Ёжик")
        assert f("Їжак")
        assert not f("Привет!")
        assert not f("Привет 1")
        assert not f("Hello")
        assert not f("123")

    def test_only_latin(self):
        f = Filter.only_latin()
        assert f("Hello")
        assert f("café")
        assert not f("Hello!")
        assert not f("Hello1")
        assert not f("Привет")

    def test_cyrillic_or_latin(self):
        f = Filter.only_cyrillic() | Filter.only_latin()
        assert f("Привет")
        assert f("Hello")
        assert not f("ПриветHello")
        assert not f("Hello1")

    def test_only_digits(self):
        f = Filter.only_digits()
        assert f("0123456789")
        assert f("١٢٣")  # Unicode decimal digits, not just ASCII
        assert not f("12.3")
        assert not f("12 ")
        assert not f("abc123")

    def test_ascii_digits_only(self):
        f = Filter.only(CharSet.ASCII_DIGITS)
        assert f("0123456789")
        assert not f("١٢٣")

    def test_no_digits(self):
        f = Filter.no_digits()
        assert f("Привет")
        assert f("Hello, world!")
        assert f("")
        assert not f("abc1")
        assert not f("abc١")

    def test_only_digits_and_dot_comma(self):
        f = Filter.only_digits_and(".,")
        assert f("123")
        assert f("1,234.50")
        assert not f("1 234")
        assert not f("1-2")
        assert not f("12a")

        explicit = Filter.only(CharSet.DIGITS | CharSet.of(".,"))
        for value in ["123", "1,234.50", "1 234", "1-2", "12a"]:
            assert f(value) == explicit(value)

    def test_contains_exclamation(self):
        f = Filter.contains("!")
        assert f("Привет!")
        assert f("!")
        assert not f("Привет")
        assert not f("")

    def test_only_digits_and_cyrillic(self):
        f = Filter.only_digits_and_cyrillic()
        assert f("Привет123")
        assert f("123")
        assert f("Привет")
        assert not f("Привет 123")
        assert not f("Hello123")
        assert not f("Привет!")

        explicit = Filter.only(CharSet.DIGITS | CharSet.CYRILLIC)
        for value in ["Привет123", "Привет 123", "Hello123"]:
            assert f(value) == explicit(value)

    def test_only_symbols(self):
        f = Filter.only_symbols()
        assert f("!@#$%^&*()_+-=[]{};:'\",.<>/?`~")
        assert f("€£¥")
        assert f("😀🚀")
        assert not f("Hello!")
        assert not f("123!")
        assert not f(" !")
        assert f("")

    def test_only_digits_and_underscore(self):
        f = Filter.only_digits_and_underscore()
        assert f("123_456")
        assert f("123")
        assert f("_")
        assert not f("12.3")
        assert not f("abc_123")


class TestEmptyStringSemantics:
    @pytest.mark.parametrize(
        "filter_",
        [
            Filter.only_cyrillic(),
            Filter.only_latin(),
            Filter.only_digits(),
            Filter.no_digits(),
            Filter.only_digits_and(".,"),
            Filter.only_symbols(),
            Filter.only_digits_and_cyrillic(),
            Filter.only_digits_and_underscore(),
        ],
    )
    def test_character_class_filters_accept_empty(self, filter_):
        assert filter_("")

    def test_contains_rejects_empty(self):
        assert not Filter.contains("!")("")

    def test_non_empty_composition_rejects_empty(self):
        f = Filter.only_digits() & Filter.non_empty()
        assert not f("")
        assert f("123")


class TestComposition:
    def test_generic_constructors_accept_literals_and_sets(self):
        only = Filter.only("abc")
        assert only("cab")
        assert only("")
        assert not only("cad")

        none = Filter.none_of("abc")
        assert none("xyz")
        assert not none("xay")

        contains = Filter.contains(CharSet.DIGITS)
        assert contains("a1")
        assert not contains("abc")

        cyr_digits = Filter.only_digits_and(CharSet.CYRILLIC)
        assert cyr_digits("Привет123")
        assert not cyr_digits("Hello123")

    def test_and_or_not_operators(self):
        f = Filter.only_cyrillic() & ~Filter.contains("!") & Filter.non_empty()
        assert f("Привет")
        assert not f("Привет!")
        assert not f("")
        assert not f("Hello")

        g = Filter.contains("!") | Filter.contains("?")
        assert g("Привет!")
        assert g("Как дела?")
        assert not g("Привет")

    def test_all_of_any_of(self):
        f = Filter.all_of(
            Filter.only_digits(),
            Filter.non_empty(),
            ~Filter.contains("0"),
        )
        assert f("123")
        assert not f("")
        assert not f("0123")
        assert not f("abc")

        g = Filter.any_of(Filter.contains("!"), Filter.contains("?"))
        assert g("!")
        assert g("?")
        assert not g(".")

    def test_all_of_and_any_of_identities(self):
        assert Filter.all_of()("anything")
        assert not Filter.any_of()("anything")
        assert Filter.always()("")
        assert not Filter.never()("")

    def test_de_morgan(self):
        a = Filter.contains("!")
        b = Filter.contains("?")
        left = ~(a | b)
        right = ~a & ~b
        for value in ["", "hello", "!", "?", "!?", "hello!"]:
            assert left(value) == right(value)

    def test_builtin_filter(self):
        f = Filter.only_digits()
        assert list(filter(f, ["123", "12a", "456", ""])) == ["123", "456", ""]

    def test_filters_are_immutable(self):
        base = Filter.only_digits()
        combined = base & Filter.non_empty()
        assert base("")
        assert not combined("")
        assert base("123")
        assert combined("123")

    @pytest.mark.parametrize(
        "value",
        ["", "a", "123", "Привет", "!@#", "a1!", "  "],
    )
    def test_double_negation(self, value):
        f = Filter.contains("!")
        assert (~~f)(value) == f(value)

    def test_input_can_be_any_python_string(self):
        f = Filter.only_cyrillic()
        assert f("Привет\n".strip())
        with pytest.raises(TypeError):
            f(b"bytes")
        with pytest.raises(TypeError):
            f(None)


class TestCharSet:
    def test_constants_and_membership(self):
        assert "Ж" in CharSet.CYRILLIC
        assert "A" in CharSet.LATIN
        assert "é" in CharSet.LATIN
        assert "5" in CharSet.DIGITS
        assert "٣" in CharSet.DIGITS
        assert "5" in CharSet.ASCII_DIGITS
        assert "٣" not in CharSet.ASCII_DIGITS
        assert "!" in CharSet.PUNCTUATION
        assert "€" in CharSet.SYMBOLS
        assert " " in CharSet.WHITESPACE
        assert "A" in CharSet.ASCII
        assert "Ж" not in CharSet.ASCII
        assert "A" in CharSet.LETTERS
        assert "!" not in CharSet.LETTERS

    def test_of_and_union(self):
        allowed = CharSet.DIGITS | CharSet.of(".,_") | CharSet.of("-")
        assert "5" in allowed
        assert "." in allowed
        assert "," in allowed
        assert "_" in allowed
        assert "-" in allowed
        assert "a" not in allowed
        assert " " not in allowed

    def test_union_does_not_mutate_constants(self):
        custom = CharSet.DIGITS | CharSet.of("_")
        assert "5" in custom
        assert "_" in custom
        assert "_" not in CharSet.DIGITS
        assert CharSet.DIGITS is CharSet.DIGITS

    def test_empty(self):
        empty = CharSet.empty()
        assert "a" not in empty
        assert " " not in empty
        union = empty | CharSet.DIGITS
        assert "5" in union
        assert "a" not in union
        assert repr(empty) == "CharSet.empty()"

    def test_membership_requires_one_character(self):
        with pytest.raises(ValueError):
            "!" * 2 in CharSet.PUNCTUATION
        with pytest.raises(ValueError):
            "" in CharSet.PUNCTUATION

    def test_repr(self):
        assert repr(CharSet.CYRILLIC) == "CharSet.CYRILLIC"
        assert repr(CharSet.DIGITS | CharSet.of("._")) == (
            "CharSet.DIGITS | CharSet.of('._')"
        )
        assert repr(CharSet.of("'")) == "CharSet.of('\\'')"


class TestMisc:
    def test_filter_repr(self):
        f = Filter.only_cyrillic() & ~Filter.contains("!")
        rendered = repr(f)
        assert "Filter.only(CharSet.CYRILLIC)" in rendered
        assert "~Filter.contains(CharSet.of('!'))" in rendered
        assert " & " in rendered

        assert repr(Filter.always()) == "Filter.always()"
        assert repr(Filter.never()) == "Filter.never()"
        assert repr(Filter.non_empty()) == "Filter.non_empty()"

    def test_bool_is_an_error_to_prevent_and_or_mistakes(self):
        with pytest.raises(TypeError, match="no truth value"):
            bool(Filter.only_digits())

    def test_invalid_filter_arguments(self):
        with pytest.raises(TypeError):
            Filter.only(123)
        with pytest.raises(TypeError):
            Filter.none_of(object())
        with pytest.raises(ValueError, match="non-empty"):
            Filter.contains("")
        with pytest.raises(TypeError):
            Filter.all_of(Filter.only_digits(), "not a filter")

    def test_import_star(self):
        namespace = {}
        exec("from strverify import *", namespace)
        assert "Filter" in namespace
        assert "CharSet" in namespace
