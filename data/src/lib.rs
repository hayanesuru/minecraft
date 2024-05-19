#![no_std]
#![allow(non_camel_case_types)]

use mser::nbt::{Compound, Tag};

#[cfg(feature = "packet")]
pub mod packet;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

/// # Example
///
/// ```
/// # use minecraft_data::{oak_log, prop_axis_x_y_z, encode_state};
/// let x = oak_log::new();
/// let x = x.with_axis(prop_axis_x_y_z::y);
/// let x = encode_state!(oak_log(x));
/// let id = x.id();
/// ```
#[macro_export]
macro_rules! encode_state {
    ($b:ident($x:expr)) => {
        $crate::block_state::new(
            $x.encode() as $crate::raw_block_state + $crate::block::$b.state_index(),
        )
    };
}

/// # Example
///
/// ```
/// # use minecraft_data::*;
///
/// let x = oak_log::new();
/// let x = x.with_axis(prop_axis_x_y_z::y);
/// let x = encode_state!(oak_log(x));
/// let id = x.id();
/// let x = block_state::new(id);
/// let x = decode_state!(oak_log(x));
/// ```
#[macro_export]
macro_rules! decode_state {
    ($b:ident($x:expr)) => {
        $crate::$b::decode((($x.id() - $crate::block::$b.state_index()) as _))
    };
}

#[cold]
#[inline(always)]
const fn cold__() {}

#[derive(Copy, Clone)]
struct NameMap<T: 'static> {
    key: [u64; 4],
    disps: &'static [(u32, u32)],
    names: *const u8,
    vals: &'static [T],
}

fn hash(key: [u64; 4], name: &[u8], disps: &'static [(u32, u32)], len: u32) -> u32 {
    let hasher = highway::HighwayHasher::new(highway::Key(key));
    let [a, b] = highway::HighwayHash::hash128(hasher, name);
    let g = (a >> 32) as u32;
    let f1 = a as u32;
    let f2 = b as u32;
    let (d1, d2) = disps[(g % (disps.len() as u32)) as usize];
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2) % len
}

impl NameMap<u16> {
    fn get(&self, name: &[u8]) -> Option<u16> {
        let index = hash(self.key, name, self.disps, self.vals.len() as u32);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let offset =
            unsafe { u32::from_ne_bytes(*self.names.add(4 * v as usize).cast::<[u8; 4]>()) };
        let len = unsafe {
            u16::from_ne_bytes(*self.names.add(offset as usize).cast::<[u8; 2]>()) as usize
        };
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset as usize + 2), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}

impl NameMap<u8> {
    fn get(&self, name: &[u8]) -> Option<u8> {
        let index = hash(self.key, name, self.disps, self.vals.len() as u32);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let offset =
            unsafe { u32::from_ne_bytes(*self.names.add(4 * v as usize).cast::<[u8; 4]>()) };
        let len = unsafe {
            u16::from_ne_bytes(*self.names.add(offset as usize).cast::<[u8; 2]>()) as usize
        };
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset as usize + 2), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}

pub fn read_block_state(
    n: &Compound,
    buf: &mut [(block_state_property_key, block_state_property_value); 16],
) -> block_state {
    let block = match n.find("Name") {
        Some(Tag::String(x)) if x.as_bytes().starts_with(b"minecraft:") => match x.get(10..) {
            Some(x) => block::parse(x.as_bytes()).unwrap_or(block::air),
            None => block::air,
        },
        _ => block::air,
    };
    if block.props().is_empty() {
        return block.state_default();
    }
    let props = match n.find("Properties") {
        Some(Tag::Compound(x)) => x,
        _ => return block.state_default(),
    };
    let mut len = 0;
    for (k, v) in props.iter() {
        let k = block_state_property_key::parse(k.as_bytes());
        let k = match k {
            Some(x) => x,
            None => continue,
        };
        let v = match v {
            Tag::String(v) => block_state_property_value::parse(v.as_bytes()),
            _ => None,
        };
        let v = match v {
            Some(val) => val,
            None => continue,
        };
        buf[len] = (k, v);
        len += 1;
    }
    make_block_state(unsafe { buf.get_unchecked_mut(0..len) }, block)
}

