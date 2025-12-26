mod list;
mod string;
mod stringify;

pub use self::list::{List, ListInfo};
pub use self::string::{IdentifierTag, RefStringTag, StringTagRaw};
pub use self::stringify::StringifyCompound;
use crate::chat::Component;
use crate::profile::ResolvableProfile;
use crate::str::BoxStr;
use crate::{Bytes, Error, Ident, Identifier, Read, UnsafeWriter, Write};
use alloc::alloc::{Allocator, Global};
use alloc::boxed::Box;
use alloc::vec::Vec;
use mser::{decode_mutf8, decode_mutf8_len, is_ascii_mutf8};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
}

#[must_use]
pub struct Kv<'a, T>(pub &'a [u8], pub T);

impl<'a, A: Allocator> Write for Kv<'a, &'a BoxStr<A>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            RefStringTag(self.1).write(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::String.sz() + StringTagRaw::new_unchecked(self.0).sz() + RefStringTag(self.1).sz()
    }
}

impl<'a> Write for Kv<'a, &'a str> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            RefStringTag(self.1).write(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::String.sz() + StringTagRaw::new_unchecked(self.0).sz() + RefStringTag(self.1).sz()
    }
}

impl<'a> Write for Kv<'a, ListInfo> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::List.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::List.sz() + StringTagRaw::new_unchecked(self.0).sz() + self.1.sz()
    }
}

impl<'a> Write for Kv<'a, StringTagRaw<'a>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::String.sz() + StringTagRaw::new_unchecked(self.0).sz() + self.1.sz()
    }
}

impl<'a> Write for Kv<'a, bool> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Byte.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::Byte.sz() + StringTagRaw::new_unchecked(self.0).sz() + self.1.sz()
    }
}

impl<'a> Write for Kv<'a, Ident<'a>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            IdentifierTag(self.1).write(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::String.sz() + StringTagRaw::new_unchecked(self.0).sz() + IdentifierTag(self.1).sz()
    }
}

impl<'a, A: Allocator> Write for Kv<'a, &'a Identifier<A>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::String.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            IdentifierTag(self.1.as_ident()).write(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::String.sz()
            + StringTagRaw::new_unchecked(self.0).sz()
            + IdentifierTag(self.1.as_ident()).sz()
    }
}

impl<'a, A: Allocator> Write for Kv<'a, &'a Component<A>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write_ty(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::Compound.sz() + StringTagRaw::new_unchecked(self.0).sz() + self.1.ty_sz()
    }
}

impl<'a, A: Allocator> Write for Kv<'a, &'a ResolvableProfile<A>> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            StringTagRaw::new_unchecked(self.0).write(w);
            self.1.write_ty(w);
        }
    }

    fn sz(&self) -> usize {
        TagType::Compound.sz() + StringTagRaw::new_unchecked(self.0).sz() + self.1.ty_sz()
    }
}

impl TagType {
    pub fn expect_bool(self, buf: &mut &[u8]) -> Result<bool, Error> {
        match self {
            Self::Byte => Ok(buf.i8()? != 0),
            Self::Short => Ok(buf.i16()? != 0),
            Self::Int => Ok(buf.i32()? != 0),
            Self::Long => Ok(buf.i64()? != 0),
            Self::Float => Ok(buf.f32()? != 0.0),
            Self::Double => Ok(buf.f64()? != 0.0),
            _ => Err(Error),
        }
    }

    pub fn expect_str(self, buf: &mut &[u8]) -> Result<BoxStr, Error> {
        match self {
            Self::String => match StringTag::read(buf) {
                Ok(x) => Ok(x.0),
                Err(e) => Err(e),
            },
            _ => Err(Error),
        }
    }

    pub fn expect_ident(self, buf: &mut &[u8]) -> Result<Identifier, Error> {
        match self {
            Self::String => match IdentifierTag::read(buf) {
                Ok(x) => unsafe {
                    Ok(Identifier {
                        namespace: if x.0.namespace == Ident::MINECRAFT {
                            None
                        } else {
                            Some(BoxStr::new_unchecked(Box::from(x.0.namespace.as_bytes())))
                        },
                        path: BoxStr::new_unchecked(Box::from(x.0.path.as_bytes())),
                    })
                },
                Err(e) => Err(e),
            },
            _ => Err(Error),
        }
    }
}

impl Read<'_> for TagType {
    #[inline]
    fn read(buf: &mut &'_ [u8]) -> Result<Self, Error> {
        let t = buf.u8()?;
        if t <= 12 {
            unsafe { Ok(core::mem::transmute::<u8, Self>(t)) }
        } else {
            Err(Error)
        }
    }
}

