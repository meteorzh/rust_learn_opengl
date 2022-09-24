#[macro_use]
extern crate glium;

use std::time;

#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{index::PrimitiveType, glutin::event::{KeyboardInput, VirtualKeyCode, ElementState}, draw_parameters};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let wb = window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // the main loop
    event_loop.run(move |event, _, control_flow| {
        render_triangle(&display);

        *control_flow = match event {
            event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                event::WindowEvent::CloseRequested => event_loop::ControlFlow::Exit,
                // Redraw the triangle when the window is resized.
                event::WindowEvent::Resized(..) => {
                    render_triangle(&display);
                    // render_rectangle(&display, false); // line mode
                    // render_rectangle(&display, true); // line mode
                    event_loop::ControlFlow::Poll
                },
                // key input
                event::WindowEvent::KeyboardInput { input, .. } => {
                    handle_keyboard_input(input)
                },
                _ => event_loop::ControlFlow::Poll,
            },
            _ => event_loop::ControlFlow::Poll,
        };
    });
}

fn handle_keyboard_input(input: KeyboardInput) -> event_loop::ControlFlow {    
    let virtual_keycode = input.virtual_keycode;
    if let None = virtual_keycode {
        return event_loop::ControlFlow::Poll;
    }

    let virtual_keycode = virtual_keycode.unwrap();
    match virtual_keycode {
        VirtualKeyCode::Escape => {
            if input.state == ElementState::Released {
                return event_loop::ControlFlow::Exit;
            }
            event_loop::ControlFlow::Poll
        },
        _ => {
            println!("unsupported keyboard input: {}", input.scancode);
            event_loop::ControlFlow::Poll
        }
    }
}

fn render_triangle(display: &glium::Display) {
    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(display,
            &[
                Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0,  0.5, 0.0], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2]).unwrap();

    // compiling shaders and linking them together
    let program = program!(display,
        // 110 => {
        //     vertex: "
        //         #version 110

        //         uniform mat4 matrix;

        //         attribute vec2 position;
        //         attribute vec3 color;

        //         varying vec3 vColor;

        //         void main() {
        //             gl_Position = vec4(position, 0.0, 1.0) * matrix;
        //             vColor = color;
        //         }
        //     ",

        //     fragment: "
        //         #version 110
        //         varying vec3 vColor;

        //         void main() {
        //             gl_FragColor = vec4(vColor, 1.0);
        //         }
        //     ",
        // },

        // 100 => {
        //     vertex: "
        //         #version 100

        //         uniform lowp mat4 matrix;

        //         attribute lowp vec2 position;
        //         attribute lowp vec3 color;

        //         varying lowp vec3 vColor;

        //         void main() {
        //             gl_Position = vec4(position, 0.0, 1.0) * matrix;
        //             vColor = color;
        //         }
        //     ",

        //     fragment: "
        //         #version 100
        //         varying lowp vec3 vColor;

        //         void main() {
        //             gl_FragColor = vec4(vColor, 1.0);
        //         }
        //     ",
        // },

        330 => {
            vertex: "
                #version 330 core
                
                in vec3 position;
                in vec3 color;

                out vec3 ourColor;
                
                void main()
                {
                    gl_Position = vec4(position, 1.0);
                    ourColor = color;
                }
            ",
            fragment: "
                #version 330 core

                in vec3 ourColor;

                out vec4 FragColor;
                
                void main() {
                    FragColor = vec4(ourColor, 1.0);
                }
            ",
        }
    ).unwrap();

    // Here we draw the black background and triangle to the screen using the previously
    // initialised resources.
    //
    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = move || {
        let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs_f64();
        let green_value = (now.sin() / 2.0 + 0.5) as f32;
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]
            ],
            ourColor: [0.0, green_value, 0.0, 1.0]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.2, 0.3, 0.3, 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    };

    // Draw the triangle to the screen.
    draw();
}

fn render_rectangle(display: &glium::Display, line_mode: bool) {
    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(display,
            &[
                Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 1.0, 0.0] },
                Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
                Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2, 0, 2, 3]).unwrap();

    // compiling shaders and linking them together
    let program = program!(display,

        330 => {
            vertex: "
                #version 330 core
                
                in vec3 position;
                in vec3 color;
                
                void main()
                {
                    gl_Position = vec4(position.x, position.y, position.z, 1.0);
                }
            ",
            fragment: "
                #version 330 core
                out vec4 FragColor;
                
                void main() {
                    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
                }
            ",
        }
    ).unwrap();

    // Here we draw the black background and triangle to the screen using the previously
    // initialised resources.
    //
    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = move || {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.2, 0.3, 0.3, 1.0);

        let mut draw_parameters = glium::DrawParameters::default();
        if line_mode {
            draw_parameters.polygon_mode = glium::PolygonMode::Line;
        }

        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_parameters).unwrap();
        target.finish().unwrap();
    };

    // Draw the triangle to the screen.
    draw();
}