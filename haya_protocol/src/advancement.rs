use crate::HolderSet;
use haya_collection::List;
use mser::{Either, Utf8};

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockPredicate<'a> {
    pub blocks: Option<HolderSet<'a>>,
    pub properties: Option<StatePropertiesPredicate<'a>>,
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
