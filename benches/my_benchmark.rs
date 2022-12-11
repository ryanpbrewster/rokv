use criterion::{criterion_group, criterion_main, Criterion};
use rokv::{Writer};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fill_1000", |b| b.iter(|| {
        let mut file = tempfile::tempfile().unwrap();
        let mut w = Writer::new(&mut file);
        for i in 0 .. 1_1000 {
            let key = format!("key-{}", i);
            let value = format!("value-{}", i);
            w.append(key.as_bytes(), value.as_bytes()).unwrap();
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);