#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use core::str::from_utf8_unchecked;
use haya_str::HayaStr;
use mser::{ByteArray, Error, Read, Reader, V21, Write, Writer};

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

pub fn parse_ident(ident: &[u8]) -> Option<Ident<'_>> {
    if !ident.is_ascii() {
        return None;
    }
    parse_ident_ascii(ident)
}

fn parse_ident_ascii(ident: &[u8]) -> Option<Ident<'_>> {
    match ident.strip_prefix(b"minecraft:") {
        Some(path) => unsafe {
            if path.iter().copied().all(is_valid_path) {
                Some(Ident::new_unchecked(None, from_utf8_unchecked(path)))
            } else {
                None
            }
        },
        None => match split_once(ident) {
            Some((ns, path)) => unsafe {
                if ns.iter().copied().all(is_valid_namespace)
                    && path.iter().copied().all(is_valid_path)
                {
                    Some(Ident::new_unchecked(
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
                    Some(Ident::new_unchecked(None, from_utf8_unchecked(ident)))
                } else {
                    None
                }
            },
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ident<'a> {
    namespace: Option<&'a str>,
    path: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceKey<'a> {
    pub identifier: Ident<'a>,
}

impl Write for ResourceKey<'_> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe { self.identifier.write(w) }
    }
    fn len_s(&self) -> usize {
        self.identifier.len_s()
    }
}

impl<'a> Read<'a> for ResourceKey<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self {
            identifier: Ident::read(buf)?,
        })
    }
}

impl<'a> Ident<'a> {
    /// # Safety
    ///
    /// The namespace and path must be valid.
    pub const unsafe fn new_unchecked(namespace: Option<&'a str>, path: &'a str) -> Self {
        Self { namespace, path }
    }

    pub const fn namespace(&self) -> Option<&str> {
        self.namespace
    }

    pub const fn path(&self) -> &str {
        self.path
    }
}

impl<'a> Read<'a> for Ident<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let identifier = ByteArray::<32767>::read(buf)?.0;
        match parse_ident(identifier) {
            Some(Ident { namespace, path }) => Ok(Self { namespace, path }),
            None => Err(Error),
        }
    }
}

impl Write for Ident<'_> {
    unsafe fn write(&self, w: &mut Writer) {
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
            None => match HayaStr::copy_from(path) {
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Inner {
    Thin { path: HayaStr },
    Heap { path: Box<str> },
    Full { namespace: Box<str>, path: Box<str> },
}

#[derive(Clone, Debug)]
pub struct TagKey<'a> {
    pub location: Ident<'a>,
}

impl Write for TagKey<'_> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe { self.location.write(w) }
    }
    fn len_s(&self) -> usize {
        self.location.len_s()
    }
}

impl<'a> Read<'a> for TagKey<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self {
            location: Ident::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use mser::write_unchecked;

    #[track_caller]
    fn test_write_read(n: Ident, expected: &str) {
        let len = n.len_s();
        let mut v = Vec::with_capacity(len);
        unsafe {
            write_unchecked(v.as_mut_ptr(), &n);
            v.set_len(len);
        }
        let mut reader = Reader::new(&v);
        let r = Ident::read(&mut reader).unwrap();
        assert_eq!(r, n);
        let mut reader = Reader::new(&v);
        let len = V21::read(&mut reader).unwrap().0 as usize;
        assert_eq!(len, reader.len());
        assert_eq!(
            reader.read_slice(reader.len()).unwrap(),
            expected.as_bytes()
        );
    }

    #[track_caller]
    fn test_parse(n: &str, expected: &str) {
        test_write_read(parse_ident(n.as_bytes()).unwrap(), expected);
    }

    fn test_parse_f(n: &str) {
        assert_eq!(parse_ident(n.as_bytes()), None);
    }

    #[test]
    fn test_ident() {
        unsafe {
            test_write_read(Ident::new_unchecked(None, "diamond"), "minecraft:diamond");
            test_write_read(Ident::new_unchecked(Some("foo"), "bar.baz"), "foo:bar.baz");
            test_write_read(
                Ident::new_unchecked(Some("minecraftwiki"), "commands/minecraft_wiki"),
                "minecraftwiki:commands/minecraft_wiki",
            );

            test_parse("bar:code", "bar:code");
            test_parse("minecraft:zombie", "minecraft:zombie");
            test_parse("diamond", "minecraft:diamond");
            test_parse(":dirt", "minecraft:dirt");
            test_parse("minecraft:", "minecraft:");
            test_parse(":", "minecraft:");
            test_parse("", "minecraft:");
            test_parse_f("foo/bar:coal");
            test_parse("minecraft/villager", "minecraft:minecraft/villager");
            test_parse_f("mymap:schrödingers_var");
            test_parse_f("custom_pack:Capital");
        }
    }
}
