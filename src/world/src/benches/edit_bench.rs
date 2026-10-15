use criterion::{Criterion, Throughput};
use rand::RngExt;
use std::env::temp_dir;
use std::hint::black_box;
use temper_core::block_state_id::BlockStateId;
use temper_macros::block;
use temper_world::World;

fn get_rand_in_range(min: i32, max: i32) -> i32 {
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}

#[expect(clippy::unit_arg)]
pub(crate) fn bench_edits(c: &mut Criterion) {
    let world = World::new(
        temp_dir().join("edit_bench"),
        &temper_config::server_config::create_dummy_config(),
    )
    .expect("Failed to create world");
    let chunk = world
        .get_or_generate_chunk(
            temper_core::pos::ChunkPos::new(0, 0)
                .full(temper_core::dimension::Dimension::Overworld),
        )
        .expect("Failed to get or generate chunk")
        .clone();

    let mut read_group = c.benchmark_group("edit_read");

    read_group.throughput(Throughput::Elements(1));

    read_group.bench_function("Read 0,0,0", |b| {
        b.iter(|| black_box(chunk.get_block((0, 0, 0).into())));
    });

    read_group.bench_function("Read 8,8,150", |b| {
        b.iter(|| black_box(chunk.get_block((8, 150, 8).into())));
    });

    read_group.bench_function("Read rand", |b| {
        b.iter(|| {
            black_box(
                chunk.get_block(
                    (
                        get_rand_in_range(0, 15) as u8,
                        get_rand_in_range(0, 255) as i16,
                        get_rand_in_range(0, 15) as u8,
                    )
                        .into(),
                ),
            )
        });
    });

    read_group.finish();

    let mut write_group = c.benchmark_group("edit_write");

    write_group.throughput(Throughput::Elements(1));

    write_group.bench_with_input("Write 0,0,0", &chunk, |b, chunk| {
        b.iter(|| {
            let mut chunk = chunk.clone();
            black_box(chunk.set_block((0, 0, 0).into(), block!("bricks")));
        });
    });

    write_group.bench_with_input("Write 8,8,150", &chunk, |b, chunk| {
        b.iter(|| {
            let mut chunk = chunk.clone();
            black_box(chunk.set_block((8, 150, 8).into(), block!("bricks")));
        });
    });

    write_group.bench_with_input("Write rand", &chunk, |b, chunk| {
        b.iter(|| {
            let mut chunk = chunk.clone();
            black_box(
                chunk.set_block(
                    (
                        get_rand_in_range(0, 15) as u8,
                        get_rand_in_range(0, 255) as i16,
                        get_rand_in_range(0, 15) as u8,
                    )
                        .into(),
                    block!("bricks"),
                ),
            );
        });
    });

    write_group.throughput(Throughput::Elements(16 * 256 * 16));

    write_group.bench_with_input("Fill", &chunk, |b, chunk| {
        b.iter(|| {
            let mut chunk = chunk.clone();
            black_box(chunk.fill(block!("bricks")));
        });
    });

    write_group.bench_with_input("Manual Fill", &chunk, |b, chunk| {
        b.iter(|| {
            let mut chunk = chunk.clone();
            for x in 0..16 {
                for y in 0..256 {
                    for z in 0..16 {
                        black_box(chunk.set_block((x, y, z).into(), block!("bricks")));
                    }
                }
            }
        });
    });

    write_group.finish();
}
