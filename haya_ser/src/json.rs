const B: u8 = b'b'; // \x08
const T: u8 = b't'; // \x09
const N: u8 = b'n'; // \x0a
const F: u8 = b'f'; // \x0c
const R: u8 = b'r'; // \x0d
const Q: u8 = b'"'; // \x22
const S: u8 = b'\\'; // \x5c
const U: u8 = 0xff; // non-printable
const E: u8 = 0xff; // error

const ESCAPE: &[u8; 256] = &[
    U, U, U, U, U, U, U, U, B, T, N, U, F, R, U, U, // 0
    U, U, U, U, U, U, U, U, U, U, U, U, U, U, U, U, // 1
    1, 1, Q, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, S, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, // 8
    E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, // 9
    E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, // A
    E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, E, // B
    E, E, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, E, E, E, E, E, E, E, E, E, E, E, // F
];

pub const fn json_char_width_escaped(ch: u8) -> u8 {
    ESCAPE[ch as usize]
}
