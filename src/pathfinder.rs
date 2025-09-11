use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// Core types

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct Point3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Point3D {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Point3D { x, y, z }
    }
    #[inline]
    pub fn manhattan_distance(&self, other: &Point3D) -> u32 {
        (self.x - other.x).abs() as u32
            + (self.y - other.y).abs() as u32
            + (self.z - other.z).abs() as u32
    }
}

// Heap state: we want lowest f at the top, so reverse comparisons.
#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    f: u32,
    g: u32,
    idx: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.g.cmp(&self.g))
            .then_with(|| other.idx.cmp(&self.idx))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Grid with flat storage
pub struct Grid {
    min: Point3D,                // inclusive
    dims: (usize, usize, usize), // (nx, ny, nz)
    costs: Vec<u16>,             // 0 => blocked; else positive movement cost
}

impl Grid {
    /// Create a grid spanning [min, max] inclusive on each axis.
    /// Fills with `default_cost`.
    pub fn new(min: Point3D, max: Point3D, default_cost: u16) -> Self {
        assert!(max.x >= min.x && max.y >= min.y && max.z >= min.z);
        let nx = (max.x - min.x + 1) as usize;
        let ny = (max.y - min.y + 1) as usize;
        let nz = (max.z - min.z + 1) as usize;
        let len = nx
            .checked_mul(ny)
            .and_then(|v| v.checked_mul(nz))
            .expect("grid too large");
        let mut costs = Vec::with_capacity(len);
        costs.resize(len, default_cost);
        Grid {
            min,
            dims: (nx, ny, nz),
            costs,
        }
    }

    #[inline]
    pub fn in_bounds(&self, p: Point3D) -> bool {
        let nx = self.dims.0 as i32;
        let ny = self.dims.1 as i32;
        let nz = self.dims.2 as i32;
        let ix = p.x - self.min.x;
        let iy = p.y - self.min.y;
        let iz = p.z - self.min.z;
        ix >= 0 && ix < nx && iy >= 0 && iy < ny && iz >= 0 && iz < nz
    }

    /// Convert a point to an index. Returns None if out of bounds.
    #[inline]
    fn idx(&self, p: Point3D) -> Option<usize> {
        if !self.in_bounds(p) {
            return None;
        }
        let ix = (p.x - self.min.x) as usize;
        let iy = (p.y - self.min.y) as usize;
        let iz = (p.z - self.min.z) as usize;
        Some((ix * self.dims.1 + iy) * self.dims.2 + iz)
    }

    /// Convert an index back to a point.
    #[inline]
    fn point(&self, idx: usize) -> Point3D {
        let (_nx, ny, nz) = self.dims;
        let ix = idx / (ny * nz);
        let rem = idx % (ny * nz);
        let iy = rem / nz;
        let iz = rem % nz;
        Point3D {
            x: self.min.x + ix as i32,
            y: self.min.y + iy as i32,
            z: self.min.z + iz as i32,
        }
    }

    /// Set a movement cost at a point. 0 means blocked.
    pub fn set_cost(&mut self, p: Point3D, cost: u16) {
        if let Some(i) = self.idx(p) {
            self.costs[i] = cost;
        }
    }

    #[inline]
    fn cost_idx(&self, idx: usize) -> u16 {
        self.costs[idx]
    }

    #[inline]
    fn cost_point(&self, p: Point3D) -> Option<u16> {
        self.idx(p).map(|i| self.costs[i])
    }
}

