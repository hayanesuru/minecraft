use alloc::vec::Vec;

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
