//! Tests driving `regex-charclass` through its public API, the way a dependent crate
//! sees it.

use std::ops::Bound;

use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};

fn set(ranges: &[(char, char)]) -> RangeSet<Char> {
    RangeSet::new_from_ranges(
        &ranges
            .iter()
            .map(|&(min, max)| AnyRange::new(Char::new(min), Char::new(max)))
            .collect::<Vec<_>>(),
    )
}

#[test]
fn included_bounds() {
    assert_eq!(set(&[('a', 'z')]), RangeSet::new_from_range_char('a'..='z'));
    assert_eq!(
        Some(set(&[('a', 'z')])),
        RangeSet::new_from_range_u32(97..=122)
    );
}

#[test]
fn excluded_bounds_step_over_one_char() {
    assert_eq!(set(&[('a', 'e')]), RangeSet::new_from_range_char('a'..'f'));
    assert_eq!(
        Some(set(&[('a', 'e')])),
        RangeSet::new_from_range_u32(97..102)
    );

    let excluded_lower = (Bound::Excluded('a'), Bound::Included('z'));
    assert_eq!(
        set(&[('b', 'z')]),
        RangeSet::new_from_range_char(excluded_lower)
    );
}

/// An unbounded upper bound used to be read as the *smallest* `char`, so every such
/// range came out inverted and gave the empty set.
#[test]
fn unbounded_bounds_reach_the_ends_of_char() {
    assert_eq!(
        set(&[('a', char::MAX)]),
        RangeSet::new_from_range_char('a'..)
    );
    assert_eq!(
        Some(set(&[('a', char::MAX)])),
        RangeSet::new_from_range_u32(97..)
    );

    assert_eq!(
        set(&[('\0', 'z')]),
        RangeSet::new_from_range_char(..='z'),
        "an unbounded lower bound starts at the first char"
    );

    assert!(RangeSet::<Char>::new_from_range_char(..).is_total());
    assert!(RangeSet::<Char>::new_from_range_u32(..).unwrap().is_total());
}

/// `'\0'..'\0'` and `char::MAX..` with the bound excluded used to underflow, panicking
/// in debug builds and giving a huge range in release builds.
#[test]
fn empty_ranges_give_the_empty_set() {
    assert!(RangeSet::<Char>::new_from_range_char('a'..'a').is_empty());
    assert!(RangeSet::<Char>::new_from_range_char('\0'..'\0').is_empty());
    assert!(RangeSet::<Char>::new_from_range_char('z'..='a').is_empty());
    assert!(
        RangeSet::<Char>::new_from_range_char((Bound::Excluded(char::MAX), Bound::Unbounded))
            .is_empty()
    );

    assert!(RangeSet::<Char>::new_from_range_u32(0..0)
        .unwrap()
        .is_empty());
    assert!(RangeSet::<Char>::new_from_range_u32(97..97)
        .unwrap()
        .is_empty());
    assert!(RangeSet::<Char>::new_from_range_u32((
        Bound::Excluded(char::MAX as u32),
        Bound::Unbounded
    ))
    .unwrap()
    .is_empty());
}

#[test]
fn codepoints_that_are_not_chars_are_rejected() {
    for range in [0xD800..=0xD800, 0xD7FF..=0xD800, 0x110000..=0x110000] {
        assert_eq!(
            None,
            RangeSet::<Char>::new_from_range_u32(range.clone()),
            "{range:?} holds a codepoint that is not a char"
        );
    }

    // An excluded bound has to be a `char` too, even though it is not in the set.
    assert_eq!(
        None,
        RangeSet::<Char>::new_from_range_u32((Bound::Excluded(0xD800), Bound::Unbounded))
    );
}

/// The surrogate block holds no `char`, so a range across it is one range, not two.
#[test]
fn ranges_join_across_the_surrogate_block() {
    let across = RangeSet::new_from_range_char('\u{D7FF}'..='\u{E000}');
    assert_eq!(2, across.get_cardinality());
    assert_eq!(2, across.iter().count());

    let below = RangeSet::new_from_range_char('\0'..='\u{D7FF}');
    let above = RangeSet::new_from_range_char('\u{E000}'..);
    assert!(below.union(&above).is_total());
}

