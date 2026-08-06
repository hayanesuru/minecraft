#![warn(clippy::shadow_reuse, clippy::use_self)]

use core::iter::repeat_n;
use core::num::NonZeroUsize;
use nested::ZString;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::env::var_os;
use std::path::PathBuf;

const V21MAX: usize = 0x1FFFFF;
const V7MAX: usize = 0x7F;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
enum Repr {
    U64,
    U32,
    U16,
    U8,
}

impl Repr {
    const fn new(size: usize) -> Self {
        if size > u32::MAX as usize {
            unreachable!()
        } else if size > u16::MAX as usize {
            Self::U32
        } else if size > u8::MAX as usize {
            Self::U16
        } else {
            Self::U8
        }
    }
    #[inline]
    #[must_use]
    const fn to_int(self) -> &'static str {
        match self {
            Self::U64 => "u64",
            Self::U32 => "u32",
            Self::U16 => "u16",
            Self::U8 => "u8",
        }
    }
}

fn read(buf: &mut Vec<u8>, path: PathBuf) -> usize {
    match buf.last().copied() {
        Some(b'\n') => (),
        Some(_) => buf.push(b'\n'),
        _ => (),
    }
    let mut file = std::fs::File::open(path).unwrap();
    let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
    buf.try_reserve_exact(size).unwrap();
    std::io::Read::read_to_end(&mut file, buf).unwrap()
}

fn main() {
    let out = PathBuf::from(var_os("OUT_DIR").unwrap());
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("generated");
    let mut w = String::with_capacity(0x80000);
    let mut data = Vec::with_capacity(0x40000);
    let mut gen_hash = GenerateHash::new();

    read(&mut data, path.join("version.txt"));
    version(&mut w, core::str::from_utf8(&data).unwrap());
    data.clear();

    let reg_len = read(&mut data, path.join("registries.txt"));
    let reg = data.len() - reg_len..data.len();

    let flu_len = read(&mut data, path.join("fluid_state.txt"));
    let flu = data.len() - flu_len..data.len();

    let blo_len = read(&mut data, path.join("block_state.txt"));
    let blo = data.len() - blo_len..data.len();

    let ite_len = read(&mut data, path.join("item.txt"));
    let ite = data.len() - ite_len..data.len();

    let ent_len = read(&mut data, path.join("entity.txt"));
    let ent = data.len() - ent_len..data.len();

    let pac_len = read(&mut data, path.join("packet.txt"));
    let pac = data.len() - pac_len..data.len();

    let s = core::str::from_utf8(&data).unwrap();
    let block_names = registries(&mut w, &s[reg], &mut gen_hash);
    registries(&mut w, &s[pac], &mut gen_hash);

    item(&mut w, &s[ite]);
    entity(&mut w, &s[ent]);

    let (bs_repr, bl_props, bs_size) = block_state(&mut w, &s[blo], &mut gen_hash, &block_names);
    fluid_state(&mut w, &s[flu], bs_repr, &bl_props, &bs_size);

    std::fs::write(out.join("data.rs"), w).unwrap();
}

fn version(w: &mut String, data: &str) {
    let mut iter = data.split('\n');
    let name = iter.next().unwrap();
    let proto = iter.next().unwrap();

    *w += "pub const NAME_VERSION: &str = \"";
    *w += name;
    *w += "\";\n";
    *w += "pub const PROTOCOL_VERSION: u32 = 0x";
    *w += proto;
    *w += ";\n";
}

fn registries<'a>(w: &mut String, data: &'a str, gen_hash: &mut GenerateHash) -> Vec<&'a str> {
    let mut zhash = Vec::<&str>::new();
    let mut iter = data.split('\n');
    let mut block_names = Vec::<&str>::new();

    while let Some(x) = iter.next() {
        if x.is_empty() {
            break;
        }
        let (name, size, repr) = head(Some(x), "");
        zhash.clear();
        zhash.reserve(size);
        for _ in 0..size {
            let data = iter.next().unwrap();
            zhash.push(data);
        }
        if name == "block" {
            block_names.clone_from(&zhash);
        }
        let name2 = name.replace('/', "_");
        enum_head(w, repr, &name2);

        for &location in &zhash {
            kw_prefix(w, location);
            *w += ",\n";
        }

        *w += "}\n";
        impl_codec(w, &name2, size, repr);
        impl_name(w, gen_hash, repr, &zhash, &name2);
        impl_common(w, &name2, repr, size, 0);
    }
    block_names
}

fn kw_prefix(w: &mut String, s: &str) {
    if s.chars().next().unwrap().is_ascii_digit() {
        *w += "d_";
        *w += s;
    } else if let "match" | "true" | "false" | "type" = s {
        *w += "r#";
        *w += s;
    } else {
        let mut last_end = 0;
        for (start, part) in s.match_indices(['.', '/']) {
            *w += unsafe { s.get_unchecked(last_end..start) };
            w.push('_');
            last_end = start + part.len();
        }
        *w += unsafe { s.get_unchecked(last_end..s.len()) };
    }
}

fn impl_common(w: &mut String, name: &str, repr: Repr, size: usize, def: u32) {
    if !name.starts_with('_') {
        *w += "pub type raw_";
        *w += name;
        *w += " = ";
        *w += repr.to_int();
        *w += ";\n";
    }

    *w += "impl ";
    *w += name;
    *w += " {\n";
    *w += "pub const MAX: ";
    *w += repr.to_int();
    *w += " = ";
    write(w, size - 1);
    *w += ";\n";

    *w += "#[inline]\n#[must_use]\n";
    *w += "pub const fn new(n: ";
    *w += repr.to_int();
    *w += ") -> ::core::option::Option<Self> {\n";
    if size == 1 {
        *w += "if n == Self::MAX {\n";
    } else {
        *w += "if n <= Self::MAX {\n";
    }
    *w += "unsafe {\ncore::option::Option::Some(::core::mem::transmute::<";
    *w += repr.to_int();
    *w += ", Self>(n))";
    *w += "\n}\n";
    *w += "} else {\n";
    *w += "::mser::cold_path();\nNone\n";
    *w += "}\n";
    *w += "}\n";

    *w += "#[inline]\n#[must_use]\n";
    *w += "pub const fn id(self) -> ";
    *w += repr.to_int();
    *w += " {\n";
    *w += "unsafe { ::core::mem::transmute::<Self, ";
    *w += repr.to_int();
    *w += ">(self) }";
    *w += "\n}\n";
    *w += "}\n";

    *w += "impl Default for ";
    *w += name;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "fn default() -> Self {\n";
    *w += "unsafe { ::core::mem::transmute::<";
    *w += repr.to_int();
    *w += ", Self>(";
    write(w, def);
    *w += ") }\n";
    *w += "}\n";
    *w += "}\n";
}

