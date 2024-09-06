use criterion::{criterion_group, criterion_main, Criterion};
use irange::{range::AnyRange, RangeSet};
use regex_charclass::{char::Char, tokens::unicode::general_category::OTHER, CharacterClass};

fn to_range_set(ranges: &[(char, char)]) -> RangeSet<Char> {
    RangeSet::new_from_ranges(
        &ranges
            .into_iter()
            .map(|(min, max)| AnyRange::from(Char::new(*min)..=Char::new(*max)))
            .collect::<Vec<_>>(),
    )
}

fn criterion_benchmark(c: &mut Criterion) {
    {
        let range1 = to_range_set(OTHER);
        let range2 = range1.complement();
        c.bench_function("to_regex", |b| {
            b.iter(|| {
                range1.to_regex();
                range2.to_regex();
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
