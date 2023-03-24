
extern crate glium;
extern crate cgmath;

use cgmath::{Matrix4, Vector3, Point3};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event, VirtualKeyCode, KeyboardInput, ElementState}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue, SamplerBehavior, SamplerWrapFunction}, framebuffer::{SimpleFrameBuffer, DepthRenderBuffer}, texture::{Texture2d, UncompressedFloatFormat, DepthFormat, MipmapsOption}, index::PrimitiveType};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube, Plane}, material, create_program, start_loop, Action, context::{LoopContext, CONTEXT_STORE, ContextValue}, lights::PointLight, event::keyboard::KeyboardInteract};

/// HDR demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let program = create_program("src/bin/advanced_lighting_hdr/hdr.vert", "src/bin/advanced_lighting_hdr/hdr.frag", &display);
    let light_program = create_program("src/bin/advanced_lighting_hdr/light.vert", "src/bin/advanced_lighting_hdr/light.frag", &display);

    let texture = material::load_texture("src/wood.png".to_string(), &display).1;

    let cube = Cube::new("cube", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(0.0, 0.0, 0.0), Matrix4::from_translation(Vector3::new(0.0, 0.0, 25.0)) * Matrix4::from_nonuniform_scale(2.5, 2.5, 27.5));
    let quad = Plane::new_vertical_center_plane("quad", 2.0, 2.0, &display, PrimitiveType::TrianglesList);

    // 点光源
    let point_lights = {
        let mut lights = Vec::with_capacity(4);
        lights.push(PointLight::new_simple([0.0, 0.0, 49.5], [200.0, 200.0, 200.0]));
        lights.push(PointLight::new_simple([-1.4, -1.9, 9.0], [0.1, 0.0, 0.0]));
        lights.push(PointLight::new_simple([0.0, -1.8, 4.0], [0.0, 0.0, 0.2]));
        lights.push(PointLight::new_simple([0.8, -1.7, 6.0], [0.0, 0.1, 0.0]));

        lights
    };

    // HDR FrameBuffer
    let hdr_texture = Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16F16, MipmapsOption::NoMipmap, size.width, size.height).unwrap();
    let hdr_render_buffer = DepthRenderBuffer::new(&display, DepthFormat::F32, size.width, size.height).unwrap();

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

    start_loop(event_loop, loop_context, move |_: Option<Event<()>>, ctx, _| {
        // 摄像机观察矩阵
        let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
        let camera_position: [f32; 3] = ctx.camera.position.into();

        // 渲染场景至framebuffer
        let mut hdr_framebuffer = SimpleFrameBuffer::with_depth_buffer(&display, &hdr_texture, &hdr_render_buffer).unwrap();
        hdr_framebuffer.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

        let mut uniforms = DynamicUniforms::new();
        uniforms.add_str_key("projection", &projection_matrix);
        uniforms.add_str_key("view", &view_matrix);
        uniforms.add_str_key("diffuseTexture", &texture);
        uniforms.add_str_key("viewPos", &camera_position);
        for (i, point_light) in point_lights.iter().enumerate() {
            let light_key = format!("lights[{}]", i);
            point_light.add_to_uniforms(light_key.as_str(), &mut uniforms);
        }

        let cube_model: [[f32; 4]; 4] = cube.calc_model().into();
        uniforms.add_str_key("model", &cube_model);
        uniforms.add_str_key_value("inverse_normals", UniformValue::Bool(true));

        hdr_framebuffer.draw(&cube.vertex_buffer, &cube.index_buffer, &light_program, &uniforms, &draw_parameters).unwrap();

        // 渲染到默认帧
        let mut target = display.draw();
        target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

        let mut uniforms = DynamicUniforms::new();
        let texture_value = UniformValue::Texture2d(&hdr_texture, Some(SamplerBehavior {
            wrap_function: (SamplerWrapFunction::Clamp, SamplerWrapFunction::Clamp, SamplerWrapFunction::Clamp),
            ..Default::default()
        }));
        uniforms.add_str_key_value("hdrBuffer", texture_value);
        let store = CONTEXT_STORE.lock().unwrap();
        if let Some(ContextValue::BOOL(v)) = store.get_value("hdr") {
            uniforms.add_str_key_value("hdr", UniformValue::Bool(*v));
        }
        if let Some(ContextValue::F32(v)) = store.get_value("exposure") {
            uniforms.add_str_key_value("exposure", UniformValue::Float(*v));
        }

        target.draw(&quad.vertex_buffer, &quad.index_buffer, &program, &uniforms, &draw_parameters).unwrap();

        target.finish().unwrap();

        Action::Continue
    });
}


pub struct KeyboardInteractor;

impl KeyboardInteract for KeyboardInteractor {

    fn init(&self) {
        let mut store = CONTEXT_STORE.lock().unwrap();
        store.set_value("exposure", ContextValue::F32(1.0));
        store.set_value("hdr", ContextValue::BOOL(false));
    }

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode> {
        vec![VirtualKeyCode::Q, VirtualKeyCode::E, VirtualKeyCode::F]
    }

    fn interact(&mut self, input: KeyboardInput) {
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
                    let hdr = store.get_value("hdr");
                    if let Some(ContextValue::BOOL(v)) = hdr {
                        let v = *v;
                        store.set_value("hdr", ContextValue::BOOL(!v));
                    }
                }
            }
        }
    }
}