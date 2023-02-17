#[macro_use]
extern crate glium;
extern crate cgmath;

use std::io::Cursor;
use std::time;
use cgmath::{Matrix4, SquareMatrix};
use glium::glutin::window::CursorGrabMode;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction};
use rust_opengl_learn::camera::{Camera, CameraController};
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{KeyboardInput, VirtualKeyCode, ElementState};
use glium::{Surface};
use glium::glutin::{self, event_loop, window, event};
use image::ImageFormat;

// cubemap示例，从网上摘下来，可以运行了，但是图片不连贯
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::new(800_u32, 600_u32);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24).with_stencil_buffer(8);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 右
    let image = image::load(Cursor::new(&include_bytes!("../../skybox/right.jpg")[..]),
                        ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let tex_posx = glium::Texture2d::new(&display, image).unwrap();

    // 左
    let image = image::load(Cursor::new(&include_bytes!("../../skybox/left.jpg")[..]),
                        ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let tex_negx = glium::Texture2d::new(&display, image).unwrap();

    // 上
    let image = image::load(Cursor::new(&include_bytes!("../../skybox/top.jpg")[..]),
                        ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let tex_posy = glium::Texture2d::new(&display, image).unwrap();

    // 下
    let image = image::load(Cursor::new(&include_bytes!("../../skybox/bottom.jpg")[..]),
                        ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let tex_negy = glium::Texture2d::new(&display, image).unwrap();

    // 后
    let image = image::load(Cursor::new(&include_bytes!("../../skybox/front.jpg")[..]),
                        ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let tex_posz = glium::Texture2d::new(&display, image).unwrap();

    // 前
    let image = image::load(Cursor::new(&include_bytes!("../../skybox/back.jpg")[..]),
                        ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions);
    let tex_negz = glium::Texture2d::new(&display, image).unwrap();

    let cubemap = glium::texture::Cubemap::empty(&display, 512).unwrap();

    // skybox
    let skybox_vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
        }

        implement_vertex!(Vertex, position);

        let side2: f32 = 50.0 / 2.0;

        glium::VertexBuffer::new(&display,
            &[
                // Front
    		Vertex { position: [-side2, -side2,  side2] },
    		Vertex { position: [ side2, -side2,  side2] },
    		Vertex { position: [ side2,  side2,  side2] },
                Vertex { position: [-side2,  side2,  side2] },
    		// Right
    		Vertex { position: [ side2, -side2,  side2] },
    		Vertex { position: [ side2, -side2, -side2] },
    		Vertex { position: [ side2,  side2, -side2] },
                Vertex { position: [ side2,  side2,  side2] },
    		// Back
    		Vertex { position: [-side2, -side2, -side2] },
    		Vertex { position: [-side2,  side2, -side2] },
    		Vertex { position: [ side2,  side2, -side2] },
                Vertex { position: [ side2, -side2, -side2] },
    		// Left
    		Vertex { position: [-side2, -side2,  side2] },
    		Vertex { position: [-side2,  side2,  side2] },
                Vertex { position: [-side2,  side2, -side2] },
                Vertex { position: [-side2, -side2, -side2] },
                // Bottom
    		Vertex { position: [-side2, -side2,  side2] },
    		Vertex { position: [-side2, -side2, -side2] },
    		Vertex { position: [ side2, -side2, -side2] },
                Vertex { position: [ side2, -side2,  side2] },
    		// Top
                Vertex { position: [-side2,  side2,  side2] },
    		Vertex { position: [ side2,  side2,  side2] },
    		Vertex { position: [ side2,  side2, -side2] },
                Vertex { position: [-side2,  side2, -side2] },
    	    ]
        ).unwrap()
    };

    let skybox_index_buffer = glium::IndexBuffer::new(&display,
            glium::index::PrimitiveType::TrianglesList,
            &[
                // Front
                0u16, 2, 1, 0, 3, 2,
                // Right
                4, 6, 5, 4, 7, 6,
                // Back
                8, 10, 9, 8, 11, 10,
                // Left
                12, 14, 13, 12, 15, 14,
                // Bottom
                16, 18, 17, 16, 19, 18,
                // Top
                20, 22, 21, 20, 23, 22,
            ]).unwrap();

    let skybox_program = glium::Program::from_source(&display,
        " #version 140

            in vec3 position;
            out vec3 ReflectDir;

            uniform mat4 model;
            uniform mat4 view;
            uniform mat4 perspective;

            void main() {
                ReflectDir = position;
                gl_Position = perspective * view * model * vec4(position, 1.0);
            }
        ",
        " #version 140
            in vec3 ReflectDir;
            out vec4 color;

            uniform samplerCube cubetex;

            void main() {
                color = texture(cubetex, ReflectDir);
            }
        ",
        None).unwrap();

    //model
    let model_vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            normal:  [f32; 3],
        }

        implement_vertex!(Vertex, position, normal);

        let side2: f32 = 2.0 / 2.0;

        glium::VertexBuffer::new(&display,
            &[
                // Front
    		Vertex { position: [-side2, -side2,  side2], normal: [ 0.0,  0.0,  1.0] },
    		Vertex { position: [ side2, -side2,  side2], normal: [ 0.0,  0.0,  1.0] },
    		Vertex { position: [ side2,  side2,  side2], normal: [ 0.0,  0.0,  1.0] },
                Vertex { position: [-side2,  side2,  side2], normal: [ 0.0,  0.0,  1.0] },
    		// Right
    		Vertex { position: [ side2, -side2,  side2], normal: [ 1.0,  0.0,  0.0] },
    		Vertex { position: [ side2, -side2, -side2], normal: [ 1.0,  0.0,  0.0] },
    		Vertex { position: [ side2,  side2, -side2], normal: [ 1.0,  0.0,  0.0] },
                Vertex { position: [ side2,  side2,  side2], normal: [ 1.0,  0.0,  0.0] },
    		// Back
    		Vertex { position: [-side2, -side2, -side2], normal: [ 0.0,  0.0, -1.0] },
    		Vertex { position: [-side2,  side2, -side2], normal: [ 0.0,  0.0, -1.0] },
    		Vertex { position: [ side2,  side2, -side2], normal: [ 0.0,  0.0, -1.0] },
                Vertex { position: [ side2, -side2, -side2], normal: [ 0.0,  0.0, -1.0] },
    		// Left
    		Vertex { position: [-side2, -side2,  side2], normal: [-1.0,  0.0,  0.0] },
    		Vertex { position: [-side2,  side2,  side2], normal: [-1.0,  0.0,  0.0] },
                Vertex { position: [-side2,  side2, -side2], normal: [-1.0,  0.0,  0.0] },
                Vertex { position: [-side2, -side2, -side2], normal: [-1.0,  0.0,  0.0] },
                // Bottom
    		Vertex { position: [-side2, -side2,  side2], normal: [ 0.0, -1.0,  0.0] },
    		Vertex { position: [-side2, -side2, -side2], normal: [ 0.0, -1.0,  0.0] },
    		Vertex { position: [ side2, -side2, -side2], normal: [ 0.0, -1.0,  0.0] },
                Vertex { position: [ side2, -side2,  side2], normal: [ 0.0, -1.0,  0.0] },
    		// Top
                Vertex { position: [-side2,  side2,  side2], normal: [ 0.0,  1.0,  0.0] },
    		Vertex { position: [ side2,  side2,  side2], normal: [ 0.0,  1.0,  0.0] },
    		Vertex { position: [ side2,  side2, -side2], normal: [ 0.0,  1.0,  0.0] },
                Vertex { position: [-side2,  side2, -side2], normal: [ 0.0,  1.0,  0.0] },
    		]
        ).unwrap()
    };

    let model_index_buffer = glium::IndexBuffer::new(&display,
            glium::index::PrimitiveType::TrianglesList,
            &[
                // Front
                0u16, 2, 1, 0, 3, 2,
                // Right
                4, 6, 5, 4, 7, 6,
                // Back
                8, 10, 9, 8, 11, 10,
                // Left
                12, 14, 13, 12, 15, 14,
                // Bottom
                16, 18, 17, 16, 19, 18,
                // Top
                20, 22, 21, 20, 23, 22,
            ]).unwrap();

    let model_program = glium::Program::from_source(&display,
        " #version 140

            in vec3 position;
            in vec3 normal;
            out vec4 v_position;
            out vec3 v_normal;

            uniform mat4 model;
            uniform mat4 view;
            uniform mat4 perspective;

            void main() {
                mat4 modelviewMatrix = view * model;
                mat3 normalMatrix = mat3(modelviewMatrix);

                v_position = modelviewMatrix * vec4(position, 1.0);
                v_normal = normalMatrix * normal;
                gl_Position = perspective * v_position;
            }
        ",
        " #version 140
            in vec4 v_position;
            in vec3 v_normal;
            out vec4 f_color;

            uniform samplerCube cubetex;
            uniform float ReflectFactor;
            uniform vec4 MaterialColor;
            uniform vec3 WorldCameraPosition;

            void main() {
                vec3 s = normalize(v_normal);
                vec3 v = normalize(WorldCameraPosition - v_position.xyz);
                vec3 ReflectDir = reflect(v, s);
                vec4 cubeMapColor = texture(cubetex, ReflectDir);
                f_color = mix(MaterialColor, cubeMapColor, ReflectFactor);
            }
        ",
        None).unwrap();

    let dest_rect1 = glium::BlitTarget {
        left: 0,
        bottom: 0,
        width: 512,
        height: 512,
    };

    let mut camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 0_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let mut controller = CameraController::new(1_f32, 0.5_f32);
    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::perspective(cgmath::Deg(45.0), 800 as f32 / 600 as f32, 0.1_f32, 100.0));

    let mut last_frame = time::Instant::now();
    // main loop
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

        controller.update_camera(&mut camera, delta_frame);

        let  framebuffer1 = glium::framebuffer::SimpleFrameBuffer::new(&display,
                        cubemap.main_level().image(glium::texture::CubeLayer::PositiveX)).unwrap();
        let  framebuffer2 = glium::framebuffer::SimpleFrameBuffer::new(&display,
                        cubemap.main_level().image(glium::texture::CubeLayer::NegativeX)).unwrap();
        let  framebuffer3 = glium::framebuffer::SimpleFrameBuffer::new(&display,
                        cubemap.main_level().image(glium::texture::CubeLayer::PositiveY)).unwrap();
        let  framebuffer4 = glium::framebuffer::SimpleFrameBuffer::new(&display,
                        cubemap.main_level().image(glium::texture::CubeLayer::NegativeY)).unwrap();
        let  framebuffer5 = glium::framebuffer::SimpleFrameBuffer::new(&display,
                        cubemap.main_level().image(glium::texture::CubeLayer::PositiveZ)).unwrap();
        let  framebuffer6 = glium::framebuffer::SimpleFrameBuffer::new(&display,
                        cubemap.main_level().image(glium::texture::CubeLayer::NegativeZ)).unwrap();

        tex_posx.as_surface().blit_whole_color_to(&framebuffer1, &dest_rect1,
                        glium::uniforms::MagnifySamplerFilter::Linear);
        tex_negx.as_surface().blit_whole_color_to(&framebuffer2, &dest_rect1,
                        glium::uniforms::MagnifySamplerFilter::Linear);
        tex_posy.as_surface().blit_whole_color_to(&framebuffer3, &dest_rect1,
                        glium::uniforms::MagnifySamplerFilter::Linear);
        tex_negy.as_surface().blit_whole_color_to(&framebuffer4, &dest_rect1,
                        glium::uniforms::MagnifySamplerFilter::Linear);
        tex_posz.as_surface().blit_whole_color_to(&framebuffer5, &dest_rect1,
                        glium::uniforms::MagnifySamplerFilter::Linear);
        tex_negz.as_surface().blit_whole_color_to(&framebuffer6, &dest_rect1,
                        glium::uniforms::MagnifySamplerFilter::Linear);

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let model = Into::<[[f32; 4]; 4]>::into(Matrix4::identity());

        let camera_position: [f32; 3]= [0.0, 0.0, -8.0];
        let view = Into::<[[f32; 4]; 4]>::into(camera.calc_matrix());
        let perspective = projection_matrix;

        let material_color: [f32; 4] = [0.9, 0.9, 0.9, 1.0];
        let reflect_factor: f32 = 0.9;

        let skybox_texture = cubemap.sampled().magnify_filter(MagnifySamplerFilter::Linear)
            .minify_filter(MinifySamplerFilter::Linear).wrap_function(SamplerWrapFunction::Clamp);

        let skybox_uniforms = uniform! {
            model: model,
            view: view,
            perspective: perspective,
	        cubetex: skybox_texture,
        };

        let model_uniforms = uniform! {
    	     model: model,
             view: view,
             perspective: perspective,
             cubetex: cubemap.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
             ReflectFactor: reflect_factor,
             MaterialColor: material_color,
             WorldCameraPosition: camera_position,
    	};

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        target.draw(&skybox_vertex_buffer, &skybox_index_buffer, &skybox_program,
                    &skybox_uniforms, &params).unwrap();
        // target.draw(&model_vertex_buffer, &model_index_buffer, &model_program,
        //             &model_uniforms, &params).unwrap();

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
