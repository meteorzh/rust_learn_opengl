// learn open gl 光照-投光物学习

#[macro_use]
extern crate glium;
extern crate cgmath;

use std::{time::{self}, sync::Mutex, collections::HashMap, io::Cursor};

use cgmath::{SquareMatrix, Point3, Matrix4, EuclideanSpace, Vector3, InnerSpace, Angle};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{index::PrimitiveType, glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState}, window::CursorGrabMode}, VertexBuffer, IndexBuffer, Program, uniforms::{UniformsStorage, EmptyUniforms, AsUniformValue, Uniforms}, texture::CompressedSrgbTexture2d};

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
                in vec2 texCoords;

                out vec3 oNormal;
                out vec3 fragPos;
                out vec2 TexCoords;
                
                uniform mat4 model;
                uniform mat4 view;
                uniform mat4 projection;
                
                void main()
                {
                    fragPos = vec3(model * vec4(position, 1.0));
                    oNormal = mat3(transpose(inverse(model))) * normal;
                    gl_Position = projection * view * vec4(fragPos, 1.0);

                    TexCoords = texCoords;
                }
            ",
            fragment: "
                #version 330 core

                in vec3 oNormal;
                in vec3 fragPos;
                in vec2 TexCoords;
        
                out vec4 FragColor;

                struct Material {
                    sampler2D diffuse;
                    sampler2D specular;
                    float shininess;
                };

                struct Light {
                    // 点光源位置
                    vec3 position;
                    // 平行光光源方向
                    vec3 direction;
                    // 环境光属性
                    vec3 ambient;
                    vec3 diffuse;
                    vec3 specular;
                    // 点光源衰减
                    float constant;
                    float linear;
                    float quadratic;
                    // 聚光灯信息
                    vec3  spot_position;
                    vec3  spot_direction;
                    float cutOff;
                    float outerCutOff;
                    vec3 spot_diffuse;
                    vec3 spot_specular;
                };
                
                uniform Material material;

                uniform Light light;
                uniform vec3 viewPos;
                
                void main() {
                    vec3 spot_dir = normalize(light.spot_position - fragPos);
                    float theta = dot(spot_dir, normalize(-light.spot_direction));
                    float epsilon = light.cutOff - light.outerCutOff;
                    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);

                    if(theta > light.outerCutOff) {
                        // ambient
                        vec3 ambient = light.ambient * texture(material.diffuse, TexCoords).rgb;
                        
                        // diffuse 
                        vec3 norm = normalize(oNormal);
                        float diff = max(dot(norm, spot_dir), 0.0);
                        vec3 diffuse = light.spot_diffuse * diff * texture(material.diffuse, TexCoords).rgb;
                        diffuse *= intensity;
                        
                        // specular
                        vec3 viewDir = normalize(viewPos - fragPos);
                        vec3 reflectDir = reflect(-spot_dir, norm);  
                        float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
                        vec3 specular = light.spot_specular * spec * texture(material.specular, TexCoords).rgb;
                        specular *= intensity;
                        
                        // attenuation
                        float distance    = length(light.spot_position - fragPos);
                        float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));    

                        // ambient  *= attenuation; // remove attenuation from ambient, as otherwise at large distances the light would be darker inside than outside the spotlight due the ambient term in the else branche
                        diffuse   *= attenuation;
                        specular *= attenuation;
                            
                        vec3 result = ambient + diffuse + specular;
                        FragColor = vec4(result, 1.0);
                    } else {
                        float distance = length(light.position - fragPos);
                        float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));

                        vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoords));
                        ambient *= attenuation;

                        vec3 norm = normalize(oNormal);
                        vec3 lightDir = normalize(-light.direction);
                        float diff = max(dot(norm, lightDir), 0.0);
                        vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoords));
                        diffuse *= attenuation;

                        vec3 viewDir = normalize(viewPos - fragPos);
                        vec3 reflectDir = reflect(-lightDir, norm);
                        float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
                        vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoords));
                        specular *= attenuation;

                        vec3 result = ambient + diffuse + specular;
        
                        FragColor = vec4(result, 1.0);
                    }
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

    let image = image::load(Cursor::new(&include_bytes!("../container2.png")), image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let opengl_texture = glium::texture::CompressedSrgbTexture2d::new(&display, image).unwrap();

    let image = image::load(Cursor::new(&include_bytes!("../container2_specular.png")), image::ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let opengl_texture_specular = glium::texture::CompressedSrgbTexture2d::new(&display, image).unwrap();

    // 初始化顶点缓冲及着色器等资源
    // building the vertex buffer, which contains all the vertices that we will draw
    let box_material = Material::new(opengl_texture, opengl_texture_specular, 32.0_f32);
    let box_cube = Cube::new("box", 0.5_f32, &display, [1.0f32, 0.5, 0.31], 
        box_program, Point3::new(0_f32, 0.0, 0.0), Matrix4::<f32>::identity());
    let light_position = Point3::new(1.2_f32, 1.0, 2.0);
    let light_model = Matrix4::<f32>::from_scale(0.2_f32);
    let light_color = [1.0f32, 1.0, 1.0];
    let light_cube = Cube::new("light", 0.5_f32, &display, light_color, 
        light_program, light_position, light_model);
    let light_specular = [1.0_f32, 1.0, 1.0];

    let light_direction = [-0.2_f32, -1.0, -0.3];

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let mut camera = Camera::new(
        cgmath::Point3::new(-2_f32, 1_f32, 1_f32), 
        cgmath::Rad::from(cgmath::Deg(0_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
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

    let kc = 1.0_f32;

    let mut last_frame = time::Instant::now();

    let mut step = 1_f32;

    // 多个正方体的位移矩阵
    let box_trans = {
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
        target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

        // let current_secs = current.elapsed().as_millis() as f32;
        // let light_color = cgmath::vec3(
        //     (step * 2.0_f32).sin(),
        //     (step * 0.7_f32).sin(),
        //     (step * 1.3_f32).sin()
        // );
        let light_color = cgmath::Vector3::from(light_color);
        let light_diffuse = cgmath::vec3(0.5_f32, 0.5, 0.5);
        let light_ambient = cgmath::vec3(0.2_f32, 0.2, 0.2);
        // step += 0.01;

        
        let box_uniforms = uniform! {
            view: Into::<[[f32; 4]; 4]>::into(view_matrix),
            projection: Into::<[[f32; 4]; 4]>::into(projection_matrix),
            // color: box_cube.color,
            // lightColor: light_color,
            // lightPos: Into::<[f32; 3]>::into(light_position),
            viewPos: Into::<[f32; 3]>::into(camera.position),
        };
        let box_uniforms = box_uniforms.add("material.diffuse", &box_material.diffuse);
        let box_uniforms = box_uniforms.add("material.specular", &box_material.specular);
        let box_uniforms = box_uniforms.add("material.shininess", box_material.shininess);
        let box_uniforms = box_uniforms.add("light.ambient", Into::<[f32; 3]>::into(light_ambient));
        let box_uniforms = box_uniforms.add("light.diffuse", Into::<[f32; 3]>::into(light_diffuse));
        let box_uniforms = box_uniforms.add("light.specular", light_specular);
        let box_uniforms = box_uniforms.add("light.position", Into::<[f32; 3]>::into(light_position));
        let box_uniforms = box_uniforms.add("light.direction", light_direction);
        let box_uniforms = box_uniforms.add("light.constant", 1.0_f32);
        let box_uniforms = box_uniforms.add("light.linear", 0.09_f32);
        let box_uniforms = box_uniforms.add("light.quadratic", 0.032_f32);
        let box_uniforms = box_uniforms.add("light.spot_position", Into::<[f32; 3]>::into(camera.position));
        let box_uniforms = box_uniforms.add("light.spot_direction", Into::<[f32; 3]>::into(camera.direction()));
        let box_uniforms = box_uniforms.add("light.cutOff", cgmath::Deg(15.0_f32).cos());
        let box_uniforms = box_uniforms.add("light.spot_diffuse", [1.0_f32, 1.0, 1.0]);
        let box_uniforms = box_uniforms.add("light.spot_specular", [1.0_f32, 1.0, 1.0]);
        let box_uniforms = box_uniforms.add("light.outerCutOff", cgmath::Deg(20.0_f32).cos());
        
        let light_uniforms = uniform! {
            model: Into::<[[f32; 4]; 4]>::into(light_cube.model * Matrix4::from_translation(light_cube.position.to_vec())),
            view: Into::<[[f32; 4]; 4]>::into(view_matrix),
            projection: Into::<[[f32; 4]; 4]>::into(projection_matrix),
            color: light_cube.color,
        };
        
        let axis = cgmath::Vector3::new(1.0_f32, 0.3, 0.5).normalize();
        for (i, box_tran) in box_trans.iter().enumerate() {
            let angle = 20.0f32 * i as f32;
            let box_uniforms = box_uniforms.add("model", Into::<[[f32; 4]; 4]>::into(Matrix4::from_axis_angle(axis, cgmath::Deg(angle)) * box_tran));
            target.draw(&box_cube.vertex_buffer, &box_cube.index_buffer, &box_cube.program, &box_uniforms, &draw_parameters).unwrap();
        }
        
        target.draw(&light_cube.vertex_buffer, &light_cube.index_buffer, &light_cube.program, &light_uniforms, &draw_parameters).unwrap();

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
    texCoords: [f32; 2],
}

implement_vertex!(Vertex, position, normal, texCoords);

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
}

impl Cube {
    /**
     * 边长，0-1，标准化设备坐标系范围内
     */
    pub fn new(id: &str, side_len: f32, display: &glium::Display, color: [f32; 3], program: Program, position: Point3<f32>, model: Matrix4<f32>) -> Cube {
        let half = side_len / 2_f32;
        Cube {
            id: id.to_string(),
            vertex_buffer: glium::VertexBuffer::new(display, &[
                // 前
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, 1_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, 0_f32, 1_f32], texCoords: [0.0_f32, 1.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, 1_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, 0_f32, 1_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 0_f32, 1_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, 0_f32, 1_f32], texCoords: [1.0_f32, 0.0] },
                // 后
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 0_f32, -1_f32], texCoords: [0.0_f32, 1.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 0_f32, -1_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, 0_f32, -1_f32], texCoords: [1.0_f32, 0.0] },
                // 左
                Vertex { position: [-half, -half, -half], normal: [-1_f32, 0_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [-1_f32, 0_f32, 0_f32], texCoords: [0.0_f32, 1.0] },
                Vertex { position: [-half, half, half], normal: [-1_f32, 0_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, -half], normal: [-1_f32, 0_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, half], normal: [-1_f32, 0_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [-1_f32, 0_f32, 0_f32], texCoords: [1.0_f32, 0.0] },
                // 右
                Vertex { position: [half, -half, half], normal: [1_f32, 0_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [half, half, half], normal: [1_f32, 0_f32, 0_f32], texCoords: [0.0_f32, 1.0] },
                Vertex { position: [half, half, -half], normal: [1_f32, 0_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [1_f32, 0_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [1_f32, 0_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [1_f32, 0_f32, 0_f32], texCoords: [1.0_f32, 0.0] },
                // 上
                Vertex { position: [-half, half, half], normal: [0_f32, 1_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [-half, half, -half], normal: [0_f32, 1_f32, 0_f32], texCoords: [0.0_f32, 1.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 1_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [-half, half, half], normal: [0_f32, 1_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [half, half, -half], normal: [0_f32, 1_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [half, half, half], normal: [0_f32, 1_f32, 0_f32], texCoords: [1.0_f32, 0.0] },
                // 下
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [-half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texCoords: [0.0_f32, 1.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [-half, -half, half], normal: [0_f32, -1_f32, 0_f32], texCoords: [0.0_f32, 0.0] },
                Vertex { position: [half, -half, -half], normal: [0_f32, -1_f32, 0_f32], texCoords: [1.0_f32, 1.0] },
                Vertex { position: [half, -half, half], normal: [0_f32, -1_f32, 0_f32], texCoords: [1.0_f32, 0.0] },
            ]).unwrap(),
            index_buffer: glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDEX_ARRAY).unwrap(),
            color: color,
            program: program,
            position: position,
            model: model,
        }
    }
}


pub struct Material {
    diffuse: CompressedSrgbTexture2d,
    specular: CompressedSrgbTexture2d,
    shininess: f32,
}

impl Material {
    pub fn new(diffuse: CompressedSrgbTexture2d, specular: CompressedSrgbTexture2d, shininess: f32) -> Material {
        Material {
            diffuse: diffuse,
            specular: specular,
            shininess: shininess
        }
    }
}