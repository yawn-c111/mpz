use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::FutureExt;
use mpz_common::{
    blocking,
    executor::{test_mt_executor, test_st_executor},
};
use pollster::block_on;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("context");

    // Measures the overhead of making a `Context::blocking` call, which
    // moves the context to a worker thread and back.
    group.bench_function("st/blocking", |b| {
        let (mut ctx, _) = test_st_executor(1024);
        b.iter(|| {
            block_on(async {
                blocking!(ctx, async {
                    black_box(ctx.id());
                })
                .unwrap();
            });
        })
    });

    // Measures the overhead of making a `Context::blocking` call, which
    // moves the context to a worker thread and back.
    group.bench_function("mt/blocking", |b| {
        let (mut exec_a, mut exec_b) = test_mt_executor(128);

        let mut ctx = block_on(async {
            futures::select! {
                ctxs = futures::future::try_join(exec_a.new_thread(), exec_b.new_thread()).fuse() => {
                    ctxs.unwrap().0
                },
                _ = fut.fuse() => {
                    panic!("connection closed");
                }
            }
        });

        b.iter(|| {
            block_on(async {
                blocking!(ctx, async {
                    black_box(ctx.id());
                })
                .unwrap();
            });
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
