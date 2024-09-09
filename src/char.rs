use std::{
    char,
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};

use irange::integer::Bounded;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub(super) static INVALID_MIN: u32 = 0xD800;
pub(super) static INVALID_MAX: u32 = 0xDFFF;
pub(super) static INVALID_SIZE: u32 = 0x800;

/// A structure holding a `char` to use within a `RangeSet`.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Char(char);

impl Char {
    /// Create a new instance from the given `char`.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::char::Char;
    ///  
    /// let c = Char::new('a');
    /// ```
    #[inline]
    pub fn new(c: char) -> Self {
        Char(c)
    }

    /// Return the `char`.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::char::Char;
    ///  
    /// let c = Char::new('a');
    /// assert_eq!('a', c.to_char());
    /// ```
    #[inline]
    pub fn to_char(&self) -> char {
        self.0
    }

    /// Create a new instance from the given `char`.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::char::Char;
    ///  
    /// let c = Char::from_u32(97);
    /// ```
    #[inline]
    pub fn from_u32(c: u32) -> Option<Self> {
        Some(Char(char::from_u32(c)?))
    }

    /// Return the `char` code as a `u32`.
    ///
    /// # Example:
    ///
    /// ```
    /// use regex_charclass::char::Char;
    ///  
    /// let c = Char::new('a');
    /// assert_eq!(97, c.to_u32());
    /// ```
    #[inline]
    pub fn to_u32(&self) -> u32 {
        self.0 as u32
    }
}

impl Display for Char {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if ('\u{20}'..'\u{7E}').contains(&self.0) {
            write!(f, "{}", self.0)
        } else {
            write!(f, "\\u{{{:04x}}}", self.to_u32())
        }
    }
}

impl Add<Char> for Char {
    type Output = Char;

    fn add(self, rhs: Self) -> Self::Output {
        let mut sum = self.0 as u32 + rhs.0 as u32;
        if sum >= INVALID_MIN && sum <= INVALID_MAX {
            sum = INVALID_MAX + 1 + sum - INVALID_MIN;
        }
        if let Some(new_char) = char::from_u32(sum) {
            Char(new_char)
        } else {
            panic!("attempt to add with overflow");
        }
    }
}

impl Sub<Char> for Char {
    type Output = Char;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut minuhend = self.0 as u32;
        if minuhend >= INVALID_MIN {
            minuhend -= INVALID_SIZE;
        }
        let mut subtrahend = rhs.0 as u32;
        if subtrahend >= INVALID_MIN {
            subtrahend -= INVALID_SIZE;
        }
        let mut sub = minuhend - subtrahend;
        if sub >= INVALID_MIN {
            sub += INVALID_SIZE;
        }
        if let Some(new_char) = char::from_u32(sub) {
            Char(new_char)
        } else {
            panic!("attempt to sub with overflow");
        }
    }
}

impl AddAssign for Char {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (*self + rhs).0;
    }
}

impl Bounded for Char {
    fn min_value() -> Self {
        Char('\0')
    }

    fn max_value() -> Self {
        Char(char::MAX)
    }

    fn one() -> Self {
        Char('\u{1}')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_add() -> Result<(), String> {
        assert_eq!(Char::new('\u{3}'), Char::new('\u{2}') + Char::one());
        assert_eq!(Char::new('\u{E000}'), Char::new('\u{D7FF}') + Char::one());
        assert_eq!(Char::new('\u{E001}'), Char::new('\u{E000}') + Char::one());

        Ok(())
    }

    #[test]
    fn char_add_assign() -> Result<(), String> {
        let mut c = Char::new('\u{2}');
        c += Char::one();
        assert_eq!(Char::new('\u{3}'), c);

        let mut c = Char::new('\u{D7FF}');
        c += Char::one();
        assert_eq!(Char::new('\u{E000}'), c);

        let mut c = Char::new('\u{E000}');
        c += Char::one();
        assert_eq!(Char::new('\u{E001}'), c);

        Ok(())
    }

    #[test]
    fn char_sub() -> Result<(), String> {
        assert_eq!(Char::new('\u{2}'), Char::new('\u{3}') - Char::one());
        assert_eq!(Char::new('\u{D7FF}'), Char::new('\u{E000}') - Char::one());
        assert_eq!(Char::new('\u{E000}'), Char::new('\u{E001}') - Char::one());

        Ok(())
    }
}
