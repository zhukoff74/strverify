//! Boolean composition of string-content predicates.

use std::fmt;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::charset::CharSet;

/// An immutable, composable predicate over text content.
///
/// Filters are created by static constructors such as [`Filter.only`],
/// [`Filter.none_of`], [`Filter.contains`] and the named presets. Combine
/// them with the bitwise-style operators `&`, `|` and `~` (not with Python's
/// `and`, `or` and `not`), then call the result or use
/// [`Filter.matches`]:
///
/// ```python
/// from strverify import CharSet, Filter
///
/// f = Filter.only_cyrillic() & ~Filter.contains("!") & Filter.non_empty()
/// assert f.matches("Привет")
/// assert not f("Привет!")
/// ```
///
/// Character-class filters accept the empty string (every character of the
/// empty string satisfies the constraint). Add `Filter.non_empty()` when an
/// empty value must be rejected.
#[pyclass(frozen, module = "strverify._strverify", name = "Filter")]
#[derive(Clone, Debug)]
pub struct Filter {
    expr: Expr,
}

#[derive(Clone, Debug)]
enum Expr {
    Always,
    Never,
    Only(CharSet),
    NoneOf(CharSet),
    Contains(CharSet),
    NonEmpty,
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
}

impl Expr {
    fn evaluate(&self, text: &str) -> bool {
        match self {
            Expr::Always => true,
            Expr::Never => false,
            Expr::Only(set) => text.chars().all(|ch| set.contains(ch)),
            Expr::NoneOf(set) => text.chars().all(|ch| !set.contains(ch)),
            Expr::Contains(set) => text.chars().any(|ch| set.contains(ch)),
            Expr::NonEmpty => !text.is_empty(),
            Expr::And(parts) => parts.iter().all(|part| part.evaluate(text)),
            Expr::Or(parts) => parts.iter().any(|part| part.evaluate(text)),
            Expr::Not(inner) => !inner.evaluate(text),
        }
    }

    fn and(self, other: Expr) -> Expr {
        match (self, other) {
            (Expr::Always, right) => right,
            (left, Expr::Always) => left,
            (Expr::Never, _) | (_, Expr::Never) => Expr::Never,
            (Expr::And(mut left), Expr::And(right)) => {
                left.extend(right);
                Expr::And(left)
            }
            (Expr::And(mut left), right) => {
                left.push(right);
                Expr::And(left)
            }
            (left, Expr::And(right)) => {
                let mut parts = Vec::with_capacity(right.len() + 1);
                parts.push(left);
                parts.extend(right);
                Expr::And(parts)
            }
            (left, right) => Expr::And(vec![left, right]),
        }
    }

    fn or(self, other: Expr) -> Expr {
        match (self, other) {
            (Expr::Never, right) => right,
            (left, Expr::Never) => left,
            (Expr::Always, _) | (_, Expr::Always) => Expr::Always,
            (Expr::Or(mut left), Expr::Or(right)) => {
                left.extend(right);
                Expr::Or(left)
            }
            (Expr::Or(mut left), right) => {
                left.push(right);
                Expr::Or(left)
            }
            (left, Expr::Or(right)) => {
                let mut parts = Vec::with_capacity(right.len() + 1);
                parts.push(left);
                parts.extend(right);
                Expr::Or(parts)
            }
            (left, right) => Expr::Or(vec![left, right]),
        }
    }

    fn negate(self) -> Expr {
        match self {
            Expr::Always => Expr::Never,
            Expr::Never => Expr::Always,
            Expr::Not(inner) => *inner,
            other => Expr::Not(Box::new(other)),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expr.fmt(f)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Always => f.write_str("Filter.always()"),
            Expr::Never => f.write_str("Filter.never()"),
            Expr::Only(set) => write!(f, "Filter.only({set})"),
            Expr::NoneOf(set) => write!(f, "Filter.none_of({set})"),
            Expr::Contains(set) => write!(f, "Filter.contains({set})"),
            Expr::NonEmpty => f.write_str("Filter.non_empty()"),
            Expr::And(parts) => {
                f.write_str("(")?;
                for (index, part) in parts.iter().enumerate() {
                    if index > 0 {
                        f.write_str(" & ")?;
                    }
                    write!(f, "{part}")?;
                }
                f.write_str(")")
            }
            Expr::Or(parts) => {
                f.write_str("(")?;
                for (index, part) in parts.iter().enumerate() {
                    if index > 0 {
                        f.write_str(" | ")?;
                    }
                    write!(f, "{part}")?;
                }
                f.write_str(")")
            }
            Expr::Not(inner) => write!(f, "~{inner}"),
        }
    }
}

impl Filter {
    fn from_expr(expr: Expr) -> Self {
        Self { expr }
    }

    fn only_set(set: CharSet) -> Self {
        Self::from_expr(Expr::Only(set))
    }

    fn contains_set(set: CharSet) -> Self {
        Self::from_expr(Expr::Contains(set))
    }

