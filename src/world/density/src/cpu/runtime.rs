use bevy_math::DVec3;
use crate::cpu::buffer::BufferId;
use crate::cpu::operation::{NegativeDecayType, Operation, ValueSource};
use crate::cpu::{unpack_coord, Workspace};

pub fn execute_function(workspace: &mut Workspace) -> Option<()> {
    for operation in workspace.operations {
        match operation {
            Operation::ClearBuffer {
                destination,
                value
            } => workspace.get_buffer_mut(*destination)?.fill(*value),
            Operation::AddBuffer {
                destination,
                source
            } => handle_add(destination, source, workspace)?,
            Operation::SubBuffer {
                destination,
                source
            } => handle_sub(destination, source, workspace)?,
            Operation::DivBuffer {
                destination,
                source
            } => handle_div(destination, source, workspace)?,
            Operation::MulBuffer {
                destination,
                source
            } => handle_mul(destination, source, workspace)?,
            Operation::MinBuffer {
                destination,
                source
            } => handle_min(destination, source, workspace)?,
            Operation::MaxBuffer {
                destination,
                source,
            } => handle_max(destination, source, workspace)?,
            Operation::AbsBuffer {
                buffer
            } => workspace.get_buffer_mut(*buffer)?.iter_mut().for_each(|v| *v = v.abs()),
            Operation::PowBuffer {
                buffer,
                amount
            } => {
                let amount = amount.as_i32();
                workspace.get_buffer_mut(*buffer)?.iter_mut().for_each(|v| *v = v.powi(amount));
            },
            Operation::NegativeDecayBuffer {
                buffer,
                kind
            } => handle_negative_decay(buffer, kind, workspace)?,
            Operation::ClampBuffer {
                buffer,
                min,
                max
            } => workspace.get_buffer_mut(*buffer)?.iter_mut().for_each(|v| *v = v.clamp(*min, *max)),
        }
    }

    Some(())
}

pub fn handle_add(destination: &BufferId, source: &ValueSource, workspace: &mut Workspace) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f += v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f += n.noise(DVec3::new(x as f64, y as f64, z as f64)) as f32;
        }),
        ValueSource::Buffer(source) if destination == source => dest.iter_mut().for_each(|f| *f *= 2.0),
        ValueSource::Buffer(source) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut().zip(src.iter()).for_each(|(f, src)| *f += src)
            } else {
                dest.iter_mut().for_each(|f| *f += src[0]) // TODO: transform coordinate to index src correctly
            }
        }
    }

    Some(())
}

pub fn handle_sub(destination: &BufferId, source: &ValueSource, workspace: &mut Workspace) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f -= v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f -= n.noise(DVec3::new(x as f64, y as f64, z as f64)) as f32;
        }),
        ValueSource::Buffer(source) if destination == source => dest.fill(0.0),
        ValueSource::Buffer(source) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut().zip(src.iter()).for_each(|(f, src)| *f -= src)
            } else {
                dest.iter_mut().for_each(|f| *f -= src[0]) // TODO: transform coordinate to index src correctly
            }
        }
    }

    Some(())
}

pub fn handle_div(destination: &BufferId, source: &ValueSource, workspace: &mut Workspace) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f /= v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f /= n.noise(DVec3::new(x as f64, y as f64, z as f64)) as f32;
        }),
        ValueSource::Buffer(source) if destination == source => dest.fill(1.0),
        ValueSource::Buffer(source) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut().zip(src.iter()).for_each(|(f, src)| *f /= src)
            } else {
                dest.iter_mut().for_each(|f| *f /= src[0]) // TODO: transform coordinate to index src correctly
            }
        }
    }

    Some(())
}

pub fn handle_mul(destination: &BufferId, source: &ValueSource, workspace: &mut Workspace) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f *= v),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f *= n.noise(DVec3::new(x as f64, y as f64, z as f64)) as f32;
        }),
        ValueSource::Buffer(source) if destination == source => dest.iter_mut().for_each(|f| *f = f.powi(2)),
        ValueSource::Buffer(source) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut().zip(src.iter()).for_each(|(f, src)| *f *= src)
            } else {
                dest.iter_mut().for_each(|f| *f *= src[0]) // TODO: transform coordinate to index src correctly
            }
        }
    }

    Some(())
}

pub fn handle_min(destination: &BufferId, source: &ValueSource, workspace: &mut Workspace) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f = f.min(*v)),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f = f.min(n.noise(DVec3::new(x as f64, y as f64, z as f64)) as f32);
        }),
        ValueSource::Buffer(source) if destination == source => {}, // f.min(f) = f so we don't do anything here
        ValueSource::Buffer(source) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut().zip(src.iter()).for_each(|(f, src)| *f = f.min(*src))
            } else {
                dest.iter_mut().for_each(|f| *f = f.min(src[0])) // TODO: transform coordinate to index src correctly
            }
        }
    }

    Some(())
}

pub fn handle_max(destination: &BufferId, source: &ValueSource, workspace: &mut Workspace) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match source {
        ValueSource::Constant(v) => dest.iter_mut().for_each(|f| *f = f.max(*v)),
        ValueSource::Noise(n) => dest.iter_mut().enumerate().for_each(|(i, f)| {
            let (x, y, z) = unpack_coord(i as u32);
            *f = f.max(n.noise(DVec3::new(x as f64, y as f64, z as f64)) as f32);
        }),
        ValueSource::Buffer(source) if destination == source => {}, // f.max(f) = f so we don't do anything here
        ValueSource::Buffer(source) => {
            let (dest, src) = workspace.get_dst_src(*destination, *source)?;

            if destination.ty == source.ty {
                dest.iter_mut().zip(src.iter()).for_each(|(f, src)| *f = f.max(*src))
            } else {
                dest.iter_mut().for_each(|f| *f = f.max(src[0])) // TODO: transform coordinate to index src correctly
            }
        }
    }

    Some(())
}

pub fn handle_negative_decay(destination: &BufferId, kind: &NegativeDecayType, workspace: &mut Workspace) -> Option<()> {
    let dest = workspace.get_buffer_mut(*destination)?;

    match kind {
        NegativeDecayType::Half => dest.iter_mut().filter(|f| f.is_sign_negative()).for_each(|f| *f /= 2.0),
        NegativeDecayType::Quarter => dest.iter_mut().filter(|f| f.is_sign_negative()).for_each(|f| *f /= 4.0),
    }

    Some(())
}
