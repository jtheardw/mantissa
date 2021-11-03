use std::fmt;

use crate::bitboard::*;
use crate::util::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Move {
    pub start: i8,
    pub end: i8,
    pub piece: u8,
    pub promote_to: u8,
}

impl Move {
    pub fn is_null(&self) -> bool {
        return self.start == self.end;
    }

    pub fn is_pawn_cap(&self) -> bool {
        return self.piece == b'p' && (self.end % 8 != self.start % 8);
    }

    pub fn get_repr(&self) -> String {
        // UCI compatible representation of move
        if self.is_null() {
            return format!("0000");
        }
        let start = idx_to_str(self.start as i8);
        let end = idx_to_str(self.end as i8);
        let mut promote = "".to_string();

        if self.promote_to != 0 {
            promote = (self.promote_to as char).to_string();
        }

        return format!("{}{}{}", start, end, promote)
    }

    pub fn piece_move(start: i8, end: i8, piece: u8) -> Move {
        // standard type of move for all pieces except pawns
        Move {
            start: start as i8,
            end: end as i8,
            piece: piece,
            promote_to: 0,
        }
    }

    pub fn ep_file(&self) -> i32 {
        let mut ep_file = -1;
        if self.piece == b'p' && (self.end - self.start).abs() == 16 {
            ep_file = (self.start as i32) % 8;
        }
        return ep_file;
    }

    pub fn pawn_move(start: i8, end: i8) -> Move {
        // pawn walk, double walk, or typical capture
        // let mut ep_file = -1;
        // if (end - start).abs() == 16 {
        //     ep_file = start % 8;
        // }
        Move {
            start: start as i8,
            end: end as i8,
            piece: b'p',
            promote_to: 0,
        }
    }

    // Next are the pawn move constructors
    pub fn ep_capture(start: i8, end: i8) -> Move {
        // capture en-passant
        Move {
            start: start as i8,
            end: end as i8,
            piece: b'p',
            promote_to: 0,
        }
    }

    pub fn promotion(start: i8, end: i8, piece: u8) -> Move {
        Move {
            start: start as i8,
            end: end as i8,
            piece: b'p',
            promote_to: piece,
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
            promote_to: 0,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_repr())
    }
}

pub fn is_quiet_move(mv: &Move, pos: &Bitboard) -> bool {
    if mv.is_pawn_cap() { return false; }
    if (idx_to_bb(mv.end) & pos.composite[!pos.side_to_move as usize]) != 0 { return false; }
    return true;
}

pub fn is_tactical_move(mv: &Move, pos: &Bitboard) -> bool {
    // current a move is "tactical" if it is:
    // - a capture
    // - a promotion
    // - a pawn walking to the 7th rank
    if mv.promote_to != 0 { return true; }
    if mv.is_pawn_cap() { return true; }
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
