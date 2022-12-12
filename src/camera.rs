use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use cgmath::{Point3, Rad, Matrix4, Vector3};

use cgmath::{prelude::*};
use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::event::{VirtualKeyCode, ElementState, MouseScrollDelta, Event, WindowEvent, DeviceEvent, KeyboardInput};

use crate::context::PrepareRender;
use crate::event::keyboard::KeyboardInteract;
use crate::event::mouse::MouseInteract;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<
        V: Into<Point3<f32>>,
        Y: Into<Rad<f32>>,
        P: Into<Rad<f32>>,
    >(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    // 观察矩阵
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                self.yaw.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin(),
            ).normalize(),
            Vector3::unit_y(),
        )
    }

    // 观察矩阵-移除位移
    pub fn calc_matrix_no_move(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(
                self.yaw.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin(),
            ).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn direction(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.0.cos(),
            self.pitch.0.sin(),
            self.yaw.0.sin(),
        ).normalize()
    }
}




#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn proccess(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                // key input
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        self.process_keyboard(key, input.state);
                    }
                },
                _ => {},
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.process_mouse(delta.0, delta.1)
                },
                DeviceEvent::MouseWheel { delta } => {
                    self.process_scroll(&delta);
                },
                _ => {},
            },
            _ => {},
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed { 2.0 } else { 0.0 };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}


/// 摄像机控制类代理
/// 保持摄像机控制类的内部可变性
pub struct CameraControllerProxy {

    controller: RefCell<CameraController>,
}

impl CameraControllerProxy {

    pub fn new(controller: CameraController) -> CameraControllerProxy {
        CameraControllerProxy { controller: RefCell::new(controller) }
    }
}

impl KeyboardInteract for CameraControllerProxy {

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode> {
        vec![VirtualKeyCode::Up, VirtualKeyCode::Down, VirtualKeyCode::Left, VirtualKeyCode::Right, VirtualKeyCode::Space, VirtualKeyCode::LShift]
    }

    fn interact(&self, input: KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            self.controller.borrow_mut().process_keyboard(keycode, input.state);
        }
    }
}

impl MouseInteract for CameraControllerProxy {

    fn motion_interact(&self, delta: (f64, f64)) {
        self.controller.borrow_mut().process_mouse(delta.0, delta.1);
    }

    fn wheel_interact(&self, delta: MouseScrollDelta) {
        self.controller.borrow_mut().process_scroll(&delta);
    }
}

impl PrepareRender for CameraControllerProxy {

    fn prepare(&self, camera: &mut Camera, frame_duration: Duration) {
        self.controller.borrow_mut().update_camera(camera, frame_duration);
    }
}