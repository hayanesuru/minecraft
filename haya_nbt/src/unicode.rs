use core::{char, fmt};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));
include!(concat!(env!("OUT_DIR"), "/generated_phf.rs"));
include!(concat!(env!("OUT_DIR"), "/generated_alias.rs"));

const HANGUL_SYLLABLE_PREFIX: &str = "HANGUL SYLLABLE ";
const NORMALISED_HANGUL_SYLLABLE_PREFIX: &str = "HANGULSYLLABLE";
const CJK_UNIFIED_IDEOGRAPH_PREFIX: &str = "CJK UNIFIED IDEOGRAPH-";
const NORMALISED_CJK_UNIFIED_IDEOGRAPH_PREFIX: &str = "CJKUNIFIEDIDEOGRAPH";

fn is_cjk_unified_ideograph(ch: char) -> bool {
    CJK_IDEOGRAPH_RANGES
        .iter()
        .any(|&(lo, hi)| lo <= ch && ch <= hi)
}

/// An iterator over the components of a code point's name. Notably implements
/// `Display`.
///
/// To reconstruct the full Unicode name from this iterator, you can concatenate
/// every string slice yielded from it. Each such slice is either a word
/// matching `[A-Z0-9]*`, a space `" "`, or a hyphen `"-"`. (In particular,
/// words can be the empty string `""`).
///
/// The [size hint] returns an exact size, by cloning the iterator and iterating
/// it fully. Cloning and iteration are cheap, and all names are relatively
/// short, so this should not have a high impact.
///
/// [size hint]: std::iter::Iterator::size_hint
#[derive(Clone)]
pub struct Name {
    data: Name_,
}
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
enum Name_ {
    Plain(IterStr),
    CJK(CJK),
    Hangul(Hangul),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy)]
struct CJK {
    emit_prefix: bool,
    idx: u8,
    // the longest character is 0x10FFFF
    data: [u8; 6],
}
#[derive(Copy)]
struct Hangul {
    emit_prefix: bool,
    idx: u8,
    // stores the choseong, jungseong, jongseong syllable numbers (in
    // that order)
    data: [u8; 3],
}
impl Clone for CJK {
    fn clone(&self) -> Self {
        *self
    }
}
impl Clone for Hangul {
    fn clone(&self) -> Self {
        *self
    }
}

#[allow(clippy::len_without_is_empty)]
impl Name {
    /// The number of bytes in the name.
    ///
    /// All names are plain ASCII, so this is also the number of
    /// Unicode codepoints and the number of graphemes.
    pub fn len(&self) -> usize {
        let counted = self.clone();
        counted.fold(0, |a, s| a + s.len())
    }
}

impl Iterator for Name {
    type Item = &'static str;

    fn next(&mut self) -> Option<&'static str> {
        match self.data {
            Name_::Plain(ref mut s) => s.next(),
            Name_::CJK(ref mut state) => {
                // we're a CJK unified ideograph
                if state.emit_prefix {
                    state.emit_prefix = false;
                    return Some(CJK_UNIFIED_IDEOGRAPH_PREFIX);
                }
                // run until we've run out of array: the construction
                // of the data means this is exactly when we have
                // finished emitting the number.
                state
                    .data
                    .get(state.idx as usize)
                    // (avoid conflicting mutable borrow problems)
                    .map(|digit| *digit as usize)
                    .map(|d| {
                        state.idx += 1;
                        const DIGITS: &str = "0123456789ABCDEF";
                        &DIGITS[d..d + 1]
                    })
            }
            Name_::Hangul(ref mut state) => {
                if state.emit_prefix {
                    state.emit_prefix = false;
                    return Some(HANGUL_SYLLABLE_PREFIX);
                }

                let idx = state.idx as usize;
                state.data.get(idx).map(|x| *x as usize).map(|x| {
                    // progressively walk through the syllables
                    state.idx += 1;
                    [CHOSEONG, JUNGSEONG, JONGSEONG][idx][x]
                })
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // we can estimate exactly by just iterating and summing up.
        let counted = self.clone();
        let n = counted.count();
        (n, Some(n))
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmtr)
    }
}
impl fmt::Display for Name {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        self.clone().try_for_each(|s| fmtr.write_str(s))
    }
}

