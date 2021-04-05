use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::SystemTime;

pub mod bb;
use bb::BB;
use bb::Mv;

pub mod tt;
use tt::TT;

pub mod pht;
use pht::PHT;

static mut EVALED: u64 = 0;
static mut HITS: u64 = 0;

const LB: i32 = -10000000;
const UB: i32 = 10000000;

const PV_NODE: u8 = 1;
const CUT_NODE: u8 = 2;
const ALL_NODE: u8 = 3;

pub fn get_time_millis() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime failed!"),
    }
}

static mut tt: TT = TT {tt: Vec::new(), bits: 0, mask: 0, valid: false};
static mut pht: PHT = PHT {pht: Vec::new(), bits: 0, mask: 0, valid: false};

pub unsafe fn print_pv(node: & mut BB, depth: i32) {
    if depth < 0 {
        eprint!("\n");
        return;
    }
    let tt_entry = tt.get(node.hash);
    if tt_entry.valid && !tt_entry.mv.is_null {
        eprint!("{} ", tt_entry.mv);
        node.do_move(&tt_entry.mv);
        print_pv(node, depth-1);
        node.undo_move(&tt_entry.mv);
    } else {
        eprint!("\n");
    }
}

pub unsafe fn best_move(node: &mut BB, maximize: bool, compute_time: u128) -> (Mv, f64) {
    EVALED = 0;
    HITS = 0;

    let mut m_depth: i32 = 1;
    let start_time: u128 = get_time_millis();
    let mut current_time: u128 = start_time;

    // initialize some values
    let mut best_move : Mv = Mv::null_move();
    let mut best_val: i32 = 0;

    // initialize tables
    if !tt.valid {
        tt = TT::get_tt(24);
    }
    if !pht.valid {
        pht = PHT::get_pht(22);
    }
    let mut k_table: [[Mv; 3]; 64] = [[Mv::null_move(); 3]; 64];

    let mut aspire = false;
    while (current_time - start_time) <= compute_time {
        let mut alpha = LB;
        let mut beta = UB;

        if aspire {
            if maximize {
                alpha = best_val - 250;
                beta = best_val + 250;
            } else {
                alpha = -best_val - 250;
                beta = -best_val + 250;
            }
        }
        let (ply_move, ply_val, node_type) = negamax_search(
            node,
            start_time,
            if best_move.is_null {100000} else {compute_time},
            m_depth,
            0,
            alpha,
            beta,
            maximize,
            true,
            true,
            true,
            &mut k_table
        );

        // kind of a misnomer, this means we ran out of time
        if ply_move.is_err {break;}
        // we ended up outside of our aspiration window
        // if ply_move.is_null {aspire = false; continue;}
        // if node_type == CUT_NODE {

        // }
        if ply_move.is_null || node_type != PV_NODE {aspire = false; continue;}

        // we maintain that positive is always white advantage here so we have to flip
        (best_move, best_val) = (ply_move, if node.white_turn {ply_val} else {-ply_val});

        // for debug
        eprintln!("{} eval'd {} this time got val {} and recommends {} with piece {}", m_depth, EVALED, best_val, best_move, best_move.piece);

        if best_val.abs() == 1000000 {
            // checkmate
            break;
        }

        current_time = get_time_millis();
        aspire = true;
        m_depth += 1;
    }

    eprintln!("{} tt hits", HITS);
    eprintln!("{} ply evaluated", m_depth - 1);
    eprintln!("{} nodes evaluated", EVALED);
    eprintln!("projected value: {}", best_val);
    eprintln!("elapsed time: {}", get_time_millis() - start_time);
    eprintln!("expected PV:");
    print_pv(node, m_depth - 1);

    // update info for spectators, &c
    println!("info depth {} score cp {}", m_depth - 1, (best_val / 10) * if maximize {1} else {-1});
    return (best_move, best_val as f64 / 1000.);
}

fn is_terminal(node: &BB) -> bool {
    // TODO better terminal
    return node.king[0] == 0 || node.king[1] == 0 || node.is_threefold();
}

fn is_move_tactical(node: &BB, mv: &Mv) -> bool {
    if mv.piece == b'p' {
        if node.white_turn && mv.end >= 36 {
            return true;
        }
        if !node.white_turn && mv.end < 24 {
            return true;
        }
    }
    return false;
}

