use std::cmp;

use crate::moveutil::*;
use crate::pht::*;
use crate::time::*;
use crate::tt::*;
use crate::util::*;

pub const EFP_DEPTH: i32 = 8;         // extended futility pruning
pub const FP_DEPTH: i32 = 5;
pub const RFP_DEPTH: i32 = 8;         // reverse futility pruning
pub const AFP_DEPTH: i32 = 5;         // reverse futility pruning
pub const NMP_DEPTH: i32 = 3;         // null-move pruning/reductions
pub const LMR_DEPTH: i32 = 2;         // late move reductions

static mut LMR_TABLE: [[i32; 64]; 64] = [[0; 64]; 64];

pub fn efp_margin(depth: i32) -> i32 {
    if depth <= 0 { return 0; }
    // if depth > 3 { return 1000 + 2000 * depth}
    // return [0, 3200, 5000, 7300][depth as usize];
    return 1000 + 1200 * depth;
    // return base + 1000 * (depth - 1);
}

pub fn fp_margin(depth: i32) -> i32 {
    return 1000 + depth * 600;
}

pub fn rfp_margin(depth: i32) -> i32 {
    return 1200 * depth;
}

pub fn afp_margin(_depth: i32) -> i32 {
    return 30000;
}

pub fn null_move_r(static_eval: i32, beta: i32, depth: i32) -> i32 {
    let mut r = 4 + (depth / 6);
    r += cmp::min(3, (static_eval - beta) / 3000) as i32;
    return r;
}

pub fn lmr_table_gen() {
    for d in 1..64 {
        for m in 1..64 {
            let r = (0.8 + ((d as f64).ln() * (m as f64).ln()) / 2.25).floor() as i32;
            unsafe {LMR_TABLE[d as usize][m as usize] = r;}
        }
    }
}

pub fn lmr_reduction(depth: i32, moves_searched: i32) -> i32 {
    let d = cmp::min(depth, 63) as usize;
    let m = cmp::min(moves_searched, 63) as usize;
    unsafe {
        return LMR_TABLE[d][m];
    }
}

pub fn lmp_count(improving: bool, depth: i32) -> i32 {
    if improving {
        4 + depth * depth
    } else {
        2 + depth * depth / 2
    }
}

#[derive(Copy, Clone)]
pub struct SearchLimits {
    pub infinite: bool,
    pub use_variable_time: bool,
    pub movetime: u128,
    pub optimum_time: u128,
    pub maximum_time: u128,
    pub depth: i32,
}

impl SearchLimits {
    // constructors
    pub fn movetime(movetime: u128) -> SearchLimits {
        SearchLimits {
            infinite: false,
            use_variable_time: false,
            movetime: movetime,
            optimum_time: 0,
            maximum_time: 0,
            depth: MAX_DEPTH as i32
        }
    }

    pub fn clock_with_inc(clock_time: i32, clock_inc: i32, overhead: i32, ply: i32, material: i32) -> SearchLimits {
        let time_info = get_time_bounds_clock_inc(clock_time, clock_inc, overhead, ply, material);
        SearchLimits {
            infinite: false,
            use_variable_time: true,
            movetime: 0,
            optimum_time: time_info.0,
            maximum_time: time_info.1,
            depth: MAX_DEPTH as i32
        }
    }

    pub fn moves_to_go(clock_time: i32, moves_to_go: i32, overhead: i32) -> SearchLimits {
        let time_info = get_time_bounds_moves_to_go(clock_time, moves_to_go, overhead);
        SearchLimits {
            infinite: false,
            use_variable_time: true,
            movetime: 0,
            optimum_time: time_info.0,
            maximum_time: time_info.1,
            depth: MAX_DEPTH as i32
        }
    }

