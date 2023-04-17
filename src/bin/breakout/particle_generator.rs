use std::{collections::VecDeque, time::Duration};

use cgmath::{Point2, Vector2, Vector4, Matrix4};
use glium::{Program, VertexBuffer, IndexBuffer, DrawParameters, Display, index::PrimitiveType, Blend, BlendingFunction, LinearBlendingFactor, BackfaceCullingMode, Surface, uniforms::UniformValue};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rust_opengl_learn::{objectsv2::RawVertexP2T, uniforms::DynamicUniforms};

use crate::{game_object::GameObject, ResourceManager};


struct Particle {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    color: Vector4<f32>,
    life: f32,
}

impl Particle {
    pub fn new(position: Point2<f32>, velocity: Vector2<f32>, color: Vector4<f32>, life: f32) -> Self {
        Particle { position, velocity, color, life }
    }
}


pub struct ParticleGenerator<'a> {
    shader: Program,
    particles: Vec<Particle>,
    amount: u32,
    texture_key: String,
    vertex_buffer: VertexBuffer<RawVertexP2T>,
    index_buffer: IndexBuffer<u16>,
    draw_parameters: DrawParameters<'a>,
    deactive: VecDeque<usize>,
    rng: StdRng,
}

impl <'a> ParticleGenerator<'a> {

    pub fn new(display: &Display, shader: Program, texture_key: String, amount: u32) -> Self {
        let (particles, deactive) = {
            let mut vec = Vec::with_capacity(amount as usize);
            let mut deactive = VecDeque::with_capacity(amount as usize);
            for i in 0..amount {
                vec.push(Particle::new(Point2::new(0.0, 0.0), Vector2::new(0.0, 0.0), Vector4::new(1.0, 1.0, 1.0, 1.0), 0.0));
                deactive.push_back(i as usize);
            }
            (vec, deactive)
        };

        ParticleGenerator {
            shader,
            particles,
            amount,
            texture_key,
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
                blend: Blend {
                    color: BlendingFunction::Addition {
                        source: LinearBlendingFactor::SourceAlpha,
                        destination: LinearBlendingFactor::One,
                    },
                    alpha: BlendingFunction::Addition {
                        source: LinearBlendingFactor::SourceAlpha,
                        destination: LinearBlendingFactor::One
                    },
                    constant_value: (0.0, 0.0, 0.0, 0.0)
                },
                backface_culling: BackfaceCullingMode::CullClockwise,
                .. Default::default()
            },
            deactive,
            rng: StdRng::seed_from_u64(0),
        }
    }

    pub fn update(&mut self, object: &GameObject, new_particles: u32, offset: Vector2<f32>, dt: Duration) {
        let dt = dt.as_secs_f32();
        
        for _ in 0..new_particles {
            // 产生新的粒子
            if let Some(index) = self.deactive.pop_front() {
                let particle = self.particles.get_mut(index).unwrap();
                let rng = &mut self.rng;
                respawn_particle(rng, particle, object, offset);
            }
        }

        // 更新所有粒子
        for (i, particle) in self.particles.iter_mut().enumerate() {
            if particle.life > 0.0 {
                particle.life -= dt;
                particle.position -= particle.velocity * dt;
                particle.color.w -= dt * 2.5;

                if particle.life <= 0.0 {
                    self.deactive.push_back(i);
                }
            }
        }
    }

    pub fn draw<T: Surface>(&self, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        let texture = resource_manager.get_texture(&self.texture_key);

        let mut uniforms = DynamicUniforms::new();

        uniforms.add_str_key_value("projection", UniformValue::Mat4(projection.into()));
        uniforms.add_str_key("sprite", texture);

        for particle in self.particles.iter() {
            if particle.life > 0.0 {
                uniforms.add_str_key_value("offset", UniformValue::Vec2(particle.position.into()));
                uniforms.add_str_key_value("color", UniformValue::Vec4(particle.color.into()));
                surface.draw(&self.vertex_buffer, &self.index_buffer, &self.shader, &uniforms, &self.draw_parameters).unwrap();
            }
        }
    }
}

fn respawn_particle(rng: &mut StdRng, particle: &mut Particle, object: &GameObject, offset: Vector2<f32>) {
    let random = (rng.gen_range(0..100) - 50) as f32 / 10.0;
    let color = 0.5 + (rng.gen_range(0..100) as f32 / 100.0);
    particle.position = object.position + offset + Vector2::new(random, random);
    particle.color = Vector4::new(color, color, color, 1.0);
    particle.life = 1.0;
    particle.velocity = object.velocity * 0.1;
}