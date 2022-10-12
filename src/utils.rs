
pub fn matrix4_to_raw<S>(matrix: cgmath::Matrix4<S>) -> [[S; 4]; 4] {
    [
        [matrix.x.x, matrix.y.x, matrix.z.x, matrix.w.x],
        [matrix.x.y, matrix.y.y, matrix.z.y, matrix.w.y],
        [matrix.x.z, matrix.y.z, matrix.z.z, matrix.w.z],
        [matrix.x.w, matrix.y.w, matrix.z.w, matrix.w.w],
    ]
}
