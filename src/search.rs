use std::cmp;
use std::time;
use std::time::SystemTime;
use std::thread;

use crate::bitboard::*;
use crate::eval::*;
use crate::moveorder::*;
use crate::moveutil::*;
use crate::searchparams::*;
use crate::searchutil::*;
use crate::see::*;
use crate::tt::*;
use crate::util::*;

static mut SEARCH_IN_PROGRESS: bool = false;
static mut ABORT: bool = false;
static mut STOP_THREAD: bool = false;
static mut START_TIME: u128 = 0;
static mut SEARCH_LIMITS: SearchLimits = SearchLimits::infinite();
static mut LAST_BESTMOVE: Move = Move::null_move();

const LB: i32 = -10000000;
const UB: i32 = 10000000;

pub static mut TI: Vec<ThreadInfo> = Vec::new();
pub static mut SS: Vec<SearchStats> = Vec::new();

pub static mut TT_VALID: u64 = 0;
pub static mut STATIC_EVALS: u64 = 0;
pub static mut MAIN_SEARCH_NODES: u64 = 0;

pub fn get_time_millis() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime failed!"),
    }
}

pub fn emit_last_bestmove() {
    unsafe {
        // println!("{}", LAST_INFO);
        println!("bestmove {}", LAST_BESTMOVE)
    }
}

pub fn ongoing_search() -> bool {
    unsafe {
        SEARCH_IN_PROGRESS
    }
}

pub fn clear_info() {
    unsafe {
        TI = Vec::new();
        SS = Vec::new();
    }
}

fn get_pv_str(pv: &Vec<Move>) -> String {
    let mut s = format!("");
    for mv in pv {
        if s.len() > 0 {
            s = format!("{} {}", s, mv);
        } else {
            s = format!("{}", mv);
        }
    }
    return s;
}

fn get_val_str(val: i32) -> String {
    if val.abs() < MATE_SCORE - 100000 { return format!("cp {}", val / 10); }
    let mut mate_score = (((MATE_SCORE - val.abs()) as f64) / 2.).ceil() as i32;
    if val < 0 {
        mate_score = -1 * mate_score;
    }
    return format!("mate {}", mate_score);
}

fn print_info(depth: i32, seldepth: i32, pv: &Vec<Move>, val: i32, time: u128, nodes: u64) {
    let pv_str = get_pv_str(pv);
    let val_str = get_val_str(val);
    let nps = (nodes * 1000) as u128 / time;
    // unsafe {
    //     LAST_INFO = fmt!("info depth {} seldepth {} score {} time {} nodes {} nps {} pv {} multipv 1",
    //          depth, seldepth, val_str, time, nodes, nps, pv_str).to_str();
    // }
    println!("info depth {} seldepth {} score {} time {} nodes {} nps {} multipv 1 pv {}",
             depth, seldepth, val_str, time, nodes, nps, pv_str
    );
    // eprintln!("info depth {} seldepth {} score {} time {} nodes {} nps {} multipv 1 pv {}",
    //          depth, seldepth, val_str, time, nodes, nps, pv_str
    // );
}

pub fn abort_search() {
    // kill a search altogether
    unsafe {
        ABORT = true;
    }
}

pub fn stop_threads() {
    // stop the currently running threads
    // but don't necessarily abandon
    // the whole search.  Used
    // to kill child threads in the ID loop
    unsafe {
        STOP_THREAD = true;
    }
}

pub fn allow_threads() {
    // re-enable threads to be spawned
    unsafe {
        STOP_THREAD = false;
    }
}

pub fn search_aborted() -> bool {
    // check if search is aborted
    unsafe {
        ABORT
    }
}

pub fn thread_killed() -> bool {
    // check if a thread should die
    unsafe {
        ABORT || STOP_THREAD
    }
}

fn check_time(search_limits: &SearchLimits) {
    if search_limits.infinite { return; }
    if search_limits.movetime == 0 && !search_limits.use_variable_time {
        return;
    }

    unsafe {
        let elapsed_time = get_time_millis() - START_TIME;
        if search_limits.movetime > 0 && elapsed_time > search_limits.movetime {
            abort_search();
            return;
        } else if search_limits.maximum_time > 0 && elapsed_time > search_limits.maximum_time - 10 {
            abort_search();
            return;
        }
    }
}

