use std::cmp;
use std::time::SystemTime;

mod node;

fn choose_move(node: &mut Node, maximize: &bool, compute_time: i32) -> (Move, i32) {
    let mut m_depth = 3;
    let mut q_depth = 0;

    let start_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime failed!"),
    };
    let mut current_time = start_time;
    let mut best_move : Move;
    let mut val = if (maximize) {-200000} else {200000};

    while ((start_time - current_time) <= compute_time) {
        (best_move, val) = minimax_search(&node, m_depth, q_depth, -200000, 200000, maximize);

        current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime failed!"),
        };
        m_depth += 1;
    }
}

fn minimax_search(node: &mut Node, depth: i32, q_depth: i32, alpha: i32, beta: i32, maximize: bool) -> (Move, i32) {
    let mut alpha = alpha;
    let mut beta = beta;

    // TODO: deepening time check?

    if is_terminal(&node) {
        return (node::Move::null_move(), evaluate_position(&node));
    }

    if (depth == 0) {
        if (q_depth == 0 || is_quiet(&node)) {
            val = evaluate_position(&node);
            return (node::Move::null_move(), val);
        } else {
            return quiescence_search(&node, q_depth, alpha, beta, maximize)
        }
    }

    let mut best_move = node::Move::null_move();
    let mut val = if (maximize) {-200000} else {200000};
    for pot_move in get_moves(&node).iter() {
        node.do_move(pot_move);
        let (_, new_val) = minimax_search(&node, depth - 1, q_depth, alpha, beta, !maximize);
        node.undo_move(pot_move);
        if (maximize) {
            if (new_val > val || best_move.isnull()) {
                best_move = pot_move;
                val = new_val;
            }
            alpha = cmp::max(alpha, val);
        } else {
            if (new_val < val || best_move.isnull()) {
                best_move = pot_move;
                val = new_val;
            }
            beta = cmp::min(beta, val);
        }
        if (alpha >= beta) {
            break;
        }
    }
    return (best_move, val);
}

fn quiescence_search(node: &mut Node, depth: i32, alpha: i32, beta: i32, maximize: bool) -> (Move, i32) {
    return (node::Move::null_move(), 0);        // TODO
}
