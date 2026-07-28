use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

pub fn criterion_benchmark(c: &mut Criterion) {
    use autobib_entry::{data::*, v0};

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
        data.try_insert(k, v).unwrap();
    }

    // initialize large data
    let mut xl_data = MutableEntryData::default();
    for ch1 in 'a'..='z' {
        for ch2 in '0'..='9' {
            let k = format!("{ch1}{ch2}");
            let v = format!("{ch1}{ch2} value");
            xl_data.try_insert(k, v).unwrap();
        }
    }

    c.bench_function("raw serialize", |b| {
        b.iter(|| black_box(archive(black_box(&data))))
    });

    let raw_bytes = archive(&data);
    let raw_xl_bytes = archive(&xl_data);

    c.bench_function("raw access title", |b| {
        b.iter(|| {
            black_box(
                ArchivedEntryData::access(black_box(&raw_bytes))
                    .unwrap()
                    .get_field("title"),
            )
        })
    });

    c.bench_function("raw access large", |b| {
        b.iter(|| {
            black_box(
                ArchivedEntryData::access(black_box(&raw_xl_bytes))
                    .unwrap()
                    .get_field("u7"),
            )
        })
    });

    c.bench_function("raw access large unchecked", |b| {
        b.iter(|| unsafe {
            black_box(ArchivedEntryData::access_unchecked(black_box(&raw_xl_bytes)).get_field("u7"))
        })
    });

    c.bench_function("raw access all", |b| {
        b.iter(|| {
            for (k, v) in ArchivedEntryData::access(black_box(&raw_bytes))
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
                ArchivedEntryData::access_unchecked(black_box(&raw_bytes)).contains_field("author")
            })
        })
    });

    c.bench_function("raw missing", |b| {
        b.iter(|| {
            black_box(
                ArchivedEntryData::access(black_box(&raw_bytes))
                    .unwrap()
                    .contains_field("missing"),
            )
        })
    });

    c.bench_function("raw missing unchecked", |b| {
        b.iter(|| {
            black_box(unsafe {
                ArchivedEntryData::access_unchecked(black_box(&raw_bytes)).contains_field("missing")
            })
        })
    });

    c.bench_function("raw deserialize", |b| {
        b.iter(|| {
            let raw = ArchivedEntryData::access(black_box(&raw_bytes)).unwrap();
            black_box(MutableEntryData::from_entry_data(raw).get_field("title"));
        })
    });

    c.bench_function("legacy serialize", |b| {
        b.iter(|| black_box(v0::archive(black_box(&data))))
    });

    let legacy_bytes = v0::archive(&data);
    let legacy_xl_bytes = v0::archive(&xl_data);

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