fn thread_handler(mut node: Bitboard, thread_depth: i32, max_depth: i32, thread_num: usize) {
    let mut depth = thread_depth;
    let mut val;
    let mut best_val = LB;
    while depth <= max_depth {
        let mut aspiration_delta_low = 250;
        let mut aspiration_delta_high = 250;
        loop {
            let mut alpha = LB;
            let mut beta = UB;
            if best_val > LB && depth > 1 {
                alpha = best_val - aspiration_delta_low;
                beta = best_val + aspiration_delta_high;
            }

            val = search(&mut node, alpha, beta, depth, 0, true, thread_num);
            if thread_killed() { return; }

            if val > alpha && val < beta {
                break;
            } else if val >= beta {
                aspiration_delta_high *= 2;
            } else {
                aspiration_delta_low *= 2;
            }
        }
        best_val = val;
        depth += 1;
    }
}

const OFF: u8 = 0;
const BRAIN: u8 = 1;
const HAND: u8 = 2;

pub fn best_move(node: &mut Bitboard, num_threads: u16, search_limits: SearchLimits, bh_mode: u8, bh_piece: i8) {
    let start_time = get_time_millis();
    let mut search_limits = search_limits;
    let max_time = search_limits.maximum_time;
    search_limits.maximum_time = 10000;
    // println!("key {}", node.hash);
    // println!("position repetition {}", (node.history.iter().filter(|&n| *n == node.hash).count() + 1));
    // loop {
    //     let mv = root_movepicker.next(node).0;
    //     if mv.is_null() { break; }
    //     node.do_move(&mv);
    //     if !node.is_check(!node.side_to_move) { root_moves.push(mv); }
    //     node.undo_move(&mv);
    // }
    unsafe {
        SEARCH_IN_PROGRESS = true;
        ABORT = false;
        STOP_THREAD = false;
        START_TIME = start_time;
        SEARCH_LIMITS = search_limits;
        STATIC_EVALS = 0;
        MAIN_SEARCH_NODES = 0;
        TT_VALID = 0;
        SS = Vec::new();
        TI = Vec::new();
        for i in 0..num_threads {
            SS.push(new_searchstats());
            TI.push(ThreadInfo::new());
            // TI[i as usize].root_moves = root_moves.clone();
            TI[i as usize].bh_piece = bh_piece;
        }
    }

    if bh_mode == HAND {
        if bh_piece == -1 {
            println!("You didn't tell me which piece to move!");
            unsafe {
                SEARCH_IN_PROGRESS = false;
            }
            return;
        }
        let mut root_movepicker = MovePicker::perft_new();
        loop {
            let mv = root_movepicker.next(node).0;
            if mv.is_null() {
                unsafe {
                    SEARCH_IN_PROGRESS = false;
                }
                println!("There are no moves for that piece!");
                return;
            }
            if mv.start == bh_piece {
                node.do_move(&mv);
                let is_legal = !node.is_check(!node.side_to_move);
                node.undo_move(&mv);
                if is_legal {
                    break;
                }
            }
        }
    }

    let mut depth: i32 = 1;
    let mut current_time: u128 = 0;

    let mut best_move_changes = 0;
    let mut last_best_move_change = 0;
    let mut best_move: Move = Move::null_move();
    let mut val: i32;
    let mut best_val: i32 = LB;
    let mut pv: &Vec<Move> = &vec![Move::null_move()];
    let mut nodes_searched = 0;


    allow_threads();
    let mut threads = Vec::new();
    for thread_num in 1..num_threads {
        // kick off threads
        let thread_depth = depth + thread_num.trailing_zeros() as i32;
        let thread_node = node.thread_copy();

        threads.push(thread::spawn(move || {
            thread_handler(thread_node, thread_depth, search_limits.depth, thread_num as usize);
        }));
    }

    while depth <= search_limits.depth {
        let mut aspiration_delta_low = 250;
        let mut aspiration_delta_high = 250;
        loop {
            let mut alpha = LB;
            let mut beta = UB;
            if depth > 1 {
                alpha = best_val - aspiration_delta_low;
                beta = best_val + aspiration_delta_high;
            }

            unsafe {
                TI[0].seldepth = 0;
                SS[0][0].pv = vec![best_move];
            }
            val = search(node, alpha, beta, depth, 0, true, 0);
            if search_aborted() {break;}

            if val > alpha && val < beta {
                break;
            } else if val >= beta {
                aspiration_delta_high *= 2;
            } else {
                aspiration_delta_low *= 2;
            }
        }
        if search_aborted() { break; }
        best_val = val;

        let elapsed_time;
        nodes_searched = 0;
        unsafe {
            if depth > 4 {
                SEARCH_LIMITS.maximum_time = max_time;
                search_limits.maximum_time = max_time;
            }
            pv = &SS[0][0].pv;
            let mv = pv[0];
            if mv != best_move {
                last_best_move_change = 0;
                best_move_changes += 1;
            } else {
                last_best_move_change += 1;
            }
            best_move = mv;

            current_time = cmp::max(get_time_millis(), start_time + 1);
            elapsed_time = current_time - start_time;

            for t_num in 0..num_threads {
                nodes_searched += TI[t_num as usize].nodes_searched;
            }

            if bh_mode == OFF {
                print_info(depth, TI[0].seldepth, &pv, best_val, current_time - start_time, nodes_searched);
            } else {
                println!("Thinking... depth {}", depth);
            }
            // for i in 0..MAX_PLY {
            //     SS[0][i].pv = Vec::new();
            // }
        }

        // we've obviously run out of time
        if search_limits.movetime > 0 && elapsed_time > search_limits.movetime {
            abort_search();
            break;
        } else if search_limits.maximum_time > 0 && elapsed_time > search_limits.maximum_time * 2 / 3 {
            abort_search();
            break;
        }

        // it's less obvious that we have
        if depth > 4 && search_limits.use_variable_time {
            if elapsed_time >= search_limits.optimum_time {
                // general idea here, inspired by some combination of SF and Ethereal
                // but then simplified by my laziness and then made sloppy
                // is that the more often the PV changes and the more recently it changed
                // the longer we allow ourselves to search

                // the specific multiplies here though are completely arbitrary
                // and subject to change
                let opttime = search_limits.optimum_time as f64;
                let last_change_factor = (18 - last_best_move_change) as f32 / 3.;
                let mod_factor = (1.0 + (best_move_changes as f32 / 4.) + last_change_factor) as f64;
                let target_time = if mod_factor < 1.0 {opttime} else {(opttime * mod_factor) as f64} as u128;

                if elapsed_time > target_time {
                    abort_search();
                    break;
                }
            }
        }

        depth += 1;
    }
    stop_threads();
    for t in threads {
        let res = t.join();
        match res {
            Err(_) => panic!("Error encountered in thread!"),
            _ => {}
        }
    }

    let emoji = if best_val <= -MIN_MATE_SCORE {
        "XO"
    } else if best_val <= -10000 {
        "X("
    } else if best_val <= -4000 {
        ">:C"
    } else if best_val <= -1000 {
        ":("
    } else if best_val < 0 {
        ":I"
    } else if best_val < 1000 {
        ":|"
    } else if best_val < 4000 {
        ":)"
    } else if best_val < 10000 {
        ":D"
    } else if best_val < MIN_MATE_SCORE {
        ":3"
    } else {
        ">:3"
    };

    if search_limits.infinite {
        unsafe {
            while !ABORT {
                std::thread::sleep(time::Duration::from_millis(50));
            }
        }
    }
    if bh_mode == BRAIN {
        let piece = match best_move.piece {
            b'k' => {"king"},
            b'q' => {"queen"},
            b'r' => {"rook"},
            b'b' => {"bishop"},
            b'n' => {"knight"},
            b'p' => {"pawn"},
            _ => {"???"}
        };
        println!("bestpiece {}", idx_to_str(best_move.start));
        // >:C :( :| :) :D
        // -4  -1  0  1  4
        eprintln!("Hey, you should move the {} on {} {}", piece, idx_to_str(best_move.start), emoji);
    } else {
        if bh_mode != OFF {
            println!("bestmove {} {}", best_move, emoji);
        } else {
            // unsafe {
            //     println!("MS NODES {} MAIN SEARCH EVALS {} TT_VALID {}", MAIN_SEARCH_NODES, STATIC_EVALS, TT_VALID);
            // }
            // unsafe {
            //     print_info(depth, TI[0].seldepth, &pv, best_val, current_time - start_time, nodes_searched);
            // }
            println!("bestmove {}", best_move);
        }
    }
    unsafe {
        LAST_BESTMOVE = best_move;
        SEARCH_IN_PROGRESS = false;
    }
}