unsafe fn evaluate_position(node: &BB) -> i32 {
    EVALED += 1;
    let mut val = 0;

    // pawn values
    let pht_entry = pht.get(node.pawn_hash);
    if pht_entry.valid {
        val += pht_entry.val;
    } else {
        let pp = node.passed_pawns_value();
        let cp = node.center_value();
        let ncp = node.near_center_value();
        let ip = node.isolated_pawns_value();
        let dp = node.doubled_pawns_value();
        let bp = node.backwards_pawns_value();
        let pv = pp + ip + dp + bp + cp + ncp;
        val += pv;
        pht.set(node.pawn_hash, pv);
    }
    val += node.material;
    val += node.mobility_value();
    val += node.pawn_defense_value();
    val += node.double_bishop_bonus();
    val += node.castled_bonus();
    val += node.pawn_advancement_value();
    val += node.get_all_pt_bonus();
    val += node.rook_on_seventh_bonus();
    val += node.rook_on_open_file_value();

    // slight tempo bonus
    val += node.tempo_bonus();

    val += node.early_queen_penalty();
    val += node.king_danger_value();
    val += node.material_advantage_bonus();

    return val * if node.white_turn {1} else {-1};
}

unsafe fn old_evaluate_position(node: &BB) -> i32 {
    EVALED += 1;
    let mut val = 0;

    // pawn values
    let pht_entry = pht.get(node.pawn_hash);
    if pht_entry.valid {
        val += pht_entry.val;
    } else {
        let pp = node.passed_pawns_value() * 500;
        let cp = node.center_value() * 300;
        let ncp = node.near_center_value() * 50;
        let ip = node.isolated_pawns_value() * -300;
        let dp = node.doubled_pawns_value() * -300;
        let bp = node.backwards_pawns_value() * -300;
        let pv = pp + ip + dp + bp + cp + ncp;
        val += pv;
        pht.set(node.pawn_hash, pv);
    }
    val += node.material;
    val += node.mobility_value() * 70;//(bb::PAWN_VALUE / 10);
    val += node.pawn_defense_value() * 50;
    val += node.double_bishop_bonus() * 500;
    val += node.castled_bonus() * (500 * (256 - node.get_phase()) / 256); // should decay as game advances
    val += node.pawn_advancement_value() * 40;
    val += node.get_all_pt_bonus();
    val += node.rook_on_seventh_bonus() * 150;
    val += node.rook_on_open_file_value() * 60;

    // slight tempo bonus
    val += if node.white_turn {150} else {-150};

    val -= node.early_queen_penalty() * 300;
    val -= node.king_danger_value();

    if node.material > 500 {val += node.phase * 2;} else if node.material < -500 {val -= node.phase * 2;}

    return val * if node.white_turn {1} else {-1};
}

pub unsafe fn print_evaluate(node: &BB) {
    eprintln!("Material: {}", node.material);
    eprintln!("Mobility: {}", node.mobility_value());
    eprintln!("doubled p: {}", node.doubled_pawns_value());
    eprintln!("isolated p: {}", node.isolated_pawns_value());
    eprintln!("backwards p: {}", node.backwards_pawns_value());
    eprintln!("passed p: {}", node.passed_pawns_value());
    eprintln!("Center: {}", node.center_value());
    eprintln!("Near Center: {}", node.near_center_value());
    eprintln!("Double bishop: {}", node.double_bishop_bonus());
    eprintln!("Pawn Defense: {}", node.pawn_defense_value());
    eprintln!("Pawn advancement: {}", node.pawn_advancement_value());
    eprintln!("Castle Bonus: {}", node.castled_bonus());
    eprintln!("Early queen penalty: {}", node.early_queen_penalty());
    eprintln!("All pt bonus: {}", node.get_all_pt_bonus());
    eprintln!("King danger value: {}", node.king_danger_value());
    eprintln!("Tempo: {}", node.tempo_bonus());
    eprintln!("Rook on 7th: {}", node.rook_on_seventh_bonus());
    eprintln!("Rook on (semi-)open file: {}", node.rook_on_open_file_value());
    eprintln!("Material lead bonus: {}", node.material_advantage_bonus());
    if pht.valid {
        let pht_entry = pht.get(node.pawn_hash);
        if pht_entry.valid {
            eprintln!("pawn entry: {}", pht_entry.val);
        }
    }
    eprintln!("Phase {} / 256", node.get_phase());
}

