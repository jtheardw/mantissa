use crate::movegen::*;
use crate::moveutil::*;
use crate::util::*;
use crate::zobrist::*;

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

    pub castling_rights: u8,         // 4-bit number KQkq
    pub ep_file: i32,

    pub history: Vec<u64>,

    pawn_history: Vec<u64>,
    ep_stack: Vec<i32>,
    cap_stack: Vec<u8>,
    castling_rights_stack: Vec<u8>,
    halfmove_stack: Vec<u8>,

    pub halfmove: u8,

    pub hash: u64,
    pub pawn_hash: u64,
}

impl Bitboard {
    // constructors
    pub fn default_board() -> Bitboard {
        let mut bitboard = Bitboard {
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
            halfmove_stack: Vec::new(),

            halfmove: 0,
            hash: 0,
            pawn_hash: 0
        };

        bitboard.hash = calculate_hash(&bitboard);
        bitboard.pawn_hash = calculate_pawn_hash(&bitboard);

        return bitboard;
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

        let halfmove: u8 = match fen_split.next() {
            Some(p) => match p.trim().parse::<u8>() {
                Ok(num) => num,
                Err(_) => panic!("Bad FEN string.  Failed to parse halfmove clock.")
            },
            None => panic!("Bad FEN string.  Missing halfmove clock")
        };

        let white_composite = white_king | white_queen | white_rook | white_bishop | white_knight | white_pawn;
        let black_composite = black_king | black_queen | black_rook | black_bishop | black_knight | black_pawn;

        let mut bitboard = Bitboard {
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
            halfmove_stack: Vec::new(),

            halfmove: halfmove,
            hash: 0,            // for the moment, we fill it in late
            pawn_hash: 0
        };

        // update things that have to be calculated from a board
        bitboard.hash = calculate_hash(&bitboard);
        bitboard.pawn_hash = calculate_pawn_hash(&bitboard);

        return bitboard;
    }

