
extern crate glium;
extern crate cgmath;

use std::borrow::Cow;

use cgmath::{Matrix4, Vector3, InnerSpace, Point3, Rad, SquareMatrix, Deg};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue}, framebuffer::{DepthRenderBuffer, MultiOutputFrameBuffer, SimpleFrameBuffer}, texture::{Texture2d, UncompressedFloatFormat, DepthFormat, MipmapsOption, RawImage2d, ClientFormat}, index::PrimitiveType};

use ouroboros::self_referencing;
use rand::{Rng};
use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube, Plane}, create_program, start_loop, Action, context::{LoopContext}, lights::PointLight, load_wavefront_obj_as_models};

pub struct Dt {
    position_texture: Texture2d,
    normal_texture: Texture2d,
    color_specular_texture: Texture2d,
    g_render_buffer: DepthRenderBuffer,
    ssao_color_texture: Texture2d,
    ssao_color_blur_texture: Texture2d,
}

#[self_referencing]
struct Data {
    dt: Dt,
    #[borrows(dt)]
    #[covariant]
    buffs: (MultiOutputFrameBuffer<'this>, SimpleFrameBuffer<'this>, SimpleFrameBuffer<'this>, Texture2d, &'this Dt),
}

/// ssao demo 屏幕空间环境光遮蔽(Screen-Space Ambient Occlusion, SSAO)
/// 褶皱，空洞，墙角等地方的光应该要暗一些
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let geometry_pass_program = create_program("src/bin/advanced_lighting_ssao/ssao_geometry.vert", "src/bin/advanced_lighting_ssao/ssao_geometry.frag", &display);
    let lighting_pass_program = create_program("src/bin/advanced_lighting_ssao/ssao.vert", "src/bin/advanced_lighting_ssao/ssao_lighting.frag", &display);
    let ssao_program = create_program("src/bin/advanced_lighting_ssao/ssao.vert", "src/bin/advanced_lighting_ssao/ssao.frag", &display);
    let ssao_blur_program = create_program("src/bin/advanced_lighting_ssao/ssao.vert", "src/bin/advanced_lighting_ssao/ssao_blur.frag", &display);

    // 加载模型
    let models = load_wavefront_obj_as_models(&display, "src/nanosuit/", "nanosuit.obj");
    let model_translate = Matrix4::from_translation(Vector3::new(-7.0, -11.5, 0.0)) * Matrix4::from_angle_z(Deg(-90.0)) * Matrix4::from_angle_y(Deg(-90.0));

    let quad = Plane::new_vertical_center_plane("quad", 2.0, 2.0, &display, PrimitiveType::TrianglesList);

    let room_cube = Cube::new("room", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(0.0, 7.0, 0.0), Matrix4::from_scale(20.0));

    // 点光源
    let point_light = PointLight::new(
        [2.0, 4.0, -2.0], 
        [0.2, 0.2, 0.7],
        0.0, 0.09, 0.032, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]
    );

    // HDR FrameBuffer
    let mut tenants = DataBuilder {
        dt: Dt {
            position_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
            normal_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
            color_specular_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8U8U8U8, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
            g_render_buffer: DepthRenderBuffer::new(&display, DepthFormat::I24, size.width, size.height).unwrap(),
            ssao_color_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
            ssao_color_blur_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
        },
        buffs_builder: |dt| {
            let output = [("GPosition", &dt.position_texture), ("GNormal", &dt.normal_texture), ("GColorSpecular", &dt.color_specular_texture)];
            let g_framebuffer = MultiOutputFrameBuffer::with_depth_buffer(&display, output.iter().cloned(), &dt.g_render_buffer).unwrap();

            let ssao_framebuffer = SimpleFrameBuffer::new(&display, &dt.ssao_color_texture).unwrap();
            let ssao_blur_framebuffer = SimpleFrameBuffer::new(&display, &dt.ssao_color_blur_texture).unwrap();

            let noise_image = RawImage2d { data: Cow::Owned({
                let mut data = Vec::with_capacity(4 * 4 * 4);
                let mut rng = rand::thread_rng();
                for _ in 0..16 {
                    data.push(rng.gen_range(0.0..1.0) * 2.0 - 1.0);
                    data.push(rng.gen_range(0.0..1.0) * 2.0 - 1.0);
                    data.push(0.0);
                    data.push(0.0);
                }
                data
            }), width: 4, height: 4, format: ClientFormat::F32F32F32F32 };
            let noise_texture = Texture2d::with_format(&display, noise_image, UncompressedFloatFormat::F32F32F32F32, MipmapsOption::NoMipmap).unwrap();
            (g_framebuffer, ssao_framebuffer, ssao_blur_framebuffer, noise_texture, dt)
        }
    }.build();

    // 生产采样kernal
    let sample_kernal = {
        let mut kernal = Vec::with_capacity(64);
        let mut rng = rand::thread_rng();
        for i in 0..64 {
            let mut vector = Vector3::new(rng.gen_range(0.0..1.0) as f32 * 2.0 - 1.0, rng.gen_range(0.0..1.0) * 2.0 - 1.0, rng.gen_range(0.0..1.0));
            vector = vector.normalize();
            vector *= rng.gen_range(0.0..1.0) as f32;

            let scale = i as f32 / 64.0;
            let scale = (scale * scale) * (1.0 - 0.1) + 0.1;
            vector *= scale;
            kernal.push(vector);
        }
        kernal
    };

    // 摄像机初始位置
    let camera = Camera::new(
        cgmath::Point3::new(12.0, -6.0, 10.0),
        cgmath::Rad::from(cgmath::Deg(-150.0)),
        cgmath::Rad::from(cgmath::Deg(-30.0))
    );
    let controller = CameraController::new(1.0, 0.5);
    // 摄像机透视矩阵
    let projection_matrix: [[f32; 4]; 4] = cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1, 100.0).into();

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let loop_context = LoopContext::new(camera, controller);

    start_loop(event_loop, loop_context, move |_: Option<Event<()>>, ctx| {
        tenants.with_mut(|fields| {
            let dt = fields.dt;
            // 摄像机观察矩阵
            let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
            // let camera_position: [f32; 3] = ctx.camera.position.into();

            let mut uniforms = DynamicUniforms::new();
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("view", &view_matrix);

            // 1. 几何处理阶段: 渲染场景的几何数据到g_framebuffer
            let g_framebuffer = &mut fields.buffs.0;
            g_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            // render room cube
            uniforms.add_str_key_value("model", UniformValue::Mat4(room_cube.calc_model().into()));
            uniforms.add_str_key_value("invertedNormals", UniformValue::Bool(true));
            g_framebuffer.draw(&room_cube.vertex_buffer, &room_cube.index_buffer, &geometry_pass_program, &uniforms, &draw_parameters).unwrap();
            // 模型倒放到地面
            uniforms.add_str_key_value("invertedNormals", UniformValue::Bool(false));
            for model in models.iter() {
                uniforms.add_str_key_value("model", UniformValue::Mat4(model_translate.into()));
                g_framebuffer.draw(&model.vertex_buffer, &model.index_buffer, &geometry_pass_program, &uniforms, &draw_parameters).unwrap();
            }

            // 生成ssao framebuffer
            let ssao_framebuffer = &mut fields.buffs.1;
            ssao_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            uniforms.clear();
            for i in 0..64 {
                let key = format!("samples[{}]", i);
                let vec = sample_kernal.get(i).unwrap();
                uniforms.add_str_key_value(key.as_str(), UniformValue::Vec3((*vec).into()));
            }
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("GPosition", &dt.position_texture);
            uniforms.add_str_key("GNormal", &dt.normal_texture);
            uniforms.add_str_key("noiseTexture", &fields.buffs.3);
            ssao_framebuffer.draw(&quad.vertex_buffer, &quad.index_buffer, &ssao_program, &uniforms, &Default::default()).unwrap();

            // blur SSAO texture to remove noise
            let ssao_blur_framebuffer = &mut fields.buffs.2;
            ssao_blur_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            uniforms.clear();
            uniforms.add_str_key("ssaoInput", &dt.ssao_color_texture);
            ssao_blur_framebuffer.draw(&quad.vertex_buffer, &quad.index_buffer, &ssao_blur_program, &uniforms, &Default::default()).unwrap();
            uniforms.clear();

            // 创建默认帧
            let mut target = display.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            // 光照计算阶段
            uniforms.clear();
            uniforms.add_str_key("GPosition", &dt.position_texture);
            uniforms.add_str_key("GNormal", &dt.normal_texture);
            uniforms.add_str_key("GColorSpecular", &dt.color_specular_texture);
            uniforms.add_str_key("ssao", &dt.ssao_color_blur_texture);
            point_light.add_to_uniforms("light", &mut uniforms);

            target.draw(&quad.vertex_buffer, &quad.index_buffer, &lighting_pass_program, &uniforms, &draw_parameters).unwrap();

            target.finish().unwrap();

            Action::Continue
        })
    });
}
