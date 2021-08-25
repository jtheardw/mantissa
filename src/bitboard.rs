use crate::movegen::*;
use crate::util::*;

const WHITE_KINGSIDE_CR_MASK: u8 = 0b1000;
const WHITE_QUEENSIDE_CR_MASK: u8 = 0b0100;
const BLACK_KINGSIDE_CR_MASK: u8 = 0b0010;
const BLACK_QUEENSIDE_CR_MASK: u8 = 0b0001;

pub struct Bitboard {
    pub side_to_move: Color,

    pub king: [u64; 2],
    pub queen: [u64; 2],
    pub rook: [u64; 2],
    pub bishop: [u64; 2],
    pub knight: [u64; 2],
    pub pawn: [u64; 2],
    pub composite: [u64; 2],

    castling_rights: u8,         // 4-bit number KQkq
    pub ep_file: i32,

    pub history: Vec<u64>,

    pawn_history: Vec<u64>,
    ep_stack: Vec<i32>,
    cap_stack: Vec<u8>,
    castling_rights_stack: Vec<u8>,

    hash: u64,
    pawn_hash: u64,
}

impl Bitboard {
    // constructors
    pub fn default_board() -> Bitboard{
        Bitboard {
            side_to_move: Color::White,

            // black, white
            king: [coord_to_bb((4, 7)), coord_to_bb((4, 0))],
            queen: [coord_to_bb((3, 7)), coord_to_bb((3, 0))],
            rook: [
                coord_to_bb((0, 7)) | coord_to_bb((7, 7)),
                coord_to_bb((0, 0)) | coord_to_bb((7, 0))
            ],
            bishop: [
                coord_to_bb((2, 7)) | coord_to_bb((5, 7)),
                coord_to_bb((2, 0)) | coord_to_bb((5, 0))
            ],
            knight: [
                coord_to_bb((1, 7)) | coord_to_bb((6, 7)),
                coord_to_bb((1, 0)) | coord_to_bb((6, 0))
            ],
            pawn: [RANK_MASKS[6], RANK_MASKS[1]],

            composite: [
                RANK_MASKS[7] | RANK_MASKS[6],
                RANK_MASKS[0] | RANK_MASKS[1]
            ],

            castling_rights: 0b1111,
            ep_file: -1,

            history: Vec::new(),
            pawn_history: Vec::new(),
            ep_stack: Vec::new(),
            cap_stack: Vec::new(),
            castling_rights_stack: Vec::new(),

            hash: 0,// get_starting_hash(),
            pawn_hash: 0//get_starting_pawn_hash(),
        }
    }

