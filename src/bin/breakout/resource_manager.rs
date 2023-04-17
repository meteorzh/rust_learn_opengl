use std::collections::HashMap;

use glium::{Display, texture::Texture2d};
use rust_opengl_learn::material;


/// 资源管理器
pub struct ResourceManager<'a> {
    
    textures: HashMap<&'a str, Texture2d>,
}

impl <'a> ResourceManager<'a> {

    pub fn new() -> Self {
        Self { textures: HashMap::new() }
    }

    pub fn load_texture(&mut self, key: &'a str, path: &str, display: &Display) {
        self.textures.insert(key, material::load_texture2(path.to_string(), display).1);
    }

    pub fn get_texture(&self, key: &str) -> &Texture2d {
        self.textures.get(key).unwrap()
    }
}