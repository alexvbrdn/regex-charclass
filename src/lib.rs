pub mod char;
mod tokens;
use std::ops::{Bound, RangeBounds};

use char::{Char, INVALID_MIN, INVALID_SIZE};
use irange::{integer::Bounded, RangeSet};
use tokens::identify_character;

pub use irange;

/// A trait for `RangeSet<Char>` to hold ranges of `char`.
/// 
/// # Example:
/// 
/// ```
/// use regex_charclass::{irange::{RangeSet, range::AnyRange}, char::Char, CharacterClass};
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
/// 
/// assert_eq!(".", range2.union(&range2_complement).to_regex());
/// assert_eq!("[]", range2.intersection(&range2_complement).to_regex());
/// 
/// assert_eq!("[g-z]", range1.difference(&range2).to_regex());
/// ```
pub trait CharacterClass: Sized {
    fn new_from_range_u32<R: RangeBounds<u32>>(range: R) -> Option<Self>;

    fn new_from_range_char<R: RangeBounds<char>>(range: R) -> Self;

    fn get_cardinality(&self) -> u32;

    fn to_regex(&self) -> String;
}

impl CharacterClass for RangeSet<Char> {
    /// Create a new instance from the given range of `u32`, return `None` if the `char` codes are invalid.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::{irange::RangeSet, CharacterClass};
    ///  
    /// let range = RangeSet::new_from_range_u32(97..=122);
    /// ```
    #[inline]
    fn new_from_range_u32<R: RangeBounds<u32>>(range: R) -> Option<Self> {
        let min = to_lowerbound_u32(range.start_bound())?;
        let max = to_upperbound_u32(range.end_bound())?;

        Some(RangeSet::new_from_range(min..=max))
    }

    /// Create a new instance from the given range of `char`.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::{irange::RangeSet, CharacterClass};
    ///  
    /// let range = RangeSet::new_from_range_char('a'..='z');
    /// ```
    #[inline]
    fn new_from_range_char<R: RangeBounds<char>>(range: R) -> Self {
        let min = to_lowerbound_char(range.start_bound());
        let max = to_upperbound_char(range.end_bound());

        RangeSet::new_from_range(min..=max)
    }

    /// Return the number of possible `char` contained.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::{char::Char, irange::RangeSet, CharacterClass};
    ///  
    /// let range = RangeSet::new_from_range_char('a'..='z');
    /// assert_eq!(26, range.get_cardinality());
    /// ```
    #[inline]
    fn get_cardinality(&self) -> u32 {
        let mut cardinality = 0;
        for r in (0..self.0.len()).step_by(2) {
            let mut minuhend = self.0[r + 1].to_u32();
            if minuhend >= INVALID_MIN {
                minuhend -= INVALID_SIZE;
            }
            let mut subtrahend = self.0[r].to_u32();
            if subtrahend >= INVALID_MIN {
                subtrahend -= INVALID_SIZE;
            }
            cardinality += minuhend - subtrahend + 1;
        }
        cardinality
    }

    /// Return a valid regular expression character class.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::{irange::{RangeSet, range::AnyRange}, char::Char, CharacterClass};
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
    /// ```
    #[inline]
    fn to_regex(&self) -> String {
        let range = self.clone();
        if self.is_empty() {
            String::from("[]")
        } else if range.is_total() {
            String::from(".")
        } else if let Some(token) = tokens::identify_class(self) {
            token.to_owned()
        } else {
            convert_to_regex(&range)
        }
    }
}

fn to_lowerbound_u32(bound: Bound<&u32>) -> Option<Char> {
    match bound {
        Bound::Included(t) => Char::from_u32(*t),
        Bound::Excluded(t) => {
            char::from_u32(*t)?;

            if let Some(c) = Char::from_u32(*t + 1) {
                Some(c)
            } else {
                Some(Char::new('\u{E000}'))
            }
        }
        Bound::Unbounded => Some(Char::min_value()),
    }
}

