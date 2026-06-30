use core::iter::repeat_n;
use formatting::Context;
use std::collections::{HashMap, hash_map};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::{char, cmp};

macro_rules! w {
    ($ctxt: expr, $($tt: tt)*) => {
        (write!($ctxt.out, $($tt)*)).unwrap()
    }
}

mod formatting;
mod phf;
mod trie;
mod util;

/// [UnicodeData.txt] contains Unicode Character Data
///
/// [UnicodeData.txt]: https://www.unicode.org/Public/16.0.0/ucd/UnicodeData.txt
const UNICODE_DATA: &str = include_str!("unicode/UnicodeData.txt");
/// Unicode aliases
///
/// [NamesList.txt] contents contains a map of unicode aliases to their
/// corresponding values.
///
/// [NamesList.txt]: https://www.unicode.org/Public/16.0.0/ucd/NameAliases.txt
const NAME_ALIASES: &str = include_str!("unicode/NameAliases.txt");

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    generate(UNICODE_DATA, &out_dir.join("generated.rs"), None);
    generate_phf(UNICODE_DATA, &out_dir.join("generated_phf.rs"), None, 3, 2);
    generate_aliases(NAME_ALIASES, &out_dir.join("generated_alias.rs"));
}

const SPLITTERS: &[u8] = b"-";

struct TableData {
    codepoint_names: Vec<(char, &'static str)>,
    cjk_ideograph_ranges: Vec<(char, char)>,
}

fn get_table_data(unicode_data: &'static str) -> TableData {
    fn extract(line: &'static str) -> Option<(char, &'static str)> {
        let splits: Vec<_> = line.splitn(15, ';').collect();
        assert_eq!(splits.len(), 15);
        let s = splits[0];
        let cp = u32::from_str_radix(s, 16)
            .ok()
            .unwrap_or_else(|| panic!("invalid {}", line));
        let c = char::from_u32(cp)?;
        let name = splits[1];
        Some((c, name))
    }

    let mut iter = unicode_data.lines();

    let mut codepoint_names = vec![];
    let mut cjk_ideograph_ranges = vec![];

    while let Some(l) = iter.next() {
        if l.is_empty() {
            break;
        }
        let (cp, name) = if let Some(extracted) = extract(l.trim()) {
            extracted
        } else {
            continue;
        };
        if name.starts_with('<') {
            assert!(name.ends_with('>'), "should >: {}", name);
            let name = &name[1..name.len() - 1];
            if name.starts_with("CJK Ideograph") {
                assert!(name.ends_with("First"));
                // should be CJK Ideograph ..., Last
                let line2 = iter.next().expect("unclosed ideograph range");
                let (cp2, name2) = if let Some(extracted) = extract(line2.trim()) {
                    extracted
                } else {
                    continue;
                };
                assert_eq!(&*name.replace("First", "Last"), &name2[1..name2.len() - 1]);

                cjk_ideograph_ranges.push((cp, cp2));
            } else if name.starts_with("Hangul Syllable") {
                // the main lib only knows this range, so lets make
                // sure we're not going out of date wrt the unicode
                // standard.
                if name.ends_with("First") {
                    assert_eq!(cp, '\u{AC00}');
                } else if name.ends_with("Last") {
                    assert_eq!(cp, '\u{D7A3}');
                } else {
                    panic!("unknown hangul syllable {}", name)
                }
            }
        } else {
            codepoint_names.push((cp, name))
        }
    }
    TableData {
        codepoint_names,
        cjk_ideograph_ranges,
    }
}

pub struct Alias {
    pub code: &'static str,
    pub alias: &'static str,
    pub category: &'static str,
}

pub fn get_aliases(name_aliases: &'static str) -> Vec<Alias> {
    let mut aliases = Vec::new();
    for line in name_aliases.lines() {
        if line.is_empty() | line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, ';');
        let code = parts.next().expect(line);
        let alias = parts.next().expect(code);
        let category = parts.next().expect(alias);
        aliases.push(Alias {
            code,
            alias,
            category,
        });
    }
    aliases
}

