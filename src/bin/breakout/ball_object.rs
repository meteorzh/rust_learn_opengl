use std::time::Duration;

use cgmath::{Point2, Vector2, Vector3, Deg, Matrix4};
use glium::Surface;

use crate::{game_object::GameObject, ResourceManager, sprite_renderer::SpriteRenderer};



/// 游戏球
#[derive(Debug)]
pub struct BallObject {
    pub game_object: GameObject,
    pub radius: f32,
    pub stuck: bool,
    pub sticky: bool,
    pub pass_through: bool,
}

impl BallObject {

    pub fn new(position: Point2<f32>, velocity: Vector2<f32>, texture_key: String, radius: f32) -> Self {
        Self {
            game_object: GameObject {
                position,
                size: Vector2::new(radius * 2.0, radius * 2.0),
                velocity,
                color: Vector3::new(1.0, 1.0, 1.0),
                rotation: Deg(0.0),
                is_solid: true,
                destroyed: false,
                texture_key,
            },
            radius: radius,
            stuck: true,
            sticky: false,
            pass_through: false,
        }
    }

    pub fn move_ball(&mut self, window_width: u32, dt: Duration) -> Point2<f32> {
        let position = &mut self.game_object.position;
        if !self.stuck {
            let velocity = &mut self.game_object.velocity;
            let size = self.game_object.size;
            *position += *velocity * dt.as_secs_f32();
            // println!("{}, {}", position.x, position.y);
            // 检查是否在窗口边界外，是的话需要反转速度并恢复到正确的位置
            if position.x <= 0.0 {
                // 超出左边
                velocity.x = -velocity.x;
                position.x = 0.0;
            } else if (position.x + size.x) >= window_width as f32 {
                // 超出右边
                velocity.x = -velocity.x;
                position.x = window_width as f32 - size.x;
            }

            if position.y <= 0.0 {
                // 超出上边
                velocity.y = -velocity.y;
                position.y = 0.0;
            }
        }
        return *position;
    }

    pub fn reset(&mut self, position: Point2<f32>, velocity: Vector2<f32>) {
        self.game_object.position = position;
        self.game_object.velocity = velocity;
        
    }

    pub fn draw<T: Surface>(&self, renderer: &SpriteRenderer, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        let game_object = &self.game_object;
        let texture = resource_manager.get_texture(&game_object.texture_key);
        renderer.draw_sprite(surface, texture, game_object.position, game_object.size, game_object.rotation, game_object.color, projection);
    }
}
