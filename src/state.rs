use crate::job::Jobs;
use crate::turtle::{Turtles, World};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<RwLock<World>>,
    pub turtles: Arc<RwLock<Turtles>>,
    pub jobs: Arc<RwLock<Jobs>>,
}

impl AppState {
    pub fn new(world: World, turtles: Turtles, jobs: Jobs) -> Self {
        Self {
            world: Arc::new(RwLock::new(world)),
            turtles: Arc::new(RwLock::new(turtles)),
            jobs: Arc::new(RwLock::new(jobs)),
        }
    }
}
