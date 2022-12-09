use cgmath::{Point3, Point2, Vector3};
use glium::implement_vertex;


#[derive(Copy, Clone)]
pub struct RawVertexP2 {
    pub position: [f32; 2],
}

implement_vertex!(RawVertexP2, position);

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


#[derive(Copy, Clone)]
pub struct RawInstanceOffsetO2 {
    pub offset: [f32; 2],
}

implement_vertex!(RawInstanceOffsetO2, offset);

// 可绘制特征，需要实现获取vertexBuffer, indexBuffer等
pub trait Drawable {
    
}

pub struct VertexV2 {
    pub position: Point3<f32>,
    pub texture: Option<Point2<f32>>,
    pub color: Option<Vector3<f32>>,
}