/// Find the name of `c`, or `None` if `c` has no name.
///
/// The return value is an iterator that yields `&'static str` components of the
/// name successively (including spaces and hyphens). It implements `Display`,
/// so can be used naturally to build `String`s or be printed. See also the
/// [type-level docs][Name].
///
/// # Example
///
/// ```rust
/// # use haya_nbt::{name,character};
/// assert_eq!(name('a').unwrap().to_string(), "LATIN SMALL LETTER A");
/// assert_eq!(name('\u{2605}').unwrap().to_string(), "BLACK STAR");
/// assert_eq!(name('☃').unwrap().to_string(), "SNOWMAN");
///
/// // control code
/// assert!(name('\x00').is_none());
/// // unassigned
/// assert!(name('\u{10FFFF}').is_none());
/// ```
pub fn name(c: char) -> Option<Name> {
    let cc = c as usize;
    let offset =
        (PHRASEBOOK_OFFSETS1[cc >> PHRASEBOOK_OFFSET_SHIFT] as usize) << PHRASEBOOK_OFFSET_SHIFT;

    let mask = (1 << PHRASEBOOK_OFFSET_SHIFT) - 1;
    let offset2 = PHRASEBOOK_OFFSETS2[offset + (cc & mask)];
    if offset2 == 0 {
        if is_cjk_unified_ideograph(c) {
            // write the hex number out right aligned in this array.
            let mut data = [b'0'; 6];
            let mut number = c as u32;
            let mut data_start = 6;
            for place in data.iter_mut().rev() {
                // this would be incorrect if U+0000 was CJK unified
                // ideograph, but it's not, so it's fine.
                if number == 0 {
                    break;
                }
                *place = (number % 16) as u8;
                number /= 16;
                data_start -= 1;
            }
            Some(Name {
                data: Name_::CJK(CJK {
                    emit_prefix: true,
                    idx: data_start,
                    data,
                }),
            })
        } else {
            // maybe it is a hangul syllable?
            syllable_decomposition(c).map(|(ch, ju, jo)| Name {
                data: Name_::Hangul(Hangul {
                    emit_prefix: true,
                    idx: 0,
                    data: [ch, ju, jo],
                }),
            })
        }
    } else {
        Some(Name {
            data: Name_::Plain(IterStr::new(offset2)),
        })
    }
}

fn fnv_hash<I: Iterator<Item = u8>>(x: I) -> u64 {
    let mut g = 0xcbf29ce484222325 ^ NAME2CODE_N;
    for b in x {
        g ^= b as u64;
        g = g.wrapping_mul(0x100000001b3);
    }
    g
}

fn displace(f1: u32, f2: u32, d1: u32, d2: u32) -> u32 {
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2)
}

fn split(hash: u64) -> (u32, u32, u32) {
    let bits = 21;
    let mask = (1 << bits) - 1;
    (
        (hash & mask) as u32,
        ((hash >> bits) & mask) as u32,
        ((hash >> (2 * bits)) & mask) as u32,
    )
}