#[test]
fn cardinality_counts_chars_not_codepoints() {
    assert_eq!(0, RangeSet::<Char>::empty().get_cardinality());
    assert_eq!(
        1,
        RangeSet::new_from_range_char('a'..='a').get_cardinality()
    );
    assert_eq!(
        26,
        RangeSet::new_from_range_char('a'..='z').get_cardinality()
    );
    assert_eq!(Char::COUNT, RangeSet::<Char>::total().get_cardinality());
    assert_eq!(1_112_064, Char::COUNT);
}

#[test]
fn to_regex_renders_the_shortest_form() {
    assert_eq!("[]", RangeSet::<Char>::empty().to_regex());
    assert_eq!(".", RangeSet::<Char>::total().to_regex());
    assert_eq!("a", RangeSet::new_from_range_char('a'..='a').to_regex());
    assert_eq!("[ab]", RangeSet::new_from_range_char('a'..='b').to_regex());
    assert_eq!("[a-c]", RangeSet::new_from_range_char('a'..='c').to_regex());
    assert_eq!("[0-9a-z]", set(&[('a', 'z'), ('0', '9')]).to_regex());

    // A set with few gaps is written as the complement of those gaps.
    assert_eq!(
        "[^a]",
        RangeSet::new_from_range_char('a'..='a')
            .complement()
            .to_regex()
    );
}

#[test]
fn to_regex_names_the_classes_it_recognises() {
    let hex = set(&[('0', '9'), ('A', 'F'), ('a', 'f')]);
    assert_eq!("\\p{ASCII_Hex_Digit}", hex.to_regex());
    assert_eq!("\\P{ASCII_Hex_Digit}", hex.complement().to_regex());

    let newline = RangeSet::new_from_range_char('\n'..='\n');
    assert_eq!("\\n", newline.to_regex());
    assert_eq!("\\t", RangeSet::new_from_range_char('\t'..='\t').to_regex());
}

/// `$` used to be written bare, which a regular expression engine reads as an anchor.
#[test]
fn metacharacters_are_escaped() {
    for character in [
        '*', '+', '?', '(', ')', '[', ']', '{', '}', '|', '\\', '-', '^', '.', '$',
    ] {
        assert_eq!(
            format!("\\{character}"),
            RangeSet::new_from_range_char(character..=character).to_regex()
        );
    }
}

#[test]
fn set_operations_agree_with_the_rendered_class() {
    let letters = RangeSet::new_from_range_char('a'..='z');
    let hex = set(&[('0', '9'), ('A', 'F'), ('a', 'f')]);

    assert_eq!("[g-z]", letters.difference(&hex).to_regex());
    assert_eq!("[a-f]", letters.intersection(&hex).to_regex());
    assert_eq!("[0-9A-Fa-z]", letters.union(&hex).to_regex());
    assert_eq!("[]", hex.intersection(&hex.complement()).to_regex());
    assert_eq!(".", hex.union(&hex.complement()).to_regex());
}

/// The bounds are a public field, so a malformed set can reach any method. None of them
/// may panic, whatever they return.
#[test]
fn no_method_panics_on_a_malformed_set() {
    let malformed = [
        vec![Char::new('a')],
        vec![Char::new('z'), Char::new('a')],
        vec![
            Char::new('x'),
            Char::new('z'),
            Char::new('a'),
            Char::new('c'),
        ],
        vec![
            Char::new('a'),
            Char::new('b'),
            Char::new('c'),
            Char::new('d'),
        ],
        vec![Char::new('a'), Char::new('c'), Char::new('b')],
    ];

    for bounds in malformed {
        let range = RangeSet(bounds);
        let _ = range.get_cardinality();
        let _ = range.to_regex();
    }
}