    pub fn depth(depth: i32) -> SearchLimits {
        SearchLimits {
            infinite: false,
            use_variable_time: false,
            movetime: 0,
            optimum_time: 0,
            maximum_time: 0,
            depth: depth
        }
    }
    pub const fn infinite() -> SearchLimits {
        SearchLimits {
            infinite: true,
            use_variable_time: false,
            movetime: 0,
            optimum_time: 0,
            maximum_time: 0,
            depth: MAX_DEPTH as i32
        }
    }
}

pub struct ThreadInfo {
    pub nodes_searched: u64,
    pub seldepth: i32,
    pub killers: [[Move; 2]; MAX_PLY],
    pub move_history: [[i32; 64]; 12],
    pub capture_history: [[[i32; 6]; 64]; 12],
    pub countermove_table: [[Move; 64]; 12],
    pub countermove_history: Vec<[[[i32; 64]; 12]; 64]>,
    pub followup_history: Vec<[[[i32; 64]; 12]; 64]>,
    pub pht: PHT,
    pub root_moves: Vec<Move>,
    pub bh_piece: i8
}

impl ThreadInfo {
    pub fn new() -> ThreadInfo {
        let killers = [[Move::null_move(); 2]; MAX_PLY];
        let move_history = [[0; 64]; 12];
        let capture_history = [[[0; 6]; 64]; 12];
        let countermove_table = [[Move::null_move(); 64]; 12];
        let followup_history = vec![[[[0; 64]; 12]; 64]; 12];
        let countermove_history = vec![[[[0; 64]; 12]; 64]; 12];
        let pht = PHT::get_pht(14);
        ThreadInfo {
            nodes_searched: 0,
            seldepth: 0,
            killers: killers,
            move_history: move_history,
            capture_history: capture_history,
            countermove_table: countermove_table,
            countermove_history: countermove_history,
            followup_history: followup_history,
            pht: pht,
            root_moves: Vec::new(),
            bh_piece: -1
        }
    }

    pub fn clear(&mut self) {
        self.killers = [[Move::null_move(); 2]; MAX_PLY];
        self.seldepth = 0;
        self.nodes_searched = 0;
self.move_history = [[0; 64]; 12];
        self.move_history = [[0; 64]; 12];
        self.capture_history = [[[0; 6]; 64]; 12];
        self.countermove_table = [[Move::null_move(); 64]; 12];
        self.followup_history = vec![[[[0; 64]; 12]; 64]; 12];
        self.countermove_history = vec![[[[0; 64]; 12]; 64]; 12];
        self.root_moves = Vec::new();
        self.bh_piece = -1;
    }

    pub fn update_move_history(&mut self, mv: Move, side: Color, depth: i32, searched_moves: &Vec<Move>) {
        if depth < 3 { return; }
        for s_mv in searched_moves {
            if *s_mv == mv {continue;}
            let piece_num = get_piece_num(s_mv.piece, side);
            let cur = self.move_history[piece_num][s_mv.end as usize];
            self.move_history[piece_num][s_mv.end as usize] = self.decay_update(cur, -depth * depth);
        }
        let piece_num = get_piece_num(mv.piece, side);
        let cur = self.move_history[piece_num][mv.end as usize];
        self.move_history[piece_num][mv.end as usize] = self.decay_update(cur, depth * depth);
    }

    fn decay_update(&self, cur: i32, delta: i32) -> i32 {
        // I've seen this formula on the talkchess forums, but I would guess
        // the first place it comes from is Ethereal.  Some other top engines use similar
        // formulae or even the same one.
        // Though others like Cheng have a different pattern for decay
        // In any case, I've tried the naive implementation, this formula, cheng-style
        // and a bayes rule inspired update, but this style seemed to be most effective in Mantissa

        // see here: http://www.talkchess.com/forum3/viewtopic.php?f=7&t=76540
        // for more explanation
        return cur - (cur * delta.abs()) / 512 + delta * 32;
    }

    pub fn update_killers(&mut self, mv: Move, ply: i32) {
        let ply = ply as usize;
        if self.killers[ply][0] != mv {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = mv;
        }
    }

