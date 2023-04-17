use std::fs;

use cgmath::{Matrix4, Point2, Vector2, Vector3, Deg};
use glium::Surface;

use crate::{game_object::GameObject, sprite_renderer::SpriteRenderer, ResourceManager};



/// 无砖块
const EMPTY: u8 = 0;
/// 坚硬的砖块，不可摧毁
const HARD_BRICK: u8 = 1;



/// 游戏关卡
pub struct GameLevel {
    pub bricks: Vec<GameObject>,
    destroyed_count: u32,
    destroyable_count: u32,
}

impl GameLevel {

    pub fn new(file: &str, level_width: u32, level_height: u32) -> Self {
        let level_str_data = fs::read_to_string(file).unwrap();
        let lines = level_str_data.lines();
        let mut tile_data: Vec<Vec<u8>> = Vec::new();
        for line in lines {
            let chars = line.split(" ");
            let mut rows = Vec::<u8>::new();
            for char in chars {
                rows.push(str::parse::<u8>(char).unwrap());
            }
            tile_data.push(rows);
        }

        let mut level = GameLevel {
            bricks: Vec::new(),
            destroyed_count: 0,
            destroyable_count: 0,
        };
        if tile_data.len() > 0 {
            level.init(tile_data, level_width, level_height);
        }

        level
    }

    pub fn reset(&mut self) {
        // 重置关卡
        self.destroyed_count = 0;
        self.destroyable_count = self.bricks.len() as u32;
        for brick in self.bricks.iter_mut() {
            brick.destroyed = false;
        }
    }

    // 渲染关卡
    pub fn draw<T: Surface>(&self, renderer: &SpriteRenderer, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        // 渲染所有未被破坏的砖块
        for brick in self.bricks.iter() {
            if !brick.destroyed {
                brick.draw(renderer, surface, resource_manager, projection);
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.destroyed_count == self.destroyable_count
    }

    fn init(&mut self, tile_data: Vec<Vec<u8>>, level_width: u32, level_height: u32) {
        // 计算每个砖块的大小
        let height = tile_data.len();
        let width = tile_data.get(0).unwrap().len();
        let unit_width = level_width as f32 / width as f32;
        let unit_height = level_height as f32 / height as f32;

        for (y, row) in tile_data.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                // 区分砖块类型
                if *col == HARD_BRICK {
                    // 坚硬的砖块
                    self.bricks.push(GameObject {
                        position: Point2::new(unit_width * x as f32, unit_height * y as f32),
                        size: Vector2::new(unit_width, unit_height),
                        velocity: Vector2::new(0.0, 0.0),
                        color: Vector3::new(0.8, 0.8, 0.7),
                        rotation: Deg(0.0),
                        is_solid: true,
                        destroyed: false,
                        texture_key: "block_solid".to_string(),
                    });
                } else if *col > 1 {
                    self.destroyable_count += 1;
                    let color = {
                        if *col == 2 {
                            Vector3::new(0.2, 0.6, 1.0)
                        } else if *col == 3 {
                            Vector3::new(0.0, 0.7, 0.0)
                        } else if *col == 4 {
                            Vector3::new(0.8, 0.8, 0.4)
                        } else if *col == 5 {
                            Vector3::new(1.0, 0.5, 0.0)
                        } else {
                            Vector3::new(1.0, 1.0, 1.0)
                        }
                    };
                    // 可被摧毁的砖块
                    self.bricks.push(GameObject {
                        position: Point2::new(unit_width * x as f32, unit_height * y as f32),
                        size: Vector2::new(unit_width, unit_height),
                        velocity: Vector2::new(0.0, 0.0),
                        color,
                        rotation: Deg(0.0),
                        is_solid: false,
                        destroyed: false,
                        texture_key: "block".to_string(),
                    });
                }
            }
        }
    }
}