
extern crate glium;
extern crate cgmath;

use std::{collections::{HashMap, VecDeque}, fs, time::Duration};

use cgmath::{Point2, Vector2, Vector3, Matrix4, Deg, EuclideanSpace, InnerSpace, Vector4};
#[allow(unused_imports)]
use glium::{glutin::{self, event, window, event_loop}, Surface};
use glium::{glutin::{event::{Event, VirtualKeyCode, KeyboardInput, ElementState, WindowEvent, DeviceEvent, MouseScrollDelta, MouseButton}, window::CursorGrabMode, dpi::LogicalSize}, Program, texture::{Texture2d}, VertexBuffer, Display, IndexBuffer, index::PrimitiveType, uniforms::UniformValue, DrawParameters, Blend, BackfaceCullingMode, BlendingFunction, LinearBlendingFactor};

use rand::{rngs::StdRng, SeedableRng, Rng};
use rust_opengl_learn::{uniforms::DynamicUniforms, create_program, Action, context::{LoopContext2D, EventHandler}, objectsv2::RawVertexP2T, material, event::{keyboard::KeyboardInteract, mouse::MouseInteract}, start_loop_2d, utils::clamp_vec2};

/// BreakOut 2D Game
fn main() {
    let event_loop = event_loop::EventLoop::new();
    let size = LogicalSize::<u32>::new(800, 600);
    let wb = window::WindowBuilder::new().with_inner_size(size);
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    display.gl_window().window().set_cursor_grab(CursorGrabMode::Confined).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let mut breakout = Game::new(size.width, size.height, Vector2::new(100.0, 20.0), 12.5);

    breakout.init(&display);

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

        breakout.render(&mut target);

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
struct PlayerController {
    amount_left: f32,
    amount_right: f32,
    amount_mouse_x: f32,
    launch_trigger: bool,
}

impl PlayerController {
    pub fn new() -> Self {
        PlayerController {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_mouse_x: 0.0,
            launch_trigger: false,
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
        vec![VirtualKeyCode::A, VirtualKeyCode::D, VirtualKeyCode::Left, VirtualKeyCode::Right, VirtualKeyCode::Space]
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
            }
        }
    }
    
}

/// 游戏状态
#[derive(PartialEq, Eq)]
enum GameState {
    GAME_ACTIVE,
    GAME_MENU,
    GAME_WIN
}

/// 资源管理器
struct ResourceManager<'a> {
    
    textures: HashMap<&'a str, Texture2d>,
}

impl <'a> ResourceManager<'a> {

    pub fn new() -> Self {
        Self { textures: HashMap::new() }
    }

    pub fn load_texture(&mut self, key: &'a str, path: &str, display: &Display) {
        self.textures.insert(key, material::load_texture2(path.to_string(), display).1);
    }

    pub fn get_texture(&self, key: &str) -> &Texture2d {
        self.textures.get(key).unwrap()
    }
}

static INITIAL_BALL_VELOCITY: Vector2<f32> = Vector2::new(100.0, -350.0);
/// 游戏类
struct Game<'a> {
    state: GameState,
    keys: [bool; 1024],
    width: u32,
    height: u32,
    projection: Matrix4<f32>,
    sprite_renderer: Option<SpriteRenderer<'a>>,
    resource_manager: ResourceManager<'a>,
    levels: Vec<GameLevel>,
    level: u32,
    player: GameObject,
    ball: BallObject,
    particle_generator: Option<ParticleGenerator<'a>>,
}

impl <'a> Game<'a> {

