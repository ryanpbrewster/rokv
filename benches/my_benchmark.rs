use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rokv::{Reader, Writer};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fill_1000", |b| {
        b.iter(|| {
            let mut file = tempfile::tempfile().unwrap();
            let mut w = Writer::new(&mut file).unwrap();
            for i in 0..1_1000 {
                let key = format!("key-{}", i);
                let value = format!("value-{}", i);
                w.append(key.as_bytes(), value.as_bytes()).unwrap();
            }
        })
    });

    c.bench_function("read_1000_of_1000", |b| {
        let mut file = tempfile::tempfile().unwrap();
        {
            let mut w = Writer::new(&mut file).unwrap();
            for i in 0..1_000 {
                let key = format!("key-{}", i);
                let value = format!("value-{}", i);
                w.append(key.as_bytes(), value.as_bytes()).unwrap();
            }
            w.finish().unwrap();
        }
        b.iter(|| {
            let mut r = Reader::new(&mut file).unwrap();
            for i in 0..1_000 {
                let key = format!("key-{}", i);
                black_box(r.read(key.as_bytes()).unwrap());
            }
        });
    });

    c.bench_function("read_1000_nonexistent", |b| {
        let mut file = tempfile::tempfile().unwrap();
        {
            let mut w = Writer::new(&mut file).unwrap();
            for i in 0..1_000 {
                let key = format!("key-{}", i);
                let value = format!("value-{}", i);
                w.append(key.as_bytes(), value.as_bytes()).unwrap();
            }
            w.finish().unwrap();
        }
        b.iter(|| {
            let mut r = Reader::new(&mut file).unwrap();
            for i in 0..1_000 {
                let key = format!("garbage-{}", i);
                let _ = black_box(r.read(key.as_bytes()));
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
