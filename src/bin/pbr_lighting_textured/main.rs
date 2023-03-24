
extern crate glium;
extern crate cgmath;

use cgmath::{Matrix4, Vector3};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue}};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Sphere}, create_program, start_loop, Action, context::{LoopContext}, lights::PointLight, material};

/// PBR lighting texture demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let pbr_program = create_program("src/bin/pbr_lighting_textured/pbr.vert", "src/bin/pbr_lighting_textured/pbr.frag", &display);

    let sphere = Sphere::new_simple(&display);
    let sphere_rows = 7;
    let sphere_columns = 7;
    let sphere_spacing = 2.5_f32;

    let albedo_texture = material::load_texture("src/pbr/rusted_iron/albedo.png".to_string(), &display).1;
    let normal_texture = material::load_texture("src/pbr/rusted_iron/normal.png".to_string(), &display).1;
    let metallic_texture = material::load_texture("src/pbr/rusted_iron/metallic.png".to_string(), &display).1;
    let roughness_texture = material::load_texture("src/pbr/rusted_iron/roughness.png".to_string(), &display).1;
    let ao_texture = material::load_texture("src/pbr/rusted_iron/ao.png".to_string(), &display).1;

    // 点光源
    let point_lights = vec![
        PointLight::new_simple([0.0, 0.0, 10.0], [150.0, 150.0, 150.0])
    ];

    // 摄像机初始位置
    let camera = Camera::new(
        cgmath::Point3::new(0.0, 0.0, 3.0),
        cgmath::Rad::from(cgmath::Deg(-90.0)),
        cgmath::Rad::from(cgmath::Deg(0.0))
    );
    let controller = CameraController::new(1.0, 0.5);
    // 摄像机透视矩阵
    let projection_matrix: [[f32; 4]; 4] = cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1, 100.0).into();

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let loop_context = LoopContext::new(camera, controller);

    start_loop(event_loop, loop_context, move |_: Option<Event<()>>, ctx, _| {
        // 摄像机观察矩阵
        let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
        let camera_position: [f32; 3] = ctx.camera.position.into();

        let mut uniforms = DynamicUniforms::new();
        uniforms.add_str_key("projection", &projection_matrix);
        uniforms.add_str_key("view", &view_matrix);
        uniforms.add_str_key("camPos", &camera_position);
        uniforms.add_str_key("albedoMap", &albedo_texture);
        uniforms.add_str_key("normalMap", &normal_texture);
        uniforms.add_str_key("metallicMap", &metallic_texture);
        uniforms.add_str_key("roughnessMap", &roughness_texture);
        uniforms.add_str_key("aoMap", &ao_texture);

        // 创建默认帧
        let mut target = display.draw();
        target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

        // 渲染球体
        for r in 0..sphere_rows {
            for c in 0..sphere_columns {
                let model = Matrix4::from_translation(Vector3::new(
                    (c - (sphere_columns / 2)) as f32 * sphere_spacing,
                    (r - (sphere_rows / 2)) as f32 * sphere_spacing,
                    0.0
                ));
                uniforms.add_str_key_value("model", UniformValue::Mat4(model.into()));
                target.draw(&sphere.vertex_buffer, &sphere.index_buffer, &pbr_program, &uniforms, &draw_parameters).unwrap();
            }
        }

        // 渲染光源
        for (i, point_light) in point_lights.iter().enumerate() {
            let light_key = format!("lights[{}]", i);
            point_light.add_to_uniforms(light_key.as_str(), &mut uniforms);
        }
        for point_light in point_lights.iter() {
            let model = Matrix4::from_translation(Vector3::new(point_light.position[0], point_light.position[1], point_light.position[2])) * Matrix4::from_scale(0.5);
            uniforms.add_str_key_value("model", UniformValue::Mat4(model.into()));
            target.draw(&sphere.vertex_buffer, &sphere.index_buffer, &pbr_program, &uniforms, &draw_parameters).unwrap();
        }
        // 有个奇怪的点：渲染球体时没有向uniforms添加光源信息，添加光源后仅渲染光源，之前球体的片段也会有光源的效果
        // 猜想：其实总体仅渲染了一次，调用draw时，仅做了一些变量绑定的工作，finish的时候做swap_buffer操作时才会调用着色器

        target.finish().unwrap();

        Action::Continue
    });
}
