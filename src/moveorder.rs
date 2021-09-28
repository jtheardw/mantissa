use std::cmp;

use crate::bitboard::*;
use crate::movegen::*;
use crate::moveutil::*;
use crate::see::*;
use crate::util::*;

const TT_MOVE: u8 = 0;
const GEN_NOISY: u8 = 1;
const OK_NOISY: u8 = 2;
const KILLER_MOVE_1: u8 = 3;
const KILLER_MOVE_2: u8 = 4;
const COUNTER_MOVE: u8 = 5;
const GEN_QUIET: u8 = 6;
const QUIET_MOVES: u8 = 7;
const BAD_NOISY: u8 = 8;

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
pub const OK_CAPTURE_OFFSET: u64 = 1 << 60;

// quiet moves
pub const KILLER_OFFSET: u64 = 1 << 50;
pub const COUNTER_OFFSET: u64 = 1 << 49;
pub const QUIET_OFFSET: u64 = 1 << 20;

// losing captures
pub const BAD_CAPTURE_OFFSET: u64 = 0;

pub struct MovePicker {
    noisy_moves_only: bool,
    pub move_stage: u8,
    tt_move: Move,
    killers: [Move; 2],
    history: [[i32; 64]; 12],
    countermove: Move,
    followup: [[i32; 64]; 12],
    scored_noisy_moves: Vec<(Move, u64)>,
    scored_quiet_moves: Vec<(Move, u64)>,
    noisy_i: usize,
    quiet_i: usize
}

impl MovePicker {

    pub fn new(tt_move: Move, killers: [Move; 2], history: [[i32; 64]; 12], countermove: Move, followup_history: [[i32; 64]; 12], q_moves_only: bool) -> MovePicker {
        let stage = if tt_move.is_null {GEN_NOISY} else {TT_MOVE};
        MovePicker {
            noisy_moves_only: q_moves_only,
            move_stage: stage,
            tt_move: tt_move,
            killers: killers,
            history: history,
            followup: followup_history,
            countermove: countermove,
            scored_noisy_moves: Vec::new(),
            scored_quiet_moves: Vec::new(),
            noisy_i: 0,
            quiet_i: 0
        }
    }

    pub fn q_new() -> MovePicker {
        MovePicker {
            noisy_moves_only: true,
            move_stage: GEN_NOISY,
            tt_move: Move::null_move(),
            killers: [Move::null_move(); 2],
            history: [[0; 64]; 12],
            followup: [[0; 64]; 12],
            countermove: Move::null_move(),
            scored_noisy_moves: Vec::new(),
            scored_quiet_moves: Vec::new(),
            noisy_i: 0,
            quiet_i: 0
        }
    }

    pub fn perft_new() -> MovePicker {
        MovePicker {
            noisy_moves_only: false,
            move_stage: GEN_NOISY,
            tt_move: Move::null_move(),
            killers: [Move::null_move(); 2],
            history: [[0; 64]; 12],
            followup: [[0; 64]; 12],
            countermove: Move::null_move(),
            scored_noisy_moves: Vec::new(),
            scored_quiet_moves: Vec::new(),
            noisy_i: 0,
            quiet_i: 0
        }
    }

    fn sort_next_move(mvs: &mut Vec<(Move, u64)>, cur_i: usize) {
        let mut highest_i = cur_i;
        let mut highest = mvs[highest_i].1;
        let mut i = cur_i + 1;
        let len = mvs.len();

        while i < len {
            let score = mvs[i].1;
            if score > highest {
                highest_i = i;
                highest = score;
            }
            i += 1;
        }

        // swap
        let (mv, score) = mvs[highest_i];
        mvs[highest_i] = mvs[cur_i];
        mvs[cur_i] = (mv, score);
    }

