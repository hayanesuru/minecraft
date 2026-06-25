use crate::{capacity_fix, key};
use alloc::vec;
use alloc::vec::Vec;
use haya_ident::Identifier;
use haya_nbt::{CompoundTag, Deserialize, ListTag, Serialize, StringTag, Tag};
use mser::Error;
use uuid::Uuid;

const NAME: &str = "name";
const ID: &str = "id";
const PROPERTIES: &str = "properties";
const PROPERTY_NAME: &str = "name";
const PROPERTY_VALUE: &str = "value";
const PROPERTY_SIGNATURE: &str = "signature";
const NAME_K: StringTag = key(NAME);
const ID_K: StringTag = key(ID);
const PROPERTIES_K: StringTag = key(PROPERTIES);
const PROPERTY_NAME_K: StringTag = key(PROPERTY_NAME);
const PROPERTY_VALUE_K: StringTag = key(PROPERTY_VALUE);
const PROPERTY_SIGNATURE_K: StringTag = key(PROPERTY_SIGNATURE);

const TEXTURE: &str = "texture";
const CAPE: &str = "cape";
const ELYTRA: &str = "elytra";
const MODEL: &str = "model";
const SLIM: &str = "slim";
const WIDE: &str = "wide";

const TEXTURE_K: StringTag = key(TEXTURE);
const CAPE_K: StringTag = key(CAPE);
const ELYTRA_K: StringTag = key(ELYTRA);
const MODEL_K: StringTag = key(MODEL);
const SLIM_K: StringTag = key(SLIM);
const WIDE_K: StringTag = key(WIDE);

#[derive(Clone)]
pub struct ResolvableProfile {
    pub name: Option<StringTag>,
    pub id: Option<Uuid>,
    pub properties: PropertyMap,
    pub skin_patch: PlayerSkinPatch,
}

impl Deserialize for ResolvableProfile {
    fn deserialize(nbt: Tag) -> Result<Self, Error> {
        match nbt {
            Tag::String(s) => {
                if !validate_name(&s) {
                    return Err(Error);
                }
                Ok(Self {
                    name: Some(s),
                    id: None,
                    properties: PropertyMap(Vec::new()),
                    skin_patch: PlayerSkinPatch {
                        texture: None,
                        cape: None,
                        elytra: None,
                        model: None,
                    },
                })
            }
            Tag::Compound(c) => {
                let mut x = Self {
                    name: None,
                    id: None,
                    properties: PropertyMap(Vec::new()),
                    skin_patch: PlayerSkinPatch {
                        texture: None,
                        cape: None,
                        elytra: None,
                        model: None,
                    },
                };
                for (k, v) in c {
                    match &*k {
                        NAME => {
                            let name = StringTag::deserialize(v)?;
                            if !validate_name(&name) {
                                return Err(Error);
                            }
                            x.name = Some(name);
                        }
                        ID => {
                            let id = Uuid::deserialize(v)?;
                            x.id = Some(id);
                        }
                        PROPERTIES => {
                            let props = match v {
                                Tag::Compound(props) => {
                                    if props.len() > 16 {
                                        return Err(Error);
                                    }
                                    let mut cap = 0;
                                    for (_, v) in props.iter() {
                                        match v {
                                            Tag::List(ListTag::String(s)) => {
                                                cap += s.len();
                                            }
                                            Tag::List(ListTag::None) => {}
                                            _ => return Err(Error),
                                        }
                                    }
                                    if cap > 512 {
                                        return Err(Error);
                                    }
                                    let mut vec = Vec::with_capacity(capacity_fix(cap));
                                    for (k, v) in props {
                                        if utf16(&k) > 64 {
                                            return Err(Error);
                                        }
                                        match v {
                                            Tag::List(ListTag::String(s)) => {
                                                if vec.len() == vec.capacity() {
                                                    return Err(Error);
                                                }
                                                if s.len() == 1 {
                                                    let tag = match s.into_iter().next() {
                                                        Some(x) => x,
                                                        None => return Err(Error),
                                                    };
                                                    if utf16(&tag) > 32767 {
                                                        return Err(Error);
                                                    }
                                                    vec.push(Property {
                                                        name: k,
                                                        value: tag,
                                                        signature: None,
                                                    });
                                                } else {
                                                    for s in s {
                                                        if utf16(&s) > 32767 {
                                                            return Err(Error);
                                                        }
                                                        vec.push(Property {
                                                            name: k.clone(),
                                                            value: s,
                                                            signature: None,
                                                        });
                                                    }
                                                }
                                            }
                                            Tag::List(ListTag::None) => {}
                                            _ => return Err(Error),
                                        }
                                    }
                                    vec
                                }
                                Tag::List(ListTag::Compound(props)) => {
                                    if props.len() > 16 {
                                        return Err(Error);
                                    }
                                    let mut v = Vec::with_capacity(capacity_fix(props.len()));
                                    for c in props {
                                        let mut name: Option<StringTag> = None;
                                        let mut value: Option<StringTag> = None;
                                        let mut signature: Option<StringTag> = None;
                                        for (k, v) in c {
                                            match &*k {
                                                PROPERTY_NAME => {
                                                    let n = StringTag::deserialize(v)?;
                                                    if utf16(&n) > 64 {
                                                        return Err(Error);
                                                    }
                                                    name = Some(n);
                                                }
                                                PROPERTY_VALUE => {
                                                    let val = StringTag::deserialize(v)?;
                                                    if utf16(&val) > 32767 {
                                                        return Err(Error);
                                                    }
                                                    value = Some(val);
                                                }
                                                PROPERTY_SIGNATURE => {
                                                    let sig = StringTag::deserialize(v)?;
                                                    if utf16(&sig) > 1024 {
                                                        return Err(Error);
                                                    }
                                                    signature = Some(sig);
                                                }
                                                _ => return Err(Error),
                                            }
                                        }
                                        if let Some(name2) = name
                                            && let Some(value2) = value
                                        {
                                            v.push(Property {
                                                name: name2,
                                                value: value2,
                                                signature,
                                            });
                                        } else {
                                            return Err(Error);
                                        }
                                    }
                                    v
                                }
                                Tag::List(ListTag::None) => Vec::new(),
                                _ => return Err(Error),
                            };
                            x.properties = PropertyMap(props);
                        }
                        TEXTURE => {
                            x.skin_patch.texture = Some(Identifier::deserialize(v)?);
                        }
                        CAPE => {
                            x.skin_patch.cape = Some(Identifier::deserialize(v)?);
                        }
                        ELYTRA => {
                            x.skin_patch.elytra = Some(Identifier::deserialize(v)?);
                        }
                        MODEL => {
                            let model = StringTag::deserialize(v)?;
                            x.skin_patch.model = match &*model {
                                SLIM => Some(PlayerModelType::Slim),
                                WIDE => Some(PlayerModelType::Wide),
                                _ => return Err(Error),
                            };
                        }
                        _ => {
                            return Err(Error);
                        }
                    }
                }
                Ok(x)
            }
            _ => Err(Error),
        }
    }
}

