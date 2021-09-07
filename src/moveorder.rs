use crate::bitboard::*;
use crate::movegen::*;
use crate::moveutil::*;
use crate::util::*;

const TT_MOVE: u8 = 0;
const REMAINING_MOVES: u8 = 1;
// TODO more stages to come later.  For now
// let's stay basic


const TT_MOVE_SCORE: u64 = 0xFFFFFFFFFFFFFFFF;
// offsets for scores
// winning and equal captures
// const QUEEN_CAPTURE_OFFSET: u64 = 1 << 60;
// const ROOK_CAPTURE_OFFSET: u64 = 1 << 59;
// const BISHOP_CAPTURE_OFFSET: u64 = 1 << 58;
// const KNIGHT_CAPTURE_OFFSET: u64 = 1 << 57;
// const PAWN_CAPTURE_OFFSET: u64 = 1 << 56;
const OK_CAPTURE_OFFSET: u64 = 1 << 60;

// quiet moves
pub const KILLER_OFFSET: u64 = 1 << 50;
pub const QUIET_OFFSET: u64 = 1 << 12;

// losing captures
const BAD_CAPTURE_OFFSET: u64 = 0;

pub struct MovePicker {
    q_moves_only: bool,
    move_stage: u8,
    tt_move: Move,
    killers: [Move; 2],
    history: [[u64; 64]; 12],
    scored_moves: Vec<(Move, u64)>,
    cur_i: usize,
}

impl MovePicker {

    pub fn new(tt_move: Move, killers: [Move; 2], history: [[u64; 64]; 12], q_moves_only: bool) -> MovePicker {
        let stage = if tt_move.is_null {REMAINING_MOVES} else {TT_MOVE};
        MovePicker {
            q_moves_only: q_moves_only,
            move_stage: stage,
            tt_move: tt_move,
            killers: killers,
            history: history,
            scored_moves: Vec::new(),
            cur_i: 0,
        }
    }

    pub fn q_new() -> MovePicker {
        MovePicker {
            q_moves_only: true,
            move_stage: REMAINING_MOVES,
            tt_move: Move::null_move(),
            killers: [Move::null_move(); 2],
            history: [[0; 64]; 12],
            scored_moves: Vec::new(),
            cur_i: 0
        }
    }

    fn score_moves(&self, pos: &Bitboard, movelist: Vec<Move>) -> Vec<(Move, u64)> {
        let mut scored_moves: Vec<(Move, u64)> = Vec::new();
        let defended_pieces = all_attacks_board(pos, !pos.side_to_move) & pos.composite[!pos.side_to_move as usize];

        for mv in movelist {
            let mv_score: u64;
            let captured = pos.piece_at_square(mv.end, !pos.side_to_move);
            if mv == self.tt_move {
                // we would have already seen this move
                continue;
            }
            if captured == 0 {
                // not a capture
                if mv == self.killers[0] || mv == self.killers[1] {
                    mv_score = KILLER_OFFSET;
                } else {
                    let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                    mv_score = QUIET_OFFSET + self.history[piece_num][mv.end as usize];
                }
            } else {
                let my_val = match mv.piece {
                    b'p' => 1,
                    b'n' => 3,
                    b'b' => 3,
                    b'r' => 5,
                    b'q' => 9,
                    b'k' => 200,
                    _ => 0
                };

                let their_val = match captured {
                    b'p' => 1,
                    b'n' => 3,
                    b'b' => 3,
                    b'r' => 5,
                    b'q' => 9,
                    b'k' => panic!("Captured king?"),
                    _ => 0
                };

                if idx_to_bb(mv.end) & defended_pieces == 0 {
                    // free capture
                    mv_score = their_val;
                } else if my_val > their_val {
                    mv_score = BAD_CAPTURE_OFFSET + (1000 - (my_val - their_val));
                } else {
                    let offset = their_val - my_val;
                    mv_score = OK_CAPTURE_OFFSET + offset;
                }
            }

            scored_moves.push((mv, mv_score));
        }
        return scored_moves;
    }

    pub fn next(&mut self, pos: &Bitboard) -> (Move, u64) {
        if self.move_stage == TT_MOVE {
            self.move_stage = REMAINING_MOVES;
            return (self.tt_move, TT_MOVE_SCORE);
        } else {
            if self.cur_i == 0 {
                let movelist: Vec<Move>;
                // generate the moves
                if self.q_moves_only {
                    movelist = qmoves(pos);
                } else {
                    movelist = moves(pos);
                }
                self.scored_moves = self.score_moves(pos, movelist);
            }
            if self.cur_i == self.scored_moves.len() {
                return (Move::null_move(), 0);
            }

            let mut highest_i = self.cur_i;

            for i in (self.cur_i + 1)..self.scored_moves.len() {
                if self.scored_moves[i].1 > self.scored_moves[highest_i].1 {
                    highest_i = i;
                }
            }

            let (mv, score) = self.scored_moves[highest_i];
            self.scored_moves[highest_i] = self.scored_moves[self.cur_i];
            self.scored_moves[self.cur_i] = (mv, score);
            self.cur_i += 1;

            return (mv, score)
        }
    }
}
