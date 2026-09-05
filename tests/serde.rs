//! Tests for the `serde` feature, driven through the public API.
#![cfg(feature = "serde")]

use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};

#[test]
fn round_trips_through_json() {
    let range = RangeSet::new_from_ranges(&[
        AnyRange::from(Char::new('a')..=Char::new('f')),
        AnyRange::from(Char::new('0')..=Char::new('9')),
    ]);

    let json = serde_json::to_string(&range).unwrap();
    assert_eq!("[\"0\",\"9\",\"a\",\"f\"]", json);

    let back: RangeSet<Char> = serde_json::from_str(&json).unwrap();
    assert_eq!(range, back);
    assert_eq!(range.to_regex(), back.to_regex());
}

#[test]
fn round_trips_the_edge_cases() {
    for range in [
        RangeSet::<Char>::empty(),
        RangeSet::<Char>::total(),
        RangeSet::new_from_range_char('\0'..='\0'),
        RangeSet::new_from_range_char(char::MAX..=char::MAX),
        // A range spanning the surrogate block, which is stored as one range.
        RangeSet::new_from_range_char('\u{D7FF}'..='\u{E000}'),
    ] {
        let json = serde_json::to_string(&range).unwrap();
        let back: RangeSet<Char> = serde_json::from_str(&json).unwrap();
        assert_eq!(
            range,
            back,
            "round trip of {} through {json}",
            range.to_regex()
        );
    }
}

/// `irange` validates the bounds when deserializing, so a malformed set cannot be built
/// from untrusted input.
#[test]
fn malformed_input_is_rejected() {
    for json in [
        // An odd number of bounds.
        "[\"a\"]",
        // An inverted range.
        "[\"z\",\"a\"]",
        // Unsorted ranges.
        "[\"x\",\"z\",\"a\",\"c\"]",
        // Adjacent ranges that should have been merged.
        "[\"a\",\"b\",\"c\",\"d\"]",
    ] {
        assert!(
            serde_json::from_str::<RangeSet<Char>>(json).is_err(),
            "{json} should not deserialize"
        );
    }
}

#[test]
fn a_char_serializes_as_a_char() {
    let c = Char::new('é');
    assert_eq!("\"é\"", serde_json::to_string(&c).unwrap());
    assert_eq!(c, serde_json::from_str::<Char>("\"é\"").unwrap());
}