fn write_cjk_ideograph_ranges(ctxt: &mut Context, ranges: &[(char, char)]) {
    ctxt.write_debugs("CJK_IDEOGRAPH_RANGES", "(char, char)", ranges)
}

/// Construct a huge string storing the text data, and return it,
/// along with information about the position and frequency of the
/// constituent words of the input.
fn create_lexicon_and_offsets(
    mut codepoint_names: Vec<(char, &str)>,
) -> (String, Vec<(usize, Vec<u8>, usize)>) {
    codepoint_names.sort_by(|a, b| a.1.len().cmp(&b.1.len()).reverse());

    // a trie of all the suffixes of the data,
    let mut t = trie::Trie::new();
    let mut output = String::new();

    for &(_, name) in codepoint_names.iter() {
        for n in util::split(name, SPLITTERS) {
            if n.len() == 1 && SPLITTERS.contains(&n.as_bytes()[0]) {
                continue;
            }

            let (already, _previous_was_exact) = t.insert(n.bytes(), None, false);
            if already {
            } else {
                // completely new element, i.e. not a substring of
                // anything, so record its position & add it.
                let offset = output.len();
                t.set_offset(n.bytes(), offset);
                output.push_str(n);

                // insert the suffixes of this word which saves about
                // 10KB (we could theoretically insert all substrings,
                // upto a certain length, but this only saves ~300
                // bytes or so and is noticeably slower).
                for i in 1..n.len() {
                    if t.insert(n[i..].bytes(), Some(offset + i), true).0 {
                        // once we've found a string that's already
                        // been inserted, we know all suffixes will've
                        // been inserted too.
                        break;
                    }
                }
            }
        }
    }
    let words: Vec<_> = t
        .iter()
        .map(|(a, b, c)| (a, b, c.expect("unset offset?")))
        .collect();
    (output, words)
}

// creates arrays t1, t2 and a shift such that `dat[i] == t2[t1[i >>
// shift] << shift + i & mask]`; this allows us to share blocks of
// length `1 << shift`, and so compress an array with a lot of repeats
// (like the 0's of the phrasebook_offsets below).
fn bin_data(dat: &[u32]) -> (Vec<u32>, Vec<u32>, usize) {
    let mut smallest = 0xFFFFFFFF;
    let mut data = (vec![], vec![], 0);
    let mut cache = HashMap::new();

    // brute force search for the shift that words best.
    for shift in 0..14 {
        cache.clear();

        let mut t1 = vec![];
        let mut t2 = vec![];
        for chunk in dat.chunks(1 << shift) {
            // have we stored this chunk already?
            let index = *match cache.entry(chunk) {
                hash_map::Entry::Occupied(o) => o.into_mut(),
                hash_map::Entry::Vacant(v) => {
                    // no :(, better put it in.
                    let index = t2.len();
                    t2.extend(chunk.iter().cloned());
                    v.insert(index)
                }
            };
            t1.push((index >> shift) as u32)
        }

        let my_size = t1.len() * util::smallest_type(t1.iter().copied())
            + t2.len() * util::smallest_type(t2.iter().copied());
        if my_size < smallest {
            data = (t1, t2, shift);
            smallest = my_size
        }
    }

    // verify.
    {
        let (ref t1, ref t2, shift) = data;
        let mask = (1 << shift) - 1;
        for (i, &elem) in dat.iter().enumerate() {
            assert_eq!(
                elem,
                t2[((t1[i >> shift] << shift) + (i as u32 & mask)) as usize]
            )
        }
    }

    data
}

