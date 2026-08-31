use std::ops::RangeInclusive;
use crate::cpu::buffer::BufferId;
use crate::cpu::operation::{NegativeDecayType, Operation, ValueSource};
use crate::cpu::{unpack_buffer_coord, unpack_coord, Workspace};
use bevy_math::DVec3;
use temper_core::math::lerp;

pub fn execute_function(workspace: &mut Workspace) -> Option<()> {
    for operation in workspace.operations {
        match operation {
            Operation::ClearBuffer { destination, source } if let ValueSource::Constant(value) = source => {
                workspace.get_buffer_mut(*destination)?.fill(*value);
            }
            Operation::ClearBuffer { destination, source } => {
                let noise = match source {
                    ValueSource::Noise(accessor) => accessor,
                    ValueSource::Buffer(..) => panic!("cannot clear buffer with another buffer"),
                    ValueSource::Constant(_) => unreachable!(),
                };

                let buffer = workspace.get_buffer_mut(*destination)?;
                let ty = destination.ty;

                buffer.iter_mut().enumerate().for_each(|(i, v)| {
                    let (x, y, z) = unpack_buffer_coord(i as u32, ty);
                    *v = noise.noise(DVec3::new(x as _, y as _, z as _))
                })
            }
            Operation::YClampedGradient {
                destination,
                y_range,
                value_range,
            } => handle_y_clamped_gradient(*destination, y_range.clone(), value_range.clone(), workspace)?,
            Operation::AddBuffer {
                destination,
                source,
            } => handle_add(destination, source, workspace)?,
            Operation::SubBuffer {
                destination,
                source,
            } => handle_sub(destination, source, workspace)?,
            Operation::DivBuffer {
                destination,
                source,
            } => handle_div(destination, source, workspace)?,
            Operation::MulBuffer {
                destination,
                source,
            } => handle_mul(destination, source, workspace)?,
            Operation::MinBuffer {
                destination,
                source,
            } => handle_min(destination, source, workspace)?,
            Operation::MaxBuffer {
                destination,
                source,
            } => handle_max(destination, source, workspace)?,
            Operation::AbsBuffer { buffer } => workspace
                .get_buffer_mut(*buffer)?
                .iter_mut()
                .for_each(|v| *v = v.abs()),
            Operation::PowBuffer { buffer, amount } => {
                let amount = amount.as_i32();
                workspace
                    .get_buffer_mut(*buffer)?
                    .iter_mut()
                    .for_each(|v| *v = v.powi(amount));
            }
            Operation::NegativeDecayBuffer { buffer, kind } => {
                handle_negative_decay(buffer, kind, workspace)?
            }
            Operation::ClampBuffer { buffer, min, max } => workspace
                .get_buffer_mut(*buffer)?
                .iter_mut()
                .for_each(|v| *v = v.clamp(*min, *max)),
        }
    }

    Some(())
}

pub fn handle_y_clamped_gradient(
    destination: BufferId,
    y_range: RangeInclusive<i16>,
    value_range: RangeInclusive<f32>,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(destination)?;

    dest.fill(*value_range.start());

    dest
        .pos_iter()
        .filter(|(_, y, _, _)| {
            y > y_range.start()
        })
        .for_each(|(_, y, _, v)| {
            if y > *y_range.end() {
                *v = *value_range.end()
            } else {
                *v = lerp((y as f64 - *y_range.start() as f64) / (y_range.end() - y_range.start()) as f64, [*value_range.start() as f64, *value_range.end() as f64]) as f32;
            }
        });

    Some(())
}

pub fn handle_add(
    destination: &BufferId,
    source: &ValueSource,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f += v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f += n.noise(DVec3::new(x as f64, y as f64, z as f64));
        }),
        ValueSource::Buffer(source, _) if destination == source => {
            dest.iter_mut().for_each(|f| *f *= 2.0)
        }
        ValueSource::Buffer(source, projection) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut()
                    .zip(src.iter())
                    .for_each(|(f, src)| *f += src)
            } else {
                for (i, f) in dest.iter_mut().enumerate() {
                    *f += src[projection.project(i)]
                }
            }
        }
    }

    Some(())
}

pub fn handle_sub(
    destination: &BufferId,
    source: &ValueSource,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f -= v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f -= n.noise(DVec3::new(x as f64, y as f64, z as f64));
        }),
        ValueSource::Buffer(source, _) if destination == source => dest.fill(0.0),
        ValueSource::Buffer(source, projection) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut()
                    .zip(src.iter())
                    .for_each(|(f, src)| *f -= src)
            } else {
                for (i, f) in dest.iter_mut().enumerate() {
                    *f -= src[projection.project(i)]
                }
            }
        }
    }

    Some(())
}

pub fn handle_div(
    destination: &BufferId,
    source: &ValueSource,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f /= v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f /= n.noise(DVec3::new(x as f64, y as f64, z as f64));
        }),
        ValueSource::Buffer(source, _) if destination == source => dest.fill(1.0),
        ValueSource::Buffer(source, projection) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut()
                    .zip(src.iter())
                    .for_each(|(f, src)| *f /= src)
            } else {
                for (i, f) in dest.iter_mut().enumerate() {
                    *f /= src[projection.project(i)]
                }
            }
        }
    }

    Some(())
}

pub fn handle_mul(
    destination: &BufferId,
    source: &ValueSource,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f *= v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f *= n.noise(DVec3::new(x as f64, y as f64, z as f64));
        }),
        ValueSource::Buffer(source, _) if destination == source => {
            dest.iter_mut().for_each(|f| *f = f.powi(2))
        }
        ValueSource::Buffer(source, projection) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut()
                    .zip(src.iter())
                    .for_each(|(f, src)| *f *= src)
            } else {
                for (i, f) in dest.iter_mut().enumerate() {
                    *f *= src[projection.project(i)]
                }
            }
        }
    }

    Some(())
}

pub fn handle_min(
    destination: &BufferId,
    source: &ValueSource,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f = f.min(*v)),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f = f.min(n.noise(DVec3::new(x as f64, y as f64, z as f64)));
        }),
        ValueSource::Buffer(source, _) if destination == source => {} // f.min(f) = f so we don't do anything here
        ValueSource::Buffer(source, projection) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut()
                    .zip(src.iter())
                    .for_each(|(f, src)| *f = f.min(*src))
            } else {
                for (i, f) in dest.iter_mut().enumerate() {
                    *f = f.min(src[projection.project(i)])
                }
            }
        }
    }

    Some(())
}

pub fn handle_max(
    destination: &BufferId,
    source: &ValueSource,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f = f.max(*v)),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f = f.max(n.noise(DVec3::new(x as f64, y as f64, z as f64)));
        }),
        ValueSource::Buffer(source, _) if destination == source => {} // f.max(f) = f so we don't do anything here
        ValueSource::Buffer(source, projection) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut()
                    .zip(src.iter())
                    .for_each(|(f, src)| *f = f.max(*src))
            } else {
                for (i, f) in dest.iter_mut().enumerate() {
                    *f = f.max(src[projection.project(i)])
                }
            }
        }
    }

    Some(())
}

pub fn handle_negative_decay(
    destination: &BufferId,
    kind: &NegativeDecayType,
    workspace: &mut Workspace,
) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match kind {
        NegativeDecayType::Half => dest
            .iter_mut()
            .filter(|f| f.is_sign_negative())
            .for_each(|f| *f /= 2.0),
        NegativeDecayType::Quarter => dest
            .iter_mut()
            .filter(|f| f.is_sign_negative())
            .for_each(|f| *f /= 4.0),
    }

    Some(())
}
