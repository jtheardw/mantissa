use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::SystemTime;

pub mod bb;
use bb::BB;
use bb::Mv;

pub mod tt;
use tt::TT;
use tt::TTEntry;

pub mod pht;
use pht::PHT;
use pht::PHTEntry;

static mut evaled: u64 = 0;
static mut hits: u64 = 0;
static mut eval_hits: u64 = 0;
static mut pawn_hits: u64 = 0;

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

pub unsafe fn best_move(node: &mut BB, maximize: bool, compute_time: u128) -> (Mv, f64) {
    let mut m_depth = 3;
    let mut q_depth = 6;
    evaled = 0;
    hits = 0;
    pawn_hits = 0;
    let start_time = get_time_millis();
    let mut current_time = start_time;
    let mut best_move : Mv = Mv::null_move();
    let mut best_val = 0;
    let mut z_table: HashMap<u64, (Mv, i32, i32)> = HashMap::new();
    if !tt.valid {
        tt = TT::get_tt(24);
    } else {
        eprintln!("tt saved");
    }
    if !pht.valid {
        pht = PHT::get_pht(22);
    } else {
        eprintln!("pht saved");
    }
    let mut k_table: [[Mv; 3]; 64] = [[Mv::null_move(); 3]; 64];
    let mut aspire = false;

    while (current_time - start_time) <= compute_time {
        let mut alpha = LB;
        let mut beta = UB;

        if aspire {
            alpha = best_val - 250;
            beta = best_val + 250;
        }
        let (ply_move, ply_val, node_type) = negamax_search(
            node,
            start_time,
            compute_time,
            m_depth,
            0,
            alpha,
            beta,
            maximize,
            true,
            true,
            &mut z_table,
            &mut k_table
        );

        if ply_move.is_err {break;}
        if ply_move.is_null || node_type != PV_NODE {aspire = false; continue;}

        (best_move, best_val) = (ply_move, if node.white_turn {ply_val} else {-ply_val});
        eprintln!("{} eval'd {} this time got val {} and recommends {} with piece {}", m_depth, evaled, best_val, best_move, best_move.piece);
        if best_val.abs() == 1000000 {
            break;
        }
        current_time = get_time_millis();
        aspire = true;
        m_depth += 1;
    }
    eprintln!("{} tt hits", hits);
    eprintln!("{} pht hits", pawn_hits);
    eprintln!("{} ply evaluated", m_depth - 1);
    eprintln!("{} nodes evaluated", evaled);
    eprintln!("projected value: {}", best_val);
    eprintln!("elapsed time: {}", current_time - start_time);
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
    evaled += 1;
    let mut val = 0;

    // pawn values
    let pht_entry = pht.get(node.pawn_hash);
    if pht_entry.valid {
        val += pht_entry.val;
        pawn_hits += 1;
    } else {
        let pp = node.passed_pawns_value() * 500;
        let cp = node.center_value() * 500;
        let ncp = node.near_center_value() * 50;
        let ip = node.isolated_pawns_value() * -300;
        let dp = node.doubled_pawns_value() * -400;
        let bp = node.backwards_pawns_value() * -100;
        let pv = pp + ip + dp + bp + cp + ncp;
        val += pv;
        pht.set(node.pawn_hash, pv);
    }
    val += node.material;
    val += node.mobility_value() * 70;//(bb::PAWN_VALUE / 10);
    val += node.pawn_defense_value() * 100;
    val += node.double_bishop_bonus() * 500;
    val += node.castled_bonus() * 500;
    // val += node.pawn_advancement_value() * 20;
    val += node.get_all_pt_bonus();
    // val += node.passed_pawns_value() * 400;

    // val -= node.doubled_pawns_value() * 400;
    // val -= node.isolated_pawns_value() * 300;
    // val -= node.backwards_pawns_value() * 100;
    val -= node.early_queen_penalty() * 300;
    val -= node.king_danger_value();

    return val * if node.white_turn {1} else {-1};
}