fn fluid_state(
    w: &mut String,
    data: &str,
    bs_repr: Repr,
    bl_props: &[u32],
    bs_size: &[NonZeroUsize],
) {
    let mut iter = data.split('\n');
    let (name, size, repr) = head(iter.next(), "fluid_state");
    struct_head(w, repr, name);
    impl_common(w, name, repr, size, 0);
    *w += "impl ";
    *w += name;
    *w += " {\n";
    for (index, name) in (&mut iter).take(size).enumerate() {
        *w += "pub const ";
        *w += name;
        *w += ": Self = Self(";
        write(w, index);
        *w += ");\n";
    }
    *w += "}\n";
    let (_, size, _) = head(iter.next(), "fluid_to_block");
    list_ty(w, "FLUID_STATE_TO_BLOCK", bs_repr, size);
    list(w, (&mut iter).take(size).map(parse_u32));
    *w += ";\n";
    let (_, size, _) = head(iter.next(), "fluid_state_level");
    list_ty(w, "FLUID_STATE_LEVEL", Repr::U8, size);
    list(w, (&mut iter).take(size).map(parse_u32));
    *w += ";\n";

    let (_, size, _) = head(iter.next(), "fluid_state_falling");
    list_ty(w, "FLUID_STATE_FALLING", Repr::U8, size);
    list(w, (&mut iter).take(size).map(parse_u32));
    *w += ";\n";

    let (_, size, _) = head(iter.next(), "fluid_state_to_fluid");
    list_ty(w, "FLUID_STATE_TO_FLUID", Repr::U8, size);
    list(w, (&mut iter).take(size).map(parse_u32));
    *w += ";\n";

    let (_, fs_size, fs_repr) = head(iter.next(), "fluid_state_array");

    let arr = (&mut iter)
        .take(fs_size)
        .map(|arr| hex_line(arr).collect::<Box<_>>())
        .collect::<Box<_>>();
    let (_, size, _) = head(iter.next(), "block_to_fluid_state");

    let mut out = Vec::<u32>::new();
    assert_eq!(size, bl_props.len());
    for (bound, prop) in read_rl(size, &mut iter).zip(bl_props.iter().copied()) {
        let bounds = &*arr[bound as usize];
        let len = bs_size[prop as usize].get();
        match bounds[..] {
            [] => {
                out.extend(repeat_n(0, len));
            }
            [b] => {
                out.extend(repeat_n(b, len));
            }
            ref bounds2 => {
                assert_eq!(len, bounds2.len());
                out.extend(bounds2.iter().copied());
            }
        }
    }
    list_ty(w, "FLUID_STATE_INDEX", fs_repr, out.len());
    list(w, out.into_iter());
    *w += ";\n";
}

