use crate::pathfinder::Point3D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    InProgress,
    Paused,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: JobId,
    pub status: JobStatus,
    pub progress: f32,
    pub assigned_to: Option<i32>, // None = unassigned
    pub kind: JobKind,
}

#[derive(Debug, Clone)]
pub enum JobKind {
    /// Move to a specific point.
    Goto { target: Point3D, tolerance: f32 },

    /// Excavate a rectangular area
    Quarry {
        top_corner: Point3D,
        bottom_corner: Point3D,
        valuables: Vec<String>,
        storage: Option<Point3D>,
        dump_site: Option<Point3D>,
    },

    /// Create a strip mine
    StripMine {
        start: Point3D,
        direction: Direction3,
        length: u32,
        spacing: u32,
        lanes: u32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Direction3 {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

#[derive(Default, Debug)]
pub struct Jobs {
    jobs: Vec<Job>,
}

impl Jobs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, job: Job) -> JobId {
        self.jobs.push(job.clone());
        job.id
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Job> {
        self.jobs.iter_mut()
    }
}

impl Job {
    pub fn new(id: JobId, kind: JobKind) -> Self {
        Self {
            id,
            status: JobStatus::Pending,
            progress: 0.0,
            assigned_to: None,
            kind,
        }
    }

    pub fn path_goal(&self) -> Option<Point3D> {
        match &self.kind {
            JobKind::Goto { target, .. } => Some(*target),
            JobKind::Quarry { top_corner, .. } => Some(*top_corner),
            JobKind::StripMine { start, .. } => Some(*start),
        }
    }
}
