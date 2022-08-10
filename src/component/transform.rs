#[derive(Debug)]
pub struct Transform {
    pub position: nalgebra_glm::Vec3,
    pub rotation: nalgebra_glm::Vec3,
    pub size: nalgebra_glm::Vec3,
    q_rotation: nalgebra_glm::Quat,
}

impl Default for Transform {
    fn default() -> Self {
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

        let rotation = (rot_x * rot_y * rot_z).normalize();

        Self {
            position: nalgebra_glm::zero(),
            rotation: nalgebra_glm::quat_euler_angles(&rotation),
            size: nalgebra_glm::vec3(1.0, 1.0, 1.0),
            q_rotation: rotation,
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

        (rot_x * rot_y * rot_z).normalize()
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

    pub fn add_rotation_x(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &self.right(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));
    }

    pub fn add_rotation_y(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &nalgebra_glm::Vec3::y(),
            // &self.up(), // weird rotation doing circles with mouse
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));
    }

    pub fn add_rotation_z(&mut self, angle: f32) {
        let rot = nalgebra_glm::quat_rotate(
            &nalgebra_glm::Quat::identity(),
            angle.to_radians(),
            &self.forward(),
        );

        self.q_rotation = (self.q_rotation * rot).normalize();

        self.rotation = nalgebra_glm::degrees(&nalgebra_glm::quat_euler_angles(&self.q_rotation));
    }
}
