#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    Black = 0,
    White = 1
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        return [Color::White, Color::Black][self as usize];
    }
}

// Node types
// 2-bit number where LSB means lower bound, MSB means upper bound
// so PV is both!
pub const ALL_NODE: u8 = 1;
pub const CUT_NODE: u8 = 2;
pub const PV_NODE: u8 = 3;

// coordinate-related convenience functions

#[inline]
pub fn coord_to_idx(coord: (i32, i32)) -> i32 {
    // turn an x, y coordinate into a bb (0-63) index
    (coord.1 << 3) + coord.0
}

#[inline]
pub fn idx_to_coord(idx: i32) -> (i32, i32) {
    // turn a bb index into an x, y coordinate
    (idx % 8, idx >> 3)
}

#[inline]
pub fn idx_to_bb(idx: i32) -> u64 {
    // turn a bb index into a bitboard with 1 at that index
    1 << idx
}

#[inline]
pub fn coord_to_bb(coord: (i32, i32)) -> u64 {
    // turn an x, y coordinate into a bitboard with a 1 at that coordinate
    idx_to_bb(coord_to_idx(coord))
}

#[inline]
fn at_idx(bb: u64, idx: i32) -> u64{
    bb & (1 << idx)
}

pub fn idx_to_str(idx: i32) -> String {
    let coords = (idx % 8, idx >> 3);
    let f = "abcdefgh".as_bytes()[coords.0 as usize] as char;
    let r = (coords.1 + 1).to_string();
    return format!("{}{}", f.to_string(), r);
}

pub fn str_to_idx(s: String) -> i32 {
    let s = s.as_bytes();
    return coord_to_idx(((s[0] - b'a') as i32, (s[1] - b'1') as i32));
}

pub fn bb_str(bb: u64) -> String {
    let mut s = String::new();
    let mut b = bb;
    for _ in 0..8 {
        let rank = b & ((1 << 8) - 1);
        b = b >> 8;
        s.push_str(format!("{:08b}\n", rank).as_str());
    }
    return s.chars().rev().collect();
}


// various convenience bitboard masks

pub const FILE_MASKS: [u64; 8] =
    [
        0x0101010101010101,     // A FILE
        0x0202020202020202,     // B FILE
        0x0404040404040404,     // ...
        0x0808080808080808,
        0x1010101010101010,
        0x2020202020202020,
        0x4040404040404040,
        0x8080808080808080      // H FILE
    ];

pub const RANK_MASKS: [u64; 8] =
    [
        0x00000000000000FF,     // 1ST RANK
        0x000000000000FF00,     // 2ND RANK
        0x0000000000FF0000,     // ...
        0x00000000FF000000,
        0x000000FF00000000,
        0x0000FF0000000000,
        0x00FF000000000000,
        0xFF00000000000000      // 8TH RANK
    ];

pub const CENTER_MASK: u64 = 0x0000001818000000;
pub const NEAR_CENTER_MASK: u64 = 0x00003C24243C0000;
pub const AHEAD_RANKS: [[u64; 8]; 2] =
    [
        [
            0x0000000000000000,
            0x00000000000000FF,
            0x000000000000FFFF,
            0x0000000000FFFFFF,
            0x00000000FFFFFFFF,
            0x000000FFFFFFFFFF,
            0x0000FFFFFFFFFFFF,
            0x00FFFFFFFFFFFFFF,
        ],
        [
            0xFFFFFFFFFFFFFF00,
            0xFFFFFFFFFFFF0000,
            0xFFFFFFFFFF000000,
            0xFFFFFFFF00000000,
            0xFFFFFF0000000000,
            0xFFFF000000000000,
            0xFF00000000000000,
            0x0000000000000000
        ],
    ];