    pub fn new(width: u32, height: u32, player_size: Vector2<f32>, ball_radius: f32) -> Self {
        let player_position = Point2::new(width as f32 / 2.0 - player_size.x / 2.0, height as f32 - player_size.y);
        // 计算球的初始位置，球的位置应该在挡板上边
        let ball_position = player_position + Vector2::new(player_size.x / 2.0 - ball_radius, -ball_radius * 2.0);
        Self {
            state: GameState::GAME_ACTIVE,
            keys: [false; 1024],
            width,
            height,
            projection: cgmath::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0),
            sprite_renderer: None,
            resource_manager: ResourceManager::new(),
            levels: Vec::new(),
            level: 0,
            player: GameObject {
                position: player_position,
                size: player_size,
                velocity: Vector2::new(500.0, 0.0),
                color: Vector3::new(1.0, 1.0, 1.0),
                rotation: Deg(0.0),
                is_solid: true,
                destroyed: false,
                texture_key: "paddle".to_string(),
            },
            ball: BallObject::new(ball_position, INITIAL_BALL_VELOCITY, "face".to_string(), ball_radius),
            particle_generator: None
        }
    }

    pub fn init(&mut self, display: &Display) {
        let sprite_program = create_program("src/bin/breakout/test.vert", "src/bin/breakout/test.frag", display);
        let particle_program = create_program("src/bin/breakout/particle.vert", "src/bin/breakout/particle.frag", display);

        // 加载纹理
        self.resource_manager.load_texture("background", "src/textures/background.jpg", display);
        self.resource_manager.load_texture("face", "src/awesomeface.png", display);
        self.resource_manager.load_texture("block", "src/textures/block.png", display);
        self.resource_manager.load_texture("block_solid", "src/textures/block_solid.png", display);
        self.resource_manager.load_texture("paddle", "src/textures/paddle.png", display);
        self.resource_manager.load_texture("particle", "src/textures/particle.png", display);
        self.sprite_renderer = Some(SpriteRenderer::new(
            display, 
            sprite_program
        ));
        // 加载关卡
        let level_one = GameLevel::new("src/bin/breakout/levels/one.lvl", self.width, self.height / 2);
        let level_two = GameLevel::new("src/bin/breakout/levels/two.lvl", self.width, self.height / 2);
        let level_three = GameLevel::new("src/bin/breakout/levels/three.lvl", self.width, self.height / 2);
        let level_four = GameLevel::new("src/bin/breakout/levels/four.lvl", self.width, self.height / 2);
        self.levels.push(level_one);
        self.levels.push(level_two);
        self.levels.push(level_three);
        self.levels.push(level_four);

        // 粒子生成器
        self.particle_generator = Some(ParticleGenerator::new(
            display,
            particle_program,
            "particle".to_string(),
            500
        ));
    }

    fn update_player(&mut self, player_controller: &mut PlayerController, dt: Duration) {
        if self.state != GameState::GAME_ACTIVE {
            // 游戏尚未开始，不做操作
            return;
        }
        // 计算移动量
        let amount = {
            // 鼠标优先
            if player_controller.amount_mouse_x != 0.0 {
                let amount = player_controller.amount_mouse_x * 0.7;
                player_controller.amount_mouse_x = 0.0;
                amount
            } else {
                player_controller.amount_right - player_controller.amount_left
            }
        };
        if amount != 0.0 {
            // 有移动量才需要更新挡板位置
            let amount = amount * self.player.velocity.x * dt.as_secs_f32();
            let mut move_stuck_ball = false;
            // 更新玩家挡板的位置
            let position = &mut self.player.position;
            if amount > 0.0 {
                // 往右
                let max_x = self.width as f32 - self.player.size.x;
                if position.x <= max_x {
                    position.x = num_traits::clamp_max(position.x + amount, max_x);
                    move_stuck_ball = true;
                }
            } else {
                // 往左
                if position.x >= 0.0 {
                    position.x = num_traits::clamp_min(position.x + amount, 0.0);
                    move_stuck_ball = true;
                }
            }

            // 若球被固定，更新球的位置
            if self.ball.stuck && move_stuck_ball {
                self.ball.game_object.position.x = position.x + self.player.size.x / 2.0 - self.ball.radius;
            }
        }

        // 如果球当前是被固定的状态，且标记为发射，则修改球的固定状态
        if self.ball.stuck && player_controller.launch_trigger {
            self.ball.stuck = false;
        }
        player_controller.launch_trigger = false;
    }

    fn update(&mut self, dt: Duration) {
        // 绘制球
        self.ball.move_ball(self.width, dt);

        // 检测碰撞
        self.do_collisions();

        // 检测球是否超出底部边界
        if self.ball.game_object.position.y >= self.height as f32 {
            // 重置关卡和玩家挡板
            self.reset_level();
            self.reset_player();
        }

        // 更新粒子
        if let Some(particle_generator) = &mut self.particle_generator {
            let offset = Vector2::new(self.ball.radius / 2.0, self.ball.radius / 2.0);
            particle_generator.update(&self.ball.game_object, 2, offset, dt);
        }
    }

    fn reset_level(&mut self) {
        let current_level = self.levels.get_mut(self.level as usize).unwrap();
        current_level.reset();
    }

    fn reset_player(&mut self) {
        // 重置球和玩家的位置
        let player_position = Point2::new(self.width as f32 / 2.0 - self.player.size.x / 2.0, self.height as f32 - self.player.size.y);
        self.player.position = player_position;
        // 计算球的初始位置，球的位置应该在挡板上边
        let ball_position = player_position + Vector2::new(self.player.size.x / 2.0 - self.ball.radius, -self.ball.radius * 2.0);
        self.ball.reset(ball_position, INITIAL_BALL_VELOCITY);
        self.ball.stuck = true;
    }

    fn do_collisions(&mut self) {
        // 检测当前关卡中所有砖块的碰撞
        let level = self.levels.get_mut(self.level as usize).unwrap();
        for brick in level.bricks.iter_mut() {
            if !brick.destroyed {
                // 碰撞检测
                if let Some((direction, vec)) = check_collisions_aabb_round(&self.ball, brick) {
                    if !brick.is_solid {
                        // 摧毁砖块
                        brick.destroyed = true;
                    }
                    // 处理碰撞
                    let ball_velocity = &mut self.ball.game_object.velocity;
                    let ball_position = &mut self.ball.game_object.position;
                    match direction {
                        Direction::Left | Direction::Right => {
                            ball_velocity.x = -ball_velocity.x;
                            // 重定位
                            let penetration = self.ball.radius - num_traits::abs(vec.x);
                            if direction == Direction::Left {
                                ball_position.x += penetration;
                            } else {
                                ball_position.x -= penetration;
                            }
                        },
                        Direction::Up | Direction::Down => {
                            ball_velocity.y = -ball_velocity.y;
                            let penetration = self.ball.radius - num_traits::abs(vec.y);
                            if direction == Direction::Up {
                                ball_position.y -= penetration;
                            } else {
                                ball_position.y += penetration;
                            }
                        },
                    }
                }
            }
        }

        // 判断球和玩家挡板的碰撞
        if !self.ball.stuck {
            if let Some((_, _)) = check_collisions_aabb_round(&self.ball, &self.player) {
                let player_center = self.player.position.x + self.player.size.x / 2.0;
                // 检查碰到挡板哪个位置，并根据位置来改变速度
                let distance = self.ball.game_object.position.x + self.ball.radius - player_center;
                let percentage = distance / (self.player.size.x / 2.0);
                // 根据结果移动
                let strength = 2.0_f32;
                let old_velocity = self.ball.game_object.velocity;
                // let tmp_velocity = Vector2::new(INITIAL_BALL_VELOCITY.x * percentage * strength, -old_velocity.y);
                let tmp_velocity = Vector2::new(INITIAL_BALL_VELOCITY.x * percentage * strength, -num_traits::abs(old_velocity.y)); // 处理粘板问题
                self.ball.game_object.velocity = tmp_velocity.normalize() * old_velocity.magnitude();
            }
        }
    }

    fn render<T: Surface>(&self, surface: &mut T) {
        if let Some(renderer) = &self.sprite_renderer {
            // 绘制背景
            renderer.draw_sprite(
                surface, 
                self.resource_manager.get_texture("background"), 
                Point2::new(0.0, 0.0), 
                Vector2::new(self.width as f32, self.height as f32), 
                Deg(0.0), 
                Vector3::new(1.0, 1.0, 1.0),
                self.projection
            );
            // 绘制关卡
            let level = self.levels.get(self.level as usize).unwrap();
            level.draw(renderer, surface, &self.resource_manager, self.projection);
            // 绘制挡板
            self.player.draw(renderer, surface, &self.resource_manager, self.projection);

            // 如果游戏激活，需要绘制球
            if self.state == GameState::GAME_ACTIVE {
                self.ball.draw(renderer, surface, &self.resource_manager, self.projection);
            }
        }
        // 渲染粒子
        if let Some(particle_generator) = &self.particle_generator {
            particle_generator.draw(surface, &self.resource_manager, self.projection);
        }
    }
}


