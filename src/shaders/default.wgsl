const OPENGL_TO_WGPU_MATRIX: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.5, 0.0),
    vec4<f32>(0.0, 0.0, 0.5, 1.0),
);

struct CameraUniform {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct TransformInput {
    @location(5) matrix_0: vec4<f32>,
    @location(6) matrix_1: vec4<f32>,
    @location(7) matrix_2: vec4<f32>,
    @location(8) matrix_3: vec4<f32>,
};


@vertex
fn v_main(model: VertexInput, transform: TransformInput) -> VertexOutput {
    let transform_matrix = mat4x4<f32>(
        transform.matrix_0,
        transform.matrix_1,
        transform.matrix_2,
        transform.matrix_3,
    );

    var out: VertexOutput;

    out.clip_position = OPENGL_TO_WGPU_MATRIX * camera.projection * camera.view * transform_matrix * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    return out;
}


@fragment
fn f_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
