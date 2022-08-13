use crate::vertex_type::*;

pub fn create_grid(width: f32, depth: f32, m: u32, n: u32) -> (Vec<LineVertex>, Vec<u16>) {
    let vertex_count: u32 = m * n;
    let face_count: u32 = (m - 1) * (n - 1) * 2;

    let half_width = width * 0.5;
    let half_depth = depth * 0.5;

    let dx = width / (n as f32 - 1.0);
    let dz = depth / (m as f32 - 1.0);

    let mut vertices = vec![
        LineVertex {
            position: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0],
        };
        vertex_count as usize
    ];

    for i in 0..m {
        let z = half_depth - i as f32 * dz;
        for j in 0..n {
            let x = -half_width + j as f32 * dx;

            vertices[(i * n + j) as usize] = LineVertex {
                position: [x, 0.0, z],
                color: [1.0, 0.0, 0.0],
            };
        }
    }

    let mut indices = vec![0; (face_count * 4) as usize];
    for i in 0..(m - 1) {
        for j in 0..(n - 1) {
            indices.push((i * n + j) as u16);
            indices.push((i * n + (j + 1)) as u16);
            indices.push((i * n + (j + 1)) as u16);
            indices.push(((i + 1) * n + j + 1) as u16);

            indices.push(((i + 1) * n + j + 1) as u16);
            indices.push(((i + 1) * n + j) as u16);
            indices.push(((i + 1) * n + j) as u16);
            indices.push((i * n + j) as u16);
        }
    }

    (vertices, indices)
}