pub fn make_block_state(
    mut buf: &mut [(block_state_property_key, block_state_property_value)],
    block: block,
) -> block_state {
    let mut offset = 0_u16;
    let mut index = 0_u16;

    for &prop in block.props().iter().rev() {
        let key = prop.key();
        let vals = prop.val();

        let val = buf.iter().position(|&(x, _)| x == key);
        let val = match val {
            Some(x) => unsafe {
                let y = buf.len() - 1;
                let x = if x != y {
                    let y = *buf.get_unchecked_mut(y);
                    let x = buf.get_unchecked_mut(x);
                    Some(core::mem::replace(x, y))
                } else {
                    let x = buf.get_unchecked_mut(x);
                    Some(*x)
                };
                buf = buf.get_unchecked_mut(0..buf.len() - 1);
                x
            },
            None => None,
        };
        let val = match val {
            Some((_, val)) => match vals.iter().position(|&v| v == val) {
                None => 0,
                Some(x) => x as u16,
            },
            None => {
                let def = block.state_default().id() - block.state_index();
                let x = if index == 0 { def } else { def / index };
                x % vals.len() as u16
            }
        };
        if index == 0 {
            offset = val;
            index = vals.len() as u16;
        } else {
            offset += val * index;
            index *= vals.len() as u16;
        }
    }
    block_state::new(block.state_index() + offset)
}

pub fn block_state_props(
    state: block_state,
    buf: &mut [(block_state_property_key, block_state_property_value)],
) -> &[(block_state_property_key, block_state_property_value)] {
    let mut iter = buf.iter_mut();
    let kind = state.to_block();
    let mut raw = state.id() - kind.state_index();
    for prop in kind.props().iter().rev() {
        let v = prop.val();
        let idx = raw % v.len() as u16;
        raw /= v.len() as u16;
        match iter.next_back() {
            Some(x) => {
                *x = (prop.key(), unsafe { *v.get_unchecked(idx as usize) });
                continue;
            }
            None => break,
        }
    }
    let rest = iter.into_slice().len();
    unsafe { buf.get_unchecked(rest..) }
}

impl core::fmt::Debug for block_state {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut s = f.debug_struct(self.to_block().name());
        let mut prop_buf = [(
            block_state_property_key::age,
            block_state_property_value::d_0,
        ); 16];
        for (k, v) in block_state_props(*self, &mut prop_buf) {
            s.field(k.name(), v);
        }
        s.finish()
    }
}

impl core::fmt::Display for block_state {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}

impl block_state {
    #[inline]
    pub const fn to_block(self) -> block {
        unsafe { ::core::mem::transmute(*BLOCK_STATE_TO_BLOCK.add(self.0 as usize)) }
    }
    #[inline]
    pub const fn to_fluid(self) -> fluid_state {
        unsafe { ::core::mem::transmute(*FLUID_STATE.add(self.0 as usize)) }
    }
    #[inline]
    pub const fn luminance(self) -> u8 {
        unsafe { *BLOCK_STATE_SETTINGS.add(self.0 as usize).cast::<u8>() }
    }
    #[inline]
    pub const fn has_sided_transparency(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 128 != 0
    }
    #[inline]
    pub const fn lava_ignitable(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 64 != 0
    }
    #[inline]
    pub const fn material_replaceable(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 32 != 0
    }
    #[inline]
    pub const fn opaque(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 16 != 0
    }
    #[inline]
    pub const fn tool_required(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 8 != 0
    }
    #[inline]
    pub const fn exceeds_cube(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 4 != 0
    }
    #[inline]
    pub const fn redstone_power_source(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 2 != 0
    }
    #[inline]
    pub const fn has_comparator_output(self) -> bool {
        let x = unsafe {
            *BLOCK_STATE_SETTINGS
                .add(self.0 as usize)
                .cast::<u8>()
                .add(1)
        };
        x & 1 != 0
    }
    #[inline]
    pub const fn opacity(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() >> 4 })
        }
    }
    #[inline]
    pub const fn solid(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 8 != 0 })
        }
    }
    #[inline]
    pub const fn translucent(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 4 != 0 })
        }
    }
    #[inline]
    pub const fn full_cube(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 2 != 0 })
        }
    }
    #[inline]
    pub const fn opaque_full_cube(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 1 != 0 })
        }
    }
    #[inline]
    pub const fn side_solid_full(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(1) })
        }
    }
    #[inline]
    pub const fn side_solid_center(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(2) })
        }
    }
    #[inline]
    pub const fn side_solid_rigid(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(3) })
        }
    }
    #[inline]
    pub const fn collision_shape(self) -> Option<&'static [[f64; 6]]> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            let index = unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1 + 4).cast::<[u8; 2]>() };
            let index = u16::from_ne_bytes(index) as usize;
            Some(unsafe { *SHAPES.as_ptr().add(index) })
        }
    }
    #[inline]
    pub const fn culling_shape(self) -> Option<&'static [[f64; 6]]> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_ne_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            cold__();
            None
        } else {
            let index = unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1 + 6).cast::<[u8; 2]>() };
            let index = u16::from_ne_bytes(index) as usize;
            Some(unsafe { *SHAPES.as_ptr().add(index) })
        }
    }
}

