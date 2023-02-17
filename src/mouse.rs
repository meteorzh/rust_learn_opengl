// use std::time::Duration;

// use glium::glutin::event::MouseScrollDelta;

// use crate::{camera::{CameraController, Camera}, event::mouse::MouseEventHandler};


// pub struct MouseController {

//     camera_controller: CameraController,
// }

// impl MouseController {

//     pub fn new(camera_controller: CameraController) -> MouseController {
//         MouseController { camera_controller: camera_controller }
//     }
// }

// impl MouseEventHandler for MouseController {

//     fn handle_motion(&mut self, delta: (f64, f64)) {
//         self.camera_controller.process_mouse(delta.0, delta.1);
//     }

//     fn handle_wheel(&mut self, delta: MouseScrollDelta) {
//         self.camera_controller.process_scroll(&delta);
//     }

//     fn update_camera(&mut self, camera: &mut Camera, frame_delta: Duration) {
//         self.camera_controller.update_camera(camera, frame_delta);
//     }
// }