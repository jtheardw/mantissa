use crate::bitboard::Bitboard;
use crate::magic::*;
use crate::moveutil::*;
use crate::util::*;

pub static mut KING_MASK: [u64; 64] = [0; 64];
pub static mut KNIGHT_MASK: [u64; 64] = [0; 64];
pub static mut BISHOP_MASK: [u64; 64] = [0; 64];
pub static mut ROOK_MASK: [u64; 64] = [0; 64];

fn gen_knight_mask() {
    for idx in 0..64 {
        let mut bb: u64 = 0;
        let (kx, ky) = idx_to_coord(idx);
        for (dx, dy) in [
            (-1, 2),
            ( 1, 2),
            (-2, 1),
            ( 2, 1),
            (-2,-1),
            ( 2,-1),
            (-1,-2),
            ( 1,-2)
        ].iter() {
            let (nx, ny) = (kx + dx, ky + dy);
            if (nx >= 8 || nx < 0) || (ny >= 8 || ny < 0) { continue; }
            let new_idx = coord_to_idx((nx, ny));
            bb |= 1 << new_idx;
        }
        unsafe {
            KNIGHT_MASK[idx as usize] = bb;
        }
    }
}

fn gen_rook_mask() {
    for idx in 0..64 {
        let mut bb: u64 = 0;
        let (rx, ry) = idx_to_coord(idx);
        for (sx, sy) in [(0, 1), (0, -1), (-1, 0), (1, 0)].iter() {
            let mut d = 1;
            while d < 8 {
                let (nx, ny) = (rx + (sx * d), ry + (sy * d));
                if ((nx < 1 || nx >= 7) && (nx != rx)) || ((ny < 1 || ny >= 7) && (ny != ry)) {
                    break;
                }
                let new_idx = coord_to_idx((nx, ny));
                bb |= 1 << new_idx;
                d += 1;
            }
        }
        unsafe {
            ROOK_MASK[idx as usize] = bb;
        }
    }
}

pub fn gen_bishop_mask() {
    for idx in 0..64 {
        let mut bb: u64 = 0;
        let (bx, by) = idx_to_coord(idx);
        for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)].iter() {
            let mut d = 1;
            while d < 8 {
                let (nx, ny) = (bx + (sx * d), by + (sy * d));
                if ((nx < 1 || nx >= 7) && (nx != bx)) || ((ny < 1 || ny >= 7) && (ny != by)) {
                    break;
                }
                let new_idx = coord_to_idx((nx, ny));
                bb |= 1 << new_idx;
                d += 1;
            }
        }
        unsafe {
            BISHOP_MASK[idx as usize] = bb;
        }
    }
}

pub fn gen_king_mask() {
    for idx in 0..64 {
        let mut bb: u64 = 0;
        let (kx, ky) = idx_to_coord(idx);
        for (dx, dy) in [
            (-1, 1), (1, 1), (-1, -1), (1, -1),
            (0, 1), (0, -1), (-1, 0), (1, 0)
        ].iter() {
            let (nx, ny) = (kx + dx, ky + dy);
            if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                continue;
            }
            let new_idx = coord_to_idx((nx, ny));
            bb |= 1 << new_idx;
        }
        unsafe {
            KING_MASK[idx as usize] = bb;
        }
    }
}

pub fn initialize_masks() {
    gen_knight_mask();
    gen_king_mask();
    gen_bishop_mask();
    gen_rook_mask();
}

fn get_piece_movelist(pos: &Bitboard, idx: i8, piece: u8, move_board: u64, moves: &mut Vec<Move>) {
    // gets *all* pseudo-legal moves from the moveboard (not just quiet ones)
    // this should appropriately filter out capturing own pieces, etc.
    let own_occ = pos.composite[pos.side_to_move as usize];
    let mut move_board = move_board & !own_occ;

    while move_board != 0 {
        let end_idx = move_board.trailing_zeros() as i8;
        moves.push(Move::piece_move(idx, end_idx, piece));
        move_board &= move_board - 1; // cute trick I learned from Expositor code
    }

}

fn get_piece_quiets(pos: &Bitboard, idx: i8, piece: u8, move_board: u64, moves: &mut Vec<Move>) {
    // gets *all* pseudo-legal moves from the moveboard (not just quiet ones)
    // this should appropriately filter out capturing own pieces, etc.
    let occ = pos.composite[0] | pos.composite[1];
    let mut move_board = move_board & !occ;

    while move_board != 0 {
        let end_idx = move_board.trailing_zeros() as i8;
        moves.push(Move::piece_move(idx, end_idx, piece));
        move_board &= move_board - 1; // cute trick I learned from Expositor code
    }
}

