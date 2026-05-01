use crate::RgbColor;
use haya_ident::ResourceKey;
use mser::{Either, Utf8};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct TrackedWaypoint<'a> {
    pub identifier: Either<Uuid, Utf8<'a>>,
    pub icon: Icon<'a>,
    pub ty: TrackedWaypointContent,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Icon<'a> {
    pub style: ResourceKey<'a>,
    pub color: Option<RgbColor>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum TrackedWaypointType {
    Empty,
    Vec3i,
    Chunk,
    Azimuth,
}

#[derive(Clone, Serialize, Deserialize)]
#[mser(header = TrackedWaypointType, camel_case)]
pub enum TrackedWaypointContent {
    Empty(EmptyWaypoint),
    Vec3i(Vec3iWaypoint),
    Chunk(ChunkWaypoint),
    Azimuth(AzimuthWaypoint),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EmptyWaypoint {}

#[derive(Clone, Serialize, Deserialize)]
pub struct Vec3iWaypoint {
    #[mser(varint)]
    pub x: i32,
    #[mser(varint)]
    pub y: i32,
    #[mser(varint)]
    pub z: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkWaypoint {
    #[mser(varint)]
    pub x: i32,
    #[mser(varint)]
    pub z: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AzimuthWaypoint {
    pub angle: f32,
}
