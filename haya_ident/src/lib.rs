#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use core::str::from_utf8_unchecked;
use haya_str::HayaStr;
use mser::{ByteArray, Error, Read, UnsafeWriter, V21, Write};

pub const MINECRAFT: &str = "minecraft";

const fn is_valid_path(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/')
}

const fn is_valid_namespace(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.')
}

fn split_once(n: &[u8]) -> Option<(&[u8], &[u8])> {
    let index = n.iter().position(|&x| x == b':')?;
    Some((&n[..index], &n[index + 1..]))
}

pub fn parse_ident(ident: &[u8]) -> Option<(Option<&str>, &str)> {
    if !ident.is_ascii() {
        return None;
    }
    parse_ident_ascii(ident)
}

fn parse_ident_ascii(ident: &[u8]) -> Option<(Option<&str>, &str)> {
    match ident.strip_prefix(b"minecraft:") {
        Some(path) => unsafe {
            if path.iter().copied().all(is_valid_path) {
                Some((None, from_utf8_unchecked(path)))
            } else {
                None
            }
        },
        None => match split_once(ident) {
            Some((ns, path)) => unsafe {
                if ns.iter().copied().all(is_valid_namespace)
                    && path.iter().copied().all(is_valid_path)
                {
                    Some((
                        if !ns.is_empty() {
                            Some(from_utf8_unchecked(ns))
                        } else {
                            None
                        },
                        from_utf8_unchecked(path),
                    ))
                } else {
                    None
                }
            },
            None => unsafe {
                if ident.iter().copied().all(is_valid_path) {
                    Some((None, from_utf8_unchecked(ident)))
                } else {
                    None
                }
            },
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ident<'a> {
    namespace: Option<&'a str>,
    path: &'a str,
}

impl<'a> Ident<'a> {
    pub fn namespace(&self) -> Option<&str> {
        self.namespace
    }

    pub fn path(&self) -> &str {
        self.path
    }
}

impl<'a> Read<'a> for Ident<'a> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let identifier = ByteArray::<32767>::read(buf)?.0;
        match parse_ident(identifier) {
            Some((namespace, path)) => Ok(Self { namespace, path }),
            None => Err(Error),
        }
    }
}

impl Write for Ident<'_> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let namespace = match self.namespace {
                Some(x) => x,
                None => MINECRAFT,
            };
            V21((namespace.len() + 1 + self.path.len()) as _).write(w);
            w.write(namespace.as_bytes());
            w.write_byte(b':');
            w.write(self.path.as_bytes());
        }
    }

    fn len_s(&self) -> usize {
        let namespace = match self.namespace {
            Some(x) => x,
            None => MINECRAFT,
        };
        let a = namespace.len() + 1 + self.path.len();
        V21(a as u32).len_s() + a
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Identifier(Inner);

impl Identifier {
    pub fn new(ident: Ident) -> Self {
        let Ident { namespace, path } = ident;
        match namespace {
            Some(namespace) => {
                let namespace = namespace.to_owned().into_boxed_str();
                let path = path.to_owned().into_boxed_str();
                Self(Inner::Full { namespace, path })
            }
            None => match HayaStr::new(path) {
                Ok(path) => Self(Inner::Thin { path }),
                Err(_) => Self(Inner::Heap {
                    path: path.to_owned().into_boxed_str(),
                }),
            },
        }
    }

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
}

#[derive(Clone, PartialEq, Eq)]
enum Inner {
    Thin { path: HayaStr },
    Heap { path: Box<str> },
    Full { namespace: Box<str>, path: Box<str> },
}
