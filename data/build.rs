use mser::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env::var_os;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Repr {
    U32,
    U16,
    U8,
}

impl Repr {
    const fn new(size: usize) -> Self {
        if size > u16::MAX as usize {
            Self::U32
        } else if size > u8::MAX as usize {
            Self::U16
        } else {
            Self::U8
        }
    }
    #[inline]
    const fn to_int(self) -> &'static str {
        match self {
            Self::U32 => "u32",
            Self::U16 => "u16",
            Self::U8 => "u8",
        }
    }
    #[inline]
    const fn to_arr(self) -> &'static str {
        match self {
            Self::U32 => "[u8; 4]",
            Self::U16 => "[u8; 2]",
            Self::U8 => "[u8; 1]",
        }
    }
}

fn read(buf: &mut String, path: PathBuf) -> std::io::Result<usize> {
    let last = buf.chars().next_back();
    if let Some(last) = last {
        if last != '\n' {
            buf.push('\n');
        }
    }
    let mut file = std::fs::File::open(path)?;
    let size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
    buf.try_reserve_exact(size)?;
    std::io::Read::read_to_end(&mut file, unsafe { buf.as_mut_vec() })
}

fn main() -> std::io::Result<()> {
    let out = PathBuf::from(var_os("OUT_DIR").unwrap());
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut w = String::with_capacity(0x1000);
    let mut wn = Vec::with_capacity(0x1000);
    let mut data = String::with_capacity(0x20000);
    let mut gen_hash = GenerateHash::new();

    read(&mut data, path.join("version.txt"))?;
    version(&mut w, &data);
    data.clear();

    let reg = read(&mut data, path.join("registries.txt"))?;
    let reg = 0..reg;

    let flu = read(&mut data, path.join("fluid_state.txt"))?;
    let flu = reg.end..reg.end + flu;

    let blo = read(&mut data, path.join("block_state.txt"))?;
    let blo = flu.end..flu.end + blo;

    let ite = read(&mut data, path.join("item.txt"))?;
    let ite = blo.end..blo.end + ite;

    let ent = read(&mut data, path.join("entity.txt"))?;
    let ent = ite.end..ite.end + ent;

    let pac = read(&mut data, path.join("packet.txt"))?;
    let pac = ent.end..ent.end + pac;

    let block_names = registries(&mut w, &mut wn, data.get(reg).unwrap(), &mut gen_hash);
    registries(&mut w, &mut wn, data.get(pac).unwrap(), &mut gen_hash);

    item(&mut w, data.get(ite).unwrap());
    entity(&mut w, &mut wn, data.get(ent).unwrap());

    let bsrepr = block_state(
        &mut w,
        &mut wn,
        data.get(blo).unwrap(),
        &mut gen_hash,
        &block_names,
    );
    fluid_state(&mut w, &mut wn, data.get(flu).unwrap(), bsrepr);
    w += "const NAMES: &[u8; ";
    w += itoa::Buffer::new().format(wn.len());
    w += "] = include_bytes!(\"";
    w += "data";
    w += ".bin\");\n";
    std::fs::write(out.join("data.rs"), w)?;
    std::fs::write(out.join("data.bin"), wn)?;
    Ok(())
}

fn version(w: &mut String, data: &str) {
    let mut data = data.split('\n');
    let name = data.next().unwrap();
    let proto = data.next().unwrap();

    *w += "pub const NAME_VERSION: &str = \"";
    *w += name;
    *w += "\";\n";
    *w += "pub const PROTOCOL_VERSION: u32 = 0x";
    *w += proto;
    *w += ";\n";
}

fn registries<'a>(
    w: &mut String,
    wn: &mut Vec<u8>,
    data: &'a str,
    gen_hash: &mut GenerateHash,
) -> Vec<&'a str> {
    let mut zhash = Vec::<&str>::new();
    let mut iter = data.split('\n');
    let mut block_names = Vec::<&str>::new();

    while let Some(x) = iter.next() {
        if x.is_empty() {
            break;
        }
        let (name, size, repr) = head(Some(x));
        let name = name.replace('/', "__");
        zhash.clear();
        zhash.reserve(size);
        for _ in 0..size {
            let data = iter.next().unwrap();
            zhash.push(data);
        }
        if name == "block" {
            block_names.clone_from(&zhash);
        }
        gen_enum(&zhash, size, w, repr, &name);
        *w += "impl ";
        *w += &name;
        *w += " {\n";
        gen_max(w, &zhash);
        namemap(w, gen_hash, wn, repr, &zhash);
        *w += "}\n";
        impl_name(w, &name);
    }
    block_names
}

