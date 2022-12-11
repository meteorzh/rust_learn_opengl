use std::time::Duration;

use glium::glutin::{event_loop::ControlFlow, event::{KeyboardInput, Event, WindowEvent, DeviceEvent}};

use crate::{keyboard::KeyboardController, mouse::MouseController, camera::Camera};

use self::mouse::MouseEventHandler;

pub mod mouse;

pub struct EventHandler {

    keyboard_controller: KeyboardController,

    mouse_handler: Box<dyn MouseEventHandler>,
}

impl EventHandler {

    pub fn new(keyboard_controller: KeyboardController, mouse_handler: Box<dyn MouseEventHandler>) -> EventHandler {
        EventHandler { keyboard_controller: keyboard_controller, mouse_handler: mouse_handler }
    }

    pub fn handle(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                // key input
                WindowEvent::KeyboardInput { input, .. } => {
                    self.keyboard_controller.process_keyboard(*input);
                },
                _ => {},
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.mouse_handler.handle_motion((delta.0, delta.1));
                },
                DeviceEvent::MouseWheel { delta } => {
                    self.mouse_handler.handle_wheel(*delta);
                },
                _ => {},
            },
            _ => {},
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, frame_delta: Duration) {
        self.mouse_handler.update_camera(camera, frame_delta);
    }
}