use mser::V32;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FoodProperties {
    pub nutrition: V32,
    pub saturation: f32,
    pub can_always_eat: bool,
}
