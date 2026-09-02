use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use temper_core::pos::ChunkPos;
use temper_core::random::XoroshiroRandomSource;
use temper_density::cpu::buffer::{Buffer, BufferId, BufferType};
use temper_density::cpu::noise::{NoiseAccessType, NoiseAccessor};
use temper_density::cpu::operation::{Operation, ValueSource};
use temper_density::cpu::workspace::Workspace;
use temper_noise::NormalNoise;

fn bench_density(c: &mut Criterion) {
    let mut rand = XoroshiroRandomSource::new(10);

    let operations = [
        Operation::ClearBuffer {
            destination: BufferId::OUT,
            source: ValueSource::Constant(5.0),
        },
        // Operation::AddBuffer { destination: BufferId::OUT, source: ValueSource::Noise(NormalNoise::new(&mut rand, 1, &[3.0, 2.0, 1.0])) },
        Operation::MulBuffer {
            destination: BufferId::OUT,
            source: ValueSource::Constant(5.0),
        },
        Operation::ClearBuffer {
            destination: BufferId::flat(0),
            source: ValueSource::Constant(3.0),
        },
        Operation::AddBuffer {
            destination: BufferId::flat(0),
            source: ValueSource::Noise(
                NoiseAccessor::new_noise(
                    NormalNoise::new(&mut rand, 4, &[1.0, 2.0, 3.0]),
                    NoiseAccessType::Basic { xz_scale: 1.0, y_scale: 1.0, },
                ),
            ),
        },
        Operation::AddBuffer {
            destination: BufferId::OUT,
            source: ValueSource::Buffer(BufferId::flat(0)),
        },
    ];

    let mut workspace = Workspace {
        out: Buffer::new(BufferType::Out),
        full: Vec::new(),
        flat: vec![Buffer::new(BufferType::Flat)],
        flat_cell: Vec::new(),
        interpolated: Vec::new(),
        operations: &operations,
        current_pos: ChunkPos::new(0, 0),
    };

    let mut group = c.benchmark_group("density execution");
    group.throughput(Throughput::Bytes(
        (BufferType::Out.size().get() * size_of::<f32>()) as u64,
    ));
    group.bench_function("density execution", |b| {
        b.iter(|| black_box(workspace.execute()))
    });
    group.finish();
}

criterion_group!(benches, bench_density);
criterion_main!(benches);
