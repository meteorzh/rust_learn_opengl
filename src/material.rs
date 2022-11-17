use std::rc::Rc;

use glium::texture::CompressedSrgbTexture2d;

use crate::uniforms::{DynamicUniforms, add_to_uniforms};

/**
 * 材质
 */
#[derive(Default)]
pub struct Material {
    /// The ambient color, specified by `Ka`
    pub ambient: Option<[f32; 3]>,
    /// The diffuse color, specified by `Kd`
    pub diffuse: Option<[f32; 3]>,
    /// The specular color, specified by `Ks`
    pub specular: Option<[f32; 3]>,
    /// The emissive color, specified by `Ke`
    pub emissive: Option<[f32; 3]>,
    /// The transmission filter, specified by `Tf`
    pub transmission_filter: Option<[f32; 3]>,
    /// shininess, specified by `Ns`
    pub shininess: Option<f32>,
    /// The illumination model to use for this material; see the `.mtl` spec for more details. specified by `illum`
    pub illumination_model: Option<i32>,
    /// The dissolve (opacity) of the material, specified by `d`
    pub dissolve: Option<f32>,
    /// The specular exponent, specified by `Ne`
    pub specular_exponent: Option<f32>,
    /// The optical density, i.e. index of refraction, specified by `Ni`
    pub optical_density: Option<f32>,
    /// The ambient color map, specified by `map_Ka`
    pub ambient_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// The diffuse color map, specified by `map_Kd`
    pub diffuse_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// The specular color map, specified by `map_Ks`
    pub specular_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// The emissive color map, specified by `map_Ke`
    pub emissive_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// The dissolve map, specified by `map_d`
    pub dissolve_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// The bump map (normal map), specified by `bump`
    pub bump_map: Option<Rc<CompressedSrgbTexture2d>>,
}

impl Material {
    // pub fn new(diffuse: CompressedSrgbTexture2d, specular: CompressedSrgbTexture2d, shininess: f32) -> Material {
    //     Material {
    //         diffuse: diffuse,
    //         specular: specular,
    //         shininess: shininess
    //     }
    // }

    pub fn add_to_uniforms<'a: 'b, 'b>(&'a self, key: &str, uniforms: &'b mut DynamicUniforms<'a>) {
        if let Some(diffuse) = &self.diffuse_map {
            add_to_uniforms(key, ".diffuse", diffuse.as_ref(), uniforms);
        }
        
        if let Some(specular) = &self.specular_map {
            add_to_uniforms(key, ".specular", specular.as_ref(), uniforms);
        }

        if let Some(shininess) = &self.shininess {
            add_to_uniforms(key, ".shininess", shininess, uniforms);
        }
    }
}