    pub fn fen(&self) -> String {
        let mut fen_str = String::new();
        let mut acc = 0;

        for i in 0..8 {
            for j in 0..8 {
                let r = 7 - i;
                let f = j;
                let idx = coord_to_idx((f, r));

                // try to find white piece, then black piece
                let mut piece_byte = self.piece_at_square(idx, Color::White);
                let piece = if piece_byte != 0 {
                    (piece_byte as char).to_ascii_uppercase()
                } else {
                    piece_byte = self.piece_at_square(idx, Color::Black);
                    if piece_byte != 0 {
                        piece_byte as char
                    } else {
                        'a'     // :C
                    }
                };

                if piece_byte != 0 {
                    if acc != 0 {
                        fen_str.push_str(format!("{}", acc).as_str());
                    }
                    fen_str.push(piece);
                    acc = 0;
                } else {
                    acc += 1;
                }
            }
            if acc != 0 {
                fen_str.push_str(format!("{}", acc).as_str());
            }
            if i != 7 {
                fen_str.push('/');
            }
            acc = 0;
        }

        let side_to_move = if self.side_to_move == Color::White { "w" } else { "b" };

        let mut castling_rights = String::new();
        if self.castling_rights == 0 {
           castling_rights.push('-');
        } else {
            if self.castling_rights & WHITE_KINGSIDE_CR_MASK != 0 { castling_rights.push('K'); }
            if self.castling_rights & WHITE_QUEENSIDE_CR_MASK != 0 { castling_rights.push('Q'); }
            if self.castling_rights & BLACK_KINGSIDE_CR_MASK != 0 { castling_rights.push('k'); }
            if self.castling_rights & BLACK_QUEENSIDE_CR_MASK != 0 { castling_rights.push('q'); }
        }

        let mut ep = String::new();
        if self.ep_file == -1 {
            ep.push('-');
        } else {
            ep.push_str(idx_to_str(coord_to_idx((self.ep_file, if self.side_to_move == Color::White { 5 } else { 2 }))).as_str());
        }

        let move_clock = format!("{}", 1 + (self.history.len() / 2));

        let halfmove_clock = self.halfmove;

        fen_str = format!("{} {} {} {} {} {}", fen_str, side_to_move, castling_rights, ep, halfmove_clock, move_clock);
        return fen_str;
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
            halfmove_stack: Vec::new(),

            halfmove: self.halfmove,
            hash: self.hash,
            pawn_hash: self.pawn_hash,
        }
    }

    pub fn is_square_attacked(&self, idx: i8, side_to_move: Color) -> bool {
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
        return self.is_square_attacked(self.king[side as usize].trailing_zeros() as i8, side);
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

    pub fn piece_at_square(&self, idx: i8, side: Color) -> u8 {
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

    fn void_castling_rights(&mut self, start_idx: i8, end_idx: i8) {
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
            if (move_bb & coord_to_bb((7, 0))) != 0 {
                new_castling_rights &= !WHITE_KINGSIDE_CR_MASK;
            }
            if (move_bb & coord_to_bb((0, 7))) != 0 {
                new_castling_rights &= !BLACK_QUEENSIDE_CR_MASK;
            }
            if (move_bb & coord_to_bb((7, 7))) != 0 {
                new_castling_rights &= !BLACK_KINGSIDE_CR_MASK;
            }
        }
        self.castling_rights = new_castling_rights;
    }

    pub fn do_null_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        self.history.push(self.hash);
        self.ep_stack.push(self.ep_file);
        self.ep_file = -1;
        self.hash ^= null_move_hash();
    }

    pub fn undo_null_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        self.hash = match self.history.pop() {
            Some(p) => p,
            None => panic!("empty history!")
        };
        self.ep_file = match self.ep_stack.pop() {
            Some(p) => p,
            None => panic!("empty ep stack!")
        };
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
        if captured_piece == 0 && mv.is_pawn_cap() {
            captured_piece = b'p';
            let actual_pawn_idx = if self.side_to_move == Color::White {
                coord_to_idx((self.ep_file, 4))
            } else {
                coord_to_idx((self.ep_file, 3))
            };

            // remove enemy pawn
            self.pawn[them] ^= idx_to_bb(actual_pawn_idx);
            self.hash ^= en_passant_hash(actual_pawn_idx as i32, !self.side_to_move);
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
            let old_rook_idx: i8 = if mv.end > mv.start { mv.start + 3 } else { mv.start - 4 };
            let new_rook_idx: i8 = if mv.end > mv.start { mv.start + 1 } else { mv.start - 1 };

            // move the rook
            let rook_mask = idx_to_bb(old_rook_idx) | idx_to_bb(new_rook_idx);
            self.rook[me] ^= rook_mask;
            self.hash ^= simple_move_hash(b'r', old_rook_idx as i32, new_rook_idx as i32, self.side_to_move);
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

        let old_castling_rights = self.castling_rights;
        // update castling rights
        self.void_castling_rights(mv.start, mv.end);

        self.hash ^= update_hash(
            mv.piece,
            mv.start,
            mv.end,
            captured_piece,
            mv.promote_to,
            self.ep_file,
            mv.ep_file(),
            old_castling_rights,
            self.castling_rights,
            self.side_to_move
        );

        self.pawn_hash ^= update_pawn_hash(
            mv.piece,
            mv.start,
            mv.end,
            captured_piece,
            mv.promote_to,
            self.side_to_move
        );

        // update ep_file
        self.ep_file = mv.ep_file();

        // update composite
        self.composite[me] = self.pawn[me] | self.knight[me] | self.bishop[me] |
            self.rook[me] | self.queen[me] | self.king[me];

        self.composite[them] = self.pawn[them] | self.knight[them] | self.bishop[them] |
            self.rook[them] | self.queen[them] | self.king[them];

        if captured_piece != 0 || mv.piece == b'p' {
            self.halfmove_stack.push(self.halfmove);
            self.halfmove = 0;
        } else {
            self.halfmove += 1;
        }

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


        if mv.is_pawn_cap() && captured_piece == b'p' && (mv.end as i32 % 8) == self.ep_file && ((self.side_to_move == Color::White && mv.end / 8 == 5) || (self.side_to_move == Color::Black && mv.end / 8 == 2)) {
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
        if mv.piece == b'k' && (mv.end - mv.start).abs() == 2 {
            let old_rook_idx: i8 = if mv.end > mv.start { mv.start + 3 } else { mv.start - 4 };
            let new_rook_idx: i8 = if mv.end > mv.start { mv.start + 1 } else { mv.start - 1 };

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
        };

        if mv.piece == b'p' || captured_piece != 0 {
            self.halfmove = match self.halfmove_stack.pop() {
                Some(p) => p,
                None => panic!("Could not revert halfmove clock!")
            }
        } else {
            self.halfmove -= 1;
        }
    }

    pub fn is_repetition(&self) -> bool {
        self.history.iter().filter(|&n| *n == self.hash).count() > 0
    }

    pub fn is_quiet(&self) -> bool {
        let me = self.side_to_move as usize;
        let them = !self.side_to_move as usize;
        let occ = self.composite[0] | self.composite[1];
        let enemy_occ = self.composite[them];

        let mut queens = self.queen[me];
        while queens != 0 {
            let idx = queens.trailing_zeros() as i8;
            let atks = queen_moves_board(idx, occ) & enemy_occ;
            if atks != 0 { return false; }
            queens &= queens - 1;
        }

        let mut rooks = self.rook[me];
        while rooks != 0 {
            let idx = rooks.trailing_zeros() as i8;
            let atks = rook_moves_board(idx, occ) & enemy_occ;
            if atks != 0 { return false; }
            rooks &= rooks - 1;
        }

        let mut bishops = self.bishop[me];
        while bishops != 0 {
            let idx = bishops.trailing_zeros() as i8;
            let atks = bishop_moves_board(idx, occ) & enemy_occ;
            if atks != 0 { return false; }
            bishops &= bishops - 1;
        }

        let mut knights = self.knight[me];
        while knights != 0 {
            let idx = knights.trailing_zeros() as i8;
            let atks = knight_moves_board(idx) & enemy_occ;
            if atks != 0 { return false; }
            knights &= knights - 1;
        }

        let pawns = self.pawn[me];
        let pawn_atks = if self.side_to_move == Color::White {
            ((pawns & !FILE_MASKS[0]) << 7) | ((pawns & !FILE_MASKS[7]) << 9)
        } else {
            ((pawns & !FILE_MASKS[0]) >> 9) | ((pawns & !FILE_MASKS[7]) >> 7)
        };
        if pawn_atks & enemy_occ != 0 { return false; }

        let pawn_promotions = if self.side_to_move == Color::White {
            ((pawns << 8) & !occ) & RANK_MASKS[7]
        } else {
            ((pawns >> 8) & !occ) & RANK_MASKS[0]
        };
        if pawn_promotions != 0 { return false; }

        let mut kings = self.king[me];
        while kings != 0 {
            let idx = kings.trailing_zeros() as i8;
            let atks = king_normal_moves_board(idx) & enemy_occ;
            if atks != 0 { return false; }
            kings &= kings - 1;
        }

        return true;
    }

    pub fn has_non_pawn_material(&self) -> bool {
        if (self.pawn[0] | self.king[0]) != self.composite[0] { return true; }
        if (self.pawn[1] | self.king[1]) != self.composite[1] { return true; }
        return false;
    }

    pub fn get_last_capture(&self) -> u8 {
        return self.cap_stack[self.cap_stack.len() - 1];
    }

    pub fn get_phase(&self) -> i32 {
        let mut phase = 256;
        // beginning should be 0, end (all pawns) should be 256
        // divide up like
        phase -= ((self.queen[0] | self.queen[1]).count_ones() as i32) * QUEEN_PHASE;
        phase -= ((self.rook[0] | self.rook[1]).count_ones() as i32) * ROOK_PHASE;
        phase -= ((self.bishop[0] | self.bishop[1]).count_ones() as i32) * BISHOP_PHASE;
        phase -= ((self.knight[0] | self.knight[1]).count_ones() as i32) * KNIGHT_PHASE;

        if phase < 0 {
            return 0;
        }
        return phase;
    }

    pub fn is_fifty_move(&self) -> bool {
        return self.halfmove >= 100;
    }

    pub fn insufficient_material(&self) -> bool {
        // just kings
        // KN v K
        // KB v K
        // KNN v K
        if self.king[0] != self.composite[0] && self.king[1] != self.composite[1] {
            // at least one non-king thing is on the board
            return false;
        }

        let mut side_to_confirm: usize = 0;
        if self.king[0] == self.composite[0] {
            // side 0 has only king
            side_to_confirm = 1;
        }

        if self.king[side_to_confirm] | self.knight[side_to_confirm] == self.composite[side_to_confirm] {
            // K, KN, KNN
            if self.knight[side_to_confirm].count_ones() <= 2 {
                return true;
            }
        }

        if self.king[side_to_confirm] | self.bishop[side_to_confirm] == self.composite[side_to_confirm] {
            // KB
            if self.bishop[side_to_confirm].count_ones() == 1 {
                return true;
            }
        }

        return false;
    }

    pub fn is_pseudolegal(&self, mv: &Move) -> bool {
        if mv.is_null() || mv.piece != self.piece_at_square(mv.start, self.side_to_move) {
            return false;
        }

        if mv.piece == b'k' && (mv.end - mv.start).abs() == 2 {
            // castling
            // start will be greater than end if queenside castling
            return self.can_castle(self.side_to_move, mv.start > mv.end);
        }

        let end_bb = idx_to_bb(mv.end);
        let self_occ = self.composite[self.side_to_move as usize];
        let enemy_occ = self.composite[!self.side_to_move as usize];
        let occ = self.composite[0] | self.composite[1];
        if end_bb & self_occ != 0 {
            // capturing own piece?
            return false;
        }

        if mv.is_pawn_cap() && (enemy_occ & end_bb == 0) {
            // en passant
            return mv.end as i32 % 8 == self.ep_file;
        }

        // all that's left should be more "normal" moves
        let moves_board = match mv.piece {
            b'k' => {
                king_normal_moves_board(mv.start as i8)
            },
            b'q' => {
                queen_moves_board(mv.start as i8, occ)
            },
            b'r' => {
                rook_moves_board(mv.start as i8, occ)
            },
            b'b' => {
                bishop_moves_board(mv.start as i8, occ)
            },
            b'n' => {
                knight_moves_board(mv.start as i8)
            },
            b'p' => {
                pawn_walk_board(occ, mv.start as i8, self.side_to_move)
                    | (pawn_attack_board(mv.start as i8, self.side_to_move) & enemy_occ)
            },
            _ => {return false;}
        };

        return moves_board & end_bb != 0;
    }
}