#[derive(PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,

    Right,

    Down,

    Left,
}

static COMPASS: [(Direction, Vector2<f32>); 4] = [
    (Direction::Up, Vector2::new(0.0, 1.0)),
    (Direction::Right, Vector2::new(1.0, 0.0)),
    (Direction::Down, Vector2::new(0.0, -1.0)),
    (Direction::Left, Vector2::new(-1.0, 0.0)),
];

/// 判断碰撞方向，vector向量为球碰撞的方向向量，其中四个对比方向也是按照球的角度来思考的
fn vector_direction(vector: Vector2<f32>) -> Direction {
    let mut max = 0.0_f32;
    let mut best_match = Direction::Up;

    let normalize = vector.normalize();
    for direction in COMPASS.iter() {
        let dot = normalize.dot(direction.1);
        if dot > max {
            max = dot;
            best_match = direction.0;
        }
    }

    best_match
}


fn check_collisions_aabb_aabb(one: &GameObject, two: &GameObject) -> bool {
    // 判断x轴方向碰撞
    let collisions_x = (one.position.x + one.size.x) >= two.position.x && one.position.x <= (two.position.x + two.size.x);
    let collisions_y = (one.position.y + one.size.y) >= two.position.y && one.position.y <= (two.position.y + two.size.y);
    
    collisions_x && collisions_y
}

