#[derive(Debug)]
pub struct Transform {
    pub position: nalgebra_glm::Vec3,
    pub rotation: nalgebra_glm::Vec3,
    pub size: nalgebra_glm::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: nalgebra_glm::zero(),
            rotation: nalgebra_glm::row(
                &nalgebra_glm::quat_to_mat4(&nalgebra_glm::Quat::identity()),
                2,
            )
            .xyz(),
            size: nalgebra_glm::vec3(1.0, 1.0, 1.0),
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
        self
    }

    pub fn with_rotation(mut self, rotation: nalgebra_glm::Vec3) -> Self {
        self.0.rotation = rotation;
        self
    }

    pub fn build(self) -> Transform {
        self.0
    }
}

#[allow(dead_code)]
impl Transform {
    pub fn q_rotation(&self) -> nalgebra_glm::Quat {
        let rot_x = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(self.rotation.x),
            &nalgebra_glm::Vec3::x(),
        );
        let rot_y = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(self.rotation.y),
            &nalgebra_glm::Vec3::y(),
        );
        let rot_z = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            f32::to_radians(self.rotation.z),
            &nalgebra_glm::Vec3::z(),
        );
        let rotation = rot_y * rot_z * rot_x;

        rotation.normalize()
    }

    pub fn forward(&self) -> nalgebra_glm::Vec3 {
        let rotation = self.q_rotation();

        nalgebra_glm::quat_rotate_vec3(&rotation, &nalgebra_glm::Vec3::z()).normalize()
        // nalgebra_glm::row(&nalgebra_glm::quat_to_mat4(&rotation), 2).xyz()
    }

    pub fn up(&self) -> nalgebra_glm::Vec3 {
        let rotation = self.q_rotation();

        nalgebra_glm::quat_rotate_vec3(&rotation, &nalgebra_glm::Vec3::y()).normalize()
        // nalgebra_glm::row(&nalgebra_glm::quat_to_mat4(&rotation), 1).xyz()
    }

    pub fn right(&self) -> nalgebra_glm::Vec3 {
        let rotation = self.q_rotation();

        nalgebra_glm::quat_rotate_vec3(&rotation, &nalgebra_glm::Vec3::x()).normalize()
        // nalgebra_glm::row(&nalgebra_glm::quat_to_mat4(&rotation), 0).xyz()
    }
}
