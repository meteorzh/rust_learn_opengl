
extern crate glium;
extern crate cgmath;

use cgmath::{Matrix4, Vector3, Point3, Rad, InnerSpace};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event, VirtualKeyCode, KeyboardInput, ElementState}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue}, framebuffer::{SimpleFrameBuffer, DepthRenderBuffer, MultiOutputFrameBuffer}, texture::{Texture2d, UncompressedFloatFormat, DepthFormat, MipmapsOption}, index::PrimitiveType};

use ouroboros::self_referencing;
use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube, Plane}, material, create_program, start_loop, Action, context::{LoopContext, CONTEXT_STORE, ContextValue}, lights::PointLight, event::keyboard::KeyboardInteract};

pub struct Dt {
    hdr_textures: [Texture2d; 2],
    hdr_render_buffer: DepthRenderBuffer,
    ping_pong_textures: [Texture2d; 2],
}

#[self_referencing]
struct Data {
    dt: Dt,
    #[borrows(dt)]
    #[covariant]
    buffs: (MultiOutputFrameBuffer<'this>, [SimpleFrameBuffer<'this>; 2], &'this Dt),
}

/// Bloom demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let bloom_program = create_program("src/bin/advanced_lighting_bloom/bloom.vert", "src/bin/advanced_lighting_bloom/bloom.frag", &display);
    let light_program = create_program("src/bin/advanced_lighting_bloom/bloom.vert", "src/bin/advanced_lighting_bloom/light_box.frag", &display);
    let blur_program = create_program("src/bin/advanced_lighting_bloom/blur.vert", "src/bin/advanced_lighting_bloom/blur.frag", &display);
    let bloom_final_program = create_program("src/bin/advanced_lighting_bloom/bloom_final.vert", "src/bin/advanced_lighting_bloom/bloom_final.frag", &display);

    let wood_texture = material::load_texture("src/wood.png".to_string(), &display).1;
    let container_texture = material::load_texture("src/container2.png".to_string(), &display).1;

    let floor_cube = Cube::new("floor_cube", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(0.0, -1.0, 0.0), Matrix4::from_nonuniform_scale(12.5, 0.5, 12.5));
    let cubes = vec![
        Cube::new("cube0", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(0.0, 1.5, 0.0), Matrix4::from_scale(0.5)),
        Cube::new("cube1", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(2.0, 0.0, 1.0), Matrix4::from_scale(0.5)),
        Cube::new("cube2", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(-1.0, -1.0, 2.0), Matrix4::from_axis_angle(Vector3::new(1.0, 0.0, 1.0).normalize(), Rad(60.0))),
        Cube::new("cube3", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(0.0, 2.7, 4.0), Matrix4::from_axis_angle(Vector3::new(1.0, 0.0, 1.0).normalize(), Rad(23.0)) * Matrix4::from_scale(1.25)),
        Cube::new("cube4", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(-2.0, 1.0, -3.0), Matrix4::from_axis_angle(Vector3::new(1.0, 0.0, 1.0).normalize(), Rad(124.0))),
        Cube::new("cube5", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(-3.0, 0.0, 0.0), Matrix4::from_scale(0.5)),
    ];
    let quad = Plane::new_vertical_center_plane("quad", 2.0, 2.0, &display, PrimitiveType::TrianglesList);

    // 点光源
    let point_lights = {
        let mut lights = Vec::with_capacity(4);
        lights.push(PointLight::new_simple([0.0, 0.5, 1.5], [5.0, 5.0, 5.0]));
        lights.push(PointLight::new_simple([-4.0, 0.5, -3.0], [10.0, 0.0, 0.0]));
        lights.push(PointLight::new_simple([3.0, 0.5, 1.0], [0.0, 0.0, 15.0]));
        lights.push(PointLight::new_simple([-0.8, 2.4, -1.0], [0.0, 5.0, 0.0]));

        lights
    };

    let light_cubes = {
        let mut cubes = Vec::with_capacity(point_lights.len());
        for light in point_lights.iter() {
            cubes.push(Cube::new("light_cube", 2.0, &display, light.color, light.position.into(), Matrix4::from_scale(0.25)));
        }
        cubes
    };

    // HDR FrameBuffer
    let mut tenants = DataBuilder {
        dt: Dt {
            hdr_textures: [
                Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
                Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap()
            ],
            hdr_render_buffer: DepthRenderBuffer::new(&display, DepthFormat::F32, size.width, size.height).unwrap(),
            ping_pong_textures: [
                Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap(),
                Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap()
            ]
        },
        buffs_builder: |dt| {
            let output = [("FragColor", &dt.hdr_textures[0]), ("BrightColor", &dt.hdr_textures[1])];
            let hdr_framebuffer = MultiOutputFrameBuffer::with_depth_buffer(&display, output.iter().cloned(), &dt.hdr_render_buffer).unwrap();
            // ping-pong-framebuffer
            let ping_pong_framebuffers = [
                SimpleFrameBuffer::new(&display, &dt.ping_pong_textures[0]).unwrap(),
                SimpleFrameBuffer::new(&display, &dt.ping_pong_textures[1]).unwrap(),
            ];

            (hdr_framebuffer, ping_pong_framebuffers, dt)
        }
    }.build();

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let camera = Camera::new(
        cgmath::Point3::new(0.0, 0.0, 5.0),
        cgmath::Rad::from(cgmath::Deg(90.0)),
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

    let mut loop_context = LoopContext::new(camera, controller);
    loop_context.register_keyboard(Box::new(KeyboardInteractor{}));

    start_loop(event_loop, loop_context, move |_: Option<Event<()>>, ctx| {
        tenants.with_mut(|fields| {
            let dt = fields.dt;
            // 摄像机观察矩阵
            let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
            let camera_position: [f32; 3] = ctx.camera.position.into();

            let mut uniforms = DynamicUniforms::new();
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("view", &view_matrix);
            uniforms.add_str_key("diffuseTexture", &wood_texture);
            uniforms.add_str_key("viewPos", &camera_position);
            for (i, point_light) in point_lights.iter().enumerate() {
                let light_key = format!("lights[{}]", i);
                point_light.add_to_uniforms(light_key.as_str(), &mut uniforms);
            }

            // 渲染场景至framebuffer
            let hdr_framebuffer = &mut fields.buffs.0;
            hdr_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            // 渲染floor_cube
            uniforms.add_str_key_value("model", UniformValue::Mat4(floor_cube.calc_model().into()));
            hdr_framebuffer.draw(&floor_cube.vertex_buffer, &floor_cube.index_buffer, &bloom_program, &uniforms, &draw_parameters).unwrap();
            // 渲染其它cube
            uniforms.add_str_key("diffuseTexture", &container_texture);
            for cube in cubes.iter() {
                uniforms.add_str_key_value("model", UniformValue::Mat4(cube.calc_model().into()));
                hdr_framebuffer.draw(&cube.vertex_buffer, &cube.index_buffer, &bloom_program, &uniforms, &draw_parameters).unwrap();
            }
            // 渲染light_cubes
            let mut uniforms = DynamicUniforms::new();
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("view", &view_matrix);
            for light_cube in light_cubes.iter() {
                uniforms.add_str_key_value("model", UniformValue::Mat4(light_cube.calc_model().into()));
                uniforms.add_str_key("lightColor", &light_cube.color);
                hdr_framebuffer.draw(&light_cube.vertex_buffer, &light_cube.index_buffer, &light_program, &uniforms, &draw_parameters).unwrap();
            }

            // 高斯模糊
            let ping_pong_framebuffers = &mut fields.buffs.1;
            let mut uniforms = DynamicUniforms::new();
            let mut horizontal = true;
            let mut first_iteration = true;
            for ping_pong_framebuffer in ping_pong_framebuffers.iter_mut() {
                ping_pong_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            }
            for _ in [0..10] {
                let framebuffer = ping_pong_framebuffers.get_mut(horizontal as usize).unwrap();
                uniforms.add_str_key_value("horizontal", UniformValue::Bool(horizontal));
                let texture = {
                    if first_iteration {
                        &dt.hdr_textures[1]
                    } else {
                        &dt.ping_pong_textures[!horizontal as usize]
                    }
                };
                uniforms.add_str_key("image", texture);

                framebuffer.draw(&quad.vertex_buffer, &quad.index_buffer, &blur_program, &uniforms, &Default::default()).unwrap();

                horizontal = !horizontal;
                if first_iteration {
                    first_iteration = false;
                }
            }

            // 渲染到默认帧
            let mut target = display.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
            let mut uniforms = DynamicUniforms::new();
            uniforms.add_str_key("scene", &dt.hdr_textures[0]);
            uniforms.add_str_key("bloomBlur", &dt.ping_pong_textures[!horizontal as usize]);

            let store = CONTEXT_STORE.lock().unwrap();
            if let Some(ContextValue::BOOL(v)) = store.get_value("bloom") {
                uniforms.add_str_key_value("bloom", UniformValue::Bool(*v));
            }
            if let Some(ContextValue::F32(v)) = store.get_value("exposure") {
                uniforms.add_str_key_value("exposure", UniformValue::Float(*v));
            }

            target.draw(&quad.vertex_buffer, &quad.index_buffer, &bloom_final_program, &uniforms, &draw_parameters).unwrap();

            target.finish().unwrap();

            Action::Continue
        })
    });
}