fn check_collisions_aabb_round(one: &BallObject, two: &GameObject) -> Option<(Direction, Vector2<f32>)> {
    // 圆心
    let center = Point2::new(one.game_object.position.x + one.radius, one.game_object.position.y + one.radius);
    // aabb的信息
    let half_extends = Vector2::new(two.size.x / 2.0, two.size.y / 2.0);
    let aabb_center = Vector2::new(
        two.position.x + half_extends.x,
        two.position.y + half_extends.y
    );

    let difference = center - aabb_center;
    // difference.clamp
    let clamped = clamp_vec2(difference.to_vec(), -half_extends, half_extends);
    // aabb的中心加clamp的向量为最近点的位置
    let closest = aabb_center + clamped;
    // 根据最近点到圆心的向量的模和半径对比获得碰撞结果
    let result = closest - center.to_vec();
    
    if result.magnitude() <= one.radius {
        Some((vector_direction(result), result))
    } else {
        None
    }
}



struct SpriteRenderer<'a> {
    shader: Program,
    vertex_buffer: VertexBuffer<RawVertexP2T>,
    index_buffer: IndexBuffer<u16>,
    draw_parameters: DrawParameters<'a>,
}

impl <'a> SpriteRenderer<'a> {

    pub fn new(display: &Display, shader: Program) -> Self {
        Self { 
            shader: shader,
            vertex_buffer: VertexBuffer::new(display, &[
                RawVertexP2T { position: [0.0, 1.0], texture: [0.0, 1.0] },
                RawVertexP2T { position: [1.0, 0.0], texture: [1.0, 0.0] },
                RawVertexP2T { position: [0.0, 0.0], texture: [0.0, 0.0] },

                RawVertexP2T { position: [0.0, 1.0], texture: [0.0, 1.0] },
                RawVertexP2T { position: [1.0, 1.0], texture: [1.0, 1.0] },
                RawVertexP2T { position: [1.0, 0.0], texture: [1.0, 0.0] },
            ]).unwrap(),
            index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap(),
            draw_parameters: DrawParameters {
                blend: Blend::alpha_blending(),
                backface_culling: BackfaceCullingMode::CullClockwise,
                .. Default::default()
            },
        }
    }

