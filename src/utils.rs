use glium::{Display, vertex::VertexBufferAny};

use crate::Vertex;


pub fn matrix4_to_raw<S>(matrix: cgmath::Matrix4<S>) -> [[S; 4]; 4] {
    [
        [matrix.x.x, matrix.y.x, matrix.z.x, matrix.w.x],
        [matrix.x.y, matrix.y.y, matrix.z.y, matrix.w.y],
        [matrix.x.z, matrix.y.z, matrix.z.z, matrix.w.z],
        [matrix.x.w, matrix.y.w, matrix.z.w, matrix.w.w],
    ]
}

pub fn load_wavefront(display: &Display, data: &[u8]) -> VertexBufferAny {

    let mut data = ::std::io::BufReader::new(data);
    let data = obj::ObjData::load_buf(&mut data).unwrap();

    let mut vertex_data = Vec::new();

    for object in data.objects.iter() {
        for polygon in object.groups.iter().flat_map(|g| g.polys.iter()) {
            match polygon {
                obj::SimplePolygon(indices) => {
                    for v in indices.iter() {
                        let position = data.position[v.0];
                        let texture = v.1.map(|index| data.texture[index]);
                        let normal = v.2.map(|index| data.normal[index]);

                        let texture = texture.unwrap_or([0.0, 0.0]);
                        let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                        vertex_data.push(Vertex {
                            position,
                            normal,
                            texture,
                        })
                    }
                },
            }
        }
    }

    glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap().into()
}
