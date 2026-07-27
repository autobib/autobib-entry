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

    // initialize large data
    let mut xl_data = MutableEntryData::default();
    for ch1 in 'a'..='z' {
        for ch2 in '0'..='9' {
            let k = format!("{ch1}{ch2}");
            let v = format!("{ch1}{ch2} value");
            xl_data.check_and_insert(k, v).unwrap();
        }
    }

    c.bench_function("rkyv serialize", |b| {
        b.iter(|| black_box(rkyv::to_bytes::<Failure>(black_box(&data)).unwrap()))
    });

    let rkyv_bytes = rkyv::to_bytes::<Failure>(&data).unwrap();
    let rkyv_xl_bytes = rkyv::to_bytes::<Failure>(&xl_data).unwrap();

    c.bench_function("rkyv access title", |b| {
        b.iter(|| {
            black_box(
                rkyv::access::<ArchivedEntryData, Failure>(black_box(&rkyv_bytes))
                    .unwrap()
                    .get_field("title"),
            )
        })
    });

    c.bench_function("rkyv access large", |b| {
        b.iter(|| {
            black_box(
                rkyv::access::<ArchivedEntryData, Failure>(black_box(&rkyv_xl_bytes))
                    .unwrap()
                    .get_field("u7"),
            )
        })
    });

    c.bench_function("rkyv access large unchecked", |b| {
        b.iter(|| unsafe {
            black_box(
                rkyv::access_unchecked::<ArchivedEntryData>(black_box(&rkyv_xl_bytes))
                    .get_field("u7"),
            )
        })
    });

    c.bench_function("rkyv access all", |b| {
        b.iter(|| {
            for (k, v) in rkyv::access::<ArchivedEntryData, Failure>(black_box(&rkyv_bytes))
                .unwrap()
                .fields()
            {
                black_box((k, v));
            }
        })
    });

    c.bench_function("rkyv access unchecked", |b| {
        b.iter(|| {
            black_box(unsafe {
                rkyv::access_unchecked::<ArchivedEntryData>(black_box(&rkyv_bytes))
                    .get_field("author")
            })
        })
    });

    c.bench_function("rkyv missing", |b| {
        b.iter(|| {
            black_box(
                rkyv::access::<ArchivedEntryData, Failure>(black_box(&rkyv_bytes))
                    .unwrap()
                    .contains_field("missing"),
            )
        })
    });

    c.bench_function("rkyv missing unchecked", |b| {
        b.iter(|| {
            black_box(unsafe {
                rkyv::access_unchecked::<ArchivedEntryData>(black_box(&rkyv_bytes))
                    .contains_field("missing")
            })
        })
    });

    c.bench_function("rkyv deserialize", |b| {
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

    c.bench_function("raw serialize", |b| {
        b.iter(|| black_box(serialize(black_box(&data))))
    });

    let raw_bytes = serialize(&data);
    let raw_xl_bytes = serialize(&xl_data);

    c.bench_function("raw access title", |b| {
        b.iter(|| {
            black_box(
                RawEntryData::access(black_box(&raw_bytes))
                    .unwrap()
                    .get_field("title"),
            )
        })
    });

    c.bench_function("raw access large", |b| {
        b.iter(|| {
            black_box(
                RawEntryData::access(black_box(&raw_xl_bytes))
                    .unwrap()
                    .get_field("u7"),
            )
        })
    });

    c.bench_function("raw access large unchecked", |b| {
        b.iter(|| unsafe {
            black_box(RawEntryData::access_unchecked(black_box(&raw_xl_bytes)).get_field("u7"))
        })
    });

    c.bench_function("raw access all", |b| {
        b.iter(|| {
            for (k, v) in RawEntryData::access(black_box(&raw_bytes))
                .unwrap()
                .fields()
            {
                black_box((k, v));
            }
        })
    });

    c.bench_function("raw access unchecked", |b| {
        b.iter(|| {
            black_box(unsafe {
                RawEntryData::access_unchecked(black_box(&raw_bytes)).contains_field("author")
            })
        })
    });

    c.bench_function("raw missing", |b| {
        b.iter(|| {
            black_box(
                RawEntryData::access(black_box(&raw_bytes))
                    .unwrap()
                    .contains_field("missing"),
            )
        })
    });

    c.bench_function("raw missing unchecked", |b| {
        b.iter(|| {
            black_box(unsafe {
                RawEntryData::access_unchecked(black_box(&raw_bytes)).contains_field("missing")
            })
        })
    });

    c.bench_function("raw deserialize", |b| {
        b.iter(|| {
            let raw = RawEntryData::access(black_box(&raw_bytes)).unwrap();
            black_box(MutableEntryData::from_entry_data(raw).get_field("title"));
        })
    });

    c.bench_function("legacy serialize", |b| {
        b.iter(|| black_box(v0::serialize(black_box(&data))))
    });

    let legacy_bytes = v0::serialize(&data);
    let legacy_xl_bytes = v0::serialize(&xl_data);

    c.bench_function("legacy access title", |b| {
        b.iter(|| {
            black_box(
                v0::LegacyEntryData::access(black_box(&legacy_bytes))
                    .unwrap()
                    .get_field("title"),
            )
        })
    });

    c.bench_function("legacy access large", |b| {
        b.iter(|| {
            black_box(
                v0::LegacyEntryData::access(black_box(&legacy_xl_bytes))
                    .unwrap()
                    .get_field("u7"),
            )
        })
    });

    c.bench_function("legacy access large unchecked", |b| {
        b.iter(|| unsafe {
            black_box(
                v0::LegacyEntryData::access_unchecked(black_box(&legacy_xl_bytes)).get_field("u7"),
            )
        })
    });

    c.bench_function("legacy access all", |b| {
        b.iter(|| {
            for (k, v) in v0::LegacyEntryData::access(black_box(&legacy_bytes))
                .unwrap()
                .fields()
            {
                black_box((k, v));
            }
        })
    });

    c.bench_function("legacy access unchecked", |b| {
        b.iter(|| {
            black_box(unsafe {
                v0::LegacyEntryData::access_unchecked(black_box(&legacy_bytes))
                    .contains_field("author")
            })
        })
    });

    c.bench_function("legacy missing", |b| {
        b.iter(|| {
            black_box(
                v0::LegacyEntryData::access(black_box(&legacy_bytes))
                    .unwrap()
                    .contains_field("missing"),
            )
        })
    });

    c.bench_function("legacy missing unchecked", |b| {
        b.iter(|| {
            black_box(unsafe {
                v0::LegacyEntryData::access_unchecked(black_box(&legacy_bytes))
                    .contains_field("missing")
            })
        })
    });

    c.bench_function("legacy deserialize", |b| {
        b.iter(|| {
            let legacy = v0::LegacyEntryData::access(black_box(&legacy_bytes)).unwrap();
            black_box(MutableEntryData::from_entry_data(legacy).get_field("title"));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
