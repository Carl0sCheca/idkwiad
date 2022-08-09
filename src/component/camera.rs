#[derive(Debug)]
pub struct Camera {
    pub camera_type: CameraType,
    pub projection: nalgebra_glm::Mat4,
    pub up_dir: nalgebra_glm::Vec3,
    pub buffer: wgpu::Buffer,
    pub uniform: CameraUniform,
    pub device: std::sync::Arc<wgpu::Device>,
    pub window: std::sync::Arc<winit::window::Window>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CameraType {
    Orthographic,
    OrthographicCustom {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    },
    Perspective,
    PerspectiveCustom {
        aspect: f32,
        fovy: f32,
        near: f32,
        far: f32,
    },
}

impl Camera {
    pub fn new(
        camera_type: CameraType,
        window: std::sync::Arc<winit::window::Window>,
        device: std::sync::Arc<wgpu::Device>,
    ) -> (Self, wgpu::BindGroup, wgpu::BindGroupLayout) {
        let uniform = CameraUniform::new();

        let buffer = wgpu::util::DeviceExt::create_buffer_init(
            device.as_ref(),
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let mut camera = Self {
            camera_type,
            projection: nalgebra_glm::Mat4::identity(),
            buffer,
            uniform,
            device,
            window,
            up_dir: nalgebra_glm::Vec3::y(),
        };

        camera.projection = camera.build_projection();

        (camera, camera_bind_group, camera_bind_group_layout)
    }

    pub fn update(&mut self, transform: &super::Transform, queue: &wgpu::Queue) {
        self.up_dir = transform.up();

        self.uniform
            .update(self.build_projection(), self.build_view(transform));

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]))
    }

    pub fn build_projection(&self) -> nalgebra_glm::Mat4 {
        match self.camera_type {
            CameraType::Orthographic => {
                nalgebra_glm::ortho_rh(0.0, 1280.0, 0.0, 720.0, 0.025, 1000.0)
            }
            CameraType::Perspective => {
                nalgebra_glm::perspective_rh(1280.0 / 720.0, 45.0, 0.1, 1000.0)
            }
            CameraType::OrthographicCustom {
                left,
                right,
                bottom,
                top,
                znear,
                zfar,
            } => nalgebra_glm::ortho_rh(left, right, bottom, top, znear, zfar),
            CameraType::PerspectiveCustom {
                aspect,
                fovy,
                near,
                far,
            } => nalgebra_glm::perspective_rh(aspect, fovy, near, far),
        }
    }

    pub fn build_view(&self, transform: &super::Transform) -> nalgebra_glm::Mat4 {
        nalgebra_glm::Mat4::look_at_rh(
            &transform.position.into(),
            &(transform.position + transform.forward()).into(),
            &self.up_dir,
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            projection: nalgebra_glm::Mat4::identity().into(),
            view: nalgebra_glm::Mat4::identity().into(),
        }
    }

    pub fn update(&mut self, projection: nalgebra_glm::Mat4, view: nalgebra_glm::Mat4) {
        self.projection = projection.into();
        self.view = view.into();
    }
}