    fn and(self, other: Self) -> Self {
        Self::from_expr(self.expr.and(other.expr))
    }

    fn or(self, other: Self) -> Self {
        Self::from_expr(self.expr.or(other.expr))
    }

    fn negate(self) -> Self {
        Self::from_expr(self.expr.negate())
    }
}

/// Convert a Python `str` or [`CharSet`] argument into a `CharSet`.
fn charset_from_arg(value: &Bound<'_, PyAny>) -> PyResult<CharSet> {
    if let Ok(literal) = value.extract::<String>() {
        return Ok(CharSet::literal(&literal));
    }
    if let Ok(set) = value.extract::<PyRef<'_, CharSet>>() {
        return Ok(set.clone());
    }
    Err(PyTypeError::new_err(
        "expected a str of literal characters or a CharSet",
    ))
}

fn extract_filters(args: &Bound<'_, PyTuple>) -> PyResult<Vec<Filter>> {
    let mut filters = Vec::with_capacity(args.len());
    for item in args.iter() {
        filters.push(item.extract::<PyRef<'_, Filter>>()?.clone());
    }
    Ok(filters)
}

#[pymethods]
impl Filter {
    /// A filter that accepts any string.
    #[staticmethod]
    fn always() -> Self {
        Self::from_expr(Expr::Always)
    }

    /// A filter that rejects every string.
    #[staticmethod]
    fn never() -> Self {
        Self::from_expr(Expr::Never)
    }

    /// A filter that rejects the empty string.
    #[staticmethod]
    fn non_empty() -> Self {
        Self::from_expr(Expr::NonEmpty)
    }

    /// Accept strings whose every character belongs to `chars`.
    ///
    /// `chars` may be a [`CharSet`] or a string of literal characters.
    /// The empty string is accepted by this character-class rule; combine
    /// with [`Filter.non_empty`] to reject it.
    #[staticmethod]
    fn only(chars: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self::only_set(charset_from_arg(chars)?))
    }

    /// Accept strings that contain none of the characters in `chars`.
    #[staticmethod]
    fn none_of(chars: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self::from_expr(Expr::NoneOf(charset_from_arg(chars)?)))
    }

    /// Accept strings that contain at least one character from `chars`.
    #[staticmethod]
    fn contains(chars: &Bound<'_, PyAny>) -> PyResult<Self> {
        let set = charset_from_arg(chars)?;
        if set.is_empty() {
            return Err(PyValueError::new_err(
                "Filter.contains() requires a non-empty set of characters",
            ));
        }
        Ok(Self::contains_set(set))
    }

    /// Accept strings made only of Cyrillic script characters.
    #[staticmethod]
    fn only_cyrillic() -> Self {
        Self::only_set(CharSet::cyrillic())
    }

    /// Accept strings made only of Latin script characters.
    #[staticmethod]
    fn only_latin() -> Self {
        Self::only_set(CharSet::latin())
    }

    /// Accept strings made only of Unicode decimal digits (category `Nd`).
    #[staticmethod]
    fn only_digits() -> Self {
        Self::only_set(CharSet::digits())
    }

    /// Accept strings that contain no Unicode decimal digits.
    #[staticmethod]
    fn no_digits() -> Self {
        Self::from_expr(Expr::NoneOf(CharSet::digits()))
    }

    /// Accept strings made only of Unicode decimal digits and the characters
    /// in `chars`.
    ///
    /// ```python
    /// Filter.only_digits_and(".,")   # digits plus '.' and ','
    /// Filter.only_digits_and("_")    # digits plus '_'
    /// Filter.only_digits_and(CharSet.CYRILLIC)
    /// ```
    #[staticmethod]
    fn only_digits_and(chars: &Bound<'_, PyAny>) -> PyResult<Self> {
        let extra = charset_from_arg(chars)?;
        Ok(Self::only_set(CharSet::digits().union(&extra)))
    }

    /// Accept strings made only of Unicode decimal digits and Cyrillic
    /// characters.
    #[staticmethod]
    fn only_digits_and_cyrillic() -> Self {
        Self::only_set(CharSet::digits().union(&CharSet::cyrillic()))
    }

    /// Accept strings made only of Unicode decimal digits and `_`.
    #[staticmethod]
    fn only_digits_and_underscore() -> Self {
        Self::only_set(CharSet::digits().union(&CharSet::literal("_")))
    }

    /// Accept strings made only of Unicode punctuation and symbols.
    ///
    /// Whitespace, letters and digits are not accepted. Use
    /// `Filter.only(CharSet.PUNCTUATION | CharSet.SYMBOLS | CharSet.WHITESPACE)`
    /// if spaces should be allowed.
    #[staticmethod]
    fn only_symbols() -> Self {
        Self::only_set(CharSet::punctuation().union(&CharSet::symbols()))
    }

    /// Combine filters with logical AND.
    #[staticmethod]
    #[pyo3(signature = (*filters))]
    fn all_of(filters: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let mut expr = Expr::Always;
        for filter in extract_filters(filters)? {
            expr = expr.and(filter.expr);
        }
        Ok(Self::from_expr(expr))
    }

    /// Combine filters with logical OR.
    #[staticmethod]
    #[pyo3(signature = (*filters))]
    fn any_of(filters: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let mut expr = Expr::Never;
        for filter in extract_filters(filters)? {
            expr = expr.or(filter.expr);
        }
        Ok(Self::from_expr(expr))
    }

    /// Return `True` when `text` satisfies this filter.
    fn matches(&self, text: &str) -> bool {
        self.expr.evaluate(text)
    }

    /// Calling a filter is equivalent to :meth:`matches`.
    fn __call__(&self, text: &str) -> bool {
        self.expr.evaluate(text)
    }

    /// Logical AND: both filters must match.
    fn __and__(&self, other: &Self) -> Self {
        self.clone().and(other.clone())
    }

    /// Logical OR: at least one filter must match.
    fn __or__(&self, other: &Self) -> Self {
        self.clone().or(other.clone())
    }

    /// Logical NOT: invert this filter.
    fn __invert__(&self) -> Self {
        self.clone().negate()
    }

    /// Return a Python-like expression tree for debugging.
    fn __repr__(&self) -> String {
        self.to_string()
    }

    /// Filter objects deliberately have no truth value.
    fn __bool__(&self) -> PyResult<bool> {
        Err(PyTypeError::new_err(
            "Filter has no truth value; combine filters with &, | and ~",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(filter: Filter, text: &str) -> bool {
        filter.matches(text)
    }

    #[test]
    fn only_filters_are_vacuously_true_for_empty_text() {
        let cyr = Filter::only_cyrillic();
        assert!(check(cyr.clone(), "Привет"));
        assert!(!check(cyr, "Привет!"));
        assert!(check(Filter::only_cyrillic(), ""));
        assert!(!check(Filter::only_cyrillic(), "Hello"));
    }

    #[test]
    fn only_digits_and_no_digits() {
        let digits = Filter::only_digits();
        assert!(check(digits.clone(), "123"));
        assert!(check(digits.clone(), "١٢٣"));
        assert!(!check(digits, "12.3"));

        let no_digits = Filter::no_digits();
        assert!(check(no_digits.clone(), "abc"));
        assert!(check(no_digits.clone(), ""));
        assert!(!check(no_digits, "abc١"));
    }

    #[test]
    fn literal_characters_and_classes_can_be_mixed() {
        let filter = Filter::only_set(CharSet::digits().union(&CharSet::literal(".,_")));
        assert!(check(filter.clone(), "12.34,56_78"));
        assert!(!check(filter, "12-34"));
        assert!(!check(Filter::only_digits_and_cyrillic(), "Hello123"));
        assert!(check(Filter::only_digits_and_cyrillic(), "Привет123"));
        assert!(check(Filter::only_digits_and_underscore(), "12_34"));
    }

    #[test]
    fn contains_and_none_of() {
        let bang = Filter::contains_set(CharSet::literal("!"));
        assert!(check(bang.clone(), "Привет!"));
        assert!(!check(bang, "Привет"));

        let no_digits = Filter::from_expr(Expr::NoneOf(CharSet::digits()));
        assert!(check(no_digits.clone(), "Привет"));
        assert!(!check(no_digits, "Привет1"));
    }

    #[test]
    fn boolean_composition() {
        let filter = Filter::only_cyrillic().and(Filter::non_empty());
        assert!(check(filter.clone(), "Привет"));
        assert!(!check(filter.clone(), ""));
        assert!(!check(filter, "Привет!"));

        let with_bang = Filter::contains_set(CharSet::literal("!")).and(Filter::non_empty());
        assert!(check(with_bang.clone(), "Привет!"));
        assert!(!check(with_bang, "Привет"));
    }

    #[test]
    fn or_is_whole_string_disjunction() {
        let filter = Filter::only_digits().or(Filter::only_cyrillic());
        assert!(check(filter.clone(), "123"));
        assert!(check(filter.clone(), "Привет"));
        assert!(!check(filter, "Привет123"));
    }

    #[test]
    fn double_negation_is_identity() {
        let filter = Filter::only_digits();
        let double = filter.clone().negate().negate();
        assert!(check(double.clone(), "123"));
        assert!(!check(double, "abc"));
    }

    #[test]
    fn identities() {
        assert!(check(Filter::always(), ""));
        assert!(check(Filter::always(), "anything"));
        assert!(!check(Filter::never(), "anything"));
        assert!(!check(Filter::never(), ""));
    }

    #[test]
    fn logical_identities_are_simplified() {
        assert!(check(Filter::always().and(Filter::only_digits()), "123"));
        assert!(!check(Filter::never().or(Filter::only_digits()), "abc"));
        assert_eq!(
            Filter::always().or(Filter::only_digits()).to_string(),
            "Filter.always()"
        );
    }
}
