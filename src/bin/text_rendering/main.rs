
extern crate glium;
extern crate cgmath;

use std::{borrow::Cow};

use cgmath::{Vector3, Vector2};
use freetype::{Library, face::LoadFlag};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue}, texture::{Texture2d, RawImage2d, ClientFormat}, index::PrimitiveType, Frame, Program, VertexBuffer, Display, IndexBuffer, DrawParameters, Blend};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, create_program, start_loop, Action, context::{LoopContext}, objectsv2::RawVertexP2T};

struct Character {
    texture: Texture2d,
    size: Vector2<i32>,
    bearing: Vector2<i32>,
    advance: u32,
}

/// Text Rendering
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let text_program = create_program("src/bin/text_rendering/text.vert", "src/bin/text_rendering/text.frag", &display);

    // init font
    let characters = {
        let library = Library::init().unwrap();
        let face = library.new_face("src/fonts/Antonio-Bold.ttf", 0).unwrap();
        face.set_pixel_sizes(0, 48).unwrap();
        // face.set_char_size(0, 48, 50, 0).unwrap();
        let mut list = Vec::with_capacity(128);
        for c in 0..128 {
            face.load_char(c as usize, LoadFlag::RENDER).unwrap();
    
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            let buffer_data = bitmap.buffer();
            // println!("len: {}, width: {}, height: {}, bitmap_left: {}, bitmap_top: {}", buffer_data.len(), bitmap.width(), bitmap.rows(), glyph.bitmap_left(), glyph.bitmap_top());
            let raw_image = RawImage2d {
                data: Cow::Owned(buffer_data.to_vec()),
                width: bitmap.width() as u32,
                height: bitmap.rows() as u32,
                format: ClientFormat::U8,
            };
            let texture = Texture2d::new(&display, raw_image).unwrap();

            list.push(Character {
                texture,
                size: Vector2::new(bitmap.width(), bitmap.rows()),
                bearing: Vector2::new(glyph.bitmap_left(), glyph.bitmap_top()),
                advance: glyph.advance().x as u32,
            });
        }
        list
    };

    // 摄像机初始位置
    let camera = Camera::new(
        cgmath::Point3::new(0.0, 0.0, 3.0),
        cgmath::Rad::from(cgmath::Deg(-90.0)),
        cgmath::Rad::from(cgmath::Deg(0.0))
    );
    let controller = CameraController::new(1.0, 0.5);

    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::ortho(0.0, size.width as f32, 0.0, size.height as f32, 0.0, 10.0));

    let loop_context = LoopContext::new(camera, controller);

    start_loop(event_loop, loop_context, move |_: Option<Event<()>>, ctx| {
        // 创建默认帧
        let mut target = display.draw();
        target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

        // render_text(&mut target, &text_program, &display, "T", 25.0, 25.0, 1.0, Vector3::new(0.5, 0.8, 0.2), &characters, &projection_matrix);
        render_text(&mut target, &text_program, &display, "This is sample text", 25.0, 25.0, 1.0, Vector3::new(0.5, 0.8, 0.2), &characters, &projection_matrix);
        render_text(&mut target, &text_program, &display, "(C) LearnOpenGL.com", 540.0, 570.0, 0.5, Vector3::new(0.3, 0.7, 0.8), &characters, &projection_matrix);

        target.finish().unwrap();

        Action::Continue
    });
}

fn render_text(frame: &mut Frame, program: &Program, display: &Display, text: &str, x: f32, y: f32, scale: f32, color: Vector3<f32>, chars: &Vec<Character>, projection: &[[f32; 4]; 4]) {
    let index_buffer = IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap();
    let mut uniforms = DynamicUniforms::new();
    uniforms.add_str_key_value("textColor", UniformValue::Vec3(color.into()));
    uniforms.add_str_key("projection", projection);
    let mut x = x;
    for c in text.chars() {
        let character = chars.get(c as usize).unwrap();
        let xpos = x + (character.bearing.x) as f32 * scale;
        let ypos = y - (character.size.y as f32 - character.bearing.y as f32) * scale;
        let w = character.size.x as f32 * scale;
        let h = character.size.y as f32 * scale;

        let vertexes = vec![
            RawVertexP2T { position: [xpos, ypos + h], texture: [0.0, 0.0] },
            RawVertexP2T { position: [xpos, ypos], texture: [0.0, 1.0] },
            RawVertexP2T { position: [xpos + w, ypos], texture: [1.0, 1.0] },
            RawVertexP2T { position: [xpos, ypos + h], texture: [0.0, 0.0] },
            RawVertexP2T { position: [xpos + w, ypos], texture: [1.0, 1.0] },
            RawVertexP2T { position: [xpos + w, ypos + h], texture: [1.0, 0.0] },
        ];

        uniforms.add_str_key("text", &character.texture);

        let vertex_buffer = VertexBuffer::new(display, vertexes.as_slice()).unwrap();

        frame.draw(&vertex_buffer, &index_buffer, program, &uniforms, &DrawParameters {
            blend: Blend::alpha_blending(),
            .. Default::default()
        }).unwrap();

        x += (character.advance >> 6) as f32 * scale;
    }
}
