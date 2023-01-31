
extern crate glium;
extern crate cgmath;

use cgmath::{Matrix4, Vector3, InnerSpace, Deg};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue}};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Plane}, material, create_program, start_loop, Action, context::{LoopContext}, lights::PointLight};

/// 法线贴图demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::new(800.0, 600.0);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let program = create_program("src/bin/advanced_lighting_normal_mapping/formal.vert", "src/bin/advanced_lighting_normal_mapping/formal.frag", &display);

    let texture = material::load_texture("src/brickwall/brickwall.jpg".to_string(), &display).1;
    let normal_texture = material::load_texture("src/brickwall/brickwall_normal.jpg".to_string(), &display).1;

    // 点光源
    let point_light = PointLight::new_simple([0.5, 1.0, 0.3], [0.0, 0.0, 0.0]);

    // 砖墙
    let wall = Plane::new_vertical_center_plane("wall", 2.0, 2.0, &display, glium::index::PrimitiveType::TrianglesList);
    let mut wall_drgee: f32 = 0.0;

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 3_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let controller = CameraController::new(1_f32, 0.5);
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
        // 摄像机观察矩阵
        let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
        let camera_position: [f32; 3] = ctx.camera.position.into();

        let mut target = display.draw();
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);

        wall_drgee += 0.1;

        let wall_model = Matrix4::from_axis_angle(Vector3::new(1.0, 0.0, 1.0).normalize(), Deg(wall_drgee));

        let mut uniforms = DynamicUniforms::new();
        
        uniforms.add_str_key("projection", &projection_matrix);
        uniforms.add_str_key("view", &view_matrix);
        uniforms.add_str_key("viewPos", &camera_position);
        uniforms.add_str_key("wallTexture", &texture);
        uniforms.add_str_key("normalMap", &normal_texture);
        point_light.add_to_uniforms("light", &mut uniforms);

        uniforms.add_str_key_value("model", UniformValue::Mat4(wall_model.into()));
        // 渲染墙面
        target.draw(&wall.vertex_buffer, &wall.index_buffer, &program, &uniforms, &draw_parameters).unwrap();

        target.finish().unwrap();

        Action::Continue
    });
}
