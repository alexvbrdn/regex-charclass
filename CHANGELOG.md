# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-09-05

This release moves to UCD 17.0.0, fixes four bugs that made a set hold the wrong `char` values or panic, and makes the named-class lookup free at runtime. No API is removed and no code that compiled against 1.0.3 stops compiling, but results change where they used to be incorrect. See *Compatibility* below before upgrading.

### Fixed

- An unbounded upper bound no longer gives the empty set. `new_from_range_char('a'..)` and `new_from_range_u32(97..)` read the missing bound as the *smallest* `char` rather than the largest, so every such range came out inverted and empty. They now reach `char::MAX`.
- An empty range no longer panics or produces a huge one. `new_from_range_char('\0'..'\0')` and `new_from_range_u32(0..0)` computed the `char` before `'\0'`, which panicked with `attempt to subtract with overflow` in debug builds and wrapped around to a range of the whole Basic Multilingual Plane in release builds. They now return the empty set, as do an excluded lower bound of `char::MAX` and an inverted range.
- `to_regex` escapes `$`. A set holding only `$` was written as a bare `$`, which a regular expression engine reads as an end-of-input anchor rather than as the `char`, so the class matched nothing.
- `to_regex` writes `~` as itself. The printable-ASCII check excluded its own upper bound, so `~` came out as `\u{007e}`. The class was correct, just longer than it needed to be. The same applied to `Display for Char`.
- `Add` and `Sub` on `Char` agree with each other. `Sub` worked on the index of a `char` among all `char` values, skipping the surrogate block, while `Add` worked on raw codepoints, so `(a + b) - b` could land on a different `char` for values on either side of the block. Both now work on indices, which is what `RangeSet` needs. Stepping by `Char::one()`, which is all `RangeSet` does, was already correct and is unchanged. Both also panic consistently on overflow instead of only in debug builds.
- No method panics on a malformed set. `RangeSet` keeps its bounds in a public field, so a set with an odd number of bounds could reach `get_cardinality` and `to_regex` and panic with an index out of bounds.

### Added

- Crate-level documentation, so `docs.rs` has a landing page: it covers the forms `to_regex` produces and the flags they assume, the surrogate block, and the `serde` feature. `missing_docs` is now warned on and every public item is documented.
- `Char::COUNT`, the number of `char` values and so the largest cardinality a set can have.
- `RangeSet`, `AnyRange` and `Char` are re-exported at the crate root, so the usual import is `use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};` rather than a path through `irange::` and `char::`. Both are `irange`'s types and appear in this crate's public API, so `regex_charclass::irange` stays as it was and the existing paths keep working.
- A declared minimum supported Rust version, 1.63, in the `rust-version` field and in the README. It was never declared before, so this documents the requirement rather than raising it.
- Integration tests under `tests/`, which drive the crate through its public API the way a dependent crate does. `tests/regex_roundtrip.rs` compiles what `to_regex` produces with the `regex` crate and checks that the engine reads back exactly the set it was generated from, over every ASCII `char`, every pair of them, every escaped `char` and a few hundred random sets.
- The README is compiled and run as a doctest, through a `#[cfg(doctest)]` item that includes it, so its example cannot drift from the API.
- `CONTRIBUTING.md`, `SECURITY.md`, a pull request template, and Dependabot updates.

### Changed

- Unicode tables are now generated from UCD 17.0.0, previously UCD 16.0.0, so the scripts, general categories and boolean properties added in Unicode 17 are recognised.
- The named-class lookup table is built and sorted at compile time with a `const` function instead of being assembled lazily on first use. Lookups no longer pay the one-off initialisation cost or the atomic check on every access.
- `\s` and `\d` detection reuses the `WHITE_SPACE` and `DECIMAL_NUMBER` tables from `property_bool` / `general_category` rather than duplicating them in dedicated generated modules, shrinking the generated data.
- The crate is now `#![forbid(unsafe_code)]`. It never contained any `unsafe`; this makes that checkable.
- The generated Unicode tables are exempt from `rustfmt` as well as from `clippy`, so regenerating them is a readable diff rather than a reformatting of the whole table. `generate-classes.sh` documents the UCD download it needs and takes the directory as an argument.
- `Cargo.toml` declares `categories`, `homepage`, `documentation` and `rust-version`, and switches to an `include` allowlist so a new file at the root cannot end up in the published package by accident. The `keywords` are now `regex`, `unicode`, `character-class`, `set` and `intersection`.
- CI now checks formatting, runs clippy with `-D warnings`, tests each feature combination, builds the documentation with `-D warnings`, dry-runs the publish, and verifies the declared 1.63 MSRV. Previously it built, tested and ran clippy without failing on warnings.
- The benchmark file is renamed from `benches/my_benchmark.rs` to `benches/char_class.rs`, and covers a named-class hit, a miss, a single `char` and the constructors.

### Removed

- Dropped the `once_cell` dependency; the crate now has `irange` as its only required dependency.
- Removed the generated `perl_decimal` and `perl_space` modules, along with the `ucd-generate` invocations that produced them in `generate-classes.sh`.

### Compatibility

- The minimum supported Rust version is now declared as 1.63. No new compiler is required; the version is documented rather than raised.
- `irange` is now required at 1.2, which fixed bugs of its own in the set operations this crate builds on. A `RangeSet<Char>` serialized by an older version fails to deserialize if it holds adjacent ranges that should have been merged; rebuild it with `new_from_ranges`.
- Code that relied on an unbounded upper bound giving the empty set, or that caught the panic on an empty range, sees different (correct) values.
- A class that holds `$` or `~` is written differently. Both forms describe the same set to an engine, except that the old rendering of `$` did not.

## [1.0.3] - 2024-10-06

### Changed

- Unicode tables generated from UCD 16.0.0.

## [1.0.2] - 2024-09-09

### Changed

- Improved performance.

## [1.0.1] - 2024-09-09

### Changed

- `serde` support is no longer a default feature.

## [1.0.0] - 2024-09-06

Initial release.

[1.1.0]: https://github.com/alexvbrdn/regex-charclass/compare/v1.0.3...v1.1.0
[1.0.3]: https://github.com/alexvbrdn/regex-charclass/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/alexvbrdn/regex-charclass/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/alexvbrdn/regex-charclass/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/alexvbrdn/regex-charclass/releases/tag/v1.0.0
