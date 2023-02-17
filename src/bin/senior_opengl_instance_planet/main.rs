
extern crate glium;
extern crate cgmath;
extern crate num_traits;

use std::{time::{self, SystemTime}};

use cgmath::{Matrix4, SquareMatrix, Vector3, Deg};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{draw_parameters::{Depth}, glutin::{window::CursorGrabMode}, PolygonMode, uniforms::UniformValue, DepthTest, VertexBuffer};

use rand::{rngs::StdRng, SeedableRng, Rng};
use rust_opengl_learn::{camera::{CameraController, Camera}, uniforms::DynamicUniforms, keyboard, create_program_vgf, load_wavefront_obj_as_models, create_program, objectsv2::RawInstanceDataM4};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24).with_stencil_buffer(8).with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 随机数生成器
    let mut rng = StdRng::seed_from_u64(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64);

    // 物体着色器程序
    let planet_program = create_program(
        "src/bin/senior_opengl_instance_planet/planet.vert", 
        "src/bin/senior_opengl_instance_planet/planet.frag", 
        &display);

    let asteroid_program = create_program(
        "src/bin/senior_opengl_instance_planet/asteroids.vert", 
        "src/bin/senior_opengl_instance_planet/planet.frag", 
        &display);

    let models = load_wavefront_obj_as_models(&display, "src/planet/", "planet.obj");
    let rocks = load_wavefront_obj_as_models(&display, "src/rock/", "rock.obj");

    // 初始化1000个小行星的位置
    let instances = {
        let voffset = 5_f32;
        let hoffset = 20.0_f32;
        let radius = 60_f32;
        let amount = 20000;
        let mut rock_models = Vec::with_capacity(amount);
        let rotate_dir = Vector3::new(0.4_f32, 0.6, 0.8);
        for i in 0..amount {
            // 1. 位移：分布在半径为 'radius' 的圆形上，偏移的范围是 [-offset, offset]
            let angle = i as f32 / amount as f32 * 360.0;
            let displacement: f32 = rng.gen_range(-hoffset..=hoffset);
            let x = angle.sin() * radius + displacement;
            let displacement: f32 = rng.gen_range(-voffset..=voffset);
            let y = displacement * 0.4;
            let displacement: f32 = rng.gen_range(-hoffset..=hoffset);
            let z = angle.cos() * radius + displacement;
            let translate = Matrix4::from_translation(Vector3::new(x, y, z));

            // 2. 缩放：在 0.05 和 0.25f 之间缩放
            let scale = Matrix4::from_scale(rng.gen_range(0.05..=0.25));

            // 3. 旋转：绕着一个（半）随机选择的旋转轴向量进行随机的旋转
            let rotate = Matrix4::from_axis_angle(rotate_dir, Deg(rng.gen_range(0..360) as f32));
            rock_models.push(RawInstanceDataM4 { model: Into::<[[f32; 4]; 4]>::into(translate * rotate * scale) });
        }
        VertexBuffer::new(&display, &rock_models).unwrap()
    };

    let mut camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 9_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let mut controller = CameraController::new(5_f32, 0.8_f32);
    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 1000.0));

    let draw_parameters = glium::DrawParameters {
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Default::default()
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
            // *control_flow = event_loop::ControlFlow::
            return;
        }
        
        let current = time::Instant::now();
        let next_frame_time = current + time::Duration::from_nanos(16_666_667);
        *control_flow = event_loop::ControlFlow::WaitUntil(next_frame_time);

        let delta_frame = current.duration_since(last_frame);
        last_frame = current;

        controller.update_camera(&mut camera, delta_frame);
        let camera_position = Into::<[f32; 3]>::into(camera.position);

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);

        let mut uniforms = DynamicUniforms::new();
        uniforms.add_str_key("projection", &projection_matrix);
        uniforms.add_str_key("viewPos", &camera_position);

        let view_matrix = Into::<[[f32; 4]; 4]>::into(camera.calc_matrix());
        uniforms.add_str_key("view", &view_matrix);
        
        let model_matrix = Into::<[[f32; 4]; 4]>::into(Matrix4::identity());
        uniforms.add_str_key("model", &model_matrix);
        // 循环渲染模型
        for model in models.iter() {
            if let Some(material) = &model.material {
                if let Some(diffuse_map) = &material.diffuse_map {
                    uniforms.add_str_key("texture_diffuse", diffuse_map.as_ref());
                }
            }
            target.draw(&model.vertex_buffer, &model.index_buffer, &planet_program, &uniforms, &draw_parameters).unwrap();
        }
        uniforms.remove("model");

        // 绘制小行星带
        for model in rocks.iter() {
            if let Some(material) = &model.material {
                if let Some(diffuse_map) = &material.diffuse_map {
                    uniforms.add_str_key("texture_diffuse", diffuse_map.as_ref());
                }
            }
            target.draw((&model.vertex_buffer, instances.per_instance().unwrap()), &model.index_buffer, &asteroid_program, &uniforms, &draw_parameters).unwrap();
        }
        
        target.finish().unwrap();
    });
}
