use std::{fs::File, io::BufReader};

use crate::pathfinder::{Grid, Point3D, astar_find_path};
use bincode::{Decode, Encode, config};

use std::io::ErrorKind;
use std::path::Path;

#[derive(Encode, Decode, PartialEq, Debug)]
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
    fn apply_padding(&self, value: i32, padding: u32) -> i32 {
        if value > 0 {
            value + padding as i32
        } else {
            value - padding as i32
        }
    }
    pub fn get_path(
        &self,
        mut start: Point3D,
        mut end: Point3D,
        padding: u32,
        can_dig: bool,
    ) -> Option<Vec<Point3D>> {
        start.x = self.apply_padding(start.x, padding);
        start.y = self.apply_padding(start.y, padding);
        start.z = self.apply_padding(start.z, padding);
        end.x = self.apply_padding(end.x, padding);
        end.y = self.apply_padding(end.y, padding);
        end.z = self.apply_padding(end.z, padding);
        if end.y < -60 {
            end.y = -60;
        } else if end.y > 318 {
            end.y = 318;
        }

        let mut grid = Grid::new(start, end, 1);
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

struct Turtle {
    position: Point3D,
    id: u32,
    facing: u8, // 0: north, 1: south, 2: west, 3: east
    name: String,
}
impl Turtle {
    pub fn new(position: Point3D, id: u32, facing: u8, name: String) -> Self {
        Turtle {
            position,
            id,
            facing,
            name,
        }
    }
}

struct Turtles {
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
