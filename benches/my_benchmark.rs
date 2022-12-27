use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use rokv::sync_read::Writer;

pub fn writer_bench(c: &mut Criterion) {
    c.bench_function("writer_fill_1000", |b| {
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
}

pub fn sync_read_bench(c: &mut Criterion) {
    let mut file = tempfile::tempfile().unwrap();
    {
        let mut w = Writer::new(&mut file).unwrap();
        for i in 0..100_000_000 {
            let key = format!("key-{}", i);
            let value = format!("value-{}", i);
            w.append(key.as_bytes(), value.as_bytes()).unwrap();
        }
        w.finish().unwrap();
    }

    use rokv::sync_read::Reader;
    c.bench_function("sync_read_100mil_exists", |b| {
        let mut r = Reader::new(&mut file).unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key = format!("key-{}", prng.gen_range(0..100_000_000));
            black_box(r.read(key.as_bytes()).unwrap());
        });
    });

    c.bench_function("sync_read_100mil_nonexistent", |b| {
        let mut r = Reader::new(&mut file).unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key = format!("garbage-{}", prng.gen_range(0..100_000_000));
            let _ = black_box(r.read(key.as_bytes()));
        });
    });
}

pub fn mmap_read_bench(c: &mut Criterion) {
    let mut file = tempfile::tempfile().unwrap();
    {
        let mut w = Writer::new(&mut file).unwrap();
        for i in 0..100_000_000 {
            let key = format!("key-{}", i);
            let value = format!("value-{}", i);
            w.append(key.as_bytes(), value.as_bytes()).unwrap();
        }
        w.finish().unwrap();
    }
    use rokv::mmap::Reader;
    c.bench_function("mmap_read_100mil_exists", |b| {
        let mut r = Reader::new(&mut file).unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key = format!("key-{}", prng.gen_range(0..100_000_000));
            black_box(r.read(key.as_bytes()).unwrap());
        });
    });

    c.bench_function("mmap_read_100mil_nonexistent", |b| {
        let mut r = Reader::new(&mut file).unwrap();
        let mut prng = rand::thread_rng();
        b.iter(|| {
            let key = format!("garbage-{}", prng.gen_range(0..100_000_000));
            let _ = black_box(r.read(key.as_bytes()));
        });
    });
}

criterion_group!(benches, writer_bench, sync_read_bench, mmap_read_bench);
criterion_main!(benches);
