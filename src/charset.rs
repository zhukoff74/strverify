//! Character-set primitives used by [`Filter`](crate::filter::Filter).
//!
//! A `CharSet` is an immutable union of Unicode-aware character classes and
//! literal characters. It is intentionally *not* a general-purpose set:
//! only union is supported, because filters operate on "allowed" sets.
//! Boolean composition belongs to [`Filter`](crate::filter::Filter).

use std::collections::BTreeSet;
use std::fmt;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use unicode_general_category::{get_general_category, GeneralCategory};
use unicode_script::{Script, UnicodeScript};

pub(crate) const CAT_CYRILLIC: u16 = 1 << 0;
pub(crate) const CAT_LATIN: u16 = 1 << 1;
pub(crate) const CAT_LETTERS: u16 = 1 << 2;
pub(crate) const CAT_DIGITS: u16 = 1 << 3;
pub(crate) const CAT_ASCII_DIGITS: u16 = 1 << 4;
pub(crate) const CAT_PUNCTUATION: u16 = 1 << 5;
pub(crate) const CAT_SYMBOLS: u16 = 1 << 6;
pub(crate) const CAT_WHITESPACE: u16 = 1 << 7;
pub(crate) const CAT_ASCII: u16 = 1 << 8;

const CATEGORY_NAMES: &[(u16, &str)] = &[
    (CAT_CYRILLIC, "CYRILLIC"),
    (CAT_LATIN, "LATIN"),
    (CAT_LETTERS, "LETTERS"),
    (CAT_DIGITS, "DIGITS"),
    (CAT_ASCII_DIGITS, "ASCII_DIGITS"),
    (CAT_PUNCTUATION, "PUNCTUATION"),
    (CAT_SYMBOLS, "SYMBOLS"),
    (CAT_WHITESPACE, "WHITESPACE"),
    (CAT_ASCII, "ASCII"),
];

/// An immutable set of characters built from Unicode classes and literals.
///
/// Instances are created through the predefined class attributes or
/// [`CharSet.of`], and are combined with `|`:
///
/// ```python
/// from strverify import CharSet
///
/// allowed = CharSet.DIGITS | CharSet.of(".,_")
/// assert "1" in allowed
/// assert "_" in allowed
/// assert "a" not in allowed
/// ```
///
/// Union only: `CharSet` deliberately does not implement `&`, `-` or `~`.
/// Character-set union is different from boolean composition of filters.
/// See [`Filter`](crate::filter::Filter) for `&`, `|` and `~`.
#[pyclass(frozen, module = "strverify._strverify", name = "CharSet")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CharSet {
    categories: u16,
    literals: BTreeSet<char>,
}

impl CharSet {
    pub(crate) fn empty_set() -> Self {
        Self::default()
    }

    pub(crate) fn category(categories: u16) -> Self {
        Self {
            categories,
            literals: BTreeSet::new(),
        }
    }

    pub(crate) fn literal(chars: &str) -> Self {
        Self {
            categories: 0,
            literals: chars.chars().collect(),
        }
    }

    pub(crate) fn cyrillic() -> Self {
        Self::category(CAT_CYRILLIC)
    }

    pub(crate) fn latin() -> Self {
        Self::category(CAT_LATIN)
    }

    pub(crate) fn letters() -> Self {
        Self::category(CAT_LETTERS)
    }

    pub(crate) fn digits() -> Self {
        Self::category(CAT_DIGITS)
    }

    pub(crate) fn punctuation() -> Self {
        Self::category(CAT_PUNCTUATION)
    }

    pub(crate) fn symbols() -> Self {
        Self::category(CAT_SYMBOLS)
    }

