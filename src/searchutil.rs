use crate::moveutil::*;
use crate::time::*;
use crate::tt::*;
use crate::util::*;

pub const EFP_DEPTH: i32 = 3;         // extended futility pruning
pub const RFP_DEPTH: i32 = 3;         // reverse futility pruning
pub const NMP_DEPTH: i32 = 3;         // null-move pruning/reductions
pub const LMR_DEPTH: i32 = 3;         // late move reductions

pub fn efp_margin(depth: i32) -> i32 {
    if depth <= 0 { return 0; }
    if depth > 3 { return 4000 * depth}
    return [0, 2200, 3200, 5300][depth as usize];
    // return base + 1000 * (depth - 1);
}

pub fn rfp_margin(depth: i32) -> i32 {
    return 1300 * depth;
}

pub fn null_move_r(static_eval: i32, beta: i32, depth: i32) -> i32 {
    let mut r = if depth > 6 {3} else {2};
    r += ((static_eval - beta) / 1000) as i32;
    return r;
}

pub fn lmr_reduction(depth: i32, moves_searched: i32) -> i32 {
    return 1 + ((moves_searched - 4) / 4) + (depth / 8);
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
    pub move_history: [[u64; 64]; 12]
}

impl ThreadInfo {
    pub fn new() -> ThreadInfo {
        let killers = [[Move::null_move(); 2]; MAX_PLY];
        let move_history = [[0; 64]; 12];
        ThreadInfo {
            nodes_searched: 0,
            seldepth: 0,
            killers: killers,
            move_history: move_history
        }
    }

    pub fn clear(&mut self) {
        self.killers = [[Move::null_move(); 2]; MAX_PLY];
        self.seldepth = 0;
        self.nodes_searched = 0;
        self.move_history = [[0; 64]; 12];
    }

    pub fn update_move_history(&mut self, mv: Move, side: Color, depth: i32) {
        let piece_num = get_piece_num(mv.piece, side);
        self.move_history[piece_num][mv.end as usize] += 1 << depth;
    }

    pub fn update_killers(&mut self, mv: Move, ply: i32) {
        let ply = ply as usize;
        if self.killers[ply][0] != mv {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = mv;
        }
    }
}

pub struct SearchStatsEntry {
    pub pv: Vec<Move>,
    pub static_eval: i32,
    pub ply: i32,
    pub searching_null_move: bool,
    pub excluded_move: Move,
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
