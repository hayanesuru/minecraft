#![no_std]

use core::str::from_utf8_unchecked;

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