fn get_piece_captures(pos: &Bitboard, idx: i8, piece: u8, move_board: u64, moves: &mut Vec<Move>) {
    // this should appropriately filter out capturing own pieces, etc.
    let enemy_occ = pos.composite[!pos.side_to_move as usize];
    let mut move_board = move_board & enemy_occ;

    while move_board != 0 {
        let end_idx = move_board.trailing_zeros() as i8;
        moves.push(Move::piece_move(idx, end_idx, piece));
        move_board &= move_board - 1;
    }
}

pub fn pawn_walk_board(occ: u64, idx: i8, side_to_move: Color) -> u64 {
    let mut move_board: u64;
    if side_to_move == Color::White {
        // walk forward one if unoccupied
        move_board = idx_to_bb(idx+ 8) & !occ;
        if (move_board != 0) && (idx_to_bb(idx) & RANK_MASKS[1]) != 0 {
            // might be able to walk forward twice
            move_board |= (move_board << 8) & !occ;
        }
    } else {
        move_board = idx_to_bb(idx - 8) & !occ;
        if (move_board != 0) && (idx_to_bb(idx) & RANK_MASKS[6]) != 0 {
            // might be able to walk forward twice
            move_board |= (move_board >> 8) & !occ;
        }
    }

    return move_board
}

pub fn pawn_attack_board(idx: i8, side_to_move: Color) -> u64 {
    let start_idx_bb = idx_to_bb(idx);
    if side_to_move == Color::White {
        return ((start_idx_bb & !FILE_MASKS[0]) << 7) | ((start_idx_bb & !FILE_MASKS[7]) << 9);
    } else {
        return ((start_idx_bb & !FILE_MASKS[0]) >> 9) | ((start_idx_bb & !FILE_MASKS[7]) >> 7);
    };
}


pub fn pawn_capture_board(enemy_occ: u64, idx: i8, ep_file: i32, side_to_move: Color) -> u64 {
    let start_idx_bb = idx_to_bb(idx);
    let mut en_passant = 0;
    let attacks;
    if side_to_move == Color::White {
        attacks = ((start_idx_bb & !FILE_MASKS[0]) << 7) | ((start_idx_bb & !FILE_MASKS[7]) << 9);
        if ep_file != -1 {
            en_passant = attacks & coord_to_bb((ep_file, 5));
        }
    } else {
        attacks = ((start_idx_bb & !FILE_MASKS[0]) >> 9) | ((start_idx_bb & !FILE_MASKS[7]) >> 7);
        if ep_file != -1 {
            en_passant = attacks & coord_to_bb((ep_file, 2));
        }
    };
    let captures = (attacks & enemy_occ) | en_passant;

    return captures;
}

// captures and promotions
pub fn pawn_qmoves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let enemy_occ = pos.composite[!pos.side_to_move as usize];
    let mut promotions = 0;
    let mut captures = pawn_capture_board(enemy_occ, idx, pos.ep_file, pos.side_to_move);
    if pos.side_to_move == Color::White {
        if (idx_to_bb(idx) & RANK_MASKS[6]) != 0 {
            promotions = pawn_walk_board(occ, idx, pos.side_to_move);
            promotions |= captures & RANK_MASKS[7];
            captures &= !RANK_MASKS[7];
        }
    } else {
        if (idx_to_bb(idx) & RANK_MASKS[1]) != 0 {
            promotions = pawn_walk_board(occ, idx, pos.side_to_move);
            promotions |= captures & RANK_MASKS[0];
            captures &= !RANK_MASKS[0];
        }
    }

    while captures != 0 {
        let end_idx = captures.trailing_zeros() as i8;
        if (idx_to_bb(end_idx) & occ) == 0 {
            // capturing "empty" space.  This is en passant
            moves.push(Move::ep_capture(idx, end_idx));
        } else {
            moves.push(Move::pawn_move(idx, end_idx));
        }
        captures &= captures - 1;
    }

    while promotions != 0 {
        let end_idx = promotions.trailing_zeros() as i8;
        for p in [b'q', b'r', b'b', b'n'] {
            moves.push(Move::promotion(idx, end_idx, p));
        }
        promotions &= promotions - 1;
    }
}

