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

impl Transform {
    pub fn with_position(mut self, position: nalgebra_glm::Vec3) -> Self {
        self.position = position;
        self
    }
}