pub struct KeyboardInteractor;

impl KeyboardInteract for KeyboardInteractor {

    fn init(&self) {
        let mut store = CONTEXT_STORE.lock().unwrap();
        store.set_value("exposure", ContextValue::F32(1.0));
        store.set_value("bloom", ContextValue::BOOL(false));
    }

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode> {
        vec![VirtualKeyCode::Q, VirtualKeyCode::E, VirtualKeyCode::F]
    }

    fn interact(&self, input: KeyboardInput) {
        let mut store = CONTEXT_STORE.lock().unwrap();
        if input.state == ElementState::Pressed {
            if let Some(k) = input.virtual_keycode {
                if k == VirtualKeyCode::Q || k == VirtualKeyCode::E {
                    let exposure = store.get_value("exposure");
                    if let Some(ContextValue::F32(v)) = exposure {
                        let mut v = *v;
                        store.set_value("exposure", ContextValue::F32({
                            if k == VirtualKeyCode::Q {
                                if v > 0.0 {
                                    v -= 0.01;
                                } else {
                                    v = 0.0
                                }
                            } else {
                                v += 0.01;
                            }
                            v
                        }));
                    }
                }
            }
        }
        if input.state == ElementState::Released {
            if let Some(k) = input.virtual_keycode {
                if k == VirtualKeyCode::F {
                    let hdr = store.get_value("bloom");
                    if let Some(ContextValue::BOOL(v)) = hdr {
                        let v = *v;
                        store.set_value("bloom", ContextValue::BOOL(!v));
                    }
                }
            }
        }
    }
}