    pub(crate) fn union(&self, other: &Self) -> Self {
        let mut literals = self.literals.clone();
        literals.extend(other.literals.iter().copied());
        Self {
            categories: self.categories | other.categories,
            literals,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.categories == 0 && self.literals.is_empty()
    }

    /// Returns whether `ch` belongs to this set.
    #[inline]
    pub(crate) fn contains(&self, ch: char) -> bool {
        if self.literals.contains(&ch) {
            return true;
        }

        let cats = self.categories;
        if cats == 0 {
            return false;
        }

        if cats & CAT_DIGITS != 0 && is_decimal_digit(ch) {
            return true;
        }
        if cats & CAT_ASCII_DIGITS != 0 && ch.is_ascii_digit() {
            return true;
        }
        if cats & CAT_ASCII != 0 && ch.is_ascii() {
            return true;
        }
        if cats & CAT_WHITESPACE != 0 && ch.is_whitespace() {
            return true;
        }

        // Consult the script tables only when a script class is requested.
        // Script_Extension considers Common and Inherited characters as
        // intersecting every script; exclude Common so digits, punctuation
        // and whitespace do not count as Cyrillic/Latin. Inherited combining
        // marks remain allowed, because they are shared by several scripts.
        if cats & CAT_CYRILLIC != 0 && belongs_to_script(ch, Script::Cyrillic) {
            return true;
        }
        if cats & CAT_LATIN != 0 && belongs_to_script(ch, Script::Latin) {
            return true;
        }

        if cats & (CAT_LETTERS | CAT_PUNCTUATION | CAT_SYMBOLS) != 0 {
            let category = get_general_category(ch);
            if cats & CAT_LETTERS != 0 && is_letter(category) {
                return true;
            }
            if cats & CAT_PUNCTUATION != 0 && is_punctuation(category) {
                return true;
            }
            if cats & CAT_SYMBOLS != 0 && is_symbol(category) {
                return true;
            }
        }

        false
    }

    /// A Python-ish representation used by `Filter.__repr__` as well.
    pub(crate) fn repr(&self) -> String {
        if self.is_empty() {
            return "CharSet.empty()".to_owned();
        }

        let mut parts = Vec::new();
        for (bit, name) in CATEGORY_NAMES {
            if self.categories & bit != 0 {
                parts.push(format!("CharSet.{name}"));
            }
        }

        if !self.literals.is_empty() {
            let literal: String = self.literals.iter().collect();
            parts.push(format!("CharSet.of({})", python_string_literal(&literal)));
        }

        parts.join(" | ")
    }
}

fn belongs_to_script(ch: char, script: Script) -> bool {
    let extension = ch.script_extension();
    !extension.is_common() && extension.contains_script(script)
}

fn is_decimal_digit(ch: char) -> bool {
    ch.is_ascii_digit() || get_general_category(ch) == GeneralCategory::DecimalNumber
}

fn is_letter(category: GeneralCategory) -> bool {
    matches!(
        category,
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
    )
}

fn is_punctuation(category: GeneralCategory) -> bool {
    matches!(
        category,
        GeneralCategory::ConnectorPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::OtherPunctuation
            | GeneralCategory::OpenPunctuation
    )
}

fn is_symbol(category: GeneralCategory) -> bool {
    matches!(
        category,
        GeneralCategory::CurrencySymbol
            | GeneralCategory::ModifierSymbol
            | GeneralCategory::MathSymbol
            | GeneralCategory::OtherSymbol
    )
}

fn python_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('\'');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let code = c as u32;
                if code <= 0xFF {
                    out.push_str(&format!("\\x{code:02x}"));
                } else if code <= 0xFFFF {
                    out.push_str(&format!("\\u{code:04x}"));
                } else {
                    out.push_str(&format!("\\U{code:08x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

impl fmt::Display for CharSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.repr())
    }
}

#[pymethods]
impl CharSet {
    /// Unicode Cyrillic script (letters and accompanying combining marks).
    #[classattr]
    #[allow(non_snake_case)]
    fn CYRILLIC() -> CharSet {
        CharSet::cyrillic()
    }

    /// Unicode Latin script (letters and accompanying combining marks).
    #[classattr]
    #[allow(non_snake_case)]
    fn LATIN() -> CharSet {
        CharSet::latin()
    }

    /// Any Unicode letter (general category `L*`).
    #[classattr]
    #[allow(non_snake_case)]
    fn LETTERS() -> CharSet {
        CharSet::letters()
    }

    /// Any Unicode decimal digit (general category `Nd`, includes `0-9`).
    #[classattr]
    #[allow(non_snake_case)]
    fn DIGITS() -> CharSet {
        CharSet::digits()
    }

    /// The ASCII digits `0-9` only.
    #[classattr]
    #[allow(non_snake_case)]
    fn ASCII_DIGITS() -> CharSet {
        CharSet::category(CAT_ASCII_DIGITS)
    }

