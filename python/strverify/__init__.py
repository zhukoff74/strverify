"""Composable string-content filters with a Rust core.

The public API consists of :class:`CharSet` and :class:`Filter`:

>>> from strverify import CharSet, Filter
>>> f = Filter.only_digits_and(".,")
>>> f.matches("1 234,50")
False
>>> f.matches("1234,50")
True
"""

from ._strverify import CharSet, Filter, __version__

__all__ = ["CharSet", "Filter", "__version__"]
