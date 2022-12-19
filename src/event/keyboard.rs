use std::collections::HashMap;

use glium::glutin::event::{VirtualKeyCode, KeyboardInput};



/// 键盘交互特征
pub trait KeyboardInteract {

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode>;

    fn interact(&self, input: KeyboardInput);
}

pub struct KeyboardHandler<'a> {

    interacts: Vec<&'a dyn KeyboardInteract>,

    interact_map: HashMap<VirtualKeyCode, usize>,
}

impl <'a> KeyboardHandler<'a> {

    pub fn new() -> KeyboardHandler<'a> {
        KeyboardHandler {
            interacts: Vec::new(),
            interact_map: HashMap::new(),
        }
    }

    pub fn register2(&mut self, interact: &'a impl KeyboardInteract) {
        let index = self.interacts.len();
        self.interacts.push(interact);
        for keycode in interact.interact_keycodes().iter() {
            self.interact_map.insert(*keycode, index);
        }
    }

    pub fn process_keyboard(&self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            if let Some(index) = self.interact_map.get(&code) {
                if let Some(interact) = self.interacts.get(*index) {
                    (*interact).interact(input);
                } else {
                    println!("error: no interact found for keycode {:#?}", code);
                }
            } else {
                println!("unsupported keyboard input: {:#?}", code);
            }
        } else {
            println!("no virtual keycode found!");
        }
    }
}