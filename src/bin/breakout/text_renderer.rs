use std::{collections::HashMap, borrow::Cow};

use cgmath::{Vector2, Matrix4, Vector3};
use freetype::{Library, face::LoadFlag};
use glium::{texture::{Texture2d, RawImage2d, ClientFormat}, Program, Display, Surface, uniforms::UniformValue, IndexBuffer, index::PrimitiveType, VertexBuffer, Blend, DrawParameters};
use rust_opengl_learn::{uniforms::DynamicUniforms, objectsv2::RawVertexP2T};



struct Character {
    texture: Texture2d,
    size: Vector2<i32>,
    bearing: Vector2<i32>,
    advance: u32,
}

pub struct TextRenderer {
    map: HashMap<char, Character>,
    program: Program,
    projection: Matrix4<f32>,
}

impl TextRenderer {

    pub fn new(text_program: Program, width: u32, height: u32) -> Self {
        Self {
            map: HashMap::new(),
            program: text_program,
            projection: cgmath::ortho(0.0, width as f32, height as f32, 0.0, 0.0, 10.0),
        }
    }

    pub fn load(&mut self, display: &Display, font_path: &str, font_size: u32) {
        self.map.clear();

        let library = Library::init().unwrap();
        let face = library.new_face(font_path, 0).unwrap();
        face.set_pixel_sizes(0, font_size).unwrap();
        // face.set_char_size(0, 48, 50, 0).unwrap();
        for c in 0..128 {
            face.load_char(c as usize, LoadFlag::RENDER).unwrap();
    
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            let buffer_data = bitmap.buffer();
            
            let raw_image = RawImage2d {
                data: Cow::Owned(buffer_data.to_vec()),
                width: bitmap.width() as u32,
                height: bitmap.rows() as u32,
                format: ClientFormat::U8,
            };
            let texture = Texture2d::new(display, raw_image).unwrap();

            self.map.insert(char::from_u32(c as u32).unwrap(), Character {
                texture,
                size: Vector2::new(bitmap.width(), bitmap.rows()),
                bearing: Vector2::new(glyph.bitmap_left(), glyph.bitmap_top()),
                advance: glyph.advance().x as u32,
            });
        }
    }

    pub fn render_text<S: Surface>(&self, frame: &mut S, display: &Display, text: &str, x: f32, y: f32, scale: f32, color: Vector3<f32>) {
        let index_buffer = IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap();
        let mut uniforms = DynamicUniforms::new();
        uniforms.add_str_key_value("textColor", UniformValue::Vec3(color.into()));
        uniforms.add_str_key_value("projection", UniformValue::Mat4(self.projection.into()));

        let char_h = self.map.get(&'H').unwrap();
        let mut x = x;
        for c in text.chars() {
            let character = self.map.get(&c).unwrap();
            let xpos = x + (character.bearing.x) as f32 * scale;
            let ypos = y + (char_h.bearing.y as f32 - character.bearing.y as f32) * scale;
            let w = character.size.x as f32 * scale;
            let h = character.size.y as f32 * scale;
    
            let vertexes = vec![
                RawVertexP2T { position: [xpos, ypos + h], texture: [0.0, 1.0] },
                RawVertexP2T { position: [xpos + w, ypos], texture: [1.0, 0.0] },
                RawVertexP2T { position: [xpos, ypos], texture: [0.0, 0.0] },
                RawVertexP2T { position: [xpos, ypos + h], texture: [0.0, 1.0] },
                RawVertexP2T { position: [xpos + w, ypos + h], texture: [1.0, 1.0] },
                RawVertexP2T { position: [xpos + w, ypos], texture: [1.0, 0.0] },
            ];
    
            uniforms.add_str_key("text", &character.texture);
    
            let vertex_buffer = VertexBuffer::new(display, vertexes.as_slice()).unwrap();
    
            frame.draw(&vertex_buffer, &index_buffer, &self.program, &uniforms, &DrawParameters {
                blend: Blend::alpha_blending(),
                .. Default::default()
            }).unwrap();
    
            x += (character.advance >> 6) as f32 * scale;
        }
    }
}