use haya_collection::List;
use haya_math::BlockPosPacked;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum PathType {
    Blocked,
    Open,
    Walkable,
    WalkableDoor,
    Trapdoor,
    PowderSnow,
    DangerPowderSnow,
    Fence,
    Lava,
    Water,
    WaterBorder,
    Rail,
    UnpassableRail,
    DangerFire,
    DamageFire,
    DangerOther,
    DamageOther,
    DoorOpen,
    DoorWoodClosed,
    DoorIronClosed,
    Breach,
    Leaves,
    StickyHoney,
    Cocoa,
    DamageCautious,
    DangerTrapdoor,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub walked_distance: f32,
    pub cost_malus: f32,
    pub closed: bool,
    pub ty: PathType,
    pub f: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Target(pub Node);

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugData<'a> {
    pub target_nodes: List<'a, Target>,
    pub open_set: List<'a, Node>,
    pub closed_set: List<'a, Node>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Path<'a> {
    pub reached: bool,
    pub index_stream: u32,
    pub target: BlockPosPacked,
    pub nodes: List<'a, Node>,
    pub debug_data: DebugData<'a>,
}
