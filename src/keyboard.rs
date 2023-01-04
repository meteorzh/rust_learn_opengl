
use glium::glutin::{event::{KeyboardInput, VirtualKeyCode, ElementState}, event_loop};

use crate::camera::CameraController;


#[deprecated]
// 处理键盘输入事件
pub fn handle_keyboard_input(input: KeyboardInput, camera_controller: &mut CameraController) -> Option<event_loop::ControlFlow> {
    let virtual_keycode = input.virtual_keycode;
    if let None = virtual_keycode {
        return None;
    }

    let virtual_keycode = virtual_keycode.unwrap();
    let camera_handle = camera_controller.process_keyboard(virtual_keycode, input.state);
    if camera_handle {
        return None;
    }

    match virtual_keycode {
        VirtualKeyCode::Escape => {
            if input.state == ElementState::Released {
                return Some(event_loop::ControlFlow::Exit);
            }
        },
        _ => {
            println!("unsupported keyboard input: {}", input.scancode);
        }
    }

    None
}

// pub struct KeyboardController {
    
//     pub funcs: HashMap<VirtualKeyCode, Box<dyn Fn()>>,
// }

// impl KeyboardController {

//     pub fn new() -> KeyboardController {
//         KeyboardController {
//             funcs: HashMap::new()
//         }
//     }

//     pub fn register(&mut self, keycode: VirtualKeyCode, handler: Box<dyn Fn()>) {
//         self.funcs.insert(keycode, handler);
//     }

//     pub fn process_keyboard(&self, input: KeyboardInput) {
//         if let Some(code) = input.virtual_keycode {
//             if let Some(func) = self.funcs.get(&code) {
//                 func();
//             } else {
//                 println!("unsupported keyboard input: {:#?}", code);
//             }
//         } else {
//             println!("no virtual keycode found!");
//         }
//     }
// }