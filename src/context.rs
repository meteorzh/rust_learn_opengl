use std::{time::Duration, collections::HashMap, sync::Mutex};

use glium::glutin::event::{Event, WindowEvent, VirtualKeyCode, KeyboardInput, DeviceEvent, MouseScrollDelta};
use once_cell::sync::{Lazy};

use crate::{event::{keyboard::{KeyboardHandler, KeyboardInteract}, mouse::MouseInteract, MouseKeyboardInteract}, camera::{Camera, CameraController}};

/// 全局变量存储对象
#[derive(Debug)]
pub struct LoopStore {
    store: HashMap<String, ContextValue>,
}

impl LoopStore {
    
    pub fn new() -> Self {
        LoopStore { store: HashMap::new() }
    }

    pub fn get_value(&self, key: &str) -> Option<&ContextValue> {
        self.store.get(key)
    }

    pub fn set_value(&mut self, key: &str, value: ContextValue) {
        self.store.insert(key.to_string(), value);
    }
}

/// 全局变量存储值类型
#[derive(Debug)]
pub enum ContextValue {
    F32(f32),

    BOOL(bool),
}

/// 全局变量存储区域
/// TODO 这个地方能否不用Mutex
pub static CONTEXT_STORE: Lazy<Mutex<LoopStore>> = Lazy::new(|| {
    Mutex::new(LoopStore::new())
});



/// 渲染循环上下文对象
/// 支持摄像机功能
pub struct LoopContext {

    pub camera: Camera,

    camera_controller: CameraController,

    keyboard_handler: KeyboardHandler,
}

impl LoopContext {

    pub fn new(camera: Camera, camera_controller: CameraController) -> LoopContext {
        LoopContext {
            camera,
            camera_controller,
            keyboard_handler: KeyboardHandler::new(),
        }
    }

    /// 注册键盘交互功能
    pub fn register_keyboard(&mut self, keyboard_interact: Box<dyn KeyboardInteract>) {
        self.keyboard_handler.register(keyboard_interact);
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        self.camera_controller.proccess(event);

        match event {
            Event::WindowEvent { event, .. } => match event {
                // key input
                WindowEvent::KeyboardInput { input, .. } => {
                    self.keyboard_handler.process_keyboard(*input);
                },
                _ => {},
            },
            _ => {},
        }
    }

    pub fn prepare_render(&mut self, frame_duration: Duration) {
        self.camera_controller.update_camera(&mut self.camera, frame_duration);
    }
}

// /// 准备渲染特征
// /// 渲染前hook
// pub trait PrepareRender {
    
//     /// 暂时传一个帧间隔时间，以后有需求再做参数设计
//     fn prepare(&self, camera: &mut Camera, frame_duration: Duration);
// }

/// 键盘功能注册
pub trait KeyboardRegistry<T: KeyboardInteract> {

    fn register(&mut self, key: String, keyboard_interact: T);

    fn get(&self, key: String) -> &T;

    fn handle_keyboard(&mut self, input: KeyboardInput);
}

/// 键盘功能注册器
struct KeyboardRegister<T: KeyboardInteract> {

    interactors: HashMap<String, T>,

    interact_map: HashMap<VirtualKeyCode, String>,
}

/// 实现键盘注册器自身方法
impl <T: KeyboardInteract> KeyboardRegister<T> {
    
    pub fn new() -> KeyboardRegister<T> {
        KeyboardRegister {
            interactors: HashMap::new(),
            interact_map: HashMap::new(),
        }
    }
}

/// 实现键盘注册功能
impl <T: KeyboardInteract> KeyboardRegistry<T> for KeyboardRegister<T> {

    fn register(&mut self, key: String, keyboard_interact: T) {
        keyboard_interact.init();
        for keycode in keyboard_interact.interact_keycodes().iter() {
            self.interact_map.insert(*keycode, key.clone());
        }
        self.interactors.insert(key, keyboard_interact);
    }

    fn get(&self, key: String) -> &T {
        self.interactors.get(key.as_str()).unwrap()
    }

    fn handle_keyboard(&mut self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            if let Some(key) = self.interact_map.get(&code) {
                if let Some(interact) = self.interactors.get_mut(key) {
                    (*interact).interact(input);
                } else {
                    println!("error: no interact found for keycode {:#?}", code);
                }
            }
        } else {
            println!("no virtual keycode found!");
        }
    }
}


pub trait EventHandler {

    fn handle_event(&mut self, event: &Event<()>);
}



/// 2D渲染循环上下文
pub struct LoopContext2D<T: EventHandler> {

    handlers: HashMap<String, T>,
}

impl <T: EventHandler> LoopContext2D<T> {

    pub fn new() -> Self {
        LoopContext2D { handlers: HashMap::new() }
    }

    pub fn register(&mut self, key: &str, handler: T) {
        self.handlers.insert(key.to_string(), handler);
    }

    pub fn get(&self, key: &str) -> &T {
        self.handlers.get(key).unwrap()
    }

    pub fn get_mut(&mut self, key: &str) -> &mut T {
        self.handlers.get_mut(key).unwrap()
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        for (_, handler) in self.handlers.iter_mut() {
            handler.handle_event(event);
        }
    }
}