    pub fn from_position(fen: String) -> Bitboard {
        let mut black_king: u64 = 0;
        let mut white_king: u64 = 0;

        let mut black_queen: u64 = 0;
        let mut white_queen: u64 = 0;

        let mut black_rook: u64 = 0;
        let mut white_rook: u64 = 0;

        let mut black_bishop: u64 = 0;
        let mut white_bishop: u64 = 0;

        let mut black_knight: u64 = 0;
        let mut white_knight: u64 = 0;

        let mut black_pawn: u64 = 0;
        let mut white_pawn: u64 = 0;

        let mut rank: i32 = 7;
        let mut file: i32 = 0;

        let mut fen_split = fen.split(' ');
        let positions = match fen_split.next() {
            Some(s) => String::from(s),
            None => panic!("bad FEN string")
        };

        for c in positions.as_bytes().iter() {
            let c = *c;
            match c {
                b'k' => {black_king |= coord_to_bb((file, rank)); file += 1;},
                b'K' => {white_king |= coord_to_bb((file, rank)); file += 1;},

                b'q' => {black_queen |= coord_to_bb((file, rank)); file += 1;},
                b'Q' => {white_queen |= coord_to_bb((file, rank)); file += 1;},

                b'r' => {black_rook |= coord_to_bb((file, rank)); file += 1;},
                b'R' => {white_rook |= coord_to_bb((file, rank)); file += 1;},

                b'b' => {black_bishop |= coord_to_bb((file, rank)); file += 1;},
                b'B' => {white_bishop |= coord_to_bb((file, rank)); file += 1;},

                b'n' => {black_knight |= coord_to_bb((file, rank)); file += 1;},
                b'N' => {white_knight |= coord_to_bb((file, rank)); file += 1;},

                b'p' => {black_pawn |= coord_to_bb((file, rank)); file += 1;},
                b'P' => {white_pawn |= coord_to_bb((file, rank)); file += 1;},

                b'/' => {rank -= 1; file = 0;},
                b'1' => {file += 1},
                b'2' => {file += 2},
                b'3' => {file += 3},
                b'4' => {file += 4},
                b'5' => {file += 5},
                b'6' => {file += 6},
                b'7' => {file += 7},
                b'8' => {file += 8},
                _ => {}
            };
        }

        let mut side_to_move = Color::White;

        match fen_split.next() {
            Some(s) => {if s == "b" {side_to_move = Color::Black;}},
            None => panic!("Bad FEN string.  Missing side to move.")
        };

        let castling_rights_str = match fen_split.next() {
            Some(s) => s,
            None => panic!("Bad FEN string. Missing castling rights")
        };

        let mut castling_rights: u8 = 0;
        // KQkq
        if castling_rights_str != "-" {
            for c in String::from(castling_rights_str).as_bytes().iter() {
                match *c {
                    b'K' => {castling_rights |= WHITE_KINGSIDE_CR_MASK;},
                    b'Q' => {castling_rights |= WHITE_QUEENSIDE_CR_MASK;},
                    b'k' => {castling_rights |= BLACK_KINGSIDE_CR_MASK;},
                    b'q' => {castling_rights |= BLACK_QUEENSIDE_CR_MASK;},
                    _ => panic!("Malformatted Castling Rights")
                };
            }
        }

        let ep_str = match fen_split.next() {
            Some(s) => s,
            None => panic!("Bad FEN string.  Missing en passant file")
        };

        let mut ep_file: i32 = -1;
        if ep_str != "-" {
            let ep_str = String::from(ep_str);
            let ep_chars = ep_str.as_bytes();
            let file = ep_chars[0] - b'a';
            ep_file = file as i32;
        }

        let white_composite = white_king | white_queen | white_rook | white_bishop | white_knight | white_pawn;
        let black_composite = black_king | black_queen | black_rook | black_bishop | black_knight | black_pawn;

        let bitboard = Bitboard {
            side_to_move: side_to_move,
            king: [black_king, white_king],
            queen: [black_queen, white_queen],
            rook: [black_rook, white_rook],
            bishop: [black_bishop, white_bishop],
            knight: [black_knight, white_knight],
            pawn: [black_pawn, white_pawn],
            composite: [black_composite, white_composite],

            castling_rights: castling_rights,
            ep_file: ep_file,

            history: Vec::new(),
            pawn_history: Vec::new(),
            ep_stack: Vec::new(),
            cap_stack: Vec::new(),
            castling_rights_stack: Vec::new(),

            hash: 0,            // for the moment, we fill it in late
            pawn_hash: 0
        };

        // update things that have to be calculated from a board
        // bitboard.hash = get_position_hash(&bitboard);
        // bitboard.pawn_hash = get_position_pawn_hash(&bitboard);

        return bitboard;
    }

    pub fn thread_copy(&self) -> Bitboard {
        // make a copy usable by another thread
        // won't include certain things like the capture
        // stack, but will include history for threefold
        let mut history: Vec<u64> = Vec::new();
        for hash in &self.history {
            history.push(*hash);
        }

        Bitboard {
            side_to_move: self.side_to_move,

            king: self.king,
            queen: self.queen,
            rook: self.rook,
            bishop: self.bishop,
            knight: self.knight,
            pawn: self.pawn,
            composite: self.composite,

            castling_rights: self.castling_rights,
            ep_file: self.ep_file,

            history: history,

            pawn_history: Vec::new(),
            ep_stack: Vec::new(),
            cap_stack: Vec::new(),
            castling_rights_stack: Vec::new(),

            hash: self.hash,
            pawn_hash: self.pawn_hash,
        }
    }

