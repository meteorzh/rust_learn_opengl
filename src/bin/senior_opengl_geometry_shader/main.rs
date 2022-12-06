
extern crate glium;
extern crate cgmath;

use std::{time::{self}};

use cgmath::{SquareMatrix, Point3, Matrix4, Matrix3};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{window::CursorGrabMode}, PolygonMode, index::PrimitiveType};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube, Plane}, material, create_program, keyboard, create_program_vgf};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24).with_stencil_buffer(8).with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 物体着色器程序
    let program = create_program_vgf(
        "src/bin/senior_opengl_geometry_shader/geometry.vert", 
        "src/bin/senior_opengl_geometry_shader/geometry_house.geom", 
        "src/bin/senior_opengl_geometry_shader/geometry.frag", 
        &display);
    // let program = create_program(
    //     "src/bin/senior_opengl_geometry_shader/geometry.vert", 
    //     "src/bin/senior_opengl_geometry_shader/geometry.frag", 
    //     &display);

    let plane = Plane::new_vertical_center_plane("plane", 1.0, 1.0, &display, PrimitiveType::Points);

    let mut controller = CameraController::new(5_f32, 0.8_f32);

    let draw_parameters = glium::DrawParameters {
        polygon_mode: PolygonMode::Line,
        .. Default::default()
    };

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
                    if let Some(cf) = keyboard::handle_keyboard_input(input, &mut controller) {
                        *control_flow = cf;
                    }
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

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);

        let uniforms = DynamicUniforms::new();
        
        target.draw(&plane.vertex_buffer, &plane.index_buffer, &program, &uniforms, &draw_parameters).unwrap();
        

        target.finish().unwrap();
    });
}
