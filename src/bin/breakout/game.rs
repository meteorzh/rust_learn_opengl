use std::time::Duration;

use cgmath::{Vector2, Matrix4, Point2, Vector3, Deg, InnerSpace, EuclideanSpace};
use glium::{Display, Surface};
use rust_opengl_learn::{create_program, utils::clamp_vec2};

use crate::{game_object::GameObject, sprite_renderer::SpriteRenderer, resource_manager::ResourceManager, game_level::GameLevel, ball_object::BallObject, particle_generator::ParticleGenerator, post_processor::PostProcessor, PlayerController};

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
                    } else {
                        // 实心砖块，激活shake特效
                        if let Some(effects) = &mut self.effects {
                            effects.start_shake(0.05);
                        }
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