fn validate_name(n: &str) -> bool {
    n.len() <= 16 && n.as_bytes().iter().all(|&x| x > 32 && x < 127)
}

impl Serialize for ResolvableProfile {
    fn serialize(&self) -> Tag {
        let mut c = Vec::new();
        if let Some(name) = self.name.as_ref() {
            c.push((NAME_K, name.serialize()));
        }
        if let Some(id) = self.id.as_ref() {
            c.push((ID_K, id.serialize()));
        }
        if !self.properties.0.is_empty() {
            let mut list = Vec::with_capacity(self.properties.0.len());
            for p in self.properties.0.iter() {
                let prop = match &p.signature {
                    Some(signature) => {
                        vec![
                            (PROPERTY_NAME_K, p.name.serialize()),
                            (PROPERTY_VALUE_K, p.value.serialize()),
                            (PROPERTY_SIGNATURE_K, signature.serialize()),
                        ]
                    }
                    None => {
                        vec![
                            (PROPERTY_NAME_K, p.name.serialize()),
                            (PROPERTY_VALUE_K, p.value.serialize()),
                        ]
                    }
                };
                list.push(CompoundTag::from(prop));
            }
            c.push((PROPERTIES_K, Tag::List(ListTag::Compound(list))));
        }
        if let Some(texture) = self.skin_patch.texture.as_ref() {
            c.push((TEXTURE_K, texture.serialize()));
        }
        if let Some(cape) = self.skin_patch.cape.as_ref() {
            c.push((CAPE_K, cape.serialize()));
        }
        if let Some(elytra) = self.skin_patch.elytra.as_ref() {
            c.push((ELYTRA_K, elytra.serialize()));
        }
        if let Some(model) = self.skin_patch.model {
            c.push((
                MODEL_K,
                match model {
                    PlayerModelType::Slim => Tag::String(SLIM_K),
                    PlayerModelType::Wide => Tag::String(WIDE_K),
                },
            ));
        }
        Tag::Compound(CompoundTag::from(c))
    }
}

#[derive(Clone)]
pub struct GameProfile {
    pub name: StringTag,
    pub id: Uuid,
    pub properties: PropertyMap,
    pub patch: PlayerSkinPatch,
}

#[derive(Clone)]
pub struct PropertyMap(pub Vec<Property>);

#[derive(Clone)]
pub struct Property {
    pub name: StringTag,
    pub value: StringTag,
    pub signature: Option<StringTag>,
}

#[derive(Clone)]
pub struct PlayerSkinPatch {
    pub texture: Option<Identifier>,
    pub cape: Option<Identifier>,
    pub elytra: Option<Identifier>,
    pub model: Option<PlayerModelType>,
}

#[derive(Clone, Copy)]
pub enum PlayerModelType {
    Slim,
    Wide,
}

fn utf16(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum::<usize>()
}
