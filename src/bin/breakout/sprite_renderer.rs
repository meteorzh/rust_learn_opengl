use cgmath::{Point2, Vector2, Deg, Vector3, Matrix4};
use glium::{Program, VertexBuffer, IndexBuffer, DrawParameters, Display, index::PrimitiveType, Blend, BackfaceCullingMode, Surface, uniforms::UniformValue, texture::Texture2d};
use rust_opengl_learn::{objectsv2::RawVertexP2T, uniforms::DynamicUniforms};



pub struct SpriteRenderer<'a> {
    shader: Program,
    vertex_buffer: VertexBuffer<RawVertexP2T>,
    index_buffer: IndexBuffer<u16>,
    draw_parameters: DrawParameters<'a>,
}

impl <'a> SpriteRenderer<'a> {

    pub fn new(display: &Display, shader: Program) -> Self {
        Self { 
            shader: shader,
            vertex_buffer: VertexBuffer::new(display, &[
                RawVertexP2T { position: [0.0, 1.0], texture: [0.0, 1.0] },
                RawVertexP2T { position: [1.0, 0.0], texture: [1.0, 0.0] },
                RawVertexP2T { position: [0.0, 0.0], texture: [0.0, 0.0] },

                RawVertexP2T { position: [0.0, 1.0], texture: [0.0, 1.0] },
                RawVertexP2T { position: [1.0, 1.0], texture: [1.0, 1.0] },
                RawVertexP2T { position: [1.0, 0.0], texture: [1.0, 0.0] },
            ]).unwrap(),
            index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap(),
            draw_parameters: DrawParameters {
                blend: Blend::alpha_blending(),
                backface_culling: BackfaceCullingMode::CullClockwise,
                .. Default::default()
            },
        }
    }

    pub fn draw_sprite<S: Surface>(&self, surface: &mut S, texture: &Texture2d, position: Point2<f32>, size: Vector2<f32>, rotate: Deg<f32>, color: Vector3<f32>, projection: Matrix4<f32>) {
        let mut uniforms = DynamicUniforms::new();

        let model = Matrix4::from_translation(Vector3::new(position.x, position.y, 0.0));
        let model = model * Matrix4::from_translation(Vector3::new(0.5 * size.x, 0.5 * size.y, 0.0));
        let model = model * Matrix4::from_angle_z(rotate);
        let model = model * Matrix4::from_translation(Vector3::new(-0.5 * size.x, -0.5 * size.y, 0.0));
        let model = model * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);

        uniforms.add_str_key_value("projection", UniformValue::Mat4(projection.into()));
        uniforms.add_str_key_value("model", UniformValue::Mat4(model.into()));
        uniforms.add_str_key_value("spriteColor", UniformValue::Vec3(color.into()));
        uniforms.add_str_key("image", texture);

        surface.draw(&self.vertex_buffer, &self.index_buffer, &self.shader, &uniforms, &self.draw_parameters).unwrap();
    }
}