pub unsafe fn print_evaluate(node: &BB) {
    eprintln!("Material: {}", node.material);
    eprintln!("Mobility: {}", node.mobility_value() * 70);
    eprintln!("doubled p: {}", node.doubled_pawns_value() * -400);
    eprintln!("isolated p: {}", node.isolated_pawns_value() * -300);
    eprintln!("backwards p: {}", node.backwards_pawns_value() * -100);
    eprintln!("passed p: {}", node.passed_pawns_value() * 400);
    eprintln!("Center: {}", node.center_value() * 500);
    eprintln!("Near Center: {}", node.near_center_value() * 50);
    eprintln!("Double bishop: {}", node.double_bishop_bonus() * 500);
    eprintln!("Pawn Defense: {}", node.pawn_defense_value() * 100);
    eprintln!("Castle Bonus: {}", node.castled_bonus() * 500);
    eprintln!("Early queen penalty: {}", node.early_queen_penalty() * -300);
    eprintln!("All pt bonus: {}", node.get_all_pt_bonus());
    eprintln!("King danger value: {}", -node.king_danger_value());
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
    return (mv1.start == mv2.start && mv1.end == mv2.end && mv1.piece == mv2.piece && mv1.promote_to == mv2.promote_to && mv1.ep_tile == mv2.ep_tile && mv1.is_ep == mv2.is_ep);
}

