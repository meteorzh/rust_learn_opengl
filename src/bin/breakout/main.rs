
extern crate glium;
extern crate cgmath;

use cgmath::{Vector2};
use game::Game;
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event, VirtualKeyCode, KeyboardInput, ElementState, WindowEvent, DeviceEvent, MouseScrollDelta, MouseButton}, window::CursorGrabMode, dpi::LogicalSize}};

use resource_manager::ResourceManager;
use rust_opengl_learn::{Action, context::{LoopContext2D, EventHandler}, event::{keyboard::KeyboardInteract, mouse::MouseInteract}, start_loop_2d};

mod ball_object;
mod game;
mod game_level;
mod game_object;
mod particle_generator;
mod post_processor;
mod power_up;
mod resource_manager;
mod sprite_renderer;
mod text_renderer;

/// BreakOut 2D Game
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let mut breakout = Game::new(&display, size.width, size.height, Vector2::new(100.0, 20.0), 12.5);

    let player_controller = PlayerController::new();

    let mut loop_context = LoopContext2D::<EventHandlerType>::new();
    loop_context.register("player_controller", EventHandlerType::PlayerController(player_controller));

    start_loop_2d(event_loop, loop_context, move |_: Option<Event<()>>, ctx, dt| {
        // 创建默认帧
        let mut target = display.draw();
        target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

        // 获取player_controller
        let player_controller = {
            match ctx.get_mut("player_controller") {
                EventHandlerType::PlayerController(player_controller) => player_controller
            }
        };

        // 更新玩家挡板数据
        breakout.update_player(player_controller, dt);
        // 更新游戏中其它数据
        breakout.update(dt);

        breakout.render(&mut target, &display, dt);

        target.finish().unwrap();

        Action::Continue
    });
}

/// 键盘功能类型，包含多种键盘功能的支持
/// 为什么这样实现，可见文章：https://bennett.dev/dont-use-boxed-trait-objects-for-struct-internals/
enum EventHandlerType {

    PlayerController(PlayerController),
}

impl EventHandlerType {

    fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        match self {
            EventHandlerType::PlayerController(player_controller) => {
                player_controller.motion_interact(delta);
            }
        }
    }

    fn handle_mouse_wheel(&mut self, _: MouseScrollDelta) {
        
    }

    fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match self {
            EventHandlerType::PlayerController(player_controller) => {
                player_controller.input_interact(state, button);
            }
        }
    }

    fn handle_keyboard(&mut self, input: KeyboardInput) {
        match self {
            EventHandlerType::PlayerController(player_controller) => {
                player_controller.interact(input);
            }
        }
    }
}

impl EventHandler for EventHandlerType {

    fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    self.handle_keyboard(*input);
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    self.handle_mouse_input(*state, *button);
                },
                _ => {},
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    self.handle_mouse_motion(*delta);
                },
                DeviceEvent::MouseWheel { delta } => {
                    self.handle_mouse_wheel(*delta);
                },
                _ => {},
            },
            _ => {},
        }
    }
}

/// 玩家控制器
pub struct PlayerController {
    amount_left: f32,
    amount_right: f32,
    amount_mouse_x: f32,
    launch_trigger: bool,

    enter_pressed: bool,
    w_pressed: bool,
    s_pressed: bool,
}

impl PlayerController {
    pub fn new() -> Self {
        PlayerController {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_mouse_x: 0.0,
            launch_trigger: false,
            enter_pressed: false,
            w_pressed: false,
            s_pressed: false,
        }
    }
}

impl MouseInteract for PlayerController {

    fn motion_interact(&mut self, delta: (f64, f64)) {
        self.amount_mouse_x = delta.0 as f32;
    }

    fn wheel_interact(&mut self, _: MouseScrollDelta) {
        todo!()
    }

    fn input_interact(&mut self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            self.launch_trigger = state == ElementState::Released;
        }
    }
}

impl KeyboardInteract for PlayerController {

    fn init(&self) {
        todo!()
    }

    fn interact_keycodes(&self) -> Vec<VirtualKeyCode> {
        vec![VirtualKeyCode::A, VirtualKeyCode::D, VirtualKeyCode::Left, VirtualKeyCode::Right, VirtualKeyCode::Space, VirtualKeyCode::W, VirtualKeyCode::Return]
    }

    fn interact(&mut self, input: KeyboardInput) {
        let state = input.state;
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        if let Some(k) = input.virtual_keycode {
            if k == VirtualKeyCode::A || k == VirtualKeyCode::Left {
                self.amount_left = amount;
            } else if k == VirtualKeyCode::D || k == VirtualKeyCode::Right {
                self.amount_right = amount;
            } else if k == VirtualKeyCode::Space {
                self.launch_trigger = state == ElementState::Released;
            } else {
                // nothing
            }

            if state == ElementState::Released {
                if k == VirtualKeyCode::W && !self.w_pressed {
                    self.w_pressed = true;
                } else if k == VirtualKeyCode::S && !self.s_pressed {
                    self.s_pressed = true;
                } else if k == VirtualKeyCode::Return && !self.enter_pressed {
                    self.enter_pressed = true;
                }
            }
        }
    }
    
}
