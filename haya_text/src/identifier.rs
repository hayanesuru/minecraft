use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use haya_ident::{Ident, is_valid_path, parse_ident};
use haya_nbt::{Deserialize, Serialize, StringTag, Tag};
use haya_str::HayaStr;
use mser::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier(Inner);

impl Identifier {
    pub fn path(&self) -> &str {
        match &self.0 {
            Inner::Thin { path } => path,
            Inner::Heap { path } => path,
            Inner::Full { path, .. } => path,
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        match &self.0 {
            Inner::Thin { .. } => None,
            Inner::Heap { .. } => None,
            Inner::Full { namespace, .. } => Some(namespace),
        }
    }

    pub const fn new_const(path: &str) -> Option<Self> {
        let b = path.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if !is_valid_path(b[i]) {
                return None;
            }
            i += 1;
        }
        Some(Self(Inner::Thin {
            path: match HayaStr::copy_from(path) {
                Ok(x) => x,
                Err(_) => return None,
            },
        }))
    }

    pub fn new(ident: Ident) -> Self {
        let namespace1 = ident.namespace();
        let path1 = ident.path();
        match namespace1 {
            Some(namespace2) => {
                let namespace = namespace2.to_owned().into_boxed_str();
                let path = path1.to_owned().into_boxed_str();
                Self(Inner::Full { namespace, path })
            }
            None => match HayaStr::copy_from(path1) {
                Ok(path) => Self(Inner::Thin { path }),
                Err(_) => Self(Inner::Heap {
                    path: path1.to_owned().into_boxed_str(),
                }),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Inner {
    Thin { path: HayaStr },
    Heap { path: Box<str> },
    Full { namespace: Box<str>, path: Box<str> },
}

impl Serialize for Identifier {
    fn serialize(&self) -> Tag {
        let namespace = match self.namespace() {
            Some(x) => x,
            None => haya_ident::MINECRAFT,
        };
        let path = self.path();
        let s = match HayaStr::copy_from(namespace) {
            Ok(mut x) => match x.try_extend(":") {
                Ok(_) => match x.try_extend(path) {
                    Ok(_) => unsafe { Some(StringTag::from_ascii_nunzero_unchecked(x)) },
                    Err(_) => None,
                },
                Err(_) => None,
            },
            Err(_) => None,
        };
        match s {
            Some(x) => Tag::String(x),
            None => {
                let l = namespace.len() + 1 + path.len();
                let mut s = String::with_capacity(l);
                s.push_str(namespace);
                s.push(':');
                s.push_str(path);
                Tag::String(StringTag::from_owned(s.into_boxed_str()))
            }
        }
    }
}

impl Deserialize for Identifier {
    fn deserialize(nbt: Tag) -> Result<Self, Error> {
        match nbt {
            Tag::String(s) => match parse_ident(s.as_bytes()) {
                Some(x) => Ok(Self::new(x)),
                None => Err(Error),
            },
            _ => Err(Error),
        }
    }
}
