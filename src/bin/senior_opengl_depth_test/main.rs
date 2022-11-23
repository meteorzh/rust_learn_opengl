
extern crate glium;
extern crate cgmath;

use std::{time::{self}};

use cgmath::{SquareMatrix, Point3, Matrix4};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState}, window::CursorGrabMode}};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube, Plane}, material, create_program};

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
    let mut camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 7_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
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
        // render_triangle(&display);

        // println!("{:#?}", event);
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

        let camera_position = Into::<[f32; 3]>::into(camera.position);

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

        let model = Into::<[[f32; 4]; 4]>::into(plane.calc_model());
        box_uniforms.add_str_key("model", &model);
        box_uniforms.add_str_key("texture1", &floor_texture);
        target.draw(&plane.vertex_buffer, &plane.index_buffer, &obj_program, &box_uniforms, &draw_parameters).unwrap();

        target.finish().unwrap();
    });
}

fn handle_keyboard_input(input: KeyboardInput, camera_controller: &mut CameraController) -> Option<event_loop::ControlFlow> {
    let virtual_keycode = input.virtual_keycode;
    if let None = virtual_keycode {
        return None;
    }

    let virtual_keycode = virtual_keycode.unwrap();
    let camera_handle = camera_controller.process_keyboard(virtual_keycode, input.state);
    if camera_handle {
        return None;
    }

    match virtual_keycode {
        VirtualKeyCode::Escape => {
            if input.state == ElementState::Released {
                return Some(event_loop::ControlFlow::Exit);
            }
        },
        _ => {
            println!("unsupported keyboard input: {}", input.scancode);
        }
    }

    None
}