pub fn pawn_quiet_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];

    let mut walks = pawn_walk_board(occ, idx, pos.side_to_move);

    if pos.side_to_move == Color::White {
        walks &= !RANK_MASKS[7];
    } else {
        walks &= !RANK_MASKS[0];
    }

    while walks != 0 {
        let end_idx = walks.trailing_zeros() as i8;
        moves.push(Move::pawn_move(idx, end_idx));
        walks &= walks - 1;
    }
}

// all moves
pub fn pawn_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let enemy_occ = pos.composite[!pos.side_to_move as usize];

    let mut promotions;
    let mut captures = pawn_capture_board(enemy_occ, idx, pos.ep_file, pos.side_to_move);
    let mut walks = pawn_walk_board(occ, idx, pos.side_to_move);

    if pos.side_to_move == Color::White {
        promotions = (captures | walks) & RANK_MASKS[7];
        captures &= !RANK_MASKS[7];
        walks &= !RANK_MASKS[7];
    } else {
        promotions = (captures | walks) & RANK_MASKS[0];
        captures &= !RANK_MASKS[0];
        walks &= !RANK_MASKS[0];
    }

    while captures != 0 {
        let end_idx = captures.trailing_zeros() as i8;
        if (idx_to_bb(end_idx) & occ) == 0 {
            // capturing "empty" space.  This is en passant
            moves.push(Move::ep_capture(idx, end_idx));
        } else {
            moves.push(Move::pawn_move(idx, end_idx));
        }
        captures &= captures - 1;
    }

    while walks != 0 {
        let end_idx = walks.trailing_zeros() as i8;
        moves.push(Move::pawn_move(idx, end_idx));
        walks &= walks - 1;
    }

    while promotions != 0 {
        let end_idx = promotions.trailing_zeros() as i8;
        for p in [b'q', b'r', b'b', b'n'] {
            moves.push(Move::promotion(idx, end_idx, p));
        }
        promotions &= promotions - 1;
    }
}

pub fn knight_moves_board(idx: i8) -> u64 {
    unsafe {
        return KNIGHT_MASK[idx as usize];
    }
}

pub fn knight_captures(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let move_board = knight_moves_board(idx);
    get_piece_captures(pos, idx, b'n', move_board, moves);
}

pub fn knight_quiet_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let move_board = knight_moves_board(idx);
    get_piece_quiets(pos, idx, b'n', move_board, moves);
}

pub fn knight_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let move_board = knight_moves_board(idx);
    get_piece_movelist(pos, idx, b'n', move_board, moves);
}

pub fn bishop_moves_board(idx: i8, occ: u64) -> u64 {
    unsafe {
        let occupancy = BISHOP_MASK[idx as usize] & occ;
        let hash = bishop_magic_hash(occupancy, idx as usize);
        return BISHOP_MAGIC_TABLE[idx as usize][hash as usize];
    }
}

pub fn bishop_captures(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = bishop_moves_board(idx, occ);
    get_piece_captures(pos, idx, b'b', move_board, moves);
}

pub fn bishop_quiet_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = bishop_moves_board(idx, occ);
    get_piece_quiets(pos, idx, b'b', move_board, moves);
}

pub fn bishop_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = bishop_moves_board(idx, occ);
    get_piece_movelist(pos, idx, b'b', move_board, moves);
}

pub fn rook_moves_board(idx: i8, occ: u64) -> u64 {
    unsafe {
        let occupancy = ROOK_MASK[idx as usize] & occ;
        let hash = rook_magic_hash(occupancy, idx as usize);
        return ROOK_MAGIC_TABLE[idx as usize][hash as usize];
    }
}

pub fn rook_captures(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = rook_moves_board(idx, occ);
    get_piece_captures(pos, idx, b'r', move_board, moves);
}

pub fn rook_quiet_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = rook_moves_board(idx, occ);
    get_piece_quiets(pos, idx, b'r', move_board, moves);
}

pub fn rook_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = rook_moves_board(idx, occ);
    get_piece_movelist(pos, idx, b'r', move_board, moves);
}

pub fn queen_moves_board(idx: i8, occ: u64) -> u64 {
    return bishop_moves_board(idx, occ) | rook_moves_board(idx, occ);
}

