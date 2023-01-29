
extern crate glium;
extern crate cgmath;

use cgmath::{Point3, Matrix4, Vector3, Rad, InnerSpace, Deg};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event, VirtualKeyCode, KeyboardInput, ElementState}, window::CursorGrabMode, dpi::LogicalSize}, framebuffer::SimpleFrameBuffer, texture::{DepthCubemap}, uniforms::{UniformValue, Sampler, MagnifySamplerFilter, SamplerWrapFunction, MinifySamplerFilter}};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube}, material, create_program, start_loop, Action, context::{LoopContext, CONTEXT_STORE, ContextValue}, create_program_vgf, event::keyboard::KeyboardInteract};

/// 点光源阴影映射demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::new(800.0, 600.0);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let obj_program = create_program("src/bin/advanced_lighting_point_shadow/formal.vert", "src/bin/advanced_lighting_point_shadow/formal.frag", &display);
    let shadow_program = create_program_vgf("src/bin/advanced_lighting_point_shadow/shadow.vert", "src/bin/advanced_lighting_point_shadow/shadow.geom", "src/bin/advanced_lighting_point_shadow/shadow.frag", &display);

    let texture = material::load_texture("src/wood.png".to_string(), &display).1;

    let outter_cube = Cube::new_skybox("cube0", 10.0, &display);

    // 正方体
    let cubes = vec![
        Cube::new("cube1", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(4.0, -3.5, 0.0), Matrix4::from_scale(0.5)),
        Cube::new("cube2", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(2.0, 3.0, 1.0), Matrix4::from_scale(0.75)),
        Cube::new("cube3", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(-3.0, -1.0, 0.0), Matrix4::from_scale(0.5)),
        Cube::new("cube4", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(-1.5, 1.0, 1.5), Matrix4::from_scale(0.5)),
        Cube::new("cube5", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(-1.5, 2.0, -3.0), Matrix4::from_axis_angle(Vector3::new(1.0, 0.0, 1.0).normalize(), Rad(60.0)) * Matrix4::from_scale(0.75))
    ];

    // 阴影贴图
    let shadow_size: f32 = 1024.0;
    let shadow_cubemap = DepthCubemap::empty(&display, shadow_size as u32).unwrap();

    // 光源视角位置
    let light_view_position = Point3::new(0.0, 0.0, 0.0);

    // 本例中光源为平行定向光，所以使用正交投影矩阵，这样透视图不会变形
    let light_far_plane: f32 = 25.0;
    let light_projection = cgmath::perspective(Deg(90.0), shadow_size / shadow_size, 1.0, light_far_plane);
    // 光空间矩阵
    let light_space_matrixes: Vec<[[f32; 4]; 4]> = {
        let mut arr: Vec<[[f32; 4]; 4]> = Vec::with_capacity(6);
        // 根据cubemap各个面的观察矩阵创建各面的光空间变换矩阵
        // right
        let light_view = Matrix4::look_at_rh(light_view_position, light_view_position + Vector3::new(1.0, 0.0, 0.0), -Vector3::unit_y());
        arr.push((light_projection * light_view).into());
        // left
        let light_view = Matrix4::look_at_rh(light_view_position, light_view_position + Vector3::new(-1.0, 0.0, 0.0), -Vector3::unit_y());
        arr.push((light_projection * light_view).into());
        // top
        let light_view = Matrix4::look_at_rh(light_view_position, light_view_position + Vector3::new(0.0, 1.0, 0.0), Vector3::unit_z());
        arr.push((light_projection * light_view).into());
        // bottom
        let light_view = Matrix4::look_at_rh(light_view_position, light_view_position + Vector3::new(0.0, -1.0, 0.0), -Vector3::unit_z());
        arr.push((light_projection * light_view).into());
        // near
        let light_view = Matrix4::look_at_rh(light_view_position, light_view_position + Vector3::new(0.0, 0.0, 1.0), -Vector3::unit_y());
        arr.push((light_projection * light_view).into());
        // far
        let light_view = Matrix4::look_at_rh(light_view_position, light_view_position + Vector3::new(0.0, 0.0, -1.0), -Vector3::unit_y());
        arr.push((light_projection * light_view).into());

        arr
        
    };

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 3_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let controller = CameraController::new(1_f32, 0.5_f32);
    // 摄像机透视矩阵
    let projection_matrix: [[f32; 4]; 4] = cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 100.0).into();

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
        // 创建深度贴图的帧缓冲
        let mut depth_framebuffer = SimpleFrameBuffer::depth_only(&display, &shadow_cubemap).unwrap();
        depth_framebuffer.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        // 渲染场景到帧缓冲
        let mut uniforms = DynamicUniforms::new();
        for (i, light_space_matrix) in light_space_matrixes.iter().enumerate() {
            let light_key = format!("lightSpaceMatrixes[{}]", i);
            uniforms.add(light_key, light_space_matrix);
        }
        uniforms.add_str_key("far_plane", &light_far_plane);
        uniforms.add_str_key_value("lightPos", UniformValue::Vec3(light_view_position.into()));
        
        uniforms.add_str_key_value("model", UniformValue::Mat4(outter_cube.calc_model().into()));
        depth_framebuffer.draw(&outter_cube.vertex_buffer, &outter_cube.index_buffer, &shadow_program, &uniforms, &draw_parameters).unwrap();
        // cubes
        for cube in &cubes {
            uniforms.add_str_key_value("model", UniformValue::Mat4(cube.calc_model().into()));
            depth_framebuffer.draw(&cube.vertex_buffer, &cube.index_buffer, &shadow_program, &uniforms, &draw_parameters).unwrap();
        }

        // 渲染主要场景
        // 摄像机观察矩阵
        let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
        let camera_position: [f32; 3] = ctx.camera.position.into();

        let mut target = display.draw();
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
        
        uniforms.add_str_key("projection", &projection_matrix);
        uniforms.add_str_key("view", &view_matrix);
        uniforms.add_str_key("viewPos", &camera_position);
        uniforms.add_str_key("diffuseTexture", &texture);

        let shadow_sampler = Sampler::new(&shadow_cubemap).magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest).wrap_function(SamplerWrapFunction::Clamp);
        uniforms.add_str_key("depthMap", &shadow_sampler);

        let store = CONTEXT_STORE.lock().unwrap();
        if let Some(ContextValue::BOOL(v)) = store.get_value("shadows") {
            uniforms.add_str_key("shadows", v);
        }

        uniforms.add_str_key_value("model", UniformValue::Mat4(outter_cube.calc_model().into()));
        target.draw(&outter_cube.vertex_buffer, &outter_cube.index_buffer, &obj_program, &uniforms, &draw_parameters).unwrap();
        // cubes
        for cube in &cubes {
            let model: [[f32; 4]; 4] = cube.calc_model().into();
            uniforms.add_str_key_value("model", UniformValue::Mat4(model));
            target.draw(&cube.vertex_buffer, &cube.index_buffer, &obj_program, &uniforms, &draw_parameters).unwrap();
        }

        target.finish().unwrap();

        Action::Continue
    });
}


pub struct KeyboardInteractor;

impl KeyboardInteract for KeyboardInteractor {

    fn init(&self) {
        CONTEXT_STORE.lock().unwrap().set_value("shadows", ContextValue::BOOL(false))
    }

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode> {
        vec![VirtualKeyCode::F]
    }

    fn interact(&self, input: KeyboardInput) {
        if input.state == ElementState::Released {
            let mut store = CONTEXT_STORE.lock().unwrap();
            let blinn = store.get_value("shadows");
            if let Some(ContextValue::BOOL(v)) = blinn {
                let v = *v;
                store.set_value("shadows", ContextValue::BOOL(!v));
            }
        }
    }
}