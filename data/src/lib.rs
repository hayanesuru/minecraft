#![no_std]
#![allow(non_camel_case_types, clippy::manual_map, non_upper_case_globals)]

use core::hint::assert_unchecked;
use mser::{cold_path, hash128};

include!(concat!(env!("OUT_DIR"), "/data.rs"));

/// `block_name(prop_expr)`
#[macro_export]
macro_rules! encode_state {
    ($b:ident($x:expr)) => {
        $crate::block_state::new(
            $x.encode() as $crate::raw_block_state + $crate::block::$b.state_index(),
        )
        .unwrap()
    };
}

/// `block_name(prop_expr)`
#[macro_export]
macro_rules! decode_state {
    ($b:ident($x:expr)) => {
        $crate::$b::decode(($x.id() - $crate::block::$b.state_index()) as _)
    };
}

#[inline]
fn name_u8<const K: u64, const N: usize, const M: usize>(
    disps: [u64; N],
    names: *const u8,
    vals: [u8; M],
    name: &[u8],
) -> Option<u8> {
    let [a, b] = hash128(name, K);
    let g = (a >> 32) as u32;
    let f1 = a as u32;
    let f2 = b as u32;
    let d = unsafe { *disps.get_unchecked((g % (N as u32)) as usize) };
    let d1 = (d >> 32) as u32;
    let d2 = d as u32;
    let index = d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2) % (M as u32);
    let v = unsafe { *vals.get_unchecked(index as usize) };
    let packed = unsafe { u64::from_le_bytes(*names.add(8 * v as usize).cast::<[u8; 8]>()) };
    let len = (packed >> 32) as usize;
    let offset = (packed as u32) as usize;
    let k = unsafe { core::slice::from_raw_parts(names.add(offset), len) };
    if name == k { Some(v) } else { None }
}

#[inline]
fn name_u16<const K: u64, const N: usize, const M: usize>(
    disps: [u64; N],
    names: *const u8,
    vals: [u16; M],
    name: &[u8],
) -> Option<u16> {
    let [a, b] = hash128(name, K);
    let g = (a >> 32) as u32;
    let f1 = a as u32;
    let f2 = b as u32;
    let d = unsafe { *disps.get_unchecked((g % (N as u32)) as usize) };
    let d1 = (d >> 32) as u32;
    let d2 = d as u32;
    let index = d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2);
    let index = (index % (M as u32)) as usize;
    let v = unsafe { *vals.get_unchecked(index) };
    let packed = unsafe { u64::from_le_bytes(*names.add(8 * v as usize).cast::<[u8; 8]>()) };
    let len = (packed >> 32) as usize;
    let offset = (packed as u32) as usize;
    let k = unsafe { core::slice::from_raw_parts(names.add(offset), len) };
    if name == k { Some(v) } else { None }
}

fn make_block_state(
    buf: &mut [(block_state_property_key, block_state_property_value)],
    block: block,
) -> block_state {
    let def: raw_block_state = block.state_default().id() - block.state_index();
    let mut offset: raw_block_state = 0;
    let mut mul: raw_block_state = 1;
    let b_ptr = buf.as_mut_ptr();
    let mut b_len = buf.len();
    let props = block.props();
    let mut index = props.len();
    while index > 0 {
        index -= 1;
        let prop = props[index];
        let key = prop.key();
        let vals = prop.val();
        let vals_len = vals.len() as raw_block_state;

        let mut buf_ptr = b_ptr;
        let buf_end = unsafe { b_ptr.add(b_len) };
        loop {
            unsafe {
                assert_unchecked(mul > 0);
                assert_unchecked(vals_len > 0);
            }
            if buf_ptr == buf_end {
                let val = (def / mul) % vals_len;
                offset += val * mul;
                mul *= vals_len;
                break;
            }
            unsafe {
                if ((*buf_ptr).0) != key {
                    buf_ptr = buf_ptr.add(1);
                    continue;
                }
                let last = { b_ptr.add(b_len - 1) };
                if buf_ptr != last {
                    core::ptr::swap(buf_ptr, last);
                }
                let value = { (*last).1 };
                b_len -= 1;
                let mut val_ptr = vals.as_ptr();
                let val_end = val_ptr.add(vals.len());
                loop {
                    if (*val_ptr).id() != value.id() {
                        val_ptr = val_ptr.add(1);
                        if val_ptr != val_end {
                            continue;
                        } else {
                            offset += ((def / mul) % vals_len) * mul;
                            mul *= vals_len;
                            break;
                        }
                    } else {
                        let val = val_ptr.offset_from_unsigned(vals.as_ptr()) as raw_block_state;
                        offset += val * mul;
                        mul *= vals_len;
                        break;
                    }
                }
            }
            break;
        }
    }

    let ret = block.state_index() + offset;
    debug_assert!(ret <= block_state::MAX);
    block_state(ret)
}

