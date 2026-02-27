use criterion::{black_box, criterion_group, criterion_main, Criterion};
use navier_tui::lbm::LbmEngine;

fn lbm_benchmark(c: &mut Criterion) {
    // A standard terminal size block
    let mut engine = LbmEngine::new(120, 60, 0.6);
    
    c.bench_function("lbm tick 120x60", |b| {
        b.iter(|| {
            engine.tick();
            black_box(&engine);
        })
    });
}

criterion_group!(benches, lbm_benchmark);
criterion_main!(benches);
