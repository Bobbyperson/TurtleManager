use crate::turtle::{Turtles, World};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<RwLock<World>>,
    pub turtles: Arc<RwLock<Turtles>>,
}

impl AppState {
    pub fn new(world: World, turtles: Turtles) -> Self {
        Self {
            world: Arc::new(RwLock::new(world)),
            turtles: Arc::new(RwLock::new(turtles)),
        }
    }
}
