use std::cmp;

use crate::bitboard::*;
use crate::movegen::*;
use crate::moveutil::*;
use crate::see::*;
use crate::search::*;
use crate::util::*;

pub const TT_MOVE: u8 = 0;
pub const GEN_NOISY: u8 = 1;
pub const OK_NOISY: u8 = 2;
pub const KILLER_MOVE_1: u8 = 3;
pub const KILLER_MOVE_2: u8 = 4;
pub const COUNTER_MOVE: u8 = 5;
pub const GEN_QUIET: u8 = 6;
pub const QUIET_MOVES: u8 = 7;
pub const BAD_NOISY: u8 = 8;

// TODO more stages to come later.  For now
// let's stay basic


const TT_MOVE_SCORE: u32 = 0xFFFFFFFF;
// offsets for scores
// winning and equal captures
// const QUEEN_CAPTURE_OFFSET: u32 = 1 << 60;
// const ROOK_CAPTURE_OFFSET: u32 = 1 << 59;
// const BISHOP_CAPTURE_OFFSET: u32 = 1 << 58;
// const KNIGHT_CAPTURE_OFFSET: u32 = 1 << 57;
// const PAWN_CAPTURE_OFFSET: u32 = 1 << 56;
pub const OK_CAPTURE_OFFSET: u32 = 1 << 25;

// quiet moves
// pub const KILLER_OFFSET: u32 = 1 << 50;
// pub const COUNTER_OFFSET: u32 = 1 << 49;
pub const QUIET_OFFSET: u32 = 1 << 24;

// losing captures
pub const BAD_CAPTURE_OFFSET: u32 = 0;

pub struct MovePicker {
    noisy_moves_only: bool,
    pub move_stage: u8,
    tt_move: Move,
    thread_num: usize,
    ply: i32,
    killers: [Move; 2],
    countermove: Move,
    scored_noisy_moves: Vec<(Move, u32)>,
    scored_quiet_moves: Vec<(Move, u32)>,
    noisy_i: usize,
    quiet_i: usize
}

impl MovePicker {
    pub fn new(tt_move: Move, ply: i32, thread_num: usize, q_moves_only: bool) -> MovePicker {
        let stage = if tt_move.is_null() {GEN_NOISY} else {TT_MOVE};

        MovePicker {
            noisy_moves_only: q_moves_only,
            move_stage: stage,
            tt_move: tt_move,
            ply: ply,
            thread_num: thread_num as usize,
            killers: [Move::null_move(); 2],
            countermove: Move::null_move(),
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
            ply: 0,
            thread_num: 0,
            killers: [Move::null_move(); 2],
            // history: [[0; 64]; 12],
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
            ply: 0,
            thread_num: 0,
            killers: [Move::null_move(); 2],
            // history: [[0; 64]; 12],
            countermove: Move::null_move(),
            scored_noisy_moves: Vec::new(),
            scored_quiet_moves: Vec::new(),
            noisy_i: 0,
            quiet_i: 0
        }
    }

    fn sort_next_move(mvs: &mut Vec<(Move, u32)>, cur_i: usize) {
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
        mvs.swap(highest_i, cur_i);
    }