/// Find the character called `name`, or `None` if no such character
/// exists.
///
/// This function uses the [UAX44-LM2] loose matching scheme for lookup. For
/// more information, see the [crate-level docs][self].
///
/// [UAX44-LM2]: https://www.unicode.org/reports/tr44/tr44-34.html#UAX44-LM2
///
/// # Example
///
/// ```rust
/// # use haya_nbt::{name,character};
///
/// assert_eq!(character("LATIN SMALL LETTER A"), Some('a'));
/// assert_eq!(character("latinsmalllettera"), Some('a'));
/// assert_eq!(character("Black_Star"), Some('★'));
/// assert_eq!(character("SNOWMAN"), Some('☃'));
/// assert_eq!(character("BACKSPACE"), Some('\x08'));
///
/// assert_eq!(character("nonsense"), None);
/// ```
pub fn character(search_name: &str) -> Option<char> {
    let original_name = search_name;
    let mut buf = [0; LONGEST_NAME_LEN];
    let len = normalise_name(search_name, &mut buf);
    let search_name = &buf[..len];

    // try `HANGUL SYLLABLE <choseong><jungseong><jongseong>`
    if let Some(rest) = search_name.strip_prefix(NORMALISED_HANGUL_SYLLABLE_PREFIX.as_bytes()) {
        let (choseong, rest1) = slice_shift_choseong(rest);
        let (jungseong, rest2) = slice_shift_jungseong(rest1);
        let (jongseong, rest3) = slice_shift_jongseong(rest2);
        match (choseong, jungseong, jongseong, rest3) {
            (Some(choseong2), Some(jungseong2), Some(jongseong2), b"") => {
                let c = 0xac00 + (choseong2 * 21 + jungseong2) * 28 + jongseong2;
                return char::from_u32(c);
            }
            (_, _, _, _) => {
                // there are no other names starting with `HANGUL SYLLABLE `
                // (verified by `generator/...`).
                return None;
            }
        }
    }

    // try `CJK UNIFIED IDEOGRAPH-<digits>`
    if let Some(remaining) =
        search_name.strip_prefix(NORMALISED_CJK_UNIFIED_IDEOGRAPH_PREFIX.as_bytes())
    {
        if remaining.len() > 5 {
            return None;
        } // avoid overflow

        let mut v = 0u32;
        for &c in remaining {
            v = match c {
                b'0'..=b'9' => (v << 4) | (c - b'0') as u32,
                b'A'..=b'F' => (v << 4) | (c - b'A' + 10) as u32,
                _ => return None,
            }
        }
        let ch = char::from_u32(v)?;

        // check if the resulting code is indeed in the known ranges
        if is_cjk_unified_ideograph(ch) {
            return Some(ch);
        } else {
            // there are no other names starting with `CJK UNIFIED IDEOGRAPH-`
            // (verified by `src/generate.py`).
            return None;
        }
    }

    // get the parts of the hash...
    let (g, f1, f2) = split(fnv_hash(search_name.iter().copied()));
    // ...and the appropriate displacements...
    let (d1, d2) = NAME2CODE_DISP[g as usize % NAME2CODE_DISP.len()];

    // ...to find the right index...
    let idx = displace(f1, f2, d1 as u32, d2 as u32) as usize;
    // ...for looking up the codepoint.
    let codepoint = NAME2CODE_CODE[idx % NAME2CODE_CODE.len()];

    // Now check that this is actually correct. Since this is a
    // perfect hash table, valid names map precisely to their code
    // point (and invalid names map to anything), so we only need to
    // check the name for this codepoint matches the input and we know
    // everything. (i.e. no need for probing)
    let maybe_name = match name(codepoint) {
        None => {
            return character_by_alias(search_name);
        }
        Some(name) => name,
    };

    // `name(codepoint)` returns an iterator yielding words separated by spaces or
    // hyphens. That means whenever a name contains a non-medial hyphen, it must
    // be emulated by inserting an artificial empty word (`""`) between the
    // space and the hyphen.
    let mut cmp_name = search_name;
    for part in maybe_name {
        let part2 = match part {
            "" => "-",       // Non-medial hyphens are preserved by `normalise_name`, check them.
            " " => continue, // Spaces and medial hyphens are removed, ignore them.
            "-" if codepoint != '\u{1180}' => continue, // But the hyphen in U+1180 is preserved.
            word => word,
        };

        if let Some(rest) = cmp_name.strip_prefix(part2.as_bytes()) {
            cmp_name = rest;
        } else {
            return character_by_alias(search_name);
        }
    }

    // "HANGUL JUNGSEONG O-E" is ambiguous, returning U+116C HANGUL JUNGSEONG OE
    // instead. All other ways of spelling U+1180 will get properly detected, so
    // it's enough to just check if the hyphen is in the right place.
    if codepoint == '\u{116C}'
        && original_name
            .trim_end_matches(|c: char| c.is_ascii_whitespace() || c == '_')
            .bytes()
            .nth_back(1)
            == Some(b'-')
    {
        return Some('\u{1180}');
    }

    Some(codepoint)
}

