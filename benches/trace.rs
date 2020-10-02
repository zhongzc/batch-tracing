use batch_tracing::local::span_guard::LocalSpanGuard;
use batch_tracing::trace::tracer::Tracer;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn trace_wide_bench(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "trace_wide",
        |b, len| {
            b.iter(|| {
                let root = Tracer::root("root");
                let mut s = root.new_trapper();
                s.set_trap();

                if *len > 1 {
                    for _ in 0..*len - 1 {
                        let _g = LocalSpanGuard::new("1");
                    }
                }
            });
        },
        vec![1, 10, 100, 1000, 10000],
    );
}

criterion_group!(benches, trace_wide_bench,);
criterion_main!(benches);
