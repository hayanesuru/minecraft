#![no_std]
#![allow(non_camel_case_types, clippy::manual_map, non_upper_case_globals)]

use core::hint::assert_unchecked;
use mser::cold_path;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

#[inline]
fn name_u8<const K: u64, const N: usize, const M: usize>(
    disps: &'static [u64; N],
    names: *const &'static str,
    vals: &'static [u8; M],
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
    let k = unsafe { *names.add(v as usize) };
    if name == k.as_bytes() { Some(v) } else { None }
}

#[inline]
fn name_u16<const K: u64, const N: usize, const M: usize>(
    disps: &'static [u64; N],
    names: *const &'static str,
    vals: &'static [u16; M],
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
    let k = unsafe { *names.add(v as usize) };
    if name == k.as_bytes() { Some(v) } else { None }
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
        unsafe { fluid_state(*FLUID_STATE_INDEX.as_ptr().add(self.0 as usize)) }
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

const fn hash128(n: &[u8], seed: u64) -> [u64; 2] {
    const M: u64 = 0xc6a4a7935bd1e995;
    const N: u128 = 0xdbe6d5d5fe4cce213198a2e03707344u128;
    let mut h: u64 = seed ^ ((n.len() as u64).wrapping_mul(M));
    let mut i = 0;
    while i + 8 <= n.len() {
        h ^= u64::from_le_bytes(unsafe { *(n.as_ptr().add(i) as *const [u8; 8]) }).wrapping_mul(M);
        i += 8;
    }
    while i < n.len() {
        h ^= (unsafe { *n.as_ptr().add(i) } as u64) << ((i & 7) * 8);
        i += 1;
    }
    let h = (h as u128).wrapping_mul(N);
    let h = h ^ (h >> 64);
    let h = h.wrapping_mul(N);
    [(h >> 64) as u64, h as u64]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(game_event::block_activate.name(), "block_activate");
        assert_eq!(
            sound_event::block_bamboo_wood_pressure_plate_click_on,
            sound_event::parse(b"block.bamboo_wood_pressure_plate.click_on").unwrap()
        );
    }

    #[test]
    fn test_air() {
        let air_bl = block::air;
        assert_eq!(air_bl.name(), "air");
        assert_eq!(Some(air_bl), block::parse(air_bl.name().as_bytes()));

        let air_bs = air_bl.state_default();
        assert_eq!(air_bs.side_solid_full(), Some(0));
        assert_eq!(air_bs.side_solid_rigid(), Some(0));
        assert_eq!(air_bs.side_solid_center(), Some(0));
        assert_eq!(air_bs.full_cube(), Some(false));
    }

    #[test]
    fn test_block_state() {
        assert!(
            block::firefly_bush
                .state_default()
                .collision_shape()
                .is_some()
        );

        let white_concrete_bs = block_state::new(
            (white_concrete::new()).encode() as raw_block_state
                + block::white_concrete.state_index(),
        )
        .unwrap();
        assert_eq!(white_concrete_bs.side_solid_full(), Some(0b111111));
        assert_eq!(white_concrete_bs.side_solid_rigid(), Some(0b111111));
        assert_eq!(white_concrete_bs.side_solid_center(), Some(0b111111));
        assert_eq!(white_concrete_bs.full_cube(), Some(true));
        assert_eq!(white_concrete_bs.solid(), Some(true));
        assert!(white_concrete_bs.tool_required());

        let b = white_concrete_bs.to_block();
        assert_eq!(b.name(), "white_concrete");
        assert_eq!(Some(b), block::parse(b.name().as_bytes()));

        let oak_sapling_bs = block_state::new(
            (oak_sapling::new()).encode() as raw_block_state + block::oak_sapling.state_index(),
        )
        .unwrap();
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
        let a = block_state::new(
            (redstone_wire::decode(
                ((block::redstone_wire.state_default()).id() - block::redstone_wire.state_index())
                    as _,
            )
            .with_east(prop_east_u_s_n::side))
            .encode() as raw_block_state
                + block::redstone_wire.state_index(),
        )
        .unwrap();
        assert_eq!(
            block_state::parse(
                block::redstone_wire,
                &mut [(
                    block_state_property_key::parse(b"east").unwrap(),
                    block_state_property_value::parse(b"side").unwrap()
                )][..]
            ),
            a
        );
        let a = block_state::new(
            (redstone_wire::decode(
                ((block::redstone_wire.state_default()).id() - block::redstone_wire.state_index())
                    as _,
            )
            .with_east(prop_east_u_s_n::side)
            .with_power(prop_power::d_11))
            .encode() as raw_block_state
                + block::redstone_wire.state_index(),
        )
        .unwrap();
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
            a
        );
        assert_eq!(
            block::spruce_slab.state_default().to_fluid(),
            fluid_state::empty
        );
        let a = block_state::new(
            (spruce_slab::decode(
                ((block::spruce_slab.state_default()).id() - block::spruce_slab.state_index()) as _,
            )
            .with_waterlogged(prop_waterlogged::r#true))
            .encode() as raw_block_state
                + block::spruce_slab.state_index(),
        )
        .unwrap();
        assert_eq!(a.to_fluid(), fluid_state::water_s_8);
        assert!(!block::dispenser.is_air());
        assert!(block::dispenser.state_default().opaque_full_cube().unwrap());
        assert_eq!(block::fire.state_default().opacity().unwrap(), 0);
    }
}
