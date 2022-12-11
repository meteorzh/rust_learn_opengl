use std::time::Duration;

use glium::glutin::event::Event;

use crate::{event::EventHandler, camera::Camera};


pub struct LoopContext<'a> {

    event_handler: EventHandler,

    camera: Camera,

    prepares: Vec<&'a dyn PrepareRender>,
}

impl <'a> LoopContext<'a> {

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