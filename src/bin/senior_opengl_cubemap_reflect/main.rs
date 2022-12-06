
extern crate glium;
extern crate cgmath;

use std::{time::{self}};

use cgmath::{SquareMatrix, Point3, Matrix4};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::glutin::{window::CursorGrabMode};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube}, material, create_program, keyboard, load_wavefront_obj_as_models};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24).with_stencil_buffer(8).with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 物体着色器程序
    let obj_program = create_program("src/bin/senior_opengl_cubemap_reflect/box_reflect.vert", "src/bin/senior_opengl_cubemap_reflect/box_reflect.frag", &display);
    // skybox着色器程序
    let skybox_program = create_program("src/bin/senior_opengl_cubemap_reflect/skybox.vert", "src/bin/senior_opengl_cubemap_reflect/skybox.frag", &display);

    let cube = Cube::new("cube1", 1.0, &display, [1.0, 1.0, 1.0], Point3::new(-1.0, 0.0, -1.0), Matrix4::identity());
    let skybox = Cube::new_skybox("skybox", 2.0, &display);
    let models = load_wavefront_obj_as_models(&display, "src/nanosuit/", "nanosuit.obj");

    let skybox_texture = material::load_cubemap("src/skybox/", "jpg", &display, 2048);

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let mut camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 9_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let mut controller = CameraController::new(5_f32, 0.8_f32);
    
    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 1000.0));

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLessOrEqual,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    // let skybox_parameters = glium::DrawParameters {
    //     depth: glium::Depth {
    //         test: glium::draw_parameters::DepthTest::Overwrite,
    //         .. Default::default()
    //     },
    //     .. Default::default()
    // };

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
                    if let Some(cf) = keyboard::handle_keyboard_input(input, &mut controller) {
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

        let camera_position = Into::<[f32; 3]>::into(camera.position);

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
        
        let mut box_uniforms = DynamicUniforms::new();
        box_uniforms.add(String::from("projection"), &projection_matrix);
        box_uniforms.add(String::from("viewPos"), &camera_position);
        box_uniforms.add_str_key("skybox", &skybox_texture);
        
        // 绘制立方体
        let view_matrix = Into::<[[f32; 4]; 4]>::into(camera.calc_matrix());
        box_uniforms.add_str_key("view", &view_matrix);
        let model = Into::<[[f32; 4]; 4]>::into(cube.position_matrix() * cube.model);
        box_uniforms.add_str_key("model", &model);
        target.draw(&cube.vertex_buffer, &cube.index_buffer, &obj_program, &box_uniforms, &draw_parameters).unwrap();
        
        let model_matrix = Into::<[[f32; 4]; 4]>::into(Matrix4::identity());
        // 循环渲染模型
        for model in models.iter() {
            box_uniforms.add_str_key("model", &model_matrix);
            target.draw(&model.vertex_buffer, &model.index_buffer, &obj_program, &box_uniforms, &draw_parameters).unwrap();
        }

        box_uniforms.remove("model");

        // 绘制skybox
        // 绘制skybox时需要移除观察矩阵的位移性质，使摄像机无论如何移动，始终在天空盒内
        let view_matrix = Into::<[[f32; 4]; 4]>::into(camera.calc_matrix_no_move());
        box_uniforms.add_str_key("view", &view_matrix);
        target.draw(&skybox.vertex_buffer, &skybox.index_buffer, &skybox_program, &box_uniforms, &draw_parameters).unwrap();

        target.finish().unwrap();
    });
}
