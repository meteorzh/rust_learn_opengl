use cgmath::{Point3, Matrix4, Vector3, Transform, SquareMatrix, Point2};
use glium::{VertexBuffer, IndexBuffer, index::PrimitiveType};

use crate::{Vertex, objectsv2::RawVertexPNTTB};


static CUBE_INDEX_ARRAY: [u16; 36] = [0u16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35];


/**
 * 36个顶点的简单正方体
 */
pub struct Cube {
    id: String,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub index_buffer: IndexBuffer<u16>,
    pub color: [f32; 3],
    position: Point3<f32>,
    pub model: Matrix4<f32>,
}

impl Cube {
    /**
     * 边长，0-1，标准化设备坐标系范围内
     * 顶点顺序均为逆时针
     */
    pub fn new(id: &str, side_len: f32, display: &glium::Display, color: [f32; 3], position: Point3<f32>, model: Matrix4<f32>) -> Cube {
        let half = side_len / 2_f32;
        Cube {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // 前
                Vertex { position: [-half, half, half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 1.0] },
                // 后
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 1.0] },
                // 左
                Vertex { position: [-half, half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, -half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                // 右
                Vertex { position: [half, half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                // 上
                Vertex { position: [-half, half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                // 下
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 0.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &CUBE_INDEX_ARRAY).unwrap(),
            color: color,
            position: position,
            model: model,
        }
    }

    // 创建天空盒box
    pub fn new_skybox(id: &str, side_len: f32, display: &glium::Display) -> Cube {
        let half = side_len / 2_f32;
        Cube {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // 前
                Vertex { position: [-half, half, -half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 1.0] },
                // 后
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 1.0] },
                // 左
                Vertex { position: [-half, half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, -half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                // 右
                Vertex { position: [half, half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                // 上
                Vertex { position: [-half, half, half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, half, half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 0.0] },
                // 下
                Vertex { position: [-half, -half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 0.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 1.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &CUBE_INDEX_ARRAY).unwrap(),
            color: [0.0, 0.0, 0.0],
            position: Point3::new(0.0, 0.0, 0.0),
            model: Matrix4::identity(),
        }
    }

    pub fn position_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, self.position.z))
    }

    pub fn calc_model_with(&self, other: Matrix4<f32>) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, self.position.z)) * other * self.model
    }

    pub fn calc_model(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, self.position.z)) * self.model
    }
}



// 矩形平面
pub struct Plane {
    id: String,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub index_buffer: IndexBuffer<u16>,
    pub position: Point3<f32>,
    model: Matrix4<f32>,
}


impl Plane {

