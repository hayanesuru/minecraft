use crate::HolderSet;
use crate::item::TypedDataComponentType;
use haya_collection::List;
use haya_nbt::Tag;
use minecraft_data::{block, data_component_predicate_type, data_component_type};
use mser::{Either, Utf8};

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockPredicate<'a> {
    pub blocks: Option<HolderSet<'a, block>>,
    pub properties: Option<StatePropertiesPredicate<'a>>,
    pub nbt: Option<Tag>,
    pub components: DataComponentMatchers<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StatePropertiesPredicate<'a> {
    pub properties: List<'a, PropertyMatcher<'a>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PropertyMatcher<'a> {
    pub name: Utf8<'a>,
    pub value_matcher: ValueMatcher<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ValueMatcher<'a>(pub Either<ExactMatcher<'a>, RangedMatcher<'a>>);

#[derive(Clone, Serialize, Deserialize)]
pub struct ExactMatcher<'a>(pub Utf8<'a>);

#[derive(Clone, Serialize, Deserialize)]
pub struct RangedMatcher<'a> {
    pub min: Utf8<'a>,
    pub max: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DataComponentMatchers<'a> {
    pub exact: List<'a, TypedDataComponentType<'a>>,
    pub partial: List<'a, SingleDataComponentPredicate, 64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SingleDataComponentPredicate(
    pub Either<data_component_predicate_type, data_component_type>,
    pub Tag,
);
