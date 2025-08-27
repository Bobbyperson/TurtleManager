use crate::turtle::World;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<RwLock<World>>,
}

impl AppState {
    pub fn new(world: World) -> Self {
        Self {
            world: Arc::new(RwLock::new(world)),
        }
    }
}
