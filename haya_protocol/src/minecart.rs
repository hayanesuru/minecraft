use haya_math::{ByteAngle, Vec3};

#[derive(Clone, Serialize, Deserialize)]
pub struct MinecartStep {
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: ByteAngle,
    pub x_rot: ByteAngle,
    pub weight: f32,
}
