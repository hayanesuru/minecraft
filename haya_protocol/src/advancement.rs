use crate::HolderSet;
use haya_collection::List;

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockPredicate<'a> {
    pub blocks: Option<HolderSet<'a>>,
}
