//! Manipulate and convert regular expression character classes.
//!
//! A character class is a set of `char` values. This crate stores one as an
//! [`irange::RangeSet`] of [`Char`], which keeps it as a sorted collection
//! of non-overlapping inclusive ranges, and adds the operations a regular expression
//! engine needs on top of it through the [`CharacterClass`] trait: building a set from
//! a range of `char` or of codepoints, counting its members, and rendering it back as
//! the shortest character class it can.
//!
//! Set algebra — union, intersection, difference and complement — comes from
//! [`irange`]. [`RangeSet`], [`AnyRange`] and [`Char`] are re-exported at the crate
//! root, so a dependent needs neither its own dependency on `irange` nor a version kept
//! in step with this crate's. The whole crate is also re-exported as
//! [`regex_charclass::irange`](irange).
//!
//! # Example
//!
//! ```
//! use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};
//!
//! let letters = RangeSet::new_from_range_char('a'..='z');
//! assert_eq!(26, letters.get_cardinality());
//! assert_eq!("[a-z]", letters.to_regex());
//!
//! let hex = RangeSet::new_from_ranges(&[
//!     AnyRange::from(Char::new('0')..=Char::new('9')),
//!     AnyRange::from(Char::new('A')..=Char::new('F')),
//!     AnyRange::from(Char::new('a')..=Char::new('f')),
//! ]);
//! // A set matching a named Unicode property is rendered as that property.
//! assert_eq!("\\p{ASCII_Hex_Digit}", hex.to_regex());
//! assert_eq!("\\P{ASCII_Hex_Digit}", hex.complement().to_regex());
//!
//! assert_eq!(".", hex.union(&hex.complement()).to_regex());
//! assert_eq!("[]", hex.intersection(&hex.complement()).to_regex());
//! assert_eq!("[g-z]", letters.difference(&hex).to_regex());
//! ```
//!
//! # The generated regular expression
//!
//! [`to_regex`](CharacterClass::to_regex) returns the shortest form it finds for a set,
//! in this order:
//!
//! | Form | When |
//! |---|---|
//! | `[]` | the set is empty |
//! | `.` | the set holds every `char` |
//! | `\n`, `\r`, `\t`, `\v` | the set holds only that one `char` |
//! | `\d`, `\s`, `\w`, `\D`, `\S`, `\W` | the set is exactly that Perl class |
//! | `\p{Name}`, `\P{Name}` | the set is exactly a Unicode general category, script or boolean property |
//! | `a`, `\.` | the set holds one `char`, escaped if it is a metacharacter |
//! | `[a-z0-9]`, `[^a-z]` | otherwise, complemented when that is shorter |
//!
//! Three of them need care from whoever reads them back:
//!
//! - `[]` is a placeholder for the empty set, not a pattern. An empty bracketed class is
//!   a parse error in every engine, so a set that may be empty has to be handled before
//!   the output reaches one. `[^\s\S]` is the usual pattern for a class matching
//!   nothing.
//! - `.` stands for the total set, which includes `\n`, so the pattern needs
//!   dot-matches-newline (`(?s)`).
//! - `\p{Name}` and the Perl classes stand for Unicode sets, so the pattern needs
//!   Unicode mode, which is the default in most engines. The tables here are generated
//!   from UCD 17.0.0, so an engine built against an older version may resolve a name to
//!   a slightly different set.
//!
//! # The surrogate block
//!
//! `char` is not contiguous: the codepoints `U+D800..=U+DFFF` are surrogates and hold no
//! `char`. [`Char`] counts by index rather than by codepoint and skips that
//! block, so `'\u{D7FF}'` and `'\u{E000}'` are adjacent and the total set is one range
//! of 1,112,064 values rather than two. See [`Char`] for the details.
//!
//! # Feature flags
//!
//! - `serde`: implement `Serialize` and `Deserialize` for [`Char`], and
//!   enable the same feature on `irange` so that a `RangeSet<Char>` round-trips.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod char;
mod tokens;

/// The `rust` code blocks in `README.md`, compiled and run as doctests so that the
/// README cannot drift from the API. It exists only while rustdoc is collecting
/// doctests, so it is not part of the crate or of its documentation.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct Readme;

use std::ops::{Bound, RangeBounds};

use irange::integer::Bounded;
use tokens::identify_character;

pub use crate::char::Char;
pub use irange;
/// The set type this crate operates on, and the range type its constructors take, are
/// `irange`'s. They are re-exported here so that a dependent needs neither its own
/// dependency on `irange` nor a version kept in step with this crate's.
pub use irange::{range::AnyRange, RangeSet};

