use criterion::{criterion_group, criterion_main, Criterion};
use irange::{range::AnyRange, RangeSet};
use regex_charclass::{char::Char, CharacterClass};

fn criterion_benchmark(c: &mut Criterion) {
    let range1 = RangeSet::new_from_ranges(&[
        AnyRange::from(Char::new('a')..=Char::new('f')),
        AnyRange::from(Char::new('A')..=Char::new('F')),
        AnyRange::from(Char::new('0')..=Char::new('9')),
    ]);
    {
        let range2 = range1.complement();
        c.bench_function("to_regex_hit", |b| {
            b.iter(|| {
                range1.to_regex();
                range2.to_regex();
            })
        });
    }

    {
        let range3 = RangeSet::new_from_ranges(&[
            AnyRange::from(Char::new('a')..=Char::new('z')),
            AnyRange::from(Char::new('0')..=Char::new('9')),
        ]);
        let range4 = range3.complement();
        c.bench_function("to_regex_miss", |b| {
            b.iter(|| {
                range3.to_regex();
                range4.to_regex();
            })
        });
    }

    {
        c.bench_function("get_cardinality", |b| {
            b.iter(|| {
                range1.get_cardinality();
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
