use std::collections::HashMap;

use glium::glutin::event::{VirtualKeyCode, KeyboardInput};


/// 键盘交互特征
pub trait KeyboardInteract {

    fn init(&self);

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode>;

    fn interact(&mut self, input: KeyboardInput);
}

pub struct KeyboardHandler {

    interacts: Vec<Box<dyn KeyboardInteract>>,

    interact_map: HashMap<VirtualKeyCode, usize>,
}

impl KeyboardHandler {

    pub fn new() -> KeyboardHandler {
        KeyboardHandler {
            interacts: Vec::new(),
            interact_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, interact: Box<dyn KeyboardInteract>) {
        interact.init();
        let index = self.interacts.len();
        for keycode in interact.interact_keycodes().iter() {
            self.interact_map.insert(*keycode, index);
        }
        self.interacts.push(interact);
    }

    pub fn process_keyboard(&mut self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            if let Some(index) = self.interact_map.get(&code) {
                if let Some(interact) = self.interacts.get_mut(*index) {
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