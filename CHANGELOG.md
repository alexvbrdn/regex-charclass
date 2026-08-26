# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-08-26

### Added

- Unicode tables are now generated from UCD 17.0.0 (previously UCD 16.0.0), so
  the scripts, general categories and boolean properties added in Unicode 17 are
  recognised by `identify_class`.

### Changed

- The named-class lookup table is built and sorted at compile time with a `const`
  function instead of being assembled lazily on first use. Lookups no longer pay
  the one-off initialisation cost or the atomic check on every access.
- `\s` and `\d` detection reuses the `WHITE_SPACE` and `DECIMAL_NUMBER` tables
  from `property_bool` / `general_category` rather than duplicating them in
  dedicated generated modules, shrinking the generated data.

### Removed

- Dropped the `once_cell` dependency; the crate now has `irange` as its only
  required dependency.
- Removed the generated `perl_decimal` and `perl_space` modules, along with the
  `ucd-generate` invocations that produced them in `generate-classes.sh`.

All of the above is internal: the public API is unchanged.

[1.1.0]: https://github.com/alexvbrdn/regex-charclass/compare/v1.0.3...v1.1.0
