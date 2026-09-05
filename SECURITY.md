# Security Policy

## Supported versions

Fixes are published for the latest release of `regex-charclass`.

## Reporting a vulnerability

Please report vulnerabilities privately through GitHub's [security advisory form](https://github.com/alexvbrdn/regex-charclass/security/advisories/new) rather than in a public issue.

## Scope

`regex-charclass` contains no `unsafe` code, enforced by `#![forbid(unsafe_code)]`, and performs no I/O, so the realistic concerns are:

- A panic reachable from a set built from untrusted input. Deserializing with the `serde` feature validates its input and rejects a malformed set, and no method may panic even on a set written directly through `RangeSet`'s public field. A counterexample is a bug worth reporting.
- A class from `to_regex` that a regular expression engine reads as a different set of `char` than the one it was generated from, which could turn a set meant to exclude something into one that admits it. The escaping is checked against the `regex` crate in `tests/regex_roundtrip.rs`. Note the two documented exceptions there: `[]` is a placeholder rather than a pattern, and a `\p{Name}` class resolves against the reading engine's own Unicode version rather than the tables here, whose version the `ucd-*` features select.
