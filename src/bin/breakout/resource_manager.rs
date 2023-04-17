use std::{collections::HashMap, io::{BufReader, Cursor}, fs::{self}};

use glium::{Display, texture::Texture2d};
use rodio::{Decoder};
use rust_opengl_learn::material;


/// 资源管理器
pub struct ResourceManager<'a> {
    
    textures: HashMap<&'a str, Texture2d>,

    audios: HashMap<&'a str, Decoder<BufReader<Cursor<Vec<u8>>>>>,
}

impl <'a> ResourceManager<'a> {

    pub fn new() -> Self {
        Self { textures: HashMap::new(), audios: HashMap::new() }
    }

    pub fn load_texture(&mut self, key: &'a str, path: &str, display: &Display) {
        self.textures.insert(key, material::load_texture2(path.to_string(), display).1);
    }

    pub fn get_texture(&self, key: &str) -> &Texture2d {
        self.textures.get(key).unwrap()
    }

    pub fn load_audio(&mut self, key: &'a str, path: &str) {
        let source = load_audio(path);
        self.audios.insert(key, source);
    }
}

pub fn load_audio(path: &str) -> Decoder<BufReader<Cursor<Vec<u8>>>> {
    let file = Cursor::new(fs::read(path).unwrap());
    Decoder::new(BufReader::new(file)).unwrap()
}