    fn draw_sprite<S: Surface>(&self, surface: &mut S, texture: &Texture2d, position: Point2<f32>, size: Vector2<f32>, rotate: Deg<f32>, color: Vector3<f32>, projection: Matrix4<f32>) {
        let mut uniforms = DynamicUniforms::new();

        let model = Matrix4::from_translation(Vector3::new(position.x, position.y, 0.0));
        let model = model * Matrix4::from_translation(Vector3::new(0.5 * size.x, 0.5 * size.y, 0.0));
        let model = model * Matrix4::from_angle_z(rotate);
        let model = model * Matrix4::from_translation(Vector3::new(-0.5 * size.x, -0.5 * size.y, 0.0));
        let model = model * Matrix4::from_nonuniform_scale(size.x, size.y, 1.0);

        uniforms.add_str_key_value("projection", UniformValue::Mat4(projection.into()));
        uniforms.add_str_key_value("model", UniformValue::Mat4(model.into()));
        uniforms.add_str_key_value("spriteColor", UniformValue::Vec3(color.into()));
        uniforms.add_str_key("image", texture);

        surface.draw(&self.vertex_buffer, &self.index_buffer, &self.shader, &uniforms, &self.draw_parameters).unwrap();
    }
}

/// 无砖块
const EMPTY: u8 = 0;
/// 坚硬的砖块，不可摧毁
const HARD_BRICK: u8 = 1;



/// 游戏关卡
struct GameLevel {
    bricks: Vec<GameObject>,
    destroyed_count: u32,
    destroyable_count: u32,
}

impl GameLevel {

    fn new(file: &str, level_width: u32, level_height: u32) -> Self {
        let level_str_data = fs::read_to_string(file).unwrap();
        let lines = level_str_data.lines();
        let mut tile_data: Vec<Vec<u8>> = Vec::new();
        for line in lines {
            let chars = line.split(" ");
            let mut rows = Vec::<u8>::new();
            for char in chars {
                rows.push(str::parse::<u8>(char).unwrap());
            }
            tile_data.push(rows);
        }

        let mut level = GameLevel {
            bricks: Vec::new(),
            destroyed_count: 0,
            destroyable_count: 0,
        };
        if tile_data.len() > 0 {
            level.init(tile_data, level_width, level_height);
        }

        level
    }

    fn reset(&mut self) {
        // 重置关卡
        self.destroyed_count = 0;
        self.destroyable_count = self.bricks.len() as u32;
        for brick in self.bricks.iter_mut() {
            brick.destroyed = false;
        }
    }

    // 渲染关卡
    fn draw<T: Surface>(&self, renderer: &SpriteRenderer, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        // 渲染所有未被破坏的砖块
        for brick in self.bricks.iter() {
            if !brick.destroyed {
                brick.draw(renderer, surface, resource_manager, projection);
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.destroyed_count == self.destroyable_count
    }

    fn init(&mut self, tile_data: Vec<Vec<u8>>, level_width: u32, level_height: u32) {
        // 计算每个砖块的大小
        let height = tile_data.len();
        let width = tile_data.get(0).unwrap().len();
        let unit_width = level_width as f32 / width as f32;
        let unit_height = level_height as f32 / height as f32;

        for (y, row) in tile_data.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                // 区分砖块类型
                if *col == HARD_BRICK {
                    // 坚硬的砖块
                    self.bricks.push(GameObject {
                        position: Point2::new(unit_width * x as f32, unit_height * y as f32),
                        size: Vector2::new(unit_width, unit_height),
                        velocity: Vector2::new(0.0, 0.0),
                        color: Vector3::new(0.8, 0.8, 0.7),
                        rotation: Deg(0.0),
                        is_solid: true,
                        destroyed: false,
                        texture_key: "block_solid".to_string(),
                    });
                } else if *col > 1 {
                    self.destroyable_count += 1;
                    let color = {
                        if *col == 2 {
                            Vector3::new(0.2, 0.6, 1.0)
                        } else if *col == 3 {
                            Vector3::new(0.0, 0.7, 0.0)
                        } else if *col == 4 {
                            Vector3::new(0.8, 0.8, 0.4)
                        } else if *col == 5 {
                            Vector3::new(1.0, 0.5, 0.0)
                        } else {
                            Vector3::new(1.0, 1.0, 1.0)
                        }
                    };
                    // 可被摧毁的砖块
                    self.bricks.push(GameObject {
                        position: Point2::new(unit_width * x as f32, unit_height * y as f32),
                        size: Vector2::new(unit_width, unit_height),
                        velocity: Vector2::new(0.0, 0.0),
                        color,
                        rotation: Deg(0.0),
                        is_solid: false,
                        destroyed: false,
                        texture_key: "block".to_string(),
                    });
                }
            }
        }
    }
}