fn fluid_state(w: &mut String, wn: &mut Vec<u8>, data: &str, bsrepr: Repr) {
    let mut ib = itoa::Buffer::new();
    let mut iter = data.split('\n');
    let (name, size, repr) = head(iter.next());
    assert_eq!(name, "fluid_state");
    struct_head(w, repr, name);
    *w += "impl ";
    *w += name;
    *w += " {\n";
    for index in 0..size {
        let name = iter.next().unwrap();
        *w += "pub const ";
        *w += name;
        *w += ": Self = Self(";
        *w += ib.format(index);
        *w += ");\n";
    }
    *w += "pub const MAX: usize = ";
    *w += ib.format(size - 1);
    *w += ";\n";
    *w += "#[inline]\n";
    *w += "pub const fn new(n: ";
    *w += repr.to_int();
    *w += ") -> Self {\n";
    *w += "debug_assert!(n <= Self::MAX as ";
    *w += repr.to_int();
    *w += ");\n";
    *w += "Self(n)\n";
    *w += "}\n";
    *w += "#[inline]\n";
    *w += "pub const fn id(self) -> ";
    *w += repr.to_int();
    *w += " {\n";
    *w += "self.0\n";
    *w += "}\n";
    *w += "}\n";
    let (name, size, _) = head(iter.next());
    assert_eq!(name, "fluid_to_block");
    wn.reserve(size);
    *w += "const FLUID_STATE_TO_BLOCK: *const ";
    *w += bsrepr.to_arr();
    *w += " = unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    for _ in 0..size {
        let next = iter.next().unwrap().as_bytes();
        let (n, _) = parse_hex::<u32>(next);
        match bsrepr {
            Repr::U32 => wn.extend(n.to_le_bytes()),
            Repr::U16 => wn.extend((n as u16).to_le_bytes()),
            Repr::U8 => wn.push(n as u8),
        }
    }

    let (name, size, _) = head(iter.next());
    assert_eq!(name, "fluid_state_level");
    wn.reserve(size);

    *w += "const FLUID_STATE_LEVEL: *const [u8; 1] = unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    for _ in 0..size {
        let next = iter.next().unwrap().as_bytes();
        let (n, _) = parse_hex::<u8>(next);
        wn.push(n);
    }

    let (name, size, _) = head(iter.next());
    wn.reserve(size);
    assert_eq!(name, "fluid_state_falling");
    *w += "const FLUID_STATE_FALLING: *const [u8; 1] = unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    for _ in 0..size {
        let next = iter.next().unwrap().as_bytes();
        let (n, _) = parse_hex::<u8>(next);
        wn.push(n);
    }

    let (name, size, _) = head(iter.next());
    assert_eq!(name, "fluid_state_to_fluid");
    wn.reserve(size);

    *w += "const FLUID_STATE_TO_FLUID: *const [u8; 1] = unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    for _ in 0..size {
        let next = iter.next().unwrap().as_bytes();
        let (n, _) = parse_hex::<u8>(next);
        wn.push(n);
    }

    let (name, size, _) = head(iter.next());
    wn.reserve(size);
    assert_eq!(name, "block_to_fluid_state");

    *w += "const FLUID_STATE: *const ";
    *w += "[u8; 1]";
    *w += " = unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    wn.extend(read_rl(size, &mut iter).map(|x| x as u8));
}

