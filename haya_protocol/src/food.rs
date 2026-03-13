#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FoodProperties {
    #[mser(varint)]
    pub nutrition: u32,
    pub saturation: f32,
    pub can_always_eat: bool,
}
