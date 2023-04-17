use cgmath::{Point2, Vector2, Vector3, Deg, Matrix4};
use glium::Surface;

use crate::{ResourceManager, sprite_renderer::SpriteRenderer};

/// 游戏物体对象
#[derive(Debug)]
pub struct GameObject {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub color: Vector3<f32>,
    pub rotation: Deg<f32>,
    pub is_solid: bool,
    pub destroyed: bool,
    pub texture_key: String,
}

impl GameObject {
    
    pub fn draw<T: Surface>(&self, renderer: &SpriteRenderer, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        let texture = resource_manager.get_texture(self.texture_key.as_str());
        renderer.draw_sprite(surface, texture, self.position, self.size, self.rotation, self.color, projection);
    }
}