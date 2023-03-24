
extern crate glium;
extern crate cgmath;

use cgmath::{SquareMatrix, Point3, Matrix4, Vector3, Rad, InnerSpace};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event}, window::CursorGrabMode, dpi::LogicalSize}, framebuffer::SimpleFrameBuffer, texture::DepthTexture2d, uniforms::{UniformValue, Sampler, MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction}};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Cube, Plane}, material, create_program, start_loop, Action, context::{LoopContext}};

/// 平行光阴影映射demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let obj_program = create_program("src/bin/advanced_lighting_shadow_mapping/formal.vert", "src/bin/advanced_lighting_shadow_mapping/formal.frag", &display);
    let shadow_program = create_program("src/bin/advanced_lighting_shadow_mapping/shadow.vert", "src/bin/advanced_lighting_shadow_mapping/shadow.frag", &display);
    let debug_program = create_program("src/bin/advanced_lighting_shadow_mapping/debug_quad.vert", "src/bin/advanced_lighting_shadow_mapping/debug_quad.frag", &display);

    // 地板
    let floor = Plane::new("plane", 50.0, 50.0, -0.5_f32, &display, Point3::new(0.0, 0.0, 0.0), Matrix4::identity());

    let floor_texture = material::load_texture("src/wood.png".to_string(), &display).1;

    // 正方体
    let cubes = vec![
        Cube::new("cube1", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(0.0, 1.5, 0.0), Matrix4::from_scale(0.5)),
        Cube::new("cube2", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(2.0, 0.0, 1.0), Matrix4::from_scale(0.5)),
        Cube::new("cube3", 2.0, &display, [0.0, 0.0, 0.0], Point3::new(-1.0, 0.0, 2.0), Matrix4::from_axis_angle(Vector3::new(1.0, 0.0, 1.0).normalize(), Rad(60.0)) * Matrix4::from_scale(0.25))
    ];

    // 阴影贴图
    let shadow_texture = DepthTexture2d::empty(&display, 1024, 1024).unwrap();

    // 光源视角位置
    let light_view_position: [f32; 3] = [-2.0, 4.0, -1.0];

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 3_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let controller = CameraController::new(1_f32, 0.5_f32);
    // 摄像机透视矩阵
    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 100.0));

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
        let floor_model = UniformValue::Mat4(floor.calc_model().into());

        // 首先在光源的视角渲染一个帧缓冲，用于创建深度贴图
        // 本例中光源为平行定向光，所以使用正交投影矩阵，这样透视图不会变形
        let light_projection = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 1.0, 7.5);
        // 创建光源的视图矩阵，从光源的位置看向场景中央
        let light_view = Matrix4::look_at_rh(light_view_position.into(), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
        // 光空间矩阵
        let light_space_matrix: [[f32; 4]; 4] = (light_projection * light_view).into();

        // 创建深度贴图的帧缓冲
        let mut depth_framebuffer = SimpleFrameBuffer::depth_only(&display, &shadow_texture).unwrap();
        depth_framebuffer.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);

        // 渲染场景到帧缓冲
        let mut uniforms = DynamicUniforms::new();
        uniforms.add_str_key("lightSpaceMatrix", &light_space_matrix);
        // floor
        uniforms.add_str_key_value("model", floor_model);
        depth_framebuffer.draw(&floor.vertex_buffer, &floor.index_buffer, &shadow_program, &uniforms, &draw_parameters).unwrap();
        // cubes
        for cube in &cubes {
            uniforms.add_str_key_value("model", UniformValue::Mat4(cube.calc_model().into()));
            depth_framebuffer.draw(&cube.vertex_buffer, &cube.index_buffer, &shadow_program, &uniforms, &draw_parameters).unwrap();
        }

        // 渲染主要场景
        // 摄像机观察矩阵
        let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
        let camera_position: [f32; 3] = ctx.camera.position.into();

        let mut target = display.draw();
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
        
        uniforms.add_str_key("projection", &projection_matrix);
        uniforms.add_str_key("view", &view_matrix);
        uniforms.add_str_key("viewPos", &camera_position);
        uniforms.add_str_key("lightPos", &light_view_position);
        uniforms.add_str_key("diffuseTexture", &floor_texture);

        let shadow_sampler = Sampler::new(&shadow_texture).magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest).wrap_function(SamplerWrapFunction::Clamp);
        uniforms.add_str_key("shadowMap", &shadow_sampler);

        // floor
        uniforms.add_str_key_value("model", floor_model);
        target.draw(&floor.vertex_buffer, &floor.index_buffer, &obj_program, &uniforms, &draw_parameters).unwrap();
        // cubes
        for cube in &cubes {
            let model: [[f32; 4]; 4] = cube.calc_model().into();
            uniforms.add_str_key_value("model", UniformValue::Mat4(model));
            target.draw(&cube.vertex_buffer, &cube.index_buffer, &obj_program, &uniforms, &draw_parameters).unwrap();
        }

        target.finish().unwrap();

        Action::Continue
    });
}
