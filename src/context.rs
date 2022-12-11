use glium::glutin::event::Event;

use crate::{event::EventHandler, camera::Camera};


pub struct LoopContext {

    event_handler: EventHandler,

    camera: Camera,
}

impl LoopContext {

    pub fn handle_event(&mut self, event: &Event<()>) {
        self.event_handler.handle(event);
    }

    pub fn prepare_render(&mut self) {
        
    }
}