
extern crate glium;
extern crate cgmath;

use std::{time::{self}, marker::PhantomPinned};

use cgmath::{SquareMatrix, Point3, Matrix4};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState, Event}, window::CursorGrabMode}};

use rust_opengl_learn::{camera::{Camera, CameraController, CameraControllerProxy}, uniforms::DynamicUniforms, objects::{Cube, Plane}, material, create_program, keyboard::{handle_keyboard_input, KeyboardController}, start_loop, Action, mouse::MouseController, event::{EventHandler, keyboard::{KeyboardHandler, KeyboardInteract}, mouse::{MouseHandler, MouseInteract}}, context::{LoopContext, LoopStore}};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 物体着色器程序
    let obj_program = create_program("src/bin/senior_opengl_depth_test/obj_shader_test.vert", "src/bin/senior_opengl_depth_test/obj_shader_test.frag", &display);
    // 线性深度测试
    // let obj_program = create_program("src/bin/senior_opengl_depth_test/obj_shader_test.vert", "src/bin/senior_opengl_depth_test/obj_shader_test_linear.frag", &display);

    let cube1 = Cube::new("cube1", 1.0, &display, [1.0, 1.0, 1.0], Point3::new(-1.0, 0.0, -1.0), Matrix4::identity());
    let cube2 = Cube::new("cube2", 1.0, &display, [1.0, 1.0, 1.0], Point3::new(2.0, 0.0, 0.0), Matrix4::identity());
    let cubes = [cube1, cube2];
    let plane = Plane::new("plane", 10.0, 10.0, -0.5001_f32, &display, Point3::new(0.0, 0.0, 0.0), Matrix4::identity());

    let cube_texture = material::load_texture("src/marble.jpg".to_string(), &display).1;
    let floor_texture = material::load_texture("src/metal.png".to_string(), &display).1;

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 9_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let controller = CameraController::new(1_f32, 0.5_f32);
    let controller = CameraControllerProxy::new(controller);
    
    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 100.0));

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    // 事件处理逻辑
    let keyboard_handler = KeyboardHandler::new();

    let mouse_handler = MouseHandler::new();

    let event_handler = EventHandler::new(keyboard_handler, mouse_handler);

    let loop_store = LoopStore::new(controller);

    let loop_context = LoopContext::new(event_handler, camera);

    start_loop(event_loop, loop_store, loop_context, move |_: Option<Event<()>>, ctx| {
        // 摄像机观察矩阵
        let view_matrix = Into::<[[f32; 4]; 4]>::into(ctx.camera.calc_matrix());

        let camera_position = Into::<[f32; 3]>::into(ctx.camera.position);

        // drawing a frame
        let mut target = display.draw();
        // target.clear_color(0.2, 0.3, 0.3, 1.0);
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
        
        let mut box_uniforms = DynamicUniforms::new();
        box_uniforms.add(String::from("view"), &view_matrix);
        box_uniforms.add(String::from("projection"), &projection_matrix);
        box_uniforms.add(String::from("viewPos"), &camera_position);
        
        // 循环渲染模型
        for cube in cubes.iter() {
            let mut uniforms = box_uniforms.clone();
            let model = Into::<[[f32; 4]; 4]>::into(cube.position_matrix() * cube.model);
            uniforms.add_str_key("model", &model);
            uniforms.add_str_key("texture1", &cube_texture);
            target.draw(&cube.vertex_buffer, &cube.index_buffer, &obj_program, &uniforms, &draw_parameters).unwrap();
        }

        let model = Into::<[[f32; 4]; 4]>::into(plane.calc_model(Matrix4::identity()));
        box_uniforms.add_str_key("model", &model);
        box_uniforms.add_str_key("texture1", &floor_texture);
        target.draw(&plane.vertex_buffer, &plane.index_buffer, &obj_program, &box_uniforms, &draw_parameters).unwrap();

        target.finish().unwrap();

        Action::Continue
    });
}
