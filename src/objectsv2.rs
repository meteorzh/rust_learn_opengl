use cgmath::{Point3, Point2, Vector3};
use glium::{implement_vertex, VertexBuffer, IndexBuffer, Display, Frame};


#[derive(Copy, Clone)]
pub struct RawVertexP2 {
    pub position: [f32; 2],
}

implement_vertex!(RawVertexP2, position);

#[derive(Copy, Clone)]
pub struct RawVertexP2T {
    pub position: [f32; 2],
    pub texture: [f32; 2],
}

implement_vertex!(RawVertexP2T, position, texture);

#[derive(Copy, Clone)]
pub struct RawVertexP {
    pub position: [f32; 3],
}

implement_vertex!(RawVertexP, position);

#[derive(Copy, Clone)]
pub struct RawVertexPT {
    pub position: [f32; 3],
    pub texture: [f32; 2],
}

implement_vertex!(RawVertexPT, position, texture);

#[derive(Copy, Clone)]
pub struct RawVertexPC {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

implement_vertex!(RawVertexPC, position, color);

#[derive(Copy, Clone)]
pub struct RawVertexP2C {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

implement_vertex!(RawVertexP2C, position, color);


/// 顶点：位置，法向量，贴图坐标，切线向量，副切线向量
#[derive(Copy, Clone)]
pub struct RawVertexPNTTB {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

implement_vertex!(RawVertexPNTTB, position, normal, tex_coords, tangent, bitangent);


#[derive(Copy, Clone)]
pub struct RawInstanceOffsetO2 {
    pub offset: [f32; 2],
}

implement_vertex!(RawInstanceOffsetO2, offset);

#[derive(Copy, Clone)]
pub struct RawInstanceDataM4 {
    pub model: [[f32; 4]; 4],
}

implement_vertex!(RawInstanceDataM4, model);

// 可绘制特征，需要实现获取vertexBuffer, indexBuffer等
pub trait Drawable {
    fn draw(frame: &mut Frame);
}

pub struct VertexV2 {
    pub position: Point3<f32>,
    pub texture: Option<Point2<f32>>,
    pub color: Option<Vector3<f32>>,
}


/// idea: CharacterRepo restore textureCharacter
/// Text struct represent a renderable text, and can change char at specify position.
pub struct Text {
    // pub vertex_buffer: VertexBuffer<RawVertexP2T>,
    // pub index_buffer: IndexBuffer<u32>,
}

impl Text {

    pub fn new(display: &Display, text: &str, x: f32, ) -> Self {
        
        Text {
            // vertex_buffer: VertexBuffer::empty_dynamic(display, vertex_number).unwrap(),
            // index_buffer: ()
        }
    }
}