impl Write for TagType {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { w.write_byte(*self as u8) }
    }

    #[inline]
    fn sz(&self) -> usize {
        1
    }
}

#[derive(Clone)]
pub enum Tag<A: Allocator = Global> {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(BoxStr<A>),
    ByteArray(Vec<i8, A>),
    IntArray(Vec<i32, A>),
    LongArray(Vec<i64, A>),
    List(List<A>),
    Compound(Compound<A>),
}

impl Write for Tag {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            self.id().write(w);
            match self {
                Tag::Byte(x) => x.write(w),
                Tag::Short(x) => x.write(w),
                Tag::Int(x) => x.write(w),
                Tag::Long(x) => x.write(w),
                Tag::Float(x) => x.write(w),
                Tag::Double(x) => x.write(w),
                Tag::String(x) => RefStringTag(x).write(w),
                Tag::ByteArray(x) => {
                    (x.len() as u32).write(w);
                    w.write(&*(x.as_slice() as *const [i8] as *const [u8]));
                }
                Tag::IntArray(x) => {
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Tag::LongArray(x) => {
                    (x.len() as u32).write(w);
                    x.iter().write(w);
                }
                Tag::List(x) => x.write(w),
                Tag::Compound(x) => x.write(w),
            }
        }
    }

    fn sz(&self) -> usize {
        1 + match self {
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 4,
            Tag::Long(_) => 8,
            Tag::Float(_) => 4,
            Tag::Double(_) => 8,
            Tag::String(x) => RefStringTag(x).sz(),
            Tag::ByteArray(x) => 4 + x.len(),
            Tag::IntArray(x) => 4 + x.len() * 4,
            Tag::LongArray(x) => 4 + x.len() * 8,
            Tag::List(x) => x.sz(),
            Tag::Compound(x) => x.sz(),
        }
    }
}

impl Tag {
    #[inline]
    pub const fn id(&self) -> TagType {
        match self {
            Tag::Byte(_) => TagType::Byte,
            Tag::Short(_) => TagType::Short,
            Tag::Int(_) => TagType::Int,
            Tag::Long(_) => TagType::Long,
            Tag::Float(_) => TagType::Float,
            Tag::Double(_) => TagType::Double,
            Tag::String(_) => TagType::String,
            Tag::ByteArray(_) => TagType::ByteArray,
            Tag::IntArray(_) => TagType::IntArray,
            Tag::LongArray(_) => TagType::LongArray,
            Tag::List(_) => TagType::List,
            Tag::Compound(_) => TagType::Compound,
        }
    }
}

impl From<bool> for Tag {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Byte(value as i8)
    }
}

impl From<u8> for Tag {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Byte(value as i8)
    }
}

impl From<i8> for Tag {
    fn from(value: i8) -> Self {
        Self::Byte(value)
    }
}
impl From<i16> for Tag {
    #[inline]
    fn from(value: i16) -> Self {
        Self::Short(value)
    }
}

impl From<i32> for Tag {
    #[inline]
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<i64> for Tag {
    #[inline]
    fn from(value: i64) -> Self {
        Self::Long(value)
    }
}

impl From<f32> for Tag {
    #[inline]
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for Tag {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<Vec<u8>> for Tag {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        let mut me = core::mem::ManuallyDrop::new(value);
        Self::ByteArray(unsafe {
            Vec::from_raw_parts(me.as_mut_ptr().cast::<i8>(), me.len(), me.capacity())
        })
    }
}

impl From<Vec<i8>> for Tag {
    #[inline]
    fn from(value: Vec<i8>) -> Self {
        Self::ByteArray(value)
    }
}

impl<'a> From<&'a str> for Tag {
    #[inline]
    fn from(value: &'a str) -> Self {
        unsafe { Self::String(BoxStr::new_unchecked(Box::from(value.as_bytes()))) }
    }
}

impl<'a> From<&'a mut str> for Tag {
    #[inline]
    fn from(value: &'a mut str) -> Self {
        unsafe { Self::String(BoxStr::new_unchecked(Box::from(value.as_bytes()))) }
    }
}

impl From<BoxStr> for Tag {
    #[inline]
    fn from(value: BoxStr) -> Self {
        Self::String(value)
    }
}

impl From<List> for Tag {
    #[inline]
    fn from(value: List) -> Self {
        Self::List(value)
    }
}

impl From<Compound> for Tag {
    #[inline]
    fn from(value: Compound) -> Self {
        Self::Compound(value)
    }
}

impl From<Vec<i32>> for Tag {
    #[inline]
    fn from(value: Vec<i32>) -> Self {
        Self::IntArray(value)
    }
}

impl From<Vec<i64>> for Tag {
    #[inline]
    fn from(value: Vec<i64>) -> Self {
        Self::LongArray(value)
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Compound<A: Allocator = Global>(Vec<(BoxStr<A>, Tag), A>);

impl AsRef<[(BoxStr, Tag)]> for Compound {
    #[inline]
    fn as_ref(&self) -> &[(BoxStr, Tag)] {
        self.0.as_slice()
    }
}

impl AsMut<[(BoxStr, Tag)]> for Compound {
    #[inline]
    fn as_mut(&mut self) -> &mut [(BoxStr, Tag)] {
        self.0.as_mut_slice()
    }
}

impl Write for Compound {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            for (name, tag) in &self.0 {
                tag.id().write(w);
                RefStringTag(name).write(w);
                match tag {
                    Tag::Byte(x) => x.write(w),
                    Tag::Short(x) => x.write(w),
                    Tag::Int(x) => x.write(w),
                    Tag::Long(x) => x.write(w),
                    Tag::Float(x) => x.write(w),
                    Tag::Double(x) => x.write(w),
                    Tag::String(x) => RefStringTag(x).write(w),
                    Tag::ByteArray(x) => {
                        (x.len() as u32).write(w);
                        w.write(&*(x.as_slice() as *const [i8] as *const [u8]))
                    }
                    Tag::IntArray(x) => {
                        (x.len() as u32).write(w);
                        x.iter().for_each(|&x| x.write(w));
                    }
                    Tag::LongArray(x) => {
                        (x.len() as u32).write(w);
                        x.iter().for_each(|&x| x.write(w));
                    }
                    Tag::List(x) => x.write(w),
                    Tag::Compound(x) => x.write(w),
                }
            }
            TagType::End.write(w);
        }
    }