impl item {
    #[inline]
    pub const fn max_count(self) -> u8 {
        unsafe { *ITEM_MAX_COUNT.as_ptr().add(self as usize) }
    }

    #[inline]
    pub const fn to_block(self) -> block {
        unsafe { block::new(*ITEM.as_ptr().add(self as usize)) }
    }
}

impl block {
    #[inline]
    pub const fn hardness(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>()
        }
    }
    #[inline]
    pub const fn blast_resistance(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(1)
        }
    }
    #[inline]
    pub const fn slipperiness(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(2)
        }
    }
    #[inline]
    pub const fn velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(3)
        }
    }
    #[inline]
    pub const fn jump_velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *BLOCK_SETTINGS.as_ptr().add(x).cast::<f32>().add(4)
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum fluid_state {
    empty,
    flowing_water_falling_1,
    flowing_water_falling_2,
    flowing_water_falling_3,
    flowing_water_falling_4,
    flowing_water_falling_5,
    flowing_water_falling_6,
    flowing_water_falling_7,
    flowing_water_falling_8,
    flowing_water_1,
    flowing_water_2,
    flowing_water_3,
    flowing_water_4,
    flowing_water_5,
    flowing_water_6,
    flowing_water_7,
    flowing_water_8,
    water_falling,
    water,
    flowing_lava_falling_1,
    flowing_lava_falling_2,
    flowing_lava_falling_3,
    flowing_lava_falling_4,
    flowing_lava_falling_5,
    flowing_lava_falling_6,
    flowing_lava_falling_7,
    flowing_lava_falling_8,
    flowing_lava_1,
    flowing_lava_2,
    flowing_lava_3,
    flowing_lava_4,
    flowing_lava_5,
    flowing_lava_6,
    flowing_lava_7,
    flowing_lava_8,
    lava_falling,
    lava,
}

impl fluid_state {
    #[inline]
    pub const fn to_fluid(self) -> fluid {
        match self as u8 {
            0 => fluid::empty,
            1..=16 => fluid::flowing_water,
            17..=18 => fluid::water,
            19..=34 => fluid::flowing_lava,
            _ => fluid::lava,
        }
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        matches!(self as u8, 0)
    }

    #[inline]
    pub const fn is_flowing_water(self) -> bool {
        matches!(self as u8, 1..=16)
    }

    #[inline]
    pub const fn is_water(self) -> bool {
        matches!(self as u8, 17..=18)
    }

    #[inline]
    pub const fn is_flowing_lava(self) -> bool {
        matches!(self as u8, 19..=34)
    }

    #[inline]
    pub const fn is_lava(self) -> bool {
        matches!(self as u8, 35..=36)
    }
}

impl From<bool> for val_true_false {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            Self::r#true
        } else {
            Self::r#false
        }
    }
}

impl From<val_true_false> for bool {
    #[inline]
    fn from(value: val_true_false) -> Self {
        match value {
            val_true_false::r#true => true,
            val_true_false::r#false => false,
        }
    }
}

#[test]
fn test() {
    assert_eq!(block::white_concrete.name(), "white_concrete");
    assert_eq!(Some(block::white_concrete), block::parse(b"white_concrete"));

    let x = block::white_concrete.state_default();
    assert_eq!(x.side_solid_full(), Some(0b111111));
    assert_eq!(x.side_solid_rigid(), Some(0b111111));
    assert_eq!(x.side_solid_center(), Some(0b111111));
    let x = block::torch.state_default();
    assert_eq!(x.side_solid_full(), Some(0b000000));
    assert_eq!(x.side_solid_center(), Some(0b000000));
    assert_eq!(x.side_solid_rigid(), Some(0b000000));
}
