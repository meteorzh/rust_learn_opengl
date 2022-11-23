// learn open gl 光照-多光源结合

#[macro_use]
extern crate glium;
extern crate cgmath;

use std::{time::{self}, fs::{self}, io};

use cgmath::{SquareMatrix, Point3, Matrix4, EuclideanSpace, Angle, Vector3, Zero};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{index::PrimitiveType, glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState}, window::CursorGrabMode}, VertexBuffer, IndexBuffer, program::{ProgramCreationInput, SourceCode}, Display, Program, vertex::VertexBufferAny, PolygonMode};

use rust_opengl_learn::{camera::{Camera, CameraController}, lights::{DirLight, PointLight, SpotLight}, uniforms::DynamicUniforms, utils::load_wavefront, Vertex, load_wavefront_obj_as_models};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    // 物体着色器程序
    let obj_program = create_program("src/bin/load_model/obj_shader.vert", "src/bin/load_model/obj_shader.frag", &display);

    // 光源着色器程序
    let light_program = create_program("src/bin/load_model/light_shader.vert", "src/bin/load_model/light_shader.frag", &display);

    let models = load_wavefront_obj_as_models(&display, "src/nanosuit/", "nanosuit.obj");

    // 定向光
    let dir_light = DirLight::new(
        [-0.2_f32, -1.0, -0.3],
        [0.05_f32, 0.05, 0.05],
        [0.4_f32, 0.4, 0.4],
        [0.5_f32, 0.5, 0.5]
    );

    // 点光源
    let (point_light_boxes, point_lights) = {
        let positions = [
            [0_f32, 10.0, 7.0],
            // [2.3_f32, -3.3, -4.0],
            // [-4.0_f32, 2.0, -12.0],
            // [0.0_f32, 0.0, -3.0]
        ];

        let mut light_boxes = Vec::<Cube>::with_capacity(4);
        let mut point_lights = Vec::<PointLight>::with_capacity(4);
        let light_color = [1.0_f32, 1.0, 1.0];
        for position in positions {
            light_boxes.push(Cube::new("light", 0.7_f32, &display, light_color, Point3::from(position), Matrix4::<f32>::identity()));
            point_lights.push(PointLight::new(
                position, 
                1.0_f32, 
                0.09_f32, 
                0.032_f32, 
                [0.05_f32, 0.05, 0.05], 
                [0.8_f32, 0.8, 0.8], 
                [1.0_f32, 1.0, 1.0]
            ));
        }

        (light_boxes, point_lights)
    };

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let mut camera = Camera::new(
        cgmath::Point3::new(0_f32, 13_f32, 10_f32), 
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
        polygon_mode: PolygonMode::Fill,
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

        // 聚光灯
        let spot_light = SpotLight::new(
            camera_position,
            Into::<[f32; 3]>::into(camera.direction()),
            cgmath::Deg(12.5_f32).cos(),
            cgmath::Deg(15.0_f32).cos(),
            1.0_f32,
            0.09_f32,
            0.032_f32,
            [0.0_f32, 0.0, 0.0],
            [1.0_f32, 1.0, 1.0],
            [1.0_f32, 1.0, 1.0],
        );

        // drawing a frame
        let mut target = display.draw();
        // target.clear_color(0.2, 0.3, 0.3, 1.0);
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
        
        let mut box_uniforms = DynamicUniforms::new();
        box_uniforms.add(String::from("view"), &view_matrix);
        box_uniforms.add(String::from("projection"), &projection_matrix);
        box_uniforms.add(String::from("viewPos"), &camera_position);

        let model = Matrix4::identity();
        // model = model * Matrix4::from_translation(Vector3::zero());
        // model = model * Matrix4::from_scale(1.0_f32);
        let model = Into::<[[f32; 4]; 4]>::into(model);
        box_uniforms.add(String::from("model"), &model);
        
        // 定向光写入uniforms
        dir_light.add_to_uniforms("dirLight", &mut box_uniforms);

        // 点光源写入uniforms
        for (i, point_light) in point_lights.iter().enumerate() {
            let light_key = format!("pointLights[{}]", i);
            point_light.add_to_uniforms(light_key.as_str(), &mut box_uniforms);
        }

        // 聚光灯写入uniforms
        spot_light.add_to_uniforms("spotLight", &mut box_uniforms);
        
        // 循环渲染模型
        for model in models.iter() {
            let mut uniforms = box_uniforms.clone();
            if let Some(material) = &model.material {
                material.add_to_uniforms("material", &mut uniforms);
            }
            target.draw(&model.vertex_buffer, &model.index_buffer, &obj_program, &uniforms, &draw_parameters).unwrap();
        }
        
        for (_, light) in point_light_boxes.iter().enumerate() {
            let uniforms = uniform! {
                model: Into::<[[f32; 4]; 4]>::into(light.model * Matrix4::from_translation(light.position.to_vec())),
                view: Into::<[[f32; 4]; 4]>::into(view_matrix),
                projection: Into::<[[f32; 4]; 4]>::into(projection_matrix),
                color: light.color,
            };
            target.draw(&light.vertex_buffer, &light.index_buffer, &light_program, &uniforms, &draw_parameters).unwrap();
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





static INDEX_ARRAY: [u16; 36] = [0u16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35];


/**
 * 36个顶点的简单正方体
 */
pub struct Cube {
    id: String,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    color: [f32; 3],
    position: Point3<f32>,
    model: Matrix4<f32>,
}

impl Cube {
    /**
     * 边长，0-1，标准化设备坐标系范围内
     */
    pub fn new(id: &str, side_len: f32, display: &glium::Display, color: [f32; 3], position: Point3<f32>, model: Matrix4<f32>) -> Cube {
        let half = side_len / 2_f32;
        Cube {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // 前
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, 1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, 0_f32, 1_f32], texture: [1.0_f32, 0.0] },
                // 后
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texture: [1.0_f32, 0.0] },
                // 左
                Vertex { position: [-half, -half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [-half, half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [-1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [-1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                // 右
                Vertex { position: [half, -half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [1_f32, 0_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [1_f32, 0_f32, 0_f32], texture: [1.0_f32, 0.0] },
                // 上
                Vertex { position: [-half, half, half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, 1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 1_f32, 0_f32], texture: [1.0_f32, 0.0] },
                // 下
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32], texture: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, -1_f32, 0_f32], texture: [1.0_f32, 0.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDEX_ARRAY).unwrap(),
            color: color,
            position: position,
            model: model,
        }
    }
}


fn create_program(vert_source_path: &str, frag_source_path: &str, display: &Display) -> Program {
    let obj_vert_source = fs::read_to_string(vert_source_path).unwrap();
    let obj_frag_source = fs::read_to_string(frag_source_path).unwrap();
    let obj_shader_source = SourceCode {
        vertex_shader: obj_vert_source.as_str(),
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: obj_frag_source.as_str(),
    };

    glium::Program::new(
        display,
        ProgramCreationInput::from(obj_shader_source)
    ).unwrap()
}
