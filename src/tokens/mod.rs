//! Recognising a set of `char` as a named regular expression class.
//!
//! A set that is exactly a Unicode general category, script or boolean property, or one
//! of the Perl classes, is written as that name rather than as a list of ranges. The
//! tables come from `ucd-generate`; see [`unicode`].

use irange::RangeSet;
use unicode::{general_category, perl_word, property_bool, script};

use crate::{Char, CharacterClass};

mod unicode;

type ClassEntry = (usize, &'static [(char, char)], &'static str);
type NamedClasses = &'static [(&'static str, &'static [(char, char)])];

const CLASSES_COLLECTION_LEN: usize =
    general_category::BY_NAME.len() + property_bool::BY_NAME.len() + script::BY_NAME.len();

/// Every named class, sorted by range count then by ranges so that
/// [`find_class`] can binary search it. Built at compile time.
static CLASSES_COLLECTION: [ClassEntry; CLASSES_COLLECTION_LEN] = build_classes_collection();

const fn build_classes_collection() -> [ClassEntry; CLASSES_COLLECTION_LEN] {
    let mut collection: [ClassEntry; CLASSES_COLLECTION_LEN] =
        [(0, &[], ""); CLASSES_COLLECTION_LEN];
    let mut index = 0;

    let tables: [NamedClasses; 3] = [
        general_category::BY_NAME,
        property_bool::BY_NAME,
        script::BY_NAME,
    ];

    let mut table = 0;
    while table < tables.len() {
        let mut i = 0;
        while i < tables[table].len() {
            let (name, value) = tables[table][i];
            collection[index] = (value.len(), value, name);
            index += 1;
            i += 1;
        }
        table += 1;
    }

    // Insertion sort: `sort_unstable_by` is not available in a const context.
    let mut i = 1;
    while i < CLASSES_COLLECTION_LEN {
        let mut j = i;
        while j > 0 && is_before(collection[j], collection[j - 1]) {
            let swap = collection[j - 1];
            collection[j - 1] = collection[j];
            collection[j] = swap;
            j -= 1;
        }
        i += 1;
    }

    collection
}

/// `a < b` for the ordering [`find_class`] binary searches with.
const fn is_before(a: ClassEntry, b: ClassEntry) -> bool {
    if a.0 != b.0 {
        return a.0 < b.0;
    }

    let mut i = 0;
    while i < a.1.len() && i < b.1.len() {
        let (a_start, a_end) = a.1[i];
        let (b_start, b_end) = b.1[i];
        if a_start as u32 != b_start as u32 {
            return (a_start as u32) < (b_start as u32);
        }
        if a_end as u32 != b_end as u32 {
            return (a_end as u32) < (b_end as u32);
        }
        i += 1;
    }

    a.1.len() < b.1.len()
}

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
    property_bool::WHITE_SPACE == range
}

#[inline]
fn is_perl_decimal(range: &[(char, char)]) -> bool {
    general_category::DECIMAL_NUMBER == range
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The collection is sorted at compile time, so nothing would catch a
    /// broken ordering at runtime other than lookups silently missing.
    #[test]
    fn classes_collection_is_sorted_for_binary_search() {
        for pair in CLASSES_COLLECTION.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            assert!(
                (a.0, a.1) <= (b.0, b.1),
                "{} and {} are out of order",
                a.2,
                b.2
            );
        }
    }

    #[test]
    fn every_class_is_findable() {
        for (name, value) in general_category::BY_NAME
            .iter()
            .chain(property_bool::BY_NAME)
            .chain(script::BY_NAME)
        {
            let found = find_class(value)
                .unwrap_or_else(|| panic!("{name} is missing from the collection"));
            let found_value = CLASSES_COLLECTION
                .iter()
                .find(|(_, _, class)| class == &found)
                .unwrap()
                .1;
            assert_eq!(*value, found_value, "{name} resolved to {found}");
        }
    }
}
