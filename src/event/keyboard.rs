use std::collections::HashMap;

use glium::glutin::event::{VirtualKeyCode, KeyboardInput};



/// 键盘交互特征
pub trait KeyboardInteract {

    fn interact_keycodes() -> Vec<VirtualKeyCode>;

    fn interact(&self, input: KeyboardInput);
}

pub struct KeyboardHandler {
    
    pub funcs: HashMap<VirtualKeyCode, Box<dyn Fn()>>,
}

impl KeyboardHandler {

    pub fn new() -> KeyboardHandler {
        KeyboardHandler {
            funcs: HashMap::new()
        }
    }

    pub fn register(&mut self, keycode: VirtualKeyCode, handler: Box<dyn Fn()>) {
        self.funcs.insert(keycode, handler);
    }

    pub fn process_keyboard(&self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            if let Some(func) = self.funcs.get(&code) {
                func();
            } else {
                println!("unsupported keyboard input: {:#?}", code);
            }
        } else {
            println!("no virtual keycode found!");
        }
    }
}