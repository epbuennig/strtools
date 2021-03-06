//! This crate provides the [`StrTools`] trait which exposes a variety of helper functions for
//! handling strings for use cases like handling user input.
//!
//! # Examples
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use strtools::StrTools;
//!
//! // split a string by some separator but ignore escaped ones
//! let parts: Vec<_> = r"this string\ is split by\ spaces and commas, unless they are\ escaped"
//!     .split_non_escaped_sanitize('\\', [' ', ','])?
//!     .collect();
//!
//! assert_eq!(
//!     parts,
//!     [
//!         "this",
//!         "string is",
//!         "split",
//!         "by spaces",
//!         "and",
//!         "commas",
//!         "",
//!         "unless",
//!         "they",
//!         "are escaped"
//!     ]
//! );
//! # Ok(())
//! # }
//! ```
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use strtools::StrTools;
//!
//! let parts: Vec<_> = r"\.\/.*s(\d\d)e(\d\d[a-d])/S$1E$2/gu"
//!     .split_non_escaped_sanitize('\\', '/')?
//!     .collect();
//!
//! // parsing user input regex rules like `<rule>/<replace>/<flags>`
//! // the rule contained an escaped separator but we don't want to
//! // actually escape it for the regex engine
//! assert_eq!(parts, [r"\./.*s(\d\d)e(\d\d[a-d])", "S$1E$2", "gu"]);
//! # Ok(())
//! # }
//! ```
// keep the nightly features set small in hopes that all used features are stabilized by the time
// this crate will stabilize
#![feature(
    associated_type_defaults,
    cow_is_borrowed,
    // https://github.com/rust-lang/rust/issues/57349
    // this should be fine, the only listed regression is very niche use case, but this would block
    // stabilization
    const_mut_refs,
    decl_macro,
    is_sorted,
    let_chains
)]
// check for missing documentation
#![warn(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::missing_safety_doc
)]
// reduce unsafe scopes to their minimum
#![deny(unsafe_op_in_unsafe_fn)]

use parse::{FromStrBack, FromStrFront};
use util::Sorted;

pub mod escape;
pub mod find;
pub mod parse;
pub mod split;
pub mod util;

/// The main trait of this crate, providing various extension methods for [`str`].
/// See the individual function documentation for more info. **The methods on this trait are subject
/// to change during the development of the crates core functionality.**
pub trait StrTools: util::sealed::Sealed {
    /// Behaves similar to [`str::split`] but generic of the the amount of indices.
    ///
    /// # Panics
    /// Panics if the last index is out of bounds:
    /// `indices.last() > Some(input.len)`
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use strtools::StrTools;
    ///
    /// let value = "0123456789ab";
    /// let ([first, second], third) = value.split_n_times(&[4, 8].try_into()?);
    ///
    /// assert_eq!(first, "0123");
    /// assert_eq!(second, "4567");
    /// assert_eq!(third, "89ab");
    /// # Ok(())
    /// # }
    /// ```
    fn split_n_times<const N: usize>(&self, indices: &Sorted<usize, N>) -> ([&str; N], &str);

    /// Splits a [`str`] by the given delimiters unless they are preceded by an escape.
    /// Escapes before significant chars are removed, significant chars are the delimiters and the
    /// escape itself. Trailing escapes are ignored as if followed by a non-significant char.
    /// `delims` single char or an array of chars, which will be sorted, see the
    /// [free version][free] of this function for more control over delimiter sorting.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `esc == delim`
    ///
    /// # Complexity
    /// This algorithm requires `O(n * max(log m, 1))` time where `n` is the length of the input
    /// string and `m` is the length of the delimiters.
    ///
    /// # Allocation
    /// If no escapes are encountered in a part, no allocations are done and the part is borrowed,
    /// otherwise a [`String`] and all but the escape chars before delimiters are copied over.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use strtools::StrTools;
    ///
    /// let value = r"Pa\rt0:Part1:Part2\:StillPart2";
    /// let parts: Vec<_> = value.split_non_escaped_sanitize('\\', ':')?.collect();
    ///
    /// // notice that the escape char was removed in Part2 but not in Part1 as it's just used as
    /// // an indicator for escaping the delimiters or escapes themselves
    /// assert_eq!(parts, [r"Pa\rt0", "Part1", "Part2:StillPart2"]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [free]: split::non_escaped_sanitize
    fn split_non_escaped_sanitize<D: Into<Sorted<char, N>>, const N: usize>(
        &self,
        esc: char,
        delims: D,
    ) -> Result<split::NonEscapedSanitize<'_, N>, split::NonEscapedError>;

