use std::{collections::{HashMap, hash_map::Entry}, rc::Rc, sync::Arc, fs::{self, File}, io::{Cursor, BufReader}, future};

use cgmath::{Vector3, Zero, Vector2};
use glium::{implement_vertex, vertex::VertexBufferAny, index::{IndexBufferAny, self}, Display, IndexBuffer, texture::CompressedSrgbTexture2d, program::{SourceCode, ProgramCreationInput}, Program};
use material::{Material, MaterialLoader};
use obj::{ObjData, ObjMaterial};

pub mod utils;
pub mod camera;
pub mod lights;
pub mod uniforms;
pub mod material;
pub mod objects;


#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture: [f32; 2],
}

implement_vertex!(Vertex, position, normal, texture);


pub struct Model {
    pub vertex_buffer: VertexBufferAny,
    pub index_buffer: IndexBufferAny,
    pub material: Option<Rc<Material>>,
}

/**
 * 将一个模型数据加载为多个绘制单位
 */
pub fn load_wavefront_obj_as_models(display: &Display, basepath: &str, obj_file: &str) -> Vec<Model> {
    let mut obj_path = String::from(basepath);
    obj_path.push_str(obj_file);
    let mut obj = obj::Obj::load(obj_path).unwrap();
    // 需要手动加载材质
    obj.load_mtls().unwrap();
    let data = obj.data;

    // 加载材质
    let mut material_loader = MaterialLoader::new();
    material_loader.parse_and_load(&data.material_libs, basepath, display);

    let mut models = Vec::new();
    for obj in data.objects.iter() {
        for group in obj.groups.iter() {
            // 目前认为相同材质的所有面（具有法向量和材质坐标的顶点）可以一起绘制，放入一个model
            let mut cache = HashMap::new();
            let mut vertex_data = Vec::new();
            let mut index_data = Vec::new();

            // 创建材质
            let material = match &group.material {
                Some(material) => {
                    let name = match material {
                        ObjMaterial::Ref(name) => name,
                        ObjMaterial::Mtl(meterial) => &meterial.name,
                    };
                    material_loader.find_in_cache(name.clone())
                },
                None => None,
            };
            for poly in group.polys.iter() {
                // 按面进行绘制
                // 创建顶点
                for index in poly.0.iter() {
                    let i = match cache.get(index) {
                        Some(vertex_index) => *vertex_index,
                        None => {
                            let vertex = Vertex {
                                position: data.position[index.0],
                                normal: match index.2 {
                                    Some(i) => data.normal[i],
                                    None => Into::<[f32; 3]>::into(Vector3::zero()),
                                },
                                texture: match index.1 {
                                    Some(i) => data.texture[i],
                                    None => Into::<[f32; 2]>::into(Vector2::zero()),
                                }
                            };
                            let vertex_index = vertex_data.len();
                            vertex_data.push(vertex);
                            cache.insert(index, vertex_index);
                            vertex_index
                        }
                    };
                    index_data.push(i as u16);
                }
            }
            let vertex_buffer = glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap().into();
            let index_buffer = IndexBuffer::new(display, index::PrimitiveType::TrianglesList, &index_data).unwrap().into();
            models.push(Model {
                vertex_buffer: vertex_buffer,
                index_buffer: index_buffer,
                material: material,
            });
        }
    }
    models
}




// struct MaterialLoader {
//     cache: HashMap<String, Rc<Material>>,
//     map_cache: HashMap<String, Rc<CompressedSrgbTexture2d>>,
// }

// impl MaterialLoader {
    
//     fn new() -> MaterialLoader {
//         MaterialLoader { cache: HashMap::new(), map_cache: HashMap::new() }
//     }

//     fn load(&mut self, obj_material: &Arc<obj::Material>, basepath: &str, display: &Display) {
//         let name = obj_material.name.clone();
//         if self.cache.contains_key(&name) {
//             return;
//         }

//         let material = Material {
//             ambient: obj_material.ka,
//             diffuse: obj_material.kd,
//             specular: obj_material.ks,
//             emissive: obj_material.ke,
//             transmission_filter: obj_material.tf,
//             shininess: obj_material.ns,
//             illumination_model: obj_material.illum,
//             dissolve: obj_material.d,
//             specular_exponent: None,
//             optical_density: obj_material.ni,
//             ambient_map: self.parse_2d_texture(&obj_material.map_ka, basepath, display),
//             diffuse_map: self.parse_2d_texture(&obj_material.map_kd, basepath, display),
//             specular_map: self.parse_2d_texture(&obj_material.map_ks, basepath, display),
//             emissive_map: self.parse_2d_texture(&obj_material.map_ke, basepath, display),
//             dissolve_map: self.parse_2d_texture(&obj_material.map_d, basepath, display),
//             bump_map: self.parse_2d_texture(&obj_material.map_bump, basepath, display),
//         };
//         self.cache.insert(name, Rc::new(material));
//     }

//     fn parse_2d_texture(&mut self, file: &Option<String>, basepath: &str, display: &Display) -> Option<Rc<CompressedSrgbTexture2d>> {
//         match file {
//             Some(file) => {
//                 let mut path = String::from(basepath);
//                 path.push_str(file.as_str());
//                 // 现在缓存中找
//                 match self.map_cache.get(&path) {
//                     Some(texture_2d) => return Some(Rc::clone(texture_2d)),
//                     None => (),
//                 };

//                 let image = image::load(Cursor::new(fs::read(path.as_str()).unwrap()), image::ImageFormat::Png).unwrap().to_rgba8();
//                 let image_dimensions = image.dimensions();
//                 let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
//                 let opengl_texture = glium::texture::CompressedSrgbTexture2d::new(display, image).unwrap();

//                 let texture_map = Rc::new(opengl_texture);
//                 let result = Rc::clone(&texture_map);
//                 self.map_cache.insert(path, texture_map);
                
//                 Some(result)
//             },
//             None => None,
//         }
//     }

//     fn find_in_cache(&self, name: String) -> Option<Rc<Material>> {
//         match self.cache.get(&name) {
//             Some(material) => {
//                 Some(Rc::clone(material))
//             },
//             None => None,
//         }
//     }
// }

pub fn create_program(vert_source_path: &str, frag_source_path: &str, display: &Display) -> Program {
    let obj_vert_source = fs::read_to_string(vert_source_path).unwrap();
    let obj_frag_source = fs::read_to_string(frag_source_path).unwrap();
    let obj_shader_source = SourceCode {
        vertex_shader: obj_vert_source.as_str(),
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: obj_frag_source.as_str(),
    };

    glium::Program::new(
        display,
        ProgramCreationInput::from(obj_shader_source)
    ).unwrap()
}