use std::{rc::Rc, collections::HashMap, sync::Arc, io::Cursor, fs};

use futures::{executor::{block_on, ThreadPool, ThreadPoolBuilder}};
use glium::{texture::{CompressedSrgbTexture2d, SrgbCubemap, CubeLayer, RawImage2d}, Display, framebuffer::{SimpleFrameBuffer}, Texture2d, Surface, BlitTarget, uniforms::MagnifySamplerFilter};
use obj::Mtl;

use crate::{uniforms::{DynamicUniforms, add_to_uniforms}, utils};

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
    /// The specular highlight component
    pub specular_hightlight_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// The dissolve map, specified by `map_d`
    pub dissolve_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// The bump map (normal map), specified by `bump`
    pub bump_map: Option<Rc<CompressedSrgbTexture2d>>,
    /// spherical reflection map
    pub reflect_map: Option<Rc<CompressedSrgbTexture2d>>,
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

        if let Some(reflection) = &self.reflect_map {
            // reflection.sampled()
            add_to_uniforms(key, ".reflection", reflection.as_ref(), uniforms);
        }
    }
}



pub struct MaterialLoader {
    cache: HashMap<String, Rc<Material>>,
    map_cache: HashMap<String, Rc<CompressedSrgbTexture2d>>,
    pool: ThreadPool,
}

impl MaterialLoader {
    
    pub fn new() -> MaterialLoader {
        MaterialLoader { cache: HashMap::new(), map_cache: HashMap::new(), pool: ThreadPoolBuilder::new().name_prefix("material-loader").create().unwrap() }
    }

    pub fn parse_and_load(&mut self, mtls: &Vec<Mtl>, basepath: &str, display: &Display) {
        let mut texture_paths = HashMap::new();
        for mtl in mtls.iter() {
            println!("材质文件{}中有{}个材质需要加载...", mtl.filename, mtl.materials.len());
            // let mut futures = Vec::with_capacity(mtl.materials.len());
            for material in mtl.materials.iter() {
                let paths = Self::parse_valid_texture_paths(basepath, material);
                for path in paths.into_iter() {
                    texture_paths.insert(path, 0);
                }
            }
        }
        // 加载图片, 异步加载
        let fts = {
            let mut futures = Vec::with_capacity(texture_paths.len());
            for path in texture_paths.keys().into_iter() {
                futures.push(load_texture_async(path.clone(), display));
            }
            futures
        };
        println!("开始加载材质图片");
        let textures = block_on(futures::future::join_all(fts));
        println!("材质图片加载完成");
        for texture in textures.into_iter() {
            self.map_cache.insert(texture.0, Rc::new(texture.1));
        }

        for mtl in mtls.iter() {
            // let mut futures = Vec::with_capacity(mtl.materials.len());
            for material in mtl.materials.iter() {
                self.load(material, basepath);
            }
        }
    }

    fn load(&mut self, obj_material: &Arc<obj::Material>, basepath: &str) {
        let name = obj_material.name.clone();
        if self.cache.contains_key(&name) {
            return;
        }

        let material = Material {
            ambient: obj_material.ka,
            diffuse: obj_material.kd,
            specular: obj_material.ks,
            emissive: obj_material.ke,
            transmission_filter: obj_material.tf,
            shininess: obj_material.ns,
            illumination_model: obj_material.illum,
            dissolve: obj_material.d,
            specular_exponent: None,
            optical_density: obj_material.ni,
            ambient_map: self.find_2d_texture(&obj_material.map_ka, basepath),
            diffuse_map: self.find_2d_texture(&obj_material.map_kd, basepath),
            specular_map: self.find_2d_texture(&obj_material.map_ks, basepath),
            emissive_map: self.find_2d_texture(&obj_material.map_ke, basepath),
            specular_hightlight_map: self.find_2d_texture(&obj_material.map_ns, basepath),
            dissolve_map: self.find_2d_texture(&obj_material.map_d, basepath),
            bump_map: self.find_2d_texture(&obj_material.map_bump, basepath),
            reflect_map: self.find_2d_texture(&obj_material.map_refl, basepath),
        };
        self.cache.insert(name, Rc::new(material));
    }

    fn find_2d_texture(&self, file: &Option<String>, basepath: &str) -> Option<Rc<CompressedSrgbTexture2d>> {
        if let Some(file) = file {
            let mut path = basepath.to_string();
            path.push_str(file.as_str());
            
            if let Some(texture) = self.map_cache.get(&path) {
                return Some(Rc::clone(texture));
            }
        }
        None
    }

    pub fn find_in_cache(&self, name: String) -> Option<Rc<Material>> {
        match self.cache.get(&name) {
            Some(material) => {
                Some(Rc::clone(material))
            },
            None => None,
        }
    }

    fn parse_valid_texture_paths(basepath: &str, material: &Arc<obj::Material>) -> Vec<String> {
        let mut result = Vec::new();
        Self::add_to_vec(&mut result, &material.map_ka, basepath);
        Self::add_to_vec(&mut result, &material.map_kd, basepath);
        Self::add_to_vec(&mut result, &material.map_ks, basepath);
        Self::add_to_vec(&mut result, &material.map_ke, basepath);
        Self::add_to_vec(&mut result, &material.map_d, basepath);
        Self::add_to_vec(&mut result, &material.map_bump, basepath);

        result
    }

    fn add_to_vec(list: &mut Vec<String>, value: &Option<String>, basepath: &str) {
        if let Some(file) = value {
            let mut str = basepath.to_string();
            str.push_str(file.as_str());
            list.push(str);
        }
    }
    
}

pub async fn load_texture_async(path: String, display: &Display) -> (String, CompressedSrgbTexture2d) {
    load_texture(path, display)
}

pub fn load_texture(path: String, display: &Display) -> (String, CompressedSrgbTexture2d) {
    let temp = path.clone();
    let image = load_image(&temp);
    (path, glium::texture::CompressedSrgbTexture2d::new(display, image).unwrap())
}

pub fn load_image(path: &str) -> RawImage2d<u8> {
    println!("加载材质图片: {}", path);
    let format = {
        if path.ends_with(".png") {
            image::ImageFormat::Png
        } else if path.ends_with(".jpg") {
            image::ImageFormat::Jpeg
        } else {
            panic!("不支持的图片格式");
        }
    };
    let image = image::load(Cursor::new(fs::read(path).unwrap()), format).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions)
}

static CUBEMAP_FILES: [(&str, CubeLayer); 6] = [
    ("right", CubeLayer::PositiveX), 
    ("left", CubeLayer::NegativeX), 
    ("top", CubeLayer::PositiveY), 
    ("bottom", CubeLayer::NegativeY), 
    ("front", CubeLayer::PositiveZ), 
    ("back", CubeLayer::NegativeZ)
];

// 加载立方体贴图
pub fn load_cubemap(dir: &str, suffix: &str, display: &Display, dimensions: i32) -> SrgbCubemap {
    let cube_texture = SrgbCubemap::empty(display, dimensions as u32).unwrap();

    for item in CUBEMAP_FILES.iter() {
        let file_name = utils::build_filename(item.0, suffix);
        let path = utils::concat_filepath(dir, &file_name);
        let image = load_image(&path);

        let texture = Texture2d::new(display, image).unwrap();
        let rect = BlitTarget {
            left: 0,
            bottom: 0,
            width: dimensions,
            height: dimensions,
        };

        let framebuffer = SimpleFrameBuffer::new(display, cube_texture.main_level().image(item.1)).unwrap();
        texture.as_surface().blit_whole_color_to(&framebuffer, &rect, MagnifySamplerFilter::Linear);
    }

    cube_texture
}