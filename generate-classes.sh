#!/bin/bash
#
# Regenerate the Unicode tables under src/tokens/unicode/.
#
# It needs `ucd-generate` (`cargo install ucd-generate`) and a copy of the Unicode
# character database, which is not vendored:
#
#     mkdir -p /tmp/ucd-17.0.0
#     curl -LO https://www.unicode.org/Public/zipped/17.0.0/UCD.zip
#     unzip UCD.zip -d /tmp/ucd-17.0.0
#
# The generated files are exempt from rustfmt, so their formatting is whatever
# `ucd-generate` writes and a regeneration is a readable diff. Update the UCD version
# named in the crate documentation and in CHANGELOG.md when bumping it here.

set -euo pipefail

ucd="${1:-/tmp/ucd-17.0.0}"
out="$(dirname "$0")/src/tokens/unicode"

ucd-generate general-category "$ucd" --chars --exclude surrogate > "$out/general_category.rs"
ucd-generate perl-word "$ucd" --chars > "$out/perl_word.rs"
ucd-generate property-bool "$ucd" --chars > "$out/property_bool.rs"
ucd-generate script "$ucd" --chars > "$out/script.rs"
