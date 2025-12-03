#![no_std]
#![allow(non_camel_case_types, clippy::manual_map, non_upper_case_globals)]

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

#[derive(Copy, Clone)]
struct NameMap<T: 'static> {
    disps: &'static [(u32, u32)],
    names: *const u8,
    vals: &'static [T],
}

#[inline]
fn hash<const K: u128, const N: u32, const M: u32>(
    name: &[u8],
    disps: &'static [(u32, u32)],
) -> u32 {
    let x = name;
    let pl;
    let ptr;
    let x = match x.len() {
        len @ 0..=16 => {
            pl = len;
            ptr = x.as_ptr();
            0u128
        }
        len @ 17..=32 => unsafe {
            pl = len - 16;
            ptr = x.as_ptr().add(16);
            u128::from_le_bytes(x.get_unchecked(0..16).try_into().unwrap_unchecked())
        },
        len => unsafe {
            let n = u128::from_le_bytes(x.get_unchecked(0..16).try_into().unwrap_unchecked());
            let m = u128::from_le_bytes(x.get_unchecked(16..32).try_into().unwrap_unchecked());
            let len = len - 32;
            pl = if len <= 16 { len } else { 16 };
            ptr = x.as_ptr().add(32);
            n ^ m
        },
    };
    let x = unsafe {
        let mut p = [0u8; 16];
        core::ptr::copy_nonoverlapping(ptr, p.as_mut_ptr(), pl);
        x ^ u128::from_le_bytes(p)
    };
    let x = x ^ 0xdbe6d5d5fe4cce213198a2e03707344u128;
    let e = x.wrapping_mul(K);
    let [a, b] = [(e >> 64) as u64, e as u64];
    let g = (a >> 32) as u32;
    let f1 = a as u32;
    let f2 = b as u32;
    let (d1, d2) = unsafe { *disps.get_unchecked((g % N) as usize) };
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2) % M
}

impl NameMap<u16> {
    fn get<const K: u128, const N: u32, const M: u32>(&self, name: &[u8]) -> Option<u16> {
        let index = hash::<K, N, M>(name, self.disps);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let packed =
            unsafe { u64::from_le_bytes(*self.names.add(8 * v as usize).cast::<[u8; 8]>()) };
        let len = (packed >> 32) as usize;
        let offset = (packed as u32) as usize;
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}

impl NameMap<u8> {
    fn get<const K: u128, const N: u32, const M: u32>(&self, name: &[u8]) -> Option<u8> {
        let index = hash::<K, N, M>(name, self.disps);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let packed =
            unsafe { u64::from_le_bytes(*self.names.add(8 * v as usize).cast::<[u8; 8]>()) };
        let len = (packed >> 32) as usize;
        let offset = (packed as u32) as usize;
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}

fn make_block_state(
    buf: &mut [(block_state_property_key, block_state_property_value)],
    block: block,
) -> block_state {
    let def: raw_block_state = block.state_default().id() - block.state_index();
    let mut offset: raw_block_state = 0;
    let mut mul: raw_block_state = 1;
    let ptr = buf.as_mut_ptr();
    let mut len = buf.len();
    let props = block.props();
    let props_ptr = props.as_ptr();
    let mut props_end = unsafe { props_ptr.add(props.len()) };
    while props_end != props_ptr {
        props_end = unsafe { props_end.sub(1) };
        let prop = unsafe { *props_end };
        let key = prop.key();
        let vals = prop.val();
        let vals_len = vals.len() as raw_block_state;

        let mut buf_ptr = ptr;
        let buf_end = unsafe { ptr.add(len) };
        loop {
            if buf_ptr == buf_end {
                let val = (def / mul) % vals_len;
                offset += val * mul;
                mul *= vals_len;
                break;
            }
            if unsafe { ((*buf_ptr).0) != key } {
                buf_ptr = unsafe { buf_ptr.add(1) };
                continue;
            }
            let last = unsafe { ptr.add(len - 1) };
            if buf_ptr != last {
                unsafe { core::ptr::swap(buf_ptr, last) };
            }
            let value = unsafe { (*last).1 };
            len -= 1;
            let mut val_ptr = vals.as_ptr();
            let val_end = unsafe { val_ptr.add(vals.len()) };
            loop {
                if val_ptr == val_end {
                    let val = (def / mul) % vals_len;
                    offset += val * mul;
                    mul *= vals_len;
                    break;
                }
                if unsafe { (*val_ptr).id() != value.id() } {
                    val_ptr = unsafe { val_ptr.add(1) };
                    continue;
                }
                let val = unsafe { val_ptr.offset_from_unsigned(vals.as_ptr()) } as raw_block_state;
                offset += val * mul;
                mul *= vals_len;
                break;
            }
            break;
        }
    }

    block_state(block.state_index() + offset)
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
        let idx = raw % v.len() as raw_block_state;
        raw /= v.len() as raw_block_state;
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
            core::mem::transmute(raw_block::from_le_bytes(
                *BLOCK_STATE_TO_BLOCK.add(self.0 as usize),
            ))
        }
    }

