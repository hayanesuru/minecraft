use crate::registry::BannerPatternRef;
use crate::{DyeColor, Holder};
use haya_collection::List;
use haya_ident::Ident;
use mser::Utf8;

#[derive(Clone, Serialize, Deserialize)]
pub struct BannerPattern<'a> {
    pub asset_id: Ident<'a>,
    pub translation_key: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BannerPatternLayers<'a> {
    pub layers: List<'a, BannerPatternLayer<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BannerPatternLayer<'a> {
    pub pattern: Holder<BannerPattern<'a>, BannerPatternRef>,
    pub color: DyeColor,
}
