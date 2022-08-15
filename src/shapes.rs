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

pub fn create_quad_marching_squares(
    width: u16,
    height: u16,
) -> (Vec<crate::vertex_type::DefaultVertex>, Vec<u16>) {
    let mut vertices = vec![];
    let mut indices: Vec<u16> = vec![];

    let x: u16 = width * 2 + 2;
    let y: u16 = height * 2 + 2;

    let mut vert_values = vec![];

    // use rand::{thread_rng, Rng};
    // let mut rng = thread_rng();

    // for _ in 0..(x * y) {
    //     vert_values.push(rng.gen_range(0.0..=1.0));
    // }

    use noise::Seedable;
    let noise_generator = noise::Fbm::new();
    let mut noise_generator = noise_generator.set_seed(19);

    noise_generator.lacunarity = 0.1;
    noise_generator.persistence = 0.1;

    use noise::NoiseFn;

    for i in 0..x {
        for j in 0..y {
            let val = noise_generator.get([i as f64, j as f64]);
            vert_values.push(val);
        }
    }

    const THRESHOLD: f64 = 0.00;

    // TODO: linear interpolation

    for i in 1..x - 1 {
        for j in 1..y - 1 {
            //  a - b
            //  |   |
            //  d - c

            let position_a = i * x + j;
            let a = vert_values[position_a as usize];

            let position_b = i * x + j + 1;
            let b = vert_values[position_b as usize];

            let position_c = (i + 1) * x + j + 1;
            let c = vert_values[position_c as usize];

            let position_d = (i + 1) * x + j;
            let d = vert_values[position_d as usize];

            let pos = nalgebra_glm::vec2(
                i as f32 - width as f32 * 0.5,
                j as f32 - height as f32 * 0.5,
            );

            match (a < THRESHOLD, b < THRESHOLD, c < THRESHOLD, d < THRESHOLD) {
                (false, false, false, false) => {}
                (false, false, false, true) => {
                    let color = [0.0, 1.0, 1.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (false, false, true, false) => {
                    let color = [1.0, 1.0, 0.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (false, false, true, true) => {
                    let color = [0.0, 0.0, 0.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (false, true, false, false) => {
                    let color = [1.0, 1.0, 1.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (false, true, false, true) => {
                    let color = [1.0, 0.0, 1.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });

                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 6) as u16);
                    indices.push((vertices.len() - 5) as u16);
                    indices.push((vertices.len() - 1) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 5) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 5) as u16);
                    indices.push((vertices.len() - 4) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 3) as u16);
                }
                (false, true, true, false) => {
                    let color = [0.2, 0.4, 0.7];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (false, true, true, true) => {
                    let color = [1.0, 1.0, 1.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 5) as u16);
                    indices.push((vertices.len() - 4) as u16);

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 3) as u16);

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);
                }
                (true, false, false, false) => {
                    let color = [1.0, 1.0, 1.0];

                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                }
                (true, false, false, true) => {
                    let color = [0.7, 0.7, 0.7];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 1) as u16);

                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (true, false, true, false) => {
                    let color = [0.0, 1.0, 0.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 5) as u16);
                    indices.push((vertices.len() - 6) as u16);

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 5) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 1) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 4) as u16);
                }
                (true, false, true, true) => {
                    let color = [1.0, 1.0, 1.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 5) as u16);
                    indices.push((vertices.len() - 1) as u16);

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 1) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (true, true, false, false) => {
                    let color = [0.7, 0.2, 0.5];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 4) as u16);

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                }
                (true, true, false, true) => {
                    let color = [1.0, 0.0, 1.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 5) as u16);
                    indices.push((vertices.len() - 1) as u16);

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 4) as u16);
                    indices.push((vertices.len() - 1) as u16);

                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (true, true, true, false) => {
                    let color = [1.0, 1.0, 0.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.5 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 0.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [0.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 0.5 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 5) as u16);
                    indices.push((vertices.len() - 4) as u16);

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 5) as u16);

                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 1) as u16);
                }
                (true, true, true, true) => {
                    let color = [0.0, 1.0, 0.0];
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [1.0 + pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });
                    vertices.push(crate::vertex_type::DefaultVertex {
                        position: [pos.x, 0.0, 1.0 + pos.y],
                        color,
                    });

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 3) as u16);
                    indices.push((vertices.len() - 4) as u16);

                    indices.push((vertices.len() - 1) as u16);
                    indices.push((vertices.len() - 2) as u16);
                    indices.push((vertices.len() - 3) as u16);
                }
            }
        }
    }

    (vertices, indices)
}
