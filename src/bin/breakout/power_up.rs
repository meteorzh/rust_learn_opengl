use cgmath::{Vector3, Vector2, Deg, Point2};

use crate::game_object::GameObject;



const POWERUP_SIZE: Vector2<f32> = Vector2::new(60.0, 20.0);

const VELOCITY: Vector2<f32> = Vector2::new(0.0, 150.0);


#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum PowerUpType {
    Speed,

    Sticky,

    PassThrough,

    PadSizeIncrease,

    Confuse,

    Chaos,
}

pub struct PowerUp {
    pub game_object: GameObject,
    pub type_: PowerUpType,
    pub duration: f32,
    pub activated: bool,
}

impl PowerUp {
    
    pub fn new(type_: PowerUpType, color: Vector3<f32>, duration: f32, position: Point2<f32>, texture_key: &str) -> Self {
        PowerUp {
            game_object: GameObject {
                position,
                size: POWERUP_SIZE,
                velocity: VELOCITY,
                color,
                rotation: Deg(0.0),
                is_solid: false,
                destroyed: false,
                texture_key: texture_key.to_string(),
            },
            type_,
            duration,
            activated: false
        }
    }
}