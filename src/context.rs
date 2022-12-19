use std::time::Duration;

use glium::glutin::event::Event;

use crate::{event::{EventHandler, keyboard::KeyboardHandler, mouse::MouseHandler}, camera::{Camera, CameraControllerProxy}};


#[derive(Debug)]
pub struct LoopStore {
    pub camera_controller: CameraControllerProxy,
}

pub struct LoopContext<'a> {

    event_handler: EventHandler<'a>,

    pub camera: Camera,

    prepares: Vec<&'a dyn PrepareRender>,
}

impl <'a> LoopContext<'a> {

    pub fn new(event_handler: EventHandler<'a>, camera: Camera) -> LoopContext<'a> {
        LoopContext {
            event_handler,
            camera: camera,
            prepares: Vec::new(),
        }
    }

    pub fn setup(&mut self, store: &'static LoopStore) {
        self.prepares.push(&store.camera_controller);
    }

    pub fn register_prepare(&mut self, prepare: &'a impl PrepareRender) {
        self.prepares.push(prepare);
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        self.event_handler.handle(event);
    }

    pub fn prepare_render(&mut self, frame_duration: Duration) {
        for prepare in self.prepares.iter() {
            prepare.prepare(&mut self.camera, frame_duration);
        }
    }
}

/// 准备渲染特征
/// 渲染前hook
pub trait PrepareRender {
    
    /// 暂时传一个帧间隔时间，以后有需求再做参数设计
    fn prepare(&self, camera: &mut Camera, frame_duration: Duration);
}