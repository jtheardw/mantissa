use std::ffi::CString;

use crate::bitboard::*;
use crate::moveutil::*;
use crate::util::*;

// This integration with fathom for Syzygy has been largely
// informed by the implementation in Asymptote.  This is essentially
// a port of nearly identical behavior but compatible for Mantissa.
// Thanks to the author Maximillian Lupke and to the Fathom devs.

const TB_LOSS: u32 = 0;
const TB_BLESSED_LOSS: u32 = 1;
const TB_DRAW: u32 = 2;
const TB_CURSED_WIN: u32 = 3;
const TB_WIN: u32 = 4;
const TB_FAILED: u32 = 0xFFFFFFFF;

const TB_QUEEN: u32 = 1;
const TB_ROOK: u32 = 2;
const TB_BISHOP: u32 = 3;
const TB_KNIGHT: u32 = 4;

const TB_RESULT_WDL_MASK: u32 = 0x0000000F;
const TB_RESULT_TO_MASK: u32 = 0x000003F0;
const TB_RESULT_FROM_MASK: u32 = 0x0000FC00;
const TB_RESULT_PROMOTES_MASK: u32 = 0x00070000;
const TB_RESULT_EP_MASK: u32 = 0x00080000;
const TB_RESULT_DTZ_MASK: u32 = 0xFFF00000;
const TB_RESULT_WDL_SHIFT: u32 = 0;
const TB_RESULT_TO_SHIFT: u32 = 4;
const TB_RESULT_FROM_SHIFT: u32 = 10;
const TB_RESULT_PROMOTES_SHIFT: u32 = 16;
const TB_RESULT_EP_SHIFT: u32 = 19;
const TB_RESULT_DTZ_SHIFT: u32 = 20;

pub static mut TB_ENABLED: bool = false;

pub unsafe fn setup_tb(path: &str) -> bool {
    let c_str = match CString::new(path) {
        Ok(s) => s,
        Err(_) => {TB_ENABLED = false; return false;}
    };

    TB_ENABLED = c::tb_init(c_str.as_ptr());
    return TB_ENABLED;
}

pub fn tb_active() -> bool {
    unsafe {
        return TB_ENABLED;
    }
}

pub fn max_tb_pieces() -> u32 {
    unsafe {
        c::TB_LARGEST
    }
}

fn tb_result_to_move(pos: &Bitboard, result: u32) -> Move {
    let from = (result & TB_RESULT_FROM_MASK) >> TB_RESULT_FROM_SHIFT;
    let to = (result & TB_RESULT_TO_MASK) >> TB_RESULT_TO_SHIFT;
    let tb_promote = ((result & TB_RESULT_PROMOTES_MASK) >> TB_RESULT_PROMOTES_SHIFT);
    let promote_to = match tb_promote {
        TB_KNIGHT => b'n',
        TB_BISHOP => b'b',
        TB_ROOK => b'r',
        TB_QUEEN => b'q',
        _ => 0
    };

    let piece = pos.piece_at_square(from as i8, pos.side_to_move);

    return Move {
        piece: piece,
        start: from as i8,
        end: to as i8,
        promote_to: promote_to
    };
}

pub fn probe_root(pos: &Bitboard) -> Option<(Move, i32)> {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let ep = if pos.ep_file >= 0 {
        if pos.side_to_move == Color::White {
            coord_to_idx((pos.ep_file, 5))
        } else {
            coord_to_idx((pos.ep_file, 2))
        }
    } else {
        0
    } as u32;

    unsafe {
        let result = c::tb_probe_root_wrapper(
            pos.composite[white],
            pos.composite[black],
            pos.king[white] | pos.king[black],
            pos.queen[white] | pos.queen[black],
            pos.rook[white] | pos.rook[black],
            pos.bishop[white] | pos.bishop[black],
            pos.knight[white] | pos.knight[black],
            pos.pawn[white] | pos.pawn[black],
            pos.halfmove as u32,
            0,                  // TODO maybe, castling rights?
            ep,
            (pos.side_to_move == Color::White) as u8
        );

        if result == TB_FAILED {
            return None;
        }

        let wdl = (result & TB_RESULT_WDL_MASK) >> TB_RESULT_WDL_SHIFT;
        let dtz = ((result & TB_RESULT_DTZ_MASK) >> TB_RESULT_DTZ_SHIFT) as i32;
        let best_move = tb_result_to_move(pos, result);

        let score = match wdl {
            TB_LOSS => -TB_WIN_SCORE + dtz,
            TB_BLESSED_LOSS => 0,
            TB_DRAW => 0,
            TB_CURSED_WIN => 0,
            TB_WIN => TB_WIN_SCORE - dtz,
            _ => panic!("impossible WDL in TB probe!")
        };
        return Some((best_move, score));
    }
}

pub fn probe_wdl(pos: &Bitboard) -> Option<i32> {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let ep = if pos.ep_file >= 0 {
        if pos.side_to_move == Color::White {
            coord_to_idx((pos.ep_file, 5))
        } else {
            coord_to_idx((pos.ep_file, 2))
        }
    } else {
        0
    } as u32;
    unsafe {
        let result = c::tb_probe_wdl_wrapper(
            pos.composite[white],
            pos.composite[black],
            pos.king[white] | pos.king[black],
            pos.queen[white] | pos.queen[black],
            pos.rook[white] | pos.rook[black],
            pos.bishop[white] | pos.bishop[black],
            pos.knight[white] | pos.knight[black],
            pos.pawn[white] | pos.pawn[black],
            pos.halfmove as u32,
            0,                  // TODO maybe, castling rights?
            ep,
            (pos.side_to_move == Color::White) as u8
        );

        return match result {
            TB_LOSS => Some(-TB_WIN_SCORE),
            TB_BLESSED_LOSS => Some(0),
            TB_DRAW => Some(0),
            TB_CURSED_WIN => Some(0),
            TB_WIN => Some(TB_WIN_SCORE),
            _ => None
        };
    }
}


mod c {
    extern "C" {
        pub static TB_LARGEST: u32;
        pub fn tb_init(filename: *const i8) -> bool;
        pub fn tb_probe_wdl_wrapper(
            white: u64,
            black: u64,
            kings: u64,
            queens: u64,
            rooks: u64,
            bishops: u64,
            knights: u64,
            pawns: u64,
            rule50: u32,
            castling: u32,
            ep: u32,
            turn: u8,
        ) -> u32;

        pub fn tb_probe_root_wrapper(
            white: u64,
            black: u64,
            kings: u64,
            queens: u64,
            rooks: u64,
            bishops: u64,
            knights: u64,
            pawns: u64,
            rule50: u32,
            castling: u32,
            ep: u32,
            turn: u8,
        ) -> u32;
    }
}
