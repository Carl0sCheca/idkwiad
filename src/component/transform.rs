#![allow(dead_code)]

#[derive(Debug)]
pub struct Transform {
    position: nalgebra_glm::Vec3,
    rotation: nalgebra_glm::Vec3,
    size: nalgebra_glm::Vec3,
    q_rotation: nalgebra_glm::Quat,
    pub parent: Option<std::sync::Arc<std::sync::Mutex<Transform>>>,
    pub buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    matrix: nalgebra_glm::Mat4,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    pub transform: [[f32; 4]; 4],
}

impl Default for Transform {
    fn default() -> Self {
        let position = nalgebra_glm::zero();

        let rot_x = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(0.0),
            &nalgebra_glm::Vec3::x(),
        );
        let rot_y = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(0.0),
            &nalgebra_glm::Vec3::y(),
        );
        let rot_z = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(0.0),
            &nalgebra_glm::Vec3::z(),
        );

        let q_rotation = (rot_x * rot_y * rot_z).normalize();
        let rotation = nalgebra_glm::quat_euler_angles(&q_rotation);

        let mut matrix = nalgebra_glm::translate(&nalgebra_glm::Mat4::identity(), &position);
        matrix = matrix * nalgebra_glm::quat_to_mat4(&q_rotation);

        Self {
            position,
            rotation,
            size: nalgebra_glm::vec3(1.0, 1.0, 1.0),
            q_rotation,
            parent: None,
            buffer: None,
            matrix,
        }
    }
}

pub struct TransformBuild(Transform);

impl TransformBuild {
    pub fn new() -> Self {
        Self(Transform::default())
    }

    pub fn with_position(mut self, position: nalgebra_glm::Vec3) -> Self {
        self.0.position = position;
        self.0.matrix = self.0.matrix * nalgebra_glm::translate(&self.0.matrix, &position);

        self
    }

    pub fn with_rotation(mut self, rotation: nalgebra_glm::Vec3) -> Self {
        let rot_x = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(rotation.x),
            &nalgebra_glm::Vec3::x(),
        );
        let rot_y = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(rotation.y),
            &nalgebra_glm::Vec3::y(),
        );
        let rot_z = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(rotation.z),
            &nalgebra_glm::Vec3::z(),
        );

        let rotation = (rot_x * rot_y * rot_z).normalize();

        self.0.q_rotation = rotation;
        self.0.rotation = nalgebra_glm::quat_euler_angles(&rotation);
        self.0.matrix = self.0.matrix * nalgebra_glm::quat_to_mat4(&self.0.q_rotation);

        self
    }

    pub fn with_parent(mut self, parent: std::sync::Arc<std::sync::Mutex<Transform>>) -> Self {
        self.0.parent = Some(parent);
        self
    }

    pub fn with_buffer(mut self, device: &wgpu::Device) -> Self {
        self.0.buffer = Some(std::sync::Arc::new(
            wgpu::util::DeviceExt::create_buffer_init(
                device,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Transform Buffer"),
                    contents: bytemuck::cast_slice(&[self.0.to_raw()]),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                },
            ),
        ));

        self
    }

    pub fn build(self) -> Transform {
        self.0
    }
}

impl Transform {
    pub fn to_raw(&self) -> TransformRaw {
        let transform = self.matrix;
        let final_transform = self.get_parent_matrix() * transform;

        TransformRaw {
            transform: final_transform.into(),
        }
    }

    pub fn get_matrix(&self) -> nalgebra_glm::Mat4 {
        self.matrix
    }

    pub fn get_parent_matrix(&self) -> nalgebra_glm::Mat4 {
        let mut parent_matrix = nalgebra_glm::Mat4::identity();
        if let Some(parent) = self.parent.as_ref() {
            let _parent_matrix = parent.lock().unwrap().get_matrix();
            let _parent_parent_matrix = parent.lock().unwrap().get_parent_matrix();

            parent_matrix = _parent_parent_matrix * _parent_matrix;
        }

        parent_matrix
    }

    pub fn get_position(&self) -> nalgebra_glm::Vec3 {
        self.position
    }

    pub fn set_position(&mut self, new_position: &nalgebra_glm::Vec3) {
        let pos = new_position - self.position;

        self.position = new_position.clone();

        self.matrix = self.matrix * nalgebra_glm::translate(&nalgebra_glm::Mat4::identity(), &pos);
    }

    pub fn add_position(&mut self, new_position: &nalgebra_glm::Vec3) {
        self.position = self.position + new_position.clone();

        self.matrix =
            self.matrix * nalgebra_glm::translate(&nalgebra_glm::Mat4::identity(), &new_position);
    }

    pub fn forward(&self) -> nalgebra_glm::Vec3 {
        nalgebra_glm::row(&nalgebra_glm::quat_to_mat4(&self.q_rotation), 2).xyz()
    }

    pub fn up(&self) -> nalgebra_glm::Vec3 {
        nalgebra_glm::row(&nalgebra_glm::quat_to_mat4(&self.q_rotation), 1).xyz()
    }

    pub fn right(&self) -> nalgebra_glm::Vec3 {
        nalgebra_glm::row(&nalgebra_glm::quat_to_mat4(&self.q_rotation), 0).xyz()
    }

    pub fn get_rotation(&self) -> nalgebra_glm::Vec3 {
        self.rotation
    }

    pub fn get_q_rotation(&self) -> nalgebra_glm::Quat {
        self.q_rotation
    }

    pub fn add_rotation_x(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &self.right(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));

        self.matrix = self.matrix * nalgebra_glm::quat_to_mat4(&rot);
    }

    pub fn add_rotation_y(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &self.up(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));

        self.matrix = self.matrix * nalgebra_glm::quat_to_mat4(&rot);
    }

    pub fn add_rotation_z(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &self.forward(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));

        self.matrix = self.matrix * nalgebra_glm::quat_to_mat4(&rot);
    }

    pub fn add_rotation_global_x(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &nalgebra_glm::Vec3::x(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));

        self.matrix = self.matrix * nalgebra_glm::quat_to_mat4(&rot);
    }

    pub fn add_rotation_global_y(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &nalgebra_glm::Vec3::y(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));

        self.matrix = self.matrix * nalgebra_glm::quat_to_mat4(&rot);
    }

    pub fn add_rotation_global_z(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &nalgebra_glm::Vec3::z(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));

        self.matrix = self.matrix * nalgebra_glm::quat_to_mat4(&rot);
    }

    pub fn add_rotation_axis(&mut self, angle: f32, axis: &nalgebra_glm::Vec3) {
        let rot =
            nalgebra_glm::quat_rotate(&nalgebra_glm::Quat::identity(), angle.to_radians(), axis);

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));

        self.matrix = self.matrix * nalgebra_glm::quat_to_mat4(&rot);
    }
}