pub fn block_state_props(
    state: block_state,
    buf: &mut [(block_state_property_key, block_state_property_value); 16],
) -> &[(block_state_property_key, block_state_property_value)] {
    let mut iter = buf.iter_mut();
    let kind = state.to_block();
    let mut raw = state.id() - kind.state_index();
    for prop in kind.props().iter().rev() {
        let v = prop.val();
        let l = v.len() as raw_block_state;
        let idx = raw % l;
        raw /= l;
        match iter.next_back() {
            Some(x) => *x = (prop.key(), unsafe { *v.get_unchecked(idx as usize) }),
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
        unsafe {
            core::mem::transmute::<raw_block, block>(
                *BLOCK_STATE_TO_BLOCK.as_ptr().add(self.0 as usize),
            )
        }
    }

    #[inline]
    pub const fn to_fluid(self) -> fluid_state {
        unsafe {
            // 1
            let b = self.to_block();
            // 2
            let i = *FLUID_STATE_INDEX.as_ptr().add(b as usize);
            // 3
            let a = *FLUID_STATE_ARRAY.as_ptr().add(i as usize);
            if a.len() == 1 {
                // 4
                fluid_state(*a.as_ptr())
            } else {
                // 4
                let o = b.state_index();
                // 5
                fluid_state(*a.as_ptr().add((self.id() - o) as usize))
            }
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getLightEmission")]
    #[cfg(not(debug_assertions))]
    pub const fn luminance(self) -> u8 {
        unsafe { *BLOCK_STATE_LUMINANCE.as_ptr().add(self.0 as usize) }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getLightEmission")]
    #[cfg(debug_assertions)]
    pub const fn luminance(self) -> u8 {
        BLOCK_STATE_LUMINANCE[self.0 as usize]
    }

    #[inline]
    #[must_use]
    #[doc(alias = "useShapeForLightOcclusion")]
    pub const fn has_sided_transparency(self) -> bool {
        self.static_flags() & 128 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "ignitedByLava")]
    pub const fn lava_ignitable(self) -> bool {
        self.static_flags() & 64 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "canBeReplaced")]
    pub const fn material_replaceable(self) -> bool {
        self.static_flags() & 32 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "canOcclude")]
    pub const fn opaque(self) -> bool {
        self.static_flags() & 16 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "requiresCorrectToolForDrops")]
    pub const fn tool_required(self) -> bool {
        self.static_flags() & 8 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "hasLargeCollisionShape")]
    pub const fn exceeds_cube(self) -> bool {
        self.static_flags() & 4 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isSignalSource")]
    pub const fn redstone_power_source(self) -> bool {
        self.static_flags() & 2 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "hasAnalogOutputSignal")]
    pub const fn has_comparator_output(self) -> bool {
        self.static_flags() & 1 != 0
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    const fn static_flags(self) -> u8 {
        unsafe { *BLOCK_STATE_FLAGS.as_ptr().add(self.id() as usize) }
    }

    #[inline]
    #[cfg(debug_assertions)]
    const fn static_flags(self) -> u8 {
        BLOCK_STATE_FLAGS[self.id() as usize]
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    const fn static_bounds(self) -> u64 {
        unsafe {
            *BLOCK_STATE_BOUNDS
                .as_ptr()
                .add(*BLOCK_STATE_BOUNDS_INDEX.as_ptr().add(self.id() as usize) as usize)
        }
    }

    #[inline]
    #[cfg(debug_assertions)]
    const fn static_bounds(self) -> u64 {
        BLOCK_STATE_BOUNDS[BLOCK_STATE_BOUNDS_INDEX[self.id() as usize] as usize]
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getLightBlock")]
    pub const fn opacity(self) -> Option<u8> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some(x as u8 >> 4)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isSolidRender")]
    pub const fn solid(self) -> Option<bool> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some(x & 1 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isCollisionShapeFullBlock")]
    pub const fn full_cube(self) -> Option<bool> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some(x & 2 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "propagatesSkylightDown")]
    pub const fn transparent(self) -> Option<bool> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some(x & 4 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isRedstoneConductor")]
    pub const fn opaque_full_cube(self) -> Option<bool> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some(x & 8 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isFaceSturdyFull")]
    pub const fn side_solid_full(self) -> Option<u8> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some((x >> 8) as u8)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isFaceSturdyCenter")]
    pub const fn side_solid_center(self) -> Option<u8> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some((x >> 16) as u8)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isFaceSturdyRigid")]
    pub const fn side_solid_rigid(self) -> Option<u8> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            Some((x >> 24) as u8)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getCollisionShape")]
    pub const fn collision_shape(self) -> Option<&'static [[f64; 6]]> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            unsafe { Some(*SHAPES.as_ptr().add((x >> 32) as u16 as usize)) }
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getOcclusionShape")]
    pub const fn culling_shape(self) -> Option<&'static [[f64; 6]]> {
        let x = self.static_bounds();
        if x == 0 {
            cold_path();
            None
        } else {
            unsafe { Some(*SHAPES.as_ptr().add((x >> 48) as u16 as usize)) }
        }
    }

    #[inline]
    pub fn parse(
        block: block,
        buf: &mut [(block_state_property_key, block_state_property_value)],
    ) -> Self {
        make_block_state(buf, block)
    }
}

impl item {
    #[inline]
    #[must_use]
    #[cfg(debug_assertions)]
    pub const fn max_count(self) -> u8 {
        ITEM_MAX_COUNT[self as usize]
    }

    #[inline]
    #[must_use]
    #[cfg(not(debug_assertions))]
    pub const fn max_count(self) -> u8 {
        unsafe { *ITEM_MAX_COUNT.as_ptr().add(self as usize) }
    }

    #[inline]
    pub const fn to_block(self) -> block {
        unsafe { core::mem::transmute::<raw_block, block>(*ITEM.as_ptr().add(self as usize)) }
    }
}

impl block {
    #[inline]
    #[cfg(debug_assertions)]
    const fn settings(self) -> &'static [f32; 5] {
        &BLOCK_SETTINGS[BLOCK_SETTINGS_INDEX[self as usize] as usize]
    }

    #[inline]
    #[cfg(not(debug_assertions))]
    const fn settings(self) -> &'static [f32; 5] {
        unsafe {
            &*BLOCK_SETTINGS
                .as_ptr()
                .add(*BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "defaultDestroyTime", alias = "getDestroySpeed")]
    pub const fn hardness(self) -> f32 {
        unsafe { *self.settings().as_ptr() }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getExplosionResistance")]
    pub const fn blast_resistance(self) -> f32 {
        unsafe { *self.settings().as_ptr().add(1) }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getFriction")]
    pub const fn slipperiness(self) -> f32 {
        unsafe { *self.settings().as_ptr().add(2) }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getSpeedFactor")]
    pub const fn velocity_multiplier(self) -> f32 {
        unsafe { *self.settings().as_ptr().add(3) }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getJumpFactor")]
    pub const fn jump_velocity_multiplier(self) -> f32 {
        unsafe { *self.settings().as_ptr().add(4) }
    }

    #[inline]
    #[must_use]
    pub const fn state_index(self) -> u16 {
        unsafe { *Self::OFFSET.as_ptr().add(self as usize) }
    }

    #[inline]
    pub const fn props(self) -> &'static [block_state_property] {
        let i = unsafe { *Self::PROPS_INDEX.as_ptr().add(self as usize) };
        unsafe {
            core::mem::transmute::<&[u8], &[block_state_property]>(
                *Self::PROPS.as_ptr().add(i as usize),
            )
        }
    }
}

impl fluid_state {
    #[inline]
    pub const fn to_block(self) -> block_state {
        unsafe { block_state(*FLUID_STATE_TO_BLOCK.as_ptr().add(self.0 as usize)) }
    }

    #[inline]
    #[must_use]
    pub const fn level(self) -> u8 {
        unsafe { *FLUID_STATE_LEVEL.as_ptr().add(self.0 as usize) }
    }

    #[inline]
    #[must_use]
    pub const fn falling(self) -> bool {
        unsafe { *FLUID_STATE_FALLING.as_ptr().add(self.0 as usize) == 1 }
    }

    #[inline]
    pub const fn to_fluid(self) -> fluid {
        unsafe {
            core::mem::transmute::<raw_fluid, fluid>(
                *FLUID_STATE_TO_FLUID.as_ptr().add(self.0 as usize),
            )
        }
    }
}

impl From<bool> for val_bool {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Self::r#true } else { Self::r#false }
    }
}

impl From<val_bool> for bool {
    #[inline]
    fn from(value: val_bool) -> Self {
        match value {
            val_bool::r#true => true,
            val_bool::r#false => false,
        }
    }
}

impl entity_type {
    #[inline]
    #[must_use]
    pub const fn width(self) -> f32 {
        unsafe { *ENTITY_WIDTH.as_ptr().add(self as usize) }
    }

    #[inline]
    #[must_use]
    pub const fn height(self) -> f32 {
        unsafe { *ENTITY_HEIGHT.as_ptr().add(self as usize) }
    }

    #[inline]
    #[must_use]
    pub const fn fixed(self) -> bool {
        unsafe { *ENTITY_FIXED.as_ptr().add(self as usize) == 1 }
    }
}

impl core::fmt::Debug for block_state_property {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(self.key().name()).field(&self.val()).finish()
    }
}

impl core::fmt::Debug for fluid_state {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(self.to_fluid().name())
            .field(&self.id())
            .finish()
    }
}

#[test]
fn test_block_state() {
    assert!(
        block_state::new(block_state::MAX)
            .unwrap()
            .collision_shape()
            .is_some()
    );

    assert_eq!(game_event::block_activate.name(), "block_activate");
    assert_eq!(
        sound_event::block_bamboo_wood_pressure_plate_click_on,
        sound_event::parse(b"block.bamboo_wood_pressure_plate.click_on").unwrap()
    );

    let white_concrete_bs = encode_state!(white_concrete(white_concrete::new()));
    assert_eq!(white_concrete_bs.side_solid_full(), Some(0b111111));
    assert_eq!(white_concrete_bs.side_solid_rigid(), Some(0b111111));
    assert_eq!(white_concrete_bs.side_solid_center(), Some(0b111111));
    assert_eq!(white_concrete_bs.full_cube(), Some(true));
    assert_eq!(white_concrete_bs.solid(), Some(true));
    assert!(white_concrete_bs.tool_required());

    let b = white_concrete_bs.to_block();
    assert_eq!(b.name(), "white_concrete");
    assert_eq!(Some(b), block::parse(b.name().as_bytes()));

    let air_bl = block::air;
    assert_eq!(air_bl.name(), "air");
    assert_eq!(Some(air_bl), block::parse(air_bl.name().as_bytes()));

    let air_bs = air_bl.state_default();
    assert_eq!(air_bs.side_solid_full(), Some(0));
    assert_eq!(air_bs.side_solid_rigid(), Some(0));
    assert_eq!(air_bs.side_solid_center(), Some(0));
    assert_eq!(air_bs.full_cube(), Some(false));

    let oak_sapling_bs = encode_state!(oak_sapling(oak_sapling::new()));
    let b = oak_sapling_bs.to_block();
    assert_eq!(b.name(), "oak_sapling");
    assert_eq!(Some(b), block::parse(b.name().as_bytes()));

    assert_eq!(oak_sapling_bs.side_solid_full(), Some(0));
    assert_eq!(oak_sapling_bs.side_solid_rigid(), Some(0));
    assert_eq!(oak_sapling_bs.side_solid_center(), Some(0));
    assert_eq!(oak_sapling_bs.full_cube(), Some(false));

    let mud_bs = block::mud.state_default();
    let b = mud_bs.to_block();
    assert_eq!(b.name(), "mud");
    assert_eq!(Some(b), block::parse(b"mud"));

    assert_eq!(mud_bs.side_solid_full(), Some(0b111111));
    assert_eq!(mud_bs.side_solid_rigid(), Some(0b111111));
    assert_eq!(mud_bs.side_solid_center(), Some(0b111111));
    assert!(!mud_bs.redstone_power_source());
    assert_eq!(mud_bs.opaque_full_cube(), Some(true));
    assert_eq!(mud_bs.full_cube(), Some(false));

    let cactus_bs = block::cactus.state_default();
    assert_eq!(cactus_bs.side_solid_full(), Some(0b000000));
    assert_eq!(cactus_bs.side_solid_rigid(), Some(0b000000));
    assert_eq!(cactus_bs.side_solid_center(), Some(0b000001));
    assert_eq!(cactus_bs.full_cube(), Some(false));
    assert!(!cactus_bs.exceeds_cube());
    assert!(!cactus_bs.tool_required());
    assert_eq!(cactus_bs.full_cube(), Some(false));
    assert_eq!(
        block_state::parse(block::redstone_wire, &mut [][..]),
        block::redstone_wire.state_default()
    );
    assert_eq!(
        block_state::parse(
            block::redstone_wire,
            &mut [(
                block_state_property_key::parse(b"east").unwrap(),
                block_state_property_value::parse(b"side").unwrap()
            )][..]
        ),
        encode_state!(redstone_wire(
            decode_state!(redstone_wire(block::redstone_wire.state_default()))
                .with_east(prop_east_u_s_n::side)
        ))
    );
    assert_eq!(
        block_state::parse(
            block::redstone_wire,
            &mut [
                (
                    block_state_property_key::parse(b"east").unwrap(),
                    block_state_property_value::parse(b"side").unwrap()
                ),
                (
                    block_state_property_key::parse(b"power").unwrap(),
                    block_state_property_value::parse(b"11").unwrap()
                )
            ][..]
        ),
        encode_state!(redstone_wire(
            decode_state!(redstone_wire(block::redstone_wire.state_default()))
                .with_east(prop_east_u_s_n::side)
                .with_power(prop_power::d_11)
        ))
    );
    assert_eq!(
        block::spruce_slab.state_default().to_fluid(),
        fluid_state::empty
    );
    assert_eq!(
        encode_state!(spruce_slab(
            decode_state!(spruce_slab(block::spruce_slab.state_default()))
                .with_waterlogged(prop_waterlogged::r#true)
        ))
        .to_fluid(),
        fluid_state::water_s_8
    );
    assert!(!block::dispenser.is_air());
    assert!(block::dispenser.state_default().opaque_full_cube().unwrap());
    assert_eq!(block::fire.state_default().opacity().unwrap(), 0);
}
