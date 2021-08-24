use crate::magic::*;
use crate::util::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Move {
    pub start: i32,
    pub end: i32,
    pub piece: u8,
    pub is_ep: bool,
    pub ep_file: i32,
    pub promote_to: u8,
    pub is_null: bool
}

impl Move {
    pub fn get_repr(&self) -> String {
        // UCI compatible representation of move
        if self.is_null {
            return "0000";
        }
        let start = idx_to_str(self.start);
        let end = idx_to_str(self.end);
        let mut promote = "".to_string();

        if self.promote_to != 0 {
            promote = (self.promote_to as char).to_string();
        }

        return format!("{}{}{}", start, end, promote)
    }

    pub fn piece_move(start: i32, end: i32, piece: u8) -> Move {
        // standard type of move for all pieces except pawns
        Move {
            start: start,
            end: end,
            piece: piece,
            ep_file: -1,
            is_ep: false,
            promote_to: 0,
            is_null: false,
        }
    }

    pub fn pawn_move(start: i32, end: i32) -> Move {
        // pawn walk, double walk, or typical capture
        let mut ep_file = -1;
        if (end - start).abs() == 16 {
            ep_file = start % 8;
        }
        Move {
            start: start,
            end: end,
            piece: b'p',
            is_ep: false,
            ep_file: ep_file,
            promote_to: 0,
            is_null: false
        }
    }

    // Next are the pawn move constructors
    pub fn ep_capture(start: i32, end: i32) -> Move {
        // capture en-passant
        Move {
            start: start,
            end: end,
            piece: b'p',
            ep_file: -1,
            is_ep: true,
            promote_to: 0,
            is_null: false,
        }
    }

    pub fn promotion(start: i32, end: i32, piece: u8) -> Move {
        Move {
            start: start,
            end: end,
            piece: b'p',
            is_ep: false,
            ep_file: -1,
            promote_to: piece,
            is_null: false
        }
    }

    pub fn null_move() -> Move {
        // technically there are a lot
        // of "null moves" valid in this scheme
        // (i.e. any move with is_null set to true)
        // this is merely one of them.
        Move {
            start: 0,
            end: 0,
            piece: 0,
            is_ep: false,
            ep_file: -1,
            promote_to: 0,
            is_null: true
        }
    }
}

impl fmt::Display for Mv {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_repr())
    }
}

fn get_piece_movelist(pos: &Bitboard, idx: i32, piece: u8, move_board: u64) -> Vec<Move> {
    // gets *all* pseudo-legal moves from the moveboard (not just quiet ones)
    // this should appropriately filter out capturing own pieces, etc.
    let mut own_occ = pos.composite[pos.side_to_move as usize];
    let mut move_board = move_board & !own_occ;
    let mut moves: Vec<Move> = Vec::new();

    while move_board != 0 {
        let end_idx = move_board.trailing_zeros();
        moves.push(Move::piece_move(idx, end_idx, piece));
        move_board &= move_board - 1; // cute trick I learned from Expositor code
    }

    return moves;
}

fn get_piece_captures(pos: &Bitboard, idx: i32, piece: u8, move_board: u64) -> Vec<Move> {
    // this should appropriately filter out capturing own pieces, etc.
    let enemy_occ = pos.composite[!pos.side_to_move as usize];
    let mut move_board = move_board & enemy_occ;
    let mut captures: Vec<Move> = Vec::new();

    while move_board != 0 {
        let end_idx = move_board.trailing_zeros();
        captures.push(Move::piece_move(idx, end_idx, piece));
        move_board &= move_board - 1;
    }

    return captures;
}