/// 游戏物体对象
struct GameObject {
    position: Point2<f32>,
    size: Vector2<f32>,
    velocity: Vector2<f32>,
    color: Vector3<f32>,
    rotation: Deg<f32>,
    is_solid: bool,
    destroyed: bool,
    texture_key: String,
}

impl GameObject {
    
    fn draw<T: Surface>(&self, renderer: &SpriteRenderer, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        let texture = resource_manager.get_texture(self.texture_key.as_str());
        renderer.draw_sprite(surface, texture, self.position, self.size, self.rotation, self.color, projection);
    }
}


/// 游戏球
struct BallObject {
    game_object: GameObject,
    radius: f32,
    stuck: bool,
}

impl BallObject {

    pub fn new(position: Point2<f32>, velocity: Vector2<f32>, texture_key: String, radius: f32) -> Self {
        Self {
            game_object: GameObject {
                position,
                size: Vector2::new(radius * 2.0, radius * 2.0),
                velocity,
                color: Vector3::new(1.0, 1.0, 1.0),
                rotation: Deg(0.0),
                is_solid: true,
                destroyed: false,
                texture_key,
            },
            radius: radius,
            stuck: true
        }
    }

    fn move_ball(&mut self, window_width: u32, dt: Duration) -> Point2<f32> {
        let position = &mut self.game_object.position;
        if !self.stuck {
            let velocity = &mut self.game_object.velocity;
            let size = self.game_object.size;
            *position += *velocity * dt.as_secs_f32();
            // println!("{}, {}", position.x, position.y);
            // 检查是否在窗口边界外，是的话需要反转速度并恢复到正确的位置
            if position.x <= 0.0 {
                // 超出左边
                velocity.x = -velocity.x;
                position.x = 0.0;
            } else if (position.x + size.x) >= window_width as f32 {
                // 超出右边
                velocity.x = -velocity.x;
                position.x = window_width as f32 - size.x;
            }

            if position.y <= 0.0 {
                // 超出上边
                velocity.y = -velocity.y;
                position.y = 0.0;
            }
        }
        return *position;
    }

    fn reset(&mut self, position: Point2<f32>, velocity: Vector2<f32>) {
        self.game_object.position = position;
        self.game_object.velocity = velocity;
        
    }

    fn draw<T: Surface>(&self, renderer: &SpriteRenderer, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        let game_object = &self.game_object;
        let texture = resource_manager.get_texture(&game_object.texture_key);
        renderer.draw_sprite(surface, texture, game_object.position, game_object.size, game_object.rotation, game_object.color, projection);
    }
}


struct Particle {
    position: Point2<f32>,
    velocity: Vector2<f32>,
    color: Vector4<f32>,
    life: f32,
}

impl Particle {
    pub fn new(position: Point2<f32>, velocity: Vector2<f32>, color: Vector4<f32>, life: f32) -> Self {
        Particle { position, velocity, color, life }
    }
}


struct ParticleGenerator<'a> {
    shader: Program,
    particles: Vec<Particle>,
    amount: u32,
    texture_key: String,
    vertex_buffer: VertexBuffer<RawVertexP2T>,
    index_buffer: IndexBuffer<u16>,
    draw_parameters: DrawParameters<'a>,
    deactive: VecDeque<usize>,
    rng: StdRng,
}

impl <'a> ParticleGenerator<'a> {

