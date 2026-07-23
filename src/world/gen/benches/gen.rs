use criterion::{criterion_group, criterion_main};
use rand::random;
use std::hint::black_box;
use temper_core::pos::ChunkPos;

fn gen_benches(c: &mut criterion::Criterion) {
    bench_gen(c);
}
criterion_group!(world_bench, gen_benches);
criterion_main!(world_bench);

fn bench_gen(c: &mut criterion::Criterion) {
    // let mut group = c.benchmark_group("world_gen");
    //
    // for size in [1, 8, 16] {
    //     group.throughput(criterion::Throughput::Elements((size * size) as u64));
    //     group.bench_function(format!("generate to {}, {}", size, size), |b| {
    //         b.iter(|| {
    //             let generator = world_gen::WorldGenerator::new(random());
    //             for x in 0..size {
    //                 for z in 0..size {
    //                     let pos = black_box(ChunkPos::new(x, z));
    //                     black_box(generator.generate_chunk(pos)).unwrap();
    //                 }
    //             }
    //         });
    //     });
    // }
    //
    // group.finish();
}
