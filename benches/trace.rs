use batch_tracing::{new_span, root_scope};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn trace_wide_bench(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "trace_wide",
        |b, len| {
            b.iter(|| {
                let r = {
                    let (root_scope, collector) = root_scope("root");

                    let _sg = root_scope.start_scope();

                    if *len > 1 {
                        for _ in 0..*len - 1 {
                            let _g = new_span("span");
                        }
                    }

                    collector
                }
                .collect(false, None, None);

                black_box(r);
            });
        },
        vec![1, 10, 100, 1000, 10000],
    );
}

criterion_group!(benches, trace_wide_bench);
criterion_main!(benches);
