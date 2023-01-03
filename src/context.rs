use std::{time::Duration, marker::PhantomPinned, pin::Pin, ptr::NonNull, collections::HashMap};

use glium::glutin::event::Event;

use crate::{event::{EventHandler, keyboard::{KeyboardHandler, KeyboardInteract}, mouse::MouseHandler}, camera::{Camera, CameraControllerProxy, CameraController}};


#[derive(Debug)]
pub struct LoopStore {
    pub camera_controller: CameraControllerProxy,
    _pin: PhantomPinned,
}

impl LoopStore {
    
    pub fn new(camera_controller: CameraControllerProxy) -> Pin<Box<Self>> {
        Box::pin(LoopStore {
            camera_controller,
            _pin: PhantomPinned,
        })
    }
}

pub enum ContextValue {
    F32(f32),
}

pub struct LoopContext {

    pub camera: Camera,

    camera_controller: CameraController,

    store: HashMap<String, ContextValue>,

    keyboard_handler: KeyboardHandler,
}

impl LoopContext {

    pub fn new(camera: Camera, camera_controller: CameraController) -> LoopContext {
        LoopContext {
            camera,
            camera_controller,
            store: HashMap::new(),
            keyboard_handler: KeyboardHandler::new(),
        }
    }

    // pub fn setup(&mut self, store: &'a LoopStore) {
    //     self.prepares.push(&store.camera_controller);
    // }

    // pub fn register_prepare(&mut self, prepare: &'a impl PrepareRender) {
    //     self.prepares.push(prepare);
    // }

    pub fn register_keyboard(&mut self, keyboard_interact: Box<dyn KeyboardInteract>) {
        self.keyboard_handler.register(keyboard_interact);
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        // self.event_handler.handle(event);

    }

    pub fn prepare_render(&mut self, frame_duration: Duration) {
    //     for prepare in self.prepares.iter() {
    //         prepare.prepare(&mut self.camera, frame_duration);
    //     }
    }

    pub fn get_from_store(&self, key: &str) -> Option<&ContextValue> {
        self.store.get(key)
    }

    pub fn set_to_store(&mut self, key: &str, value: ContextValue) {
        self.store.insert(key.to_string(), value);
    }
}

/// 准备渲染特征
/// 渲染前hook
pub trait PrepareRender {
    
    /// 暂时传一个帧间隔时间，以后有需求再做参数设计
    fn prepare(&self, camera: &mut Camera, frame_duration: Duration);
}
