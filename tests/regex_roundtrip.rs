//! Check what `to_regex` produces against a real regular expression engine.
//!
//! The output is only useful if an engine reads it back as the same set of `char`, so
//! every generated class is compiled with the `regex` crate and asked about the `char`
//! values around each of its bounds, where an off-by-one or a missing escape shows up.
//!
//! Two forms are only checked for compiling, not for membership: `\p{Name}` and the
//! Perl classes name Unicode sets, and this crate's tables come from the UCD version a
//! `ucd-*` feature selects while the `regex` crate carries its own copy. Random sets
//! essentially never hit them, so this costs almost no coverage.

use rand::{RngExt, SeedableRng};
use regex::Regex;
use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};

/// Compile a class the way an engine that used it in a larger pattern would, anchored so
/// that it matches one whole `char`. `(?s)` makes `.` match `\n` too, which is what the
/// total set means here.
fn compile(class: &str) -> Regex {
    Regex::new(&format!("(?s)\\A(?:{class})\\z"))
        .unwrap_or_else(|error| panic!("{class} is not a valid regular expression: {error}"))
}

/// Whether the class names a Unicode set, whose contents depend on the engine's own UCD.
fn is_named_class(class: &str) -> bool {
    ["\\p", "\\P", "\\d", "\\D", "\\s", "\\S", "\\w", "\\W"]
        .iter()
        .any(|prefix| class.starts_with(prefix))
}

/// The `char` values worth asking about: each bound, and its neighbours on either side.
fn interesting_chars(range: &RangeSet<Char>) -> Vec<char> {
    let mut chars = vec!['\0', 'a', '0', '\n', char::MAX];

    for bound in &range.0 {
        chars.push(bound.to_char());
        if let Some(before) = char::from_u32(bound.to_u32().saturating_sub(1)) {
            chars.push(before);
        }
        if let Some(after) = char::from_u32(bound.to_u32() + 1) {
            chars.push(after);
        }
    }

    chars.sort_unstable();
    chars.dedup();
    chars
}

/// The generated class must match exactly the `char` values the set holds.
fn assert_round_trips(range: &RangeSet<Char>) {
    let class = range.to_regex();

    // `[]` is this crate's placeholder for the empty set rather than a pattern: an empty
    // bracketed class is a parse error in every engine. See the crate documentation.
    if class == "[]" {
        assert!(range.is_empty());
        return;
    }

    let regex = compile(&class);
    if is_named_class(&class) {
        return;
    }

    for character in interesting_chars(range) {
        let expected = range.contains(Char::new(character));
        let matched = regex.is_match(&character.to_string());
        assert_eq!(
            expected,
            matched,
            "{class} {} U+{:04X}, but the set does{} hold it",
            if matched { "matches" } else { "rejects" },
            character as u32,
            if expected { "" } else { " not" },
        );
    }
}

#[test]
fn the_documented_forms_round_trip() {
    let cases: Vec<RangeSet<Char>> = vec![
        RangeSet::empty(),
        RangeSet::total(),
        RangeSet::new_from_range_char('a'..='a'),
        RangeSet::new_from_range_char('a'..='z'),
        RangeSet::new_from_range_char('a'..='b'),
        RangeSet::new_from_range_char('\0'..='\0'),
        RangeSet::new_from_range_char(char::MAX..=char::MAX),
        RangeSet::new_from_range_char('a'..).complement(),
        RangeSet::new_from_range_char('\u{D7FF}'..='\u{E000}'),
        RangeSet::new_from_range_char('a'..='z').complement(),
    ];

    for range in cases {
        assert_round_trips(&range);
    }
}

/// Every `char` that this crate writes with a backslash, alone and inside a class.
#[test]
fn escaped_chars_round_trip() {
    let escaped = [
        '*', '+', '?', '(', ')', '[', ']', '{', '}', '|', '\\', '-', '^', '.', '$', '\n', '\r',
        '\t', '\u{B}', '\u{0}', '\u{7F}', '~', ' ',
    ];

    for character in escaped {
        assert_round_trips(&RangeSet::new_from_range_char(character..=character));

        // Inside a bracketed class, next to a neighbour that keeps it from being written
        // as a single `char`.
        let with_neighbour = RangeSet::new_from_ranges(&[
            AnyRange::from(Char::new(character)..=Char::new(character)),
            AnyRange::from(Char::new('\u{2000}')..=Char::new('\u{2001}')),
        ]);
        assert_round_trips(&with_neighbour);
    }
}

/// Every ASCII `char` on its own, and every pair of neighbouring ASCII `char` values,
/// which is where escaping and the `-` separator interact.
#[test]
fn every_ascii_char_round_trips() {
    for a in 0..=0x7Fu32 {
        let a = char::from_u32(a).unwrap();
        assert_round_trips(&RangeSet::new_from_range_char(a..=a));

        for b in 0..=0x7Fu32 {
            let b = char::from_u32(b).unwrap();
            if b <= a {
                continue;
            }
            let pair = RangeSet::new_from_ranges(&[
                AnyRange::from(Char::new(a)..=Char::new(a)),
                AnyRange::from(Char::new(b)..=Char::new(b)),
            ]);
            assert_round_trips(&pair);
        }
    }
}

/// Random sets, so that combinations nobody thought to write down get covered too. The
/// seed is fixed, so a failure is reproducible.
#[test]
fn random_sets_round_trip() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x5EED);

    for _ in 0..500 {
        let count = rng.random_range(1..8);
        let ranges: Vec<AnyRange<Char>> = (0..count)
            .map(|_| {
                let min = rng.random_range(0..=char::MAX as u32);
                let max = min + rng.random_range(0..64);
                let min = Char::from_u32(min).unwrap_or(Char::new('\u{E000}'));
                let max = Char::from_u32(max.min(char::MAX as u32))
                    .unwrap_or(Char::new('\u{D7FF}'))
                    .max(min);
                AnyRange::from(min..=max)
            })
            .collect();

        let range = RangeSet::new_from_ranges(&ranges);
        assert_round_trips(&range);
        assert_round_trips(&range.complement());
    }
}