/// Convert a Unicode name to a form that can be used for loose matching, as per
/// [UAX#44](https://www.unicode.org/reports/tr44/tr44-34.html#Matching_Names).
///
/// This function matches `unicode_names2_generator::normalise_name` in
/// implementation, except that the special case of U+1180 HANGUL JUNGSEONG O-E
/// isn't handled here, because we don't yet know which character is being
/// queried and a string comparison would be expensive to inspect each
/// query with given it only matches for one character. Thus the case of U+1180
/// is handled at the end of [`character`].
fn normalise_name(search_name: &str, buf: &mut [u8; LONGEST_NAME_LEN]) -> usize {
    let mut cursor = 0;
    let bytes = search_name.as_bytes();

    for (i, c) in bytes.iter().map(u8::to_ascii_uppercase).enumerate() {
        // "Ignore case, whitespace, underscore ('_'), [...]"
        if c.is_ascii_whitespace() || c == b'_' {
            continue;
        }

        // "[...] and all medial hyphens except the hyphen in U+1180 HANGUL JUNGSEONG
        // O-E." See doc comment for why U+1180 isn't handled
        if c == b'-'
            && bytes.get(i - 1).is_some_and(u8::is_ascii_alphanumeric)
            && bytes.get(i + 1).is_some_and(u8::is_ascii_alphanumeric)
        {
            continue;
        }

        if !c.is_ascii_alphanumeric() && c != b'-' {
            // All unicode names comprise only of alphanumeric characters and hyphens after
            // stripping spaces and underscores. Returning 0 effectively serves as returning
            // `None`.
            return 0;
        }

        if cursor >= buf.len() {
            // No Unicode character has this long a name.
            return 0;
        }
        buf[cursor] = c;
        cursor += 1;
    }

    cursor
}

// derived from Jamo.txt
pub const CHOSEONG: &[&str] = &[
    "G", "GG", "N", "D", "DD", "R", "M", "B", "BB", "S", "SS", "", "J", "JJ", "C", "K", "T", "P",
    "H",
];
pub const JUNGSEONG: &[&str] = &[
    "A", "AE", "YA", "YAE", "EO", "E", "YEO", "YE", "O", "WA", "WAE", "OE", "YO", "U", "WEO", "WE",
    "WI", "YU", "EU", "YI", "I",
];
pub const JONGSEONG: &[&str] = &[
    "", "G", "GG", "GS", "N", "NJ", "NH", "D", "L", "LG", "LM", "LB", "LS", "LT", "LP", "LH", "M",
    "B", "BS", "S", "SS", "NG", "J", "C", "K", "T", "P", "H",
];

pub fn is_hangul_syllable(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
}

pub fn syllable_decomposition(c: char) -> Option<(u8, u8, u8)> {
    if is_hangul_syllable(c) {
        let n = c as u32 - 0xAC00;
        // break this into the various parts.
        let jongseong = n % 28;
        let jungseong = (n / 28) % 21;
        let choseong = n / (28 * 21);
        Some((choseong as u8, jungseong as u8, jongseong as u8))
    } else {
        // outside the range
        None
    }
}

fn slice_shift_byte(a: &[u8]) -> (Option<u8>, &[u8]) {
    if a.is_empty() {
        (None, a)
    } else {
        (Some(a[0]), &a[1..])
    }
}

pub fn slice_shift_choseong(name: &[u8]) -> (Option<u32>, &[u8]) {
    match slice_shift_byte(name) {
        (Some(b'G'), n) => match slice_shift_byte(n) {
            (Some(b'G'), m) => (Some(1), m),
            (_, _) => (Some(0), n),
        },
        (Some(b'N'), n) => (Some(2), n),
        (Some(b'D'), n) => match slice_shift_byte(n) {
            (Some(b'D'), m) => (Some(4), m),
            (_, _) => (Some(3), n),
        },
        (Some(b'R'), n) => (Some(5), n),
        (Some(b'M'), n) => (Some(6), n),
        (Some(b'B'), n) => match slice_shift_byte(n) {
            (Some(b'B'), m) => (Some(8), m),
            (_, _) => (Some(7), n),
        },
        (Some(b'S'), n) => match slice_shift_byte(n) {
            (Some(b'S'), m) => (Some(10), m),
            (_, _) => (Some(9), n),
        },
        (Some(b'J'), n) => match slice_shift_byte(n) {
            (Some(b'J'), m) => (Some(13), m),
            (_, _) => (Some(12), n),
        },
        (Some(b'C'), n) => (Some(14), n),
        (Some(b'K'), n) => (Some(15), n),
        (Some(b'T'), n) => (Some(16), n),
        (Some(b'P'), n) => (Some(17), n),
        (Some(b'H'), n) => (Some(18), n),
        (_, _) => (Some(11), name),
    }
}

