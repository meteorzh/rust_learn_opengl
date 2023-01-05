use crate::uniforms::{DynamicUniforms, add_to_uniforms};

/**
 * 定向光源
 */
pub struct DirLight {
    direction: [f32; 3],

    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}

impl DirLight {

    pub fn new(direction: [f32; 3], ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3],) -> DirLight {
        DirLight { direction: direction, ambient: ambient, diffuse: diffuse, specular: specular }
    }

    pub fn add_to_uniforms<'a: 'b, 'b>(&'a self, light_key: &str, uniforms: &'b mut DynamicUniforms<'a>) {
        add_to_uniforms(light_key, ".direction", &self.direction, uniforms);
        add_to_uniforms(light_key, ".ambient", &self.ambient, uniforms);
        add_to_uniforms(light_key, ".diffuse", &self.diffuse, uniforms);
        add_to_uniforms(light_key, ".specular", &self.specular, uniforms);
    }
}


/**
 * 点光源
 */
pub struct PointLight {
    position: [f32; 3],

    color: [f32; 3],

    constant: f32,
    linear: f32,
    quadratic: f32,

    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}

impl PointLight {
    pub fn new_simple(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position: position,
            color,
            constant: 0.0,
            linear: 0.0,
            quadratic: 0.0,
            ambient: [0.0, 0.0, 0.0],
            diffuse: [0.0, 0.0, 0.0],
            specular: [0.0, 0.0, 0.0],
        }
    }

    pub fn new(position: [f32; 3], color: [f32; 3], constant: f32, linear: f32, quadratic: f32, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) -> PointLight {
        PointLight {
            position: position,
            color,
            constant: constant,
            linear: linear,
            quadratic: quadratic,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
        }
    }

    pub fn add_to_uniforms<'a: 'b, 'b>(&'a self, light_key: &str, uniforms: &'b mut DynamicUniforms<'a>) {
        add_to_uniforms(light_key, ".position", &self.position, uniforms);
        add_to_uniforms(light_key, ".color", &self.color, uniforms);
        add_to_uniforms(light_key, ".constant", &self.constant, uniforms);
        add_to_uniforms(light_key, ".linear", &self.linear, uniforms);
        add_to_uniforms(light_key, ".quadratic", &self.quadratic, uniforms);
        add_to_uniforms(light_key, ".ambient", &self.ambient, uniforms);
        add_to_uniforms(light_key, ".diffuse", &self.diffuse, uniforms);
        add_to_uniforms(light_key, ".specular", &self.specular, uniforms);
    }
}

/**
 * 聚光灯
 */
pub struct SpotLight {
    position: [f32; 3],
    direction: [f32; 3],
    cut_off: f32,
    outer_cut_off: f32,
    constant: f32,
    linear: f32,
    quadratic: f32,
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
}

impl SpotLight {

    pub fn new(position: [f32; 3], direction: [f32; 3], cut_off: f32, outer_cut_off: f32, constant: f32, linear: f32, quadratic: f32, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) -> SpotLight {
        SpotLight {
            position: position,
            direction: direction,
            cut_off: cut_off,
            outer_cut_off: outer_cut_off,
            constant: constant,
            linear: linear,
            quadratic: quadratic,
            ambient: ambient,
            diffuse: diffuse,
            specular: specular,
        }
    }
    
    pub fn add_to_uniforms<'a: 'b, 'b>(&'a self, light_key: &str, uniforms: &'b mut DynamicUniforms<'a>) {
        add_to_uniforms(light_key, ".position", &self.position, uniforms);
        add_to_uniforms(light_key, ".direction", &self.direction, uniforms);
        add_to_uniforms(light_key, ".cutOff", &self.cut_off, uniforms);
        add_to_uniforms(light_key, ".outerCutOff", &self.outer_cut_off, uniforms);
        add_to_uniforms(light_key, ".constant", &self.constant, uniforms);
        add_to_uniforms(light_key, ".linear", &self.linear, uniforms);
        add_to_uniforms(light_key, ".quadratic", &self.quadratic, uniforms);
        add_to_uniforms(light_key, ".ambient", &self.ambient, uniforms);
        add_to_uniforms(light_key, ".diffuse", &self.diffuse, uniforms);
        add_to_uniforms(light_key, ".specular", &self.specular, uniforms);
    }
}