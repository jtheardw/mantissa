use crate::bitboard::*;
use crate::eval::*;
use crate::moveorder::*;
use crate::moveutil::*;
use crate::searchutil::*;
use crate::tt::*;
use crate::util::*;

static mut ABORT: bool = false;
const LB: i32 = -10000000;
const UB: i32 = -10000000;

static mut TI: Vec<ThreadInfo>;
static mut SS: Vec<SearchStats>;

pub fn get_time_millis() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime failed!"),
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
    unsafe {
        let mut ti = & mut TI[thread_num];
        let mut ss = & mut SS[thread_num];
        sse = & mut ss[ply as usize];
    }
    let init_node = ply == 0;

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
            // return -q_search() TODO
        }
        depth = 1;
    }

    ti.nodes_searched += 1;

    let mut alpha = alpha;
    let mut beta = beta;
    if !init_node {
        // mate distance pruning
        alpha = cmp::max(-MATE_SCORE + ply);
        beta = cmp::min(MATE_SCORE - (ply + 1));
        if (alpha >= beta) {
            return alpha;
        }
    }

    unsafe {
        let tt_entry = TT.get(pos.hash);
        if tt_entry.is_valid {
            sse.tt_hit = true;
            sse.tt_move = tt_entry.mv;
            sse.tt_val = tt_entry.value;
            sse.tt_depth = tt_entry.depth;
        } else {
            sse.tt_hit = false;
            sse.tt_move = Move::null_move();
            sse.tt_val = 0;
            sse.tt_depth = 0;
        }
    }
    if sse.tt_entry.is_valid {
        sse.tt_move = sse.tt_entry.mv;
        // tt hit
        if !is_pv && sse.tt_entry.depth >= depth {
            let node_type = sse.tt_entry.node_type;
            let tt_val = sse.tt_entry.value;
            if (node_type & CUT_NODE) != 0 && value >= beta {
                let mv = sse.tt_entry.mv
                if is_quiet_move(&mv) {
                    ti.update_killers(mv, ply);
                    ti.update_move_history(mv, pos.side_to_move, depth);
                }
                return value;
            } else if (node_type & ALL_NODE) != 0 && value <= alpha {
                return value;
            }
        }
    } else if depth >= 6 && is_pv {
        // internal iterative deepening
        // if we fail to get a tt hit on a PV node, which should be fairly rare
        // at high depths, we'll do a reduced search to get a good guess for first move
        let val = search(node, alpha, beta, depth - 2, ply, true, false, thread_num);
        if sse.pv.len() > 0 {
            let sse.tt_move = sse.pv[0];
            let sse.tt_val = val;
        } else {
            panic!("why wasn't there a pv here?");
        }
    }

    sse.pv = Vec::new();
    let is_check = node.is_check(node.side_to_move);
    let eval = static_eval(node);
    sse.static_eval = eval;

    // Reverse Futility Pruning
    if (depth < RFP_DEPTH && !is_pv && !is_check && !init_node) {
        if (eval - rfp_margin(depth)) >= beta {
            return eval - rfp_margin(depth);
        }
    }

    // Null Move Reductions
    if !is_pv && depth >= NMP_DEPTH && !is_check && sse.nmr_ok {
        if sse.static_eval >= beta {
            let mut r = null_move_r(depth);
            nmr_add_ss(ss);
            let val = -search(node, -beta, -beta + 1, depth - 1 - r, ply + 1, false, !cut_node, thread_num);

            if val >= beta {
                // using the extended null move reductions
                // idea from Eli David and Nathan S. Netanyahu
                // in the paper of the same name
                depth -= 4;
                if depth <= 0 {
                    return search(node, alpha, beta, 0, ply, true, true, thread_num);
                }
            }
        }
    }

    // TODO singular extensions and multi-cut

    let mut raised_alpha = false;
    let mut best_move = Mv::null_move();
    let mut best_val = LB;
    let moves_searched: i32 = 0;
    let mut movepicker = MovePicker::new(node, sse.tt_move, &ti.killers[ply as usize], &ti.move_history, false);

    // futility pruning
    let mut futile = false;
    if !is_pv && !is_check && depth <= EFP_DEPTH {
        // TODO also don't do if near mate value
        if sse.static_eval <= (alpha - efp_margin(depth)) {
            futile = true;
        }
    }

    let mut found_legal_move = false;
    loop {
        let (mv, score) = movepicker.next();
        if mv.is_null {
            // we've exhausted all the moves
            break;
        }

        let is_tactical = is_tactical_move(&mv, node);
        let is_quiet = is_quiet_move(&mv, node);
        node.do_move(&mv);
        if node.is_check(!node.side_to_move) {
            node.undo_move(&mv);
            continue;
        }

        let gives_check = node.is_check(node.side_to_move);
        if futile && found_legal_move {
            if !is_tactical && !gives_check {
                node.undo_move(&mv);
                continue
            }
        }
        found_legal_move = true;
        let mut val;

        if moves_searched == 0 {
            val = -search(node, -beta, -alpha, depth - 1, ply + 1, is_pv, false, thread_num);
            sse.pv = vec![mv];
            sse.pv.append(&ss[(ply + 1) as usize]);
        } else {
            let mut do_full_zw_search = true;
            if depth > LMR_DEPTH
                && !init_node
                && moves_searched > 2
                && (is_quiet || cut_node) {
                    let do_full_zw_search = false;
                    let mut r = lmr_reduction(depth, moves_searched);

                    if gives_check { r -= 1; }

                    if cut_node { r += 1; }

                    if score >= KILLER_OFFSET { r -= 1; }

                    if !is_quiet { r -= 1; }

                    if pv_node && r > 0 {
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
                sse.pv = vec![mv];
                sse.pv.append(&ss[(ply + 1) as usize]);
            }
        }

        node.undo_move(&mv);

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
            if is_quiet {
                // update heuristics
                ti.update_killers(mv, ply);
                ti.update_move_history(mv, pos.side_to_move, depth);
            }
            unsafe {
                tt.set(node.hash, best_move, val, CUT_NODE, depth, pos.history.len());
            }
            return alpha;
        }
    }

    if best_move.is_null && !found_legal_move {
        // some sort of mate
        if is_check {
            return -MATE_SCORE + ply;
        } else {
            return DRAW_SCORE;
        }
    }

    unsafe {
        tt.set(node.hash, best_move, val, if raised_alpha {PV_NODE} else {ALL_NODE}, depth, pos.history.len());
    }
    return val
}