pub fn slice_shift_jungseong(name: &[u8]) -> (Option<u32>, &[u8]) {
    match slice_shift_byte(name) {
        (Some(b'A'), n) => match slice_shift_byte(n) {
            (Some(b'E'), m) => (Some(1), m),
            (_, _) => (Some(0), n),
        },
        (Some(b'Y'), n) => match slice_shift_byte(n) {
            (Some(b'A'), m) => match slice_shift_byte(m) {
                (Some(b'E'), x) => (Some(3), x),
                (_, _) => (Some(2), m),
            },
            (Some(b'E'), m) => match slice_shift_byte(m) {
                (Some(b'O'), x) => (Some(6), x),
                (_, _) => (Some(7), m),
            },
            (Some(b'O'), m) => (Some(12), m),
            (Some(b'U'), m) => (Some(17), m),
            (Some(b'I'), m) => (Some(19), m),
            (_, _) => (None, n),
        },
        (Some(b'E'), n) => match slice_shift_byte(n) {
            (Some(b'O'), m) => (Some(4), m),
            (Some(b'U'), m) => (Some(18), m),
            (_, _) => (Some(5), n),
        },
        (Some(b'O'), n) => match slice_shift_byte(n) {
            (Some(b'E'), m) => (Some(11), m),
            (_, _) => (Some(8), n),
        },
        (Some(b'W'), n) => match slice_shift_byte(n) {
            (Some(b'A'), m) => match slice_shift_byte(m) {
                (Some(b'E'), x) => (Some(10), x),
                (_, _) => (Some(9), m),
            },
            (Some(b'E'), m) => match slice_shift_byte(m) {
                (Some(b'O'), x) => (Some(14), x),
                (_, _) => (Some(15), m),
            },
            (Some(b'I'), m) => (Some(16), m),
            (_, _) => (None, n),
        },
        (Some(b'U'), n) => (Some(13), n),
        (Some(b'I'), n) => (Some(20), n),
        (_, _) => (None, name),
    }
}

pub fn slice_shift_jongseong(name: &[u8]) -> (Option<u32>, &[u8]) {
    match slice_shift_byte(name) {
        (Some(b'G'), n) => match slice_shift_byte(n) {
            (Some(b'G'), m) => (Some(2), m),
            (Some(b'S'), m) => (Some(3), m),
            (_, _) => (Some(1), n),
        },
        (Some(b'N'), n) => match slice_shift_byte(n) {
            (Some(b'J'), m) => (Some(5), m),
            (Some(b'H'), m) => (Some(6), m),
            (Some(b'G'), m) => (Some(21), m),
            (_, _) => (Some(4), n),
        },
        (Some(b'D'), n) => (Some(7), n),
        (Some(b'L'), n) => match slice_shift_byte(n) {
            (Some(b'G'), m) => (Some(9), m),
            (Some(b'M'), m) => (Some(10), m),
            (Some(b'B'), m) => (Some(11), m),
            (Some(b'S'), m) => (Some(12), m),
            (Some(b'T'), m) => (Some(13), m),
            (Some(b'P'), m) => (Some(14), m),
            (Some(b'H'), m) => (Some(15), m),
            (_, _) => (Some(8), n),
        },
        (Some(b'M'), n) => (Some(16), n),
        (Some(b'B'), n) => match slice_shift_byte(n) {
            (Some(b'S'), m) => (Some(18), m),
            (_, _) => (Some(17), n),
        },
        (Some(b'S'), n) => match slice_shift_byte(n) {
            (Some(b'S'), m) => (Some(20), m),
            (_, _) => (Some(19), n),
        },
        (Some(b'J'), n) => (Some(22), n),
        (Some(b'C'), n) => (Some(23), n),
        (Some(b'K'), n) => (Some(24), n),
        (Some(b'T'), n) => (Some(25), n),
        (Some(b'P'), n) => (Some(26), n),
        (Some(b'H'), n) => (Some(27), n),
        (_, _) => (Some(0), name),
    }
}

