use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use rkyv::rancor::Failure;

pub fn criterion_benchmark(c: &mut Criterion) {
    use autobib_entry::*;

    // initialize data
    let mut data = MutableEntryData::default();
    let fields = [
        ("author", "Alex Rutar"),
        ("journal", "Journal of Great Papers"),
        ("month", "Dec"),
        ("title", "A wonderful title"),
        ("year", "2036"),
    ];

    for (k, v) in fields {
        data.check_and_insert(k, v).unwrap();
    }

    c.bench_function("serialize rkyv", |b| {
        b.iter(|| black_box(rkyv::to_bytes::<Failure>(black_box(&data)).unwrap()))
    });

    let rkyv_bytes = rkyv::to_bytes::<Failure>(&data).unwrap();

    c.bench_function("access rkyv", |b| {
        b.iter(|| {
            black_box(
                rkyv::access::<ArchivedEntryData, Failure>(black_box(&rkyv_bytes))
                    .unwrap()
                    .get_field("title"),
            )
        })
    });

    c.bench_function("deserialize rkyv", |b| {
        b.iter(|| {
            let archived =
                rkyv::access::<ArchivedEntryData, Failure>(black_box(&rkyv_bytes[..])).unwrap();
            black_box(
                rkyv::deserialize::<MutableEntryData, Failure>(archived)
                    .unwrap()
                    .get_field("title"),
            );
        })
    });

    c.bench_function("serialize raw", |b| {
        b.iter(|| black_box(serialize(black_box(&data))))
    });

    let raw_bytes = serialize(&data);

    c.bench_function("access raw", |b| {
        b.iter(|| {
            black_box(
                RawEntryData::access(black_box(&raw_bytes))
                    .unwrap()
                    .get_field("title"),
            )
        })
    });

    c.bench_function("deserialize raw", |b| {
        b.iter(|| {
            let raw = RawEntryData::access(black_box(&raw_bytes)).unwrap();
            black_box(MutableEntryData::from_entry_data(raw).get_field("title"));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
