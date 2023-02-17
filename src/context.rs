use std::{time::Duration, collections::HashMap, sync::Mutex};

use glium::glutin::event::{Event, WindowEvent};
use once_cell::sync::{Lazy};

use crate::{event::{keyboard::{KeyboardHandler, KeyboardInteract}}, camera::{Camera, CameraController}};

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