pub fn astar_find_path(grid: &Grid, start: Point3D, goal: Point3D) -> Option<Vec<Point3D>> {
    let start_idx = grid.idx(start)?;
    let goal_idx = grid.idx(goal)?;

    if grid.cost_idx(start_idx) == 0 || grid.cost_idx(goal_idx) == 0 {
        return None; // start or goal blocked
    }

    let n = grid.costs.len();
    let mut g_score = vec![u32::MAX; n];
    g_score[start_idx] = 0;

    // parent: predecessor index for path reconstruction; u32::MAX => none
    let mut parent: Vec<u32> = vec![u32::MAX; n];

    let mut heap = BinaryHeap::new();
    let h0 = start.manhattan_distance(&goal);
    heap.push(State {
        f: h0,
        g: 0,
        idx: start_idx,
    });

    // 6-axis moves
    const DIRS: [Point3D; 6] = [
        Point3D { x: 1, y: 0, z: 0 },
        Point3D { x: -1, y: 0, z: 0 },
        Point3D { x: 0, y: 1, z: 0 },
        Point3D { x: 0, y: -1, z: 0 },
        Point3D { x: 0, y: 0, z: 1 },
        Point3D { x: 0, y: 0, z: -1 },
    ];

    while let Some(State {
        f: _f,
        g,
        idx: current_idx,
    }) = heap.pop()
    {
        // Skip stale entries.
        if g != g_score[current_idx] {
            continue;
        }

        if current_idx == goal_idx {
            return Some(reconstruct_path(grid, &parent, current_idx));
        }

        let current_pt = grid.point(current_idx);

        for d in &DIRS {
            let nb_pt = Point3D::new(current_pt.x + d.x, current_pt.y + d.y, current_pt.z + d.z);
            let Some(nb_idx) = grid.idx(nb_pt) else {
                continue;
            };

            let step = grid.cost_idx(nb_idx);
            if step == 0 {
                continue; // impassable
            }

            let tentative_g = g.saturating_add(step as u32);

            if tentative_g < g_score[nb_idx] {
                g_score[nb_idx] = tentative_g;
                parent[nb_idx] = current_idx as u32;

                let h = nb_pt.manhattan_distance(&goal);
                let f = tentative_g.saturating_add(h);
                heap.push(State {
                    f,
                    g: tentative_g,
                    idx: nb_idx,
                });
            }
        }
    }

    None
}

fn reconstruct_path(grid: &Grid, parent: &[u32], mut i: usize) -> Vec<Point3D> {
    let mut out = Vec::new();
    loop {
        out.push(grid.point(i));
        let p = parent[i];
        if p == u32::MAX {
            break;
        }
        i = p as usize;
    }
    out.reverse();
    out
}

// Path → Moves conversion with facing + digging
// Mapping:
// - up / down          => +Y / -Y
// - north / south      => -Z / +Z
// - west / east        => -X / +X
pub fn path_to_moves(grid: &Grid, path: &[Point3D]) -> Result<Vec<String>, String> {
    if path.len() <= 1 {
        return Ok(Vec::new());
    }
    let mut moves: Vec<String> = Vec::with_capacity(path.len() * 2);

    for w in path.windows(2) {
        let a = w[0];
        let b = w[1];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let dz = b.z - a.z;

        // Ensure axis-aligned, single-step moves
        let nonzero_axes = (dx != 0) as u8 + (dy != 0) as u8 + (dz != 0) as u8;
        if nonzero_axes != 1 || dx.abs() > 1 || dy.abs() > 1 || dz.abs() > 1 {
            return Err(format!(
                "Invalid step from {:?} to {:?} (dx={}, dy={}, dz={})",
                a, b, dx, dy, dz
            ));
        }

        // Determine direction words
        // lateral
        if dx == 1 || dx == -1 || dz == 1 || dz == -1 {
            let (dir_word, face_word) = if dx == 1 {
                ("east", "faceeast")
            } else if dx == -1 {
                ("west", "facewest")
            } else if dz == -1 {
                ("north", "facenorth")
            } else {
                // dz == 1
                ("south", "facesouth")
            };

            if let Some(cost) = grid.cost_point(b) {
                if cost > 1 {
                    moves.push(face_word.to_string()); // face first
                    moves.push("dig".to_string()); // then dig
                }
            }
            moves.push(dir_word.to_string()); // then move
        }
        // vertical
        else if dy == 1 || dy == -1 {
            let (move_word, dig_word) = if dy == 1 {
                ("up", "digup")
            } else {
                ("down", "digdown")
            };

            if let Some(cost) = grid.cost_point(b) {
                if cost > 1 {
                    moves.push(dig_word.to_string()); // digup/digdown first
                }
            }
            moves.push(move_word.to_string()); // then move
        }
    }

    Ok(moves)
}