    pub fn is_square_attacked(&self, idx: i32, side_to_move: Color) -> bool {
        // process:
        // create "virtual pieces" at the idx and see if they can
        // attack enemy pieces of the appropriate types.  If so,
        // there is an attack on this square
        let enemy_side = !side_to_move as usize;
        let all_composite = self.composite[0] | self.composite[1];

        // king
        // normally irrelevant for check, but might be relevant for other
        // uses, e.g. castling, SEE
        if (king_normal_moves_board(idx) & self.king[enemy_side]) != 0 { return true; }

        // knights
        if (knight_moves_board(idx) & self.knight[enemy_side]) != 0 { return true; }

        // pawns
        if (pawn_attack_board(idx, side_to_move) & self.pawn[enemy_side]) != 0 {
            return true;
        }

        // bishops + queens
        if (bishop_moves_board(idx, all_composite) & (self.bishop[enemy_side] | self.queen[enemy_side])) != 0 {
            return true;
        }

        // rooks + queens
        if (rook_moves_board(idx, all_composite) & (self.rook[enemy_side] | self.queen[enemy_side])) != 0 {
            return true;
        }

        return false;
    }

    pub fn is_check(&self, side: Color) -> bool {
        return self.is_square_attacked(self.king[side as usize].trailing_zeros() as i32, side);
    }

    pub fn can_castle(&self, side: Color, queenside: bool) -> bool {
        let relevant_castling_right = match (side, queenside) {
            (Color::White, false) => { self.castling_rights & WHITE_KINGSIDE_CR_MASK },
            (Color::White, true) => { self.castling_rights & WHITE_QUEENSIDE_CR_MASK },
            (Color::Black, false) => { self.castling_rights & BLACK_KINGSIDE_CR_MASK },
            (Color::Black, true) => { self.castling_rights & BLACK_QUEENSIDE_CR_MASK }
        };

        if relevant_castling_right == 0 {
            // can't castle this way
            return false;
        }

        // TODO this might not be flexible to some game formats
        let king_idx = if side == Color::White {4} else {60};
        let occupancy_mask = if queenside {
            0b111 << (king_idx - 3)
        } else {
            0b11 << (king_idx + 1)
        };

        if ((self.composite[0] | self.composite[1]) & occupancy_mask) != 0 {
            // something in the way
            return false;
        }

        // can't castle while in check
        if self.is_check(side) { return false; }

        // check if in check at the in-between square
        if queenside {
            return !self.is_square_attacked(king_idx - 1, side);
        } else {
            return !self.is_square_attacked(king_idx + 1, side);
        }
    }

    pub fn piece_at_square(&self, idx: i32, side: Color) -> u8 {
        let idx_board = idx_to_bb(idx);
        let side = side as usize;
        if (idx_board & self.composite[side]) == 0 {
            // no piece here
            return 0;
        }
        if (idx_board & self.pawn[side]) != 0 {
            return b'p';
        }
        if (idx_board & self.knight[side]) != 0 {
            return b'n';
        }
        if (idx_board & self.bishop[side]) != 0 {
            return b'b';
        }
        if (idx_board & self.rook[side]) != 0 {
            return b'r';
        }
        if (idx_board & self.queen[side]) != 0 {
            return b'q';
        }
        if (idx_board & self.king[side]) != 0 {
            return b'k';
        }
        panic!("Found a piece that should exist at idx {} but doesn't", idx);
    }

