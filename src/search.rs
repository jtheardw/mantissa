use std::cmp;
use std::time::SystemTime;
use std::thread;

use crate::bitboard::*;
use crate::eval::*;
use crate::moveorder::*;
use crate::moveutil::*;
use crate::searchutil::*;
use crate::see::*;
use crate::tt::*;
use crate::util::*;

static mut SEARCH_IN_PROGRESS: bool = false;
static mut ABORT: bool = false;
static mut STOP_THREAD: bool = false;
static mut START_TIME: u128 = 0;
static mut SEARCH_LIMITS: SearchLimits = SearchLimits::infinite();

const LB: i32 = -10000000;
const UB: i32 = 10000000;

static mut TI: Vec<ThreadInfo> = Vec::new();
static mut SS: Vec<SearchStats> = Vec::new();

pub fn get_time_millis() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime failed!"),
    }
}

pub fn ongoing_search() -> bool {
    unsafe {
        SEARCH_IN_PROGRESS
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

// todo move this to uci
fn print_info(depth: i32, seldepth: i32, pv: &Vec<Move>, val: i32, time: u128, nodes: u64) {
    let pv_str = get_pv_str(pv);
    let val_str = get_val_str(val);
    let nps = (nodes * 1000) as u128 / time;
    println!("info depth {} seldepth {} pv {} score {} time {} nodes {} nps {}",
             depth, seldepth, pv_str, val_str, time, nodes, nps
    );
}

pub fn abort_search() {
    unsafe {
        ABORT = true;
    }
}

pub fn stop_threads() {
    unsafe {
        STOP_THREAD = true;
    }
}

pub fn allow_threads() {
    unsafe {
        STOP_THREAD = false;
    }
}

pub fn search_aborted() -> bool {
    unsafe {
        ABORT
    }
}

pub fn thread_killed() -> bool {
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

fn thread_handler(mut node: Bitboard, thread_depth: i32, thread_num: usize) {
    let mut depth = thread_depth;
    let mut val = LB;
    loop {
        let mut aspiration_delta = 250;
        loop {
            let mut alpha = LB;
            let mut beta = UB;
            if val > LB {
                alpha = val - aspiration_delta;
                beta = val + aspiration_delta;
            }

            val = search(&mut node, alpha, beta, depth, 0, true, false, thread_num);
            if thread_killed() { return; }

            if val > alpha && val < beta {
                break;
            } else {
                aspiration_delta *= 2;
            }
        }
        depth += 1;
    }
}

pub fn best_move(node: &mut Bitboard, num_threads: u16, search_limits: SearchLimits) { // TODO threads
    let start_time = get_time_millis();
    unsafe {
        SEARCH_IN_PROGRESS = true;
        ABORT = false;
        STOP_THREAD = false;
        START_TIME = start_time;
        SEARCH_LIMITS = search_limits;
        TI = Vec::new();
        SS = Vec::new();

        for _ in 0..num_threads {
            TI.push(ThreadInfo::new());
            SS.push(new_searchstats());
        }
    }

    let mut depth: i32 = 1;
    let mut current_time: u128;

    let mut best_move_changes = 0;
    let mut last_best_move_change = 0;
    let mut best_move: Move = Move::null_move();
    let mut val: i32 = LB;
    let mut pv: &Vec<Move>;

    // TODO aspiration windows
    while depth <= search_limits.depth {
        allow_threads();
        let mut threads = Vec::new();
        for thread_num in 1..num_threads {
            // kick off threads
            let thread_depth = depth + thread_num.trailing_zeros() as i32;
            let thread_node = node.thread_copy();

            // search(&mut thread_node, alpha, beta, thread_depth, 0, true, false, thread_num as usize);
            threads.push(thread::spawn(move || {
                thread_handler(thread_node, thread_depth, thread_num as usize);
            }));
        }

        let mut aspiration_delta = 250;
        loop {
            let mut alpha = LB;
            let mut beta = UB;
            if depth > 1 {
                alpha = val - aspiration_delta;
                beta = val + aspiration_delta;
            }

            val = search(node, alpha, beta, depth, 0, true, false, 0);
            if search_aborted() {break;}

            if val > alpha && val < beta {
                break;
            } else {
                aspiration_delta *= 2;
            }
        }
        stop_threads();
        for t in threads {
            t.join();
        }
        if search_aborted() { break; }

        let elapsed_time;
        unsafe {
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

            let mut nodes_searched = 0;
            for t_num in 0..num_threads {
                nodes_searched += TI[t_num as usize].nodes_searched;
            }

            print_info(depth, TI[0].seldepth, &pv, val, current_time - start_time, nodes_searched);
            for i in 0..MAX_PLY {
                SS[0][i].pv = Vec::new();
            }
        }

        // we've obviously run out of time
        if search_limits.movetime > 0 && elapsed_time > search_limits.movetime {
            abort_search();
            break;
        } else if search_limits.maximum_time > 0 && elapsed_time > search_limits.maximum_time - 10 {
            abort_search();
            break;
        }

        // it's less obvious that we have
        if search_limits.use_variable_time {
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
    println!("bestmove {}", best_move);
    unsafe {
        SEARCH_IN_PROGRESS = false;
    }
}

fn search(node: &mut Bitboard,
          alpha: i32,
          beta: i32,
          depth: i32,
          ply: i32,
          is_pv: bool,
          cut_node: bool,
          thread_num: usize) -> i32 {
    if thread_num == 0 {
        // main thread
        unsafe {
            check_time(&SEARCH_LIMITS);
        }
    }

    if thread_killed() {
        return 0;
    }

    let mut ti: &mut ThreadInfo;
    let ss: &SearchStats;
    let mut sse: &mut SearchStatsEntry;
    unsafe {
        ti = &mut TI[thread_num];
        ss = &SS[thread_num];
        sse = &mut SS[thread_num][ply as usize];
    }
    let init_node = ply == 0;
    sse.pv = Vec::new();

    if !init_node {
        if node.is_repetition() {
            return DRAW_SCORE;
        }
    }

    if ply > ti.seldepth {
        ti.seldepth = ply;
    }

    let mut depth = depth;
    if depth <= 0 {
        let is_check = node.is_check(node.side_to_move);
        if !is_check {
            sse.pv = Vec::new();
            return qsearch(node, alpha, beta, thread_num);
        }
        depth = 1;
    }

    ti.nodes_searched += 1;

    let mut alpha = alpha;
    let mut beta = beta;
    if !init_node {
        // mate distance pruning
        alpha = cmp::max(alpha, -MATE_SCORE + ply);
        beta = cmp::min(beta, MATE_SCORE - (ply + 1));
        if alpha >= beta {
            return alpha;
        }
    }
    unsafe {
        let tt_entry = TT.get(node.hash);
        if tt_entry.valid {
            sse.tt_hit = true;
            sse.tt_move = tt_entry.mv;
            sse.tt_val = TTEntry::read_tt_score(tt_entry.value, ply);
            sse.tt_depth = tt_entry.depth;
            sse.tt_node_type = tt_entry.node_type;
        } else {
            sse.tt_hit = false;
            sse.tt_move = Move::null_move();
            sse.tt_val = 0;
            sse.tt_depth = 0;
            sse.tt_node_type = PV_NODE;
        }
    }
    if sse.tt_hit {
        if !is_pv && sse.tt_depth >= depth && sse.excluded_move.is_null {
            let node_type = sse.tt_node_type;
            let tt_val = sse.tt_val;
            if (node_type & CUT_NODE) != 0 && tt_val >= beta {
                let mv = sse.tt_move;
                if is_quiet_move(&mv, node) {
                    ti.update_killers(mv, ply);
                    ti.update_move_history(mv, node.side_to_move, depth);
                }
                return tt_val;
            } else if (node_type & ALL_NODE) != 0 && tt_val <= alpha {
                return tt_val;
            }
        }
    } else if depth >= 6 && is_pv {
        // internal iterative deepening
        // if we fail to get a tt hit on a PV node, which should be fairly rare
        // at high depths, we'll do a reduced search to get a good guess for first move
        sse.pv = Vec::new();
        let val = search(node, alpha, beta, depth - 2, ply, true, false, thread_num);
        if sse.pv.len() > 0 {
            sse.tt_move = sse.pv[0];
            sse.pv = Vec::new();
            sse.tt_val = val;
        } else {
            // panic!("why wasn't there a pv here?");
        }
    }

    let is_check = node.is_check(node.side_to_move);
    sse.static_eval = static_eval(node);
    let eval = sse.static_eval;

    ti.killers[(ply + 1) as usize] = [Move::null_move(), Move::null_move()];

    // Reverse Futility Pruning
    if depth < RFP_DEPTH && !is_pv && !is_check && !init_node {
        if (eval - rfp_margin(depth)) >= beta {
            return eval - rfp_margin(depth);
        }
    }

    // Null Move Reductions
    if !is_pv
        && depth >= NMP_DEPTH
        && !is_check
        && !init_node
        && sse.static_eval >= beta
        && (!ss[(ply - 1) as usize].searching_null_move)
        && (ply < 2 || (!ss[(ply - 2) as usize].searching_null_move))
        && sse.excluded_move.is_null
        && node.has_non_pawn_material()
        && (!sse.tt_hit || sse.tt_node_type & CUT_NODE == 0 || sse.tt_val >= beta)
    {
        let r = null_move_r(sse.static_eval, beta, depth);
        // if r > 2 && depth > 10 {
        //     println!("{} {} {} {}", depth, r, sse.static_eval, beta);
        // }
        sse.searching_null_move = true;
        node.do_null_move();
        let val = -search(node, -beta, -beta + 1, depth - 1 - r, ply + 1, false, !cut_node, thread_num);
        node.undo_null_move();
        sse.searching_null_move = false;

        if val >= beta {
            // using the extended null move reductions
            // idea from Eli David and Nathan S. Netanyahu
            // in the paper of the same name
            return beta;
            // depth -= 4;
            // if depth <= 0 {
            //     return search(node, alpha, beta, 0, ply, false, false, thread_num);
            // }
        }
    }

    if true && !init_node
        && depth >= 8
        && sse.tt_hit
        && sse.excluded_move.is_null
        && sse.tt_val.abs() < MATE_SCORE - 100000 // TODO give this a name
        && (sse.tt_node_type & CUT_NODE) != 0
        && sse.tt_depth >= depth - 3
    {
        // I've stolen stockfish's idea here to combine singular
        // move detection with multi-cut in the same search
        //
        // I can't afford to do the super-tight cutoffs stockfish does though
        let tt_val = sse.tt_val;
        let tt_move = sse.tt_move;
        let former_pv = sse.tt_node_type == PV_NODE && !is_pv;
        let margin = if former_pv {25 * (depth / 2)} else {20 * (depth / 2)};
        let depth_to_search = if former_pv {(depth + 2) / 2} else {(depth - 1) / 2};
        let target = sse.tt_val - margin;

        sse.excluded_move = sse.tt_move;
        let val = search(node, target - 1, target, depth_to_search, ply, false, cut_node, thread_num);
        sse.excluded_move = Move::null_move();

        if val < target {
            // println!("val {} tt_val {} target {}", val, sse.tt_val, target);
            // singularly extend
            depth += 1;
        } else if target >= beta {
            // pseudo-multi-cut.  This indicates multiple moves failed high
            // so this is probably a cutnode
            return target;
        } else if tt_val >= beta {
            sse.excluded_move = tt_move;
            let val = search(node, beta - 1, beta, (depth + 3) / 2, ply, false, cut_node, thread_num);
            sse.excluded_move = Move::null_move();
            if val >= beta { return beta; }
        }
    }

    let mut raised_alpha = false;
    let mut best_move = Move::null_move();
    let mut best_val = LB;
    let mut moves_searched: i32 = 0;
    let mut movepicker = MovePicker::new(sse.tt_move, ti.killers[ply as usize], ti.move_history, false);
    // futility pruning
    let mut futile = false;
    if !is_pv && !is_check && depth <= EFP_DEPTH {
        if depth == EFP_DEPTH {
            if sse.static_eval < alpha - efp_margin(EFP_DEPTH) {
                depth -= 1;
            }
        }
        if sse.static_eval <= (alpha - efp_margin(depth)) {
            futile = true;
        }
    }

    let mut found_legal_move = false;
    loop {
        let (mv, score) = movepicker.next(node);
        if mv.is_null {
            // we've exhausted all the moves
            break;
        }

        if mv == sse.excluded_move {
            continue;
        }

        let is_tactical = is_tactical_move(&mv, node);
        let is_quiet = is_quiet_move(&mv, node);
        node.do_move(&mv);
        if node.is_check(!node.side_to_move) {
            node.undo_move(&mv);
            continue;
        }
        let gives_check = node.is_check(node.side_to_move);
        if futile {
            found_legal_move = true;//&& found_legal_move {
            if !is_tactical && !gives_check {
                node.undo_move(&mv);
                continue
            }
        }

        found_legal_move = true;
        let mut val = LB;
        if moves_searched == 0 {
            val = -search(node, -beta, -alpha, depth - 1, ply + 1, is_pv, false, thread_num);
            unsafe {
                let child_ss = &mut SS[thread_num][(ply + 1) as usize];
                sse.pv = vec![mv];
                sse.pv.append(&mut child_ss.pv);
            }
        } else {
            let mut do_full_zw_search = true;
            if depth > LMR_DEPTH
                && !init_node
                && moves_searched >= 4
                && !is_check
                && score < KILLER_OFFSET
                && (is_quiet || cut_node || score < QUIET_OFFSET) {
                // && (is_quiet || score < QUIET_OFFSET) {
                    do_full_zw_search = false;
                    let mut r = lmr_reduction(depth, moves_searched);
                    if gives_check { r -= 1; }

                    if cut_node { r += 1; }

                    if score >= KILLER_OFFSET { r -= 1; }

                    if !is_quiet {
                        let cap_piece = node.get_last_capture();
                        if cap_piece != 0 {
                            node.undo_move(&mv);
                            let see_score = see(node, mv.end, cap_piece, mv.start, mv.piece);
                            if see_score < 0 {
                                r += 1;
                            } else {
                                r -= 1;
                            }
                            node.do_move(&mv);
                        }
                    }

                    if sse.tt_hit && sse.tt_node_type == PV_NODE { r -= 1; }
                    if sse.tt_hit && sse.tt_node_type != PV_NODE { r += 1; }

                    if is_pv && r > 0 {
                        r = (r * 2) / 3;
                    }

                    let lmr_depth = cmp::min(cmp::max(1, depth - 1 - r), depth - 1);

                    val = -search(node, -alpha - 1, -alpha, lmr_depth, ply + 1, false, true, thread_num);
                    if val > alpha && lmr_depth < depth - 1 {
                        do_full_zw_search = true;
                    }
                }
            if do_full_zw_search {
                val = -search(node, -alpha - 1, -alpha, depth - 1, ply + 1, false, !cut_node, thread_num);
            }
            if is_pv && val > alpha && val < beta {
                val = -search(node, -beta, -alpha, depth - 1, ply + 1, true, false, thread_num);
                unsafe {
                    let child_ss = &mut SS[thread_num][(ply + 1) as usize];
                    sse.pv = vec![mv];
                    sse.pv.append(&mut child_ss.pv);
                }
            }
        }

        node.undo_move(&mv);
        if thread_killed() { return 0; }

        moves_searched += 1;
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
                // if depth > 8 {
                //     println!("fail high on move {} score {} quiet {}", moves_searched, score, is_quiet);
                // }
                if is_quiet {
                    // update heuristics
                    ti.update_killers(mv, ply);
                    ti.update_move_history(mv, node.side_to_move, depth);
                }
                if sse.excluded_move.is_null {
                    unsafe {
                        TT.set(node.hash, best_move, TTEntry::make_tt_score(val, ply), CUT_NODE, depth, node.history.len() as i32);
                    }
                }
            }
            return val;
        }
    }

    if best_move.is_null {
        // if !found_legal_move && !sse.excluded_move.is_null {
        //     node.do_move(&sse.excluded_move);
        //     if !node.is_check(!node.side_to_move) {
        //         found_legal_move = true;
        //     }
        //     node.undo_move(&sse.excluded_move);
        // }
        if !found_legal_move {
            // some sort of mate
            sse.pv = Vec::new();
            if is_check {
                return -MATE_SCORE + ply;
            } else {
                return DRAW_SCORE;
            }
        } else {
            // futility pruning weirdness?
            return sse.static_eval;
        }
    }

    unsafe {
        if sse.excluded_move.is_null && !thread_killed() {
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

    if node.is_quiet() { return static_eval(node); }

    let mut alpha = alpha;

    let stand_pat = static_eval(node);

    let is_check = node.is_check(node.side_to_move);
    // standing pat check so we *do* stop eventually
    if !is_check {
        if stand_pat >= beta {
            return stand_pat;
        } else if stand_pat > alpha {
            alpha = stand_pat;
        }
    }
    if stand_pat < alpha - 11000 && node.has_non_pawn_material() {
        return stand_pat;
    }

    let mut best_val = stand_pat;

    let mut movepicker = MovePicker::q_new();

    loop {
        let (mv, score) = movepicker.next(node);
        if mv.is_null {
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
                b'p' => { if alpha > stand_pat + 3000 { futile = true; }},
                b'n' => { if alpha > stand_pat + 5000 { futile = true; }},
                b'b' => { if alpha > stand_pat + 5000 { futile = true; }},
                b'r' => { if alpha > stand_pat + 7000 { futile = true; }},
                b'q' => { if alpha > stand_pat + 11000 { futile = true; }},
                _ => {}
            }
            if futile {
                node.undo_move(&mv);
                continue;
            }
        }

        // todo SEE based pruning
        if score < QUIET_OFFSET {
            // see if this is a viable capture
            let cap_piece = node.get_last_capture();
            if cap_piece != 0 {
                node.undo_move(&mv);
                let see_score = see(node, mv.end, cap_piece, mv.start, mv.piece);
                if see_score < 0 {
                    continue;
                } else {
                    node.do_move(&mv);
                }
            }
        }
        let val = -qsearch(node, -beta, -alpha, thread_num);
        node.undo_move(&mv);
        if val > best_val {
            best_val = val
        }
        if val > alpha {
            alpha = val;
        }
        if val >= beta {
            return val;
        }
    }

    return best_val;
}
