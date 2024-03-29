// Node types
// 2-bit number where LSB means lower bound, MSB means upper bound
// so PV is both!
pub const ALL_NODE: u8 = 1;
pub const CUT_NODE: u8 = 2;
pub const PV_NODE: u8 = 3;

pub const TB_WIN_SCORE: i32 = 700000;
pub const MIN_TB_WIN_SCORE: i32 = 600000;
pub const MATE_SCORE: i32 = 1000000;
pub const MIN_MATE_SCORE: i32 = MATE_SCORE - 100000;
pub const DRAW_SCORE: i32 = 0;

pub const MAX_DEPTH: usize = 64;
pub const MAX_PLY: usize = 128;

pub const PAWN: u8 = 0;
pub const KNIGHT: u8 = 1;
pub const BISHOP: u8 = 2;
pub const ROOK: u8 = 3;
pub const QUEEN: u8 = 4;
pub const KING: u8 = 5;

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

// coordinate-related convenience functions

#[inline]
pub fn coord_to_idx(coord: (i32, i32)) -> i8 {
    // turn an x, y coordinate into a bb (0-63) index
    ((coord.1 << 3) + coord.0) as i8
}

#[inline]
pub fn idx_to_coord(idx: i8) -> (i32, i32) {
    // turn a bb index into an x, y coordinate
    (idx as i32 % 8, idx as i32 >> 3)
}

#[inline]
pub fn idx_to_bb(idx: i8) -> u64 {
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

pub fn idx_to_str(idx: i8) -> String {
    let coords = (idx % 8, idx >> 3);
    let f = "abcdefgh".as_bytes()[coords.0 as usize] as char;
    let r = (coords.1 + 1).to_string();
    return format!("{}{}", f.to_string(), r);
}

pub fn str_to_idx(s: String) -> i8 {
    let s = s.as_bytes();
    return coord_to_idx(((s[0] - b'a') as i32, (s[1] - b'1') as i32));
}

pub fn bytes_to_idx(file_byte: u8, rank_byte: u8) -> i8 {
    return coord_to_idx(((file_byte - b'a') as i32, (rank_byte - b'1') as i32));
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

pub fn get_piece_num(piece: u8, side: Color) -> usize {
    let piece_offset = match piece {
        b'k'=> 0,
        b'q'=> 1,
        b'r'=> 2,
        b'b'=> 3,
        b'n'=> 4,
        b'p'=> 5,
        _ => panic!("bad piece for getting num {}", piece)
    };
    if side == Color::Black {
        return piece_offset + 6;
    } else {
        return piece_offset;
    }
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

pub const QUEEN_PHASE: i32 = 40; // 2 queens = 80
pub const ROOK_PHASE: i32 = 22;  // 4 rooks = 88
pub const BISHOP_PHASE: i32 = 12; // 4 bishops = 48
pub const KNIGHT_PHASE: i32 = 10; // 4 knights = 40