fn block_state(
    w: &mut String,
    wn: &mut Vec<u8>,
    data: &str,
    gen_hash: &mut GenerateHash,
    block_names: &[&str],
) -> Repr {
    let mut ib = itoa::Buffer::new();
    let mut iter = data.split('\n');

    let (namek, size, reprk) = head(iter.next());
    assert_eq!(namek, "block_state_property_key");

    let mut pk1 = Vec::with_capacity(size);
    let mut pk2 = vec![""; size];
    let mut pk3 = vec![0_usize; size];
    for index in 0..size {
        let data = iter.next().unwrap();
        pk1.push((data, index));
    }
    pk1.sort_unstable_by(|x, y| x.0.cmp(y.0));
    for (sorted, &(value, before)) in pk1.iter().enumerate() {
        pk2[sorted] = value;
        pk3[before] = sorted;
    }

    let (namev, size, reprv) = head(iter.next());
    assert_eq!(namev, "block_state_property_value");

    let mut pv1 = Vec::with_capacity(size);
    let mut pv2 = vec![""; size];
    let mut pv3 = vec![0_usize; size];
    for index in 0..size {
        let data = iter.next().unwrap();
        pv1.push((data, index));
    }
    pv1.sort_unstable_by(|x, y| x.0.cmp(y.0));
    for (sorted, &(value, before)) in pv1.iter().enumerate() {
        pv2[sorted] = value;
        pv3[before] = sorted;
    }

    let (namekv, size, reprkv) = head(iter.next());
    assert_eq!(namekv, "block_state_property");

    let kv = (0..size)
        .map(|_| {
            let mut x = hex_line(iter.next().unwrap());
            let k = x.next().unwrap();
            let mut vec = Vec::with_capacity(4);
            vec.push(pk3[k as usize] as u32);
            vec.extend(x.map(|x| pv3[x as usize] as u32));
            vec.into_boxed_slice()
        })
        .collect::<Vec<_>>();
    drop(pk3);
    drop(pv3);

    enum_head(w, reprk, namek);
    for &ele in &pk2 {
        if ele == "type" {
            *w += "r#";
        }
        *w += ele;
        *w += ",\n";
    }
    enum_foot(w, reprk, namek);

    enum_head(w, reprv, namev);
    for &val in &pv2 {
        let b = *val.as_bytes().first().unwrap();
        if b.is_ascii_digit() {
            *w += "d_";
        } else if val == "false" || val == "true" {
            *w += "r#";
        }

        *w += val;
        *w += ",\n";
    }
    enum_foot(w, reprv, namev);

    let mut kvn = Vec::<Box<str>>::with_capacity(kv.len());
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
    let mut name = String::new();
    name += "val";
    let mut vals = x.iter().map(|(k, v)| (v, k)).collect::<Vec<_>>();
    vals.sort_unstable_by(|(x, _), (y, _)| (*x).cmp(*y));
    for (_, x) in vals {
        name.truncate(3);
        for &n in &**x {
            name.push('_');
            name += n;
        }
        let name = name.as_str();
        let repr = Repr::new(x.len() - 1);
        enum_head(w, repr, name);
        for &n in &**x {
            if n == "true" || n == "false" {
                *w += "r#";
            } else if n.as_bytes().first().unwrap().is_ascii_digit() {
                *w += "d_";
            }
            *w += n;
            *w += ",\n";
        }
        enum_foot(w, repr, name);
        *w += "impl ";
        *w += name;
        *w += " {\n";
        gen_max(w, x);
        namemap(w, gen_hash, wn, repr, x);
        *w += "}\n";
        impl_name(w, name);
    }
    let mut xn = HashMap::<&str, (usize, bool)>::new();
    let mut x2 = Vec::new();
    for arr in &kv {
        x2.clear();
        x2.extend(arr[1..].iter().map(|&x| pv2[x as usize]));
        let idx = x.get(&*x2).copied().unwrap();
        let k = pk2[arr[0] as usize];
        match xn.entry(k) {
            Entry::Vacant(x) => {
                x.insert((idx, false));
            }
            Entry::Occupied(x) => {
                let x = x.into_mut();
                if x.0 != idx {
                    x.1 = true;
                }
            }
        }
    }
    for arr in &kv {
        let k = pk2[arr[0] as usize];
        let dupe = xn.get(k).unwrap().1;
        let mut w = String::new();
        w += "prop_";
        w += k;
        if dupe {
            let x = pv2[arr[1] as usize].as_bytes();
            if x.iter().all(|x| x.is_ascii_digit()) {
                w += "_";
                w += ib.format(arr.len() - 1);
            } else {
                for v in arr[1..].iter().map(|&x| pv2[x as usize]) {
                    w += "_";
                    w += v;
                }
            }
        }
        kvn.push(w.into_boxed_str());
    }
    for (index, props) in kv.iter().enumerate() {
        *w += "pub type ";
        *w += &kvn[index];
        *w += " = val";
        for &n in &props[1..] {
            w.push('_');
            *w += pv2[n as usize];
        }
        *w += ";\n";
    }
    enum_head(w, reprkv, namekv);
    for name in &kvn {
        *w += &name[5..];
        *w += ",\n";
    }
    enum_foot(w, reprkv, namekv);

    *w += "impl ";
    *w += namek;
    *w += " {\n";
    gen_max(w, &pk2);
    namemap(w, gen_hash, wn, reprk, &pk2);
    *w += "}\n";
    impl_name(w, namek);

    *w += "impl ";
    *w += namev;
    *w += " {\n";
    gen_max(w, &pv2);
    namemap(w, gen_hash, wn, reprv, &pv2);
    *w += "}\n";
    impl_name(w, namev);

    *w += "impl ";
    *w += namekv;
    *w += " {\n";

    *w += "const MAX: usize = ";
    *w += ib.format(size);
    *w += ";\n";
    *w += "const K: [";
    *w += reprk.to_int();
    *w += "; ";
    *w += ib.format(size);
    *w += "] = [";
    for data in &kv {
        *w += ib.format(data[0]);
        *w += ", ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    *w += "const V: [&'static [";
    *w += reprv.to_int();
    *w += "]; ";
    *w += ib.format(size);
    *w += "] = [";
    for data in &kv {
        *w += "&[";
        for &v in &data[1..] {
            *w += ib.format(v as usize);
            *w += ", ";
        }
        w.pop();
        w.pop();
        *w += "], ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    *w += "#[inline]\npub const fn key(self) -> ";
    *w += namek;
    *w += " {\n";
    *w += "unsafe { ::core::mem::transmute::<";
    *w += reprk.to_int();
    *w += ", ";
    *w += namek;
    *w += ">(*Self::K.as_ptr().add(self as usize)) }\n";
    *w += "}\n";

    *w += "#[inline]\npub const fn val(self) -> &'static [";
    *w += namev;
    *w += "] {\n";
    *w += "unsafe { ::core::mem::transmute(*Self::V.as_ptr().add(self as usize)) }\n";
    *w += "}\n";

    *w += "}\n";

    let (name, size, _) = head(iter.next());
    let mut properties_size = Vec::with_capacity(size);
    assert_eq!(name, "block_state_properties");
    let mut block_state_properties = Vec::with_capacity(size);

    for _ in 0..size {
        let props = iter.next().unwrap();
        if props.is_empty() {
            properties_size.push(1);
            block_state_properties.push(Box::<[u32]>::from([]));
            continue;
        }
        let props = hex_line(props).collect::<Box<_>>();

        let mut len = 1;
        for &prop in &*props {
            let prop = &*kv[prop as usize];
            len *= prop.len() - 1;
        }
        properties_size.push(len);
        block_state_properties.push(props);
    }
    let mut psn = Vec::<Box<str>>::with_capacity(block_state_properties.len());
    for props in &block_state_properties {
        let mut name = String::new();
        name.push_str("props");
        for &x in &**props {
            let prop = &*kv[x as usize];
            name.push('_');
            name.push_str(pk2[prop[0] as usize]);
            name.push_str(ib.format(prop.len() - 1));
        }
        if props.is_empty() {
            name.push_str("_nil");
        }
        psn.push(name.into_boxed_str());
    }
    for (x, props) in block_state_properties.iter().enumerate() {
        let name = &*psn[x];

        let mut size = 1;
        let mut len = 1;
        for &prop in &**props {
            let prop = &*kv[prop as usize];

            let x = prop.len() - 2;
            let x = usize::BITS - x.leading_zeros();
            size *= 1 << x;
            len *= prop.len() - 1;
        }
        if props.is_empty() {
            if !name.starts_with('_') {
                *w += "pub type raw_";
                *w += name;
                *w += " = ";
                *w += "()";
                *w += ";\n";
            }
            *w += "#[derive(Clone, Copy, PartialEq, Eq, Hash)]\n";
            *w += "#[repr(transparent)]\n#[must_use]\n";
            *w += "pub struct ";
            *w += name;
            *w += ";\n";
            *w += "impl Default for ";
            *w += name;
            *w += " {\n";
            *w += "#[inline]\n";
            *w += "fn default() -> Self {\n";
            *w += "Self\n";
            *w += "}\n}\n\n";
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
            *w += ib.format(len);
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
                let prop = &*kv[prop as usize];

                let (k, v) = prop.split_first().unwrap();
                let k = pk2[*k as usize];
                if !flag {
                    *w += " + ";
                }
                flag = false;
                *w += "self.";
                if k == "type" {
                    *w += "r#";
                }
                *w += k;
                *w += "()";
                *w += " as ";
                *w += repr.to_int();
                if index != 1 {
                    *w += " * ";
                    *w += ib.format(index);
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
            *w += ib.format(len);
            *w += ");\n";
            *w += "Self(n)\n";
            *w += "}\n";
        } else {
            *w += "const M: [";
            *w += repr.to_int();
            *w += "; ";
            *w += ib.format(len);
            *w += "] = [";

            let mut vec1 = Vec::<u32>::with_capacity(len);
            let mut vec2 = Vec::<u32>::with_capacity(len);
            let mut index = 0;
            for &prop in props.iter().rev() {
                let prop = &*kv[prop as usize];

                let (_, v) = prop.split_first().unwrap();
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
            for &ele in &vec1 {
                *w += ib.format(ele);
                *w += ", ";
            }
            w.pop();
            w.pop();

            *w += "];\n";
            *w += "#[inline]\n";
            *w += "pub const fn decode(n: ";
            *w += repr.to_int();
            *w += ") -> Self {\n";
            *w += "debug_assert!(n < ";
            *w += ib.format(len);
            *w += ");\n";
            *w += "unsafe { Self(*Self::M.as_ptr().add(n as usize)) }\n";
            *w += "}\n";
        }

        let mut index = 0;
        for &prop_ in props.iter().rev() {
            let prop = &*kv[prop_ as usize];
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
            if k == "type" {
                *w += "r#";
            }
            *w += k;
            *w += "(self) -> ";
            *w += &kvn[prop_ as usize];
            *w += " {\n";
            *w += "unsafe { ::core::mem::transmute(";
            if repr != reprp {
                *w += "(";
            }
            if index != 0 {
                *w += "(";
            }
            let mut m = 0_u32;
            for n in index..index + x {
                m |= 1 << n;
            }
            *w += "self.0";
            if props.len() != 1 {
                *w += " & ";
                *w += ib.format(m);
            }
            if index != 0 {
                *w += ") >> ";
                *w += ib.format(index);
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
        for &prop_ in props.iter().rev() {
            let prop = &*kv[prop_ as usize];
            let (k, v) = prop.split_first().unwrap();
            let k = pk2[*k as usize];
            let x = usize::BITS - (v.len() - 1).leading_zeros();
            *w += "#[inline]\n";
            *w += "pub const fn with_";
            *w += k;
            *w += "(self, ";
            if k == "type" {
                *w += "r#";
            }
            *w += k;
            *w += ": ";
            *w += &kvn[prop_ as usize];
            *w += ") -> Self {\n";
            *w += "Self(";
            if props.len() != 1 {
                *w += "(";
                let mut m = (size - 1) as u32;
                for n in index..index + x {
                    m ^= 1 << n;
                }
                *w += "self.0 & ";
                *w += ib.format(m);
                *w += ")";

                *w += " | (";
            }

            if index != 0 {
                *w += "(";
            }
            if k == "type" {
                *w += "r#";
            }
            *w += k;
            *w += " as ";
            *w += repr.to_int();
            if index != 0 {
                *w += ")";
            }

            if index != 0 {
                *w += " << ";
                *w += ib.format(index);
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

    let (bsname, bssize, _) = head(iter.next());
    assert_eq!(bsname, "block_state");
    assert_eq!(bssize, block_names.len());

    let mut offsets = Vec::with_capacity(block_names.len());
    let mut y = 0;
    let mut block_state = Vec::<u32>::with_capacity(block_names.len());
    let mut x = block_names.iter();
    for props in read_rl(bssize, &mut iter) {
        offsets.push(y as u32);
        block_state.push(props);
        y += properties_size[props as usize];

        *w += "pub type ";
        *w += x.next().unwrap();
        *w += " = ";
        *w += &psn[props as usize];
        *w += ";\n";
    }

    let bssize = y + 1;
    let bsrepr = Repr::new(bssize);

    *w += "impl block {\n";
    *w += "const OFFSET: [";
    *w += bsrepr.to_int();
    *w += "; ";
    *w += ib.format(block_names.len());
    *w += "] = [";
    for &offset in &offsets {
        *w += ib.format(offset);
        *w += ", ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    *w += "const PROPS_INDEX: [";
    *w += Repr::new(block_state_properties.len() + 1).to_int();
    *w += "; ";
    *w += ib.format(block_names.len());
    *w += "] = [";
    for &index in &block_state {
        *w += ib.format(index);
        *w += ", ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    *w += "const PROPS: [&'static [";
    *w += reprkv.to_int();
    *w += "]; ";
    *w += ib.format(block_state_properties.len() + 1);
    *w += "] = [\n&[],\n";
    for prop in &block_state_properties {
        *w += "&[";
        for &x in &**prop {
            *w += ib.format(x);
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

    *w += "#[inline]\n";
    *w += "pub const fn state_index(self) -> ";
    *w += bsrepr.to_int();
    *w += " {\n";
    *w += "unsafe { *Self::OFFSET.as_ptr().add(self as usize) }\n";
    *w += "}\n";
    *w += "#[inline]\n";
    *w += "pub const fn props(self) -> &'static [";
    *w += namekv;
    *w += "] {\n";
    *w += "let i = unsafe { *Self::PROPS_INDEX.as_ptr().add(self as usize) };\n";
    *w += "unsafe { *Self::PROPS.as_ptr().add(i as usize).cast() }\n";
    *w += "}\n";
    *w += "}\n";

    struct_head(w, bsrepr, bsname);
    *w += "impl ";
    *w += bsname;
    *w += " {\n";
    *w += "pub const AIR: Self = block::air.state_default();\n";
    *w += "pub const MAX: usize = ";
    *w += ib.format(bssize - 1);
    *w += ";\n";
    *w += "#[inline]\n";
    *w += "pub const fn new(n: ";
    *w += bsrepr.to_int();
    *w += ") -> Self {\n";
    *w += "debug_assert!(n <= Self::MAX as ";
    *w += bsrepr.to_int();
    *w += ");\n";
    *w += "Self(n)\n";
    *w += "}\n";
    *w += "#[inline]\n";
    *w += "pub const fn id(self) -> ";
    *w += bsrepr.to_int();
    *w += " {\n";
    *w += "self.0\n";
    *w += "}\n";
    *w += "}\n";

    *w += "unsafe impl ::mser::Write for ";
    *w += bsname;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "unsafe fn sz(&self) -> usize {\n";
    if bssize <= V7MAX {
        *w += "1usize";
    } else if bssize <= V21MAX {
        *w += "::mser::V21(self.0 as u32).sz()";
    } else {
        *w += "::mser::V32(self.0 as u32).sz()";
    }
    *w += "\n}\n";
    *w += "#[inline]\n";
    *w += "unsafe fn write(&self, w: &mut ::mser::UnsafeWriter) {\n";
    if bssize <= V7MAX {
        *w += "w.write_byte(self.0 as u8);";
    } else if bssize <= V21MAX {
        *w += "::mser::Write::write(&::mser::V21(self.0 as u32), w);";
    } else {
        *w += "::mser::Write::write(&::mser::V32(self.0 as u32), w);";
    }
    *w += "\n}\n}\n";

    let reprblock = Repr::new(offsets.len());
    *w += "const BLOCK_STATE_TO_BLOCK: *const [u8; ";
    if reprblock == Repr::U16 {
        *w += "2";
    } else {
        *w += "4";
    }
    *w += "] = ";
    *w += "unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";

    if reprblock == Repr::U16 {
        for (index, &offset) in block_state.iter().enumerate() {
            for _ in 0..properties_size[offset as usize] {
                wn.extend((index as u16).to_le_bytes());
            }
        }
    } else {
        for (index, &offset) in block_state.iter().enumerate() {
            for _ in 0..properties_size[offset as usize] {
                wn.extend((index as u32).to_le_bytes());
            }
        }
    }

    let (name, size, _) = head(iter.next());
    assert_eq!(name, "block_to_default_block_state");
    assert_eq!(size, block_names.len());

    *w += "impl block {\n";
    *w += "const DEFAULT: [";
    *w += bsrepr.to_int();
    *w += "; ";
    *w += ib.format(block_names.len());
    *w += "] = [";
    let mut prev = 0u32;
    for x in read_rl(size, &mut iter) {
        *w += ib.format(x + prev);
        prev += 1 + x;
        *w += ", ";
    }
    w.pop();
    w.pop();
    *w += "];\n";
    *w += "#[inline]\n";
    *w += "pub const fn state_default(self) -> block_state {\n";
    *w += "unsafe { block_state(*Self::DEFAULT.as_ptr().add(self as usize)) }\n";
    *w += "}\n";
    *w += "}\n";
    let (name, size, _) = head(iter.next());
    assert_eq!(name, "block_item_to_block");

    *w += "const ITEM: [raw_block; item::MAX + 1] = [";
    prev = 0;
    for x in read_rl(size, &mut iter) {
        *w += ib.format(x.wrapping_add(prev));
        prev = 1 + x.wrapping_add(prev);
        *w += ", ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    let (name, size, _) = head(iter.next());
    assert_eq!(name, "float32_table");

    let mut f32t = Vec::with_capacity(size);
    for _ in 0..size {
        let (x, _) = parse_hex::<u32>(iter.next().unwrap().as_bytes());
        f32t.push(x);
    }
    let (name, size, _) = head(iter.next());
    assert_eq!(name, "float64_table");

    let mut f64t = Vec::with_capacity(size);
    for _ in 0..size {
        let (x, _) = parse_hex::<u64>(iter.next().unwrap().as_bytes());
        let x = f64::from_be_bytes(x.to_be_bytes());
        f64t.push(x);
    }
    let (name, size, shape_repr) = head(iter.next());
    assert_eq!(name, "shape_table");

    *w += "const SHAPES: [&[[f64; 6]]; ";
    *w += ib.format(size);
    *w += "] = [\n";
    let mut shape = Vec::new();
    let mut rb = ryu::Buffer::new();
    for _ in 0..size {
        let mut s = iter.next().unwrap().as_bytes();
        loop {
            let (x, y) = parse_hex::<u32>(s);
            if y == 0 {
                break;
            }
            s = &s[y..];
            if let [b' ', rest @ ..] = s {
                s = rest;
            }
            shape.push(f64t[x as usize]);
        }
        *w += "&[";
        let mut first2 = true;
        for x in shape.chunks_exact(6) {
            if !first2 {
                *w += ", ";
            }
            first2 = false;
            *w += "[";
            let mut first = true;
            for &x in x {
                if !first {
                    *w += ", ";
                }
                first = false;
                *w += rb.format(x);
            }
            *w += "]";
        }

        *w += "],\n";
        shape.clear();
    }
    *w += "];\n";
    let (name, size, _) = head(iter.next());
    assert_eq!(name, "block_settings_table#hardness blast_resistance slipperiness velocity_multiplier jump_velocity_multiplier");

    let mut bsettings = Vec::with_capacity(size);
    for _ in 0..size {
        let mut s = iter.next().unwrap().as_bytes();
        let mut settings = [0_u32; 5];
        for s1 in &mut settings {
            let (x, y) = parse_hex::<u32>(s);
            s = &s[y..];
            if let [b' ', rest @ ..] = s {
                s = rest;
            }
            *s1 = f32t[x as usize];
        }
        bsettings.push(settings);
    }
    *w += "const BLOCK_SETTINGS: [";
    *w += "[f32; 5]";
    *w += "; ";
    *w += ib.format(size);
    *w += "] = [";
    for &x in bsettings.iter() {
        *w += "[";
        for x in x {
            *w += rb.format(f32::from_bits(x));
            *w += ", ";
        }
        w.pop();
        w.pop();
        *w += "], ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    let repr = Repr::new(size);
    let (name, size, _) = head(iter.next());
    assert_eq!(name, "block_settings");

    *w += "const BLOCK_SETTINGS_INDEX: [";
    *w += repr.to_int();
    *w += "; ";
    *w += ib.format(block_names.len());
    *w += "] = [";
    for x in read_rl(size, &mut iter) {
        *w += ib.format(x);
        *w += ", ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    let (name, size, _) = head(iter.next());
    assert_eq!(name, "block_state_flags#(has_sided_transparency lava_ignitable material_replaceable opaque tool_required exceeds_cube redstone_power_source has_comparator_output)");

    *w += "const BLOCK_STATE_FLAGS: *const u8 = ";
    *w += "unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    wn.extend(read_rl(size, &mut iter).map(|x| x as u8));

    let (name, size, _) = head(iter.next());
    assert_eq!(name, "block_state_luminance");

    *w += "const BLOCK_STATE_LUMINANCE: *const u8 = ";
    *w += "unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    wn.extend(read_rl(size, &mut iter).map(|x| x as u8));

    let (name, size, repr) = head(iter.next());
    assert_eq!(name, "block_state_static_bounds_table#(opacity(4) solid_block translucent full_cube opaque_full_cube) side_solid_full side_solid_center side_solid_rigid collision_shape culling_shape");

    assert_eq!(shape_repr, Repr::U16);
    *w += "const BLOCK_STATE_BOUNDS: *const u8 = ";
    *w += "unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";

    wn.reserve(8 * size);
    for _ in 0..size {
        let mut s = iter.next().unwrap().as_bytes();
        if s.is_empty() {
            wn.extend([0u8; 8]);
            continue;
        }
        let (x, y) = parse_hex::<u8>(s);
        wn.push(x);
        s = &s[y + 1..];
        let (x, y) = parse_hex::<u8>(s);
        wn.push(x);
        s = &s[y + 1..];
        let (x, y) = parse_hex::<u8>(s);
        wn.push(x);
        s = &s[y + 1..];
        let (x, y) = parse_hex::<u8>(s);
        wn.push(x);
        s = &s[y + 1..];
        let (x, y) = parse_hex::<u16>(s);
        wn.extend(x.to_le_bytes());
        s = &s[y + 1..];
        let (x, _) = parse_hex::<u16>(s);
        wn.extend(x.to_le_bytes());
    }

    let (name, size, _) = head(iter.next());
    assert_eq!(name, "block_state_static_bounds_map");

    *w += "const BLOCK_BOUNDS: [&[";
    *w += repr.to_int();
    *w += "]; ";
    *w += ib.format(size);
    *w += "] = [\n";
    for x in (&mut iter)
        .take(size)
        .map(|arr| arr.split(' ').map(|x| parse_hex::<u32>(x.as_bytes()).0))
    {
        *w += "&[";
        let mut first = true;
        for x in x {
            if !first {
                *w += ", ";
            }
            first = false;
            *w += ib.format(x);
        }
        *w += "],\n";
    }
    *w += "];\n";

    let (name, size, repr) = head(iter.next());
    assert_eq!(name, "block_state_static_bounds");
    *w += "const BLOCK_STATE_BOUNDS_INDEX: *const ";
    *w += repr.to_arr();
    *w += " = unsafe { NAMES.as_ptr().add(";
    *w += ib.format(wn.len());
    *w += ").cast() };\n";
    for n in read_rl(size, &mut iter) {
        match repr {
            Repr::U32 => wn.extend(n.to_le_bytes()),
            Repr::U16 => wn.extend((n as u16).to_le_bytes()),
            Repr::U8 => wn.extend((n as u8).to_le_bytes()),
        }
    }

    bsrepr
}

fn item(w: &mut String, data: &str) {
    let mut ib = itoa::Buffer::new();
    let mut iter = data.split('\n');
    let (name, size, _) = head(iter.next());
    assert_eq!(name, "item_max_count");

    *w += "const ITEM_MAX_COUNT: [u8; ";
    *w += ib.format(size);
    *w += "] = [";
    let mut x = size;
    loop {
        if x == 0 {
            break;
        }
        let next = iter.next().unwrap().as_bytes();
        let (n, count) = match next.first().copied() {
            Some(b'~') => {
                let (a, b) = parse_hex::<u32>(&next[1..]);
                let next = &next[b + 2..];
                let n = parse_hex::<u8>(next);
                (n.0, a as usize)
            }
            _ => {
                let n = parse_hex::<u8>(next);
                (n.0, 1)
            }
        };
        let n = ib.format(n);
        for _ in 0..count {
            *w += n;
            *w += ", ";
            x -= 1;
        }
    }
    w.pop();
    w.pop();
    *w += "];\n";
}

fn entity(w: &mut String, wn: &mut Vec<u8>, data: &str) {
    let mut ib = itoa::Buffer::new();
    let mut rb = ryu::Buffer::new();
    let mut iter = data.split('\n');

    let (name, size, _) = head(iter.next());
    wn.reserve(size);
    if !name.starts_with("entity_type_height") {
        panic!("{name}");
    }
    *w += "const ENTITY_HEIGHT: [f32; ";
    *w += ib.format(size);
    *w += "] = [";

    for n in read_rl(size, &mut iter) {
        *w += rb.format(f32::from_bits(n));
        *w += ", ";
    }
    w.pop();
    w.pop();
    *w += "];\n";

    let (name, size, _) = head(iter.next());
    wn.reserve(size);
    if !name.starts_with("entity_type_width") {
        panic!("{name}");
    }
    *w += "const ENTITY_WIDTH: [f32; ";
    *w += ib.format(size);
    *w += "] = [";

    for n in read_rl(size, &mut iter) {
        *w += rb.format(f32::from_bits(n));
        *w += ", ";
    }
    w.pop();
    w.pop();

    *w += "];\n";

    let (name, size, _) = head(iter.next());
    wn.reserve(size);
    if !name.starts_with("entity_type_fixed") {
        panic!("{name}");
    }
    *w += "const ENTITY_FIXED: [bool; ";
    *w += ib.format(size);
    *w += "] = [";

    w.extend(read_rl(size, &mut iter).map(|x| if x == 1 { "true, " } else { "false, " }));
    w.pop();
    w.pop();

    *w += "];\n";
}

fn gen_enum(zhash: &[&str], size: usize, w: &mut String, repr: Repr, name: &str) {
    enum_head(w, repr, name);
    if name == "sound_event" || name == "attribute" {
        for &data in zhash {
            let data = data.replace('.', "_");
            *w += &data;
            *w += ",\n";
        }
    } else {
        for &data in zhash {
            if let "match" | "true" | "false" | "type" = data {
                *w += "r#"
            }
            *w += data;
            *w += ",\n";
        }
    }
    enum_foot(w, repr, name);
    *w += "unsafe impl ::mser::Write for ";
    *w += name;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "unsafe fn sz(&self) -> usize {\n";
    if size <= V7MAX {
        *w += "1usize";
    } else if size <= V21MAX {
        *w += "::mser::V21(*self as u32).sz()";
    } else {
        *w += "::mser::V32(*self as u32).sz()";
    }
    *w += "\n}\n";
    *w += "#[inline]\n";
    *w += "unsafe fn write(&self, w: &mut ::mser::UnsafeWriter) {\n";
    if size <= V7MAX {
        *w += "w.write_byte(*self as u8);";
    } else if size <= V21MAX {
        *w += "::mser::Write::write(&::mser::V21(*self as u32), w);";
    } else {
        *w += "::mser::Write::write(&::mser::V32(*self as u32), w);";
    }
    *w += "\n}\n}\n";
    *w += "impl ::mser::Read for ";
    *w += name;
    *w += " {\n";
    *w += "#[inline]\n";
    *w += "fn read(n: &mut &[u8]) -> Option<Self> {\n";
    if size <= V7MAX {
        *w += "let x = ::mser::Bytes::u8(n)?;\n";
    } else {
        *w += "let x = ::mser::Bytes::v32(n)?;\n";
        *w += "let x = x as ";
        *w += repr.to_int();
        *w += ";\n";
    }
    *w += "if ::mser::unlikely(x > Self::MAX as ";
    *w += repr.to_int();
    *w += ") {\n";
    *w += "None\n";
    *w += "} else {\n";
    *w += "Some(unsafe { ::core::mem::transmute::<";
    *w += repr.to_int();
    *w += ", Self>";
    *w += "(x) }) }\n";
    *w += "}\n";
    *w += "}\n";
}

fn head(raw: Option<&str>) -> (&str, usize, Repr) {
    let raw = raw.expect("EOF");
    let (x, first) = raw.split_at(1);
    if x != ";" {
        panic!("invalid head: {raw}");
    }
    let (name, size) = first.split_once(';').unwrap();
    let (size, _) = parse_hex::<u32>(size.as_bytes());
    let size = size as usize;
    (name, size, Repr::new(size))
}

fn namemap(w: &mut String, g: &mut GenerateHash, w2: &mut Vec<u8>, repr: Repr, names: &[&str]) {
    let mut ib = itoa::Buffer::new();

    *w += "const M: crate::NameMap<";
    *w += repr.to_int();
    *w += "> = crate::NameMap { key: [";

    let state = g.generate_hash(names.iter());

    let mut flag = true;
    for x in state.key.0 {
        if !flag {
            *w += ", ";
        }
        flag = false;
        *w += ib.format(x);
    }
    *w += "], disps: &[";

    for &(x, y) in state.disps {
        *w += "(";
        *w += ib.format(x);
        *w += ", ";
        *w += ib.format(y);
        *w += "), ";
    }
    if !state.disps.is_empty() {
        w.pop();
        w.pop();
    }
    *w += "], names: Self::N, vals: &[";
    for &ele in state.map {
        *w += ib.format(ele.unwrap());
        *w += ", ";
    }
    if !state.map.is_empty() {
        w.pop();
        w.pop();
    }
    *w += "] };\n";

    let start = w2.len();
    w2.reserve(names.len() * 16);
    let mut offset = names.len() * 4;
    for val in names {
        w2.extend(u32::try_from(offset).unwrap().to_le_bytes());
        offset += val.len() + 2;
    }
    for val in names {
        w2.extend(u16::try_from(val.len()).unwrap().to_le_bytes());
        w2.extend(val.as_bytes());
    }
    *w += "const N: *const u8 = ";
    if start != 0 {
        *w += "unsafe { NAMES.as_ptr().add(";
        *w += ib.format(start);
        *w += ") }"
    } else {
        *w += "NAMES.as_ptr()";
    }
    *w += ";\n";
}

fn enum_head(w: &mut String, repr: Repr, name: &str) {
    if !name.starts_with('_') && name != "biome" && name != "dimension" {
        *w += "pub type raw_";
        *w += name;
        *w += " = ";
        *w += repr.to_int();
        *w += ";\n";
    }
    *w += "#[derive(Clone, Copy, PartialEq, Eq, Hash)]\n";
    *w += "#[repr(";
    *w += repr.to_int();
    *w += ")]\n#[must_use]\n";
    *w += "pub enum ";
    *w += name;
    *w += " {\n";
}

fn enum_foot(w: &mut String, repr: Repr, name: &str) {
    *w += "}\nimpl ";
    *w += name;
    *w += " {
#[inline]
pub const fn id(self) -> ";
    *w += repr.to_int();
    *w += " {\nself as ";
    *w += repr.to_int();
    *w += "\n}\n";
    *w += "#[inline]\n";
    *w += "pub const fn new(x: ";
    *w += repr.to_int();
    *w += ") -> Self {\n";
    *w += "debug_assert!(x <= Self::MAX as ";
    *w += repr.to_int();
    *w += ");\n";
    *w += "unsafe { ::core::mem::transmute::<";
    *w += repr.to_int();
    *w += ", Self>(x) }\n";
    *w += "}\n";
    *w += "}\n";
}

fn struct_head(w: &mut String, repr: Repr, name: &str) {
    if !name.starts_with('_') {
        *w += "pub type raw_";
        *w += name;
        *w += " = ";
        *w += repr.to_int();
        *w += ";\n";
    }
    *w += "#[derive(Clone, Copy, PartialEq, Eq, Hash)]\n";
    *w += "#[repr(transparent)]\n#[must_use]\n";
    *w += "pub struct ";
    *w += name;
    *w += "(";
    *w += repr.to_int();
    *w += ");\n";
}

fn gen_max(w: &mut String, slice: &[&str]) {
    *w += "pub const MAX: usize = ";
    *w += itoa::Buffer::new().format(slice.len() - 1);
    *w += ";\n";
}

fn impl_name(w: &mut String, name: &str) {
    *w += "impl ";
    *w += name;
    *w += " {
#[inline]
#[must_use]
pub const fn name(self) -> &'static str {
unsafe {
let offset = u32::from_le_bytes(*Self::N.add(4 * self as usize).cast::<[u8; 4]>());
let len = u16::from_le_bytes(*Self::N.add(offset as usize).cast::<[u8; 2]>()) as usize;
::core::str::from_utf8_unchecked(::core::slice::from_raw_parts(Self::N.add(offset as usize + 2), len))
}
}
#[inline]
pub fn parse(name: &[u8]) -> Option<Self> {
match Self::M.get(name) {
Some(x) => unsafe { Some(::core::mem::transmute::<raw_";
    *w += name;
    *w += ", Self>(x)) },
None => None,
}
}
}
";

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

fn hex_line(x: &str) -> impl Iterator<Item = u32> + '_ {
    x.as_bytes()
        .split(|&x| x == b' ')
        .map(|x| parse_hex::<u32>(x).0)
}

struct GenerateHash {
    wy_rand: u64,
    hashes: Vec<[u64; 2]>,
    buckets: Vec<Bucket>,
    values_to_add: Vec<(usize, u32)>,
    map: Vec<Option<u32>>,
    disps: Vec<(u32, u32)>,
    try_map: Vec<u64>,
}

impl GenerateHash {
    fn new() -> Self {
        Self {
            wy_rand: 0xE3D172B05F73CBC3u64
                ^ SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64,
            hashes: Vec::new(),
            buckets: Vec::new(),
            values_to_add: Vec::new(),
            map: Vec::new(),
            try_map: Vec::new(),
            disps: Vec::new(),
        }
    }

    fn generate_hash(
        &mut self,
        entries: impl Iterator<Item = impl AsRef<str>> + Clone,
    ) -> HashState {
        'key: loop {
            let key = highway::Key([self.nx(), self.nx(), self.nx(), self.nx()]);
            let hasher = highway::HighwayHasher::new(key);
            self.hashes.clear();
            self.hashes.extend(entries.clone().map(|entry| {
                highway::HighwayHash::hash128(hasher.clone(), entry.as_ref().as_bytes())
            }));

            let buckets_len = self.hashes.len().div_ceil(DEFAULT_LAMBDA);
            let table_len = self.hashes.len();

            self.buckets.truncate(buckets_len);
            for (i, bucket) in self.buckets.iter_mut().enumerate() {
                bucket.idx = i;
                bucket.keys.clear();
            }
            if self.buckets.len() < buckets_len {
                self.buckets
                    .extend((self.buckets.len()..buckets_len).map(|i| Bucket {
                        idx: i,
                        keys: Vec::new(),
                    }));
            }

            for (i, [hash, _]) in self.hashes.iter().enumerate() {
                self.buckets[((hash >> 32) as u32 % buckets_len as u32) as usize]
                    .keys
                    .push(i as u32);
            }

            self.buckets
                .sort_unstable_by(|a, b| a.keys.len().cmp(&b.keys.len()).reverse());

            self.map.clear();
            self.map.extend(core::iter::repeat(None).take(table_len));

            self.disps.clear();
            self.disps
                .extend(core::iter::repeat((0u32, 0u32)).take(buckets_len));

            self.try_map.clear();
            self.try_map
                .extend(core::iter::repeat(0u64).take(table_len));

            let mut generation = 0u64;

            'buckets: for bucket in &*self.buckets {
                for d1 in 0..(table_len as u32) {
                    'disps: for d2 in 0..(table_len as u32) {
                        self.values_to_add.clear();
                        generation += 1;

                        for &key in &bucket.keys {
                            let [a, b] = (&mut self.hashes)[key as usize];
                            let f1 = a as u32;
                            let f2 = b as u32;

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
    fn nx(&mut self) -> u64 {
        self.wy_rand = self.wy_rand.wrapping_add(0xa0761d6478bd642f);
        let x = (self.wy_rand ^ 0xe7037ed1a0b428db) as u128;
        let t = (self.wy_rand as u128).wrapping_mul(x);
        (t.wrapping_shr(64) ^ t) as u64
    }
}

const DEFAULT_LAMBDA: usize = 5;

struct HashState<'a> {
    key: highway::Key,
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
            let next = self.i.next()?.as_bytes();
            self.take -= 1;
            match next.first().copied() {
                Some(b'~') => {
                    let (a, b) = parse_hex::<u32>(&next[1..]);
                    let next = &next[b + 2..];
                    let (ctx, _) = parse_hex::<u32>(next);
                    self.count = a as usize - 1;
                    self.prev = ctx;
                    Some(ctx)
                }
                _ => Some(parse_hex::<u32>(next).0),
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
