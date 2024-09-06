use std::collections::HashMap;

use ahash::RandomState;
use irange::RangeSet;

use crate::char::Char;

#[allow(clippy::all)]
pub mod general_category;

#[allow(clippy::all)]
pub mod property_bool;

#[allow(clippy::all)]
pub mod script;

#[allow(clippy::all)]
pub mod perl_decimal;

#[allow(clippy::all)]
pub mod perl_space;

#[allow(clippy::all)]
pub mod perl_word;

pub fn build_range_map(
    general_category: bool,
    property_bool: bool,
    script: bool,
) -> HashMap<&'static [(char, char)], &'static str, RandomState> {
    let mut hash = HashMap::with_capacity_and_hasher(120, RandomState::new());
    if general_category {
        for (name, value) in general_category::BY_NAME {
            hash.insert(*value, *name);
        }
    }
    if property_bool {
        for (name, value) in property_bool::BY_NAME {
            hash.insert(*value, *name);
        }
    }
    if script {
        for (name, value) in script::BY_NAME {
            hash.insert(*value, *name);
        }
    }
    hash
}

pub fn get_perl_class(range: &[(char, char)]) -> Option<&'static str> {
    if is_perl_decimal(range) {
        Some("\\d")
    } else if is_perl_space(range) {
        Some("\\s")
    } else if is_perl_word(range) {
        Some("\\w")
    } else {
        None
    }
}

fn is_perl_word(range: &[(char, char)]) -> bool {
    perl_word::PERL_WORD == range
}

fn is_perl_space(range: &[(char, char)]) -> bool {
    perl_space::WHITE_SPACE == range
}

fn is_perl_decimal(range: &[(char, char)]) -> bool {
    perl_decimal::DECIMAL_NUMBER == range
}

pub fn convert_to_range(range_set: &RangeSet<Char>) -> Vec<(char, char)> {
    range_set
        .0
        .chunks_exact(2)
        .map(|chunk| (chunk[0].to_char(), chunk[1].to_char()))
        .collect()
}
