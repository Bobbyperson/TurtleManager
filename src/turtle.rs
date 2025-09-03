use std::time::Instant;
use std::{fs::File, io::BufReader};

use crate::pathfinder::{Grid, Point3D, astar_find_path};
use bincode::{Decode, Encode, config};
use serde::Deserialize;

use std::io::ErrorKind;
use std::path::Path;

#[derive(Encode, Decode, PartialEq, Debug, Deserialize)]
pub struct Block {
    position: Point3D,
    block_type: String,
}
impl Block {
    pub fn new(position: Point3D, block_type: String) -> Self {
        Block {
            position,
            block_type,
        }
    }

    pub fn is_solid(&self) -> bool {
        self.block_type != "minecraft:air"
    }
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct World {
    blocks: Vec<Block>,
}
impl World {
    pub fn new() -> Self {
        World { blocks: Vec::new() }
    }
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
    pub fn get_block(&self, position: Point3D) -> Option<&Block> {
        self.blocks.iter().find(|b| b.position == position)
    }
    pub fn get_block_mut(&mut self, position: Point3D) -> Option<&mut Block> {
        self.blocks.iter_mut().find(|b| b.position == position)
    }
    pub fn set_block(&mut self, block: Block) {
        if let Some(existing) = self.get_block_mut(block.position) {
            *existing = block;
        } else {
            self.add_block(block);
        }
    }
    pub fn get_path(
        &self,
        start: Point3D,
        mut end: Point3D,
        padding: u32,
        can_dig: bool,
    ) -> Option<Vec<Point3D>> {
        if end.y < -60 {
            end.y = -60;
        } else if end.y > 318 {
            end.y = 318;
        }
        println!("Finding path from {:?} to {:?}", start, end);

        let mut min = Point3D::new(start.x.min(end.x), start.y.min(end.y), start.z.min(end.z));
        let mut max = Point3D::new(start.x.max(end.x), start.y.max(end.y), start.z.max(end.z));

        min.x -= padding as i32;
        min.y -= padding as i32;
        min.z -= padding as i32;
        max.x += padding as i32;
        max.y += padding as i32;
        max.z += padding as i32;

        println!("Using grid from {:?} to {:?}", min, max);

        let mut grid = Grid::new(min, max, 1);
        grid.set_cost(start, 1);
        for block in &self.blocks {
            if block.is_solid() {
                if can_dig && block.block_type != "minecraft:bedrock" {
                    grid.set_cost(block.position, 2);
                } else {
                    grid.set_cost(block.position, 0);
                }
            }
        }
        astar_find_path(&grid, start, end)
    }
    pub fn load_world<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match File::open(&path) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let cfg = config::standard();
                let loaded: World = bincode::decode_from_std_read(&mut reader, cfg)?;
                *self = loaded;
                Ok(())
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                *self = World::new();
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }
    pub fn save_world(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        let cfg = config::standard();
        bincode::encode_into_std_write(self, &mut file, cfg)?;
        Ok(())
    }
}

pub struct Turtle {
    position: Point3D,
    id: u32,
    facing: u8, // 0: north, 1: south, 2: west, 3: east
    name: String,
    status: String,
    last_heartbeat: Instant,
}
impl Turtle {
    pub fn new(position: Point3D, id: u32, facing: u8, name: String, status: String) -> Self {
        Turtle {
            position,
            id,
            facing,
            name,
            status,
            last_heartbeat: Instant::now(),
        }
    }
}

pub struct Turtles {
    turtles: Vec<Turtle>,
}
impl Turtles {
    pub fn new() -> Self {
        Turtles {
            turtles: Vec::new(),
        }
    }

    pub fn add_turtle(&mut self, turtle: Turtle) {
        self.turtles.push(turtle);
    }

    pub fn get_turtle(&self, id: u32) -> Option<&Turtle> {
        self.turtles.iter().find(|t| t.id == id)
    }
}
