use rand::Rng;

use crate::bitboard::Bitboard;
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
const EP_OFFSET: usize = 772;          // abcdefgh
const STM_OFFSET: usize = 780;         // on if white is stm

pub fn initialize_zobrist_table() {
    let mut rng = rand::thread_rng();
    for i in 0..781 {
        unsafe {
            ZOBRIST_TABLE[i] = rng.gen();
        }
    }
}

fn get_piece_tile_idx(piece_num: usize, idx: i32) -> usize {
    (piece_num << 6) + idx as usize
}

fn get_piece_zobrist(piece: u8, side: Color, idx: i32) -> u64 {
    unsafe {
        return ZOBRIST_TABLE[get_piece_tile_idx(get_piece_num(piece, side), idx)];
    }
}

fn get_zobrist_for_piece_board(piece: u8, side: Color, board: u64) -> u64 {
    let mut board = board;
    let mut hash = 0;
    while board != 0 {
        let idx = board.trailing_zeros() as i32;
        hash ^= get_piece_zobrist(piece, side, idx);
        board &= board - 1;
    }
    return hash;
}

pub fn calculate_hash(pos: &Bitboard) -> u64 {
    let mut hash: u64 = 0;

    for side_to_move in [Color::Black, Color::White] {
        let side = side_to_move as usize;

        let pawns = pos.pawn[side];
        hash ^= get_zobrist_for_piece_board(b'p', side_to_move, pawns);

        let knights = pos.knight[side];
        hash ^= get_zobrist_for_piece_board(b'n', side_to_move, knights);

        let bishops = pos.bishop[side];
        hash ^= get_zobrist_for_piece_board(b'b', side_to_move, bishops);

        let rooks = pos.rook[side];
        hash ^= get_zobrist_for_piece_board(b'r', side_to_move, rooks);

        let queens = pos.queen[side];
        hash ^= get_zobrist_for_piece_board(b'q', side_to_move, queens);

        let kings = pos.king[side];
        hash ^= get_zobrist_for_piece_board(b'k', side_to_move, kings);
    }

    unsafe {
        for i in 0..4 {
            let mask: u8 = 1 << i;
            if pos.castling_rights & mask != 0 { hash ^= ZOBRIST_TABLE[CR_OFFSET + (3 - i) as usize]; }
        }

        if pos.ep_file != -1 {
            hash ^= ZOBRIST_TABLE[EP_OFFSET + pos.ep_file as usize];
        }

        if pos.side_to_move == Color::White {
            hash ^= ZOBRIST_TABLE[STM_OFFSET];
        }
    }

    return hash;
}

pub fn calculate_pawn_hash(pos: &Bitboard) -> u64 {
    let mut hash: u64 = 0;
    for side_to_move in [Color::Black, Color::White] {
        let pawns = pos.pawn[side_to_move as usize];
        hash ^= get_zobrist_for_piece_board(b'p', side_to_move, pawns);
    }
    return hash;
}

pub fn simple_move_hash(piece: u8,
                        start_idx: i32,
                        end_idx: i32,
                        side_to_move: Color) -> u64 {
    let piece_num = get_piece_num(piece, side_to_move);
    let mut hash = 0;
    unsafe {
        hash ^= ZOBRIST_TABLE[get_piece_tile_idx(piece_num, start_idx)];
        hash ^= ZOBRIST_TABLE[get_piece_tile_idx(piece_num, end_idx)];
    }
    return hash;
}

pub fn en_passant_hash(ep_idx: i32, removed_side: Color) -> u64 {
    let captured_num = get_piece_num(b'p', removed_side);
    unsafe {
        return ZOBRIST_TABLE[get_piece_tile_idx(captured_num, ep_idx)];
    }
}

pub fn update_hash(piece: u8,
                   start_idx: i32,
                   end_idx: i32,
                   captured_piece: u8,
                   promoted_piece: u8,
                   old_ep_file: i32, // -1 if none
                   new_ep_file: i32, // -1 if none
                   old_cr: u8,
                   new_cr: u8,
                   side_to_move: Color) -> u64 {
    // A bit cumbersome, but this is meant to be called
    // by other, more convenient functions, so it has to be flexible

    let piece_num = get_piece_num(piece, side_to_move);
    let mut hash = 0;
    unsafe {
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

        if old_ep_file >= 0 {
            hash ^= ZOBRIST_TABLE[EP_OFFSET + old_ep_file as usize];
        }

        if new_ep_file >= 0 {
            hash ^= ZOBRIST_TABLE[EP_OFFSET + new_ep_file as usize];
        }

        let mut changed_cr = old_cr ^ new_cr;
        while changed_cr != 0 {
            let idx = changed_cr.trailing_zeros() as i32;
            hash ^= ZOBRIST_TABLE[CR_OFFSET + (3 - idx) as usize];
            changed_cr &= changed_cr - 1;
        }

        hash ^= ZOBRIST_TABLE[STM_OFFSET];
    }
    return hash;
}

pub fn update_pawn_hash(piece: u8,
                               start_idx: i32,
                               end_idx: i32,
                               captured_piece: u8,
                               promoted_piece: u8,
                               side_to_move: Color) -> u64 {
    let mut hash = 0;
    unsafe {
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
    }
    return hash;
}

pub fn null_move_hash() -> u64 {
    unsafe {
        ZOBRIST_TABLE[STM_OFFSET]
    }
}
