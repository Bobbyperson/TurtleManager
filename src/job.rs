use crate::pathfinder::Point3D;

pub struct Jobs {
    jobs: Vec<Job>,
}
impl Jobs {
    pub fn new() -> Self {
        Jobs { jobs: Vec::new() }
    }
}

pub struct Job {
    job_type: String,
    status: String,
    progress: f32,
    path_goal: Option<Point3D>,
    assigned_to: i32, // -1 if unassigned, otherwise the user ID
}
impl Job {
    pub fn new(
        job_type: String,
        status: String,
        progress: f32,
        path_goal: Option<Point3D>,
        assigned_to: i32,
    ) -> Self {
        Job {
            job_type,
            status,
            progress,
            path_goal,
            assigned_to,
        }
    }
}
