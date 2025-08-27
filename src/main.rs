#![allow(dead_code)]
mod pathfinder;
mod state;
mod turtle;
use pathfinder::{Grid, Point3D, astar_find_path, path_to_moves};

use crate::turtle::World;
use serde::{Deserialize, Serialize};
use state::AppState;
use std::{path::Path, time::Duration};

use axum::{
    Json, Router,
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
    main_world.load_world(SAVE_PATH);
    let app_state = AppState::new(main_world);

    tracing_subscriber::fmt::init();
    let config: Config = toml::from_str(
        &std::fs::read_to_string("config.toml").expect("Failed to read config.toml"),
    )
    .expect("Failed to parse config.toml");
    let app = Router::new()
        .route("/", get(root))
        .route("/request-path", post(path_request))
        .route("/update-blocks", post(block_update))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_string() + &config.port)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Turtle Manager v0.1.0"
}

async fn block_update(Json(payload): Json<BlockUpdate>) -> impl IntoResponse {
    // TODO
    StatusCode::OK
}

async fn path_request(Json(payload): Json<PathRequest>) -> impl IntoResponse {
    if !key_is_valid(&payload.secret_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(Message {
                text: "Invalid secret key".to_string(),
            }),
        )
            .into_response();
    }

    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
struct BlockUpdate {
    secret_key: String,
    block_type: String,
    position: Point3D,
}

// most likely temporary for now for testing, maybe keep if manually
// repositioning turtles is desirable
#[derive(Deserialize)]
struct PathRequest {
    secret_key: String,
    start: Point3D,
    goal: Point3D,
}

// fn main() {
//     // Define bounds (inclusive). This is a 201 × 201 × 201 grid.
//     let min = Point3D::new(-100, -100, -100);
//     let max = Point3D::new(100, 100, 100);

//     // Build grid with default movement cost = 1.
//     let mut grid = Grid::new(min, max, 1);

//     // Block a small 3×3×3 cube at the center to mimic "impassable" (cost = 0).
//     // for x in -1..=1 {
//     //     for y in -1..=1 {
//     //         for z in -1..=1 {
//     //             grid.set_cost(Point3D::new(x, y, z), 0);
//     //         }
//     //     }
//     // }

//     // // Example of weighted region: a "slow" plane at z = 10 with cost 3.
//     // for x in -50..=50 {
//     //     for y in -50..=50 {
//     //         grid.set_cost(Point3D::new(x, y, 10), 3);
//     //     }
//     // }

//     for x in -100..=100 {
//         for y in -100..=100 {
//             for z in -100..=100 {
//                 grid.set_cost(Point3D::new(x, y, z), rng().random_range(1..=10)); // Random costs between 1 and 10
//             }
//         }
//     }

//     // for x in -100..=100 {
//     //     for y in -100..=100 {
//     //         for z in -100..=100 {
//     //             // Randomly block some points with cost 0
//     //             if rng().random_bool(0.5) { // 20% chance to block
//     //                 grid.set_cost(Point3D::new(x, y, z), 0);
//     //             }
//     //         }
//     //     }
//     // }

//     let start = Point3D::new(-99, -99, -99);
//     let goal = Point3D::new(99, 99, 99);

//     let t0 = std::time::Instant::now();
//     match astar_find_path(&grid, start, goal) {
//         Some(path) => {
//             let dt = t0.elapsed();
//             println!("Found path with {} steps in {:.3?}", path.len(), dt);

//             // Convert to moves
//             match path_to_moves(&grid, &path) {
//                 Ok(moves) => {
//                     println!("Moves count: {}", moves.len());
//                     let preview = moves.len().min(20);
//                     if preview > 0 {
//                         println!("First {} moves:", preview);
//                         for m in &moves[..preview] {
//                             println!("{}", m);
//                         }
//                     }
//                     if moves.len() > preview {
//                         println!("... ({} more moves) ...", moves.len() - preview);
//                     }
//                     let mut instructions = Instructions::new();
//                     instructions.steps = moves;
//                     let json = serde_json::to_string_pretty(&instructions)
//                         .expect("Failed to serialize instructions to JSON");
//                     let mut file = File::create("data/out.json").expect("Failed to create output file");
//                     file.write_all(json.as_bytes())
//                         .expect("Failed to write to output file");
//                 }
//                 Err(e) => {
//                     eprintln!("Path contained an invalid step: {}", e);
//                 }
//             }
//         }
//         None => {
//             let dt = t0.elapsed();
//             println!("No path found (took {:.3?})", dt);
//         }
//     }
// }
