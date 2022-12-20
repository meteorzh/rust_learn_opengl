
use glium::glutin::{event::{Event, WindowEvent, DeviceEvent}};

use self::{mouse::{MouseHandler, MouseInteract}, keyboard::{KeyboardHandler, KeyboardInteract}};

pub mod mouse;
pub mod keyboard;

pub struct EventHandler<'a> {

    keyboard_handler: KeyboardHandler<'a>,

    mouse_handler: MouseHandler<'a>,
}

impl <'a> EventHandler<'a> {

    pub fn new(keyboard_handler: KeyboardHandler<'a>, mouse_handler: MouseHandler<'a>) -> EventHandler<'a> {
        EventHandler { keyboard_handler, mouse_handler }
    }

    pub fn handle(&self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                // key input
                WindowEvent::KeyboardInput { input, .. } => {
                    self.keyboard_handler.process_keyboard(*input);
                },
                _ => {},
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { .. } | DeviceEvent::MouseWheel { .. } => {
                    self.mouse_handler.process_mouse(event);
                },
                _ => {},
            },
            _ => {},
        }
    }

    pub fn register_keyboard(&mut self, interact: &'a impl KeyboardInteract) {
        self.keyboard_handler.register(interact);
    }

    pub fn register_mouse(&mut self, interact: &'a impl MouseInteract) {
        self.mouse_handler.register(interact);
    }
}