pub fn queen_captures(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = queen_moves_board(idx, occ);
    get_piece_captures(pos, idx, b'q', move_board, moves);
}

pub fn queen_quiet_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = queen_moves_board(idx, occ);
    get_piece_quiets(pos, idx, b'q', move_board, moves);
}

pub fn queen_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = queen_moves_board(idx, occ);
    get_piece_movelist(pos, idx, b'q', move_board, moves);
}

pub fn king_normal_moves_board(idx: i8) -> u64 {
    // let (kx, ky) = idx_to_coord(idx);
    // let mut bb = 0;
    // let (kx, ky) = idx_to_coord(idx);
    // for (dx, dy) in [
    //     (-1, 1), (1, 1), (-1, -1), (1, -1),
    //     (0, 1), (0, -1), (-1, 0), (1, 0)
    // ].iter() {
    //     let (nx, ny) = (kx + dx, ky + dy);
    //     if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
    //         continue;
    //     }
    //     let new_idx = coord_to_idx((nx, ny));
    //     bb |= 1 << new_idx;
    // }
    let (kx, _) = idx_to_coord(idx);
    let mut bb = idx_to_bb(idx);
    bb |= if kx > 0 {bb >> 1} else {0} | if kx < 7 {bb << 1} else {0};
    bb |= (bb << 8) | (bb >> 8);
    bb ^= idx_to_bb(idx);
    return bb;
    // unsafe {
    //     return KING_MASK[idx as usize];
    // }
}

pub fn king_captures(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let move_board = king_normal_moves_board(idx);
   get_piece_captures(pos, idx, b'k', move_board, moves);
}

pub fn king_quiet_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let move_board = king_normal_moves_board(idx);
    get_piece_quiets(pos, idx, b'k', move_board, moves);

    // kingside castle
    if pos.can_castle(pos.side_to_move, false) {
        let end_idx = idx + 2;
        moves.push(Move::piece_move(idx, end_idx, b'k'));
    }
    // queenside castle
    if pos.can_castle(pos.side_to_move, true) {
        let end_idx = idx - 2;
        moves.push(Move::piece_move(idx, end_idx, b'k'));
    }
}

pub fn king_moves(pos: &Bitboard, idx: i8, moves: &mut Vec<Move>) {
    let move_board = king_normal_moves_board(idx);
    get_piece_movelist(pos, idx, b'k', move_board, moves);

    // kingside castle
    if pos.can_castle(pos.side_to_move, false) {
        let end_idx = idx + 2;
        moves.push(Move::piece_move(idx, end_idx, b'k'));
    }
    // queenside castle
    if pos.can_castle(pos.side_to_move, true) {
        let end_idx = idx - 2;
        moves.push(Move::piece_move(idx, end_idx, b'k'));
    }
}

pub fn moves(pos: &Bitboard) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::with_capacity(24);
    let me = pos.side_to_move as usize;
    let mut pawns = pos.pawn[me];
    let mut knights = pos.knight[me];
    let mut bishops = pos.bishop[me];
    let mut rooks = pos.rook[me];
    let mut queens = pos.queen[me];
    let mut kings = pos.king[me];

    while pawns != 0 {
        let idx = pawns.trailing_zeros() as i8;
        pawn_moves(pos, idx, &mut moves);
        pawns &= pawns - 1;
    }

    while knights != 0 {
        let idx = knights.trailing_zeros() as i8;
        knight_moves(pos, idx, &mut moves);
        knights &= knights - 1;
    }

    while bishops != 0 {
        let idx = bishops.trailing_zeros() as i8;
        bishop_moves(pos, idx, &mut moves);
        bishops &= bishops - 1;
    }

    while rooks != 0 {
        let idx = rooks.trailing_zeros() as i8;
        rook_moves(pos, idx, &mut moves);
        rooks &= rooks - 1;
    }

    while queens != 0 {
        let idx = queens.trailing_zeros() as i8;
        queen_moves(pos, idx, &mut moves);
        queens &= queens - 1;
    }

    while kings != 0 {
        let idx = kings.trailing_zeros() as i8;
        king_moves(pos, idx, &mut moves);
        kings &= kings - 1;
    }

    return moves;
}