fn search(node: &mut Bitboard, alpha: i32, beta: i32, depth: i32, ply: i32, is_pv: bool, thread_num: usize) -> i32 {
    if thread_killed() {
        return 0;
    }

    let mut ti: &mut ThreadInfo;
    let ss: &SearchStats;
    let mut sse: &mut SearchStatsEntry;
    unsafe {
        // x86_64::_mm_prefetch(TT.get_ptr(node.hash), x86_64::_MM_HINT_T0);
        ti = &mut TI[thread_num];
        ss = &SS[thread_num];
        sse = &mut SS[thread_num][ply as usize];

        if thread_num == 0 && ti.nodes_searched % 1024 == 0 {
            // main thread
            check_time(&SEARCH_LIMITS);
        }
    }

    let init_node = ply == 0;

    sse.pv.clear();
    sse.current_move = Move::null_move();

    if ply > ti.seldepth {
        ti.seldepth = ply;
    }
    ti.nodes_searched += 1;

    let mut alpha = alpha;
    let mut beta = beta;
    if !init_node {
        if node.is_repetition() || node.is_fifty_move() || node.insufficient_material() {
            return DRAW_SCORE;
        }

        // mate distance pruning
        alpha = cmp::max(alpha, -MATE_SCORE + ply);
        beta = cmp::min(beta, MATE_SCORE - (ply + 1));
        if alpha >= beta {
            return alpha;
        }
    }

    let mut depth = depth;
    let is_check = node.is_check(node.side_to_move);

    // EXPERIMENT: check extensions *before* qsearch
    if is_check {
        depth += 1
    }

    if depth <= 0 {
        sse.pv.clear();
        return qsearch(node, alpha, beta, thread_num);
    }

    if sse.excluded_move.is_null() {
        unsafe {
            let tt_entry = TT.get(node.hash);
            if tt_entry.valid() {
                sse.tt_hit = true;
                sse.tt_move = tt_entry.mv;
                sse.tt_val = TTEntry::read_tt_score(tt_entry.value, ply);
                sse.tt_depth = tt_entry.depth as i32;
                sse.tt_node_type = tt_entry.node_type;
            } else {
                sse.tt_hit = false;
                sse.tt_move = Move::null_move();
                sse.tt_val = 0;
                sse.tt_depth = 0;
                sse.tt_node_type = PV_NODE;
            }
        }
    }
    if sse.tt_hit {
        if !is_pv && sse.tt_depth >= depth && sse.excluded_move.is_null() {
            let node_type = sse.tt_node_type;
            let tt_val = sse.tt_val;
            if (node_type & CUT_NODE) != 0 && tt_val >= beta {
                return tt_val;
            } else if (node_type & ALL_NODE) != 0 && tt_val <= alpha {
                return tt_val;
            }
        }
    } else if depth >= 6 && !init_node {
        // internal iterative reductions
        // First place I can find IIR comes from a thread by Ed Schroeder (ProDeo author)
        // wherein they found success simply reducing the depth at unsorted subtrees
        // After hundreds of games of self-play, the idea does bear fruit in Mantissa
        // to the tune of about 20 self-ply Elo
        depth -= 1;
    }

    if is_check {
        // We won't do any pruning based on static eval here
        // so this is for the improving flag.  Set the eval prohibitively high
        // so the improving flag for subsequent move is false
        sse.static_eval = MATE_SCORE + 1;
    } else if sse.excluded_move.is_null() {
        sse.static_eval = static_eval(node, &mut ti.pht);
        unsafe {STATIC_EVALS += 1; TT_VALID += if sse.tt_hit {1} else {0};}
    }
    let eval = sse.static_eval;

    // is our position getting better than it was a move ago?
    // if so we might want to be more conservative about reductions and pruning
    // moreso because we want to slightly more aggressively prune moves that *aren't*
    // making things better.
    let improving = ply >= 2 && (!is_check && eval > ss[(ply - 2) as usize].static_eval);

    // there are some situations in which we typically don't want to prune
    // - there is an excluded move
    // - we are in check
    // - we are in a pv node search
    let pruning_safe = !is_check && !is_pv && !init_node && (ply + depth > 3) && sse.excluded_move.is_null();

    // Reverse Futility Pruning
    // AKA if our position is really really good
    // like better than it ought to be
    // chances are it will remain too good to be true
    // in the remaining ply of the search
    if depth < RFP_DEPTH && pruning_safe  {
        if (eval - rfp_margin(depth)) >= beta {
            return eval;
        }
    }

    // If we are way, way below alpha (generally an amount that is considered unsalvageable)
    // just stop here.  You're not going to make back up being 25 pawns below alpha in the
    // remaining few ply of the search.
    if depth < AFP_DEPTH && pruning_safe {
        if (eval + afp_margin(depth)) <= alpha {
            return eval;
        }
    }

    // // Razoring
    // // if we're adjacent to a leaf and behind alpha by a lot, just drop into quiescence search
    if depth < 3 && pruning_safe {
        if (eval + RAZORING_MARGIN * depth) < alpha {
            let val = qsearch(node, alpha, beta, thread_num);
            if val <= alpha {
                return val;
            }
        }
    }

    // reset killers for the next ply
    if ply < (MAX_PLY - 1) as i32 {
        ti.killers[(ply + 1) as usize] = [Move::null_move(); 2];
    }

    // Null Move Pruning
    // Similar idea to RFP above, but more nuanced.
    // If we have a position that seems to be above beta, we check if the position is in fact so good
    // that we can still be ahead even if we stay still and give our opponent a second move in a row.
    //
    // There are some restrictions, mostly that we want to avoid messing up in zugzwang positions
    // by checking for nonpawn material (most zugzwang occurs in king and pawn endgames)
    //
    // we also don't want to do things like 2 null moves in a row
    // and we don't want to pollute our singular move search
    if pruning_safe
        && depth >= NMP_DEPTH
        && eval >= beta
        && !ss[(ply - 1) as usize].searching_null_move
        && (ply < 2 || !ss[(ply - 2) as usize].searching_null_move)
        && node.stm_has_non_pawn_material()
    {
        let r = null_move_r(eval, beta, depth);

        sse.searching_null_move = true;
        node.do_null_move();
        let val = -search(node, -beta, -beta + 1, depth - r, ply + 1, false, thread_num);
        node.undo_null_move();
        sse.searching_null_move = false;

        if val >= beta {
            return beta;
            // if depth < 14 {
            //     return val;
            // } else {
            //     // super duper jank verification jank hack adventure
            //     sse.searching_null_move = true;
            //     let verification_val = search(node, beta-1, beta, cmp::max(depth / 2, depth - r - 2), ply, false, thread_num);
            //     sse.searching_null_move = false;
            //     if verification_val >= beta {
            //         return val;
            //     }
            // }
        }
    }

    // Singular Extensions + Multi-cut:
    // If we have a decent guess for best move from the TT, we want
    // to test this move for 'singularity'. That is, is this move the only
    // sane move in this position by a fair margin.  If so, we're in
    // a sort of extended set of tactics or walking a fine line, and we
    // definitely don't want to stop searching before we see this situation to a
    // conclusion.

    // On the other hand, if we have a move that already gives a outcome that's
    // better than beta, and even when excluding that move we can still beat beta
    // (at least, in a reduced search), then this node is probably a cut-node.

    // This takes an approach very similar to the one found in Stockfish, where
    // singular extensions and multi-cut are handled via the same search.  Though SF
    // takes it one step further
    let mut sing_extend = false;
    if !init_node
        && depth >= 8
        && !is_check
        && sse.tt_hit
        && sse.excluded_move.is_null()
        && sse.tt_val.abs() < MIN_MATE_SCORE
        && (sse.tt_node_type & CUT_NODE) != 0
        && sse.tt_depth >= depth - 3
    {
        let margin = SINGULAR_MARGIN_FACTOR * depth;
        let depth_to_search = (depth - 1) / 2;
        let target = sse.tt_val - margin;

        sse.excluded_move = sse.tt_move;
        let val = search(node, target - 1, target, depth_to_search, ply, false, thread_num);
        sse.excluded_move = Move::null_move();

        if val < target {
            sing_extend = true;
        } else if target >= beta {
            return target;
        }
    }

    let mut raised_alpha = false;
    let mut best_move = Move::null_move();
    let mut best_val = LB;
    let mut moves_searched: i32 = 0;
    let prev_mv = if !init_node {ss[(ply - 1) as usize].current_move} else {Move::null_move()};
    let my_prev_mv = if ply >= 2 {ss[(ply - 2) as usize].current_move} else {Move::null_move()};

    // Countermove Heuristic
    // The idea here is that many moves have a natural response
    // i.e. a counter. When a quiet move causes a fail high
    // we may consider that move a potential "counter" to the move
    // that preceded it, so we give it a bonus in move ordering
    // let countermove = if prev_mv.is_null() {
    //     Move::null_move()
    // } else {
    //     let piece_num = get_piece_num(prev_mv.piece, !node.side_to_move);
    //     ti.countermove_table[piece_num][prev_mv.end as usize]
    // };

    let mut movepicker = MovePicker::new(sse.tt_move, ply, thread_num, false);

    // Futility Pruning.  The 'futile' flag signals
    // to skip quiet moves.  Different conditions in the search
    // loop can activate this flag to prune all subsequent quiet moves
    // from then on.
    let mut futile = false;

    let mut found_legal_move = false;
    let mut searched_moves: Vec<Move> = Vec::new();

    let mut move_idx = 0;

    loop {
        let (mv, score) = if true || !init_node {
            movepicker.next(node)
        } else {
            move_idx += 1;
            if move_idx > ti.root_moves.len() {
                (Move::null_move(), 0)
            } else {
                (ti.root_moves[move_idx-1], OK_CAPTURE_OFFSET)
            }
        };
        sse.current_move = mv;

        if mv.is_null() {
            // exhausted all moves
            break;
        }

        if init_node && ti.bh_piece >= 0 {
            if mv.start != ti.bh_piece {
                continue;
            }
        }

        if mv == sse.excluded_move {
            continue;
        }

        let is_tactical = is_tactical_move(&mv, node);
        let is_quiet = is_quiet_move(&mv, node);

        if !is_check && (depth + ply > 3) && !init_node && best_val > -MIN_MATE_SCORE && !futile {
            let lmr_depth = depth - 1 - lmr_reduction(depth, moves_searched);

            // Basic form of late move pruning
            if depth <= 8 && moves_searched >= lmp_count(improving, depth) {
                futile = true;
            }

            if depth < EFP_DEPTH && eval + efp_margin(depth) <= alpha && alpha.abs() < MIN_MATE_SCORE && !futile {
                futile = true;
            }

            // History-leaf pruning
            // Slightly more lenient on eval margin than futility pruning
            // but won't kick in until we've tried all moves with good history scores
            if lmr_depth <= 6 && is_quiet && eval + fp_margin(depth) <= alpha && alpha.abs() < MIN_MATE_SCORE && movepicker.move_stage > GEN_QUIET && !futile {
                let hist = (score as i32) - QUIET_OFFSET as i32;
                if hist < HISTORY_LEAF_PRUNING_MARGIN {
                    futile = true
                }
            }

            if lmr_depth < 4 && movepicker.move_stage > GEN_QUIET && is_quiet && !futile {
                // let mut move_hist = 0;
                let countermove_hist;
                let followup_hist;

                let piece_num = get_piece_num(mv.piece, node.side_to_move);
                // move_hist = ti.move_history[piece_num][mv.end as usize];

                // Continuation history pruning
                // As we near the leaves, quiet moves which have particularly bad
                // counter or followup histories are pruned.  Ethereal does something
                // similar here, but instead of basing the threshold on 'improving'
                // I instead make it depth-based.
                if !prev_mv.is_null() {
                    let prev_piece_num = get_piece_num(prev_mv.piece, !node.side_to_move);
                    countermove_hist = ti.countermove_history[prev_piece_num][prev_mv.end as usize][piece_num][mv.end as usize];
                    if countermove_hist <= COUNTERMOVE_PRUNING_FACTOR * lmr_depth { continue; }
                }
                if !my_prev_mv.is_null() {
                    let prev_piece_num = get_piece_num(my_prev_mv.piece, node.side_to_move);
                    followup_hist = ti.followup_history[prev_piece_num][my_prev_mv.end as usize][piece_num][mv.end as usize];
                    if followup_hist <= FOLLOWUP_PRUNING_FACTOR * lmr_depth { continue; }
                }
            }
        }

        if futile {
            // found_legal_move = true;
            if !is_tactical && movepicker.move_stage > GEN_QUIET {
                // node.undo_move(&mv);
                continue;
            }
        }

        node.do_move(&mv);
        if node.is_check(!node.side_to_move) {
            // Illegal
            node.undo_move(&mv);
            continue;
        }
        if depth > 1 {
            // unsafe {x86_64::_mm_prefetch(TT.get_ptr(node.hash), x86_64::_MM_HINT_T0);}
            unsafe {TT.prefetch(node.hash);}
        }

        found_legal_move = true;
        if is_quiet {
            searched_moves.push(mv);
        }
        moves_searched += 1;
        let mut val = LB;
        if moves_searched == 1 {
            val = -search(node, -beta, -alpha, if sing_extend {depth} else {depth - 1}, ply + 1, is_pv, thread_num);
            if is_pv {
                unsafe {
                    let child_ss = &mut SS[thread_num][(ply + 1) as usize];
                    sse.pv.push(mv);
                    sse.pv.append(&mut child_ss.pv);
                }
            }
        } else {
            let mut do_full_zw_search = true;
            if depth > LMR_DEPTH
                // && !init_node
                && moves_searched > if is_pv {3} else {2}
                && is_quiet
            {
                do_full_zw_search = false;
                let mut r = lmr_reduction(depth, moves_searched);

                if is_check { r -= 1; }
                if !improving { r += 1; }
                if is_pv { r -= 1; }
                // give a bit of leeway to killers and countermove
                if movepicker.move_stage <= GEN_QUIET { r -= 1; }

                // adjust r based on history of other quiet moves
                let hist = (score as i32) - QUIET_OFFSET as i32;
                // let hist = move_hist + countermove_hist + followup_hist;
                r -= cmp::max(-2, cmp::min(2, hist / LMR_HISTORY_DENOMINATOR));

                let lmr_depth = cmp::min(cmp::max(1, depth - 1 - r), depth - 1);
                val = -search(node, -alpha - 1, -alpha, lmr_depth, ply + 1, false, thread_num);
                if val > alpha && lmr_depth < depth - 1 {
                    // if we raise alpha in the reduced search,
                    // we have to see if we can raise it in the full-depth
                    do_full_zw_search = true;
                }
            }
            if do_full_zw_search {
                val = -search(node, -alpha - 1, -alpha, depth - 1, ply + 1, false, thread_num);
            }
            if is_pv && val > alpha && val < beta {
                val = -search(node, -beta, -alpha, depth - 1, ply + 1, true, thread_num);
                unsafe {
                    let child_ss = &mut SS[thread_num][(ply + 1) as usize];
                    sse.pv.clear();
                    sse.pv.push(mv);//= vec![mv];
                    sse.pv.append(&mut child_ss.pv);
                }
                // if init_node {
                //     // move the move to the front
                //     ti.root_moves.remove(move_idx - 1);
                //     ti.root_moves.insert(0, mv);
                // }
            }
        }
        node.undo_move(&mv);
        if thread_killed() { return 0; }

        if val > best_val {
            best_move = mv;
            best_val = val;
        }
        if val > alpha {
            alpha = val;
            raised_alpha = true;
        }
        if alpha >= beta {
            // fail-high
            if !thread_killed() {
                if sse.excluded_move.is_null() {
                    if is_quiet {
                        // update heuristics
                        ti.update_killers(mv, ply);
                        ti.update_move_history(mv, node.side_to_move, depth, &searched_moves);
                        ti.update_countermove(prev_mv, mv, !node.side_to_move);
                        ti.update_followup(my_prev_mv, mv, node.side_to_move, depth, &searched_moves);
                        ti.update_countermove_history(prev_mv, mv, node.side_to_move, depth, &searched_moves);
                    }
                    unsafe {
                        TT.set(node.hash, best_move, TTEntry::make_tt_score(val, ply), CUT_NODE, depth, node.history.len() as i32);
                    }
                }
            }
            return val;
        }
    }

    if best_move.is_null() {
        if !sse.excluded_move.is_null() {
            return alpha;
        }
        if !found_legal_move {
            // some sort of mate
            sse.pv = Vec::new();
            if is_check {
                return -MATE_SCORE + ply;
            } else {
                return DRAW_SCORE;
            }
        } else {
            return alpha;
        }
    }

    unsafe {
        if sse.excluded_move.is_null() && !thread_killed() {
            TT.set(node.hash, best_move, TTEntry::make_tt_score(best_val, ply), if raised_alpha {PV_NODE} else {ALL_NODE}, depth, node.history.len() as i32);
        }
    }

    return best_val;
}


