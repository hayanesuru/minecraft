use crate::Component;
use haya_collection::Map;
use haya_ident::{Ident, ResourceKey};
use mser::Utf8;

#[derive(Clone, Serialize, Deserialize)]
pub struct TrimMaterial<'a> {
    pub assets: MaterialAssetGroup<'a>,
    pub description: Component,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MaterialAssetGroup<'a> {
    pub base: AssetInfo<'a>,
    pub overrides: Map<'a, ResourceKey<'a>, AssetInfo<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AssetInfo<'a> {
    pub suffix: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrimPattern<'a> {
    pub asset_id: Ident<'a>,
    pub description: Component,
    pub decal: bool,
}