/// The `char` set operations a [`RangeSet<Char>`](irange::RangeSet) gains.
///
/// This trait is implemented for `RangeSet<Char>` and is not meant to be implemented
/// elsewhere. Import it to call the methods below on a set of `char`.
///
/// # Example
///
/// ```
/// use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};
///
/// let range1 = RangeSet::new_from_range_char('a'..='z');
/// assert_eq!(26, range1.get_cardinality());
/// assert_eq!("[a-z]", range1.to_regex());
///
/// let range2 = RangeSet::new_from_ranges(&[
///     AnyRange::from(Char::new('0')..=Char::new('9')),
///     AnyRange::from(Char::new('A')..=Char::new('F')),
///     AnyRange::from(Char::new('a')..=Char::new('f')),
/// ]);
/// assert_eq!("\\p{ASCII_Hex_Digit}", range2.to_regex());
///
/// let range2_complement = range2.complement();
/// assert_eq!("\\P{ASCII_Hex_Digit}", range2_complement.to_regex());
///
/// assert_eq!(".", range2.union(&range2_complement).to_regex());
/// assert_eq!("[]", range2.intersection(&range2_complement).to_regex());
///
/// assert_eq!("[g-z]", range1.difference(&range2).to_regex());
/// ```
pub trait CharacterClass: Sized {
    /// Create a new instance from the given range of codepoints, or return `None` if a
    /// bound is not a `char`.
    ///
    /// A range holding no codepoint, such as `0..0`, gives the empty set.
    fn new_from_range_u32<R: RangeBounds<u32>>(range: R) -> Option<Self>;

    /// Create a new instance from the given range of `char`.
    ///
    /// A range holding no `char`, such as `'a'..'a'`, gives the empty set.
    fn new_from_range_char<R: RangeBounds<char>>(range: R) -> Self;

    /// Return the number of `char` values contained, at most 1,112,064.
    fn get_cardinality(&self) -> u32;

    /// Return the set as a regular expression character class.
    ///
    /// See [the crate documentation](crate#the-generated-regular-expression) for the
    /// forms this returns and the flags they assume.
    fn to_regex(&self) -> String;
}

impl CharacterClass for RangeSet<Char> {
    /// Create a new instance from the given range of codepoints, or return `None` if a
    /// bound is not a `char`, i.e. if it is a surrogate or above [`char::MAX`].
    ///
    /// A range holding no codepoint gives the empty set.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::{CharacterClass, RangeSet};
    ///
    /// let range = RangeSet::new_from_range_u32(97..=122).unwrap();
    /// assert_eq!("[a-z]", range.to_regex());
    ///
    /// // An empty range, and a bound that is not a `char`.
    /// assert!(RangeSet::new_from_range_u32(97..97).unwrap().is_empty());
    /// assert_eq!(None, RangeSet::new_from_range_u32(0xD800..=0xD800));
    /// ```
    #[inline]
    fn new_from_range_u32<R: RangeBounds<u32>>(range: R) -> Option<Self> {
        let min = match range.start_bound() {
            Bound::Included(&t) => Some(Char::from_u32(t)?),
            // A `char` may be excluded, but only if it is one in the first place.
            Bound::Excluded(&t) => Char::from_u32(t)?.next(),
            Bound::Unbounded => Some(Char::min_value()),
        };
        let max = match range.end_bound() {
            Bound::Included(&t) => Some(Char::from_u32(t)?),
            Bound::Excluded(&t) => Char::from_u32(t)?.previous(),
            Bound::Unbounded => Some(Char::max_value()),
        };

        Some(new_from_bounds(min, max))
    }

    /// Create a new instance from the given range of `char`.
    ///
    /// A range holding no `char` gives the empty set.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::{CharacterClass, RangeSet};
    ///
    /// let range = RangeSet::new_from_range_char('a'..='z');
    /// assert_eq!("[a-z]", range.to_regex());
    ///
    /// assert_eq!("[a-y]", RangeSet::new_from_range_char('a'..'z').to_regex());
    /// assert!(RangeSet::new_from_range_char('a'..'a').is_empty());
    /// ```
    #[inline]
    fn new_from_range_char<R: RangeBounds<char>>(range: R) -> Self {
        let min = match range.start_bound() {
            Bound::Included(&t) => Some(Char::new(t)),
            Bound::Excluded(&t) => Char::new(t).next(),
            Bound::Unbounded => Some(Char::min_value()),
        };
        let max = match range.end_bound() {
            Bound::Included(&t) => Some(Char::new(t)),
            Bound::Excluded(&t) => Char::new(t).previous(),
            Bound::Unbounded => Some(Char::max_value()),
        };

        new_from_bounds(min, max)
    }

