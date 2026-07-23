use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::AcqRel;

use crossbeam_queue::SegQueue;
use dashmap::DashMap;

use crate::{JobEntry, JobKey, RequestEntry, RequestId};

#[derive(Debug)]
pub struct SchedulerState {
    pub jobs: DashMap<JobKey, Arc<JobEntry>>,
    pub requests: DashMap<RequestId, Arc<RequestEntry>>,
    pub global_ready: SegQueue<JobKey>,
    next_request_id: AtomicU64,
    next_ready_sequence: AtomicU64,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self {
            jobs: DashMap::new(),
            requests: DashMap::new(),
            global_ready: SegQueue::new(),
            next_request_id: AtomicU64::new(0),
            next_ready_sequence: AtomicU64::new(0),
        }
    }
}

impl SchedulerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_request_id(&self) -> RequestId {
        RequestId::new(self.next_request_id.fetch_add(1, AcqRel))
    }

    pub fn next_ready_sequence(&self) -> u64 {
        self.next_ready_sequence.fetch_add(1, AcqRel)
    }
}
