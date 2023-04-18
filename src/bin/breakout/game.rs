use std::{time::Duration, collections::HashMap, thread, sync::mpsc::{self, Sender}, io::{BufReader, Cursor}};

use cgmath::{Vector2, Matrix4, Point2, Vector3, Deg, InnerSpace, EuclideanSpace};
use glium::{Display, Surface};
use rand::{rngs::StdRng, SeedableRng, Rng};
use rodio::{OutputStream, Sink, Decoder};
use rust_opengl_learn::{create_program, utils::clamp_vec2};

use crate::{game_object::GameObject, sprite_renderer::SpriteRenderer, resource_manager::{ResourceManager, load_audio}, game_level::GameLevel, ball_object::BallObject, particle_generator::ParticleGenerator, post_processor::PostProcessor, PlayerController, power_up::{PowerUp, PowerUpType}};

/// 游戏状态
#[derive(PartialEq, Eq)]
enum GameState {
    GAME_ACTIVE,
    GAME_MENU,
    GAME_WIN
}

static INITIAL_BALL_VELOCITY: Vector2<f32> = Vector2::new(100.0, -350.0);
/// 游戏类
pub struct Game<'a> {
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
    effects: Option<PostProcessor>,
    power_ups: Vec<PowerUp>,
    rng: StdRng,
    effect_sender: Option<Sender<Cursor<Vec<u8>>>>,
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
            particle_generator: None,
            effects: None,
            power_ups: vec![],
            rng: StdRng::seed_from_u64(0),
            effect_sender: None,
        }
    }

    pub fn init(&mut self, display: &Display) {
        let sprite_program = create_program("src/bin/breakout/test.vert", "src/bin/breakout/test.frag", display);
        let particle_program = create_program("src/bin/breakout/particle.vert", "src/bin/breakout/particle.frag", display);
        let effects_program = create_program("src/bin/breakout/post_processing.vert", "src/bin/breakout/post_processing.frag", display);

        // 加载纹理
        self.resource_manager.load_texture("background", "src/textures/background.jpg", display);
        self.resource_manager.load_texture("face", "src/awesomeface.png", display);
        self.resource_manager.load_texture("block", "src/textures/block.png", display);
        self.resource_manager.load_texture("block_solid", "src/textures/block_solid.png", display);
        self.resource_manager.load_texture("paddle", "src/textures/paddle.png", display);
        self.resource_manager.load_texture("particle", "src/textures/particle.png", display);
        self.resource_manager.load_texture("speed", "src/textures/powerup_speed.png", display);
        self.resource_manager.load_texture("sticky", "src/textures/powerup_sticky.png", display);
        self.resource_manager.load_texture("passthrough", "src/textures/powerup_passthrough.png", display);
        self.resource_manager.load_texture("increase", "src/textures/powerup_increase.png", display);
        self.resource_manager.load_texture("confuse", "src/textures/powerup_confuse.png", display);
        self.resource_manager.load_texture("chaos", "src/textures/powerup_chaos.png", display);

        self.resource_manager.load_audio("audio_bleep_brick", "src/audio/bleep.mp3");
        self.resource_manager.load_audio("audio_bleep_player", "src/audio/bleep.wav");
        self.resource_manager.load_audio("audio_power_up", "src/audio/powerup.wav");
        self.resource_manager.load_audio("audio_solid", "src/audio/solid.wav");
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

        // 后期处理
        self.effects = Some(PostProcessor::new(display, effects_program, self.width, self.height));

        // 游戏音频，开启新线程播放
        thread::spawn(|| {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let main_sink = Sink::try_new(&stream_handle).unwrap();
            let main_source = load_audio("src/audio/breakout.mp3");
            main_sink.append(main_source);
            main_sink.sleep_until_end();
        });
        let (effect_sender, effect_receiver) = mpsc::channel();
        self.effect_sender = Some(effect_sender);
        // 音效音频
        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            loop {
                let source = effect_receiver.recv().unwrap();
                sink.append(Decoder::new(BufReader::new(source)).unwrap());
                sink.sleep_until_end();
            }
        });
    }

    pub fn update_player(&mut self, player_controller: &mut PlayerController, dt: Duration) {
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

    pub fn update(&mut self, dt: Duration) {
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

        // 更新道具
        self.update_power_ups(dt);

        // 更新后期处理对象
        if let Some(effects) = &mut self.effects {
            effects.update(dt);
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

    fn should_spawn(&mut self, change: f64) -> bool {
        self.rng.gen_bool(change)
    }

    fn spawn_power_ups(&mut self, block_position: Point2<f32>) {
        // 1/75的概率
        if self.should_spawn(1.0 / 75.0) {
            self.power_ups.push(PowerUp::new(PowerUpType::Speed, Vector3::new(0.5, 0.5, 1.0), 0.0, block_position, "speed"));
        }
        if self.should_spawn(1.0 / 75.0) {
            self.power_ups.push(PowerUp::new(PowerUpType::Sticky, Vector3::new(1.0, 0.5, 1.0), 20.0, block_position, "sticky"));
        }
        if self.should_spawn(1.0 / 75.0) {
            self.power_ups.push(PowerUp::new(PowerUpType::PassThrough, Vector3::new(0.5, 1.0, 0.5), 10.0, block_position, "passthrough"));
        }
        if self.should_spawn(1.0 / 75.0) {
            self.power_ups.push(PowerUp::new(PowerUpType::PadSizeIncrease, Vector3::new(1.0, 0.6, 0.4), 0.0, block_position, "increase"));
        }
        
        // debuff 1/15的概率
        if self.should_spawn(1.0 / 15.0) {
            self.power_ups.push(PowerUp::new(PowerUpType::Confuse, Vector3::new(1.0, 0.3, 0.3), 15.0, block_position, "confuse"));
        }   
        if self.should_spawn(1.0 / 15.0) {
            self.power_ups.push(PowerUp::new(PowerUpType::Chaos, Vector3::new(0.9, 0.25, 0.25), 15.0, block_position, "chaos"));
        }
    }

    fn update_power_ups(&mut self, dt: Duration) {
        let delta_time = dt.as_secs_f32();

        // 按道具类型统计个数
        let mut count_map = HashMap::new();
        for power_up in self.power_ups.iter() {
            if let Some(v) = count_map.get_mut(&power_up.type_) {
                *v += 1;
            } else {
                count_map.insert(power_up.type_, 1);
            }
        }

        for power_up in self.power_ups.iter_mut() {
            let object = &mut power_up.game_object;
            object.position += object.velocity * delta_time;
            if power_up.activated {
                power_up.duration -= delta_time;
                if power_up.duration <= 0.0 {
                    // 道具失效
                    power_up.activated = false;

                    match power_up.type_ {
                        PowerUpType::Speed => {
                            
                        },
                        PowerUpType::Sticky => {
                            if *count_map.get(&power_up.type_).unwrap() == 1 {
                                // 此类型仅这一个生效的道具则失效
                                self.ball.sticky = false;
                                self.player.color = Vector3::new(1.0, 1.0, 1.0);
                            }
                        },
                        PowerUpType::PassThrough => {
                            if *count_map.get(&power_up.type_).unwrap() == 1 {
                                self.ball.pass_through = false;
                                self.ball.game_object.color = Vector3::new(1.0, 1.0, 1.0);
                            }
                        },
                        PowerUpType::PadSizeIncrease => {
                            
                        },
                        PowerUpType::Confuse => {
                            if *count_map.get(&power_up.type_).unwrap() == 1 {
                                if let Some(effects) = &mut self.effects {
                                    effects.confuse = false;
                                }
                            }
                        },
                        PowerUpType::Chaos => {
                            if *count_map.get(&power_up.type_).unwrap() == 1 {
                                if let Some(effects) = &mut self.effects {
                                    effects.chaos = false;
                                }
                            }
                        },
                    }
                }
            }
        }

        self.power_ups.retain(|power_up| !(power_up.game_object.destroyed && !power_up.activated));
    }

    fn activate_power_up(player: &mut GameObject, ball: &mut BallObject, effects: &mut PostProcessor, power_up: &PowerUp) {
        match power_up.type_ {
            PowerUpType::Speed => {
                // 球加速
                ball.game_object.velocity *= 1.2;
            },
            PowerUpType::Sticky => {
                ball.sticky = true;
                player.color = Vector3::new(1.0, 0.5, 1.0);
            },
            PowerUpType::PassThrough => {
                ball.pass_through = true;
                ball.game_object.color = Vector3::new(1.0, 0.5, 0.5);
            },
            PowerUpType::PadSizeIncrease => {
                player.size.x += 50.0;
            },
            PowerUpType::Confuse => {
                if !effects.chaos {
                    // 仅chaos未生效时才能confuse
                    effects.confuse = true;
                }
            },
            PowerUpType::Chaos => {
                if !effects.confuse {
                    // 仅confuse未生效时才能chaos
                    effects.chaos = true;
                }
            },
        }
    }

    fn play_effect_sound(sender: &Option<Sender<Cursor<Vec<u8>>>>, resource_manager: &ResourceManager, sound_key: &str) {
        if let Some(effect_sender) = sender {
            let source = resource_manager.get_audio(sound_key);
            effect_sender.send(source).unwrap();
        }
    }

    fn do_collisions(&mut self) {
        // 检测当前关卡中所有砖块的碰撞
        let level = self.levels.get_mut(self.level as usize).unwrap();
        let mut power_up_positions = Vec::new();
        for brick in level.bricks.iter_mut() {
            if !brick.destroyed {
                // 碰撞检测
                if let Some((direction, vec)) = check_collisions_aabb_round(&self.ball, &brick) {
                    if !brick.is_solid {
                        // 摧毁砖块
                        brick.destroyed = true;
                        power_up_positions.push(brick.position);
                        
                        // 音效
                        Game::play_effect_sound(&self.effect_sender, &self.resource_manager, "audio_bleep_brick");
                    } else {
                        // 实心砖块，激活shake特效
                        if let Some(effects) = &mut self.effects {
                            effects.start_shake(0.05);
                        }
                        // 音效
                        Game::play_effect_sound(&self.effect_sender, &self.resource_manager, "audio_solid");
                    }
                    // 处理碰撞
                    // 不能穿过或者碰墙则反弹，能穿过并且不是墙则穿过
                    if !(self.ball.pass_through && !brick.is_solid) {
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
        }

        // 判断球和玩家挡板的碰撞
        if !self.ball.stuck {
            if let Some((_, _)) = check_collisions_aabb_round(&self.ball, &self.player) {
                // 判定道具效果，当前有sticky道具开启的话，球和玩家挡板碰撞时球需要被固定
                self.ball.stuck = self.ball.sticky;

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

                Game::play_effect_sound(&self.effect_sender, &self.resource_manager, "audio_bleep_player")
            }
        }

        // 再本次渲染销毁的砖块处，生成道具
        for position in power_up_positions {
            self.spawn_power_ups(position);
        }

        // 判断玩家挡板和道具的碰撞
        for power_up in self.power_ups.iter_mut() {
            let object = &mut power_up.game_object;
            if !object.destroyed {
                // 超出当前窗口高度，标记道具为销毁
                if object.position.y >= self.height as f32 {
                    object.destroyed = true;
                }

                // 判断碰撞
                if check_collisions_aabb_aabb(&self.player, &object) {
                    object.destroyed = true;
                    power_up.activated = true;
                    // 激活道具
                    if let Some(effects) = &mut self.effects {
                        Game::activate_power_up(&mut self.player, &mut self.ball, effects, &power_up);
                    }

                    Game::play_effect_sound(&self.effect_sender, &self.resource_manager, "audio_power_up");
                }
            }
        }
    }

    pub fn render<T: Surface>(&mut self, surface: &mut T, dt: Duration) {
        // 后期处理
        if let Some(effects) = &mut self.effects {
            effects.post_process(|framebuffer| {
                if let Some(renderer) = &self.sprite_renderer {
                    // 绘制背景
                    renderer.draw_sprite(
                        framebuffer, 
                        self.resource_manager.get_texture("background"), 
                        Point2::new(0.0, 0.0), 
                        Vector2::new(self.width as f32, self.height as f32), 
                        Deg(0.0), 
                        Vector3::new(1.0, 1.0, 1.0),
                        self.projection
                    );
                    // 绘制关卡
                    let level = self.levels.get(self.level as usize).unwrap();
                    level.draw(renderer, framebuffer, &self.resource_manager, self.projection);
                    // 绘制挡板
                    self.player.draw(renderer, framebuffer, &self.resource_manager, self.projection);
        
                    // 如果游戏激活，需要绘制球
                    if self.state == GameState::GAME_ACTIVE {
                        self.ball.draw(renderer, framebuffer, &self.resource_manager, self.projection);
                    }
                    // 渲染道具
                    for power_up in self.power_ups.iter() {
                        if !power_up.game_object.destroyed {
                            power_up.game_object.draw(renderer, framebuffer, &self.resource_manager, self.projection);
                        }
                    }
                }
                // 渲染粒子
                if let Some(particle_generator) = &self.particle_generator {
                    particle_generator.draw(framebuffer, &self.resource_manager, self.projection);
                }
            });
            
            // 将后期处理结果渲染到指定surface
            effects.render(surface, dt);
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