    fn void_castling_rights(&mut self, start_idx: i32, end_idx: i32) {
        let mut new_castling_rights = self.castling_rights;
        if new_castling_rights == 0 { return; }
        if start_idx == 4 {
            // white king square
            new_castling_rights &= !(WHITE_KINGSIDE_CR_MASK | WHITE_QUEENSIDE_CR_MASK);
        } else if start_idx == 60 {
            // black king square
            new_castling_rights &= !(BLACK_KINGSIDE_CR_MASK | BLACK_QUEENSIDE_CR_MASK);
        } else {
            let move_bb = idx_to_bb(start_idx) | idx_to_bb(end_idx);
            if (move_bb & coord_to_bb((0, 0))) != 0 {
                new_castling_rights &= !WHITE_QUEENSIDE_CR_MASK;
            }
            if (move_bb * coord_to_bb((7, 0))) != 0 {
                new_castling_rights &= !WHITE_KINGSIDE_CR_MASK;
            }
            if (move_bb * coord_to_bb((0, 7))) != 0 {
                new_castling_rights &= !BLACK_QUEENSIDE_CR_MASK;
            }
            if (move_bb * coord_to_bb((7, 7))) != 0 {
                new_castling_rights &= !BLACK_KINGSIDE_CR_MASK;
            }
        }
        self.castling_rights = new_castling_rights;
    }

    pub fn do_move(&mut self, mv: &Move) {
        // push stacks

        // check capture
        // check castling
        // check promotion

        // move piece

        // update castling rights
        // update composites
        // update hash
        // flip turn

        self.history.push(self.hash);
        self.pawn_history.push(self.pawn_hash);
        self.ep_stack.push(self.ep_file);
        self.castling_rights_stack.push(self.castling_rights);

        let start_point: u64 = idx_to_bb(mv.start);
        let end_point: u64 = idx_to_bb(mv.end);
        let me = self.side_to_move as usize;
        let them = !self.side_to_move as usize;

        // determine captured piece for all non-ep captures
        let mut captured_piece: u8 = self.piece_at_square(mv.end, !self.side_to_move);

        // check ep first
        if mv.is_ep {
            captured_piece = b'p';
            let actual_pawn_idx = if self.side_to_move == Color::White {
                coord_to_idx((self.ep_file, 4))
            } else {
                coord_to_idx((self.ep_file, 3))
            };

            // remove enemy pawn
            self.pawn[them] ^= idx_to_bb(actual_pawn_idx);
            // TODO: hash
        } else if captured_piece != 0 {
            match captured_piece {
                b'p' => { self.pawn[them] ^= end_point; },
                b'n' => { self.knight[them] ^= end_point; },
                b'b' => { self.bishop[them] ^= end_point; },
                b'r' => { self.rook[them] ^= end_point; },
                b'q' => { self.queen[them] ^= end_point; },
                _ => panic!("Captured uncapturable piece {}!", captured_piece)
            };
        }
        self.cap_stack.push(captured_piece);

        // castling
        if mv.piece == b'k' && (mv.end - mv.start).abs() == 2 {
            let old_rook_idx: i32 = if mv.end > mv.start { mv.start + 3 } else { mv.start - 4 };
            let new_rook_idx: i32 = if mv.end > mv.start { mv.start + 1 } else { mv.start - 1 };

            // move the rook
            let rook_mask = idx_to_bb(old_rook_idx) | idx_to_bb(new_rook_idx);
            self.rook[me] ^= rook_mask;
        }

        // move piece
        if mv.piece == b'p' && mv.promote_to != 0 {
            self.pawn[me] ^= start_point;
            match mv.promote_to {
                b'q' => { self.queen[me] |= end_point; },
                b'r' => { self.rook[me] |= end_point; },
                b'b' => { self.bishop[me] |= end_point; },
                b'n' => { self.knight[me] |= end_point; },
                _ => { panic!("illegal promotion on mv {}", mv); }
            }
        } else {
            let move_mask = start_point | end_point;
            match mv.piece {
                b'k' => { self.king[me] ^= move_mask; },
                b'q' => { self.queen[me] ^= move_mask; },
                b'r' => { self.rook[me] ^= move_mask; },
                b'b' => { self.bishop[me] ^= move_mask; },
                b'n' => { self.knight[me] ^= move_mask; },
                b'p' => { self.pawn[me] ^= move_mask; },
                _ => { panic!("moved nonexistent piece {} in mv {}", mv.piece, mv); }
            }
        }

        // TODO update hash

        // update castling rights
        self.void_castling_rights(mv.start, mv.end);

        // update ep_file
        self.ep_file = mv.ep_file;

        // update composite
        self.composite[me] = self.pawn[me] | self.knight[me] | self.bishop[me] |
            self.rook[me] | self.queen[me] | self.king[me];

        self.composite[them] = self.pawn[them] | self.knight[them] | self.bishop[them] |
            self.rook[them] | self.queen[them] | self.king[them];

        self.side_to_move = !self.side_to_move;
    }