fn write_codepoint_maps(ctxt: &mut Context, codepoint_names: Vec<(char, &str)>) {
    let (lexicon_string, mut lexicon_words) = create_lexicon_and_offsets(codepoint_names.clone());

    let num_escapes = lexicon_words.len().div_ceil(256);

    // we reserve the high bit (end of word) and 127,126... for
    // non-space splits.  The high bit saves about 10KB, and doing the
    // extra splits reduces the space required even more (e.g. - is a
    // reduction of 14KB).
    let short = 128 - SPLITTERS.len() - num_escapes;

    // find the `short` most common elements
    lexicon_words.sort_by(|a, b| a.cmp(b).reverse());

    // and then sort the rest into groups of equal length, to allow us
    // to avoid storing the full length table; just the indices. The
    // ordering is irrelevant here; just that they are in groups.
    lexicon_words[short..].sort_by_key(|(_, a, _)| a.len());

    // the encoding for each word, to avoid having to recompute it
    // each time, we can just blit it out of here.
    let mut word_encodings = HashMap::new();
    for (i, x) in SPLITTERS.iter().enumerate() {
        // precomputed
        word_encodings.insert(vec![*x], vec![128 - 1 - i as u32]);
    }

    // the indices into the main string
    let mut lexicon_offsets = vec![];
    // and their lengths, for the most common strings, since these
    // have no information about their length (they were chosen by
    // frequency).
    let mut lexicon_short_lengths = vec![];
    let mut iter = lexicon_words.into_iter().enumerate();

    for (i, (_, word, offset)) in iter.by_ref().take(short) {
        lexicon_offsets.push(offset);
        lexicon_short_lengths.push(word.len());
        // encoded as a single byte.
        assert!(word_encodings.insert(word, vec![i as u32]).is_none())
    }

    // this stores (end point, length) for each block of words of a
    // given length, where `end point` is one-past-the-end.
    let mut lexicon_ordered_lengths = vec![];
    let mut previous_len = 0xFFFF;
    for (i, (_, word, offset)) in iter {
        let (hi, lo) = (short + i / 256, i % 256);
        assert!(short <= hi && hi < 128 - SPLITTERS.len());
        lexicon_offsets.push(offset);
        let len = word.len();
        if len != previous_len {
            if previous_len != 0xFFFF {
                lexicon_ordered_lengths.push((i, previous_len));
            }
            previous_len = len;
        }

        assert!(
            word_encodings
                .insert(word, vec![hi as u32, lo as u32])
                .is_none()
        );
    }
    // don't forget the last one.
    lexicon_ordered_lengths.push((lexicon_offsets.len(), previous_len));

    // phrasebook encodes the words out of the lexicon that make each
    // codepoint name.
    let mut phrasebook = vec![0u32];
    // this is a map from `char` -> the index in phrasebook. it is
    // currently huge, but it has a lot of 0's, so we compress it
    // using the binning, below.
    let mut phrasebook_offsets = std::iter::repeat_n(0, 0x10FFFF + 1).collect::<Vec<_>>();
    let mut longest_name = String::new();
    for &(cp, name) in codepoint_names.iter() {
        longest_name = cmp::max_by_key(normalise_name(name, cp), longest_name, |s| s.len());

        let start = phrasebook.len() as u32;
        phrasebook_offsets[cp as usize] = start;

        let mut last_len = 0;
        for w in util::split(name, SPLITTERS) {
            let data = word_encodings
                .get(w.as_bytes())
                .expect(concat!("option on ", line!()));
            last_len = data.len();
            // info!("{}: '{}' {}", name, w, data);

            // blit the data.
            phrasebook.extend(data.iter().cloned())
        }

        // add the high bit to the first byte of the last encoded
        // phrase, to indicate the end.
        let idx = phrasebook.len() - last_len;
        phrasebook[idx] |= 0b1000_0000;
    }

    // compress the offsets, hopefully collapsing all the 0's.
    let (t1, t2, shift) = bin_data(&phrasebook_offsets);

    w!(
        ctxt,
        "pub const LONGEST_NAME: &str = {longest_name:?};\n\
        pub const LONGEST_NAME_LEN: usize = LONGEST_NAME.len();\n"
    );
    ctxt.write_plain_string("LEXICON", &lexicon_string);
    ctxt.write_debugs("LEXICON_OFFSETS", "u32", &lexicon_offsets);
    ctxt.write_debugs("LEXICON_SHORT_LENGTHS", "u8", &lexicon_short_lengths);
    w!(
        ctxt,
        "pub const LEXICON_ORDERED_LENGTHS_LEN: usize = {};\n",
        lexicon_ordered_lengths.len()
    );
    ctxt.write_debugs(
        "LEXICON_ORDERED_LENGTHS",
        "(usize, u8)",
        &lexicon_ordered_lengths,
    );
    w!(ctxt, "pub const PHRASEBOOK_SHORT: u8 = {};\n", short);
    ctxt.write_debugs("PHRASEBOOK", "u8", &phrasebook);
    w!(
        ctxt,
        "pub const PHRASEBOOK_OFFSET_SHIFT: usize = {};\n",
        shift
    );
    ctxt.write_debugs(
        "PHRASEBOOK_OFFSETS1",
        &util::smallest_u(t1.iter().copied()),
        &t1,
    );
    ctxt.write_debugs(
        "PHRASEBOOK_OFFSETS2",
        &util::smallest_u(t2.iter().copied()),
        &t2,
    );
}

