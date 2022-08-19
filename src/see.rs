use std::cmp;

use crate::bitboard::*;
use crate::eval::*;
use crate::movegen::*;
use crate::moveutil::*;
use crate::util::*;

fn lva(pos: &Bitboard, atk_bb: u64, side: Color) -> (u64, u8) {
    let side = side as usize;
    if atk_bb & pos.pawn[side] != 0 {
        return (idx_to_bb((atk_bb & pos.pawn[side]).trailing_zeros() as i8), b'p');
    }
    if atk_bb & pos.knight[side] != 0 {
        return (idx_to_bb((atk_bb & pos.knight[side]).trailing_zeros() as i8), b'n');
    }
    if atk_bb & pos.bishop[side] != 0 {
        return (idx_to_bb((atk_bb & pos.bishop[side]).trailing_zeros() as i8), b'b');
    }
    if atk_bb & pos.rook[side] != 0 {
        return (idx_to_bb((atk_bb & pos.rook[side]).trailing_zeros() as i8), b'r');
    }
    if atk_bb & pos.queen[side] != 0 {
        return (idx_to_bb((atk_bb & pos.queen[side]).trailing_zeros() as i8), b'q');
    }
    if atk_bb & pos.king[side] != 0 {
        return (idx_to_bb((atk_bb & pos.king[side]).trailing_zeros() as i8), b'k');
    }
    return (0, 0);
}

fn get_piece_value(piece: u8) -> i32 {
    return match piece {
        b'q' => 9000,
        b'r' => 5000,
        b'b' => 3000,
        b'n' => 3000,
        b'p' => 1000,
        b'k' => 200000,
        _ => 0
    };
}

fn idx_attacks(pos: &Bitboard, idx: i8, occ: u64) -> u64 {
    // process:
    // create "virtual pieces" at the idx and see
    // if they can attack enemy pieces of the appropriate
    // types.  If so, there is an attack on this square
    //
    // should provide an alternative function to get all attacks
    // by a side, but this is optimized for checking *just* a single
    // square
    let self_bb = idx_to_bb(idx);
    let mut attackers_bb: u64 = 0;

    // knights
    let virt_knight_bb = knight_moves_board(idx);
    attackers_bb |= virt_knight_bb & (pos.knight[1] | pos.knight[0]);

    // king
    let virt_king_bb = king_normal_moves_board(idx);
    attackers_bb |= virt_king_bb & (pos.king[1] | pos.king[0]);

    // pawns
    let virt_pawn_white_bb = ((self_bb & !FILE_MASKS[0]) << 7) | ((self_bb & !FILE_MASKS[7]) << 9);
    let virt_pawn_black_bb = ((self_bb & !FILE_MASKS[0]) >> 9) | ((self_bb & !FILE_MASKS[7]) >> 7);
    attackers_bb |= virt_pawn_white_bb & pos.pawn[0];
    attackers_bb |= virt_pawn_black_bb & pos.pawn[1];

    // bishops & queens
    let virt_bishop_bb = bishop_moves_board(idx, occ);
    attackers_bb |= virt_bishop_bb & (pos.bishop[0] | pos.bishop[1] | pos.queen[0] | pos.queen[1]);

    // rooks & queens
    let virt_rook_bb = rook_moves_board(idx, occ);
    attackers_bb |= virt_rook_bb & (pos.rook[0] | pos.rook[1] | pos.queen[0] | pos.queen[1]);

    return attackers_bb;
}

fn idx_diag_attacks(pos: &Bitboard, idx: i8, occ: u64) -> u64 {
    // bishops & queens
    let virt_bishop_bb = bishop_moves_board(idx, occ);
    return virt_bishop_bb & (pos.bishop[0] | pos.bishop[1] | pos.queen[0] | pos.queen[1]);
}

fn idx_cardinal_attacks(pos: &Bitboard, idx: i8, occ: u64) -> u64 {
    // rooks & queens
    let virt_rook_bb = rook_moves_board(idx, occ);
    return virt_rook_bb & (pos.rook[0] | pos.rook[1] | pos.queen[0] | pos.queen[1]);
}

pub fn see(pos: &Bitboard, to_idx: i8, target_piece: u8, from_idx: i8, atk_piece: u8) -> i32 {
    let white = Color::White as usize;
    let black = Color::Black as usize;

    let mut gain: [i32; 64] = [0; 64];
    let mut depth: usize = 0;
    let xrayable_bb = pos.pawn[black] | pos.pawn[white] | pos.rook[black] | pos.rook[white] | pos.bishop[black] | pos.bishop[white] | pos.queen[black] | pos.queen[white];

    let mut from_sq: u64 = idx_to_bb(from_idx);
    let mut occ = pos.composite[black] | pos.composite[white];
    let mut attack_bb = idx_attacks(pos, to_idx, occ);
    let mut done_atks: u64 = 0;
    let mut atk_piece = atk_piece;
    let mut target_piece = target_piece;

    while from_sq != 0 {
        gain[depth] = get_piece_value(target_piece) - if depth > 0 {gain[depth - 1]} else {0};

        // remove the attacker from occupancy and attacks
        occ ^= from_sq;
        done_atks |= from_sq;
        if (from_sq & xrayable_bb) != 0 {
            // gotta add xrays to the board
            if atk_piece == b'p' || atk_piece == b'b' || atk_piece == b'q' {
                attack_bb |= idx_diag_attacks(pos, to_idx, occ);
            }
            if atk_piece == b'p' || atk_piece == b'r' || atk_piece == b'q' {
                attack_bb |= idx_cardinal_attacks(pos, to_idx, occ);
            }
        }
        attack_bb &= !done_atks;

        target_piece = atk_piece;
        let lva_info = lva(pos, attack_bb, if (depth % 2) != 0 {pos.side_to_move} else {!pos.side_to_move});
        from_sq = lva_info.0;
        atk_piece = lva_info.1;
        if atk_piece == 0 {
            break;
        }
        depth += 1;
    }
    while depth > 0 {
        gain[depth - 1] = -cmp::max(-gain[depth - 1], gain[depth]);
        depth -= 1;
    }

    return gain[0];
}