    // y: 4个顶点的共同y值
    pub fn new(id: &str, length: f32, width: f32, y: f32, display: &glium::Display, position: Point3<f32>, model: Matrix4<f32>) -> Plane {
        let x = length / 2.0_f32;
        let z = width / 2.0_f32;
        Plane {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // texture Coords (note we set these higher than 1 (together with GL_REPEAT as texture wrapping mode). this will cause the floor texture to repeat)
                Vertex { position: [-x, y, z], normal: [0_f32, 1.0, 0.0], texture: [0.0_f32, 0.0] },
                Vertex { position: [-x, y, -z], normal: [0_f32, 1.0, 0.0], texture: [0.0_f32, 2.0] },
                Vertex { position: [x, y, -z], normal: [0_f32, 1.0, 0.0], texture: [2.0_f32, 2.0] },

                Vertex { position: [-x, y, z], normal: [0_f32, 1.0, 0.0], texture: [0.0_f32, 0.0] },
                Vertex { position: [x, y, -z], normal: [0_f32, 1.0, 0.0], texture: [2.0_f32, 2.0] },
                Vertex { position: [x, y, z], normal: [0_f32, 1.0, 0.0], texture: [2.0_f32, 0.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap(),
            position: position,
            model: model,
        }
    }

    pub fn calc_model_with(&self, other: Matrix4<f32>) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, self.position.z)) * other * self.model
    }

    pub fn calc_model(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, self.position.z)) * self.model
    }

    pub fn new_vertical_plane(id: &str, height: f32, width: f32, display: &glium::Display, position: Point3<f32>, model: Matrix4<f32>) -> Plane {
        let x = width / 2.0_f32;
        Plane {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // texture Coords (note we set these higher than 1 (together with GL_REPEAT as texture wrapping mode). this will cause the floor texture to repeat)
                Vertex { position: [-x, 0.0, 0.0], normal: [0_f32, 0.0, 1.0], texture: [0.0_f32, 0.0] },
                Vertex { position: [x, 0.0, 0.0], normal: [0_f32, 0.0, 1.0], texture: [1.0_f32, 0.0] },
                Vertex { position: [-x, height, 0.0], normal: [0_f32, 0.0, 1.0], texture: [0.0_f32, 1.0] },
                Vertex { position: [x, height, 0.0], normal: [0_f32, 0.0, 1.0], texture: [1.0_f32, 1.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 1, 2, 3]).unwrap(),
            position: position,
            model: model,
        }
    }

    pub fn new_vertical_center_plane(id: &str, height: f32, width: f32, display: &glium::Display, primitive_type: PrimitiveType) -> Plane {
        let x = width / 2.0_f32;
        let y = height / 2.0_f32;
        Plane {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // texture Coords (note we set these higher than 1 (together with GL_REPEAT as texture wrapping mode). this will cause the floor texture to repeat)
                Vertex { position: [-x, y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [0.0_f32, 1.0] },
                Vertex { position: [-x, -y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [0.0_f32, 0.0] },
                Vertex { position: [x, -y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [1.0_f32, 0.0] },
                Vertex { position: [x, y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [1.0_f32, 1.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, primitive_type, &[0u16, 1, 2, 0, 2, 3]).unwrap(),
            position: Point3::new(0.0, 0.0, 0.0),
            model: Matrix4::identity(),
        }
    }

    // 以原点为中心创建2d平面
    pub fn new_2d_plane(id: &str, height: f32, width: f32, display: &glium::Display) -> Plane {
        let x = width / 2.0_f32;
        let y = height / 2.0_f32;
        Plane {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                Vertex { position: [-x, y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [0.0_f32, 1.0] },
                Vertex { position: [-x, -y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [0.0_f32, 0.0] },
                Vertex { position: [x, -y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [1.0_f32, 0.0] },
                Vertex { position: [-x, y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [0.0_f32, 1.0] },
                Vertex { position: [x, -y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [1.0_f32, 0.0] },
                Vertex { position: [x, y, 0.0], normal: [0_f32, 0.0, 1.0], texture: [1.0_f32, 1.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap(),
            position: Point3 { x: 0.0_f32, y: 0.0, z: 0.0 },
            model: Matrix4::identity(),
        }
    }
}


// 矩形平面
pub struct PlaneV2 {
    pub vertex_buffer: VertexBuffer<RawVertexPNTTB>,
    pub index_buffer: IndexBuffer<u16>,
}

impl PlaneV2 {

    pub fn new_vertical(width: f32, height: f32, display: &glium::Display) -> PlaneV2 {
        let x = width / 2.0;
        let y = height / 2.0;

        let point1 = Point3::new(-x, y, 0.0);
        let point2 = Point3::new(-x, -y, 0.0);
        let point3 = Point3::new(x, -y, 0.0);
        let point4 = Point3::new(x, y, 0.0);

        let tex_coords1 = Point2::<f32>::new(0.0, 1.0);
        let tex_coords2 = Point2::<f32>::new(0.0, 0.0);
        let tex_coords3 = Point2::<f32>::new(1.0, 0.0);
        let tex_coords4 = Point2::<f32>::new(1.0, 1.0);

        let normal = Vector3::<f32>::new(0.0, 0.0, 1.0);

        let mut edge1 = point2 - point1;
        let mut edge2 = point3 - point1;
        let mut delta_uv1 = tex_coords2 - tex_coords1;
        let mut delta_uv2 = tex_coords3 - tex_coords1;

        let mut f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tangent1 = Vector3::<f32>::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        );
        let bitangent1 = Vector3::<f32>::new(
            f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
            f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
            f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z)
        );

        edge1 = point3 - point1;
        edge2 = point4 - point1;
        delta_uv1 = tex_coords3 - tex_coords1;
        delta_uv2 = tex_coords4 - tex_coords1;

        f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tangent2 = Vector3::<f32>::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        );
        let bitangent2 = Vector3::<f32>::new(
            f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
            f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
            f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z),
        );

        PlaneV2 {
            vertex_buffer: glium::VertexBuffer::new(display, &[
                RawVertexPNTTB { position: point1.into(), normal: normal.into(), tex_coords: tex_coords1.into(), tangent: tangent1.into(), bitangent: bitangent1.into() },
                RawVertexPNTTB { position: point2.into(), normal: normal.into(), tex_coords: tex_coords2.into(), tangent: tangent1.into(), bitangent: bitangent1.into() },
                RawVertexPNTTB { position: point3.into(), normal: normal.into(), tex_coords: tex_coords3.into(), tangent: tangent1.into(), bitangent: bitangent1.into() },

                RawVertexPNTTB { position: point1.into(), normal: normal.into(), tex_coords: tex_coords1.into(), tangent: tangent2.into(), bitangent: bitangent2.into() },
                RawVertexPNTTB { position: point3.into(), normal: normal.into(), tex_coords: tex_coords3.into(), tangent: tangent2.into(), bitangent: bitangent2.into() },
                RawVertexPNTTB { position: point4.into(), normal: normal.into(), tex_coords: tex_coords4.into(), tangent: tangent2.into(), bitangent: bitangent2.into() },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap(),
        }
    }
}