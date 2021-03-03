use std::cmp;
use std::collections::HashMap;
use std::time::SystemTime;

pub mod nodes;
pub mod bb;
use nodes::Node;
use nodes::Move;
use bb::BB;

static mut evaled: u64 = 0;

pub unsafe fn choose_move(node: &mut Node, maximize: bool, compute_time: u128, check_check: bool) -> (Move, f64) {
    let mut m_depth = 3;
    let mut q_depth = 4;

    let start_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis(),
        Err(_) => panic!("SystemTime failed!"),
    };
    let mut current_time = start_time;
    let mut best_move : Move = Move::null_move();
    let mut val = if maximize {-200000} else {200000};
    let mut z_table : HashMap<u64, (Move, i32, i32)> = HashMap::new();
    let mut aspire: bool = false;

    while (current_time - start_time) <= compute_time {
        evaled = 0;
        let mut alpha = -1000000;
        let mut beta = 1000000;
        if aspire {
            alpha = val - 250;
            beta = val + 250;
        }
        eprintln!("alpha {} beta {}", alpha, beta);
        let res = minimax_search(node, start_time, compute_time, m_depth, q_depth, alpha, beta, maximize, & mut z_table, check_check);
        if res.0.is_err {
            // timeout
            break;
        }
        if res.0.is_null {
            // null move
            aspire = false;
            continue;
        }
        if aspire && (res.1 <= alpha || res.1 >= beta) {
            eprintln!("aspiration violated");
            aspire = false;
            continue;
        }
        (best_move, val) = res;
        eprintln!("{} eval'd {} this time got val {}", m_depth, evaled, val);
        current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis(),
            Err(_) => panic!("SystemTime failed!"),
        };
        aspire = true;
        m_depth += 1;
    }
    eprintln!("{}", m_depth);
    eprintln!("eval'd {}", evaled);
    eprintln!("projected: {}", val);
    return (best_move, val as f64 / 1000.);
}

fn is_terminal(node: &Node) -> bool {
    return node.material.abs() >= 100000 || node.is_threefold();
}

unsafe fn evaluate_position(node: &Node) -> i32 {
    if node.is_threefold() {
        return 0;
    }
    evaled += 1;
    let mut val = node.material;
    let (mob, kdv) = node.mobility_value();
    val += mob / 10;
    val += node.center_value() / 3;
    val -= node.doubled_pawns_value() / 2;
    val -= node.isolated_pawns_value() / 2;
    val -= node.backwards_pawns_value() / 4;
    val -= kdv;
    val += node.piece_synergy_values();

    val += if node.white_turn {200} else {-200};
    return val;
}

