use criterion::{criterion_group, criterion_main, Criterion};
use regex_charclass::{AnyRange, Char, CharacterClass, RangeSet};

/// A set of `count` one-`char` ranges, which matches no named class and so exercises the
/// slowest path through `to_regex`: a lookup for the set, a lookup for its complement,
/// and then rendering every range.
fn many_ranges(count: u32) -> RangeSet<Char> {
    let ranges: Vec<AnyRange<Char>> = (0..count)
        .map(|i| {
            let c = Char::from_u32(0x4000 + i * 2).unwrap();
            AnyRange::from(c..=c)
        })
        .collect();

    RangeSet::new_from_ranges(&ranges)
}

fn criterion_benchmark(c: &mut Criterion) {
    let hex = RangeSet::new_from_ranges(&[
        AnyRange::from(Char::new('0')..=Char::new('9')),
        AnyRange::from(Char::new('A')..=Char::new('F')),
        AnyRange::from(Char::new('a')..=Char::new('f')),
    ]);
    let hex_complement = hex.complement();

    // `\w`, a named class with many ranges, and its complement.
    let word = RangeSet::new_from_ranges(&[
        AnyRange::from(Char::new('0')..=Char::new('9')),
        AnyRange::from(Char::new('A')..=Char::new('Z')),
        AnyRange::from(Char::new('_')..=Char::new('_')),
        AnyRange::from(Char::new('a')..=Char::new('z')),
    ]);

    let scattered = many_ranges(512);
    let single = RangeSet::new_from_range_char('.'..='.');

    c.bench_function("to_regex_named", |b| b.iter(|| hex.to_regex()));

    c.bench_function("to_regex_negated_named", |b| {
        b.iter(|| hex_complement.to_regex())
    });

    c.bench_function("to_regex_ascii_word", |b| b.iter(|| word.to_regex()));

    c.bench_function("to_regex_ranges", |b| b.iter(|| scattered.to_regex()));

    c.bench_function("to_regex_single_char", |b| b.iter(|| single.to_regex()));

    c.bench_function("get_cardinality", |b| {
        b.iter(|| scattered.get_cardinality())
    });

    c.bench_function("new_from_range_char", |b| {
        b.iter(|| RangeSet::new_from_range_char('a'..='z'))
    });

    c.bench_function("new_from_range_u32", |b| {
        b.iter(|| RangeSet::new_from_range_u32(97..=122))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