    pub fn new(display: &Display, shader: Program, texture_key: String, amount: u32) -> Self {
        let (particles, deactive) = {
            let mut vec = Vec::with_capacity(amount as usize);
            let mut deactive = VecDeque::with_capacity(amount as usize);
            for i in 0..amount {
                vec.push(Particle::new(Point2::new(0.0, 0.0), Vector2::new(0.0, 0.0), Vector4::new(1.0, 1.0, 1.0, 1.0), 0.0));
                deactive.push_back(i as usize);
            }
            (vec, deactive)
        };

        ParticleGenerator {
            shader,
            particles,
            amount,
            texture_key,
            vertex_buffer: VertexBuffer::new(display, &[
                RawVertexP2T { position: [0.0, 1.0], texture: [0.0, 1.0] },
                RawVertexP2T { position: [1.0, 0.0], texture: [1.0, 0.0] },
                RawVertexP2T { position: [0.0, 0.0], texture: [0.0, 0.0] },

                RawVertexP2T { position: [0.0, 1.0], texture: [0.0, 1.0] },
                RawVertexP2T { position: [1.0, 1.0], texture: [1.0, 1.0] },
                RawVertexP2T { position: [1.0, 0.0], texture: [1.0, 0.0] },
            ]).unwrap(),
            index_buffer: IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 3, 4, 5]).unwrap(),
            draw_parameters: DrawParameters {
                blend: Blend {
                    color: BlendingFunction::Addition {
                        source: LinearBlendingFactor::SourceAlpha,
                        destination: LinearBlendingFactor::One,
                    },
                    alpha: BlendingFunction::Addition {
                        source: LinearBlendingFactor::SourceAlpha,
                        destination: LinearBlendingFactor::One
                    },
                    constant_value: (0.0, 0.0, 0.0, 0.0)
                },
                backface_culling: BackfaceCullingMode::CullClockwise,
                .. Default::default()
            },
            deactive,
            rng: StdRng::seed_from_u64(0),
        }
    }

    fn update(&mut self, object: &GameObject, new_particles: u32, offset: Vector2<f32>, dt: Duration) {
        let dt = dt.as_secs_f32();
        
        for _ in 0..new_particles {
            // 产生新的粒子
            if let Some(index) = self.deactive.pop_front() {
                let particle = self.particles.get_mut(index).unwrap();
                let rng = &mut self.rng;
                respawn_particle(rng, particle, object, offset);
            }
        }

        // 更新所有粒子
        for (i, particle) in self.particles.iter_mut().enumerate() {
            if particle.life > 0.0 {
                particle.life -= dt;
                particle.position -= particle.velocity * dt;
                particle.color.w -= dt * 2.5;

                if particle.life <= 0.0 {
                    self.deactive.push_back(i);
                }
            }
        }
    }

    fn draw<T: Surface>(&self, surface: &mut T, resource_manager: &ResourceManager, projection: Matrix4<f32>) {
        let texture = resource_manager.get_texture(&self.texture_key);

        let mut uniforms = DynamicUniforms::new();

        uniforms.add_str_key_value("projection", UniformValue::Mat4(projection.into()));
        uniforms.add_str_key("sprite", texture);

        for particle in self.particles.iter() {
            if particle.life > 0.0 {
                uniforms.add_str_key_value("offset", UniformValue::Vec2(particle.position.into()));
                uniforms.add_str_key_value("color", UniformValue::Vec4(particle.color.into()));
                surface.draw(&self.vertex_buffer, &self.index_buffer, &self.shader, &uniforms, &self.draw_parameters).unwrap();
            }
        }
    }
}

fn respawn_particle(rng: &mut StdRng, particle: &mut Particle, object: &GameObject, offset: Vector2<f32>) {
    let random = (rng.gen_range(0..100) - 50) as f32 / 10.0;
    let color = 0.5 + (rng.gen_range(0..100) as f32 / 100.0);
    particle.position = object.position + offset + Vector2::new(random, random);
    particle.color = Vector4::new(color, color, color, 1.0);
    particle.life = 1.0;
    particle.velocity = object.velocity * 0.1;
}