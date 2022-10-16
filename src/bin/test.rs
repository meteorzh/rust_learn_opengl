use cgmath::{prelude::*};

fn main() {
    let deg = cgmath::Deg(270_f32);
    println!("({}, 0, {})", deg.cos(), deg.sin())
}