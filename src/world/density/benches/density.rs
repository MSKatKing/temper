use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;
use temper_core::random::XoroshiroRandomSource;
use temper_density::cpu::buffer::{BufferType, Flat};
use temper_density::cpu::compiler::Compiler;
use temper_density::cpu::workspace::Workspace;
use temper_density::{DensityFunction, DensityFunctionArgument};

fn bench_density(c: &mut Criterion) {
    let mut rand = XoroshiroRandomSource::new(10);

    let mut func = DensityFunctionArgument::Function(Box::new(
        DensityFunction::Add {
            left: DensityFunctionArgument::Function(Box::new(
                DensityFunction::Noise {
                    noise: "minecraft:surface".to_string(),
                    xz_scale: 1.0,
                    y_scale: 1.0,
                }
            )),
            right: DensityFunctionArgument::Function(Box::new(
                DensityFunction::YClampedGradient {
                    from_y: 32,
                    to_y: 96,
                    from_value: 1.0,
                    to_value: -1.0,
                }
            ))
        }
    ));

    func.link_arg(&HashMap::new());
    let func = func.fold();
    let func = Compiler::compile(&mut rand, func);
    
    let mut workspace = Workspace::new(&func);

    let mut group = c.benchmark_group("density execution");
    group.throughput(Throughput::Bytes(
        (<Flat as BufferType>::SIZE * size_of::<f32>()) as u64,
    ));
    group.bench_function("density execution", |b| {
        b.iter(|| black_box(workspace.execute()))
    });
    group.finish();
}

criterion_group!(benches, bench_density);
criterion_main!(benches);
