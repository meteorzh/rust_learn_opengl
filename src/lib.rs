use std::{collections::{HashMap, hash_map::Entry}, rc::Rc, sync::Arc, fs::{self, File}, io::{Cursor, BufReader}, future, time::{Instant, Duration}, pin::Pin};

use camera::CameraController;
use cgmath::{Vector3, Zero, Vector2};
use context::{LoopContext, LoopStore};
use event::{EventHandler, keyboard::KeyboardInteract, mouse::MouseInteract};
use futures::lock::Mutex;
use glium::{implement_vertex, vertex::VertexBufferAny, index::{IndexBufferAny, self}, Display, IndexBuffer, texture::CompressedSrgbTexture2d, program::{SourceCode, ProgramCreationInput}, Program, glutin::{event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent, DeviceEvent, StartCause, KeyboardInput, VirtualKeyCode, ElementState}}};
use material::{Material, MaterialLoader};
use obj::{ObjData, ObjMaterial};
use once_cell::sync::OnceCell;

pub mod utils;
pub mod camera;
pub mod lights;
pub mod uniforms;
pub mod material;
pub mod objects;
pub mod keyboard;
pub mod objectsv2;
pub mod event;
pub mod mouse;
pub mod context;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture: [f32; 2],
}

implement_vertex!(Vertex, position, normal, texture);

#[derive(Copy, Clone)]
pub struct ColorVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

implement_vertex!(ColorVertex, position, color);


pub struct Model {
    pub vertex_buffer: VertexBufferAny,
    pub index_buffer: IndexBufferAny,
    pub material: Option<Rc<Material>>,
}

/**
 * 将一个模型数据加载为多个绘制单位
 */
pub fn load_wavefront_obj_as_models(display: &Display, basepath: &str, obj_file: &str) -> Vec<Model> {
    let mut obj_path = String::from(basepath);
    obj_path.push_str(obj_file);
    let mut obj = obj::Obj::load(obj_path).unwrap();
    // 需要手动加载材质
    obj.load_mtls().unwrap();
    let data = obj.data;

    // 加载材质
    let mut material_loader = MaterialLoader::new();
    material_loader.parse_and_load(&data.material_libs, basepath, display);

    let mut models = Vec::new();
    for obj in data.objects.iter() {
        for group in obj.groups.iter() {
            // 目前认为相同材质的所有面（具有法向量和材质坐标的顶点）可以一起绘制，放入一个model
            let mut cache = HashMap::new();
            let mut vertex_data = Vec::new();
            let mut index_data = Vec::new();

            // 创建材质
            let material = match &group.material {
                Some(material) => {
                    let name = match material {
                        ObjMaterial::Ref(name) => name,
                        ObjMaterial::Mtl(meterial) => &meterial.name,
                    };
                    material_loader.find_in_cache(name.clone())
                },
                None => None,
            };
            for poly in group.polys.iter() {
                // 按面进行绘制
                // 创建顶点
                for index in poly.0.iter() {
                    let i = match cache.get(index) {
                        Some(vertex_index) => *vertex_index,
                        None => {
                            let vertex = Vertex {
                                position: data.position[index.0],
                                normal: match index.2 {
                                    Some(i) => data.normal[i],
                                    None => Into::<[f32; 3]>::into(Vector3::zero()),
                                },
                                texture: match index.1 {
                                    Some(i) => data.texture[i],
                                    None => Into::<[f32; 2]>::into(Vector2::zero()),
                                }
                            };
                            let vertex_index = vertex_data.len();
                            vertex_data.push(vertex);
                            cache.insert(index, vertex_index);
                            vertex_index
                        }
                    };
                    index_data.push(i as u16);
                }
            }
            let vertex_buffer = glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap().into();
            let index_buffer = IndexBuffer::new(display, index::PrimitiveType::TrianglesList, &index_data).unwrap().into();
            models.push(Model {
                vertex_buffer: vertex_buffer,
                index_buffer: index_buffer,
                material: material,
            });
        }
    }
    models
}

pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(event_loop: EventLoop<()>, store: Pin<Box<LoopStore>>, mut ctx: LoopContext<'static>, mut render_func: F) 
    where F: 'static + FnMut(Option<Event<'_, ()>>, &mut LoopContext<'_>) -> Action {

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        let mut render = false;
        let mut raw_event: Option<Event<()>> = None;

        if let Some(event) = event.to_static() {
            match &event {
                Event::WindowEvent { event, .. } => match event {
                    // Break from the main loop when the window is closed.
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    // key input
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(cf) = handle_keyboard_input(*input) {
                            *control_flow = cf;
                        }
                    },
                    _ => {},
                },
                Event::NewEvents(cause) => match cause {
                    StartCause::ResumeTimeReached { .. } => {
                        // 帧时间限制达到后可以渲染
                        render = true;
                    },
                    StartCause::Init => {
                        // 初始化时可以渲染
                        render = true;

                        // 设置context
                        ctx.setup(Pin::get_ref(store.as_ref()));
                    },
                    _ => {},
                },
                _ => {},
            }
            ctx.handle_event(&event);

            if !render {
                return;
            }

            raw_event = Some(event);
        }
        let current = Instant::now();
        let frame_duration = current.duration_since(last_frame);

        ctx.prepare_render(frame_duration);

        match render_func(raw_event, &mut ctx) {
            Action::Continue => {
                // 下一帧时间
                let next_frame_time = current + Duration::from_nanos(16_666_667);
                *control_flow = ControlFlow::WaitUntil(next_frame_time);
                // 更新上一帧时间变量
                last_frame = current;
            },
            Action::Stop => *control_flow = ControlFlow::Exit
        }
    });
}

fn handle_keyboard_input(input: KeyboardInput) -> Option<ControlFlow> {
    if let Some(keycode) = input.virtual_keycode {
        match keycode {
            VirtualKeyCode::Escape => {
                if input.state == ElementState::Released {
                    return Some(ControlFlow::Exit);
                }
            },
            _ => {}
        }
    }

    None
}




pub fn create_program(vert_source_path: &str, frag_source_path: &str, display: &Display) -> Program {
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

pub fn create_program_vgf(vert_source_path: &str, geometry_source_path: &str, frag_source_path: &str, display: &Display) -> Program {
    let obj_vert_source = fs::read_to_string(vert_source_path).unwrap();
    let obj_frag_source = fs::read_to_string(frag_source_path).unwrap();
    let geometry_source = fs::read_to_string(geometry_source_path).unwrap();
    let obj_shader_source = SourceCode {
        vertex_shader: obj_vert_source.as_str(),
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: Some(geometry_source.as_str()),
        fragment_shader: obj_frag_source.as_str(),
    };

    glium::Program::new(
        display,
        ProgramCreationInput::from(obj_shader_source)
    ).unwrap()
}