    fn sz(&self) -> usize {
        let mut w = 1 + self.0.len();
        for (name, tag) in &self.0 {
            w += RefStringTag(name).sz();
            w += match tag {
                Tag::Byte(_) => 1,
                Tag::Short(_) => 2,
                Tag::Int(_) => 4,
                Tag::Long(_) => 8,
                Tag::Float(_) => 4,
                Tag::Double(_) => 8,
                Tag::String(x) => RefStringTag(x).sz(),
                Tag::ByteArray(x) => 4 + x.len(),
                Tag::IntArray(x) => 4 + x.len() * 4,
                Tag::LongArray(x) => 4 + x.len() * 8,
                Tag::List(x) => x.sz(),
                Tag::Compound(x) => Write::sz(x),
            };
        }
        w
    }
}

#[derive(Clone)]
pub struct NamedCompound<A: Allocator = Global>(pub BoxStr<A>, pub Compound<A>);

impl Read<'_> for NamedCompound {
    #[inline]
    fn read(n: &mut &[u8]) -> Result<Self, Error> {
        if matches!(TagType::read(n)?, TagType::Compound) {
            Ok(Self(StringTag::read(n)?.0, decode_raw(n)?))
        } else {
            Err(Error)
        }
    }
}

impl Write for NamedCompound {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            RefStringTag(self.0.as_str()).write(w);
            self.1.write(w);
        }
    }

    #[inline]
    fn sz(&self) -> usize {
        1 + Write::sz(&RefStringTag(self.0.as_str())) + Write::sz(&self.1)
    }
}

#[derive(Clone)]
pub struct UnamedCompound(pub Compound);

impl Read<'_> for UnamedCompound {
    #[inline]
    fn read(n: &mut &[u8]) -> Result<Self, Error> {
        if matches!(TagType::read(n)?, TagType::Compound) {
            Ok(Self(decode_raw(n)?))
        } else {
            Err(Error)
        }
    }
}

impl Write for UnamedCompound {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            TagType::Compound.write(w);
            self.0.write(w);
        }
    }

    #[inline]
    fn sz(&self) -> usize {
        1 + Write::sz(&self.0)
    }
}

impl From<Compound> for UnamedCompound {
    #[inline]
    fn from(value: Compound) -> Self {
        Self(value)
    }
}

impl<K: Into<BoxStr>, V> FromIterator<(K, V)> for Compound
where
    Tag: From<V>,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(x, y)| (x.into(), Tag::from(y)))
                .collect(),
        )
    }
}

impl Default for Compound {
    fn default() -> Self {
        Self::new()
    }
}