fn block_state(
    w: &mut String,
    data: &str,
    gen_hash: &mut GenerateHash,
    block_names: &[&str],
) -> (Repr, Vec<u32>, Vec<NonZeroUsize>) {
    let mut iter = data.split('\n');

    let (name_k, size_k, repr_k) = head(iter.next(), "block_state_property_key");

    let mut pk1 = Vec::with_capacity(size_k);
    let mut pk2 = vec![""; size_k];
    let mut pk3 = vec![0_usize; size_k];
    for index in 0..size_k {
        let data = iter.next().unwrap();
        pk1.push((data, index));
    }
    pk1.sort_unstable_by(|x, y| x.0.cmp(y.0));
    for (sorted, &(value, before)) in pk1.iter().enumerate() {
        pk2[sorted] = value;
        pk3[before] = sorted;
    }

    let (name_v, size_v, repr_v) = head(iter.next(), "block_state_property_value");

    let mut pv1 = Vec::with_capacity(size_v);
    let mut pv2 = vec![""; size_v];
    let mut pv3 = vec![0_usize; size_v];
    for index in 0..size_v {
        let data = iter.next().unwrap();
        pv1.push((data, index));
    }
    pv1.sort_unstable_by(|x, y| x.0.cmp(y.0));
    for (sorted, &(value, before)) in pv1.iter().enumerate() {
        pv2[sorted] = value;
        pv3[before] = sorted;
    }

    let (name_kv, size_kv, repr_kv) = head(iter.next(), "block_state_property");
    assert!(Repr::U8 == repr_kv);

    let kv = (0..size_kv)
        .map(|_| {
            let mut line = hex_line(iter.next().unwrap());
            let k = line.next().unwrap();
            let mut vec = Vec::with_capacity(4);
            vec.push(pk3[k as usize] as u32);
            vec.extend(line.map(|x| pv3[x as usize] as u32));
            vec.into_boxed_slice()
        })
        .collect::<Vec<_>>();

    enum_head(w, repr_k, name_k);
    for &s in &pk2 {
        kw_prefix(w, s);
        *w += ",\n";
    }
    *w += "}\n";

    enum_head(w, repr_v, name_v);
    for &val in &pv2 {
        kw_prefix(w, val);
        *w += ",\n";
    }
    *w += "}\n";

    impl_common(w, name_k, repr_k, size_k, 0);
    impl_common(w, name_v, repr_v, size_v, 0);

    let mut kvn = ZString::with_capacity(kv.len(), kv.len() * 4);
    let mut x = HashMap::<Box<[&str]>, usize>::new();
    for arr in &kv {
        let key = arr[1..].iter().map(|&x| pv2[x as usize]).collect();
        let len = x.len();

        match x.entry(key) {
            Entry::Occupied(_) => {}
            Entry::Vacant(y) => {
                y.insert(len);
            }
        }
    }
    let mut vals = x.iter().map(|(k, v)| (v, k)).collect::<Vec<_>>();
    vals.sort_unstable_by(|(x, _), (y, _)| (*x).cmp(*y));
    let mut val_names = ZString::with_capacity(vals.len(), vals.len() * 4);
    let mut name_ = "val_".to_owned();
    for (_, names) in vals {
        name_.truncate(4);
        let is_digit = names
            .iter()
            .all(|x| x.as_bytes().iter().all(|y| y.is_ascii_digit()));
        if is_digit {
            name_.push_str(names[0]);
            name_.push('_');
            name_.push_str(names[names.len() - 1]);
        } else {
            if let ["true", "false"] = names[..] {
                name_.push_str("bool");
            } else {
                let mut first = true;
                for &n in &**names {
                    if first {
                        first = false;
                    } else {
                        name_.push('_');
                    }
                    name_.extend(n.split('_').map(|x| x.chars().next().unwrap()));
                }
            }
        }

        let name = name_.as_str();
        let repr = Repr::new(names.len() - 1);
        enum_head(w, repr, name);
        for &n in &**names {
            kw_prefix(w, n);
            *w += ",\n";
        }
        *w += "}\n";
        impl_common(w, name, repr, names.len(), 0);
        impl_name(w, gen_hash, repr, names, name);

        val_names.push(name);
    }
    let mut xn = HashMap::<&str, (usize, bool)>::new();
    let mut x2 = Vec::new();
    for arr in &kv {
        x2.clear();
        x2.extend(arr[1..].iter().map(|&x| pv2[x as usize]));
        let idx = x.get(&*x2).copied().unwrap();
        match xn.entry(pk2[arr[0] as usize]) {
            Entry::Vacant(x) => {
                x.insert((idx, false));
            }
            Entry::Occupied(x) => {
                let y = x.into_mut();
                if y.0 != idx {
                    y.1 = true;
                }
            }
        }
    }
    name_.clear();
    name_ += "prop_";

    for arr in &kv {
        let dupe = xn.get(pk2[arr[0] as usize]).unwrap().1;
        name_.truncate(5);
        name_ += pk2[arr[0] as usize];
        if dupe {
            name_.push('_');
            let is_digit = arr[1..]
                .iter()
                .map(|&x| pv2[x as usize])
                .all(|x| x.as_bytes().iter().all(|y| y.is_ascii_digit()));
            if is_digit {
                name_.push_str(pv2[arr[1] as usize]);
                name_.push('_');
                name_.push_str(pv2[arr[arr.len() - 1] as usize]);
            } else {
                let mut first = true;
                for n in arr[1..].iter().map(|&x| pv2[x as usize]) {
                    if first {
                        first = false;
                    } else {
                        name_.push('_');
                    }
                    name_.extend(n.split('_').map(|x| x.chars().next().unwrap()));
                }
            }
        }
        kvn.push(name_.as_str());
    }
    let mut tmp = Vec::new();
    for (index, props) in kv.iter().enumerate() {
        tmp.clear();
        tmp.extend(props[1..].iter().map(|&x| pv2[x as usize]));
        let key = &val_names[*x.get(&*tmp).unwrap()];
        *w += "pub use ";
        *w += key;
        *w += " as ";
        *w += &kvn[index];
        *w += ";\n";
    }
    enum_head(w, repr_kv, name_kv);
    for name in kvn.iter() {
        *w += &name[5..];
        *w += ",\n";
    }
    *w += "}\n";
    impl_common(w, name_kv, repr_kv, size_kv, 0);

    impl_name(w, gen_hash, repr_k, &pk2, name_k);
    impl_name(w, gen_hash, repr_v, &pv2, name_v);

    *w += "impl ";
    *w += name_kv;
    *w += " {\n";

    list_ty(w, "K", repr_k, size_kv);
    list(w, kv.iter().map(|x| x[0]));
    *w += ";\n";

    *w += "const V: &[&'static [";
    *w += repr_v.to_int();
    *w += "]; ";
    write(w, size_kv);
    *w += "] = &[";
    for data in &kv {
        *w += "&[";
        for &v in &data[1..] {
            write(w, v as usize);
            *w += ", ";
        }
        w.pop();
        w.pop();
        *w += "],\n";
    }
    *w += "];\n";

    *w += "#[inline]\npub const fn key(self) -> ";
    *w += name_k;
    *w += " {\n";
    *w += "unsafe { ::core::mem::transmute::<";
    *w += repr_k.to_int();
    *w += ", ";
    *w += name_k;
    *w += ">(*Self::K.as_ptr().add(self as usize)) }\n";
    *w += "}\n";

    *w += "#[inline]\npub const fn val(self) -> &'static [";
    *w += name_v;
    *w += "] {\n";
    *w += "unsafe { ::core::mem::transmute::<&'static [";
    *w += repr_v.to_int();
    *w += "], &'static [";
    *w += name_v;
    *w += "]>(*Self::V.as_ptr().add(self as usize)) }\n";
    *w += "}\n";

    *w += "}\n";

    let (_, size, _) = head(iter.next(), "block_state_properties");
    let mut bs_prop_size = Vec::with_capacity(size);
    let mut bs_properties = Vec::with_capacity(size);

    for _ in 0..size {
        let props = iter.next().unwrap();
        if props.is_empty() {
            bs_prop_size.push(NonZeroUsize::new(1).unwrap());
            bs_properties.push(Box::<[u32]>::from([]));
            continue;
        }
        let props2 = hex_line(props).collect::<Box<_>>();

        let mut len = 1;
        for &prop in &*props2 {
            let prop2 = &*kv[prop as usize];
            len *= prop2.len() - 1;
        }
        bs_prop_size.push(NonZeroUsize::new(len).unwrap());
        bs_properties.push(props2);
    }

    let mut psn = ZString::with_capacity(bs_properties.len(), bs_properties.len() * 10);
    let mut dedup = HashMap::<Vec<u64>, u32>::with_capacity(bs_properties.len());
    for props in &bs_properties {
        let mut ctx = Vec::with_capacity(props.len());
        for &x in &**props {
            let prop = &*kv[x as usize];
            ctx.push(prop[0] as u64 | ((prop.len() as u64) << 32));
        }
        match dedup.entry(ctx) {
            Entry::Vacant(x) => {
                x.insert(0);
            }
            Entry::Occupied(x) => {
                *x.into_mut() += 1;
            }
        }
    }
    let mut ctx = Vec::<u64>::with_capacity(16);
    for props in bs_properties.iter().map(|x| &**x) {
        if props.is_empty() {
            psn.push("props_nil");
            continue;
        }
        ctx.clear();
        for &x in props {
            let prop = &*kv[x as usize];
            ctx.push(prop[0] as u64 | ((prop.len() as u64) << 32));
        }
        let dupe = match dedup.get_mut(&ctx) {
            None => unreachable!(),
            Some(n) => *n != 0,
        };
        name_.clear();
        name_ += "props";

        if dupe {
            for &x in props {
                name_.push('_');
                let prop = &*kv[x as usize];
                name_ += pk2[prop[0] as usize];
                for n in prop[1..].iter().map(|&x| pv2[x as usize]) {
                    name_ += "_";
                    name_.extend(n.split('_').map(|x| x.chars().next().unwrap()));
                }
            }
        } else {
            let mut c = 0;
            for &x in props {
                c += pk2[kv[x as usize][0] as usize].len();
            }
            if c > 32 {
                for &x in props {
                    name_.push('_');
                    let prop = &*kv[x as usize];
                    name_ += pk2[prop[0] as usize];
                }
            } else {
                for &x in props {
                    name_.push('_');
                    let prop = &*kv[x as usize];
                    name_ += pk2[prop[0] as usize];
                    write(&mut name_, prop.len() - 1);
                }
            }
        }
        psn.push(&*name_);
    }

    let mut vec1 = Vec::<u32>::with_capacity(256);
    let mut vec2 = Vec::<u32>::with_capacity(256);
    for (x, props) in bs_properties.iter().enumerate() {
        let name = &psn[x];

        let mut size = 1;
        let mut len = 1;
        for &prop in &**props {
            let prop2 = &*kv[prop as usize];

            let x = prop2.len() - 2;
            let y = usize::BITS - x.leading_zeros();
            size *= 1 << y;
            len *= prop2.len() - 1;
        }
        if props.is_empty() {
            *w += "#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]\n";
            *w += "#[repr(transparent)]\n#[must_use]\n";
            *w += "pub struct ";
            *w += name;
            *w += ";\n";
            *w += "impl ";
            *w += name;
            *w += " {\n";
            *w += "#[inline]\n";
            *w += "pub const fn new() -> Self {\n";
            *w += "Self\n";
            *w += "}\n";
            *w += "#[inline]\n#[must_use]\n";
            *w += "pub const fn encode(self) -> u8 {\n";
            *w += "0\n";
            *w += "}\n";
            *w += "#[inline]\n";
            *w += "pub const fn decode(n: u8) -> Self {\n";
            *w += "debug_assert!(n < ";
            write(w, len);
            *w += ");\n";
            *w += "Self\n";
            *w += "}\n";
            *w += "}\n";
            continue;
        }
        let bad = props[1..]
            .iter()
            .any(|&x| (kv[x as usize].len() - 1).count_ones() != 1);
        let repr = Repr::new(size);
        struct_head(w, repr, name);
        *w += "impl Default for ";
        *w += name;
        *w += " {\n";
        *w += "#[inline]\n";
        *w += "fn default() -> Self {\n";
        *w += "Self(0)\n";
        *w += "}\n}\n\n";
        *w += "impl ";
        *w += name;
        *w += " {\n";
        *w += "#[inline]\n";
        *w += "pub const fn new() -> Self {\n";
        *w += "Self(0)\n";
        *w += "}\n";
        if !bad {
            *w += "#[inline]\n#[must_use]\n";
            *w += "pub const fn encode(self) -> ";
            *w += repr.to_int();
            *w += " {\n";
            *w += "self.0\n";
            *w += "}\n";
        } else {
            *w += "#[inline]\n#[must_use]\n";
            *w += "pub const fn encode(self) -> ";
            *w += repr.to_int();
            *w += " {\n";

            let mut index = 1;
            let mut flag = true;
            for &prop in props.iter().rev() {
                let prop2 = &*kv[prop as usize];

                let (k, v) = prop2.split_first().unwrap();
                let s = pk2[*k as usize];
                if !flag {
                    *w += " +\n";
                }
                flag = false;
                *w += "self.";
                kw_prefix(w, s);
                *w += "()";
                *w += " as ";
                *w += repr.to_int();
                if index != 1 {
                    *w += " *\n";
                    write(w, index);
                }
                index *= v.len();
            }
            *w += "\n}\n";
        }
        if !bad {
            *w += "#[inline]\n";
            *w += "pub const fn decode(n: ";
            *w += repr.to_int();
            *w += ") -> Self {\n";
            *w += "debug_assert!(n < ";
            write(w, len);
            *w += ");\n";
            *w += "Self(n)\n";
            *w += "}\n";
        } else {
            list_ty(w, "M", repr, len);
            vec1.clear();
            vec2.clear();
            let mut index = 0;
            for &prop in props.iter().rev() {
                let prop2 = &*kv[prop as usize];

                let (_, v) = prop2.split_first().unwrap();
                let x = usize::BITS - (v.len() - 1).leading_zeros();

                if index == 0 {
                    vec1.extend(0..v.len() as u32);
                } else {
                    vec2.clear();
                    for v in 0..v.len() as u32 {
                        for &e in &vec1 {
                            vec2.push(e | (v << index));
                        }
                    }
                    core::mem::swap(&mut vec1, &mut vec2);
                }
                index += x;
            }
            list(w, vec1.iter().copied());
            *w += ";\n";
            *w += "#[inline]\n";
            *w += "pub const fn decode(n: ";
            *w += repr.to_int();
            *w += ") -> Self {\n";
            *w += "debug_assert!(n < ";
            write(w, len);
            *w += ");\n";
            *w += "unsafe { Self(*Self::M.as_ptr().add(n as usize)) }\n";
            *w += "}\n";
        }

        let mut index = 0;
        for &prop1 in props.iter().rev() {
            let prop = &*kv[prop1 as usize];
            let (&k_, v) = prop.split_first().unwrap();
            let k = pk2[k_ as usize];
            let x = usize::BITS - (v.len() - 1).leading_zeros();
            let reprp = if x > 16 {
                Repr::U32
            } else if x > 8 {
                Repr::U16
            } else {
                Repr::U8
            };
            *w += "#[inline]\n";
            *w += "pub const fn ";
            kw_prefix(w, k);
            *w += "(self) -> ";
            *w += &kvn[prop1 as usize];
            *w += " {\n";
            *w += "unsafe { ::core::mem::transmute::<";
            *w += reprp.to_int();
            *w += ", ";
            *w += &kvn[prop1 as usize];
            *w += ">(";
            if repr != reprp {
                *w += "(";
            }
            if index != 0 {
                *w += "(";
            }
            let mut m = 0u32;
            for n in index..index + x {
                m |= 1 << n;
            }
            *w += "self.0";
            if props.len() != 1 {
                *w += " & ";
                write(w, m);
            }
            if index != 0 {
                *w += ") >> ";
                write(w, index);
            }
            if repr != reprp {
                *w += ") as ";
                *w += reprp.to_int();
            }
            *w += ") }";
            *w += "\n}\n";
            index += x;
        }
        index = 0;
        for &prop1 in props.iter().rev() {
            let prop = &*kv[prop1 as usize];
            let (k, v) = prop.split_first().unwrap();
            let s = pk2[*k as usize];
            let x = usize::BITS - (v.len() - 1).leading_zeros();
            *w += "#[inline]\n";
            *w += "pub const fn with_";
            *w += s;
            *w += "(self, ";
            kw_prefix(w, s);
            *w += ": ";
            *w += &kvn[prop1 as usize];
            *w += ") -> Self {\n";
            *w += "Self(";
            if props.len() != 1 {
                *w += "(";
                let mut m = (size - 1) as u32;
                for n in index..index + x {
                    m ^= 1 << n;
                }
                *w += "self.0 & ";
                write(w, m);
                *w += ")";
                *w += " | (";
            }

            if index != 0 {
                *w += "(";
            }
            kw_prefix(w, s);
            *w += " as ";
            *w += repr.to_int();
            if index != 0 {
                *w += ")";
            }

            if index != 0 {
                *w += " << ";
                write(w, index);
            }

            if props.len() != 1 {
                *w += ")";
            }
            *w += ")";
            *w += "\n}\n";
            index += x;
        }
        *w += "}\n";
    }

    let (bs_name, block_size, _) = head(iter.next(), "block_state");
    assert_eq!(block_size, block_names.len());

    let mut offsets = Vec::<u32>::with_capacity(block_names.len());
    let mut bs_idx = 0;

    let mut bl_props = Vec::<u32>::with_capacity(block_names.len());
    let mut name_iter = block_names.iter();
    for props in read_rl(block_size, &mut iter) {
        offsets.push(bs_idx as u32);
        bl_props.push(props);
        let size = bs_prop_size[props as usize].get();
        bs_idx += size;

        *w += "pub use ";
        *w += &psn[props as usize];
        *w += " as ";
        *w += name_iter.next().unwrap();
        *w += ";\n";
    }

    let bs_size = bs_idx;
    let bs_repr = Repr::new(bs_size);

    *w += "impl block {\n";
    list_ty(w, "OFFSET", bs_repr, block_names.len());
    list(w, offsets.iter().copied());
    *w += ";\n";

    list_ty(
        w,
        "PROPS_INDEX",
        Repr::new(bs_properties.len()),
        block_names.len(),
    );
    list(w, bl_props.iter().copied());
    *w += ";\n";

    *w += "const PROPS: &[&[";
    *w += repr_kv.to_int();
    *w += "]; ";
    write(w, bs_properties.len());
    *w += "] = &[\n";
    for prop in &bs_properties {
        *w += "&[";
        for &x in &**prop {
            write(w, x);
            *w += ", ";
        }
        if !prop.is_empty() {
            w.pop();
            w.pop();
        }
        *w += "],\n";
    }
    w.pop();
    w.pop();
    *w += "];\n";
    *w += "}\n";

    struct_head(w, bs_repr, bs_name);
    impl_common(w, bs_name, bs_repr, bs_size, 0);
    impl_codec_struct(w, bs_name, bs_size, bs_repr);
    list_ty(
        w,
        "BLOCK_STATE_TO_BLOCK",
        Repr::new(block_names.len()),
        bs_size,
    );
    list(
        w,
        bl_props.iter().enumerate().flat_map(|(index, &offset)| {
            core::iter::repeat_n(index, bs_prop_size[offset as usize].get())
        }),
    );
    *w += ";\n";

    let (_, size, _) = head(iter.next(), "block_to_default_block_state");
    assert_eq!(size, block_names.len());

    *w += "impl block {\n";
    list_ty(w, "DEFAULT", bs_repr, block_names.len());
    let mut out = Vec::with_capacity(size);
    let _: u32 = read_rl(size, &mut iter).fold(0, |prev, x| {
        out.push(x.wrapping_add(prev));
        1 + x.wrapping_add(prev)
    });
    list(w, out.into_iter());
    *w += ";\n";
    *w += "#[inline]\n";
    *w += "pub const fn state_default(self) -> block_state {\n";
    *w += "unsafe { block_state(*Self::DEFAULT.as_ptr().add(self as usize)) }\n";
    *w += "}\n";
    *w += "}\n";
    let (_, size, _) = head(iter.next(), "block_item_to_block");

    *w += "const ITEM: &[raw_block; item::MAX as usize + 1] = ";
    let mut out = Vec::with_capacity(size);
    let _: u32 = read_rl(size, &mut iter).fold(0, |prev, x| {
        out.push(x.wrapping_add(prev));
        1 + x.wrapping_add(prev)
    });
    list(w, out.into_iter());
    *w += ";\n";

    let (_, size, _) = head(iter.next(), "float32_table");

    let mut f32t = Vec::with_capacity(size);
    for _ in 0..size {
        let x = parse_u32(iter.next().unwrap());
        f32t.push(x);
    }
    let (_, size, _) = head(iter.next(), "float64_table");

    let mut f64t = Vec::with_capacity(size);
    for _ in 0..size {
        let x = parse_u64(iter.next().unwrap());
        let y = f64::from_be_bytes(x.to_be_bytes());
        f64t.push(y);
    }
    let (_, size, shape_repr) = head(iter.next(), "shape_table");

    *w += "const SHAPES: &[&[[f64; 6]]; ";
    write(w, size);
    *w += "] = &[";
    let mut shape = Vec::new();
    for _ in 0..size {
        let s = hex_line(iter.next().unwrap());
        for x in s {
            shape.push(f64t[x as usize]);
        }
        *w += "&[";
        let mut first2 = true;
        for x in shape.as_chunks::<6>().0 {
            if !first2 {
                *w += ", ";
            }
            first2 = false;
            *w += "[\n";
            let mut first = true;
            for &x in x {
                if !first {
                    *w += ", ";
                }
                first = false;
                write(w, x);
                *w += "f64";
            }
            *w += "\n]";
        }

        *w += "],\n";
        shape.clear();
    }
    *w += "];\n";
    let (_, size, _) = head(
        iter.next(),
        "block_settings_table#hardness blast_resistance slipperiness velocity_multiplier jump_velocity_multiplier",
    );

    let mut bs_ettings = Vec::with_capacity(size);
    for _ in 0..size {
        let mut s = hex_line(iter.next().unwrap());
        let mut settings = [0_u32; 5];
        for s1 in &mut settings {
            let x = s.next().unwrap();
            *s1 = f32t[x as usize];
        }
        bs_ettings.push(settings);
    }
    *w += "const BLOCK_SETTINGS: &[";
    *w += "[f32; 5]";
    *w += "; ";
    write(w, size);
    *w += "] = &[";
    for &x in bs_ettings.iter() {
        *w += "[";
        for x in x {
            write(w, f32::from_bits(x));
            *w += "f32";
            *w += ", ";
        }
        w.pop();
        w.pop();
        *w += "],\n";
    }
    *w += "];\n";

    let repr = Repr::new(size);
    let (_, size, _) = head(iter.next(), "block_settings");
    list_ty(w, "BLOCK_SETTINGS_INDEX", repr, block_names.len());
    list(w, read_rl(size, &mut iter));
    *w += ";\n";

    let (_, size, _) = head(
        iter.next(),
        "block_state_flags#(has_sided_transparency lava_ignitable material_replaceable opaque tool_required exceeds_cube redstone_power_source has_comparator_output)",
    );
    assert_eq!(size, bs_size);
    list_ty(w, "BLOCK_STATE_FLAGS", Repr::U8, size);
    list(w, read_rl(size, &mut iter).map(|x| x as u8));
    *w += ";\n";

    let (_, size, _) = head(iter.next(), "block_state_luminance");
    list_ty(w, "BLOCK_STATE_LUMINANCE", Repr::U8, size);
    list(w, read_rl(size, &mut iter).map(|x| x as u8));
    *w += ";\n";
    let (_, size, _) = head(
        iter.next(),
        "block_state_static_bounds_table#(opacity(4) solid_block translucent full_cube opaque_full_cube) side_solid_full side_solid_center side_solid_rigid collision_shape culling_shape",
    );

    assert_eq!(shape_repr, Repr::U16);
    list_ty(w, "BLOCK_STATE_BOUNDS", Repr::U64, size);
    let mut out = Vec::with_capacity(size);
    for _ in 0..size {
        let s = iter.next().unwrap();
        if s.is_empty() {
            out.push(0u64);
            continue;
        }
        let mut line = hex_line(s);
        let n1 = line.next().unwrap() as u8;
        let n2 = line.next().unwrap() as u8;
        let n3 = line.next().unwrap() as u8;
        let n4 = line.next().unwrap() as u8;
        let m5 = line.next().unwrap() as u16;
        let m6 = line.next().unwrap() as u16;
        let [n5, n6] = m5.to_le_bytes();
        let [n7, n8] = m6.to_le_bytes();
        let v = u64::from_le_bytes([n1, n2, n3, n4, n5, n6, n7, n8]);
        assert_ne!(v, 0);
        out.push(v);
    }
    list(w, out.into_iter());
    *w += ";\n";
    let (_, bsb_size, _) = head(iter.next(), "block_state_static_bounds_map");

    let prop_bounds = (&mut iter)
        .take(bsb_size)
        .map(|arr| hex_line(arr).collect::<Box<_>>())
        .collect::<Box<_>>();

    let (_, size, _) = head(iter.next(), "block_state_static_bounds");
    assert_eq!(size, block_size);
    list_ty(w, "BLOCK_STATE_BOUNDS_INDEX", Repr::U16, bs_size);

    let mut out = Vec::with_capacity(bs_size);
    assert_eq!(size, bl_props.len());
    for (bounds, prop) in read_rl(size, &mut iter).zip(bl_props.iter().copied()) {
        let bounds2 = &*prop_bounds[bounds as usize];
        let len = bs_prop_size[prop as usize].get();
        match bounds2[..] {
            [] => {
                out.extend(repeat_n(0, len));
            }
            [b] => {
                out.extend(repeat_n(b, len));
            }
            ref bounds3 => {
                assert_eq!(len, bounds3.len());
                out.extend(bounds3.iter().copied());
            }
        }
    }

    list(w, out.into_iter());
    *w += ";\n";

    (bs_repr, bl_props, bs_prop_size)
}