    fn score_moves(&self, pos: &Bitboard, movelist: Vec<Move>) -> Vec<(Move, u64)> {
        let mut scored_moves: Vec<(Move, u64)> = Vec::new();
        for mv in movelist {
            let mv_score: u64;
            let captured = pos.piece_at_square(mv.end, !pos.side_to_move);
            if mv == self.tt_move {
                continue;
            }
            if captured == 0 {
                // not a capture
                if mv == self.killers[0] || mv == self.killers[1] {
                    // mv_score = KILLER_OFFSET;
                    // this move will be handled in a different stage
                    continue;
                } else if mv == self.countermove {
                    // mv_score = COUNTER_OFFSET;
                    // same here
                    continue;
                } else {
                    let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                    mv_score = QUIET_OFFSET + (self.history[piece_num][mv.end as usize] + self.followup[piece_num][mv.end as usize]) as u64;
                }
            } else {
                let score = see(pos, mv.end, captured, mv.start, mv.piece);
                if score >= 0 {
                    mv_score = OK_CAPTURE_OFFSET + (score as u64);
                } else {
                    mv_score = QUIET_OFFSET - cmp::min(score.abs() as u64, QUIET_OFFSET);
                }
            }
            scored_moves.push((mv, mv_score));
        }
        return scored_moves;
    }

    pub fn next(&mut self, pos: &Bitboard) -> (Move, u64) {
        if self.move_stage == TT_MOVE {
            self.move_stage = GEN_NOISY;
            if pos.is_pseudolegal(&self.tt_move) {
                return (self.tt_move, TT_MOVE_SCORE);
            }
        }
        if self.move_stage == GEN_NOISY {
            self.move_stage = OK_NOISY;
            self.scored_noisy_moves = self.score_moves(pos, noisy_moves(pos));
        }
        if self.move_stage == OK_NOISY {
            if self.noisy_i == self.scored_noisy_moves.len() {
                if self.noisy_moves_only {
                    self.move_stage = BAD_NOISY;
                } else {
                    self.move_stage = KILLER_MOVE_1;
                }
            } else {
                // still noisy moves
                MovePicker::sort_next_move(&mut self.scored_noisy_moves, self.noisy_i);
                let (mv, score) = self.scored_noisy_moves[self.noisy_i];
                if score < QUIET_OFFSET {
                    // consider the rest "bad"
                    if self.noisy_moves_only {
                        self.move_stage = BAD_NOISY;
                    } else {
                        self.move_stage = KILLER_MOVE_1;
                    }
                } else {
                    self.noisy_i += 1;
                    return (mv, score);
                }
            }
        }
        if self.move_stage == KILLER_MOVE_1 {
            self.move_stage = KILLER_MOVE_2;
            if pos.is_pseudolegal(&self.killers[0]) {
                return (self.killers[0], KILLER_OFFSET);
            }
        }
        if self.move_stage == KILLER_MOVE_2 {
            self.move_stage = COUNTER_MOVE;
            if pos.is_pseudolegal(&self.killers[1]) {
                return (self.killers[1], KILLER_OFFSET);
            }
        }
        if self.move_stage == COUNTER_MOVE {
            self.move_stage = GEN_QUIET;
            if pos.is_pseudolegal(&self.countermove) {
                return (self.countermove, COUNTER_OFFSET);
            }
        }
        if self.move_stage == GEN_QUIET {
            self.move_stage = QUIET_MOVES;
            self.scored_quiet_moves = self.score_moves(pos, quiet_moves(pos));
        }
        if self.move_stage == QUIET_MOVES {
            if self.quiet_i == self.scored_quiet_moves.len() {
                self.move_stage = BAD_NOISY;
            } else {
                // still quiet moves
                MovePicker::sort_next_move(&mut self.scored_quiet_moves, self.quiet_i);
                let (mv, score) = self.scored_quiet_moves[self.quiet_i];
                self.quiet_i += 1;
                return (mv, score);
            }
        }
        if self.move_stage == BAD_NOISY {
            if self.noisy_i == self.scored_noisy_moves.len() {
                return (Move::null_move(), 0);
            }
            // here we reverse the cadence because the next move was already sorted in the good noisy
            // moves.
            let (mv, score) = self.scored_noisy_moves[self.noisy_i];
            self.noisy_i += 1;
            if self.noisy_i < self.scored_noisy_moves.len() {
                MovePicker::sort_next_move(&mut self.scored_noisy_moves, self.noisy_i);
            }
            return (mv, score);
        }
        return (Move::null_move(), 0);
    }
}
