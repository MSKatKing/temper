use crate::cpu::buffer::BufferId;
use crate::cpu::operation::{NegativeDecayType, Operation, ValueSource};
use crate::cpu::Workspace;
use std::ops::RangeInclusive;
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
                    ValueSource::Buffer(..) => todo!("copy buffer to buffer"),
                    ValueSource::Constant(_) => unreachable!(),
                };
                
                let chunk_pos = workspace.current_pos;
                let buffer = workspace.get_buffer_mut(*destination)?;

                buffer.pos_iter().for_each(|(local_pos, v)| {
                    *v = noise.noise(chunk_pos.chunk_block(local_pos))
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
        .filter(|(local_pos, _)| {
            local_pos.y() > *y_range.start()
        })
        .for_each(|(local_pos, v)| {
            if local_pos.y() > *y_range.end() {
                *v = *value_range.end()
            } else {
                *v = lerp((local_pos.y() as f64 - *y_range.start() as f64) / (y_range.end() - y_range.start()) as f64, [*value_range.start() as f64, *value_range.end() as f64]) as f32;
            }
        });

    Some(())
}

pub fn handle_add(
    destination: &BufferId,
    source: &ValueSource,
    workspace: &mut Workspace,
) -> Option<()> {
    let chunk_pos = workspace.current_pos;
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f += v),
        ValueSource::Noise(n) => dest.pos_iter().for_each(|(local_pos, f)| {
            *f += n.noise(chunk_pos.chunk_block(local_pos));
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
    let chunk_pos = workspace.current_pos;
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f -= v),
        ValueSource::Noise(n) => dest.pos_iter().for_each(|(local_pos, f)| {
            *f -= n.noise(chunk_pos.chunk_block(local_pos));
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
    let chunk_pos = workspace.current_pos;
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f /= v),
        ValueSource::Noise(n) => dest.pos_iter().for_each(|(local_pos, f)| {
            *f /= n.noise(chunk_pos.chunk_block(local_pos));
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
    let chunk_pos = workspace.current_pos;
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f *= v),
        ValueSource::Noise(n) => dest.pos_iter().for_each(|(local_pos, f)| {
            *f *= n.noise(chunk_pos.chunk_block(local_pos));
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
    let chunk_pos = workspace.current_pos;
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f = f.min(*v)),
        ValueSource::Noise(n) => dest.pos_iter().for_each(|(local_pos, f)| {
            *f = f.min(n.noise(chunk_pos.chunk_block(local_pos)));
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
    let chunk_pos = workspace.current_pos;
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f = f.max(*v)),
        ValueSource::Noise(n) => dest.pos_iter().for_each(|(local_pos, f)| {
            *f = f.max(n.noise(chunk_pos.chunk_block(local_pos)));
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