fn item(w: &mut String, data: &str) {
    let mut iter = data.split('\n');
    let (_, size, _) = head(iter.next(), "item_max_count");
    list_ty(w, "ITEM_MAX_COUNT", Repr::U8, size);
    list(w, read_rl(size, &mut iter).map(|x| x as u8));
    *w += ";\n";
}

fn entity(w: &mut String, data: &str) {
    let mut iter = data.split('\n');

    let (_, size, _) = head(iter.next(), "entity_type_height");
    *w += "const ENTITY_HEIGHT: &[f32; ";
    write(w, size);
    *w += "] = ";
    list(w, read_rl(size, &mut iter).map(f32::from_bits));
    *w += ";\n";

    let (_, size, _) = head(iter.next(), "entity_type_width");
    *w += "const ENTITY_WIDTH: &[f32; ";
    write(w, size);
    *w += "] = ";
    list(w, read_rl(size, &mut iter).map(f32::from_bits));
    *w += ";\n";

    let (_, size, _) = head(iter.next(), "entity_type_fixed");
    *w += "const ENTITY_FIXED: &[u8; ";
    write(w, size);
    *w += "] = ";
    list(w, read_rl(size, &mut iter));
    *w += ";\n";
}

fn head<'a>(raw: Option<&'a str>, expected: &str) -> (&'a str, usize, Repr) {
    let raw2 = raw.expect("EOF");
    let Some(first) = raw2.strip_prefix(';') else {
        panic!("invalid head: {raw2}");
    };
    let (name, rest) = first.split_once(';').unwrap();
    let (_ty, size) = rest.split_once(';').unwrap();
    let size2 = parse_u32(size);
    let size3 = size2 as usize;
    if !expected.is_empty() {
        assert_eq!(expected, name);
    }
    (name, size3, Repr::new(size3))
}