    pub fn update_countermove(&mut self, prev_mv: Move, mv: Move, side: Color) {
        if prev_mv.is_null() || mv.is_null() { return; }
        let piece_num = get_piece_num(prev_mv.piece, side);
        self.countermove_table[piece_num][prev_mv.end as usize] = mv;
    }

    pub fn update_countermove_history(&mut self, prev_mv: Move, mv: Move, side: Color, depth: i32, searched_moves: &Vec<Move>) {
        if depth < 3 { return; }
        if prev_mv.is_null() || mv.is_null() { return; }
        let prev_piece_num = get_piece_num(prev_mv.piece, !side);
        let prev_end = prev_mv.end as usize;
        for s_mv in searched_moves {
            if *s_mv == mv {
                continue;
            }
            let piece_num = get_piece_num(s_mv.piece, side);
            let end = s_mv.end as usize;

            let cur = self.countermove_history[prev_piece_num][prev_end][piece_num][end];
            self.countermove_history[prev_piece_num][prev_end][piece_num][end] = self.decay_update(cur, -depth * depth);
        }

        let piece_num = get_piece_num(mv.piece, side);
        let end = mv.end as usize;

        let cur = self.countermove_history[prev_piece_num][prev_end][piece_num][end];
        self.countermove_history[prev_piece_num][prev_end][piece_num][end] = self.decay_update(cur, depth * depth);
    }

    pub fn update_followup(&mut self, prev_mv: Move, mv: Move, side: Color, depth: i32, searched_moves: &Vec<Move>) {
        if depth < 3 { return; }
        if prev_mv.is_null() || mv.is_null() { return; }
        let prev_piece_num = get_piece_num(prev_mv.piece, side);
        let prev_end = prev_mv.end as usize;
        for s_mv in searched_moves {
            if *s_mv == mv {
                continue;
            }
            let piece_num = get_piece_num(s_mv.piece, side);
            let end = s_mv.end as usize;

            let cur = self.followup_history[prev_piece_num][prev_end][piece_num][end];
            self.followup_history[prev_piece_num][prev_end][piece_num][end] = self.decay_update(cur, -depth * depth);
        }

        let piece_num = get_piece_num(mv.piece, side);
        let end = mv.end as usize;

        let cur = self.followup_history[prev_piece_num][prev_end][piece_num][end];
        self.followup_history[prev_piece_num][prev_end][piece_num][end] = self.decay_update(cur, depth * depth);
    }
}

pub struct SearchStatsEntry {
    pub pv: Vec<Move>,
    pub static_eval: i32,
    pub ply: i32,
    pub searching_null_move: bool,
    pub excluded_move: Move,
    pub current_move: Move,
    pub tt_hit: bool,
    pub tt_move: Move,
    pub tt_val: i32,
    pub tt_depth: i32,
    pub tt_node_type: u8
}

impl SearchStatsEntry {
    pub fn new() -> SearchStatsEntry {
        SearchStatsEntry {
            pv: Vec::new(),
            static_eval: 0,
            ply: 0,
            searching_null_move: false,
            excluded_move: Move::null_move(),
            current_move: Move::null_move(),
            tt_hit: false,
            tt_move: Move::null_move(),
            tt_val: 0,
            tt_depth: 0,
            tt_node_type: PV_NODE
        }
    }

    pub fn clear(&mut self) {
        self.pv = Vec::new();
        self.static_eval = 0;
        self.ply = 0;
        self.searching_null_move = false;
        self.excluded_move = Move::null_move();
        self.current_move = Move::null_move();
        self.tt_hit = false;
    }
}

pub type SearchStats = Vec<SearchStatsEntry>;

pub fn new_searchstats() -> SearchStats {
    let mut ss: SearchStats = Vec::new();
    for _ in 0..MAX_PLY {
        ss.push(SearchStatsEntry::new());
    }
    return ss;
}
