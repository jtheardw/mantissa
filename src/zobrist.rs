use rand::Rng;

use crate::util::*;

// 6 * 64 entries for white pieces
// 6 * 64 entries for black pieces
// 4 entries for castling rights
// 8 entries for en passant file
// 1 entry for side to move
// total is
static mut ZOBRIST_TABLE: [u64; 781] = [0; 781];

const WHITE_PIECE_OFFSET: usize = 0; // KQRBNP
const BLACK_PIECE_OFFSET: usize = 384; // KQRBNP
const CR_OFFSET: usize = 768;          // KQkq
const EP_OFFSET: usize = 776;          // abcdefgh
const STM_OFFSET: usize = 780;         // on if white is stm

pub fn initialize_zobrist_table() {
    let mut rng = rand::thread_rng();
    for i in 0..781 {
        unsafe {
            ZOBRIST_TABLE[i] = rng.gen();
        }
    }
}

pub fn get_piece_num(piece: u8, side: Color) -> usize {
    let piece_offset = match piece {
        b'k'=> 0,
        b'q'=> 1,
        b'r'=> 2,
        b'b'=> 3,
        b'n'=> 4,
        b'p'=> 5,
        _ => panic!("bad piece for getting num")
    };
    if side == Color::Black {
        return piece_offset + 6;
    } else {
        return piece_offset;
    }
}

fn get_piece_tile_idx(piece_num: usize, idx: i32) -> usize {
    piece_num * 64 + idx as usize
}

pub unsafe fn update_hash(current_hash: u64,
                   piece: u8,
                   start_idx: i32,
                   end_idx: i32,
                   captured_piece: u8,
                   promoted_piece: u8,
                   old_ep_file: i32, // -1 if none
                   new_ep_file: i32, // -1 if none
                   old_cr: (bool, bool, bool, bool),
                   new_cr: (bool, bool, bool, bool),
                   side_to_move: Color) -> u64 {
    // A bit cumbersome, but this is meant to be called
    // by other, more convenient functions, so it has to be flexible

    let piece_num = get_piece_num(piece, side_to_move);
    let mut hash = current_hash;
    // undo current location of piece
    hash ^= ZOBRIST_TABLE[get_piece_tile_idx(piece_num, start_idx)];

    // apply the piece showing up at the end
    if promoted_piece != 0 {
        let promoted_num = get_piece_num(promoted_piece, side_to_move);
        hash ^= ZOBRIST_TABLE[get_piece_tile_idx(promoted_num, end_idx)];
    } else {
        hash ^= ZOBRIST_TABLE[get_piece_tile_idx(piece_num, end_idx)];
    }

    // if we captured, we need to "remove" that piece
    if captured_piece != 0 {
        let captured_num = get_piece_num(captured_piece, !side_to_move);
        hash ^= ZOBRIST_TABLE[get_piece_tile_idx(captured_num, end_idx)];
    }

    if old_ep_file > 0 {
        hash ^= ZOBRIST_TABLE[EP_OFFSET + old_ep_file as usize];
    }

    if new_ep_file > 0 {
        hash ^= ZOBRIST_TABLE[EP_OFFSET + new_ep_file as usize];
    }

    if old_cr.0 != new_cr.0 { hash ^= ZOBRIST_TABLE[CR_OFFSET + 0]; }
    if old_cr.1 != new_cr.1 { hash ^= ZOBRIST_TABLE[CR_OFFSET + 1]; }
    if old_cr.2 != new_cr.2 { hash ^= ZOBRIST_TABLE[CR_OFFSET + 2]; }
    if old_cr.3 != new_cr.3 { hash ^= ZOBRIST_TABLE[CR_OFFSET + 3]; }

    hash ^= ZOBRIST_TABLE[STM_OFFSET];
    return hash;
}

pub unsafe fn update_pawn_hash(current_hash: u64,
                        piece: u8,
                        start_idx: i32,
                        end_idx: i32,
                        captured_piece: u8,
                        promoted_piece: u8,
                        side_to_move: Color) -> u64 {
    let mut hash = current_hash;
    if piece == b'p' {
        // undo current location of pawn
        let piece_num = get_piece_num(b'p', side_to_move);
        hash ^= ZOBRIST_TABLE[get_piece_tile_idx(piece_num, start_idx)];
        if promoted_piece == 0 {
            hash ^= ZOBRIST_TABLE[get_piece_tile_idx(piece_num, end_idx)];
        }
    }

    if captured_piece == b'p' {
        let piece_num = get_piece_num(b'p', !side_to_move);
        hash ^= ZOBRIST_TABLE[get_piece_tile_idx(piece_num, end_idx)];
    }

    return hash;
}
