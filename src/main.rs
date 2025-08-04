mod pathfinder;
mod turtle;
use pathfinder::{Grid, Point3D, astar_find_path, path_to_moves};

use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Instructions {
    steps: Vec<String>,
}
impl Instructions {
    fn new() -> Self {
        Instructions { steps: Vec::new() }
    }
}

fn main() {
    // Define bounds (inclusive). This is a 201 × 201 × 201 grid.
    let min = Point3D::new(-100, -100, -100);
    let max = Point3D::new(100, 100, 100);

    // Build grid with default movement cost = 1.
    let mut grid = Grid::new(min, max, 1);

    // Block a small 3×3×3 cube at the center to mimic "impassable" (cost = 0).
    // for x in -1..=1 {
    //     for y in -1..=1 {
    //         for z in -1..=1 {
    //             grid.set_cost(Point3D::new(x, y, z), 0);
    //         }
    //     }
    // }

    // // Example of weighted region: a "slow" plane at z = 10 with cost 3.
    // for x in -50..=50 {
    //     for y in -50..=50 {
    //         grid.set_cost(Point3D::new(x, y, 10), 3);
    //     }
    // }

    for x in -100..=100 {
        for y in -100..=100 {
            for z in -100..=100 {
                grid.set_cost(Point3D::new(x, y, z), rng().random_range(1..=10)); // Random costs between 1 and 10
            }
        }
    }

    // for x in -100..=100 {
    //     for y in -100..=100 {
    //         for z in -100..=100 {
    //             // Randomly block some points with cost 0
    //             if rng().random_bool(0.5) { // 20% chance to block
    //                 grid.set_cost(Point3D::new(x, y, z), 0);
    //             }
    //         }
    //     }
    // }

    let start = Point3D::new(-99, -99, -99);
    let goal = Point3D::new(99, 99, 99);

    let t0 = std::time::Instant::now();
    match astar_find_path(&grid, start, goal) {
        Some(path) => {
            let dt = t0.elapsed();
            println!("Found path with {} steps in {:.3?}", path.len(), dt);

            // Convert to moves
            match path_to_moves(&grid, &path) {
                Ok(moves) => {
                    println!("Moves count: {}", moves.len());
                    let preview = moves.len().min(20);
                    if preview > 0 {
                        println!("First {} moves:", preview);
                        for m in &moves[..preview] {
                            println!("{}", m);
                        }
                    }
                    if moves.len() > preview {
                        println!("... ({} more moves) ...", moves.len() - preview);
                    }
                    let mut instructions = Instructions::new();
                    instructions.steps = moves;
                    let json = serde_json::to_string_pretty(&instructions)
                        .expect("Failed to serialize instructions to JSON");
                    let mut file = File::create("data/out.json").expect("Failed to create output file");
                    file.write_all(json.as_bytes())
                        .expect("Failed to write to output file");
                }
                Err(e) => {
                    eprintln!("Path contained an invalid step: {}", e);
                }
            }
        }
        None => {
            let dt = t0.elapsed();
            println!("No path found (took {:.3?})", dt);
        }
    }
}
