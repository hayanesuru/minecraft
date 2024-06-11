pub fn read_block_state(
    n: &mser::nbt::Compound,
    buf: &mut [(block_state_property_key, block_state_property_value); 16],
) -> block_state {
    let block = match n.find("Name") {
        Some(mser::nbt::Tag::String(x)) if x.as_bytes().starts_with(b"minecraft:") => {
            match x.get(10..) {
                Some(x) => block::parse(x.as_bytes()).unwrap_or(block::air),
                None => block::air,
            }
        }
        _ => block::air,
    };
    if block.props().is_empty() {
        return block.state_default();
    }
    let props = match n.find("Properties") {
        Some(mser::nbt::Tag::Compound(x)) => x,
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
            mser::nbt::Tag::String(v) => block_state_property_value::parse(v.as_bytes()),
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
        unsafe {
            block::new(raw_block::from_le_bytes(
                *BLOCK_STATE_TO_BLOCK.add(self.0 as usize),
            ))
        }
    }
    #[inline]
    pub const fn to_fluid(self) -> fluid_state {
        unsafe {
            fluid_state::new(raw_fluid_state::from_le_bytes(
                *FLUID_STATE.add(self.0 as usize),
            ))
        }
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
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() >> 4 })
        }
    }
    #[inline]
    pub const fn solid(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 8 != 0 })
        }
    }
    #[inline]
    pub const fn transparent(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 4 != 0 })
        }
    }
    #[inline]
    pub const fn full_cube(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 2 != 0 })
        }
    }
    #[inline]
    pub const fn opaque_full_cube(self) -> Option<bool> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>() & 1 != 0 })
        }
    }
    #[inline]
    pub const fn side_solid_full(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(1) })
        }
    }
    #[inline]
    pub const fn side_solid_center(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(2) })
        }
    }
    #[inline]
    pub const fn side_solid_rigid(self) -> Option<u8> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            Some(unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1).cast::<u8>().add(3) })
        }
    }
    #[inline]
    pub const fn collision_shape(self) -> Option<&'static [[f64; 6]]> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            let index = unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1 + 4).cast::<[u8; 2]>() };
            let index = u16::from_le_bytes(index) as usize;
            Some(unsafe { *SHAPES.as_ptr().add(index) })
        }
    }
    #[inline]
    pub const fn culling_shape(self) -> Option<&'static [[f64; 6]]> {
        let n = self.0 as usize;
        let n = unsafe { u16::from_le_bytes(*BLOCK_STATE_BOUNDS_INDEX.add(n)) };
        if n == 0 {
            crate::cold__();
            None
        } else {
            let index = unsafe { *BLOCK_STATE_BOUNDS.add(n as usize - 1 + 6).cast::<[u8; 2]>() };
            let index = u16::from_le_bytes(index) as usize;
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
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr()
        }
    }
    #[inline]
    pub const fn blast_resistance(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(1)
        }
    }
    #[inline]
    pub const fn slipperiness(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(2)
        }
    }
    #[inline]
    pub const fn velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(3)
        }
    }
    #[inline]
    pub const fn jump_velocity_multiplier(self) -> f32 {
        unsafe {
            let x = *BLOCK_SETTINGS_INDEX.as_ptr().add(self as usize) as usize;
            *(*BLOCK_SETTINGS.as_ptr().add(x)).as_ptr().add(4)
        }
    }
}

impl fluid_state {
    #[inline]
    pub const fn to_block(self) -> block_state {
        unsafe {
            block_state::new(raw_block_state::from_le_bytes(
                *FLUID_STATE_TO_BLOCK
                    .add(self as usize * ::core::mem::size_of::<raw_block_state>()),
            ))
        }
    }

    #[inline]
    pub const fn level(self) -> u8 {
        unsafe { *(*FLUID_STATE_LEVEL.add(self as usize)).as_ptr() }
    }

    #[inline]
    pub const fn falling(self) -> bool {
        unsafe { *(*FLUID_STATE_FALLING.add(self as usize)).as_ptr() == 1 }
    }

    #[inline]
    pub const fn to_fluid(self) -> fluid {
        unsafe { fluid::new(*(*FLUID_STATE_TO_FLUID.add(self as usize)).as_ptr()) }
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