    /// Splits a [`str`] by the given delimiters unless they are preceded by an escape.
    /// Escapes before significant chars are removed, significant chars are the delimiters and the
    /// escape itself. Trailing escapes are ignored as if followed by a non-significant char.
    /// `delims` single char or an array of chars, which will be sorted, see the
    /// [free version][free] of this function for more control over delimiter sorting.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `esc == delim`
    ///
    /// # Complexity
    /// This algorithm requires `O(n * max(log m, 1))` time where `n` is the length of the input
    /// string and `m` is the length of the delimiters.
    ///
    /// # Allocation
    /// No allocations are done.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use strtools::StrTools;
    ///
    /// let value = r"Pa\rt0:Part1:Part2\:StillPart2";
    /// let parts: Vec<_> = value.split_non_escaped('\\', ':')?.collect();
    ///
    /// // no sanitization is done here the separators are simply ignored
    /// assert_eq!(parts, [r"Pa\rt0", "Part1", r"Part2\:StillPart2"]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [free]: split::non_escaped
    fn split_non_escaped<D: Into<Sorted<char, N>>, const N: usize>(
        &self,
        esc: char,
        delims: D,
    ) -> Result<split::NonEscaped<'_, N>, split::NonEscapedError>;

    /// Attempts to parse `T` from the beginning of the [`str`], returns the rest of the `input` and
    /// `T` if parsing succeeded.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the start of `input` contain any valid representation of `Self`
    /// - `input` did not contain a complete representation of `Self`
    ///
    /// # Examples
    /// ```
    /// use strtools::StrTools;
    ///
    /// let result = "-128 Look mom, no error!".parse_front::<i8>();
    /// assert_eq!(result, Ok((-128, " Look mom, no error!")));
    /// ```
    fn parse_front<T: FromStrFront>(&self) -> Result<(T, &str), T::Error>;

    /// Attempts to parse `T` from the end of the [`str`], returns the rest of the `input` and T` if
    /// parsing succeeded.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the start of `input` contain any valid representation of `Self`
    /// - `input` did not contain a complete representation of `Self`
    ///
    /// # Examples
    /// ```
    /// use strtools::StrTools;
    ///
    /// let result = "Look mom, no error! -128".parse_back::<i8>();
    /// assert_eq!(result, Ok((-128, "Look mom, no error! ")));
    /// ```
    fn parse_back<T: FromStrBack>(&self) -> Result<(T, &str), T::Error>;
}

impl StrTools for str {
    fn split_n_times<const N: usize>(&self, indices: &Sorted<usize, N>) -> ([&str; N], &str) {
        split::n_times(self, indices)
    }

    fn split_non_escaped_sanitize<D: Into<Sorted<char, N>>, const N: usize>(
        &self,
        esc: char,
        delims: D,
    ) -> Result<split::NonEscapedSanitize<'_, N>, split::NonEscapedError> {
        split::non_escaped_sanitize(self, esc, delims.into())
    }

    fn split_non_escaped<D: Into<Sorted<char, N>>, const N: usize>(
        &self,
        esc: char,
        delims: D,
    ) -> Result<split::NonEscaped<'_, N>, split::NonEscapedError> {
        split::non_escaped(self, esc, delims.into())
    }

    fn parse_front<T: FromStrFront>(&self) -> Result<(T, &str), T::Error> {
        T::from_str_front(self)
    }

    fn parse_back<T: FromStrBack>(&self) -> Result<(T, &str), T::Error> {
        T::from_str_back(self)
    }
}