fn order_moves(mut moves: VecDeque<Mv>, best_move: Mv) -> VecDeque<Mv> {
    if best_move.is_null { return moves; }
    let mut found_move = false;
    let mut new_q: VecDeque<Mv> = VecDeque::new();

    for mv in moves.drain(0..) {
        if moves_equivalent(&mv, &best_move) {
            found_move = true;
        } else {
            new_q.push_back(mv);
        }
    }
    if found_move {
        new_q.push_front(best_move);
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
    k_table[ply][(mv.start % 3) as usize] = mv;
}

unsafe fn negamax_search(node: &mut BB, start_time: u128, compute_time: u128, depth: i32,
                         ply: i32, alpha: i32, beta: i32, maximize: bool, nmr_ok: bool, init: bool,
                         z_table: & mut HashMap<u64, (Mv, i32, i32)>,
                         k_table: & mut [[Mv; 3]; 64],
) -> (Mv, i32, u8) {
    let current_time = get_time_millis();
    if current_time - start_time > compute_time { return (Mv::err_move(), 0, PV_NODE); }

    let mut first_move = Mv::null_move();
    let mut depth = depth;

    // match z_table.get(&node.hash) {
    //     Some(p) => {
    //         hits += 1;
    //         let mv = p.0;
    //         if p.2 >= depth { return (mv, p.1, PV_NODE); }
    //         else {first_move = mv;}
    //     },
    //     None => {}
    // }

    let tt_entry = tt.get(node.hash);
    if tt_entry.valid {
        hits += 1;
        let mv = tt_entry.mv;
        first_move = mv;
        if tt_entry.depth > depth && tt_entry.node_type == PV_NODE {
            let mut moves = order_moves(node.order_capture_moves(node.moves(), &k_table[ply as usize]), first_move);
            match moves.pop_front() {
                Some(fm) => {
                    if moves_equivalent(&first_move, &fm) {
                        return (mv, tt_entry.value, tt_entry.node_type);
                    } else {
                        // weird collision
                        first_move = Mv::null_move();
                    }
                },
                None => {first_move = Mv::null_move();}
            };

        }
    } else if depth > 6 {
        let mut moves = order_moves(node.order_capture_moves(node.moves(), &k_table[ply as usize]), first_move);
        let mut best_val = -10000000;
        for mv in moves.drain(0..) {
            node.do_move(&mv);
            if node.is_check(maximize) {
                node.undo_move(&mv);
                continue
            }
            let val = -negamax_search(node, start_time, compute_time, depth - 4, ply+1, -beta, -alpha, !maximize, true, false, z_table, k_table).1;
            node.undo_move(&mv);
            if val > best_val {
                best_val = val;
                first_move = mv;
            }
        }
    }

    let mut alpha = alpha;
    let mut beta = beta;

    if is_terminal(&node) {
        if node.is_threefold() {
            return (Mv::null_move(), 0, PV_NODE);
        }
        return (Mv::null_move(), evaluate_position(&node), PV_NODE);
    }

    if depth <= 0 {
        if is_quiet(node) {
            let val = evaluate_position(&node);
            return (Mv::null_move(), val, PV_NODE);
        } else {
            let (val, node_type) = quiescence_search(node, 8, alpha, beta, maximize);
            return (Mv::null_move(), val, node_type);
        }
    }

    let mut raised_alpha = false;
    let mut is_check = node.is_check(maximize);
    let mut best_move = Mv::null_move();
    let mut val = LB;

    let mut moves = order_moves(node.order_capture_moves(node.moves(), &k_table[ply as usize]), first_move);
    let mut num_moves = 0;
    let mut search_pv = true;

    if !is_check && nmr_ok {
        let depth_to_search = depth - if depth > 6 {5} else {4};
        node.do_null_move();
        let nmr_val = -negamax_search(node, start_time, compute_time, depth_to_search, ply+1, -beta, -beta + 1, !maximize, false, false, z_table, k_table).1;
        node.undo_null_move();
        if nmr_val >= beta {
            depth -= 4;
        }
    }

    let is_futile = depth == 1 && evaluate_position(&node) < (alpha - 3500);
    let mut legal_move = false;
    for mv in moves.drain(0..) {
        let is_tactical_move = is_move_tactical(&node, &mv);
        node.do_move(&mv);

        if node.is_check(maximize) {
        // if (init || is_check) && node.is_check(maximize) {
            // skip, illegal
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
        let mut res: (Mv, i32, u8);
        let mut depth_to_search = depth - 1;
        if depth > 3 && num_moves > 4 {
            if !is_check &&
                is_quiet(node) &&
                !is_terminal(node) &&
                !node.is_check(node.white_turn) &&
                (mv.promote_to == 0) &&
                !is_tactical_move {
                    depth_to_search = depth - 2;
                }
        }

        let quiet = is_quiet(node) && mv.promote_to == 0 && !mv.is_ep;
        if true && num_moves > 0 && !first_move.is_null {
            // pvs search bit
            res = negamax_search(node, start_time, compute_time, depth_to_search, ply+1, -alpha - 1, -alpha, !maximize, true, false, z_table, k_table);
            if -res.1 > alpha && -res.1 < beta { // failed high
                res = negamax_search(node, start_time, compute_time, depth_to_search, ply+1, -beta, -alpha, !maximize, true, false, z_table, k_table);
            }
        } else {
            res = negamax_search(node, start_time, compute_time, depth_to_search, ply+1, -beta, -alpha, !maximize, true, false, z_table, k_table);
        }
        if -res.1 >= beta && depth_to_search == depth - 2 {
            depth_to_search = depth - 1;
            res = negamax_search(node, start_time, compute_time, depth_to_search, ply+1, -beta, -alpha, !maximize, true, false, z_table, k_table);
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
            // search_pv = false;
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

    // if true || raised_alpha {
        // z_table.insert(node.hash, (best_move, val, depth));
    tt.set(node.hash, best_move, val, if !raised_alpha {ALL_NODE} else {PV_NODE}, depth);
    // }
    return (best_move, val, if !raised_alpha {ALL_NODE} else {PV_NODE});
}

unsafe fn quiescence_search(node: &mut BB, depth: i32, alpha: i32, beta: i32, maximize: bool) -> (i32, u8) {
    if depth == 0 || is_terminal(node) || is_quiet(node) {
        return (evaluate_position(node), PV_NODE);
    }

    let mut alpha = alpha;
    let mut beta = beta;
    let mut raised_alpha = false;
    let curr_val = evaluate_position(node);
    if curr_val >= beta {
        // beta cutoff
        return (beta, CUT_NODE);
    }
    if curr_val > alpha {
        raised_alpha = true;
        alpha = curr_val;
    }

    for mv in node.moves().drain(0..) {
        node.do_move(&mv);
        let res = quiescence_search(node, depth - 1, -beta, -alpha, !maximize);
        node.undo_move(&mv);
        let val = -res.0;

        if val >= beta {
            return (beta, CUT_NODE);
        }
        if val > alpha {
            raised_alpha = true;
            alpha = val;
        }
    }

    return (alpha, if raised_alpha {PV_NODE} else {ALL_NODE});
}
