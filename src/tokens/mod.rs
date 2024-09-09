use irange::RangeSet;
use once_cell::sync::Lazy;
use unicode::{general_category, perl_decimal, perl_space, perl_word, property_bool, script};

use crate::{Char, CharacterClass};

mod unicode;

type ClassesCollection = Vec<(usize, &'static [(char, char)], &'static str)>;

static CLASSES_COLLECTION: Lazy<ClassesCollection> = Lazy::new(|| {
    let mut collection = Vec::with_capacity(
        general_category::BY_NAME.len() + property_bool::BY_NAME.len() + script::BY_NAME.len(),
    );

    for (name, value) in general_category::BY_NAME {
        collection.push((value.len(), *value, *name));
    }

    for (name, value) in property_bool::BY_NAME {
        collection.push((value.len(), *value, *name));
    }

    for (name, value) in script::BY_NAME {
        collection.push((value.len(), *value, *name));
    }

    collection.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    collection
});

pub(super) fn identify_class(this: &RangeSet<Char>) -> Option<String> {
    if this.get_cardinality() == 1 {
        if let Some(character) = identify_character(this.iter().next()?.to_char()) {
            return Some(character.to_owned());
        }
    }

    let char = convert_to_range(this);
    if let Some(perl_class) = get_perl_class(&char) {
        return Some(perl_class.to_owned());
    }
    if let Some(class) = find_class(char.as_slice()) {
        return Some(format!("\\p{{{}}}", class));
    }

    let this = this.complement();
    let char = convert_to_range(&this);
    if let Some(perl_class) = get_perl_class(&char) {
        return Some(perl_class.to_uppercase());
    }
    if let Some(class) = find_class(char.as_slice()) {
        return Some(format!("\\P{{{}}}", class));
    }

    None
}

#[inline]
fn find_class(ranges: &[(char, char)]) -> Option<&'static str> {
    CLASSES_COLLECTION
        .binary_search_by(|(len, ranges_cmp, _)| {
            len.cmp(&ranges.len()).then_with(|| ranges_cmp.cmp(&ranges))
        })
        .ok()
        .map(|index| CLASSES_COLLECTION[index].2)
}

#[inline]
pub(super) fn identify_character(this: char) -> Option<&'static str> {
    if this == '\n' {
        Some("\\n")
    } else if this == '\r' {
        Some("\\r")
    } else if this == '\t' {
        Some("\\t")
    } else if this == '\u{B}' {
        Some("\\v")
    } else {
        None
    }
}

#[inline]
fn convert_to_range(range_set: &RangeSet<Char>) -> Vec<(char, char)> {
    range_set
        .0
        .chunks_exact(2)
        .map(|chunk| (chunk[0].to_char(), chunk[1].to_char()))
        .collect()
}

#[inline]
fn get_perl_class(range: &[(char, char)]) -> Option<&'static str> {
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

#[inline]
fn is_perl_word(range: &[(char, char)]) -> bool {
    perl_word::PERL_WORD == range
}

#[inline]
fn is_perl_space(range: &[(char, char)]) -> bool {
    perl_space::WHITE_SPACE == range
}

#[inline]
fn is_perl_decimal(range: &[(char, char)]) -> bool {
    perl_decimal::DECIMAL_NUMBER == range
}