fn is_quiet(node: &mut BB) -> bool {
    let mut loud = false;
    match node.cap_stack.pop() {
        Some(cap) => {
            loud |= (cap != 0);
            node.cap_stack.push(cap);
        },
        None => {}
    }
    return !loud;
}

fn moves_equivalent(mv1: &Mv, mv2: &Mv) -> bool {
    return  mv1.start == mv2.start
            && mv1.end == mv2.end
            && mv1.piece == mv2.piece
            && mv1.promote_to == mv2.promote_to
            && mv1.ep_tile == mv2.ep_tile
            && mv1.is_ep == mv2.is_ep;
}

fn order_moves(mut moves: Vec<Mv>, best_move: Mv) -> Vec<Mv> {
    if best_move.is_null { return moves; }
    let mut found_move = false;
    let mut new_q: Vec<Mv> = Vec::new();
    new_q.push(best_move);
    for mv in moves {
        if moves_equivalent(&mv, &best_move) {
            found_move = true;
        } else {
            new_q.push(mv);
        }
    }
    if !found_move {
        // new_q.push_front(best_move);
        new_q.remove(0);
    }
    return new_q;
}

fn update_k_table(k_table: & mut [[Mv; 3]; 64], mv: Mv, ply: i32) {
    let ply = ply as usize;
    if moves_equivalent(&k_table[ply][0], &mv) {return;}
    if moves_equivalent(&k_table[ply][1], &mv) {return;}
    if moves_equivalent(&k_table[ply][2], &mv) {return;}

    if k_table[ply][0].is_null {k_table[ply][0] = mv; return;}
    if k_table[ply][1].is_null {k_table[ply][1] = mv; return;}
    if k_table[ply][2].is_null {k_table[ply][2] = mv; return;}

    // otherwise replace one.  Let's choose arbitrarily
    k_table[ply][(get_time_millis() % 3) as usize] = mv;
}