    #[inline]
    pub const fn to_fluid(self) -> fluid_state {
        unsafe {
            // 1
            let b = self.to_block();
            // 2
            let i = *FLUID_STATE_INDEX.add(b as usize);
            // 3
            let a = *FLUID_STATE_ARRAY
                .as_ptr()
                .add(u8::from_le_bytes(i) as usize);
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
    pub const fn luminance(self) -> u8 {
        unsafe { *BLOCK_STATE_LUMINANCE.add(self.0 as usize) }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "useShapeForLightOcclusion")]
    pub const fn has_sided_transparency(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 128 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "ignitedByLava")]
    pub const fn lava_ignitable(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 64 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "canBeReplaced")]
    pub const fn material_replaceable(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 32 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "canOcclude")]
    pub const fn opaque(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 16 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "requiresCorrectToolForDrops")]
    pub const fn tool_required(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 8 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "hasLargeCollisionShape")]
    pub const fn exceeds_cube(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 4 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isSignalSource")]
    pub const fn redstone_power_source(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 2 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "hasAnalogOutputSignal")]
    pub const fn has_comparator_output(self) -> bool {
        let x = unsafe { *BLOCK_STATE_FLAGS.add(self.0 as usize) };
        x & 1 != 0
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getLightBlock")]
    pub const fn opacity(self) -> Option<u8> {
        unsafe {
            // 1
            let b = self.to_block();
            let index = b.id() as usize;
            // 2
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            // 3
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                // 4
                *bounds.as_ptr()
            } else {
                // 4
                let offset = b.state_index();
                // 5
                *bounds.as_ptr().add((self.id() - offset) as _)
            };
            // 5 / 6
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8) >> 4)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isSolidRender")]
    pub const fn solid(self) -> Option<bool> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8) & 1 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isCollisionShapeFullBlock")]
    pub const fn full_cube(self) -> Option<bool> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8) & 2 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "propagatesSkylightDown")]
    pub const fn transparent(self) -> Option<bool> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8) & 4 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isRedstoneConductor")]
    pub const fn opaque_full_cube(self) -> Option<bool> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8) & 8 != 0)
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isFaceSturdyFull")]
    pub const fn side_solid_full(self) -> Option<u8> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8 + 1))
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isFaceSturdyCenter")]
    pub const fn side_solid_center(self) -> Option<u8> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8 + 2))
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "isFaceSturdyRigid")]
    pub const fn side_solid_rigid(self) -> Option<u8> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            Some(*BLOCK_STATE_BOUNDS.add(index as usize * 8 + 3))
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getCollisionShape")]
    pub const fn collision_shape(self) -> Option<&'static [[f64; 6]]> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            let index = *BLOCK_STATE_BOUNDS
                .add(index as usize * 8 + 4)
                .cast::<[u8; 2]>();
            let index = u16::from_le_bytes(index) as usize;
            Some(*SHAPES.as_ptr().add(index))
        }
    }

    #[inline]
    #[must_use]
    #[doc(alias = "getOcclusionShape")]
    pub const fn culling_shape(self) -> Option<&'static [[f64; 6]]> {
        unsafe {
            let b = self.to_block();
            let index = b.id() as usize;
            let index = u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(index));
            if ::mser::unlikely(index == 0) {
                return None;
            }
            let offset = self.id() - b.state_index();
            let bounds = *BLOCK_BOUNDS.as_ptr().add(index as usize);
            let index = if ::mser::unlikely(bounds.len() == 1) {
                *bounds.as_ptr()
            } else {
                *bounds.as_ptr().add(offset as _)
            };
            let index = *BLOCK_STATE_BOUNDS
                .add(index as usize * 8 + 6)
                .cast::<[u8; 2]>();
            let index = u16::from_le_bytes(index) as usize;
            Some(*SHAPES.as_ptr().add(index))
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
    #[must_use]
    #[doc(alias = "defaultDestroyTime", alias = "getDestroySpeed")]
    pub const fn hardness(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr()
        }
    }
    #[inline]
    #[must_use]
    #[doc(alias = "getExplosionResistance")]
    pub const fn blast_resistance(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(1)
        }
    }
    #[inline]
    #[must_use]
    #[doc(alias = "getFriction")]
    pub const fn slipperiness(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(2)
        }
    }
    #[inline]
    #[must_use]
    #[doc(alias = "getSpeedFactor")]
    pub const fn velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(3)
        }
    }
    #[inline]
    #[must_use]
    #[doc(alias = "getJumpFactor")]
    pub const fn jump_velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(4)
        }
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
            *Self::PROPS
                .as_ptr()
                .add(i as usize)
                .cast::<&'static [block_state_property]>()
        }
    }
}

