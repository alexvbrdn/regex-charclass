use std::collections::HashMap;

use ahash::RandomState;
use irange::RangeSet;
use once_cell::sync::Lazy;
use unicode::get_perl_class;

use crate::{Char, CharacterClass};

pub mod unicode;

type ClassesMap = HashMap<&'static [(char, char)], &'static str, RandomState>;

static CLASSES_MAP: Lazy<ClassesMap> =
    Lazy::new(|| unicode::build_range_map(true, true, true));

pub(crate) fn identify_class(this: &RangeSet<Char>) -> Option<String> {
    if this.get_cardinality() == 1 {
        if let Some(character) = identify_character(this.iter().next()?.to_char()) {
            return Some(character.to_owned());
        }
    }

    let char = unicode::convert_to_range(this);
    if let Some(perl_class) = get_perl_class(&char) {
        return Some(perl_class.to_owned());
    }
    if let Some(class) = CLASSES_MAP.get(char.as_slice()).copied() {
        return Some(format!("\\p{{{}}}", class));
    }

    let this = this.complement();
    let char = unicode::convert_to_range(&this);
    if let Some(perl_class) = get_perl_class(&char) {
        return Some(perl_class.to_uppercase());
    }
    if let Some(class) = CLASSES_MAP.get(char.as_slice()).copied() {
        return Some(format!("\\P{{{}}}", class));
    }

    None
}

pub(crate) fn identify_character(this: char) -> Option<&'static str> {
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