fn is_quiet(node: &mut Node) -> bool {
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

unsafe fn minimax_search(node: &mut Node, start_time: u128, compute_time: u128, depth: i32, q_depth: i32, alpha: i32, beta: i32, maximize: bool, table: & mut HashMap<u64, (Move, i32, i32)>, check_check: bool) -> (Move, i32) {
    // check time
    let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis(),
            Err(_) => panic!("SystemTime failed!"),
    };
    if current_time - start_time > compute_time {return (Move::err_move(), 0)}
    let mut alpha = alpha;
    let mut beta = beta;
    let mut first_move = nodes::Move::null_move();
    // check cache
    match table.get(&node.get_hash()) {
        Some(p) => {
            if !p.0.is_null && !p.0.is_err {
                let mv = nodes::Move::copy_move(&p.0);
                if p.2 >= depth { return (mv, p.1) }
                else {first_move = mv}
            }
        }
        None => {}
    }

    if is_terminal(&node) {
        return (nodes::Move::null_move(), evaluate_position(&node));
    }

    if depth == 0 {
        if q_depth == 0 || is_quiet(node) {
            let val = evaluate_position(&node);
            return (nodes::Move::null_move(), val);
        } else {
            return (nodes::Move::null_move(), quiescence_search(node, q_depth, alpha, beta, maximize));
        }
    }

    let mut best_move = nodes::Move::null_move();
    let mut val = if maximize {-1000000} else {1000000};
    let mut moves_to_assess = node.moves();
    let mut best_move_guess = false;
    let mut first = false;
    if !first_move.is_null {
        if moves_to_assess.contains(&first_move) {
            best_move_guess = true;
            first = true;
            moves_to_assess.push_front(first_move);
        }
    }

    if moves_to_assess.len() == 0 {
        if node.is_check(maximize) {
            return (best_move, if maximize {node.material - 200000} else {node.material + 200000});
        }
        return (best_move, 0);
    }
    for pot_move in moves_to_assess.drain(0..) {
        node.do_move(&pot_move);
        let mut mv : Move;
        let mut new_val: i32;
        if !best_move_guess || first {
            (mv, new_val) = minimax_search(node, start_time, compute_time, depth - 1, q_depth, alpha, beta, !maximize, table, check_check);
        } else {
            if maximize {
                (mv, new_val) = minimax_search(node, start_time, compute_time, depth - 1, q_depth, alpha, alpha + 1, !maximize, table, check_check);
            } else {
                (mv, new_val) = minimax_search(node, start_time, compute_time, depth - 1, q_depth, beta - 1, beta, !maximize, table, check_check);
            }

            if !mv.is_err && (alpha < new_val && beta > new_val) {
                (mv, new_val) = minimax_search(node, start_time, compute_time, depth - 1, q_depth, alpha, beta, !maximize, table, check_check);
            }
        }
        node.undo_move(&pot_move);
        first = false;
        if mv.is_err { return (mv, 0) }
        if maximize {
            if new_val > val || best_move.is_null {
                if val.abs() >= 100000 || check_check {
                    node.do_move(&pot_move);
                    if node.is_check(maximize) {
                        node.undo_move(&pot_move);
                        continue;
                    }
                    node.undo_move(&pot_move);
                }
                best_move = pot_move;
                val = new_val;
            }
            alpha = cmp::max(alpha, val);
        } else {
            if new_val < val || best_move.is_null {
                if val.abs() >= 100000 || check_check {
                    node.do_move(&pot_move);
                    if node.is_check(maximize) {
                        node.undo_move(&pot_move);
                        continue;
                    }
                    node.undo_move(&pot_move);
                }
                best_move = pot_move;
                val = new_val;
            }
            beta = cmp::min(beta, val);
        }
        if alpha >= beta {
            if maximize {
                return (best_move, beta);
            } else {
                return (best_move, alpha);
            }
        }
    }
    if best_move.is_null {
        if node.is_check(maximize) {
            return (best_move, if maximize {node.material - 200000} else {node.material + 200000});
        }
        return (best_move, 0);
    }

    table.insert(node.get_hash(), (nodes::Move::copy_move(&best_move), val, depth));
    return (best_move, val);
}

unsafe fn quiescence_search(node: &mut Node, depth: i32, alpha: i32, beta: i32, maximize: bool) -> i32 {
    if depth == 0 || is_terminal(node) || is_quiet(node) {
        return evaluate_position(node);
    }

    let mut alpha = alpha;
    let mut beta = beta;
    let standing_pat = evaluate_position(node);

    if maximize {
        if standing_pat >= beta {
            return beta;
        }
        if standing_pat > alpha {
            alpha = standing_pat;
        }
    } else {
        if standing_pat <= alpha {
            return alpha;
        }
        if standing_pat < beta {
            beta = standing_pat;
        }
    }

    for pot_move in node.moves().drain(0..) {
        node.do_move(&pot_move);
        let val = quiescence_search(node, depth - 1, alpha, beta, !maximize);
        node.undo_move(&pot_move);
        if maximize {
            if val >= beta { return beta; }
            if val > alpha { alpha = val; }
        } else {
            if val <= alpha { return alpha; }
            if val < beta { beta = val; }
        }
    }
    return if maximize {alpha} else {beta};
}
