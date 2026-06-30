use crate::chat::TextComponent;
use crate::key;
use alloc::boxed::Box;
use alloc::vec::Vec;
use haya_ident::Identifier;
use haya_nbt::{CompoundTag, Deserialize, Serialize, StringTag, Tag};
use mser::Error;
use uuid::Uuid;

const ACTION: &str = "action";
const SHOW_TEXT: &str = "show_text";
const SHOW_ITEM: &str = "show_item";
const SHOW_ENTITY: &str = "show_entity";
const SHOW_TEXT_VALUE: &str = "value";
const SHOW_ITEM_COUNT: &str = "count";
const SHOW_ITEM_COMPONENTS: &str = "components";
const ID: &str = "id";
const SHOW_ENTITY_UUID: &str = "uuid";
const SHOW_ENTITY_NAME: &str = "name";

const ACTION_K: StringTag = key(ACTION);
const SHOW_TEXT_K: StringTag = key(SHOW_TEXT);
const SHOW_ITEM_K: StringTag = key(SHOW_ITEM);
const SHOW_ENTITY_K: StringTag = key(SHOW_ENTITY);
const SHOW_TEXT_VALUE_K: StringTag = key(SHOW_TEXT_VALUE);
const SHOW_ITEM_COUNT_K: StringTag = key(SHOW_ITEM_COUNT);
const SHOW_ITEM_COMPONENTS_K: StringTag = key(SHOW_ITEM_COMPONENTS);
const ID_K: StringTag = key(ID);
const SHOW_ENTITY_UUID_K: StringTag = key(SHOW_ENTITY_UUID);
const SHOW_ENTITY_NAME_K: StringTag = key(SHOW_ENTITY_NAME);

#[derive(Clone)]
pub enum HoverEvent {
    Entity(Box<ShowEntity>),
    Item(Box<ShowItem>),
    Text(Box<ShowText>),
}

impl Serialize for HoverEvent {
    fn serialize(&self) -> Tag {
        let action = match self {
            Self::Entity(..) => SHOW_ENTITY_K,
            Self::Item(..) => SHOW_ITEM_K,
            Self::Text(..) => SHOW_TEXT_K,
        };
        let action1 = (ACTION_K, Tag::String(action));
        let cap = match self {
            Self::Entity(show_entity) => {
                if show_entity.name.is_some() {
                    3
                } else {
                    2
                }
            }
            Self::Item(show_item) => {
                show_item.components.is_some() as usize + show_item.count.is_some() as usize + 1
            }
            Self::Text(_) => 1,
        };
        let mut vec = Vec::with_capacity(cap + 1);
        vec.push(action1);
        match self {
            Self::Entity(show_entity) => {
                vec.push((ID_K, show_entity.id.serialize()));
                vec.push((SHOW_ENTITY_UUID_K, show_entity.uuid.serialize()));
                if let Some(name) = show_entity.name.as_ref() {
                    vec.push((SHOW_ENTITY_NAME_K, name.serialize()))
                }
            }
            Self::Item(show_item) => {
                vec.push((ID_K, show_item.id.serialize()));
                if let Some(count) = &show_item.count {
                    vec.push((SHOW_ITEM_COUNT_K, count.serialize()));
                }
                if let Some(components) = &show_item.components {
                    vec.push((SHOW_ITEM_COMPONENTS_K, components.serialize()));
                }
            }
            Self::Text(show_text) => {
                vec.push((SHOW_TEXT_VALUE_K, show_text.value.serialize()));
            }
        }
        Tag::Compound(CompoundTag::from(vec))
    }
}

impl Deserialize for HoverEvent {
    fn deserialize(nbt: Tag) -> Result<Self, Error> {
        let c = match nbt {
            Tag::Compound(x) => x,
            _ => return Err(Error),
        };
        let mut action = None;
        let mut id = None;
        let mut value = None;
        let mut count = None;
        let mut components = None;
        let mut uuid = None;
        let mut name = None;
        for (k, v) in c {
            match &*k {
                ACTION => action = Some(StringTag::deserialize(v)?),
                ID => id = Some(Identifier::deserialize(v)?),
                SHOW_TEXT_VALUE => value = Some(TextComponent::deserialize(v)?),
                SHOW_ITEM_COUNT => count = Some(i32::deserialize(v)?),
                SHOW_ITEM_COMPONENTS => components = Some(v),
                SHOW_ENTITY_UUID => {
                    uuid = Some(match v {
                        Tag::String(x) => match Uuid::parse_str(&x) {
                            Ok(y) => y,
                            Err(_) => return Err(Error),
                        },
                        t => Uuid::deserialize(t)?,
                    })
                }
                SHOW_ENTITY_NAME => name = Some(TextComponent::deserialize(v)?),
                _ => return Err(Error),
            }
        }
        let action1 = match action {
            Some(a) => a,
            None => return Err(Error),
        };
        match &*action1 {
            SHOW_TEXT => match value {
                Some(v) => Ok(Self::Text(Box::new(ShowText { value: v }))),
                None => Err(Error),
            },
            SHOW_ITEM => match id {
                Some(id1) => {
                    let show_item = match count {
                        Some(count1) => match components {
                            Some(components1) => ShowItem {
                                id: id1,
                                count: Some(count1),
                                components: Some(components1),
                            },
                            None => ShowItem {
                                id: id1,
                                count: Some(count1),
                                components: None,
                            },
                        },
                        None => match components {
                            Some(components1) => ShowItem {
                                id: id1,
                                count: None,
                                components: Some(components1),
                            },
                            None => ShowItem {
                                id: id1,
                                count: None,
                                components: None,
                            },
                        },
                    };
                    Ok(Self::Item(Box::new(show_item)))
                }
                _ => Err(Error),
            },
            SHOW_ENTITY => match uuid {
                Some(uuid1) => match id {
                    Some(id1) => match name {
                        Some(name1) => Ok(Self::Entity(Box::new(ShowEntity {
                            id: id1,
                            uuid: uuid1,
                            name: Some(name1),
                        }))),
                        None => Ok(Self::Entity(Box::new(ShowEntity {
                            id: id1,
                            uuid: uuid1,
                            name: None,
                        }))),
                    },
                    None => Err(Error),
                },
                None => Err(Error),
            },
            _ => Err(Error),
        }
    }
}

#[derive(Clone)]
pub struct ShowEntity {
    pub id: Identifier,
    pub uuid: Uuid,
    pub name: Option<TextComponent>,
}

#[derive(Clone)]
pub struct ShowItem {
    pub id: Identifier,
    pub count: Option<i32>,
    pub components: Option<Tag>,
}

#[derive(Clone)]
pub struct ShowText {
    pub value: TextComponent,
}
