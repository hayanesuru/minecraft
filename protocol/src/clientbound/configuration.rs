use crate::nbt::Tag;
use crate::{Ident, KnownPack, List, RegistryKey};

#[derive(Clone, Serialize, Deserialize)]
pub struct FinishConfiguration {}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegistryData<'a> {
    pub registry: RegistryKey<'a>,
    pub entries: List<'a, RegistryEntry<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RegistryEntry<'a> {
    pub id: Ident<'a>,
    pub data: Option<Tag>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateEnabledFeatures<'a> {
    pub features: List<'a, Ident<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SelectKnownPacks<'a> {
    pub known_packs: List<'a, KnownPack<'a>>,
}
