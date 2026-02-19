use crate::{KnownPack, List};

#[derive(Clone, Serialize, Deserialize)]
pub struct FinishConfiguration {}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectKnownPacks<'a> {
    pub known_packs: List<'a, KnownPack<'a>, 64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AcceptCodeOfConduct {}
