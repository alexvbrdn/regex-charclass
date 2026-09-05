# regex-charclass

[![Crates.io Version](https://img.shields.io/crates/v/regex-charclass)](https://crates.io/crates/regex-charclass)
[![docs.rs](https://img.shields.io/docsrs/regex-charclass)](https://docs.rs/regex-charclass)
[![Build status](https://img.shields.io/github/actions/workflow/status/alexvbrdn/regex-charclass/rust.yml?branch=main)](https://github.com/alexvbrdn/regex-charclass/actions/workflows/rust.yml)
[![License](https://img.shields.io/crates/l/regex-charclass)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.63%2B-blue)](#minimum-supported-rust-version)

Manipulate and convert regular expression character classes.

A character class is a set of `char` values. This crate stores one as a sorted collection of non-overlapping inclusive ranges, gives it the usual set algebra, and renders it back as the shortest character class it can find — a named Unicode property where one matches, and a bracketed class otherwise.

The set is an [`irange::RangeSet`](https://github.com/alexvbrdn/irange). `RangeSet`, `AnyRange` and `Char` are re-exported at the crate root, so a dependent needs neither its own dependency on `irange` nor a version kept in step with this crate's. The whole crate is also re-exported as `regex_charclass::irange`.

## Installation

```toml
[dependencies]
regex-charclass = "1.2"
```

The optional `serde` feature implements `Serialize` and `Deserialize` for a set of `char`:

```toml
[dependencies]
regex-charclass = { version = "1.2", features = ["serde"] }
```

## Example

```rust
use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};

let letters = RangeSet::new_from_range_char('a'..='z');
assert_eq!(26, letters.get_cardinality());
assert_eq!("[a-z]", letters.to_regex());

let hex = RangeSet::new_from_ranges(&[
    AnyRange::from(Char::new('0')..=Char::new('9')),
    AnyRange::from(Char::new('A')..=Char::new('F')),
    AnyRange::from(Char::new('a')..=Char::new('f')),
]);
// A set matching a named Unicode property is rendered as that property.
assert_eq!("\\p{ASCII_Hex_Digit}", hex.to_regex());
assert_eq!("\\P{ASCII_Hex_Digit}", hex.complement().to_regex());

assert_eq!(".", hex.union(&hex.complement()).to_regex());
assert_eq!("[]", hex.intersection(&hex.complement()).to_regex());
assert_eq!("[g-z]", letters.difference(&hex).to_regex());
```

## The generated regular expression

`to_regex` returns the shortest form it finds for a set, in this order:

| Form | When |
|---|---|
| `[]` | the set is empty |
| `.` | the set holds every `char` |
| `\n`, `\r`, `\t`, `\v` | the set holds only that one `char` |
| `\d`, `\s`, `\w`, `\D`, `\S`, `\W` | the set is exactly that Perl class |
| `\p{Name}`, `\P{Name}` | the set is exactly a Unicode general category, script or boolean property |
| `a`, `\.` | the set holds one `char`, escaped if it is a metacharacter |
| `[a-z0-9]`, `[^a-z]` | otherwise, complemented when that is shorter |

Three of them need care from whoever reads them back:

- `[]` is a placeholder for the empty set, not a pattern. An empty bracketed class is a parse error in every engine, so a set that may be empty has to be handled before the output reaches one. `[^\s\S]` is the usual pattern for a class matching nothing.
- `.` stands for the total set, which includes `\n`, so the pattern needs dot-matches-newline (`(?s)`).
- `\p{Name}` and the Perl classes stand for Unicode sets, so the pattern needs Unicode mode, which is the default in most engines. The tables here are generated from the Unicode character database a [feature flag](#feature-flags) selects, UCD 17.0.0 by default, so an engine built against another version may resolve a name to a slightly different set.

## Feature flags

| Feature | Effect |
|---|---|
| `serde` | `Serialize` and `Deserialize` for `Char`, and the same feature on `irange`, so that a `RangeSet<Char>` round-trips. |
| `ucd-17` | Generate the named classes from Unicode 17.0.0. |
| `ucd-16` | Generate them from Unicode 16.0.0 instead. |

The `ucd-*` features pick the version of the Unicode character database the named classes come from, which decides both the set of names `to_regex` can return and the `char` each of them covers. Pick the one matching the engine that will read the output:

```toml
[dependencies]
regex-charclass = { version = "1.2", features = ["ucd-16"] }
```

A build that names none of them gets the newest version, so the plain `regex-charclass = "1.2"` above is Unicode 17.0.0. Only the selected version is compiled in, so an unused one costs neither build time nor binary size. The version in force is `regex_charclass::UCD_VERSION`.

Naming a version is what pins it: since none of them is a default feature, `features = ["ucd-16"]` selects Unicode 16 rather than being unified with a default that outranks it, and it stays on 16 when a later release adds a newer version. A build that names none floats to the newest version each release, as this one did from 16.0.0 to 17.0.0.

Cargo features are additive, though, so this only pins the version for a binary. If one dependent asks for `ucd-16` while another asks for `ucd-17`, both features are enabled and the *highest* wins for the whole build. That keeps the outcome the same whatever order the features were enabled in, but it does mean a library cannot pin the UCD version its own dependents get. Where the exact version matters, assert on `UCD_VERSION` rather than assuming the feature you named is the one in force.

## Operations

`n` is the number of ranges in the set. The set operations come from `irange` and are documented there; the table below is what this crate adds.

| Operation | Description | Time | Space |
|---|---|---|---|
| `new_from_range_char` | Build a set from a range of `char`. | `O(1)` | `O(1)` |
| `new_from_range_u32` | Build a set from a range of codepoints, or `None` if a bound is not a `char`. | `O(1)` | `O(1)` |
| `get_cardinality` | Return the number of `char` contained, at most 1,112,064. | `O(n)` | `O(1)` |
| `to_regex` | Return the set as a regular expression character class. | `O(n log m)` | `O(n)` |

`m` is the number of named Unicode classes, which is a constant: `to_regex` binary searches them for a set with the same ranges.

See the [API documentation](https://docs.rs/regex-charclass) for the full list, and [CHANGELOG.md](CHANGELOG.md) for notable changes.

## The surrogate block

`char` is not contiguous: the codepoints `U+D800..=U+DFFF` are surrogates and hold no `char`. `Char` counts by index rather than by codepoint and skips that block, so `'\u{D7FF}'` and `'\u{E000}'` are adjacent and the total set is one range of 1,112,064 values rather than two.

## Minimum supported Rust version

`regex-charclass` builds with Rust 1.63 and later. Raising this version is a breaking change.

Enabling the `serde` feature pulls in `serde_derive`, whose own dependencies require a more recent compiler.

## Regenerating the Unicode tables

`src/tokens/unicode/v16/` and `v17/` are generated by [`ucd-generate`](https://github.com/BurntSushi/ucd-generate), one directory per UCD version; `generate-classes.sh 17.0.0` regenerates one of them and its header has the UCD download it needs. The generated files are exempt from `rustfmt`, so a regeneration is a readable diff. Adding a version also means declaring its feature in `Cargo.toml` and adding a block to `src/tokens/unicode/mod.rs`.

## Contributing

Bug reports and pull requests are welcome, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the [MIT License](LICENSE).