    /// Any Unicode punctuation character (general category `P*`).
    #[classattr]
    #[allow(non_snake_case)]
    fn PUNCTUATION() -> CharSet {
        CharSet::punctuation()
    }

    /// Any Unicode symbol (general category `S*`), for example `+`, `€`, `😀`.
    #[classattr]
    #[allow(non_snake_case)]
    fn SYMBOLS() -> CharSet {
        CharSet::symbols()
    }

    /// Unicode whitespace (`char::is_whitespace`).
    #[classattr]
    #[allow(non_snake_case)]
    fn WHITESPACE() -> CharSet {
        CharSet::category(CAT_WHITESPACE)
    }

    /// Any ASCII character (`U+0000..=U+007F`).
    #[classattr]
    #[allow(non_snake_case)]
    fn ASCII() -> CharSet {
        CharSet::category(CAT_ASCII)
    }

    /// Create a set from literal characters: `CharSet.of(".,_")`.
    #[staticmethod]
    fn of(chars: &str) -> CharSet {
        CharSet::literal(chars)
    }

    /// An empty set. It matches nothing; useful as an identity value.
    #[staticmethod]
    fn empty() -> CharSet {
        CharSet::empty_set()
    }

    /// Union two character sets: `CharSet.DIGITS | CharSet.of("._")`.
    fn __or__(&self, other: &Self) -> Self {
        self.union(other)
    }

    /// Test a single character for membership.
    ///
    /// ```python
    /// "5" in CharSet.DIGITS  # True
    /// ```
    fn __contains__(&self, value: &str) -> PyResult<bool> {
        let mut chars = value.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Ok(self.contains(ch)),
            _ => Err(PyValueError::new_err(
                "CharSet membership expects exactly one character",
            )),
        }
    }

    /// Return a Python-like description of the set.
    fn __repr__(&self) -> String {
        self.repr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyrillic_and_latin_are_scripts() {
        let cyr = CharSet::cyrillic();
        assert!(cyr.contains('Ж'));
        assert!(cyr.contains('ё'));
        assert!(cyr.contains('Ї'));
        assert!(!cyr.contains('A'));
        assert!(!cyr.contains('1'));
        assert!(!cyr.contains('!'));

        let lat = CharSet::latin();
        assert!(lat.contains('A'));
        assert!(lat.contains('é'));
        assert!(!lat.contains('Ж'));
        assert!(!lat.contains('1'));
        assert!(!lat.contains('!'));
    }

    #[test]
    fn digits_are_unicode_decimal_numbers() {
        let digits = CharSet::digits();
        assert!(digits.contains('0'));
        assert!(digits.contains('9'));
        assert!(digits.contains('٣'));
        assert!(!digits.contains('a'));

        let ascii = CharSet::category(CAT_ASCII_DIGITS);
        assert!(ascii.contains('5'));
        assert!(!ascii.contains('٣'));
    }

    #[test]
    fn punctuation_and_symbols_are_distinct_but_composable() {
        let punct = CharSet::punctuation();
        let symbols = CharSet::symbols();
        assert!(punct.contains('!'));
        assert!(!punct.contains('+'));
        assert!(symbols.contains('+'));
        assert!(symbols.contains('€'));
        assert!(symbols.contains('😀'));
        assert!(!symbols.contains(' '));

        let both = punct.union(&symbols);
        assert!(both.contains('!'));
        assert!(both.contains('+'));
        assert!(!both.contains('A'));
        assert!(!both.contains('1'));
        assert!(!both.contains(' '));
    }

    #[test]
    fn literal_union_deduplicates() {
        let set = CharSet::literal("abca").union(&CharSet::literal("bd"));
        for ch in ['a', 'b', 'c', 'd'] {
            assert!(set.contains(ch));
        }
        assert!(!set.contains('e'));
    }

    #[test]
    fn empty_set_contains_nothing() {
        let set = CharSet::empty_set();
        assert!(set.is_empty());
        assert!(!set.contains('a'));
        assert!(!set.contains(' '));
    }

    #[test]
    fn repr_is_stable() {
        let set = CharSet::digits().union(&CharSet::literal("._"));
        assert_eq!(set.repr(), "CharSet.DIGITS | CharSet.of('._')");
        assert_eq!(CharSet::empty_set().repr(), "CharSet.empty()");
    }
}
