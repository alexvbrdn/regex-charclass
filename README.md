# regex-charclass

[![Crates.io Version](https://img.shields.io/crates/v/regex-charclass)](https://crates.io/crates/regex-charclass)

A data structure to store and manipulate ranges of characters with set operations.

This library is based on [`irange`](https://github.com/alexvbrdn/irange).

## Installation

Add the following line in your `Cargo.toml`:

```toml
[dependencies]
regex-charclass = "1.0"
```

If you need `serde` support you can include the following feature flag:

```toml
[dependencies]
regex-charclass = { version = "1.0", features = ["serde"] }
```

## Examples

```rust
use regex_charclass::{irange::{RangeSet, range::AnyRange}, char::Char, CharacterClass};

let range1 = RangeSet::new_from_range_char('a'..='z');
assert_eq!(26, range1.get_cardinality());
assert_eq!("[a-z]", range1.to_regex());

let range2 = RangeSet::new_from_ranges(&[
    AnyRange::from(Char::new('0')..=Char::new('9')),
    AnyRange::from(Char::new('A')..=Char::new('F')),
    AnyRange::from(Char::new('a')..=Char::new('f')),
]);
assert_eq!("\\p{ASCII_Hex_Digit}", range2.to_regex());

let range2_complement = range2.complement();
assert_eq!("\\P{ASCII_Hex_Digit}", range2_complement.to_regex());

assert_eq!(".", range2.union(&range2_complement).to_regex());
assert_eq!("[]", range2.intersection(&range2_complement).to_regex());

assert_eq!("[g-z]", range1.difference(&range2).to_regex());
```
