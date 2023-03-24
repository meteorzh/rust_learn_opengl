
extern crate glium;
extern crate cgmath;

use std::{fs::File, io::BufReader};

use cgmath::{Matrix4, Vector3, Point3, SquareMatrix, Deg};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event}, window::CursorGrabMode, dpi::LogicalSize}, uniforms::{UniformValue}, framebuffer::{DepthRenderBuffer, SimpleFrameBuffer}, texture::{DepthFormat, Texture2d, UncompressedFloatFormat, MipmapsOption, Cubemap, RawImage2d}, DepthTest, Rect, index::PrimitiveType};
use image::io::Reader as ImageReader;

use ouroboros::self_referencing;
use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Sphere, Cube, Plane}, create_program, start_loop, Action, context::{LoopContext}, lights::PointLight, material, create_program_vgf};

pub struct Dt {
    hdr_env_texture: Texture2d,
    env_cubemap: Cubemap,
    capture_render_buffer: DepthRenderBuffer,
    env2irr_render_buffer: DepthRenderBuffer,
    irradiance_cubemap: Cubemap,
    cube: Cube,
    plane: Plane,
    skybox: Cube,
    test_texture: Texture2d,
}

#[self_referencing]
struct Data {
    dt: Dt,
    #[borrows(dt)]
    #[covariant]
    buffs: (SimpleFrameBuffer<'this>, SimpleFrameBuffer<'this>, &'this Dt),
}

/// ibl irradiance demo
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let pbr_program = create_program("src/bin/ibl_irradiance/pbr.vert", "src/bin/ibl_irradiance/pbr.frag", &display);
    let equirectangular_to_cubemap_program = create_program_vgf("src/bin/ibl_irradiance/cubemap.vert", "src/bin/ibl_irradiance/render_to_cubemap.geom", "src/bin/ibl_irradiance/equirectangular_to_cubemap.frag", &display);
    let equirectangular_to_cubemap_program2 = create_program("src/bin/ibl_irradiance/cubemap2.vert", "src/bin/ibl_irradiance/equirectangular_to_cubemap.frag", &display);
    let irradiance_program = create_program_vgf("src/bin/ibl_irradiance/cubemap.vert", "src/bin/ibl_irradiance/render_to_cubemap.geom", "src/bin/ibl_irradiance/irradiance_convolution.frag", &display);
    let background_program = create_program("src/bin/ibl_irradiance/background.vert", "src/bin/ibl_irradiance/background.frag", &display);
    let test_hdr_image_program = create_program("src/bin/ibl_irradiance/test.vert", "src/bin/ibl_irradiance/test.frag", &display);
    let test_cubemap_program = create_program("src/bin/ibl_irradiance/test_cubemap.vert", "src/bin/ibl_irradiance/test_cubemap.frag", &display);

    let sphere = Sphere::new_simple(&display);
    let sphere_rows = 7;
    let sphere_columns = 7;
    let sphere_spacing = 2.5_f32;

