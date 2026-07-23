use std::sync::Mutex;
use std::sync::atomic::Ordering::{AcqRel, Acquire, Release};
use std::sync::atomic::{AtomicU8, AtomicUsize};

use gen_core::GenStage;
use temper_core::dimension::Dimension;
use temper_core::pos::ChunkPos;

use crate::RequestId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JobKey {
    pub dimension: Dimension,
    pub pos: ChunkPos,
    pub stage: GenStage,
}

impl JobKey {
    pub const fn new(dimension: Dimension, pos: ChunkPos, stage: GenStage) -> Self {
        Self {
            dimension,
            pos,
            stage,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum JobState {
    Waiting = 0,
    Ready = 1,
    Running = 2,
    Complete = 3,
    Failed = 4,
}

impl JobState {
    const fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::Waiting,
            1 => Self::Ready,
            2 => Self::Running,
            3 => Self::Complete,
            4 => Self::Failed,
            _ => Self::Failed,
        }
    }
}

#[derive(Debug)]
pub struct AtomicJobState {
    state: AtomicU8,
}

impl AtomicJobState {
    pub const fn new(state: JobState) -> Self {
        Self {
            state: AtomicU8::new(state as u8),
        }
    }

    pub fn load(&self) -> JobState {
        JobState::from_raw(self.state.load(Acquire))
    }

    pub fn mark_ready(&self) -> bool {
        self.state
            .compare_exchange(
                JobState::Waiting as u8,
                JobState::Ready as u8,
                AcqRel,
                Acquire,
            )
            .is_ok()
    }

    pub fn try_claim(&self) -> bool {
        self.state
            .compare_exchange(
                JobState::Ready as u8,
                JobState::Running as u8,
                AcqRel,
                Acquire,
            )
            .is_ok()
    }

    pub fn mark_complete(&self) {
        self.state.store(JobState::Complete as u8, Release);
    }

    pub fn mark_complete_once(&self) -> bool {
        self.state.swap(JobState::Complete as u8, AcqRel) != JobState::Complete as u8
    }

    pub fn mark_failed(&self) {
        self.state.store(JobState::Failed as u8, Release);
    }
}

#[derive(Debug)]
pub struct JobEntry {
    pub key: JobKey,
    pub state: AtomicJobState,
    pub remaining_dependencies: AtomicUsize,
    dependents: Mutex<Vec<JobKey>>,
    interested_requests: Mutex<Vec<RequestId>>,
}

impl JobEntry {
    pub fn new(key: JobKey, remaining_dependencies: usize) -> Self {
        let state = if remaining_dependencies == 0 {
            JobState::Ready
        } else {
            JobState::Waiting
        };

        Self {
            key,
            state: AtomicJobState::new(state),
            remaining_dependencies: AtomicUsize::new(remaining_dependencies),
            dependents: Mutex::new(Vec::new()),
            interested_requests: Mutex::new(Vec::new()),
        }
    }

    pub fn add_dependent(&self, dependent: JobKey) {
        let mut dependents = self.dependents.lock().expect("dependent list poisoned");
        if !dependents.contains(&dependent) {
            dependents.push(dependent);
        }
    }

    pub fn dependents(&self) -> Vec<JobKey> {
        self.dependents
            .lock()
            .expect("dependent list poisoned")
            .clone()
    }

    pub fn add_interested_request(&self, request: RequestId) -> bool {
        let mut interested_requests = self
            .interested_requests
            .lock()
            .expect("interested request list poisoned");
        if !interested_requests.contains(&request) {
            interested_requests.push(request);
            true
        } else {
            false
        }
    }

    pub fn interested_requests(&self) -> Vec<RequestId> {
        self.interested_requests
            .lock()
            .expect("interested request list poisoned")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_state_claims_ready_work_once() {
        let state = AtomicJobState::new(JobState::Ready);

        assert!(state.try_claim());
        assert!(!state.try_claim());
        assert_eq!(state.load(), JobState::Running);
    }
}
