use std::time::Duration;

use glium::{Program, framebuffer::{SimpleFrameBuffer, RenderBuffer}, Display, texture::{UncompressedFloatFormat, Texture2d}, uniforms::{UniformValue, MagnifySamplerFilter}, VertexBuffer, IndexBuffer, index::PrimitiveType, Surface, Rect, BlitMask, BlitTarget, DrawParameters};
use ouroboros::self_referencing;
use rust_opengl_learn::{uniforms::DynamicUniforms, objects::Plane, objectsv2::RawVertexP2T};

struct Dt {
    texture: Texture2d,
    ms_renderbuffer: RenderBuffer,
}

struct Holder<'a> {
    ms_framebuffer: SimpleFrameBuffer<'a>,
    framebuffer: SimpleFrameBuffer<'a>,
    base_uniforms: DynamicUniforms<'a>,
}

#[self_referencing]
struct Actual {
    dt: Dt,

    #[borrows(dt)]
    #[covariant]
    holder: Holder<'this>,
}

pub struct PostProcessor {
    program: Program,
    actual: Actual,
    confuse: bool,
    chaos: bool,
    shake: bool,
    shake_time: f32,
    width: u32,
    height: u32,
    rect: Rect,
    blit_target: BlitTarget,
    vertex_buffer: VertexBuffer<RawVertexP2T>,
    index_buffer: IndexBuffer<u16>,
}

impl PostProcessor {
    
    pub fn new(display: &Display, program: Program, width: u32, height: u32) -> Self {
        let actual = ActualBuilder {
            dt: Dt {
                texture: Texture2d::empty(display, width, height).unwrap(),
                ms_renderbuffer: RenderBuffer::new_multisample(display, UncompressedFloatFormat::U8U8U8, width, height, 4).unwrap(),
            },
            holder_builder: |dt| {
                let mut base_uniforms = DynamicUniforms::new();
                base_uniforms.add_str_key("scene", &dt.texture);

                let offset = (1.0 / 300.0) as f32;
                let offsets = [
                    [ -offset, offset  ],  // top-left
                    [  0.0,    offset  ],  // top-center
                    [  offset, offset  ],  // top-right
                    [ -offset, 0.0     ],  // center-left
                    [  0.0,    0.0     ],  // center-center
                    [  offset, 0.0     ],  // center - right
                    [ -offset, -offset ],  // bottom-left
                    [  0.0,    -offset ],  // bottom-center
                    [  offset, -offset ]   // bottom-right    
                ];
                for i in 0..9 {
                    let key = format!("offsets[{}]", i);
                    base_uniforms.add_str_key_value(key.as_str(), UniformValue::Vec2(offsets[i]));
                }

                let edge_kernel = [-1, -1, -1, -1,  8, -1, -1, -1, -1];
                for i in 0..9 {
                    let key = format!("edge_kernel[{}]", i);
                    base_uniforms.add_str_key_value(key.as_str(), UniformValue::SignedInt(edge_kernel[i]));
                }

                let blur_kernel: [f32; 9] = [
                    1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0,
                    2.0 / 16.0, 4.0 / 16.0, 2.0 / 16.0,
                    1.0 / 16.0, 2.0 / 16.0, 1.0 / 16.0
                ];
                for i in 0..9 {
                    let key = format!("blur_kernel[{}]", i);
                    base_uniforms.add_str_key_value(key.as_str(), UniformValue::Float(blur_kernel[i]));
                }
                
                Holder {
                    ms_framebuffer: SimpleFrameBuffer::new(display, &dt.ms_renderbuffer).unwrap(),
                    framebuffer: SimpleFrameBuffer::new(display, &dt.texture).unwrap(),
                    base_uniforms,
                }
            },
        }.build();

        Self {
            program,
            actual,
            confuse: false,
            chaos: false,
            shake: false,
            shake_time: 0.0,
            width,
            height,
            rect: Rect { left: 0, bottom: 0, width: width, height: height },
            blit_target: BlitTarget { left: 0, bottom: 0, width: width as i32, height: height as i32 },
            vertex_buffer: VertexBuffer::new(display, &[
                RawVertexP2T { position: [-1.0, -1.0], texture: [0.0, 0.0] },
                RawVertexP2T { position: [1.0, 1.0], texture: [1.0, 1.0] },
                RawVertexP2T { position: [-1.0, 1.0], texture: [0.0, 1.0] },

                RawVertexP2T { position: [-1.0, -1.0], texture: [0.0, 0.0] },
                RawVertexP2T { position: [1.0, -1.0], texture: [1.0, 0.0] },
                RawVertexP2T { position: [1.0, 1.0], texture: [1.0, 1.0] },
            ]).unwrap(),
            index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap(),
        }
    }

    /// 后期处理
    /// render - 调用方渲染逻辑，将多重采样缓冲提供给调用方，供其渲染
    pub fn post_process<F>(&mut self, render: F) where F: Fn(&mut SimpleFrameBuffer) {
        self.actual.with_mut(|fields| {
            let holder = fields.holder;
            holder.ms_framebuffer.clear_color(0.0, 0.0, 0.0, 1.0);

            // 使用多重采样缓冲渲染
            render(&mut holder.ms_framebuffer);

            // 将多重采用缓存数据复制到framebuffer中以填充后期处理的结果至texture
            holder.framebuffer.blit_buffers_from_simple_framebuffer(&holder.ms_framebuffer, &self.rect, &self.blit_target, MagnifySamplerFilter::Nearest, BlitMask::color());
        });
    }

    pub fn render<T: Surface>(&mut self, target: &mut T, time: Duration) {
        // 将后期处理结果渲染到指定surface
        self.actual.with_mut(|fields| {
            let holder = fields.holder;

            let time = time.as_secs_f32();
            
            let mut uniforms = holder.base_uniforms.clone();
            uniforms.add_str_key("time", &time);
            uniforms.add_str_key("confuse", &self.confuse);
            uniforms.add_str_key("chaos", &self.chaos);
            uniforms.add_str_key("shake", &self.shake);

            target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &DrawParameters::default()).unwrap();
        });
    }

    pub fn update(&mut self, dt: Duration) {
        if self.shake_time > 0.0 {
            self.shake_time -= dt.as_secs_f32();
            if self.shake_time <= 0.0 {
                self.shake = false;
            }
        }
    }

    pub fn start_shake(&mut self, shake_time: f32) {
        self.shake = true;
        self.shake_time = shake_time;
    }
}