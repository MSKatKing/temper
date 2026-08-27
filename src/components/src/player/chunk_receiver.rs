use bevy_ecs::prelude::Component;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::{HashSet, VecDeque};
use temper_core::pos::ChunkPos;
use uuid::Uuid;

pub enum PreparedChunk {
    Ready {
        pos: ChunkPos,
        packet_data: Vec<u8>,
        entities: Vec<(Uuid, u16)>,
        is_new_load: bool,
    },
    Failed {
        pos: ChunkPos,
    },
}

impl PreparedChunk {
    /// The chunk this message is about, whether it succeeded or not.
    pub fn pos(&self) -> ChunkPos {
        match self {
            Self::Ready { pos, .. } | Self::Failed { pos } => *pos,
        }
    }
}

#[derive(Component)]
pub struct ChunkReceiver {
    pub loading: VecDeque<(i32, i32)>,
    pub dirty: VecDeque<(i32, i32)>,
    pub loaded: HashSet<(i32, i32)>,
    pub in_flight: HashSet<(i32, i32)>, // dispatched, not yet harvested
    pub unloading: VecDeque<(i32, i32)>,
    pub chunks_per_tick: f32,
    pub ready_tx: Sender<PreparedChunk>,
    pub ready_rx: Receiver<PreparedChunk>,
}

impl ChunkReceiver {
    pub fn new() -> Self {
        let (ready_tx, ready_rx) = unbounded();
        Self {
            loading: VecDeque::new(),
            loaded: HashSet::new(),
            in_flight: HashSet::new(),
            unloading: VecDeque::new(),
            dirty: VecDeque::new(),
            chunks_per_tick: 32.5,
            ready_tx,
            ready_rx,
        }
    }
}

impl Default for ChunkReceiver {
    fn default() -> Self {
        Self::new()
    }
}
