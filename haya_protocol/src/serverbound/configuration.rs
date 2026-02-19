use crate::{KnownPack, List};
use haya_ident::Ident;
use haya_nbt::Tag;

#[derive(Clone, Serialize, Deserialize)]
pub struct FinishConfiguration {}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectKnownPacks<'a> {
    pub known_packs: List<'a, KnownPack<'a>, 64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomClickAction<'a> {
    pub id: Ident<'a>,
    pub payload: Option<Tag>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AcceptCodeOfConduct {}
