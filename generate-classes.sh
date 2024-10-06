#!/bin/bash

ucd-generate general-category /tmp/ucd-16.0.0 --chars --exclude surrogate > src/tokens/unicode/general_category.rs
ucd-generate general-category /tmp/ucd-16.0.0 --chars --include decimalnumber > src/tokens/unicode/perl_decimal.rs
ucd-generate property-bool /tmp/ucd-16.0.0 --chars --include whitespace > src/tokens/unicode/perl_space.rs
ucd-generate perl-word /tmp/ucd-16.0.0 --chars > src/tokens/unicode/perl_word.rs
ucd-generate property-bool /tmp/ucd-16.0.0 --chars > src/tokens/unicode/property_bool.rs
ucd-generate script /tmp/ucd-16.0.0 --chars > src/tokens/unicode/script.rs