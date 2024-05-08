use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mpz_circuits::circuits::build_sha256;

fn criterion_benchmark(c: &mut Criterion) {
    let length = 512;

    c.bench_function("build_sha256", move |bench| {
        bench.iter(|| black_box(build_sha256(0, length)))
    });

    let sha256 = build_sha256(0, length);
    c.bench_function("compute_sha256", |bench| {
        bench.iter(|| {
            black_box(
                sha256
                    .evaluate(&[[0u32; 8].into(), vec![0u8; length].into()])
                    .unwrap(),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
