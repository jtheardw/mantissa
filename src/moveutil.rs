use std::fmt;

use crate::bitboard::*;
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
            return format!("0000");
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

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_repr())
    }
}


pub fn is_quiet_move(mv: &Move, pos: &Bitboard) -> bool {
    if mv.is_ep { return false; }
    if (idx_to_bb(mv.end) & pos.composite[!pos.side_to_move as usize]) != 0 { return false; }
    return true;
}

pub fn is_tactical_move(mv: &Move, pos: &Bitboard) -> bool {
    // current a move is "tactical" if it is:
    // - a capture
    // - a promotion
    // - a pawn walking to the 7th rank
    if mv.promote_to != 0 { return true; }
    if mv.is_ep { return true; }
    if mv.piece == b'p' {
        if pos.side_to_move == Color::White {
            if mv.end >= 48 {
                return true;
            }
        } else {
            if mv.end < 16 {
                return true;
            }
        }
    }
    if (idx_to_bb(mv.end) & pos.composite[!pos.side_to_move as usize]) != 0 { return true; }
    return false;
}