    let mut tenants = DataBuilder {
        dt: Dt {
            hdr_env_texture: {
                let hdr_env_image = ImageReader::open("src/hdr/newport_loft.hdr").unwrap().decode().unwrap();
                // let hdr_env_image = material::load_as_dynamic("src/hdr/newport_loft.hdr");
                let hdr_env_image = hdr_env_image.into_rgb32f();
                let dimensions = hdr_env_image.dimensions();
                let hdr_env_image = RawImage2d::from_raw_rgb_reversed(&hdr_env_image.into_raw(), dimensions);
                Texture2d::with_format(&display, hdr_env_image, UncompressedFloatFormat::F16F16F16, MipmapsOption::NoMipmap).unwrap()
            },
            // pbr: setup cubemap to render to and attach to framebuffer
            env_cubemap: Cubemap::empty_with_format(&display, UncompressedFloatFormat::F16F16F16, MipmapsOption::NoMipmap, 512).unwrap(),
            capture_render_buffer: DepthRenderBuffer::new(&display, DepthFormat::I24, 512, 512).unwrap(),
            env2irr_render_buffer: DepthRenderBuffer::new(&display, DepthFormat::I24, 512, 512).unwrap(),
            irradiance_cubemap: Cubemap::empty_with_format(&display, UncompressedFloatFormat::F16F16F16, MipmapsOption::NoMipmap, 32).unwrap(),
            cube: Cube::new("cube", 10.0, &display, [0.0, 0.0, 0.0], Point3::new(0.0, 0.0, 0.0), Matrix4::identity()),
            plane: Plane::new_vertical_center_plane("plane", 2.0, 2.0, &display, PrimitiveType::TrianglesList),
            skybox: Cube::new_skybox("skybox", 2.0, &display),
            test_texture: Texture2d::empty_with_format(&display, UncompressedFloatFormat::F16F16F16, MipmapsOption::NoMipmap, 512, 512).unwrap(),
        },
        buffs_builder: |dt| {
            let mut hdr2env_framebuffer = SimpleFrameBuffer::with_depth_buffer(&display, &dt.env_cubemap, &dt.capture_render_buffer).unwrap();

            // pbr: set up projection and view matrices for capturing data onto the 6 cubemap face directions
            // 这个地方和官方示例不一样，官方是分6次渲染，这里采用点状阴影中使用几何着色器的方式
            let capture_projection: [[f32; 4]; 4] = cgmath::perspective(Deg(90.0), 1.0, 0.1, 10.0).into();
            let captures_views: [[[f32; 4]; 4]; 6] = [
                Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), -Vector3::unit_y()).into(),
                Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), Point3::new(-1.0, 0.0, 0.0), -Vector3::unit_y()).into(),
                Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0), Vector3::unit_z()).into(),
                Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, -1.0, 0.0), -Vector3::unit_z()).into(),
                Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 1.0), -Vector3::unit_y()).into(),
                Matrix4::look_at_rh(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, -1.0), -Vector3::unit_y()).into(),
            ];

            // pbr: convert HDR equirectangular environment map to cubemap equivalent
            let draw_parameters_v512 = glium::DrawParameters {
                depth: glium::Depth {
                    test: DepthTest::IfLessOrEqual,
                    write: true,
                    .. Default::default()
                },
                viewport: Some(Rect {
                    left: 0,
                    bottom: 0,
                    width: 512,
                    height: 512,
                }),
                .. Default::default()
            };
            let mut uniforms = DynamicUniforms::new();
            uniforms.add_str_key("equirectangularMap", &dt.hdr_env_texture);
            uniforms.add_str_key("capture_projection", &capture_projection);
            for (index, view) in captures_views.iter().enumerate() {
                let key = format!("cube_views[{}]", index);
                uniforms.add_str_key_value(key.as_str(), UniformValue::Mat4(*view));
            }
            // uniforms.add_str_key("view", captures_views.get(0).unwrap());
            hdr2env_framebuffer.draw(&dt.cube.vertex_buffer, &dt.cube.index_buffer, &equirectangular_to_cubemap_program, &uniforms, &draw_parameters_v512).unwrap();

            // pbr: create an irradiance cubemap, and re-scale capture FBO to irradiance scale.
            // did in Dt

            // pbr: solve diffuse integral by convolution to create an irradiance (cube)map.
            let mut env2irr_framebuffer = SimpleFrameBuffer::with_depth_buffer(&display, &dt.irradiance_cubemap, &dt.env2irr_render_buffer).unwrap();
            let draw_parameters_v32 = glium::DrawParameters {
                depth: glium::Depth {
                    test: DepthTest::IfLessOrEqual,
                    write: true,
                    .. Default::default()
                },
                viewport: Some(Rect {
                    left: 0,
                    bottom: 0,
                    width: 32,
                    height: 32,
                }),
                .. Default::default()
            };
            uniforms.remove("equirectangularMap");
            uniforms.add_str_key("environmentMap", &dt.env_cubemap);
            env2irr_framebuffer.draw(&dt.cube.vertex_buffer, &dt.cube.index_buffer, &irradiance_program, &uniforms, &draw_parameters_v32).unwrap();

            (hdr2env_framebuffer, env2irr_framebuffer, dt)
        }
    }.build();

    // 点光源
    let point_lights = vec![
        PointLight::new_simple([-10.0, 10.0, 10.0], [300.0, 300.0, 300.0]),
        PointLight::new_simple([10.0, 10.0, 10.0], [300.0, 300.0, 300.0]),
        PointLight::new_simple([-10.0, -10.0, 10.0], [300.0, 300.0, 300.0]),
        PointLight::new_simple([10.0, -10.0, 10.0], [300.0, 300.0, 300.0])
    ];

    // 摄像机初始位置
    let camera = Camera::new(
        cgmath::Point3::new(0.0, 0.0, 3.0),
        cgmath::Rad::from(cgmath::Deg(-90.0)),
        cgmath::Rad::from(cgmath::Deg(0.0))
    );
    let controller = CameraController::new(1.0, 0.5);
    // 摄像机透视矩阵
    let projection_matrix: [[f32; 4]; 4] = cgmath::perspective(Deg(45.0), size.width as f32 / size.height as f32, 0.1, 100.0).into();

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: DepthTest::IfLessOrEqual,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let loop_context = LoopContext::new(camera, controller);

    start_loop(event_loop, loop_context, move |_: Option<Event<()>>, ctx, _| {
        tenants.with_mut(|fields| {
            let dt = fields.dt;
            // 摄像机观察矩阵
            let view_matrix: [[f32; 4]; 4] = ctx.camera.calc_matrix().into();
            let camera_position: [f32; 3] = ctx.camera.position.into();

            let mut uniforms = DynamicUniforms::new();
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("view", &view_matrix);
            uniforms.add_str_key("camPos", &camera_position);
            uniforms.add_str_key_value("albedo", UniformValue::Vec3([0.5, 0.0, 0.0]));
            uniforms.add_str_key_value("ao", UniformValue::Float(1.0));
            uniforms.add_str_key("irradianceMap", &dt.irradiance_cubemap);

            // 创建默认帧
            let mut target = display.draw();
            target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

            // 测试：显示等距柱状投影图
            // let mut test_uniforms = DynamicUniforms::new();
            // test_uniforms.add_str_key("image", &dt.hdr_env_texture);
            // test_uniforms.add_str_key_value("exposure", UniformValue::Float(1.0));
            // target.draw(&dt.plane.vertex_buffer, &dt.plane.index_buffer, &test_hdr_image_program, &test_uniforms, &draw_parameters).unwrap();
            // 测试：显示环境贴图
            // let mut test_uniforms = DynamicUniforms::new();
            // test_uniforms.add_str_key("projection", &projection_matrix);
            // test_uniforms.add_str_key("view", &view_matrix);
            // test_uniforms.add_str_key("camPos", &camera_position);
            // test_uniforms.add_str_key("image", &dt.env_cubemap);
            // test_uniforms.add_str_key_value("exposure", UniformValue::Float(1.0));
            // target.draw(&dt.cube.vertex_buffer, &dt.cube.index_buffer, &test_cubemap_program, &test_uniforms, &draw_parameters).unwrap();
            
            // target.finish().unwrap();
            // return Action::Continue;

            // 渲染球体
            for r in 0..sphere_rows {
                uniforms.add_str_key_value("metallic", UniformValue::Float(r as f32 / sphere_rows as f32));
                for c in 0..sphere_columns {
                    let roughness = (c as f32 / sphere_columns as f32).max(0.05).min(1.0);
                    uniforms.add_str_key_value("roughness", UniformValue::Float(roughness));

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

            // 渲染天空盒
            uniforms.clear();
            uniforms.add_str_key("projection", &projection_matrix);
            uniforms.add_str_key("view", &view_matrix);
            uniforms.add_str_key("environmentMap", &dt.env_cubemap);
            target.draw(&dt.cube.vertex_buffer, &dt.cube.index_buffer, &background_program, &uniforms, &draw_parameters).unwrap();

            target.finish().unwrap();

            Action::Continue
        })
    });
}
