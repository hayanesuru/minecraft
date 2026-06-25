#![no_std]
#![warn(clippy::shadow_reuse, clippy::use_self)]

use haya_nbt::StringTag;

extern crate alloc;

pub mod chat;
pub mod click_event;
pub mod color;
pub mod decoration;
pub mod dialog;
pub mod hover_event;
pub mod json;
pub mod profile;

const fn key(n: &'static str) -> StringTag {
    StringTag::from_ascii_nonzero(n.as_bytes()).unwrap()
}

#[inline(always)]
fn capacity_fix(len: usize) -> usize {
    usize::min(len, 256)
}

fn exact_one<T>(mut iter: alloc::vec::IntoIter<T>) -> Result<T, mser::Error> {
    if iter.len() != 1 {
        Err(mser::Error)
    } else {
        match iter.next() {
            Some(x) => Ok(x),
            None => Err(mser::Error),
        }
    }
}