fn impl_name(w: &mut String, g: &mut GenerateHash, repr: Repr, names: &[&str], name: &str) {
    *w += "impl ";
    *w += name;
    *w += " {\n";
    *w += "const N: &[&str; ";
    write(w, names.len());
    *w += "] = &[\n";
    for &val in names {
        *w += "\"";
        *w += val;
        *w += "\",\n";
    }
    *w += "];\n";
    *w += "#[inline]
#[must_use]
pub const fn name(self) -> &'static str {
unsafe {
*Self::N.as_ptr().add(self as usize)
}
}
";

    let state = if names.len() <= 8 {
        None
    } else {
        let state = g.generate_hash(names);
        list_ty(w, "DISPS", Repr::U64, state.disps.len());
        list(
            w,
            state
                .disps
                .iter()
                .map(|&(h, l)| ((h as u64) << 32) | l as u64),
        );
        *w += ";\n";
        list_ty(w, "VALS", repr, state.map.len());
        list(w, state.map.iter().map(|&ele| ele.unwrap()));
        *w += ";\n";
        Some(state)
    };
    *w += "}\n";
    *w += "impl ::core::str::FromStr for ";
    *w += name;
    *w += " {
type Err = ::mser::Error;
fn from_str(n: &str) -> Result<Self, Self::Err> {
";
    if let Some(state1) = state {
        match repr {
            Repr::U8 => {
                *w += "match crate::name_u8::<";
                write(w, state1.key);
                *w += ", ";
                write(w, state1.disps.len());
                *w += ", ";
                write(w, names.len());
            }
            Repr::U16 => {
                *w += "match crate::name_u16::<";
                write(w, state1.key);
                *w += ", ";
                write(w, state1.disps.len());
                *w += ", ";
                write(w, names.len());
            }
            _ => unimplemented!(),
        }

        *w += ">(Self::DISPS, Self::N.as_ptr(), Self::VALS, n) {\n";
        *w += "::core::option::Option::Some(x) => unsafe { ::core::result::Result::Ok(::core::mem::transmute::<";
        *w += repr.to_int();
        *w += ", Self>(x)) },
::core::option::Option::None => ::core::result::Result::Err(::mser::Error),
}";
    } else {
        *w += "unsafe {\n";
        *w += "::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(match n {\n";
        for (i, &val) in names.iter().enumerate() {
            *w += "\"";
            *w += val;
            *w += "\" => ";
            write(w, i);
            *w += ",\n";
        }
        *w += "_ => return ::core::result::Result::Err(::mser::Error),\n";
        *w += "}\n)\n)\n}";
    }
    *w += "\n}\n}\n";
    *w += "impl ::core::fmt::Display for ";
    *w += name;
    *w += " {
