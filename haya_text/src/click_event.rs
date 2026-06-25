use crate::{exact_one, key};
use alloc::boxed::Box;
use alloc::vec;
use haya_ident::Identifier;
use haya_nbt::{CompoundTag, Deserialize, Serialize, StringTag, Tag};
use mser::Error;

const ACTION: &str = "action";
const VALUE: &str = "value";
const URL: &str = "url";
const PATH: &str = "path";
const COMMAND: &str = "command";
const PAGE: &str = "page";
const ID: &str = "id";
const PAYLOAD: &str = "payload";

const ACTION_K: StringTag = key(ACTION);
const VALUE_K: StringTag = key(VALUE);
const URL_K: StringTag = key(URL);
const PATH_K: StringTag = key(PATH);
const COMMAND_K: StringTag = key(COMMAND);
const PAGE_K: StringTag = key(PAGE);
const ID_K: StringTag = key(ID);
const PAYLOAD_K: StringTag = key(PAYLOAD);

const OPEN_URL: &str = "open_url";
const OPEN_FILE: &str = "open_file";
const RUN_COMMAND: &str = "run_command";
const SUGGEST_COMMAND: &str = "suggest_command";
const SHOW_DIALOG: &str = "show_dialog";
const CHANGE_PAGE: &str = "change_page";
const COPY_TO_CLIPBOARD: &str = "copy_to_clipboard";
const CUSTOM: &str = "custom";

const OPEN_URL_K: StringTag = key(OPEN_URL);
const OPEN_FILE_K: StringTag = key(OPEN_FILE);
const RUN_COMMAND_K: StringTag = key(RUN_COMMAND);
const SUGGEST_COMMAND_K: StringTag = key(SUGGEST_COMMAND);
const SHOW_DIALOG_K: StringTag = key(SHOW_DIALOG);
const CHANGE_PAGE_K: StringTag = key(CHANGE_PAGE);
const COPY_TO_CLIPBOARD_K: StringTag = key(COPY_TO_CLIPBOARD);
const CUSTOM_K: StringTag = key(CUSTOM);

#[derive(Clone)]
pub enum ClickEvent {
    OpenUrl(StringTag),
    OpenFile(StringTag),
    RunCommand(StringTag),
    SuggestCommand(StringTag),
    ShowDialog(Tag),
    ChangePage(i32),
    CopyToClipboard(StringTag),
    Custom(Box<(Identifier, Option<Tag>)>),
}

impl Serialize for ClickEvent {
    fn serialize(&self) -> Tag {
        let action = match self {
            Self::OpenUrl(..) => OPEN_URL_K,
            Self::OpenFile(..) => OPEN_FILE_K,
            Self::RunCommand(..) => RUN_COMMAND_K,
            Self::SuggestCommand(..) => SUGGEST_COMMAND_K,
            Self::ShowDialog(..) => SHOW_DIALOG_K,
            Self::ChangePage(..) => CHANGE_PAGE_K,
            Self::CopyToClipboard(..) => COPY_TO_CLIPBOARD_K,
            Self::Custom(..) => CUSTOM_K,
        };
        let value = match self {
            Self::OpenUrl(url) => vec![(URL_K, url.serialize())],
            Self::OpenFile(path) => vec![(PATH_K, path.serialize())],
            Self::RunCommand(command) => vec![(COMMAND_K, command.serialize())],
            Self::SuggestCommand(command) => vec![(COMMAND_K, command.serialize())],
            Self::ShowDialog(t) => vec![(SHOW_DIALOG_K, t.serialize())],
            Self::ChangePage(page) => vec![(PAGE_K, page.serialize())],
            Self::CopyToClipboard(v) => vec![(VALUE_K, v.serialize())],
            Self::Custom(b) => {
                let (id, payload) = &**b;
                match payload {
                    Some(payload1) => {
                        vec![(ID_K, id.serialize()), (PAYLOAD_K, payload1.serialize())]
                    }
                    None => {
                        vec![(ID_K, id.serialize())]
                    }
                }
            }
        };
        Tag::Compound(CompoundTag::from(vec![
            (ACTION_K, Tag::String(action)),
            (VALUE_K, Tag::Compound(CompoundTag::from(value))),
        ]))
    }
}

impl Deserialize for ClickEvent {
    fn deserialize(nbt: Tag) -> Result<Self, Error> {
        let c = match nbt {
            Tag::Compound(c) => c,
            _ => return Err(Error),
        };
        let mut action = None;
        let mut value = None;
        for (k, v) in c {
            match &*k {
                ACTION => action = Some(v),
                VALUE => value = Some(v),
                _ => return Err(Error),
            }
        }
        let Some(Tag::String(action1)) = action else {
            return Err(Error);
        };
        let Some(Tag::Compound(value1)) = value else {
            return Err(Error);
        };
        let iter = value1.into_iter();
        match &*action1 {
            OPEN_URL => {
                let (k, v) = exact_one(iter)?;
                if &*k == URL {
                    Ok(Self::OpenUrl(StringTag::deserialize(v)?))
                } else {
                    Err(Error)
                }
            }
            OPEN_FILE => {
                let (k, v) = exact_one(iter)?;
                if &*k == PATH {
                    Ok(Self::OpenFile(StringTag::deserialize(v)?))
                } else {
                    Err(Error)
                }
            }
            RUN_COMMAND => {
                let (k, v) = exact_one(iter)?;
                if &*k == COMMAND {
                    let command = StringTag::deserialize(v)?;
                    if command.chars().all(|x| x != '\u{a7}' && !x.is_control()) {
                        Ok(Self::RunCommand(command))
                    } else {
                        Err(Error)
                    }
                } else {
                    Err(Error)
                }
            }
            SUGGEST_COMMAND => {
                let (k, v) = exact_one(iter)?;
                if &*k == COMMAND {
                    let command = StringTag::deserialize(v)?;
                    if command.chars().all(|x| x != '\u{a7}' && !x.is_control()) {
                        Ok(Self::SuggestCommand(command))
                    } else {
                        Err(Error)
                    }
                } else {
                    Err(Error)
                }
            }
            SHOW_DIALOG => {
                let (k, v) = exact_one(iter)?;
                if &*k == SHOW_DIALOG {
                    Ok(Self::ShowDialog(v))
                } else {
                    Err(Error)
                }
            }
            CHANGE_PAGE => {
                let (k, v) = exact_one(iter)?;
                if &*k == PAGE {
                    let page = i32::deserialize(v)?;
                    if page > 0 {
                        Ok(Self::ChangePage(page))
                    } else {
                        Err(Error)
                    }
                } else {
                    Err(Error)
                }
            }
            COPY_TO_CLIPBOARD => {
                let (k, v) = exact_one(iter)?;
                if &*k == VALUE {
                    Ok(Self::CopyToClipboard(StringTag::deserialize(v)?))
                } else {
                    Err(Error)
                }
            }
            CUSTOM => {
                let mut payload = None;
                let mut id = None;
                for (k, v) in iter {
                    match &*k {
                        ID => id = Some(Identifier::deserialize(v)?),
                        PAYLOAD => payload = Some(v),
                        _ => return Err(Error),
                    }
                }
                let id1 = match id {
                    Some(x) => x,
                    None => return Err(Error),
                };
                Ok(Self::Custom(Box::new((id1, payload))))
            }
            _ => Err(Error),
        }
    }
}
