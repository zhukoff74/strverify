//! Composable string-content filters.
//!
//! The Python API is intentionally small:
//!
//! * [`CharSet`] describes *which characters* are allowed (or forbidden).
//! * [`Filter`] is a composable predicate over a whole string.
//!
//! Character-set composition (`|`) and predicate composition (`&`, `|`, `~`)
//! are separate operations. This distinction makes expressions such as
//! "only digits and Cyrillic" precise:
//!
//! ```python
//! from strverify import CharSet, Filter
//!
//! Filter.only(CharSet.DIGITS | CharSet.CYRILLIC)  # correct
//! Filter.only_digits_and_cyrillic()               # the same preset
//! ```
//!
//! whereas `Filter.only_digits() | Filter.only_cyrillic()` means "the whole
//! string is all digits, or the whole string is all Cyrillic".

use pyo3::prelude::*;

mod charset;
mod filter;

pub use charset::CharSet;
pub use filter::Filter;

/// Low-level Rust extension module.
///
/// Import the public API from the `strverify` package instead.
#[pymodule]
fn _strverify(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<CharSet>()?;
    module.add_class::<Filter>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