pub fn qsearch(node: &mut Bitboard, alpha: i32, beta: i32, thread_num: usize) -> i32 {
    let mut ti: &mut ThreadInfo;
    unsafe {
        ti = &mut TI[thread_num];
    }
    ti.nodes_searched += 1;
    // unsafe {STATIC_EVALS += 1;}
    if node.is_quiet() { return static_eval(node, &mut ti.pht); }
    // unsafe {
    //     let tt_entry = TT.get(node.hash);
    //     if tt_entry.valid() {
    //         if (tt_entry.node_type == PV_NODE) {
    //             return tt_entry.value;
    //         }
    //         if (tt_entry.node_type & CUT_NODE != 0) && tt_entry.value >= beta {
    //             return tt_entry.value;
    //         }
    //         if (tt_entry.node_type & ALL_NODE != 0) && tt_entry.value <= alpha {
    //             return tt_entry.value;
    //         }
    //     }
    // }

    // let mut raised_alpha = false;
    let mut alpha = alpha;

    let stand_pat = static_eval(node, &mut ti.pht);

    let is_check = node.is_check(node.side_to_move);
    let phase = node.get_phase();
    // standing pat check so we *do* stop eventually
    if !is_check {
        if stand_pat >= beta {
            return stand_pat;
        } else if stand_pat > alpha {
            // raised_alpha = true;
            alpha = stand_pat;
        }
    }
    if stand_pat < alpha - (taper_score(QUEEN_VALUE, phase) + 2000) {
        return stand_pat;
    }

    let mut best_val = stand_pat;

    let mut movepicker = MovePicker::q_new();
    loop {
        let (mv, score) = movepicker.next(node);
        if mv.is_null() {
            break;
        }

        node.do_move(&mv);
        if node.is_check(!node.side_to_move) { node.undo_move(&mv); continue; }

        // delta pruning
        // if we're very behind of where we could be (alpha)
        // we should only accept exceptionally good captures

        if node.has_non_pawn_material() {
            let mut futile = false;
            match node.get_last_capture() {
                b'p' => { if alpha > stand_pat + taper_score(PAWN_VALUE, phase) + 2000 { futile = true; }},
                b'n' => { if alpha > stand_pat + taper_score(KNIGHT_VALUE, phase) + 2000 { futile = true; }},
                b'b' => { if alpha > stand_pat + taper_score(BISHOP_VALUE, phase) + 2000 { futile = true; }},
                b'r' => { if alpha > stand_pat + taper_score(ROOK_VALUE, phase) + 2000 { futile = true; }},
                b'q' => { if alpha > stand_pat + taper_score(QUEEN_VALUE, phase) + 2000 { futile = true; }},
                _ => {}
            }
            if futile {
                node.undo_move(&mv);
                continue;
            }
        }

        if score < QUIET_OFFSET && mv.promote_to == 0 {
            node.undo_move(&mv);
            continue;
        }
        let val = -qsearch(node, -beta, -alpha, thread_num);
        node.undo_move(&mv);
        if val > best_val {
            best_val = val;
        }
        if val > alpha {
            // raised_alpha = true;
            alpha = val;
        }
        if val >= beta {
            break;
        }
    }

    // let node_type = if best_val >= beta {CUT_NODE} else {if raised_alpha {PV_NODE} else {ALL_NODE}};
    // unsafe { TT.set(node.hash, Move::null_move(), best_val, node_type, 0, 0); }

    return best_val;
}
