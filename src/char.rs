//! The `char` wrapper a [`RangeSet`](irange::RangeSet) stores.

use std::{
    char,
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};

use irange::integer::Bounded;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The first codepoint of the surrogate block, which holds no `char`.
pub(crate) const INVALID_MIN: u32 = 0xD800;
/// The last codepoint of the surrogate block.
pub(crate) const INVALID_MAX: u32 = 0xDFFF;
/// The number of codepoints in the surrogate block.
pub(crate) const INVALID_SIZE: u32 = INVALID_MAX - INVALID_MIN + 1;

/// The index of `codepoint` among all the `char` values.
///
/// The surrogate block holds no `char`, so codepoints above it are shifted down by
/// its size to make the indices contiguous.
const fn to_index(codepoint: u32) -> u32 {
    if codepoint >= INVALID_MIN {
        codepoint - INVALID_SIZE
    } else {
        codepoint
    }
}

/// The inverse of [`to_index`]: the codepoint at the given index.
const fn from_index(index: u32) -> u32 {
    if index >= INVALID_MIN {
        index + INVALID_SIZE
    } else {
        index
    }
}

/// A `char` that can be stored in a [`RangeSet`](irange::RangeSet).
///
/// `RangeSet` needs its element type to be a contiguous integer type: it steps from one
/// value to the next to decide whether two ranges touch. `char` is not contiguous, since
/// the surrogate block `U+D800..=U+DFFF` holds no `char`, so `Char` counts by *index*
/// rather than by codepoint and skips that block. `'\u{D7FF}'` and `'\u{E000}'` are
/// therefore adjacent, and the ranges `'\u{0}'..='\u{D7FF}'` and `'\u{E000}'..` are
/// stored merged as the total set.
///
/// # Example
///
/// ```
/// use regex_charclass::{Char, RangeSet};
///
/// let range = RangeSet::new_from_range(Char::new('a')..=Char::new('z'));
/// assert!(range.contains(Char::new('q')));
/// ```
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Char(char);

impl Char {
    /// The number of `char` values, which is the largest cardinality a set of `char`
    /// can have. It is smaller than the number of codepoints, since the surrogate block
    /// holds no `char`.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::{Char, CharacterClass, RangeSet};
    ///
    /// assert_eq!(1_112_064, Char::COUNT);
    /// assert_eq!(Char::COUNT, RangeSet::<Char>::total().get_cardinality());
    /// ```
    pub const COUNT: u32 = char::MAX as u32 + 1 - INVALID_SIZE;

    /// Create a new instance from the given `char`.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::Char;
    ///
    /// let c = Char::new('a');
    /// assert_eq!('a', c.to_char());
    /// ```
    #[inline]
    pub fn new(c: char) -> Self {
        Char(c)
    }

    /// Return the `char`.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::Char;
    ///
    /// let c = Char::new('a');
    /// assert_eq!('a', c.to_char());
    /// ```
    #[inline]
    pub fn to_char(&self) -> char {
        self.0
    }

    /// Create a new instance from the given codepoint, or return `None` if it is not a
    /// `char`, i.e. if it is a surrogate or above [`char::MAX`].
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::Char;
    ///
    /// assert_eq!(Some(Char::new('a')), Char::from_u32(97));
    /// assert_eq!(None, Char::from_u32(0xD800));
    /// ```
    #[inline]
    pub fn from_u32(c: u32) -> Option<Self> {
        Some(Char(char::from_u32(c)?))
    }

    /// Return the `char` code as a `u32`.
    ///
    /// # Example
    ///
    /// ```
    /// use regex_charclass::Char;
    ///
    /// let c = Char::new('a');
    /// assert_eq!(97, c.to_u32());
    /// ```
    #[inline]
    pub fn to_u32(&self) -> u32 {
        self.0 as u32
    }

    /// The next `char`, or `None` if this is [`char::MAX`].
    ///
    /// This is the successor in the order `RangeSet` counts in, so the `char` after
    /// `'\u{D7FF}'` is `'\u{E000}'`.
    #[inline]
    pub(crate) fn next(self) -> Option<Self> {
        Self::from_index(self.index() + 1)
    }

    /// The previous `char`, or `None` if this is `'\0'`.
    #[inline]
    pub(crate) fn previous(self) -> Option<Self> {
        Self::from_index(self.index().checked_sub(1)?)
    }

    /// The index of this `char` among all the `char` values, i.e. the number of `char`
    /// values below it. It is at most [`Char::COUNT`]` - 1`.
    #[inline]
    pub(crate) fn index(self) -> u32 {
        to_index(self.to_u32())
    }

    /// The `char` at the given index, the inverse of [`Char::index`], or `None` if the
    /// index is past the last `char`.
    #[inline]
    pub(crate) fn from_index(index: u32) -> Option<Self> {
        Self::from_u32(from_index(index))
    }
}

