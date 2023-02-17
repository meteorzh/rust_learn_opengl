#[macro_use]
extern crate glium;
extern crate cgmath;

use std::{time::{self}, io::Cursor};

#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{index::PrimitiveType, glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState}, window::CursorGrabMode}};

use rust_opengl_learn::camera::{Camera, CameraController};

// 使用了网上找的封装了camera的类

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 初始化顶点缓冲及着色器等资源
    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 3],
            tex: [f32; 2],
        }

        implement_vertex!(Vertex, position, color, tex);

        glium::VertexBuffer::new(&display,
            &[
                // 前
                Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0], tex: [0.0, 0.0] },
                Vertex { position: [-0.5, 0.5, -0.5], color: [0.0, 0.0, 1.0], tex: [0.0, 1.0] },
                Vertex { position: [0.5, 0.5, -0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 1.0] },
                Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 0.0] },
                // 后
                Vertex { position: [0.5, -0.5, 0.5], color: [1.0, 0.0, 0.0], tex: [0.0, 0.0] },
                Vertex { position: [0.5, 0.5, 0.5], color: [1.0, 0.0, 0.0], tex: [0.0, 1.0] },
                Vertex { position: [-0.5, 0.5, 0.5], color: [0.0, 0.0, 1.0], tex: [1.0, 1.0] },
                Vertex { position: [-0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0], tex: [1.0, 0.0] },
                // 左
                Vertex { position: [-0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0], tex: [0.0, 0.0] },
                Vertex { position: [-0.5, 0.5, 0.5], color: [0.0, 0.0, 1.0], tex: [0.0, 1.0] },
                Vertex { position: [-0.5, 0.5, -0.5], color: [0.0, 0.0, 1.0], tex: [1.0, 1.0] },
                Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0], tex: [1.0, 0.0] },
                // 右
                Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0], tex: [0.0, 0.0] },
                Vertex { position: [0.5, 0.5, -0.5], color: [1.0, 0.0, 0.0], tex: [0.0, 1.0] },
                Vertex { position: [0.5, 0.5, 0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 1.0] },
                Vertex { position: [0.5, -0.5, 0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 0.0] },
                // 上
                Vertex { position: [-0.5, 0.5, -0.5], color: [0.0, 0.0, 1.0], tex: [0.0, 0.0] },
                Vertex { position: [-0.5, 0.5, 0.5], color: [0.0, 0.0, 1.0], tex: [0.0, 1.0] },
                Vertex { position: [0.5, 0.5, 0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 1.0] },
                Vertex { position: [0.5, 0.5, -0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 0.0] },
                // 下
                Vertex { position: [-0.5, -0.5, 0.5], color: [0.0, 1.0, 0.0], tex: [0.0, 0.0] },
                Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0], tex: [0.0, 1.0] },
                Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 1.0] },
                Vertex { position: [0.5, -0.5, 0.5], color: [1.0, 0.0, 0.0], tex: [1.0, 0.0] },
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23]).unwrap();

    let image = image::load(Cursor::new(&include_bytes!("../container.jpg")), image::ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let opengl_texture = glium::texture::CompressedSrgbTexture2d::new(&display, image).unwrap();

    let image2 = image::load(Cursor::new(&include_bytes!("../awesomeface.png")), image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions2 = image2.dimensions();
    let image2 = glium::texture::RawImage2d::from_raw_rgba_reversed(&image2.into_raw(), image_dimensions2);
    let opengl_texture2 = glium::texture::CompressedSrgbTexture2d::new(&display, image2).unwrap();

    // compiling shaders and linking them together
    let program = program!(&display,

        330 => {
            vertex: "
                #version 330 core
                
                in vec3 position;
                in vec3 color;
                in vec2 tex;

                out vec3 ourColor;
                out vec2 texCoord;

                uniform mat4 transForm;
                uniform mat4 model;
                uniform mat4 view;
                uniform mat4 projection;
                
                void main()
                {
                    gl_Position = projection * view * model * transForm * vec4(position, 1.0);
                    ourColor = color;
                    texCoord = tex;
                }
            ",
            fragment: "
                #version 330 core

                in vec3 ourColor;
                in vec2 texCoord;

                out vec4 FragColor;

                uniform sampler2D ourTexture;
                uniform sampler2D ourTexture2;
                
                void main() {
                    FragColor = mix(texture(ourTexture, texCoord), texture(ourTexture2, texCoord), 0.2);
                }
            ",
        }
    ).unwrap();

    let mut degree = 0_f32;
    let mut scale = 1_f32;
    let mut scale_step = 0.01_f32;

    let model_matrix = cgmath::Matrix4::from_angle_x(cgmath::Deg(-60.0_f32));
    // let model_matrix = cgmath::Matrix4::<f32>::identity();
    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = 270°;
    let mut camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 3_f32), 
        cgmath::Rad::from(cgmath::Deg(270_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let mut controller = CameraController::new(1_f32, 0.5_f32);
    
    let projection_matrix = cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 100.0);
    // println!("{:#?}", projection_matrix);

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };
    // 多个正方体的位移矩阵
    let unit_trans = {
        let mut s = Vec::<cgmath::Matrix4<f32>>::new();
        let vecs = [
            cgmath::Vector3::new(0.0f32,  0.0,  0.0),
            cgmath::Vector3::new(2.0f32,  5.0, -15.0),
            cgmath::Vector3::new(-1.5f32, -2.2, -2.5),
            cgmath::Vector3::new(-3.8f32, -2.0, -12.3),
            cgmath::Vector3::new(2.4f32, -0.4, -3.5),
            cgmath::Vector3::new(-1.7f32,  3.0, -7.5),
            cgmath::Vector3::new(1.3f32, -2.0, -2.5),
            cgmath::Vector3::new(1.5f32,  2.0, -2.5),
            cgmath::Vector3::new(1.5f32,  0.2, -1.5),
            cgmath::Vector3::new(-1.3f32,  1.0, -1.5)
        ];
        for vec in vecs {
            s.push(cgmath::Matrix4::from_translation(vec));
        }
        s
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

        degree += 0.1;
        if scale >= 1.5_f32 {
            // 开始缩小
            scale_step = -0.01;
        }
        if scale <= 0.3 {
            // 开始放大
            scale_step = 0.01;
        }
        scale += scale_step;

        // 摄像机观察矩阵
        controller.update_camera(&mut camera, delta_frame);
        let view_matrix = camera.calc_matrix();

        let trans = cgmath::Matrix4::from_angle_z(cgmath::Deg(degree));
        // let trans = trans * cgmath::Matrix4::from_scale(scale);

        let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64();
        let green_value = (now.sin() / 2.0 + 0.5) as f32;
        // let t = glium::uniforms::UniformValue::CompressedSrgbTexture2d(opengl_texture, Some(glium::uniforms::SamplerBehavior::default()));

        // drawing a frame
        let mut target = display.draw();
        // target.clear_color(0.2, 0.3, 0.3, 1.0);
        target.clear_color_and_depth((0.2, 0.3, 0.3, 1.0), 1.0);
        // 渲染多个正方体
        for unit_tran in &unit_trans {
            // building the uniforms
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]
                ],
                ourColor: [0.0, green_value, 0.0, 1.0],
                // tex: &opengl_texture
                ourTexture: &opengl_texture,
                ourTexture2: &opengl_texture2,
                transForm: Into::<[[f32; 4]; 4]>::into(trans),
                model: Into::<[[f32; 4]; 4]>::into(*unit_tran * model_matrix),
                view: Into::<[[f32; 4]; 4]>::into(view_matrix),
                projection: Into::<[[f32; 4]; 4]>::into(projection_matrix)
            };
            target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_parameters).unwrap();
        }
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
