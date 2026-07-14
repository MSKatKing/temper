use criterion::{criterion_group, criterion_main};
use rand::random;
use std::hint::black_box;
use temper_core::pos::ChunkPos;

fn gen_benches(c: &mut criterion::Criterion) {
    bench_gun(c);
}
criterion_group!(world_bench, gen_benches);
criterion_main!(world_bench);

fn bench_gun(c: &mut criterion::Criterion) {
    let generator = world_gen::WorldGenerator::new(random());

    c.bench_function("generate chunk", |b| {
        b.iter(|| {
            let pos = ChunkPos::new(
                rand::random_range(i32::from(i16::MIN)..i32::from(i16::MAX)),
                rand::random_range(i32::from(i16::MIN)..i32::from(i16::MAX)),
            );
            black_box(generator.generate_chunk(pos)).unwrap();
        });
    });
}