    fn score_moves(&self, pos: &Bitboard, movelist: Vec<Move>) -> Vec<(Move, u32)> {
        let mut scored_moves: Vec<(Move, u32)> = Vec::new();
        let mut prev_mv = Move::null_move();
        let mut my_prev_mv = Move::null_move();
        let mut prev_piece_num = 0;
        let mut my_prev_piece_num = 0;
        let mut seen_quiet = false;

        for mv in movelist {
            let mv_score: u32;
            let captured = pos.piece_at_square(mv.end, !pos.side_to_move);
            if mv == self.tt_move {
                continue;
            }
            if captured == 0 && mv.promote_to == 0 {
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
                    // mv_score = QUIET_OFFSET + (self.history[piece_num][mv.end as usize] + self.followup[piece_num][mv.end as usize]) as u64;
                    let mut history_score = 0;
                    unsafe {
                        if !seen_quiet {
                            if self.ply > 0 {
                                prev_mv = SS[self.thread_num][(self.ply - 1) as usize].current_move;
                                if !prev_mv.is_null() {
                                    prev_piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                                }
                            }
                            if self.ply > 1 {
                                my_prev_mv = SS[self.thread_num][(self.ply - 2) as usize].current_move;
                                if !my_prev_mv.is_null() {
                                    my_prev_piece_num = get_piece_num(my_prev_mv.piece, pos.side_to_move);
                                }
                            }
                            seen_quiet = true;
                        }
                        history_score = TI[self.thread_num].move_history[piece_num][mv.end as usize];
                        if !prev_mv.is_null() {
                            history_score += TI[self.thread_num].countermove_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                        }
                        if !my_prev_mv.is_null() {
                            history_score += TI[self.thread_num].followup_history[my_prev_piece_num][my_prev_mv.end as usize][piece_num][mv.end as usize];
                        }
                    }
                    mv_score = (QUIET_OFFSET as i64 + history_score as i64) as u32;
                }
            } else if mv.promote_to == 0 {
                let score = see(pos, mv.end, captured, mv.start, mv.piece);
                if score >= 0 {
                    // let victim_val = match captured {
                    //     b'p' => 1,
                    //     b'n' => 3,
                    //     b'b' => 3,
                    //     b'r' => 5,
                    //     b'q' => 9,
                    //     _ => panic!("illegal capture!")
                    // };
                    // let atk_val = match mv.piece {
                    //     b'p' => 9,
                    //     b'n' => 7,
                    //     b'b' => 7,
                    //     b'r' => 5,
                    //     b'q' => 1,
                    //     b'k' => 0,
                    //     _ => 0
                    // };
                    // if self.noisy_moves_only {
                    //     mv_score = OK_CAPTURE_OFFSET + (victim_val << 4) + atk_val;//(score as u64);
                    // } else {
                    //     let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                    //     let cap_piece_num = get_piece_num(captured, pos.side_to_move) % 6;
                    //     mv_score = OK_CAPTURE_OFFSET + (victim_val << 4) + atk_val;//(score as u64);
                    //     // mv_score = (OK_CAPTURE_OFFSET as i64 + (victim_val as i32 * 1000 + self.capture_history[piece_num][mv.end as usize][cap_piece_num]) as i64) as u64;
                    // }
                    mv_score = OK_CAPTURE_OFFSET + score as u32;
                } else {
                    mv_score = QUIET_OFFSET - cmp::min(score.abs() as u32, QUIET_OFFSET);
                }
            } else {
                let score = match mv.promote_to {
                    b'n' => 3000,
                    b'b' => 3000,
                    b'r' => 5000,
                    b'q' => 9000,
                    _ => panic!("wat. Bad promotion")
                };
                mv_score = OK_CAPTURE_OFFSET + score;
            }
            scored_moves.push((mv, mv_score));
        }
        return scored_moves;
    }

    pub fn next(&mut self, pos: &Bitboard) -> (Move, u32) {
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
            unsafe {
                self.killers = TI[self.thread_num].killers[self.ply as usize];
            }
            self.move_stage = KILLER_MOVE_2;
            if self.killers[0] != self.tt_move && pos.is_pseudolegal(&self.killers[0]) {
                let mv = self.killers[0];
                let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                let mut history_score = 0;
                unsafe {
                    history_score = TI[self.thread_num].move_history[piece_num][mv.end as usize];
                    if self.ply > 0 {
                        let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                        let prev_mv = SS[self.thread_num][(self.ply - 1) as usize].current_move;
                        if !prev_mv.is_null() {
                            let prev_piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                            history_score += TI[self.thread_num].countermove_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                        }
                        if self.ply > 1 {
                            let prev_mv = SS[self.thread_num][(self.ply - 2) as usize].current_move;
                            if !prev_mv.is_null() {
                                let prev_piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                                history_score += TI[self.thread_num].followup_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                            }
                        }
                    }
                }
                let mv_score = (QUIET_OFFSET as i64 + history_score as i64) as u32;
                return (self.killers[0], mv_score);
            }
        }
        if self.move_stage == KILLER_MOVE_2 {
            self.move_stage = COUNTER_MOVE;
            if self.killers[1] != self.tt_move && pos.is_pseudolegal(&self.killers[1]) {
                let mv = self.killers[1];
                let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                let mut history_score = 0;
                unsafe {
                    history_score = TI[self.thread_num].move_history[piece_num][mv.end as usize];
                    if self.ply > 0 {
                        let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                        let prev_mv = SS[self.thread_num][(self.ply - 1) as usize].current_move;
                        if !prev_mv.is_null() {
                            let prev_piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                            history_score += TI[self.thread_num].countermove_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                        }
                        if self.ply > 1 {
                            let prev_mv = SS[self.thread_num][(self.ply - 2) as usize].current_move;
                            if !prev_mv.is_null() {
                                let prev_piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                                history_score += TI[self.thread_num].followup_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                            }
                        }
                    }
                }
                let mv_score = (QUIET_OFFSET as i64 + history_score as i64) as u32;
                return (self.killers[1], mv_score);
            }
        }
        if self.move_stage == COUNTER_MOVE {
            unsafe {
                if self.ply != 0 {
                    let prev_mv = SS[self.thread_num][(self.ply - 1) as usize].current_move;
                    if !prev_mv.is_null() {
                        let piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                        self.countermove = TI[self.thread_num].countermove_table[piece_num][prev_mv.end as usize];
                    }
                }
            }
            self.move_stage = GEN_QUIET;
            if self.countermove != self.killers[0] &&
                self.countermove != self.killers[1] &&
                self.countermove != self.tt_move &&
                pos.is_pseudolegal(&self.countermove) {
                    let mv = self.countermove;
                    let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                    let mut history_score = 0;
                    unsafe {
                        history_score = TI[self.thread_num].move_history[piece_num][mv.end as usize];
                        if self.ply > 0 {
                            let piece_num = get_piece_num(mv.piece, pos.side_to_move);
                            let prev_mv = SS[self.thread_num][(self.ply - 1) as usize].current_move;
                            if !prev_mv.is_null() {
                                let prev_piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                                history_score += TI[self.thread_num].countermove_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                            }
                            if self.ply > 1 {
                                let prev_mv = SS[self.thread_num][(self.ply - 2) as usize].current_move;
                                if !prev_mv.is_null() {
                                    let prev_piece_num = get_piece_num(prev_mv.piece, !pos.side_to_move);
                                    history_score += TI[self.thread_num].followup_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                                }
                            }
                        }
                    }
                    let mv_score = (QUIET_OFFSET as i64 + history_score as i64) as u32;
                    return (self.countermove, mv_score);
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