unsafe fn negamax_search(node: &mut BB,
                         start_time: u128,
                         compute_time: u128,
                         depth: i32,
                         ply: i32,
                         alpha: i32,
                         beta: i32,
                         maximize: bool,
                         nmr_ok: bool,
                         init: bool,
                         is_pv: bool,
                         k_table: & mut [[Mv; 3]; 64],
) -> (Mv, i32, u8) {
    let current_time = get_time_millis();
    if current_time - start_time > compute_time { return (Mv::err_move(), 0, PV_NODE); }

    if is_terminal(&node) {
        if node.is_threefold() {
            return (Mv::null_move(), 0, PV_NODE);
        }
        return (Mv::null_move(), evaluate_position(&node), PV_NODE);
    }

    let mut first_move = Mv::null_move();
    let mut depth = depth;
    let mut is_cut = false;

    let tt_entry = tt.get(node.hash);
    if tt_entry.valid {
        HITS += 1;
        let mv = tt_entry.mv;
        first_move = mv;
        is_cut = tt_entry.node_type == CUT_NODE && !init;
        // is_pv = tt_entry.node_type == PV_NODE;
        if tt_entry.depth >= depth {
            if tt_entry.node_type == PV_NODE
            // if ((tt_entry.value < alpha || tt_entry.value >= beta) && tt_entry.node_type == PV_NODE)
                || (tt_entry.value < alpha && tt_entry.node_type == ALL_NODE) // all node is upper bound
                || (tt_entry.value >= beta && tt_entry.node_type == CUT_NODE) // cut node is lower bound
            {
                let mut moves = order_moves(node.order_capture_moves(node.moves(), &k_table[ply as usize]), first_move);
                if moves.len() > 0 {
                    let fm = moves[0];
                    if moves_equivalent(&first_move, &fm) {
                        // check 3 fold
                        if !node.is_repitition() {
                            let node_type = if tt_entry.value > alpha {if tt_entry.value >= beta {CUT_NODE} else {PV_NODE}} else {ALL_NODE};
                            return (mv, if node_type ==  CUT_NODE {beta} else {tt_entry.value}, node_type);
                        }
                    } else {
                        // weird collision
                        first_move = Mv::null_move();
                    }
                } else {
                    first_move = Mv::null_move();
                }
            }
        }
    } else if depth > 6 {
        // internal iterative deepending
        let mut moves = order_moves(node.order_capture_moves(node.moves(), &k_table[ply as usize]), first_move);
        let mut best_val = LB;
        for mv in moves {
            node.do_move(&mv);
            if node.is_check(maximize) {
                node.undo_move(&mv);
                continue
            }
            let val = -negamax_search(node, start_time, compute_time, depth - 5, ply+1, -beta, -alpha, !maximize, true, false, false, k_table).1;
            node.undo_move(&mv);
            if val > best_val {
                best_val = val;
                first_move = mv;
            }
        }
    }

    if depth <= 0 {
        if is_quiet(node) {
            let val = evaluate_position(&node);
            let node_type = if val > alpha {if val >= beta {CUT_NODE} else {PV_NODE}} else {ALL_NODE};
            return (Mv::null_move(), val, node_type);
        } else {
            let (val, node_type) = quiescence_search(node, 16, alpha, beta, maximize);
            return (Mv::null_move(), val, node_type);
        }
    }

    let mut alpha = alpha;
    let beta = beta;

    let mut raised_alpha = false;
    let is_check = node.is_check(maximize);
    let mut best_move = Mv::null_move();
    let mut val = LB;

    let mut moves = order_moves(node.order_capture_moves(node.moves(), &k_table[ply as usize]), first_move);
    let mut num_moves = 0;

    if !is_check && nmr_ok && !init && !is_pv { // && evaluate_position(&node) >= beta {
        // null move reductions
        let depth_to_search = depth - if depth > 6 {5} else {4};
        node.do_null_move();
        let nmr_val = -negamax_search(node, start_time, compute_time, depth_to_search, ply+1, -beta, -beta + 1, !maximize, false, false, false, k_table).1;
        node.undo_null_move();
        if nmr_val >= beta {
            depth -= 4;
            if depth <= 0 {
                return negamax_search(node, start_time, compute_time, 0, ply, alpha, beta, maximize, false, false, false, k_table);
            }
        }
    }

    let mut tried = 0;
    let mut cuts = 0;
    let m = 12;
    let c = 3;
    let r = 3;
    let mut already_cut: [i32; 3] = [-1; 3];

    // multicut
    if is_cut && depth >= 3 {
        for mv in moves.iter() {
            node.do_move(&mv);
            let score = -negamax_search(node, start_time, compute_time, depth - 1 - r, ply+1, -beta, -beta + 1, !maximize, false, false, false, k_table).1;
            node.undo_move(&mv);
            if score >= beta {
                if already_cut[0] != mv.start && already_cut[1] != mv.start && already_cut[2] != mv.start {
                    already_cut[cuts] = mv.start;
                    cuts += 1;
                    if cuts >= c {
                        return (Mv::null_move(), beta, CUT_NODE);
                    }
                }
            }

            tried += 1;
            if tried >= m {
                break;
            }
        }
    }

    let mut is_futile = false;
    if !init && depth == 1 {
        let futile_val = evaluate_position(&node);
        if futile_val < (alpha - 3500) {
            val = futile_val;
        }
    }
    // let mut is_futile = !init && (depth == 1 && evaluate_position(&node) < (alpha - 3500)); // || (depth == 2 && evaluate_position(&node) < (alpha - 5500)));
    let mut legal_move = false;
    for mv in moves { // .drain(..) {
        let is_tactical_move = is_move_tactical(&node, &mv);
        node.do_move(&mv);

        if node.is_check(maximize) {
            node.undo_move(&mv);
            continue;
        } else if is_futile {
            legal_move = true;
            if is_quiet(node) &&
                !is_terminal(node) &&
                !node.is_check(node.white_turn) &&
                (mv.promote_to == 0) &&
                !is_tactical_move {
                    node.undo_move(&mv);
                    continue;
                }
        }

        legal_move = true;
        let mut res: (Mv, i32, u8) = (Mv::null_move(), LB, PV_NODE);

        let quiet = is_quiet(node) && mv.promote_to == 0 && !mv.is_ep;

        let mut reduced = false;
        if num_moves == 0 {
            res = negamax_search(node, start_time, compute_time, depth - 1, ply+1, -beta, -alpha, !maximize, true, false, true, k_table);
        } else {
            if depth > 3
                && !is_pv
                && num_moves > 4
                && !is_check
                && is_quiet(node)
                && !is_terminal(node)
                && !node.is_check(node.white_turn)
                && (mv.promote_to == 0)
                && !is_tactical_move {
                    let mut depth_to_search = depth - 2;
                    // if num_moves > 10 && !is_pv {
                    //     depth_to_search = depth - 1 - (depth / 3);
                    // }
                    res = negamax_search(node, start_time, compute_time, depth_to_search, ply + 1, -alpha - 1, -alpha, !maximize, true, false, false, k_table);
                    if -res.1 <= alpha {
                        reduced = true;
                    }
                } else {
                    reduced = false;
                }

            if !reduced {
                res = negamax_search(node, start_time, compute_time, depth - 1, ply + 1, -alpha - 1, -alpha, !maximize, true, false, false, k_table);
                if -res.1 > alpha && -res.1 < beta { // failed high
                    res = negamax_search(node, start_time, compute_time, depth - 1, ply + 1, -beta, -alpha, !maximize, true, false, true, k_table);
                }
            }

        }

        node.undo_move(&mv);
        num_moves += 1;
        let ret_move = res.0;
        if ret_move.is_err {
            return (Mv::err_move(), 0, PV_NODE);
        }
        let ret_val = -res.1;
        let ret_node_type = res.2;
        if ret_val > val {
            best_move = mv;
            val = ret_val;
        }
        if val > alpha {
            alpha = val;
            raised_alpha = true;
        }

        if alpha >= beta {
            // beta cutoff
            if quiet {
                update_k_table(k_table, mv, ply);
            }
            tt.set(node.hash, best_move, val, CUT_NODE, depth);
            return (best_move, beta, CUT_NODE);
        }
    }

    if best_move.is_null && !legal_move {
        // some sort of mate
        if is_check {
            return (best_move, -1000000, ALL_NODE);
        } else {
            return (best_move, 0, if alpha > 0 {ALL_NODE} else if beta < 0 {CUT_NODE} else {PV_NODE});
        }
    }

    tt.set(node.hash, best_move, val, if !raised_alpha {ALL_NODE} else {PV_NODE}, depth);
    return (best_move, val, if !raised_alpha {ALL_NODE} else {PV_NODE});
}