/// Print the `char` itself when it is printable ASCII, and its escaped codepoint
/// otherwise. This is a readable rendering of a single `char`, not a regular
/// expression; use [`CharacterClass::to_regex`](crate::CharacterClass::to_regex) for
/// that.
impl Display for Char {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if ('\u{20}'..='\u{7E}').contains(&self.0) {
            write!(f, "{}", self.0)
        } else {
            write!(f, "\\u{{{:04x}}}", self.to_u32())
        }
    }
}

/// Add two `char` values by index, so that `c + Char::one()` is the `char` after `c`.
///
/// # Panics
///
/// Panics if the sum is past [`char::MAX`], like the built-in integers do in debug
/// builds. `RangeSet` never adds past the maximum value.
impl Add<Char> for Char {
    type Output = Char;

    fn add(self, rhs: Self) -> Self::Output {
        Char::from_index(self.index() + rhs.index()).expect("attempt to add with overflow")
    }
}

/// Subtract two `char` values by index, so that `c - Char::one()` is the `char` before
/// `c`. It is the inverse of the [`Add`] implementation.
///
/// # Panics
///
/// Panics if the difference is below `'\0'`, like the built-in integers do in debug
/// builds. `RangeSet` never subtracts past the minimum value.
impl Sub<Char> for Char {
    type Output = Char;

    fn sub(self, rhs: Self) -> Self::Output {
        let index = self
            .index()
            .checked_sub(rhs.index())
            .expect("attempt to subtract with overflow");

        Char::from_index(index).expect("attempt to subtract with overflow")
    }
}

impl AddAssign for Char {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Bounded for Char {
    #[inline]
    fn min_value() -> Self {
        Char('\0')
    }

    #[inline]
    fn max_value() -> Self {
        Char(char::MAX)
    }

    /// The distance between two adjacent `char` values, which is one *index* rather
    /// than one codepoint. See the [type documentation](Char).
    #[inline]
    fn one() -> Self {
        Char('\u{1}')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_add() {
        assert_eq!(Char::new('\u{3}'), Char::new('\u{2}') + Char::one());
        assert_eq!(Char::new('\u{E000}'), Char::new('\u{D7FF}') + Char::one());
        assert_eq!(Char::new('\u{E001}'), Char::new('\u{E000}') + Char::one());
    }

    #[test]
    fn char_add_assign() {
        let mut c = Char::new('\u{2}');
        c += Char::one();
        assert_eq!(Char::new('\u{3}'), c);

        let mut c = Char::new('\u{D7FF}');
        c += Char::one();
        assert_eq!(Char::new('\u{E000}'), c);

        let mut c = Char::new('\u{E000}');
        c += Char::one();
        assert_eq!(Char::new('\u{E001}'), c);
    }

    #[test]
    fn char_sub() {
        assert_eq!(Char::new('\u{2}'), Char::new('\u{3}') - Char::one());
        assert_eq!(Char::new('\u{D7FF}'), Char::new('\u{E000}') - Char::one());
        assert_eq!(Char::new('\u{E000}'), Char::new('\u{E001}') - Char::one());
    }

    /// The two used to disagree: `Add` worked on raw codepoints and `Sub` on indices,
    /// so `(a + b) - b` could land on another `char`.
    #[test]
    fn add_and_sub_are_inverses_across_the_surrogate_block() {
        for a in [0u32, 1, 0xD7FE, 0xD7FF, 0xE000, 0xE001, 0x10FFFF] {
            for b in [0u32, 1, 0x1000, 0xD7FF, 0xE000] {
                let (a, b) = (Char::from_u32(a).unwrap(), Char::from_u32(b).unwrap());
                if a.index() < b.index() {
                    continue;
                }
                assert_eq!(a, (a - b) + b, "({a} - {b}) + {b}");
            }
        }
    }

    #[test]
    fn next_and_previous_skip_the_surrogate_block() {
        assert_eq!(Some(Char::new('\u{E000}')), Char::new('\u{D7FF}').next());
        assert_eq!(
            Some(Char::new('\u{D7FF}')),
            Char::new('\u{E000}').previous()
        );
        assert_eq!(None, Char::max_value().next());
        assert_eq!(None, Char::min_value().previous());
    }

    #[test]
    fn indices_are_contiguous() {
        assert_eq!(0, Char::min_value().index());
        assert_eq!(Char::COUNT - 1, Char::max_value().index());
        assert_eq!(
            Char::new('\u{D7FF}').index() + 1,
            Char::new('\u{E000}').index()
        );
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn add_past_the_last_char_panics() {
        let _ = Char::max_value() + Char::one();
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn sub_below_the_first_char_panics() {
        let _ = Char::min_value() - Char::one();
    }

    #[test]
    fn display_covers_printable_ascii() {
        assert_eq!("~", Char::new('~').to_string());
        assert_eq!(" ", Char::new(' ').to_string());
        assert_eq!("\\u{007f}", Char::new('\u{7F}').to_string());
        assert_eq!("\\u{000a}", Char::new('\n').to_string());
    }
}