#[inline]
fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
f.write_str(self.name())
}
}
";

    *w += "impl ::core::fmt::Debug for ";
    *w += name;
    *w += " {
#[inline]
fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
f.write_str(self.name())
}
}
";
}

fn enum_head(w: &mut String, repr: Repr, name: &str) {
    *w += "#[derive(Clone, Copy, PartialEq, Eq, Hash)]\n";
    *w += "#[repr(";
    *w += repr.to_int();
    *w += ")]\n#[must_use]\n";
    *w += "pub enum ";
    *w += name;
    *w += " {\n";
}

fn struct_head(w: &mut String, repr: Repr, name: &str) {
    *w += "#[derive(Clone, Copy, PartialEq, Eq, Hash)]\n";
    *w += "#[repr(transparent)]\n#[must_use]\n";
    *w += "pub struct ";
    *w += name;
    *w += "(";
    *w += repr.to_int();
    *w += ");\n";
}

fn hex_line(x: &str) -> impl Iterator<Item = u32> + Clone + '_ {
    x.split_ascii_whitespace()
        .map(|n| u32::from_str_radix(n, 16).expect("parse hex"))
}

struct GenerateHash {
    wy_rand: u64,
    hashes: Vec<u64>,
    buckets: Vec<Bucket>,
    values_to_add: Vec<(usize, u32)>,
    map: Vec<Option<u32>>,
    disps: Vec<(u32, u32)>,
    try_map: Vec<u64>,
}

