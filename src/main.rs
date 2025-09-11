#![allow(dead_code)]
mod job;
mod pathfinder;
mod state;
mod turtle;
use axum::http::HeaderMap;
use pathfinder::Point3D;

use crate::job::{Job, Jobs};
use crate::turtle::{Block, Turtles, World};
use serde::{Deserialize, Serialize};
use state::AppState;
use std::time::Duration;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

const SAVE_PATH: &str = "data/world.bin";
const SAVE_EVERY: Duration = Duration::from_secs(120);

#[derive(Serialize, Deserialize)]
struct Instructions {
    steps: Vec<String>,
}
impl Instructions {
    fn new() -> Self {
        Instructions { steps: Vec::new() }
    }
}

#[derive(Serialize, Deserialize)]
struct Message {
    text: String,
}

#[derive(Deserialize)]
struct Config {
    secret_key: String,
    port: String,
}

fn key_is_valid(key: &str) -> bool {
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").expect("Failed to read config.toml"),
    )
    .expect("Failed to parse config.toml");
    config.secret_key == key
}

#[tokio::main]
async fn main() {
    let mut main_world = World::new();
    let _ = main_world.load_world(SAVE_PATH);
    let turtles = Turtles::new();
    let jobs = Jobs::new();
    let app_state = AppState::new(main_world, turtles, jobs);

    tokio::spawn(start_periodic_saves(
        app_state.clone(),
        SAVE_PATH.into(),
        SAVE_EVERY,
    ));

    tracing_subscriber::fmt::init();
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").expect("Failed to read config.toml"),
    )
    .expect("Failed to parse config.toml");
    let app = Router::new()
        .route("/", get(root))
        .route("/request-path", post(path_request))
        .route("/update-block", post(block_update))
        .route("/get-instructions", get(get_instructions))
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_string() + &config.port)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Turtle Manager v0.1.0"
}

// main endpoint that is gonna get spammed
async fn get_instructions(State(st): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let instructions = Instructions::new();
    let world = st.world.read().await;
    let jobs = st.jobs.read().await;
    let mut turtles = st.turtles.write().await;
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    if !key_is_valid(auth) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Message {
                text: "Invalid secret key".to_string(),
            }),
        )
            .into_response();
    }
    let turtle_id: u32 = match headers
        .get("turtle-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse().ok())
    {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(Message {
                    text: "Invalid turtle-id header".to_string(),
                }),
            )
                .into_response();
        }
    };
    let turtle = match turtles.get_turtle(turtle_id) {
        Some(t) => Some(t),
        // Turtles will be registered in the post info handler
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(Message {
                    text: "Turtle not found".to_string(),
                }),
            )
                .into_response();
        }
    };

    (StatusCode::OK, axum::Json(instructions)).into_response()
}

async fn block_update(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<StatusUpdate>,
) -> impl IntoResponse {
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    if !key_is_valid(auth) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Message {
                text: "Invalid secret key".to_string(),
            }),
        )
            .into_response();
    }
    let mut world = st.world.write().await; // write lock for concurrent writers
    for block in payload.blocks {
        world.set_block(block);
    }

    StatusCode::OK.into_response()
}

async fn path_request(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PathRequest>,
) -> impl IntoResponse {
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    if !key_is_valid(auth) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Message {
                text: "Invalid secret key".to_string(),
            }),
        )
            .into_response();
    }
    if payload.start == payload.goal {
        let instructions = Instructions::new();
        return (StatusCode::OK, axum::Json(instructions)).into_response();
    }

    let padding: u32 = 2;
    let can_dig: bool = true;

    let world = app.world.read().await;
    let t0 = std::time::Instant::now();
    match world.get_path(payload.start, payload.goal, padding, can_dig) {
        Some(path) => {
            let mut instructions = Instructions::new();
            instructions.steps = path;
            let dt = t0.elapsed();
            println!(
                "Handled request with {} moves in {:.3?}",
                instructions.steps.len(),
                dt
            );
            (StatusCode::OK, axum::Json(instructions)).into_response()
        }
        None => {
            let mut instructions = Instructions::new();
            instructions.steps.push("Error: No path found".to_string());
            let dt = t0.elapsed();
            println!("No path found (took {:.3?})", dt);
            (StatusCode::OK, axum::Json(instructions)).into_response()
        }
    }
}

#[derive(Deserialize)]
struct StatusUpdate {
    blocks: Vec<Block>,
    position: Point3D,
    rotation: u8,
}

// most likely temporary for now for testing, maybe keep if manually
// repositioning turtles is desirable
#[derive(Deserialize)]
struct PathRequest {
    start: Point3D,
    goal: Point3D,
}

async fn save_once(app_state: &AppState, path: &str) {
    // Snapshot JSON while holding a read lock, then write atomically.
    let world = app_state.world.read().await;
    world.save_world(path).unwrap();
}

async fn start_periodic_saves(app_state: AppState, path: String, every: Duration) {
    let mut ticker = tokio::time::interval(every);
    println!("Starting periodic saves every {:?} to {}", every, path);
    loop {
        ticker.tick().await;
        save_once(&app_state, &path).await;
        println!("Saved world to {}", path);
    }
}
