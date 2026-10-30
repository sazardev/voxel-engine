//! Chunk work off the main thread when the device has threads, inline within a budget when it
//! does not (ADR 0013).

use bevy::prelude::Resource;
use bevy::tasks::futures::check_ready;
use bevy::tasks::{AsyncComputeTaskPool, Task, available_parallelism};

/// Where chunk generation and meshing run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum ChunkWorkers {
    /// On the async compute task pool; the main thread only starts jobs and takes results.
    Background,
    /// On the main thread, a few per frame: the path for devices without threads (web).
    Inline,
}

impl ChunkWorkers {
    /// Background when the device runs more than one thread, inline otherwise.
    #[must_use]
    pub fn for_this_device() -> Self {
        if available_parallelism() > 1 {
            Self::Background
        } else {
            Self::Inline
        }
    }

    /// How much chunk work may start, run and finish per frame with these workers.
    #[must_use]
    pub const fn budget(self) -> StreamBudget {
        match self {
            Self::Background => StreamBudget {
                generation_starts: 32,
                generation_in_flight: 64,
                meshing_starts: 16,
                meshing_in_flight: 32,
                mesh_uploads: 8,
            },
            Self::Inline => StreamBudget {
                generation_starts: 8,
                generation_in_flight: 8,
                meshing_starts: 4,
                meshing_in_flight: 4,
                mesh_uploads: 4,
            },
        }
    }
}

/// Bounds on chunk work. Every queue in streaming is capped by one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamBudget {
    pub generation_starts: usize,
    pub generation_in_flight: usize,
    pub meshing_starts: usize,
    pub meshing_in_flight: usize,
    /// Meshes added to the asset store (and so uploaded to the GPU) per frame.
    pub mesh_uploads: usize,
}

/// One unit of chunk work. Dropping a running job cancels it.
#[derive(Debug)]
pub enum Job<T> {
    Done(Option<T>),
    Running(Task<T>),
}

impl<T: Send + 'static> Job<T> {
    pub fn start(workers: ChunkWorkers, work: impl FnOnce() -> T + Send + 'static) -> Self {
        match workers {
            ChunkWorkers::Inline => Self::Done(Some(work())),
            ChunkWorkers::Background => {
                Self::Running(AsyncComputeTaskPool::get().spawn(async move { work() }))
            }
        }
    }

    /// The result, once; `None` while the job is still running or after it was taken.
    pub fn poll(&mut self) -> Option<T> {
        match self {
            Self::Done(result) => result.take(),
            Self::Running(task) => check_ready(task),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_inline_job_is_done_at_once_and_yields_its_result_once() {
        let mut job = Job::start(ChunkWorkers::Inline, || 7);

        assert_eq!(job.poll(), Some(7));
        assert_eq!(job.poll(), None);
    }

    #[test]
    fn background_workers_may_keep_more_work_in_flight() {
        let (background, inline) = (
            ChunkWorkers::Background.budget(),
            ChunkWorkers::Inline.budget(),
        );

        assert!(background.meshing_in_flight > inline.meshing_in_flight);
        assert!(inline.generation_starts <= inline.generation_in_flight);
    }
}