impl GenerateHash {
    fn new() -> Self {
        Self {
            wy_rand: 0x3BD39E10CB0EF593u64,
            hashes: Vec::new(),
            buckets: Vec::new(),
            values_to_add: Vec::new(),
            map: Vec::new(),
            try_map: Vec::new(),
            disps: Vec::new(),
        }
    }

    fn generate_hash(&mut self, entries: &[&str]) -> HashState<'_> {
        'key: loop {
            let key = self.next();
            self.hashes.clear();
            self.hashes
                .extend(entries.iter().map(|&x| hash64(x.as_bytes(), key)));
            let table_len = self.hashes.len();
            let buckets_len = table_len.div_ceil(DEFAULT_LAMBDA);

            if self.buckets.len() < buckets_len {
                self.buckets
                    .extend((self.buckets.len()..buckets_len).map(|i| Bucket {
                        idx: i,
                        keys: Vec::new(),
                    }));
            }

            for (i, bucket) in self.buckets[0..buckets_len].iter_mut().enumerate() {
                bucket.idx = i;
                bucket.keys.clear();
            }
            for (i, a) in self.hashes.iter().enumerate() {
                self.buckets[(((a >> 24) as u32) % buckets_len as u32) as usize]
                    .keys
                    .push(i as u32);
            }

            self.buckets[0..buckets_len]
                .sort_unstable_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

            self.map.clear();
            self.map.extend(repeat_n(None, table_len));

            self.disps.clear();
            self.disps.extend(repeat_n((0u32, 0u32), buckets_len));

            self.try_map.clear();
            self.try_map.extend(repeat_n(0u64, table_len));

            let mut generation = 0u64;

            'buckets: for bucket in &self.buckets[0..buckets_len] {
                let max_d = table_len as u32;
                for d1 in 0..max_d {
                    'disps: for d2 in 0..max_d {
                        self.values_to_add.clear();
                        generation += 1;

                        for &key in &bucket.keys {
                            let a = (&mut self.hashes)[key as usize];
                            let f1 = a as u32;
                            let f2 = (a >> 32) as u32;
                            let x = d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2);
                            let idx = (x % (table_len as u32)) as usize;
                            if self.map[idx].is_some() || self.try_map[idx] == generation {
                                continue 'disps;
                            }
                            self.try_map[idx] = generation;
                            self.values_to_add.push((idx, key));
                        }
                        self.disps[bucket.idx] = (d1, d2);
                        for &(idx, key) in self.values_to_add.iter() {
                            self.map[idx] = Some(key);
                        }
                        continue 'buckets;
                    }
                }
                continue 'key;
            }

            return HashState {
                key,
                disps: &self.disps,
                map: &self.map,
            };
        }
    }

    #[inline]
    fn next(&mut self) -> u64 {
        self.wy_rand = self.wy_rand.wrapping_add(0xa0761d6478bd642f);
        let x = (self.wy_rand ^ 0xe7037ed1a0b428db) as u128;
        let t = (self.wy_rand as u128).wrapping_mul(x);
        (t.wrapping_shr(64) ^ t) as u64
    }
}

const DEFAULT_LAMBDA: usize = 5;

struct HashState<'a> {
    key: u64,
    disps: &'a [(u32, u32)],
    map: &'a [Option<u32>],
}

struct Bucket {
    idx: usize,
    keys: Vec<u32>,
}

struct RunLength<T> {
    i: T,
    count: usize,
    prev: u32,
    take: usize,
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for RunLength<T> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.take == 0 {
            None
        } else if self.count == 0 {
            let next = self.i.next()?;
            self.take -= 1;
            match next.strip_prefix('~') {
                Some(rest) => {
                    let mut r = hex_line(rest);
                    let a = r.next().unwrap();
                    let ctx = r.next().unwrap();
                    self.count = a as usize - 1;
                    self.prev = ctx;
                    Some(ctx)
                }
                _ => Some(parse_u32(next)),
            }
        } else {
            self.take -= 1;
            self.count -= 1;
            Some(self.prev)
        }
    }
}

