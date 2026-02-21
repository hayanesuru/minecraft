#![no_std]

use alloc::boxed::Box;
use alloc::vec::Vec;
use haya_ident::Ident;
use haya_nbt::Tag;
use mser::{Error, Read, UnsafeWriter, V21, V32, Write};

// pub mod chat;
pub mod clientbound;
// pub mod dialog;
pub mod item;
pub mod profile;
pub mod serverbound;
pub mod stat;
pub mod types;

#[macro_use]
extern crate mser_macro;
extern crate alloc;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ClientIntent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

impl Write for ClientIntent {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write_byte(*self as u8);
        }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> Read<'a> for ClientIntent {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        match V32::read(buf)?.0 {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(Error),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Utf8<'a, const MAX: usize = 32767>(pub &'a str);

impl<'a, const MAX: usize> Write for Utf8<'a, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            V21(self.0.len() as u32).write(w);
            w.write(self.0.as_bytes());
        }
    }

    fn len_s(&self) -> usize {
        V21(self.0.len() as u32).len_s() + self.0.len()
    }
}

impl<'a, const MAX: usize> Read<'a> for Utf8<'a, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX * 3 {
            return Err(Error);
        }
        let bytes = match buf.split_at_checked(len) {
            Some((x, y)) => {
                *buf = y;
                x
            }
            None => return Err(Error),
        };
        let s = match core::str::from_utf8(bytes) {
            Ok(x) => x,
            Err(_) => return Err(Error),
        };
        if s.chars().map(|x| x.len_utf16()).sum::<usize>() <= MAX {
            Ok(Utf8(s))
        } else {
            Err(Error)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Map<'a, K: 'a, V: 'a, const MAX: usize = { usize::MAX }>(pub List<'a, (K, V), MAX>);

impl<'a, K: Write + 'a, V: Write + 'a, const MAX: usize> Write for Map<'a, K, V, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let x = match &self.0 {
                List::Borrowed(x) => x,
                List::Owned(x) => &x[..],
            };
            V21(x.len() as u32).write(w);
            for y in x {
                y.0.write(w);
                y.1.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let x = match &self.0 {
            List::Borrowed(x) => x,
            List::Owned(x) => &x[..],
        };
        let mut len = V21(x.len() as u32).len_s();
        for y in x {
            len += y.0.len_s();
            len += y.1.len_s();
        }
        len
    }
}

impl<'a, K: Read<'a>, V: Read<'a>, const MAX: usize> Read<'a> for Map<'a, K, V, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        let mut vec = Vec::with_capacity(usize::min(len, 65536));
        for _ in 0..len {
            let k = K::read(buf)?;
            let v = V::read(buf)?;
            vec.push((k, v));
        }
        Ok(Self(List::Owned(vec.into_boxed_slice())))
    }
}

#[derive(Clone, Debug)]
pub enum List<'a, T: 'a, const MAX: usize = { usize::MAX }> {
    Borrowed(&'a [T]),
    Owned(Box<[T]>),
}

impl<'a, T: Write + 'a, const MAX: usize> Write for List<'a, T, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let x = match self {
                Self::Borrowed(x) => x,
                Self::Owned(x) => &x[..],
            };
            V21(x.len() as u32).write(w);
            for y in x {
                y.write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let x = match self {
            Self::Borrowed(x) => x,
            Self::Owned(x) => &x[..],
        };
        let mut len = V21(x.len() as u32).len_s();
        for y in x {
            len += y.len_s();
        }
        len
    }
}

impl<'a, T: Read<'a>, const MAX: usize> Read<'a> for List<'a, T, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        if len > MAX {
            return Err(Error);
        }
        let mut vec = Vec::with_capacity(usize::min(len, 65536));
        for _ in 0..len {
            vec.push(T::read(buf)?);
        }
        Ok(Self::Owned(vec.into_boxed_slice()))
    }
}

#[derive(Clone, Copy)]
pub struct Rest<'a, const MAX: usize = { usize::MAX }>(pub &'a [u8]);

impl<'a, const MAX: usize> Read<'a> for Rest<'a, MAX> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let len = buf.len();
        if len > MAX {
            return Err(Error);
        }
        match buf.split_at_checked(len) {
            Some((x, y)) => {
                *buf = y;
                Ok(Self(x))
            }
            None => Err(Error),
        }
    }
}

impl<'a, const MAX: usize> Write for Rest<'a, MAX> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { w.write(self.0) }
    }

    fn len_s(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComponentJson<'a>(pub Utf8<'a, 262144>);

