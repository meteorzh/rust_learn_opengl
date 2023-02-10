
extern crate glium;
extern crate cgmath;

use cgmath::{Matrix4, Vector3};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue, MagnifySamplerFilter}, framebuffer::{DepthRenderBuffer, MultiOutputFrameBuffer}, texture::{Texture2d, UncompressedFloatFormat, DepthFormat, MipmapsOption}, index::PrimitiveType, BlitTarget, Rect, BlitMask};

use ouroboros::self_referencing;
use rand::{SeedableRng, rngs::StdRng, Rng};
use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube, Plane}, create_program, start_loop, Action, context::{LoopContext}, lights::PointLight, load_wavefront_obj_as_models};

pub struct Dt {
    position_texture: Texture2d,
    normal_texture: Texture2d,
    color_specular_texture: Texture2d,
    g_render_buffer: DepthRenderBuffer,
}

#[self_referencing]
struct Data {
    dt: Dt,
    #[borrows(dt)]
    #[covariant]
    buffs: (MultiOutputFrameBuffer<'this>, &'this Dt),
}

/// Deferred Shading Volume demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let geometry_pass_program = create_program("src/bin/advanced_lighting_deferred_shading_volumes/g_buffer.vert", "src/bin/advanced_lighting_deferred_shading_volumes/g_buffer.frag", &display);
    let lighting_pass_program = create_program("src/bin/advanced_lighting_deferred_shading_volumes/deferred_shading.vert", "src/bin/advanced_lighting_deferred_shading_volumes/deferred_shading.frag", &display);
    let light_cube_program = create_program("src/bin/advanced_lighting_deferred_shading_volumes/light_box.vert", "src/bin/advanced_lighting_deferred_shading_volumes/light_box.frag", &display);

    let rect = Rect {
        left: 0,
        bottom: 0,
        width: size.width,
        height: size.height,
    };

    let blit_target = BlitTarget {
        left: 0,
        bottom: 0,
        width: size.width as i32,
        height: size.height as i32,
    };

    // 加载模型
    let models = load_wavefront_obj_as_models(&display, "src/backpack/", "backpack.obj");
    let model_translations = vec![
        Matrix4::from_translation(Vector3::new(-3.0, -0.5, -3.0)),
        Matrix4::from_translation(Vector3::new(0.0, -0.5, -3.0)),
        Matrix4::from_translation(Vector3::new(3.0, -0.5, -3.0)),
        Matrix4::from_translation(Vector3::new(-3.0, -0.5, 0.0)),
        Matrix4::from_translation(Vector3::new(0.0, -0.5, 0.0)),
        Matrix4::from_translation(Vector3::new(3.0, -0.5, 0.0)),
        Matrix4::from_translation(Vector3::new(-3.0, -0.5, 3.0)),
        Matrix4::from_translation(Vector3::new(0.0, -0.5, 3.0)),
        Matrix4::from_translation(Vector3::new(3.0, -0.5, 3.0)),
    ];

    let quad = Plane::new_vertical_center_plane("quad", 2.0, 2.0, &display, PrimitiveType::TrianglesList);

    // 点光源
    let point_lights = {
        // 随机数生成器
        let light_num = 2;
        let mut rng = StdRng::seed_from_u64(13 as u64);
        let mut lights = Vec::with_capacity(4);
        let mut rand = |div: f32| {
            rng.gen_range(0..100) as f32 / div
        };
        for _ in 0..light_num {
            let mut light = PointLight::new(
                [rand(100.0) * 6.0 - 3.0, rand(100.0) * 6.0 - 4.0, rand(100.0) * 6.0 - 3.0], 
                [rand(200.0) + 0.5, rand(200.0) + 0.5, rand(200.0) + 0.5],
                1.0, 0.7, 1.8, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]
            );
            let max_brightness = light.color[0].max(light.color[1]).max(light.color[2]);
            light.radius = (-light.linear + (light.linear * light.linear - 4.0 * light.quadratic * (light.constant - (256.0 / 5.0) * max_brightness)).sqrt()) / (2.0 * light.quadratic);
            lights.push(light);
        }

        lights
    };

    let light_cubes = {
        let mut cubes = Vec::with_capacity(point_lights.len());
        for light in point_lights.iter() {
            cubes.push(Cube::new("light_cube", 2.0, &display, light.color, light.position.into(), Matrix4::from_scale(0.125)));
        }
        cubes
    };

    // HDR FrameBuffer
    let mut tenants = DataBuilder {
        dt: Dt {
            position_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
            normal_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
            color_specular_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::U8U8U8U8, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
            g_render_buffer: DepthRenderBuffer::new(&display, DepthFormat::I24, size.width, size.height).unwrap(),
        },
        buffs_builder: |dt| {
            let output = [("GPosition", &dt.position_texture), ("GNormal", &dt.normal_texture), ("GColorSpecular", &dt.color_specular_texture)];
            let g_framebuffer = MultiOutputFrameBuffer::with_depth_buffer(&display, output.iter().cloned(), &dt.g_render_buffer).unwrap();

            (g_framebuffer, dt)
        }
    }.build();

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let camera = Camera::new(
        cgmath::Point3::new(0.0, 0.0, 5.0),
        cgmath::Rad::from(cgmath::Deg(-90.0)),
        cgmath::Rad::from(cgmath::Deg(0.0))
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
            let camera_position: [f32; 3] = ctx.camera.position.into();

            let mut uniforms = DynamicUniforms::new();
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("view", &view_matrix);

            // 1. 几何处理阶段: 渲染场景的几何数据到g_framebuffer
            let g_framebuffer = &mut fields.buffs.0;
            g_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            // 渲染模型
            for model in models.iter() {    
                if let Some(material) = &model.material {
                    if let Some(diffuse_map) = &material.diffuse_map {
                        uniforms.add_str_key("texture_diffuse1", diffuse_map.as_ref());
                    }
                    if let Some(specular_map) = &material.specular_map {
                        uniforms.add_str_key("texture_specular1", specular_map.as_ref());
                    }
                }
                for model_translation in model_translations.iter() {
                    let model_matrix = Matrix4::from_scale(0.5) * model_translation;
                    uniforms.add_str_key_value("model", UniformValue::Mat4(model_matrix.into()));
                    g_framebuffer.draw(&model.vertex_buffer, &model.index_buffer, &geometry_pass_program, &uniforms, &draw_parameters).unwrap();
                }
            }

            // 创建默认帧
            let mut target = display.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            // 2. 光照计算阶段
            uniforms.clear();
            uniforms.add_str_key("gPosition", &dt.position_texture);
            uniforms.add_str_key("gNormal", &dt.normal_texture);
            uniforms.add_str_key("gAlbedoSpec", &dt.color_specular_texture);
            for (i, light) in point_lights.iter().enumerate() {
                let light_key = format!("lights[{}]", i);
                light.add_to_uniforms(light_key.as_str(), &mut uniforms);
            }
            uniforms.add_str_key("viewPos", &camera_position);
            target.draw(&quad.vertex_buffer, &quad.index_buffer, &lighting_pass_program, &uniforms, &draw_parameters).unwrap();
            // 将深度信息从 g_framebuffer 复制到默认帧
            target.blit_buffers_from_multioutput_framebuffer(&g_framebuffer, &rect, &blit_target, MagnifySamplerFilter::Nearest, BlitMask::depth());

            // 渲染光源立方体
            uniforms.clear();
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("view", &view_matrix);
            for light_cube in light_cubes.iter() {
                uniforms.add_str_key_value("model", UniformValue::Mat4(light_cube.calc_model().into()));
                uniforms.add_str_key("lightColor", &light_cube.color);
                target.draw(&light_cube.vertex_buffer, &light_cube.index_buffer, &light_cube_program, &uniforms, &draw_parameters).unwrap();
            }

            target.finish().unwrap();

            Action::Continue
        })
    });
}
