use std::cmp;
use std::time::SystemTime;

pub mod bb;
use bb::BB;
use bb::Mv;

static mut evaled: u64 = 0;

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

pub unsafe fn best_move(node: &mut BB, maximize: bool, compute_time: u128) -> (Mv, f64) {
    let mut m_depth = 3;
    let mut q_depth = 6;

    let start_time = get_time_millis();
    let mut current_time = start_time;
    let mut best_move : Mv = Mv::null_move();
    let mut best_val = 0;
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
            q_depth,
            alpha,
            beta,
            maximize
        );

        if ply_move.is_err {break;}
        if ply_move.is_null || node_type != PV_NODE {aspire = false; continue;}

        (best_move, best_val) = (ply_move, ply_val);
        eprintln!("{} eval'd {} this time got val {}", m_depth, evaled, best_val);
        current_time = get_time_millis();
        aspire = true;
        m_depth += 1;
    }
    eprintln!("{} ply evaluated", m_depth - 1);
    eprintln!("{} nodes evaluated", evaled);
    eprintln!("projected value: {}", best_val);
    eprintln!("elapsed time: {}", current_time - start_time);
    return (best_move, best_val as f64 / 1000.);
}

fn is_terminal(node: &BB) -> bool {
    // TODO better terminal
    return node.material.abs() >= 100000;
}

unsafe fn evaluate_position(node: &BB) -> i32 {
    evaled += 1;
    return node.material * if node.white_turn {1} else {-1};
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

unsafe fn negamax_search(node: &mut BB, start_time: u128, compute_time: u128, depth: i32,
                         q_depth: i32, alpha: i32, beta: i32, maximize: bool) -> (Mv, i32, u8) {
    let current_time = get_time_millis();
    if current_time - start_time > compute_time { return (Mv::err_move(), 0, PV_NODE); }

    let mut alpha = alpha;
    let mut beta = beta;

    if is_terminal(&node) {
        return (Mv::null_move(), evaluate_position(&node), PV_NODE);
    }

    if depth == 0 {
        if is_quiet(node) {
            let val = evaluate_position(&node);
            return (Mv::null_move(), val, PV_NODE);
        } else {
            let (val, node_type) = quiescence_search(node, q_depth, alpha, beta, maximize);
            return (Mv::null_move(), val, node_type);
        }
    }

    let mut best_move = Mv::null_move();
    let mut val = LB;
    let mut moves = node.moves();
    for mv in moves.drain(0..) {
        node.do_move(&mv);
        let res = negamax_search(node, start_time, compute_time, depth - 1, q_depth, -beta, -alpha, !maximize);
        node.undo_move(&mv);
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
        alpha = cmp::max(alpha, val);

        if alpha >= beta {
            // beta cutoff
            return (best_move, beta, CUT_NODE);
        }
    }

    return (best_move, val, if val < alpha {ALL_NODE} else {PV_NODE});
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