pub fn pawn_walk_board(occ: u64, idx: i32, side_to_move: Color) {
    let mut move_board: u64 = 0;
    if side_to_move == Color::White {
        // walk forward one if unoccupied
        move_board = idx_to_bb(idx + 8) & !occ;
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

pub fn pawn_capture_board(enemy_occ: u64, idx: i32, ep_file: i32, side_to_move: Color) {
    let mut move_board: u64 = 0;
    let start_idx_bb = idx_to_bb(idx);
    let attacks = if side_to_move == Color::White {
        ((start_idx_bb & !FILE_MASKS[0]) << 7) | ((start_idx_bb & !FILE_MASKS[7]) << 9)
    } else {
        ((start_idx_bb & !FILE_MASKS[0]) >> 9) | ((start_idx_bb & !FILE_MASKS[7]) >> 7)
    };
    let en_passant = if ep_file != -1 {attacks & FILE_MASKS[ep_file]} else {0};
    let captures = (attacks & enemy_occ) | en_passant;

    return captures;
}

// captures and promotions
pub fn pawn_qmoves(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let occ = pos.composite[0] | pos.composite[1];
    let enemy_occ = pos.composite[!pos.side_to_move as usize];
    let mut promotions = 0;
    let mut captures = pawn_capture_board(enemy_occ, idx, pos.ep_file, pos.side_to_move);
    if pos.side_to_move == Color::White {
        if idx_to_bb(idx) & RANK_MASKS[6] {
            promotions = pawn_walk_board(occ, idx, pos.side_to_move);
            promotions |= captures & RANK_MASKS[7];
            captures &= !RANK_MASKS[7];
        }
    } else {
        if idx_to_bb(idx) & RANK_MASKS[1] {
            promotions = pawn_walk_board(occ, idx, pos.side_to_move);
            promotions |= captures & RANK_MASKS[0];
            captures &= !RANK_MASKS[0];
        }
    }

    while captures != 0 {
        let end_idx = captures.trailing_zeros();
        if (idx_to_bb(end_idx) & occ) == 0 {
            // capturing "empty" space.  This is en passant
            moves.push(Move::ep_capture(idx, end_idx));
        } else {
            moves.push(Move::pawn_move(idx, end_idx));
        }
        captures &= captures - 1;
    }

    while promotions != 0 {
        let end_idx = promotions.trailing_zeros();
        for p in [b'q', b'r', b'b', b'n'] {
            moves.push(Move::promotion(idx, end_idx, p));
        }
    }
    return moves;
};

// all moves
pub fn pawn_moves() -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let occ = pos.composite[0] | pos.composite[1];
    let enemy_occ = pos.composite[!pos.side_to_move as usize];

    let mut promotions = 0;
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
        let end_idx = captures.trailing_zeros();
        if (idx_to_bb(end_idx) & occ) == 0 {
            // capturing "empty" space.  This is en passant
            moves.push(Move::ep_capture(idx, end_idx));
        } else {
            moves.push(Move::pawn_move(idx, end_idx));
        }
        captures &= captures - 1;
    }

    while walks != 0 {
        let end_idx = walks.trailing_zeros();
        moves.push(Move::pawn_move(idx, end_idx));
        walks &= walks - 1;
    }

    while promotions != 0 {
        let end_idx = promotions.trailing_zeros();
        for p in [b'q', b'r', b'b', b'n'] {
            moves.push(Move::promotion(idx, end_idx, p));
        }
    }

    return moves;
};

pub fn knight_moves_board(idx: i32) -> u64 {
    knight_mask[idx as usize]
};

pub fn knight_captures(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let move_board = knight_moves_board(idx);
    return get_piece_captures(pos, idx, b'n', move_board);
};

pub fn knight_moves(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let move_board = knight_moves_board(idx);
    return get_piece_movelist(pos, idx, b'n', move_board);
};

pub fn bishop_moves_board(occ: u64, idx: i32) -> u64 {
    let occupancy = bishop_mask[idx as usize] & occ;
    let hash = bishop_magic_hash(occupancy, idx as usize);
    return bishop_magic_table[idx as usize][hash as usize];
};

pub fn bishop_captures(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = bishop_moves_board(occ, idx);
    return get_piece_captures(pos, idx, b'b', move_board);
};

pub fn bishop_moves(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = bishop_moves_board(occ, idx);
    return get_piece_movelist(pos, idx, b'b', move_board);
};

pub fn rook_moves_board(occ: u64, idx: i32) -> u64 {
    let occupancy = rook_mask[idx as usize] & occ
    let hash = rook_magic_hash(occupancy, idx as usize);
    return rook_magic_table[idx as usize][hash as usize];
};

pub fn rook_captures(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = rook_moves_board(occ, idx);
    return get_piece_captures(pos, idx, b'r', move_board);
};

pub fn rook_moves(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = rook_moves_board(occ, idx);
    return get_piece_movelist(pos, idx, b'r', move_board);
};

pub fn queen_moves_board(occ: u64, idx: i32) -> u64 {
    return bishop_moves_board(pos, idx) | rook_moves_board(pos, idx);
};

pub fn queen_captures(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = queen_moves_board(occ, idx);
    return get_piece_captures(pos, idx, b'q', move_board);
};

pub fn queen_moves(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let occ = pos.composite[0] | pos.composite[1];
    let move_board = queen_moves_board(occ, idx);
    return get_piece_movelist(pos, idx, b'q', move_board);
};

pub fn king_normal_moves_board(idx: i32) -> u64 {
    king_mask[idx as usize]
};

pub fn king_captures(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let move_board = king_moves_board(idx);
    return get_piece_captures(pos, idx, b'k', move_board);
};

pub fn king_moves(pos: &Bitboard, idx: i32) -> Vec<Move> {
    let move_board = king_moves_board(idx);
    let mut normal_moves = get_piece_movelist(pos, idx, b'k', move_board);

    // TODO: castling
};