fn make_context() -> Context {
    Context {
        out: Vec::with_capacity(4096),
    }
}

#[allow(clippy::type_complexity)]
fn get_truncated_table_data(
    unicode_data: &'static str,
    truncate: Option<usize>,
) -> (Vec<(char, &'static str)>, Vec<(char, char)>) {
    let TableData {
        mut codepoint_names,
        cjk_ideograph_ranges: cjk,
    } = get_table_data(unicode_data);
    if let Some(n) = truncate {
        codepoint_names.truncate(n)
    }
    (codepoint_names, cjk)
}

pub fn generate_phf(
    unicode_data: &'static str,
    path: &Path,
    truncate: Option<usize>,
    lambda: usize,
    tries: usize,
) {
    let (codepoint_names, _) = get_truncated_table_data(unicode_data, truncate);

    let codepoint_names: Vec<_> = codepoint_names
        .into_iter()
        .map(|(c, s)| (c, normalise_name(s, c)))
        .collect();

    let mut ctxt = make_context();
    let (n, disps, data) = phf::create_phf(&codepoint_names, lambda, tries);

    w!(ctxt, "pub const NAME2CODE_N: u64 = {};\n", n);
    ctxt.write_debugs("NAME2CODE_DISP", "(u16, u16)", &disps);

    ctxt.write_debugs("NAME2CODE_CODE", "char", &data);

    std::fs::write(path, ctxt.out).unwrap()
}

/// Convert a Unicode name to a form that can be used for loose matching, as per
/// [UAX#44](https://www.unicode.org/reports/tr44/tr44-34.html#Matching_Names)
///
/// This function matches `unicode_names2::normalise_name` in implementation,
/// thus the result of one can be used to query a PHF generated from the other.
fn normalise_name(s: &str, codepoint: char) -> String {
    let mut normalised = String::new();
    let bytes = s.as_bytes();
    for (i, c) in bytes.iter().map(u8::to_ascii_uppercase).enumerate() {
        // "Ignore case, whitespace, underscore ('_'), [...]"
        if c.is_ascii_whitespace() || c == b'_' {
            continue;
        }

        // "[...] and all medial hyphens except the hyphen in U+1180 HANGUL JUNGSEONG
        // O-E."
        if codepoint != '\u{1180}' // HANGUL JUNGSEONG O-E
            && c == b'-'
            && bytes.get(i - 1).is_some_and(u8::is_ascii_alphanumeric)
            && bytes.get(i + 1).is_some_and(u8::is_ascii_alphanumeric)
        {
            continue;
        }
        assert!(
            c.is_ascii_alphanumeric() || c == b'-',
            "U+{:04X} contains an invalid character for a Unicode name: {:?}",
            codepoint as u32,
            s
        );

        normalised.push(c as char);
    }

    normalised
}

pub fn generate(unicode_data: &'static str, path: &Path, truncate: Option<usize>) {
    let (codepoint_names, cjk) = get_truncated_table_data(unicode_data, truncate);
    let mut ctxt = make_context();

    write_cjk_ideograph_ranges(&mut ctxt, &cjk);
    let _ = ctxt.out.write(b"\n").unwrap();
    write_codepoint_maps(&mut ctxt, codepoint_names);

    std::fs::write(path, ctxt.out).unwrap()
}

pub fn generate_aliases(name_aliases: &'static str, path: &Path) {
    use std::fmt::Write;
    let aliases = get_aliases(name_aliases);
    let entries = aliases.iter().map(|x| x.alias).collect::<Vec<_>>();
    let mut state = GenerateHash::new();
    let state = state.generate_hash(&entries);
    let mut w = String::new();
    w += "const NAMES: &[&str; ";
    write!(&mut w, "{}", entries.len()).unwrap();
    w += "] = &[\n";
    for &val in &entries {
        w += "\"";
        w += val;
        w += "\",\n";
    }
    w += "];\n";
    w += "const CODE: &[char; ";
    write!(&mut w, "{}", entries.len()).unwrap();
    w += "] = &[\n";
    for Alias { code, .. } in aliases.iter() {
        write!(&mut w, "'\\u{{{code}}}'").unwrap();
        w += ",\n";
    }
    w += "];\n";
    w += "#[allow(clippy::large_const_arrays)]\n";
    w += "const ";
    w += "DISPS";
    w += ": &[";
    w += "u64";
    w += "; ";
    write!(&mut w, "{}", state.disps.len()).unwrap();
    w += "] = ";
    let mut iter = state.disps.iter().copied();
    let first = iter.next();
    let first = match first {
        Some((h, l)) => ((h as u64) << 32) | l as u64,
        None => {
            w += "&[]";
            return;
        }
    };
    let mut c = 0usize;
    w += "&[\n";
    write!(&mut w, "{first}").unwrap();
    for (h, l) in iter {
        w.push(',');
        c += 1;
        if c == 8 {
            w.push('\n');
            c = 0;
        } else {
            w.push(' ');
        }
        write!(&mut w, "{}", ((h as u64) << 32) | l as u64).unwrap();
    }
    w.push(',');
    w.push('\n');
    w.push(']');
    w.push(';');
    w.push('\n');

    w += "#[allow(clippy::large_const_arrays)]\n";
    w += "const ";
    w += "VALS";
    w += ": &[";
    w += "u64";
    w += "; ";
    write!(&mut w, "{}", state.map.len()).unwrap();
    w += "] = ";
    let mut iter = state.map.iter().copied();
    let first = iter.next();
    let first = match first {
        Some(x) => x.unwrap(),
        None => {
            w += "&[]";
            return;
        }
    };
    let mut c = 0usize;
    w += "&[\n";
    write!(&mut w, "{first}").unwrap();
    for x in iter {
        w.push(',');
        c += 1;
        if c == 8 {
            w.push('\n');
            c = 0;
        } else {
            w.push(' ');
        }
        write!(&mut w, "{}", x.unwrap()).unwrap();
    }
    w.push(',');
    w.push('\n');
    w.push(']');
    w.push(';');
    w.push('\n');
    write!(
        &mut w,
        "fn character_by_alias(n: &[u8]) -> Option<char> {{
let mut h: u64 = {} ^ (n.len() as u64);
let mut i = 0;
while i < n.len() {{
h ^= n[i] as u64;
h = h.wrapping_mul(0x100000001b3);
i += 1;
}}
let a = h;
let g = (a >> 24) as u32;
let f1 = a as u32;
let f2 = (a >> 32) as u32;
let d = unsafe {{ *DISPS.get_unchecked((g % ({}_u32)) as usize) }};
let d1 = (d >> 32) as u32;
let d2 = d as u32;
let index = d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2);
let index1 = (index % ({}_u32)) as usize;
let v = unsafe {{ *VALS.get_unchecked(index1) }};
let k = unsafe {{ *NAMES.get_unchecked(v as usize) }};
if n == k.as_bytes() {{ unsafe {{ Some(*CODE.get_unchecked(v as usize)) }} }} else {{ None }}
}}",
        state.key,
        state.disps.len(),
        state.map.len()
    )
    .unwrap();

    std::fs::write(path, w).unwrap();
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