    /// Return the number of `char` values contained, at most 1,112,064.
    ///
    /// The surrogate block holds no `char`, so a range spanning it does not count it.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::{CharacterClass, RangeSet};
    ///
    /// let range = RangeSet::new_from_range_char('a'..='z');
    /// assert_eq!(26, range.get_cardinality());
    ///
    /// assert_eq!(1_112_064, RangeSet::total().get_cardinality());
    /// ```
    #[inline]
    fn get_cardinality(&self) -> u32 {
        self.0
            .chunks_exact(2)
            .map(|bounds| {
                bounds[1]
                    .index()
                    .checked_sub(bounds[0].index())
                    .map_or(0, |size| size + 1)
            })
            .fold(0, u32::saturating_add)
    }

    /// Return the set as a regular expression character class.
    ///
    /// See [the crate documentation](crate#the-generated-regular-expression) for the
    /// forms this returns and the flags they assume.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};
    ///
    /// let range = RangeSet::new_from_range_char('a'..='z');
    /// assert_eq!("[a-z]", range.to_regex());
    ///
    /// let range = RangeSet::<Char>::new_from_ranges(&[
    ///     AnyRange::from(Char::new('0')..=Char::new('9')),
    ///     AnyRange::from(Char::new('A')..=Char::new('F')),
    ///     AnyRange::from(Char::new('a')..=Char::new('f')),
    /// ]);
    /// assert_eq!("\\p{ASCII_Hex_Digit}", range.to_regex());
    ///
    /// // A single `char` needs no class, but is escaped if it is a metacharacter.
    /// assert_eq!("\\.", RangeSet::new_from_range_char('.'..='.').to_regex());
    /// ```
    #[inline]
    fn to_regex(&self) -> String {
        if self.is_empty() {
            String::from("[]")
        } else if self.is_total() {
            String::from(".")
        } else if let Some(token) = tokens::identify_class(self) {
            token
        } else {
            convert_to_regex(self)
        }
    }
}

/// The set of every `char` from `min` to `max`, or the empty set if either bound fell
/// outside `char` or the range is inverted.
#[inline]
fn new_from_bounds(min: Option<Char>, max: Option<Char>) -> RangeSet<Char> {
    match (min, max) {
        // `new_from_range` gives the empty set for an inverted range.
        (Some(min), Some(max)) => RangeSet::new_from_range(min..=max),
        _ => RangeSet::empty(),
    }
}

/// Render a set that matched no named class as a bracketed character class, or as a
/// single escaped `char` when it holds only one.
fn convert_to_regex(range: &RangeSet<Char>) -> String {
    // A set with few gaps is shorter written as the complement of those gaps.
    let complement = range.complement();
    let (range_to_use, is_complement) = if complement.0.len() < range.0.len() {
        (&complement, true)
    } else {
        (range, false)
    };

    let mut sb = String::new();
    // Whether what was written is a single `char`, which needs no brackets around it.
    let mut is_single_char = true;

    for bounds in range_to_use.0.chunks_exact(2) {
        let (min, max) = (bounds[0], bounds[1]);
        if min == max {
            is_single_char = sb.is_empty();
            sb.push_str(&get_printable_char(min.to_char()));
            continue;
        }

        is_single_char = false;
        let separator = if min.next() == Some(max) { "" } else { "-" };
        sb.push_str(&get_printable_char(min.to_char()));
        sb.push_str(separator);
        sb.push_str(&get_printable_char(max.to_char()));
    }

    if sb.is_empty() {
        // Only reachable from a malformed set, whose bounds do not pair up.
        String::from("[]")
    } else if is_complement {
        format!("[^{sb}]")
    } else if is_single_char {
        sb
    } else {
        format!("[{sb}]")
    }
}

/// A `char` as it appears inside a character class: itself when it is printable ASCII,
/// escaped when it is a metacharacter, and its codepoint otherwise.
fn get_printable_char(character: char) -> String {
    if ('\u{20}'..='\u{7E}').contains(&character) {
        // Escaped whether or not the `char` ends up inside brackets, since `-` and `^`
        // are special inside them and `$` and `.` are special outside them.
        if matches!(
            character,
            '*' | '+'
                | '?'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '|'
                | '\\'
                | '-'
                | '^'
                | '.'
                | '$'
        ) {
            format!("\\{character}")
        } else {
            String::from(character)
        }
    } else if let Some(c) = identify_character(character) {
        c.to_owned()
    } else {
        format!("\\u{{{:04x}}}", character as u32)
    }
}

