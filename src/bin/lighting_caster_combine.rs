// learn open gl 光照-多光源结合

#[macro_use]
extern crate glium;
extern crate cgmath;

use std::{time::{self}, collections::HashMap, io::Cursor, ops::Add};

use cgmath::{SquareMatrix, Point3, Matrix4, EuclideanSpace, InnerSpace, Angle};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{index::PrimitiveType, glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState}, window::CursorGrabMode}, VertexBuffer, IndexBuffer, uniforms::{AsUniformValue, Uniforms, UniformValue}, texture::CompressedSrgbTexture2d};

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
                out vec3 FragPos;
                out vec2 TexCoords;
                
                uniform mat4 model;
                uniform mat4 view;
                uniform mat4 projection;
                
                void main()
                {
                    FragPos = vec3(model * vec4(position, 1.0));
                    oNormal = mat3(transpose(inverse(model))) * normal;
                    gl_Position = projection * view * vec4(FragPos, 1.0);

                    TexCoords = texCoords;
                }
            ",
            fragment: "
                #version 330 core

                // 材质
                struct Material {
                    sampler2D diffuse;
                    sampler2D specular;
                    float shininess;
                };

                // 定向光源
                struct DirLight {
                    vec3 direction;
                
                    vec3 ambient;
                    vec3 diffuse;
                    vec3 specular;
                };
                // 计算定向光源函数（光源方向, 片段法向量，视角方向）
                vec3 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir);

                // 点光源
                struct PointLight {
                    vec3 position;
                
                    float constant;
                    float linear;
                    float quadratic;
                
                    vec3 ambient;
                    vec3 diffuse;
                    vec3 specular;
                };
                // 计算点光源函数（点光源，法向量，片段位置，视角向量）
                vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir);

                // 聚光灯
                struct SpotLight {
                    vec3 position;
                    vec3 direction;
                    float cutOff;
                    float outerCutOff;
                  
                    float constant;
                    float linear;
                    float quadratic;
                  
                    vec3 ambient;
                    vec3 diffuse;
                    vec3 specular;       
                };
                // 计算聚光灯函数（聚光灯，法向量，片段位置，视角向量）
                vec3 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir);
                
                #define NR_POINT_LIGHTS 4
        
                in vec3 oNormal;
                in vec3 FragPos;
                in vec2 TexCoords;

                uniform vec3 viewPos;
                uniform DirLight dirLight;
                uniform PointLight pointLights[NR_POINT_LIGHTS];
                uniform SpotLight spotLight;
                uniform Material material;

                out vec4 FragColor;

                void main()
                {
                    // 属性
                    vec3 norm = normalize(oNormal);
                    vec3 viewDir = normalize(viewPos - FragPos);

                    // 第一阶段：定向光照
                    vec3 result = CalcDirLight(dirLight, norm, viewDir);
                    // 第二阶段：点光源
                    for(int i = 0; i < NR_POINT_LIGHTS; i++)
                        result += CalcPointLight(pointLights[i], norm, FragPos, viewDir);
                    // 第三阶段：聚光
                    result += CalcSpotLight(spotLight, norm, FragPos, viewDir);

                    FragColor = vec4(result, 1.0);
                }

                vec3 CalcDirLight(DirLight light, vec3 normal, vec3 viewDir)
                {
                    vec3 lightDir = normalize(-light.direction);
                    // 漫反射着色
                    float diff = max(dot(normal, lightDir), 0.0);
                    // 镜面光着色
                    vec3 reflectDir = reflect(-lightDir, normal);
                    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
                    // 合并结果
                    vec3 ambient  = light.ambient  * vec3(texture(material.diffuse, TexCoords));
                    vec3 diffuse  = light.diffuse  * diff * vec3(texture(material.diffuse, TexCoords));
                    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoords));
                    return (ambient + diffuse + specular);
                }

                vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir)
                {
                    vec3 lightDir = normalize(light.position - fragPos);
                    // 漫反射着色
                    float diff = max(dot(normal, lightDir), 0.0);
                    // 镜面光着色
                    vec3 reflectDir = reflect(-lightDir, normal);
                    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
                    // 衰减
                    float distance    = length(light.position - fragPos);
                    float attenuation = 1.0 / (light.constant + light.linear * distance + 
                                light.quadratic * (distance * distance));
                    // 合并结果
                    vec3 ambient  = light.ambient  * vec3(texture(material.diffuse, TexCoords));
                    vec3 diffuse  = light.diffuse  * diff * vec3(texture(material.diffuse, TexCoords));
                    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoords));
                    ambient  *= attenuation;
                    diffuse  *= attenuation;
                    specular *= attenuation;
                    return (ambient + diffuse + specular);
                }

                // calculates the color when using a spot light.
                vec3 CalcSpotLight(SpotLight light, vec3 normal, vec3 fragPos, vec3 viewDir)
                {
                    vec3 lightDir = normalize(light.position - fragPos);
                    // diffuse shading
                    float diff = max(dot(normal, lightDir), 0.0);
                    // specular shading
                    vec3 reflectDir = reflect(-lightDir, normal);
                    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
                    // attenuation
                    float distance = length(light.position - fragPos);
                    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));
                    // spotlight intensity
                    float theta = dot(lightDir, normalize(-light.direction));
                    float epsilon = light.cutOff - light.outerCutOff;
                    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
                    // combine results
                    vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoords));
                    vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoords));
                    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoords));
                    ambient *= attenuation * intensity;
                    diffuse *= attenuation * intensity;
                    specular *= attenuation * intensity;
                    return (ambient + diffuse + specular);
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
    let box_cubes = {
        let mut boxes = Vec::<Cube>::with_capacity(10);
        let trans = [
            [0.0f32,  0.0,  0.0],
            [2.0f32,  5.0, -15.0],
            [-1.5f32, -2.2, -2.5],
            [-3.8f32, -2.0, -12.3],
            [2.4f32, -0.4, -3.5],
            [-1.7f32,  3.0, -7.5],
            [1.3f32, -2.0, -2.5],
            [1.5f32,  2.0, -2.5],
            [1.5f32,  0.2, -1.5],
            [-1.3f32,  1.0, -1.5]
        ];

        for tran in trans {
            boxes.push(Cube::new("box", 0.5_f32, &display, [0.1_f32, 0.1, 0.1], Point3::from(tran), Matrix4::<f32>::identity()));
        }
        boxes
    };
    

    // 定向光
    let dir_light = DirLight::new(
        [-0.2_f32, -1.0, -0.3],
        [0.2_f32, 0.2, 0.2],
        [0.5_f32, 0.5, 0.5],
        [1.0_f32, 1.0, 1.0]
    );

    // 点光源
    let (point_light_boxes, point_lights) = {
        let positions = [
            [0.7_f32, 0.2, 2.0],
            [2.3_f32, -3.3, -4.0],
            [-4.0_f32, 2.0, -12.0],
            [0.0_f32, 0.0, -3.0]
        ];

        let mut light_boxes = Vec::<Cube>::with_capacity(4);
        let mut point_lights = Vec::<PointLight>::with_capacity(4);
        let light_color = [1.0_f32, 1.0, 1.0];
        for position in positions {
            light_boxes.push(Cube::new("light", 0.1_f32, &display, light_color, Point3::from(position), Matrix4::<f32>::identity()));
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
        cgmath::Point3::new(-2_f32, 1_f32, 1_f32), 
        cgmath::Rad::from(cgmath::Deg(0_f32)), 
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

    let mut box_angle = 0.0_f32;

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
        let spot_light = SpotLight {
            position: camera_position,
            direction: Into::<[f32; 3]>::into(camera.direction()),
            cut_off: cgmath::Deg(12.5_f32).cos(),
            outer_cut_off: cgmath::Deg(15.0_f32).cos(),
            constant: 1.0_f32,
            linear: 0.09_f32,
            quadratic: 0.032_f32,
            ambient: [0.0_f32, 0.0, 0.0],
            diffuse: [1.0_f32, 1.0, 1.0],
            specular: [1.0_f32, 1.0, 1.0],
        };

        // drawing a frame
        let mut target = display.draw();
        // target.clear_color(0.2, 0.3, 0.3, 1.0);
        target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

        box_angle += 0.1;
        if box_angle >= 360.0 {
            box_angle = 0.0;
        }
        
        let mut box_uniforms = DynamicUniforms::new();
        box_uniforms.add("view", &view_matrix);
        box_uniforms.add("projection", &projection_matrix);
        box_uniforms.add("viewPos", &camera_position);
        
        box_uniforms.add("dirLight.direction", &dir_light.direction);
        box_uniforms.add("dirLight.ambient", &dir_light.ambient);
        box_uniforms.add("dirLight.diffuse", &dir_light.diffuse);
        box_uniforms.add("dirLight.specular", &dir_light.specular);

        box_uniforms.add("pointLights[0].constant", &point_lights[0].constant);
        box_uniforms.add("pointLights[0].linear", &point_lights[0].linear);
        box_uniforms.add("pointLights[0].quadratic", &point_lights[0].quadratic);
        box_uniforms.add("pointLights[0].ambient", &point_lights[0].ambient);
        box_uniforms.add("pointLights[0].diffuse", &point_lights[0].diffuse);
        box_uniforms.add("pointLights[0].specular", &point_lights[0].specular);

        box_uniforms.add("pointLights[1].constant", &point_lights[1].constant);
        box_uniforms.add("pointLights[1].linear", &point_lights[1].linear);
        box_uniforms.add("pointLights[1].quadratic", &point_lights[1].quadratic);
        box_uniforms.add("pointLights[1].ambient", &point_lights[1].ambient);
        box_uniforms.add("pointLights[1].diffuse", &point_lights[1].diffuse);
        box_uniforms.add("pointLights[1].specular", &point_lights[1].specular);

        box_uniforms.add("pointLights[2].constant", &point_lights[2].constant);
        box_uniforms.add("pointLights[2].linear", &point_lights[2].linear);
        box_uniforms.add("pointLights[2].quadratic", &point_lights[2].quadratic);
        box_uniforms.add("pointLights[2].ambient", &point_lights[2].ambient);
        box_uniforms.add("pointLights[2].diffuse", &point_lights[2].diffuse);
        box_uniforms.add("pointLights[2].specular", &point_lights[2].specular);

        box_uniforms.add("pointLights[3].constant", &point_lights[3].constant);
        box_uniforms.add("pointLights[3].linear", &point_lights[3].linear);
        box_uniforms.add("pointLights[3].quadratic", &point_lights[3].quadratic);
        box_uniforms.add("pointLights[3].ambient", &point_lights[3].ambient);
        box_uniforms.add("pointLights[3].diffuse", &point_lights[3].diffuse);
        box_uniforms.add("pointLights[3].specular", &point_lights[3].specular);

        box_uniforms.add("material.diffuse", &box_material.diffuse);
        box_uniforms.add("material.specular", &box_material.specular);
        box_uniforms.add("material.shininess", &box_material.shininess);

        box_uniforms.add("spotLight.direction", &spot_light.direction);
        box_uniforms.add("spotLight.position", &spot_light.position);
        box_uniforms.add("spotLight.cutOff", &spot_light.cut_off);
        box_uniforms.add("spotLight.outerCutOff", &spot_light.outer_cut_off);
        box_uniforms.add("spotLight.constant", &spot_light.constant);
        box_uniforms.add("spotLight.linear", &spot_light.linear);
        box_uniforms.add("spotLight.quadratic", &spot_light.quadratic);
        box_uniforms.add("spotLight.ambient", &spot_light.ambient);
        box_uniforms.add("spotLight.diffuse", &spot_light.diffuse);
        box_uniforms.add("spotLight.specular", &spot_light.specular);
        
        let axis = cgmath::Vector3::new(1.0_f32, 0.3, 0.5).normalize();
        let mut models = Vec::with_capacity(box_cubes.len());
        for (_, box_cube) in box_cubes.iter().enumerate() {
            models.push(Into::<[[f32; 4]; 4]>::into(Matrix4::from_translation(box_cube.position.to_vec()) * Matrix4::from_axis_angle(axis, cgmath::Deg(box_angle))));
        }
        for (i, box_cube) in box_cubes.iter().enumerate() {
            box_uniforms.add("model", &models[i]);
            target.draw(&box_cube.vertex_buffer, &box_cube.index_buffer, &box_program, &box_uniforms, &draw_parameters).unwrap();
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


fn build_name<'a>(variable_name: &'a str, name: &'a str) -> String {
    variable_name.to_string().add(".").add(name)
}


/**
 * 定向光源
 */
struct DirLight {
    direction: [f32; 3],

    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}

impl DirLight {

    pub fn new(direction: [f32; 3], ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3],) -> DirLight {
        DirLight { direction: direction, ambient: ambient, diffuse: diffuse, specular: specular }
    }
}


/**
 * 点光源
 */
struct PointLight {
    position: [f32; 3],

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}

impl PointLight {
    pub fn new(position: [f32; 3], constant: f32, linear: f32, quadratic: f32, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) -> PointLight {
        PointLight {
            position: position,
            constant: constant,
            linear: linear,
            quadratic: quadratic,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
        }
    }
}

/**
 * 聚光灯
 */
struct SpotLight {
    position: [f32; 3],
    direction: [f32; 3],
    cut_off: f32,
    outer_cut_off: f32,
    constant: f32,
    linear: f32,
    quadratic: f32,
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}



#[macro_export]
macro_rules! dynamic_uniform{
    () => {
        $crate::uniforms::DynamicUniforms::new()
    };

    ($($field:ident: $value:expr), *,) => {
        {
            let mut tmp = $crate::DynamicUniforms::new();
            $(
                tmp.add(stringify!($field), $value);
            )*
            tmp
        }
    };
}

#[derive(Clone)]
pub struct DynamicUniforms<'a, 's>{
    map: HashMap<&'s str, UniformValue<'a>>,
}

impl<'a, 's> DynamicUniforms<'a, 's>{
    /// Creates new DynamicUniforms
    pub fn new() -> Self{
        Self{
            map: HashMap::new()
        }
    }

    /// Add a value to the DynamicUniforms
    #[inline]
    pub fn add(&mut self, key: &'s str, value: &'a dyn AsUniformValue){
        self.map.insert(key, value.as_uniform_value());
    }
}

impl Uniforms for DynamicUniforms<'_, '_>{
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut output: F) {
        for (key, value) in self.map.iter(){
            output(key, *value);
        }
    }
}