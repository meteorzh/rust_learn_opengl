// learn open gl 光照学习

#[macro_use]
extern crate glium;
extern crate cgmath;

use std::{time::{self}, sync::Mutex, collections::HashMap};

use cgmath::{SquareMatrix, Point3, Matrix4, EuclideanSpace, Vector3};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{index::PrimitiveType, glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState}, window::CursorGrabMode}, VertexBuffer, IndexBuffer, Program, uniforms::{UniformsStorage, EmptyUniforms, AsUniformValue}};

use rust_opengl_learn::camera::{Camera, CameraController};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let box_program = program!(&display,
        330 => {
            vertex: "
                #version 330 core
                
                in vec3 position;
                in vec3 normal;

                out vec3 oNormal;
                out vec3 fragPos;
                
                uniform mat4 model;
                uniform mat4 view;
                uniform mat4 projection;
                
                void main()
                {
                    gl_Position = projection * view * model * vec4(position, 1.0);
                    fragPos = vec3(model * vec4(position, 1.0));
                    oNormal = mat3(transpose(inverse(model))) * normal;
                }
            ",
            fragment: "
                #version 330 core

                in vec3 oNormal;
                in vec3 fragPos;
        
                out vec4 FragColor;
        
                uniform vec3 color;
                uniform vec3 lightColor;
                uniform vec3 lightPos;
                uniform vec3 viewPos;
                uniform vec3 ambient;
                uniform vec3 diffuse;
                uniform vec3 specular;
                uniform float shininess;
                
                void main() {
                    vec3 ambient = vec3(0.1) * ambient;

                    vec3 norm = normalize(oNormal);
                    vec3 lightDir = normalize(lightPos - fragPos);
                    float diff = max(dot(norm, lightDir), 0.0);
                    vec3 diffuse = vec3(1.0) * (diff * diffuse);

                    vec3 viewDir = normalize(viewPos - fragPos);
                    vec3 reflectDir = reflect(-lightDir, norm);
                    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
                    vec3 specular = vec3(1.0) * (spec * specular);

                    vec3 result = (ambient + diffuse + specular) * color;
        
                    FragColor = vec4(result, 1.0);
                }
            ",
        }
    ).unwrap();

    let light_program = program!(&display,
        330 => {
            vertex: "
                #version 330 core
                
                in vec3 position;
                
                uniform mat4 model;
                uniform mat4 view;
                uniform mat4 projection;
                
                void main()
                {
                    gl_Position = projection * view * model * vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 330 core

                out vec4 FragColor;
        
                uniform vec3 color;
                
                void main() {
                    FragColor = vec4(color, 1.0);
                }
            ",
        }
    ).unwrap();

    // 初始化顶点缓冲及着色器等资源
    // building the vertex buffer, which contains all the vertices that we will draw
    let box_cube = Cube::new("box", 0.8_f32, &display, [1.0f32, 0.5, 0.31], 
        box_program, Point3::new(0_f32, 0.0, 0.0), Matrix4::<f32>::identity(), 
        Material::new([1.0f32, 0.5, 0.31], [1.0f32, 0.5, 0.31], [0.5f32, 0.5, 0.5], 32.0_f32));
    let light_position = Point3::new(4_f32, 4.0, -8.0);
    let light_model = Matrix4::<f32>::from_scale(0.2_f32);
    let light_color = [1.0f32, 2.0, 1.0];
    let light_cube = Cube::new("light", 0.5_f32, &display, light_color, 
        light_program, light_position, light_model, Material::empty());

    let cubes = vec![box_cube, light_cube];

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = 270°;
    let mut camera = Camera::new(
        cgmath::Point3::new(3_f32, 2_f32, 3_f32), 
        cgmath::Rad::from(cgmath::Deg(232_f32)), 
        cgmath::Rad::from(cgmath::Deg(-18_f32))
    );
    let mut controller = CameraController::new(1_f32, 0.5_f32);
    
    let projection_matrix = cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 100.0);

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
        let view_matrix = camera.calc_matrix();

        // drawing a frame
        let mut target = display.draw();
        // target.clear_color(0.2, 0.3, 0.3, 1.0);
        target.clear_color_and_depth((0.1, 0.1, 0.1, 0.0), 1.0);
        
        // 渲染多个正方体
        for cube in &cubes {
            // building the uniforms
            let uniforms = uniform! {
                model: Into::<[[f32; 4]; 4]>::into(cube.model * Matrix4::from_translation(cube.position.to_vec())),
                view: Into::<[[f32; 4]; 4]>::into(view_matrix),
                projection: Into::<[[f32; 4]; 4]>::into(projection_matrix),
                color: cube.color,
                lightColor: light_color,
                lightPos: Into::<[f32; 3]>::into(light_position),
                viewPos: Into::<[f32; 3]>::into(camera.position),
                ambient: cube.material.ambient,
                diffuse: cube.material.diffuse,
                specular: cube.material.specular,
                shininess: cube.material.shininess,
            };
            target.draw(&cube.vertex_buffer, &cube.index_buffer, &cube.program, &uniforms, &draw_parameters).unwrap();
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




#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

implement_vertex!(Vertex, position, normal);

static INDEX_ARRAY: [u16; 36] = [0u16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35];


/**
 * 36个顶点的简单正方体
 */
pub struct Cube {
    id: String,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    color: [f32; 3],
    program: Program,
    position: Point3<f32>,
    model: Matrix4<f32>,
    material: Material,
}

impl Cube {
    /**
     * 边长，0-1，标准化设备坐标系范围内
     */
    pub fn new(id: &str, side_len: f32, display: &glium::Display, color: [f32; 3], program: Program, position: Point3<f32>, model: Matrix4<f32>, material: Material) -> Cube {
        let half = side_len / 2_f32;
        Cube {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // 前
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, 1_f32] },
                Vertex { position: [-half, half, half], normal: [0_f32, 0_f32, 1_f32] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, 1_f32] },
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, 1_f32] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, 1_f32] },
                Vertex { position: [half, -half, half], normal: [0_f32, 0_f32, 1_f32] },
                // 后
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 0_f32, -1_f32] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32] },
                Vertex { position: [half, -half, -half], normal: [0_f32, 0_f32, -1_f32] },
                // 左
                Vertex { position: [-half, -half, -half], normal: [-1_f32, 0_f32, 0_f32] },
                Vertex { position: [-half, half, -half], normal: [-1_f32, 0_f32, 0_f32] },
                Vertex { position: [-half, half, half], normal: [-1_f32, 0_f32, 0_f32] },
                Vertex { position: [-half, -half, -half], normal: [-1_f32, 0_f32, 0_f32] },
                Vertex { position: [-half, half, half], normal: [-1_f32, 0_f32, 0_f32] },
                Vertex { position: [-half, -half, half], normal: [-1_f32, 0_f32, 0_f32] },
                // 右
                Vertex { position: [half, -half, half], normal: [1_f32, 0_f32, 0_f32] },
                Vertex { position: [half, half, half], normal: [1_f32, 0_f32, 0_f32] },
                Vertex { position: [half, half, -half], normal: [1_f32, 0_f32, 0_f32] },
                Vertex { position: [half, -half, half], normal: [1_f32, 0_f32, 0_f32] },
                Vertex { position: [half, half, -half], normal: [1_f32, 0_f32, 0_f32] },
                Vertex { position: [half, -half, -half], normal: [1_f32, 0_f32, 0_f32] },
                // 上
                Vertex { position: [-half, half, half], normal: [0_f32, 1_f32, 0_f32] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 1_f32, 0_f32] },
                Vertex { position: [half, half, -half], normal: [0_f32, 1_f32, 0_f32] },
                Vertex { position: [-half, half, half], normal: [0_f32, 1_f32, 0_f32] },
                Vertex { position: [half, half, -half], normal: [0_f32, 1_f32, 0_f32] },
                Vertex { position: [half, half, half], normal: [0_f32, 1_f32, 0_f32] },
                // 下
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, -1_f32, 0_f32] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32] },
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32] },
                Vertex { position: [half, -half, half], normal: [0_f32, -1_f32, 0_f32] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDEX_ARRAY).unwrap(),
            color: color,
            program: program,
            position: position,
            model: model,
            material: material,
        }
    }
}


pub struct Material {
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
    shininess: f32,
}

impl Material {
    pub fn new(ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3], shininess: f32) -> Material {
        Material {
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
            shininess: shininess
        }
    }

    pub fn empty() -> Material {
        Material::new([0_f32; 3], [0_f32; 3], [0_f32; 3], 0_f32)
    }
}

impl AsUniformValue for Material {

    fn as_uniform_value(&self) -> glium::uniforms::UniformValue<'_> {
        // glium::uniforms::UniformValue::Block((), ())
        todo!()
    }
}