#[derive(Clone)]
struct PhrasebookIter {
    index: u32,
}

impl PhrasebookIter {
    fn empty() -> Self {
        Self {
            index: PHRASEBOOK.len() as u32,
        }
    }
}

impl Iterator for PhrasebookIter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let b = *PHRASEBOOK.get(self.index as usize)?;
        self.index += 1;
        Some(b)
    }
}

#[derive(Clone)]
pub struct IterStr {
    phrasebook: PhrasebookIter,
    last_was_word: bool,
}

impl IterStr {
    pub fn new(start_index: u32) -> Self {
        Self {
            phrasebook: PhrasebookIter { index: start_index },
            last_was_word: false,
        }
    }
}

const HYPHEN: u8 = 127;

/// An array where `arr[i]` holds the largest lexicon index with length `i`.
const LEXICON_ORDERED_LENGTH_INDICES: [u16; LEXICON_ORDERED_LENGTHS_LEN] = {
    let mut arr = [0u16; LEXICON_ORDERED_LENGTHS_LEN];

    let mut prev_len = None;
    let mut i = 0;
    while i < LEXICON_ORDERED_LENGTHS.len() {
        let (end_idx, length) = LEXICON_ORDERED_LENGTHS[i];

        // make sure this is contiguous - that there are no gaps where e.g. there
        // are words of length 15 and of length 17 but none of length 16.
        if let Some(l) = prev_len {
            assert!(length == l + 1);
        }
        prev_len = Some(length);

        assert!(end_idx <= u16::MAX as usize);
        arr[i] = end_idx as u16 - 1;
        i += 1;
    }

    arr
};

impl Iterator for IterStr {
    type Item = &'static str;
    fn next(&mut self) -> Option<&'static str> {
        let mut tmp = self.phrasebook.clone();
        let raw_b = tmp.next()?;
        // the first byte includes if it is the last in this name
        // in the high bit.
        let (is_end, b) = (raw_b & 0b1000_0000 != 0, raw_b & 0b0111_1111);

        let ret = if b == HYPHEN {
            // have to handle this before the case below, because a -
            // replaces the space entirely.
            self.last_was_word = false;
            "-"
        } else if self.last_was_word {
            self.last_was_word = false;
            // early return, we don't want to update the
            // phrasebook (i.e. we're pretending we didn't touch
            // this byte).
            return Some(" ");
        } else {
            self.last_was_word = true;

            let (length, idx) = if b < PHRASEBOOK_SHORT {
                let idx = b as usize;
                // these lengths are hard-coded
                (LEXICON_SHORT_LENGTHS[idx] as usize, idx)
            } else {
                let idx = u16::from_be_bytes([b - PHRASEBOOK_SHORT, tmp.next().unwrap()]);

                // The value at each index `i` in the array `LEXICON_ORDERED_LENGTH_INDICES`
                // (herein referred to as `arr`) is the largest lexicon index with length `i`.
                let length = match LEXICON_ORDERED_LENGTH_INDICES.binary_search(&idx) {
                    // In this case, `idx` is equal to the index at `arr[i]`,
                    // so `i` is the correct length.
                    Ok(i) => i,
                    // `binary_search(idx)` returning `Err(i)` means that `arr[i-1] < idx < arr[i]`.
                    // Therefore, `idx` is larger than the largest index with length `i - 1`, but
                    // smaller than the largest index with length `i`, meaning its length is `i`.
                    Err(i) => i,
                };

                (length, idx as usize)
            };
            let offset = LEXICON_OFFSETS[idx] as usize;
            &LEXICON[offset..offset + length]
        };
        self.phrasebook = if is_end { PhrasebookIter::empty() } else { tmp };
        Some(ret)
    }
}