    pub fn undo_move(&mut self, mv: &Move) {
        // do everything backwards from above
        self.side_to_move = !self.side_to_move;

        let start_point: u64 = idx_to_bb(mv.start);
        let end_point: u64 = idx_to_bb(mv.end);
        let me = self.side_to_move as usize;
        let them = !self.side_to_move as usize;

        let captured_piece = match self.cap_stack.pop() {
            Some(p) => p,
            None => panic!("empty capture stack!")
        };
        self.ep_file = match self.ep_stack.pop() {
            Some(p) => p,
            None => panic!("empty ep stack!")
        };
        self.castling_rights = match self.castling_rights_stack.pop() {
            Some(p) => p,
            None => panic!("empty cr stack!")
        };

        if mv.is_ep {
            let actual_pawn_idx = if self.side_to_move == Color::White {
                coord_to_idx((self.ep_file, 4))
            } else {
                coord_to_idx((self.ep_file, 3))
            };

            // replace enemy pawn
            self.pawn[them] ^= idx_to_bb(actual_pawn_idx);
        } else if captured_piece != 0 {
            match captured_piece {
                b'p' => { self.pawn[them] ^= end_point; },
                b'n' => { self.knight[them] ^= end_point; },
                b'b' => { self.bishop[them] ^= end_point; },
                b'r' => { self.rook[them] ^= end_point; },
                b'q' => { self.queen[them] ^= end_point; },
                _ => panic!("Captured uncapturable piece {}!", captured_piece)
            };
        }

        // castling
        // castling
        if mv.piece == b'k' && (mv.end - mv.start).abs() == 2 {
            let old_rook_idx: i32 = if mv.end > mv.start { mv.start + 3 } else { mv.start - 4 };
            let new_rook_idx: i32 = if mv.end > mv.start { mv.start + 1 } else { mv.start - 1 };

            // move the rook
            let rook_mask = idx_to_bb(old_rook_idx) | idx_to_bb(new_rook_idx);
            self.rook[me] ^= rook_mask;
        }

        // move piece
        if mv.piece == b'p' && mv.promote_to != 0 {
            self.pawn[me] ^= start_point;
            match mv.promote_to {
                b'q' => { self.queen[me] ^= end_point; },
                b'r' => { self.rook[me] ^= end_point; },
                b'b' => { self.bishop[me] ^= end_point; },
                b'n' => { self.knight[me] ^= end_point; },
                _ => { panic!("illegal promotion on mv {}", mv); }
            }
        } else {
            let move_mask = start_point | end_point;
            match mv.piece {
                b'k' => { self.king[me] ^= move_mask; },
                b'q' => { self.queen[me] ^= move_mask; },
                b'r' => { self.rook[me] ^= move_mask; },
                b'b' => { self.bishop[me] ^= move_mask; },
                b'n' => { self.knight[me] ^= move_mask; },
                b'p' => { self.pawn[me] ^= move_mask; },
                _ => { panic!("moved nonexistent piece {} in mv {}", mv.piece, mv); }
            }
        }

        // update composite
        self.composite[me] = self.pawn[me] | self.knight[me] | self.bishop[me] |
            self.rook[me] | self.queen[me] | self.king[me];

        self.composite[them] = self.pawn[them] | self.knight[them] | self.bishop[them] |
            self.rook[them] | self.queen[them] | self.king[them];

        self.hash = match self.history.pop() {
            Some(p) => p,
            None => panic!("History stack empty!")
        };
        self.pawn_hash = match self.pawn_history.pop() {
            Some(p) => p,
            None => panic!("Pawn History stack empty!")
        }
    }
}