fn read_rl<'a, T: Iterator<Item = &'a str>>(size: usize, iter: T) -> RunLength<T> {
    RunLength {
        i: iter,
        count: 0,
        prev: 0,
        take: size,
    }
}

fn list_ty(w: &mut String, name: &str, repr: Repr, size: usize) {
    *w += "#[allow(clippy::large_const_arrays)]\n";
    *w += "const ";
    *w += name;
    *w += ": &[";
    *w += repr.to_int();
    *w += "; ";
    write(w, size);
    *w += "] = ";
}

fn list(w: &mut String, mut iter: impl Iterator<Item = impl Format>) {
    let first = iter.next();
    let f = match first {
        Some(x) => x,
        None => {
            *w += "&[]";
            return;
        }
    };
    let mut c = 0usize;
    *w += "&[\n";
    f.format(w);
    for x in iter {
        w.push(',');
        c += 1;
        if c == 8 {
            w.push('\n');
            c = 0;
        } else {
            w.push(' ');
        }
        x.format(w);
    }
    w.push(',');
    w.push('\n');
    w.push(']');
}

trait Format {
    fn format(&self, w: &mut String);
}

impl Format for str {
    fn format(&self, w: &mut String) {
        w.push_str(self);
    }
}

impl Format for usize {
    fn format(&self, w: &mut String) {
        write(w, *self);
    }
}

impl Format for u8 {
    fn format(&self, w: &mut String) {
        write(w, *self);
    }
}

impl Format for u32 {
    fn format(&self, w: &mut String) {
        write(w, *self);
    }
}

impl Format for u64 {
    fn format(&self, w: &mut String) {
        write(w, *self);
    }
}

impl Format for f32 {
    fn format(&self, w: &mut String) {
        write(w, *self);
        *w += "f32";
    }
}

fn parse_u32(n: &str) -> u32 {
    u32::from_str_radix(n.trim_ascii(), 16).expect("parse hex")
}

fn parse_u64(n: &str) -> u64 {
    u64::from_str_radix(n.trim_ascii(), 16).expect("parse hex")
}

const fn hash64(n: &[u8], seed: u64) -> u64 {
    let mut h: u64 = seed ^ (n.len() as u64);
    let mut i = 0;
    while i < n.len() {
        h ^= n[i] as u64;
        h = h.wrapping_mul(0x100000001b3);
        i += 1;
    }
    h
}

fn impl_codec_struct(w: &mut String, name: &str, size: usize, repr: Repr) {
    *w += "impl ::mser::Write for ";
    *w += name;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "fn len_s(&self) -> usize {\n";
    if size <= V7MAX {
        *w += "1usize";
    } else if size <= V21MAX {
        *w += "::mser::V21(self.0 as u32).len_s()";
    } else {
        *w += "::mser::V32(self.0 as u32).len_s()";
    }
    *w += "\n}\n";
    *w += "#[inline]\n";
    *w += "unsafe fn write(&self, w: &mut ::mser::Writer) {\n";
    if size <= V7MAX {
        *w += "unsafe { w.write_byte(self.0 as u8); }";
    } else if size <= V21MAX {
        *w += "unsafe { ::mser::Write::write(&::mser::V21(self.0 as u32), w); }";
    } else {
        *w += "unsafe { ::mser::Write::write(&::mser::V32(self.0 as u32), w); }";
    }
    *w += "\n}\n}\n";

    *w += "impl ::mser::Read<'_> for ";
    *w += name;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "fn read(n: &mut ::mser::Reader<'_>) -> ::core::result::Result<Self, ::mser::Error> {\n";
    *w += "let __x = <";
    if size <= V21MAX {
        *w += "::mser::V21 as ::mser::Read>::read(n)?.0";
    } else {
        *w += "::mser::V32 as ::mser::Read>::read(n)?.0";
    }
    *w += ";\n";
    if size == 1 {
        *w += "if __x == 0 {\n";
    } else {
        *w += "if __x <= ";
        write(w, size - 1);
        *w += " {\n";
    }
    *w += "::core::result::Result::Ok(Self(__x as ";
    *w += repr.to_int();
    *w += "))\n";
    *w += "} else {\n";
    *w += "::core::result::Result::Err(::mser::Error)\n";
    *w += "}\n}\n}\n";
}

fn impl_codec(w: &mut String, name: &str, size: usize, repr: Repr) {
    *w += "impl ::mser::Write for ";
    *w += name;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "fn len_s(&self) -> usize {\n";
    if size <= V7MAX {
        *w += "1usize";
    } else if size <= V21MAX {
        *w += "::mser::V21(*self as u32).len_s()";
    } else {
        *w += "::mser::V32(*self as u32).len_s()";
    }
    *w += "\n}\n";
    *w += "#[inline]\n";
    *w += "unsafe fn write(&self, w: &mut ::mser::Writer) {\n";
    if size <= V7MAX {
        *w += "unsafe { w.write_byte(*self as u8); }";
    } else if size <= V21MAX {
        *w += "unsafe { ::mser::Write::write(&::mser::V21(*self as u32), w); }";
    } else {
        *w += "unsafe { ::mser::Write::write(&::mser::V32(*self as u32), w); }";
    }
    *w += "\n}\n}\n";

    *w += "impl ::mser::Read<'_> for ";
    *w += name;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "fn read(n: &mut ::mser::Reader<'_>) -> ::core::result::Result<Self, ::mser::Error> {\n";
    *w += "let __x = <";
    if size <= V21MAX {
        *w += "::mser::V21 as ::mser::Read>::read(n)?.0";
    } else {
        *w += "::mser::V32 as ::mser::Read>::read(n)?.0";
    }
    *w += ";\n";
    if size == 1 {
        *w += "if __x == 0 {\n";
    } else {
        *w += "if __x <= ";
        write(w, size - 1);
        *w += " {\n";
    }
    *w += "unsafe { ::core::result::Result::Ok(::core::mem::transmute::<";
    *w += repr.to_int();
    *w += ", Self>(__x as ";
    *w += repr.to_int();
    *w += ")) }\n";
    *w += "} else {\n";
    *w += "::core::result::Result::Err(::mser::Error)\n";
    *w += "}\n}\n}\n";
}

fn write(w: &mut String, f: impl core::fmt::Display) {
    core::fmt::Write::write_fmt(w, format_args!("{f}")).unwrap();
}