impl fluid_state {
    #[inline]
    pub const fn to_block(self) -> block_state {
        unsafe {
            block_state(raw_block_state::from_le_bytes(
                *FLUID_STATE_TO_BLOCK
                    .add(self.0 as usize * ::core::mem::size_of::<raw_block_state>()),
            ))
        }
    }

    #[inline]
    #[must_use]
    pub const fn level(self) -> u8 {
        unsafe { *(*FLUID_STATE_LEVEL.add(self.0 as usize)).as_ptr() }
    }

    #[inline]
    #[must_use]
    pub const fn falling(self) -> bool {
        unsafe { *(*FLUID_STATE_FALLING.add(self.0 as usize)).as_ptr() == 1 }
    }

    #[inline]
    pub const fn to_fluid(self) -> fluid {
        unsafe { core::mem::transmute(*(*FLUID_STATE_TO_FLUID.add(self.0 as usize)).as_ptr()) }
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
        unsafe { *ENTITY_FIXED.as_ptr().add(self as usize) }
    }
}

impl core::fmt::Debug for block_state_property {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(self.key().name()).field(&self.val()).finish()
    }
}

#[test]
fn test_block_state() {
    assert_eq!(game_event::block_activate.name(), "block_activate");
    assert_eq!(
        sound_event::block_bamboo_wood_pressure_plate_click_on,
        sound_event::parse(b"block.bamboo_wood_pressure_plate.click_on").unwrap()
    );

    let x = encode_state!(white_concrete(white_concrete::new()));
    assert_eq!(x.side_solid_full(), Some(0b111111));
    assert_eq!(x.side_solid_rigid(), Some(0b111111));
    assert_eq!(x.side_solid_center(), Some(0b111111));
    assert_eq!(x.full_cube(), Some(true));
    assert_eq!(x.solid(), Some(true));
    assert!(x.tool_required());

    let b = x.to_block();
    assert_eq!(b.name(), "white_concrete");
    assert_eq!(Some(b), block::parse(b"white_concrete"));

    assert_eq!(block::air.name(), "air");
    assert_eq!(Some(block::air), block::parse(b"air"));

    let x = block::air.state_default();
    assert_eq!(x.side_solid_full(), Some(0));
    assert_eq!(x.side_solid_rigid(), Some(0));
    assert_eq!(x.side_solid_center(), Some(0));
    assert_eq!(x.full_cube(), Some(false));

    let x = encode_state!(oak_sapling(oak_sapling::new()));
    let b = x.to_block();
    assert_eq!(b.name(), "oak_sapling");
    assert_eq!(Some(b), block::parse(b"oak_sapling"));

    assert_eq!(x.side_solid_full(), Some(0));
    assert_eq!(x.side_solid_rigid(), Some(0));
    assert_eq!(x.side_solid_center(), Some(0));
    assert_eq!(x.full_cube(), Some(false));

    let x = block::mud.state_default();
    let b = x.to_block();
    assert_eq!(b.name(), "mud");
    assert_eq!(Some(b), block::parse(b"mud"));

    assert_eq!(x.side_solid_full(), Some(0b111111));
    assert_eq!(x.side_solid_rigid(), Some(0b111111));
    assert_eq!(x.side_solid_center(), Some(0b111111));
    assert!(!x.redstone_power_source());
    assert_eq!(x.opaque_full_cube(), Some(true));
    assert_eq!(x.full_cube(), Some(false));

    let x = block::cactus.state_default();
    assert_eq!(x.side_solid_full(), Some(0b000000));
    assert_eq!(x.side_solid_rigid(), Some(0b000000));
    assert_eq!(x.side_solid_center(), Some(0b000001));
    assert_eq!(x.full_cube(), Some(false));
    assert!(!x.exceeds_cube());
    assert!(!x.tool_required());
    assert_eq!(x.full_cube(), Some(false));
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
                .with_east(val_up_side_none::side)
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
                .with_east(prop_east_up_side_none::side)
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

impl core::fmt::Debug for fluid_state {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(self.to_fluid().name())
            .field(&self.id())
            .finish()
    }
}
