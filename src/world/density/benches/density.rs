use std::hint::black_box;
use criterion::{criterion_group, Criterion, criterion_main};
use temper_core::random::XoroshiroRandomSource;
use temper_density::cpu::buffer::{Buffer, BufferId, BufferType};
use temper_density::cpu::operation::{Operation, ValueSource};
use temper_density::cpu::{run, Workspace, OUT_BUFFER_LEN};
use temper_noise::NormalNoise;

fn bench_density(c: &mut Criterion) {
    let mut rand = XoroshiroRandomSource::new(10);

    let ops = [
        Operation::ClearBuffer { destination: BufferId::OUT, value: 5.0 },
        // Operation::AddBuffer { destination: BufferId::OUT, source: ValueSource::Noise(NormalNoise::new(&mut rand, 1, &[3.0, 2.0, 1.0])) },
        Operation::MulBuffer { destination: BufferId::OUT, source: ValueSource::Constant(5.0) },
        Operation::ClearBuffer { destination: BufferId::flat(0), value: 3.0 },
        Operation::AddBuffer { destination: BufferId::flat(0), source: ValueSource::Noise(NormalNoise::new(&mut rand, 4, &[1.0, 2.0, 3.0])) },
        Operation::AddBuffer { destination: BufferId::OUT, source: ValueSource::Buffer(BufferId::flat(0)) },
    ];

    let mut workspace = Workspace {
        out: Buffer { ty: BufferType::Out, data: vec![0.0; OUT_BUFFER_LEN].into_boxed_slice() },
        full: Vec::new(),
        flat: vec![Buffer { ty: BufferType::Flat, data: vec![0.0; BufferType::Flat.size()].into_boxed_slice() }],
        flat_cell: Vec::new(),
        interpolated: Vec::new(),
    };

    c.bench_function("density execution", |b| {
        b.iter(|| {
            black_box(run(black_box(&ops), black_box(&mut workspace)));
        });
    });
}

criterion_group!(benches, bench_density);
criterion_main!(benches);