impl Compound {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, (BoxStr, Tag)> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, (BoxStr, Tag)> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn sort(&mut self) {
        self.0.sort_unstable_by(|x, y| (*x.0).cmp(&*y.0));
    }

    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<(&str, &Tag)> {
        #[allow(clippy::manual_map)]
        match self.0.get(index) {
            Some((x, y)) => Some((x, y)),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    /// # Safety
    ///
    /// `index` < [`len`]
    ///
    /// [`len`]: Self::len
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> (&str, &mut Tag) {
        let (x, y) = unsafe { self.0.get_unchecked_mut(index) };
        (x.as_str(), y)
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn push(&mut self, k: impl Into<BoxStr>, v: impl Into<Tag>) {
        self.push_(k.into(), v.into());
    }

    #[inline]
    fn push_(&mut self, k: BoxStr, v: Tag) {
        self.0.push((k, v));
    }

    #[deprecated]
    #[inline]
    pub fn decode(buf: &mut &[u8]) -> Result<Self, Error> {
        match UnamedCompound::read(buf) {
            Ok(x) => Ok(x.0),
            Err(e) => Err(e),
        }
    }

    #[deprecated]
    #[inline]
    pub fn decode_named(buf: &mut &[u8]) -> Result<(BoxStr, Self), Error> {
        match NamedCompound::read(buf) {
            Ok(x) => Ok((x.0, x.1)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    pub fn find(&self, name: &str) -> Option<&Tag> {
        for (x, y) in &self.0 {
            let x = &**x;
            if x == name {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    pub fn find_remove(&mut self, name: &str) -> Option<Tag> {
        for (i, (x, _)) in self.0.iter_mut().enumerate() {
            let x = &**x;
            if x == name {
                return Some(self.0.swap_remove(i).1);
            }
        }
        None
    }

    #[inline]
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Tag> {
        for (x, y) in &mut self.0 {
            let x = &**x;
            if x == name {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    pub fn into_inner(self) -> Vec<(BoxStr, Tag)> {
        self.0
    }
}

impl Read<'_> for Compound {
    #[inline]
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        decode_raw(buf)
    }
}

impl From<Vec<(BoxStr, Tag)>> for Compound {
    #[inline]
    fn from(value: Vec<(BoxStr, Tag)>) -> Self {
        Self(value)
    }
}

fn decode_raw(n: &mut &[u8]) -> Result<Compound, Error> {
    let mut compound = Compound(Default::default());
    loop {
        let ty = TagType::read(n)?;
        if mser::unlikely(matches!(ty, TagType::End)) {
            compound.0.shrink_to_fit();
            return Ok(compound);
        }
        let k = StringTag::read(n)?.0;
        let v = match ty {
            TagType::Byte => Tag::from(n.u8()?),
            TagType::Short => Tag::from(n.i16()?),
            TagType::Int => Tag::from(n.i32()?),
            TagType::Long => Tag::from(n.i64()?),
            TagType::Float => Tag::from(n.f32()?),
            TagType::Double => Tag::from(n.f64()?),
            TagType::ByteArray => {
                let len = n.i32()? as usize;
                let v = Vec::from(n.slice(len)?);
                Tag::from(v)
            }
            TagType::String => Tag::from(StringTag::read(n)?.0),
            TagType::List => {
                let info = ListInfo::read(n)?;
                Tag::from(list::decode_raw(n, info)?)
            }
            TagType::Compound => Tag::from(decode_raw(n)?),
            TagType::IntArray => {
                let len = n.i32()? as usize;
                let mut slice = n.slice(len * 4)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i32()?);
                }
                Tag::from(v)
            }
            TagType::LongArray => {
                let len = n.i32()? as usize;
                let mut slice = n.slice(len * 8)?;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(slice.i64()?);
                }
                Tag::from(v)
            }
            TagType::End => unsafe { core::hint::unreachable_unchecked() },
        };
        compound.push_(k, v);
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct StringTag<A: Allocator = Global>(pub BoxStr<A>);

impl Read<'_> for StringTag {
    #[inline]
    fn read(buf: &mut &'_ [u8]) -> Result<Self, Error> {
        let len = buf.u16()? as usize;
        let a = buf.slice(len)?;

        if is_ascii_mutf8(a) {
            unsafe { Ok(Self(BoxStr::new_unchecked(Box::from(a)))) }
        } else {
            let len = decode_mutf8_len(a)?;
            let mut x = Vec::with_capacity(len);
            unsafe {
                mser::write_unchecked(x.as_mut_ptr(), &(DecodeMutf8(a, len)));
                x.set_len(len);
                Ok(Self(BoxStr::new_unchecked(x.into_boxed_slice())))
            }
        }
    }
}

struct DecodeMutf8<'a>(&'a [u8], usize);

impl Write for DecodeMutf8<'_> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { decode_mutf8(self.0, w).unwrap_unchecked() }
    }

    fn sz(&self) -> usize {
        self.1
    }
}

impl Write for StringTag {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { RefStringTag(&self.0).write(w) }
    }

    #[inline]
    fn sz(&self) -> usize {
        RefStringTag(&self.0).sz()
    }
}
