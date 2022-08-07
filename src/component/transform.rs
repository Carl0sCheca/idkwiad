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
            rotation: nalgebra_glm::zero(),
            size: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        }
    }
}
