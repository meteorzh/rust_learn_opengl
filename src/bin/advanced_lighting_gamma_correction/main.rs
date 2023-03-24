
extern crate glium;
extern crate cgmath;

use cgmath::{SquareMatrix, Point3, Matrix4};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState, Event}, window::CursorGrabMode}};

use rust_opengl_learn::{camera::{Camera, CameraController}, uniforms::DynamicUniforms, objects::{Plane}, material, create_program, start_loop, Action, event::{keyboard::{KeyboardInteract}}, context::{LoopContext, CONTEXT_STORE, ContextValue}, lights::PointLight};

fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 600).into();
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let obj_program = create_program("src/bin/advanced_lighting_gamma_correction/light.vert", "src/bin/advanced_lighting_gamma_correction/light.frag", &display);

    let floor = Plane::new("plane", 20.0, 20.0, -0.5_f32, &display, Point3::new(0.0, 0.0, 0.0), Matrix4::identity());

    let floor_texture = material::load_texture("src/wood.png".to_string(), &display).1;

    // 点光源
    let point_lights = {
        vec![
            PointLight::new_simple([-3.0, 0.0, 0.0], [0.25, 0.25, 0.25]),
            PointLight::new_simple([-1.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
            PointLight::new_simple([1.0, 0.0, 0.0], [0.75, 0.75, 0.75]),
            PointLight::new_simple([3.0, 0.0, 0.0], [1.0, 1.0, 1.0])
        ]
    };

    // 摄像机初始位置(0, 0, 3), pitch = 0°, yaw = -90°;
    let camera = Camera::new(
        cgmath::Point3::new(0_f32, 0_f32, 3_f32), 
        cgmath::Rad::from(cgmath::Deg(-90_f32)), 
        cgmath::Rad::from(cgmath::Deg(0_f32))
    );
    let controller = CameraController::new(1_f32, 0.5_f32);
    
    let projection_matrix = Into::<[[f32; 4]; 4]>::into(cgmath::perspective(cgmath::Deg(45.0), size.width as f32 / size.height as f32, 0.1_f32, 100.0));

    let draw_parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let mut loop_context = LoopContext::new(camera, controller);
    loop_context.register_keyboard(Box::new(KeyboardInteractor{}));

    start_loop(event_loop, loop_context, move |_: Option<Event<()>>, ctx, _| {
        // 摄像机观察矩阵
        let view_matrix = Into::<[[f32; 4]; 4]>::into(ctx.camera.calc_matrix());

        let camera_position = Into::<[f32; 3]>::into(ctx.camera.position);

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
        
        let mut uniforms = DynamicUniforms::new();
        uniforms.add(String::from("view"), &view_matrix);
        uniforms.add(String::from("projection"), &projection_matrix);
        uniforms.add(String::from("viewPos"), &camera_position);

        let store = CONTEXT_STORE.lock().unwrap();
        if let Some(ContextValue::BOOL(v)) = store.get_value("gamma") {
            uniforms.add_str_key("gamma", v);
        }

        for (i, light) in point_lights.iter().enumerate() {
            let light_key = format!("lights[{}]", i);
            light.add_to_uniforms(light_key.as_str(), &mut uniforms);
        }

        uniforms.add_str_key("floorTexture", &floor_texture);
        target.draw(&floor.vertex_buffer, &floor.index_buffer, &obj_program, &uniforms, &draw_parameters).unwrap();

        target.finish().unwrap();

        Action::Continue
    });
}

pub struct KeyboardInteractor;

impl KeyboardInteract for KeyboardInteractor {

    fn init(&self) {
        CONTEXT_STORE.lock().unwrap().set_value("gamma", ContextValue::BOOL(false))
    }

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode> {
        vec![VirtualKeyCode::G]
    }

    fn interact(&mut self, input: KeyboardInput) {
        if input.state == ElementState::Released {
            let mut store = CONTEXT_STORE.lock().unwrap();
            let blinn = store.get_value("gamma");
            if let Some(ContextValue::BOOL(v)) = blinn {
                let v = *v;
                println!("{} gamma校正", !v);
                store.set_value("gamma", ContextValue::BOOL(!v));
            }
        }
    }
}
