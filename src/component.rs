pub mod camera;
// pub mod input;
pub mod render;
pub mod transform;

pub use camera::{Camera, CameraType};
pub use render::Render;

pub use transform::Transform;
pub use transform::TransformBuild;

pub type TransformType = std::sync::Arc<std::sync::Mutex<Transform>>;