fn to_upperbound_u32(bound: Bound<&u32>) -> Option<Char> {
    match bound {
        Bound::Included(t) => Char::from_u32(*t),
        Bound::Excluded(t) => {
            char::from_u32(*t)?;

            if let Some(c) = Char::from_u32(*t - 1) {
                Some(c)
            } else {
                Some(Char::new('\u{D7FF}'))
            }
        }
        Bound::Unbounded => Some(Char::min_value()),
    }
}

fn to_lowerbound_char(bound: Bound<&char>) -> Char {
    match bound {
        Bound::Included(t) => Char::new(*t),
        Bound::Excluded(t) => {
            if let Some(c) = Char::from_u32(*t as u32 + 1) {
                c
            } else {
                Char::new('\u{E000}')
            }
        }
        Bound::Unbounded => Char::min_value(),
    }
}

fn to_upperbound_char(bound: Bound<&char>) -> Char {
    match bound {
        Bound::Included(t) => Char::new(*t),
        Bound::Excluded(t) => {
            if let Some(c) = Char::from_u32(*t as u32 - 1) {
                c
            } else {
                Char::new('\u{D7FF}')
            }
        }
        Bound::Unbounded => Char::min_value(),
    }
}

fn convert_to_regex(range: &RangeSet<Char>) -> String {
    let mut sb = String::new();

    let is_complement;
    let range_to_use;
    let complement = range.complement();
    if complement.0.len() < range.0.len() {
        range_to_use = &complement;
        is_complement = true;
    } else {
        range_to_use = range;
        is_complement = false;
    }

    for r in (0..range_to_use.0.len()).step_by(2) {
        let (min, max) = (range_to_use.0[r], range_to_use.0[r + 1]);
        if min == max {
            sb.push_str(get_printable_char(min.to_char()).as_str());
        } else if min + Char::one() == max {
            sb.push_str(
                format!(
                    "{}{}",
                    get_printable_char(min.to_char()),
                    get_printable_char(max.to_char())
                )
                .as_str(),
            );
        } else {
            sb.push_str(
                format!(
                    "{}-{}",
                    get_printable_char(min.to_char()),
                    get_printable_char(max.to_char())
                )
                .as_str(),
            );
        }
    }

    if is_complement || range_to_use.0.len() > 2 || range_to_use.0[0] != range_to_use.0[1] {
        if is_complement {
            return format!("[^{}]", sb);
        } else {
            return format!("[{}]", sb);
        }
    }

    sb
}

fn get_printable_char(character: char) -> String {
    if ('\u{20}'..'\u{7E}').contains(&character) {
        if character == '*'
            || character == '+'
            || character == '?'
            || character == '('
            || character == ')'
            || character == '['
            || character == ']'
            || character == '{'
            || character == '}'
            || character == '|'
            || character == '\\'
            || character == '-'
            || character == '^'
            || character == '.'
        {
            format!("\\{}", character)
        } else {
            format!("{}", character)
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
    fn test_empty_and_total() -> Result<(), String> {
        let range = RangeSet::<Char>::empty();
        assert!(range.is_empty());
        assert_eq!("[]", range.to_regex());
        assert_eq!(0, range.get_cardinality());

        let range = RangeSet::<Char>::total();
        assert!(range.is_total());
        assert_eq!(".", range.to_regex());
        assert_eq!(1_112_064, range.get_cardinality());
        Ok(())
    }

    #[test]
    fn test_operations() -> Result<(), String> {
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

        Ok(())
    }

    #[test]
    fn test_to_regex() -> Result<(), String> {
        let range = RangeSet::<Char>::new_from_range_char('.'..='.');
        assert_eq!("\\.", range.to_regex());

        let range = RangeSet::<Char>::new_from_ranges(&[
            AnyRange::from(Char::new('0')..=Char::new('9')),
            AnyRange::from(Char::new('A')..=Char::new('F')),
            AnyRange::from(Char::new('a')..=Char::new('f')),
        ]);
        assert_eq!("\\p{ASCII_Hex_Digit}", range.to_regex());

        Ok(())
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() -> Result<(), String> {
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
        Ok(())
    }
}