#[derive(Clone, Serialize, Deserialize)]
pub struct Component(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct Dialog(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct RegistryKey<'a> {
    pub identifier: Ident<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KnownPack<'a> {
    pub namespace: Utf8<'a>,
    pub id: Utf8<'a>,
    pub version: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerLinkUntrustedEntry<'a> {
    pub ty: Either<KnownLinkType, Component>,
    pub url: Utf8<'a>,
}

#[derive(Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<'a, L: Read<'a>, R: Read<'a>> Read<'a> for Either<L, R> {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if bool::read(buf)? {
            Ok(Self::Left(L::read(buf)?))
        } else {
            Ok(Self::Right(R::read(buf)?))
        }
    }
}

impl<L: Write, R: Write> Write for Either<L, R> {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            match self {
                Self::Left(l) => {
                    true.write(w);
                    l.write(w);
                }
                Self::Right(r) => {
                    false.write(w);
                    r.write(w);
                }
            }
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Left(l) => true.len_s() + l.len_s(),
            Self::Right(r) => false.len_s() + r.len_s(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum KnownLinkType {
    ReportBug,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

impl KnownLinkType {
    pub const fn key(self) -> &'static str {
        match self {
            Self::ReportBug => "known_server_link.report_bug",
            Self::CommunityGuidelines => "known_server_link.community_guidelines",
            Self::Support => "known_server_link.support",
            Self::Status => "known_server_link.status",
            Self::Feedback => "known_server_link.feedback",
            Self::Community => "known_server_link.community",
            Self::Website => "known_server_link.website",
            Self::Forums => "known_server_link.forums",
            Self::News => "known_server_link.news",
            Self::Announcements => "known_server_link.announcements",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientInformation<'a> {
    pub language: Utf8<'a, 16>,
    pub view_distance: u8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: HumanoidArm,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Full => "options.chat.visibility.full",
            Self::System => "options.chat.visibility.system",
            Self::Hidden => "options.chat.visibility.hidden",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum HumanoidArm {
    Left,
    Right,
}

impl HumanoidArm {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Left => "options.mainHand.left",
            Self::Right => "options.mainHand.right",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

impl ParticleStatus {
    pub const fn key(self) -> &'static str {
        match self {
            Self::All => "options.particles.all",
            Self::Decreased => "options.particles.decreased",
            Self::Minimal => "options.particles.minimal",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Peaceful => "options.difficulty.peaceful",
            Self::Easy => "options.difficulty.easy",
            Self::Normal => "options.difficulty.normal",
            Self::Hard => "options.difficulty.hard",
        }
    }

    pub const fn parse(n: &[u8]) -> Option<Self> {
        match n {
            b"peaceful" => Some(Self::Peaceful),
            b"easy" => Some(Self::Easy),
            b"normal" => Some(Self::Normal),
            b"hard" => Some(Self::Hard),
            _ => None,
        }
    }
}
pub fn json_escaped_string(s: &str, w: &mut Vec<u8>) {
    let mut start = 0;
    let mut cur = 0;
    let n = s.as_bytes();

    while let Some(&byte) = n.get(cur) {
        let esc = mser::json_char_width_escaped(byte);
        if esc <= 4 {
            cur += esc as usize;
            continue;
        }
        w.extend(unsafe { n.get_unchecked(start..cur) });
        if esc == 0xff {
            let (d1, d2) = mser::u8_to_hex(byte);
            w.extend(&[b'\\', b'u', b'0', b'0', d1, d2]);
        } else {
            w.extend(&[b'\\', esc]);
        }
        cur += 1;
        start = cur;
    }
    w.extend(unsafe { n.get_unchecked(start..) });
}

#[test]
fn test_write() {
    use crate::clientbound::login::LoginFinished;
    use crate::profile::GameProfileRef;
    use crate::types::Id as _;
    use minecraft_data::clientbound__login;
    use uuid::Uuid;

    let packet: LoginFinished = LoginFinished {
        game_profile: GameProfileRef {
            id: Uuid::nil(),
            name: Utf8("abc"),
            peoperties: List::Borrowed(&[]),
        },
    };

    let id = LoginFinished::ID;
    let len1 = id.len_s();
    let len2 = packet.len_s() + len1;
    let data = unsafe {
        let mut data = alloc::vec::Vec::with_capacity(len2);
        mser::write_unchecked(data.as_mut_ptr(), &id);
        mser::write_unchecked(data.as_mut_ptr().add(len1), &packet);
        data.set_len(len2);
        data.into_boxed_slice()
    };
    let mut data = &data[..];
    let id = V32::read(&mut data).unwrap().0;
    assert_eq!(clientbound__login::new(id as _).unwrap(), LoginFinished::ID);
    assert_eq!(Uuid::read(&mut data).unwrap(), Uuid::nil());
    assert_eq!(Utf8::<16>::read(&mut data).unwrap().0, "abc");
    assert_eq!(V32::read(&mut data).unwrap().0, 0);
    assert!(data.is_empty());
}