#[cfg(test)]
mod tests {
    use irange::range::AnyRange;

    use super::*;

    #[test]
    fn test_empty_and_total() {
        let range = RangeSet::<Char>::empty();
        assert!(range.is_empty());
        assert_eq!("[]", range.to_regex());
        assert_eq!(0, range.get_cardinality());

        let range = RangeSet::<Char>::total();
        assert!(range.is_total());
        assert_eq!(".", range.to_regex());
        assert_eq!(Char::COUNT, range.get_cardinality());
        assert_eq!(1_112_064, range.get_cardinality());
    }

    #[test]
    fn test_operations() {
        let range1 = RangeSet::new_from_range_char('a'..='z');
        assert_eq!("[a-z]", range1.to_regex());

        for char in range1.iter() {
            assert!(range1.contains(char))
        }

        let range2 = RangeSet::<Char>::new_from_ranges(&[
            AnyRange::from(Char::new('0')..Char::new('2')),
            AnyRange::from(Char::new('A')..=Char::new('F')),
            AnyRange::from(Char::new('a')..=Char::new('f')),
        ]);
        assert_eq!("[01A-Fa-f]", range2.to_regex());

        for char in range2.iter() {
            assert!(range2.contains(char))
        }

        let intersection = range1.intersection(&range2);
        assert_eq!("[a-f]", intersection.to_regex());

        for char in intersection.iter() {
            assert!(intersection.contains(char))
        }
    }

    #[test]
    fn test_to_regex() {
        let range = RangeSet::<Char>::new_from_range_char('.'..='.');
        assert_eq!("\\.", range.to_regex());

        let range = RangeSet::<Char>::new_from_ranges(&[
            AnyRange::from(Char::new('0')..=Char::new('9')),
            AnyRange::from(Char::new('A')..=Char::new('F')),
            AnyRange::from(Char::new('a')..=Char::new('f')),
        ]);
        assert_eq!("\\p{ASCII_Hex_Digit}", range.to_regex());
    }

    /// `$` used to be written bare, which reads as an anchor rather than as the `char`.
    #[test]
    fn metacharacters_are_escaped() {
        for character in [
            '*', '+', '?', '(', ')', '[', ']', '{', '}', '|', '\\', '-', '^', '.', '$',
        ] {
            let range = RangeSet::new_from_range_char(character..=character);
            assert_eq!(format!("\\{character}"), range.to_regex());
        }
    }

    /// `~` is printable, but the check used to exclude it and wrote its codepoint.
    #[test]
    fn the_whole_printable_ascii_range_is_written_as_itself() {
        assert_eq!("~", RangeSet::new_from_range_char('~'..='~').to_regex());
        assert_eq!("[ -~]", RangeSet::new_from_range_char(' '..='~').to_regex());
    }

    /// The bounds are a public field, so a malformed set can reach any method.
    #[test]
    fn no_method_panics_on_a_malformed_set() {
        let malformed = [
            // An odd number of bounds.
            vec![Char::new('a')],
            // An inverted range.
            vec![Char::new('z'), Char::new('a')],
            // Unsorted ranges.
            vec![
                Char::new('x'),
                Char::new('z'),
                Char::new('a'),
                Char::new('c'),
            ],
            // Ranges that should have been merged.
            vec![
                Char::new('a'),
                Char::new('b'),
                Char::new('c'),
                Char::new('d'),
            ],
        ];

        for bounds in malformed {
            let range = RangeSet(bounds);
            let _ = range.get_cardinality();
            let _ = range.to_regex();
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        let range = RangeSet::empty();
        let serialized = serde_json::to_string(&range).unwrap();
        let unserialized: RangeSet<Char> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(range, unserialized);

        let range = RangeSet::<Char>::total();
        let serialized = serde_json::to_string(&range).unwrap();
        let unserialized: RangeSet<Char> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(range, unserialized);

        let range = RangeSet::new_from_ranges(&[
            AnyRange::from(Char::new('3')..=Char::new('4')),
            AnyRange::from(Char::new('7')..Char::new('9')),
        ]);
        let serialized = serde_json::to_string(&range).unwrap();
        let unserialized: RangeSet<Char> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(range, unserialized);
    }
}