unsafe fn quiescence_search(node: &mut BB, depth: i32, alpha: i32, beta: i32, maximize: bool) -> (i32, u8) {
    if depth == 0 || is_terminal(node) || is_quiet(node) {
        return (evaluate_position(node), PV_NODE);
    }

    let mut alpha = alpha;
    let beta = beta;
    let mut raised_alpha = false;
    if !node.is_check(maximize) {
        let curr_val = evaluate_position(node);
        if curr_val >= beta {
            // beta cutoff
            return (beta, CUT_NODE);
        }
        if curr_val > alpha {
            raised_alpha = true;
            alpha = curr_val;
        } else {
            if node.get_phase() < 180 && curr_val < alpha - 9200 {
                return (curr_val, ALL_NODE);
            }
        }
    }

    let mut best_val = LB;
    let mut best_mv: Mv = Mv::null_move();
    for mv in node.moves() {
        node.do_move(&mv);
        let res = quiescence_search(node, depth - 1, -beta, -alpha, !maximize);
        node.undo_move(&mv);
        let val = -res.0;

        if val >= beta {
            return (beta, CUT_NODE);
        }
        if val >= best_val {
            best_val = val;
            best_mv = mv;
        }
        if val > alpha {
            raised_alpha = true;
            alpha = val;
        }
    }

    // if we didn't find further captures, we don't want to erroneously return the
    // min value, but we also need to be consistent with the fail-soft lower bound of non-QS
    let node_val = if !best_mv.is_null {best_val} else {evaluate_position(node)};
    return (node_val, if raised_alpha {PV_NODE} else {ALL_NODE});
}