pub fn quiet_moves(pos: &Bitboard) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::with_capacity(24);

    let me = pos.side_to_move as usize;
    let mut pawns = pos.pawn[me];
    let mut knights = pos.knight[me];
    let mut bishops = pos.bishop[me];
    let mut rooks = pos.rook[me];
    let mut queens = pos.queen[me];
    let mut kings = pos.king[me];

    while pawns != 0 {
        let idx = pawns.trailing_zeros() as i8;
        pawn_quiet_moves(pos, idx, &mut moves);
        pawns &= pawns - 1;
    }

    while knights != 0 {
        let idx = knights.trailing_zeros() as i8;
        knight_quiet_moves(pos, idx, &mut moves);
        knights &= knights - 1;
    }

    while bishops != 0 {
        let idx = bishops.trailing_zeros() as i8;
        bishop_quiet_moves(pos, idx, &mut moves);
        bishops &= bishops - 1;
    }

    while rooks != 0 {
        let idx = rooks.trailing_zeros() as i8;
        rook_quiet_moves(pos, idx, &mut moves);
        rooks &= rooks - 1;
    }

    while queens != 0 {
        let idx = queens.trailing_zeros() as i8;
        queen_quiet_moves(pos, idx, &mut moves);
        queens &= queens - 1;
    }

    while kings != 0 {
        let idx = kings.trailing_zeros() as i8;
        king_quiet_moves(pos, idx, &mut moves);
        kings &= kings - 1;
    }

    return moves;
}

pub fn noisy_moves(pos: &Bitboard) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::with_capacity(24);

    let me = pos.side_to_move as usize;
    let mut pawns = pos.pawn[me];
    let mut knights = pos.knight[me];
    let mut bishops = pos.bishop[me];
    let mut rooks = pos.rook[me];
    let mut queens = pos.queen[me];
    let mut kings = pos.king[me];

    while pawns != 0 {
        let idx = pawns.trailing_zeros() as i8;
        // moves.append(&mut pawn_qmoves(pos, idx));
        pawn_qmoves(pos, idx, &mut moves);
        pawns &= pawns - 1;
    }

    while knights != 0 {
        let idx = knights.trailing_zeros() as i8;
        knight_captures(pos, idx, &mut moves);
        knights &= knights - 1;
    }

    while bishops != 0 {
        let idx = bishops.trailing_zeros() as i8;
        bishop_captures(pos, idx, &mut moves);
        bishops &= bishops - 1;
    }

    while rooks != 0 {
        let idx = rooks.trailing_zeros() as i8;
        rook_captures(pos, idx, &mut moves);
        rooks &= rooks - 1;
    }

    while queens != 0 {
        let idx = queens.trailing_zeros() as i8;
        queen_captures(pos, idx, &mut moves);
        queens &= queens - 1;
    }

    while kings != 0 {
        let idx = kings.trailing_zeros() as i8;
        king_captures(pos, idx, &mut moves);
        kings &= kings - 1;
    }

    return moves;
}

pub fn all_attacks_board(pos: &Bitboard, side: Color) -> u64 {
    let me = side as usize;
    let pawns = pos.pawn[me];
    let mut knights = pos.knight[me];
    let mut bishops = pos.bishop[me];
    let mut rooks = pos.rook[me];
    let mut queens = pos.queen[me];
    let mut kings = pos.king[me];
    let occ = pos.composite[0] | pos.composite[1];

    let mut attacks: u64 = 0;
    attacks |= if side == Color::White {
        ((pawns & !FILE_MASKS[0]) << 7) | ((pawns & !FILE_MASKS[7]) << 9)
    } else {
        ((pawns & !FILE_MASKS[0]) >> 9) | ((pawns & !FILE_MASKS[7]) >> 7)
    };

    while knights != 0 {
        let idx = knights.trailing_zeros() as i8;
        attacks |= knight_moves_board(idx);
        knights &= knights - 1;
    }

    while bishops != 0 {
        let idx = bishops.trailing_zeros() as i8;
        attacks |= bishop_moves_board(idx, occ);
        bishops &= bishops - 1;
    }

    while rooks != 0 {
        let idx = rooks.trailing_zeros() as i8;
        attacks |= rook_moves_board(idx, occ);
        rooks &= rooks - 1;
    }

    while queens != 0 {
        let idx = queens.trailing_zeros() as i8;
        attacks |= queen_moves_board(idx, occ);
        queens &= queens - 1;
    }

    while kings != 0 {
        let idx = kings.trailing_zeros() as i8;
        attacks |= king_normal_moves_board(idx);
        kings &= kings - 1;
    }

    return attacks;
}
