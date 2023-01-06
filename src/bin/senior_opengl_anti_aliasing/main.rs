
extern crate glium;
extern crate cgmath;

use std::{time::{self}};

use cgmath::{SquareMatrix, Point3, Matrix4};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{window::CursorGrabMode, dpi::LogicalSize}};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube}, create_program, keyboard::handle_keyboard_input};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_multisampling(4);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 物体着色器程序
    let program = create_program("src/bin/senior_opengl_anti_aliasing/cube.vert", "src/bin/senior_opengl_anti_aliasing/cube.frag", &display);

    let cube = Cube::new("cube", 1.0, &display, [1.0, 1.0, 1.0], Point3::new(0.0, 0.0, 0.0), Matrix4::identity());

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let mut camera = Camera::new(
        cgmath::Point3::new(-1_f32, 1_f32, 2_f32), 
        cgmath::Rad::from(cgmath::Deg(-70_f32)), 
        cgmath::Rad::from(cgmath::Deg(-20_f32))
    );
    let mut controller = CameraController::new(1_f32, 0.5_f32);
    
    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 100.0));

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let mut last_frame = time::Instant::now();

    // the main loop
    event_loop.run(move |event, _, control_flow| {
        let mut render = false;
        match event {
            event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                event::WindowEvent::CloseRequested => {
                    *control_flow = event_loop::ControlFlow::Exit;
                },
                // Redraw the triangle when the window is resized.
                event::WindowEvent::Resized(..) => {
                    // render(&display, time::Instant::now(), &degree);
                    // render_rectangle(&display, false); // line mode
                    // render_rectangle(&display, true); // line mode
                },
                // key input
                event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(cf) = handle_keyboard_input(input, &mut controller) {
                        *control_flow = cf;
                    }
                },
                _ => {},
            },
            event::Event::DeviceEvent { event, .. } => match event {
                event::DeviceEvent::MouseMotion { delta } => {
                    controller.process_mouse(delta.0, delta.1)
                },
                event::DeviceEvent::MouseWheel { delta } => {
                    controller.process_scroll(&delta);
                },
                _ => {},
            },
            event::Event::NewEvents(cause) => match cause {
                event::StartCause::ResumeTimeReached { .. } => {
                    // 帧时间限制达到后可以渲染
                    render = true;
                },
                event::StartCause::Init => {
                    // 初始化时可以渲染
                    render = true;
                },
                _ => {},
            },
            _ => {},
        }
        if !render {
            return;
        }

        // 帧率设为60FPS，那么1帧16.66666666~毫秒，取16666667纳秒
        let current = time::Instant::now();
        let next_frame_time = current + time::Duration::from_nanos(16_666_667);
        *control_flow = event_loop::ControlFlow::WaitUntil(next_frame_time);

        let delta_frame = current.duration_since(last_frame);
        last_frame = current;

        // 摄像机观察矩阵
        controller.update_camera(&mut camera, delta_frame);
        let view_matrix = Into::<[[f32; 4]; 4]>::into(camera.calc_matrix());

        // drawing a frame
        let mut target = display.draw();
        // target.clear_color(0.2, 0.3, 0.3, 1.0);
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
        
        let mut uniforms = DynamicUniforms::new();
        uniforms.add_str_key("view", &view_matrix);
        uniforms.add_str_key("projection", &projection_matrix);

        let model = Into::<[[f32; 4]; 4]>::into(cube.calc_model());
        uniforms.add_str_key("model", &model);
        
        // 循环渲染模型
        target.draw(&cube.vertex_buffer, &cube.index_buffer, &program, &uniforms, &draw_parameters).unwrap();

        target.finish().unwrap();
    });
}

