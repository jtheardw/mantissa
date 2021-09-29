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

fn print_info(depth: i32, seldepth: i32, pv: &Vec<Move>, val: i32, time: u128, nodes: u64) {
    let pv_str = get_pv_str(pv);
    let val_str = get_val_str(val);
    let nps = (nodes * 1000) as u128 / time;
    println!("info depth {} seldepth {} score {} time {} nodes {} nps {} pv {}",
             depth, seldepth, val_str, time, nodes, nps, pv_str
    );
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
    let mut val = LB;
    while depth <= max_depth {
        let mut aspiration_delta = 250;
        loop {
            let mut alpha = LB;
            let mut beta = UB;
            if val > LB {
                alpha = val - aspiration_delta;
                beta = val + aspiration_delta;
            }

            val = search(&mut node, alpha, beta, depth, 0, true, thread_num);
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

pub fn best_move(node: &mut Bitboard, num_threads: u16, search_limits: SearchLimits) {
    let start_time = get_time_millis();
    let mut search_limits = search_limits;
    let max_time = search_limits.maximum_time;
    search_limits.maximum_time = 10000;
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

    let mut depth: i32 = 2;
    let mut current_time: u128;

    let mut best_move_changes = 0;
    let mut last_best_move_change = 0;
    let mut best_move: Move = Move::null_move();
    let mut val: i32 = LB;
    let mut pv: &Vec<Move>;

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
        let mut aspiration_delta = 250;
        loop {
            let mut alpha = LB;
            let mut beta = UB;
            if depth > 1 {
                alpha = val - aspiration_delta;
                beta = val + aspiration_delta;
            }

            val = search(node, alpha, beta, depth, 0, true, 0);
            if search_aborted() {break;}

            if val > alpha && val < beta {
                break;
            } else {
                aspiration_delta *= 2;
            }
        }
        if search_aborted() { break; }

        let elapsed_time;
        unsafe {
            SEARCH_LIMITS.maximum_time = max_time;
            search_limits.maximum_time = max_time;
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
    abort_search();
    for t in threads {
        let res = t.join();
        match res {
            Err(_) => panic!("Error encountered in thread!"),
            _ => {}
        }
    }

    println!("bestmove {}", best_move);
    unsafe {
        SEARCH_IN_PROGRESS = false;
    }
}

fn see_score(pos: &mut Bitboard, mv: Move) -> i32 {
    return see(pos, mv.end, pos.piece_at_square(mv.end, !pos.side_to_move), mv.start, mv.piece);
}

fn search(node: &mut Bitboard,
          alpha: i32,
          beta: i32,
          depth: i32,
          ply: i32,
          is_pv: bool,
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
    if depth <= 0 {
        let is_check = node.is_check(node.side_to_move);
        if !is_check {
            sse.pv = Vec::new();
            return qsearch(node, alpha, beta, thread_num);
        }
        depth = 1;
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
                    // we still want to update our heuristic counters
                    // here.
                    let my_prev_mv = if ply >= 2 {
                        ss[(ply - 2) as usize].current_move
                    } else {
                        Move::null_move()
                    };
                    let prev_mv = if !init_node {
                        ss[(ply - 1) as usize].current_move
                    } else {
                        Move::null_move()
                    };

                    ti.update_killers(mv, ply);
                    ti.update_move_history(mv, node.side_to_move, depth, &Vec::new());
                    ti.update_countermove(prev_mv, mv, !node.side_to_move);
                    ti.update_followup(my_prev_mv, mv, node.side_to_move, depth, &Vec::new());
                }
                return tt_val;
            } else if (node_type & ALL_NODE) != 0 && tt_val <= alpha {
                return tt_val;
            }
        }
    } else if depth >= 6 && !init_node {
        // internal iterative reductions
        // First place I can find IIR comes from a thread by Rebel and ProDeo author
        // wherein they found success simply reducing the depth at unsorted subtrees
        // After hundreds of games of self-play, the idea does bear fruit in Mantissa
        // to the tune of about 20 self-ply Elo
        depth -= 1;
    }

    let is_check = node.is_check(node.side_to_move);
    sse.static_eval = static_eval(node, &mut ti.pht);
    let eval = sse.static_eval;

    // is our position getting better than it was a move ago?
    // if so we might want to be more conservative about reductions and pruning
    // moreso because we want to slightly more aggressively prune moves that *aren't*
    // making things better.
    let improving = ply >= 2 && !is_check && eval > ss[(ply - 2) as usize].static_eval;

    // clear out the killers for our children.
    // this is an experiment in move ordering that I've seen in Ethereal
    // there are plenty of strong engines that don't do this
    // so it's just being played around with.
    ti.killers[(ply + 1) as usize] = [Move::null_move(), Move::null_move()];

    // Reverse Futility Pruning
    // AKA if our position is really really good
    // like better than it ought to be
    // chances are it will remain too good to be true
    // in the remaining ply of the search
    if depth < RFP_DEPTH && !is_pv && !is_check {
        if (eval - rfp_margin(depth)) > beta {
            return eval - rfp_margin(depth);
        }
    }

    // If we are way, way below alpha (generally an amount that is considered unsalvageable)
    // just stop here.  You're not going to make back up being 25 pawns below alpha in the
    // remaining 8 or 9 ply of the search.
    if depth < AFP_DEPTH && !is_pv & !is_check {
        if (eval + afp_margin(depth)) <= alpha {
            return eval;
        }
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

        sse.searching_null_move = true;
        node.do_null_move();
        let val = -search(node, -beta, -beta + 1, depth - 1 - r, ply + 1, false, thread_num);
        node.undo_null_move();
        sse.searching_null_move = false;

        if val >= beta {
            return beta;
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
    let mut sing_extend = false;
    if !init_node
        && depth >= 8
        && sse.tt_hit
        && sse.excluded_move.is_null
        && sse.tt_val.abs() < MATE_SCORE - 100000 // TODO give this a name
        && (sse.tt_node_type & CUT_NODE) != 0
        && sse.tt_depth >= depth - 3
    {
        // I've stolen stockfish's idea here to combine singular
        // move detection with multi-cut in the same search
        let former_pv = sse.tt_node_type == PV_NODE && !is_pv;
        let margin = if former_pv {30 * (depth / 2)} else {25 * (depth / 2)};
        let depth_to_search = if former_pv {(depth + 2) / 2} else {(depth - 1) / 2};
        let target = sse.tt_val - margin;

        sse.excluded_move = sse.tt_move;
        let val = search(node, target - 1, target, depth_to_search, ply, false, thread_num);
        sse.excluded_move = Move::null_move();

        if val < target {
            sing_extend = true;
        } else if target >= beta {
            // pseudo-multi-cut.  This indicates multiple moves failed high
            // so this is probably a cutnode
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
    let countermove = if prev_mv.is_null {
        Move::null_move()
    } else {
        let piece_num = get_piece_num(prev_mv.piece, !node.side_to_move);
        ti.countermove_table[piece_num][prev_mv.end as usize]
    };

    // Followup History
    // Similarly to the CM heuristic above, moves by you may
    // have a natural follow-up in executing a plan.  Here instead
    // of a specific move, we hold on to a full history table for each move
    let followup_table = if my_prev_mv.is_null {
        [[0; 64]; 12]
    } else {
        let piece_num = get_piece_num(my_prev_mv.piece, node.side_to_move);
        ti.followup_history[piece_num][my_prev_mv.end as usize]
    };

    let mut movepicker = MovePicker::new(sse.tt_move, ti.killers[ply as usize], ti.move_history, countermove, followup_table, false);

    // Futility Pruning.  The 'futile' flag signals
    // to skip quiet moves.  Different conditions in the search
    // loop can activate this flag to prune all subsequent quiet moves
    // from then on.
    let mut futile = false;

    let mut found_legal_move = false;
    let mut searched_moves: Vec<Move> = Vec::new();
    loop {
        let (mv, score) = movepicker.next(node);
        sse.current_move = mv;
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
            // Illegal move
            node.undo_move(&mv);
            continue;
        }
        let gives_check = node.is_check(node.side_to_move);

        // Basic form of late move pruning
        if best_val > -MIN_MATE_SCORE && depth <= 8 && moves_searched >= lmp_count(improving, depth) {
            futile = true;
        }

        if is_quiet && !futile && score < COUNTER_OFFSET && best_val > - MIN_MATE_SCORE {
            // Futility/History leaf pruning
            // modified from the older version to be able to come into play at higher
            // depths but conditioned on reaching a move with bad history
            let fp_margin = fp_margin(depth);
            let hist = if score >= QUIET_OFFSET {(score - QUIET_OFFSET) as i32} else {-((QUIET_OFFSET - score) as i32)};
            if !is_check
                && eval + fp_margin <= alpha
                && depth < FP_DEPTH
                && hist < if improving {5000} else {9000} {
                    futile = true;
                }
        }

        if futile {
            found_legal_move = true;
            if !is_tactical && !gives_check {
                node.undo_move(&mv);
                continue
            }
        }

        found_legal_move = true;
        if is_quiet {
            searched_moves.push(mv);
        }
        let mut val = LB;
        if moves_searched == 0 {
            val = -search(node, -beta, -alpha, if sing_extend {depth} else {depth - 1}, ply + 1, is_pv, thread_num);
            unsafe {
                let child_ss = &mut SS[thread_num][(ply + 1) as usize];
                sse.pv = vec![mv];
                sse.pv.append(&mut child_ss.pv);
            }
        } else {
            let mut do_full_zw_search = true;
            if depth > LMR_DEPTH
                && !init_node
                && moves_searched >= 2
                && !is_check
                && (is_quiet || score < QUIET_OFFSET) {
                    // Late Move Reductions.
                    // the ideas here are a mish-mash of various ideas that are
                    // constantly being added, removed, and played around with
                    // some are ideas I've come up with on my own
                    // some are taken from SF, Ethereal, and others and modified slightly
                    // to see if they are compatible with Mantissa's search.
                    do_full_zw_search = false;

                    // get base reduction
                    let mut r = lmr_reduction(depth, moves_searched);

                    // this is a common idea in several strong engines
                    // if we're not improving our position, we're often not going
                    // to get better, so we reduce further.
                    if !improving { r += 1; }

                    // This one is pretty clear.  Killer/Counter moves are
                    // likely to be good.
                    if score >= COUNTER_OFFSET { r -= 1; }

                    // adjust r based on history of other quiet moves
                    if is_quiet && score < COUNTER_OFFSET {
                        let quiet_score = (score as i32) - QUIET_OFFSET as i32;
                        r -= cmp::max(-3, cmp::min(2, quiet_score / 4000))
                    }

                    // the tt node type here gives us some impression on if we
                    // expect this to be a PV node.  Reduce more or less according to
                    // that expectation.
                    if sse.tt_hit && sse.tt_node_type == PV_NODE { r -= 1; }
                    if sse.tt_hit && sse.tt_node_type != PV_NODE { r += 1; }

                    // in potential PV nodes, we'll be more careful
                    if is_pv && r > 0 {
                        r = (r * 2) / 3;
                    }

                    // not (yet) allowing extensions from LMR.
                    // we also don't want to dump directly into qsearch.
                    let lmr_depth = cmp::min(cmp::max(1, depth - r), depth - 1);

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
                if is_quiet {
                    // update heuristics
                    ti.update_killers(mv, ply);
                    ti.update_move_history(mv, node.side_to_move, depth, &searched_moves);
                    ti.update_countermove(prev_mv, mv, !node.side_to_move);
                    ti.update_followup(my_prev_mv, mv, node.side_to_move, depth, &searched_moves);
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

    if node.is_quiet() { return static_eval(node, &mut ti.pht); }

    let mut alpha = alpha;

    let stand_pat = static_eval(node, &mut ti.pht);

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

        if score <= OK_CAPTURE_OFFSET {
            // see if this is a viable capture
            let cap_piece = node.get_last_capture();
            if cap_piece != 0 {
                node.undo_move(&mv);
                let see_score = see(node, mv.end, cap_piece, mv.start, mv.piece);
                if see_score <= cmp::max(0, alpha - stand_pat) {
                    continue;
                } else {
                    node.do_move(&mv);
                }
            }
        }
        let val = -qsearch(node, -beta, -alpha, thread_num);
        node.undo_move(&mv);
        if val > best_val {
            best_val = val;
        }
        if val > alpha {
            alpha = val;
        }
        if val >= beta {
            break;
        }
    }

    return best_val;
}
