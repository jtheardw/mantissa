use core::arch::x86_64;
use std::cmp;
use std::sync::mpsc::{self, TryRecvError};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

pub mod bb;
use bb::BB;
use bb::Mv;

pub mod tt;
use tt::TT;
use tt::TTEntry;

pub mod pht;
use pht::PHT;

const LB: i32 = -10000000;
const UB: i32 = 10000000;

const PV_NODE: u8 = 1;
const CUT_NODE: u8 = 2;
const ALL_NODE: u8 = 3;

const BH_NONE: u8 = 0;
const BH_BRAIN: u8 = 1;
const BH_HAND: u8 = 2;

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

pub unsafe fn thread_handler(mut node: BB,
                             tnum: usize,
                             move_tx: Sender<(Mv, i32, u8)>,
                             depth: i32,
                             alpha: i32,
                             beta: i32,
                             maximize: bool,
                             forced_piece_idx: i32
) {
    let mut h_table = h_tables[tnum];
    let mut k_table = k_tables[tnum];
    let h_index = node.history.len();
    let result = negamax_search(&mut node, depth, 0, alpha, beta, maximize, true, true, true, &mut k_table, &mut h_table, Mv::null_move(), h_index, forced_piece_idx);
    move_tx.send(result).unwrap();
    evaled[tnum] += node.nodes_evaluated;
    hits[tnum] += node.tt_hits;
    k_tables[tnum] = k_table;
    h_tables[tnum] = h_table;
}

static mut kill_threads: bool = false;
static mut evaled: Vec<u64> = Vec::new();
static mut hits: Vec<u64> = Vec::new();
static mut k_tables: Vec<[[(Mv, i32); 2]; 64]> = Vec::new();
static mut h_tables: Vec<[[[u64; 64]; 6]; 2]> = Vec::new();

pub unsafe fn best_move(node: &mut BB, maximize: bool, compute_time: u128, nthreads: usize, bh_mode: u8, h_idx: i32) -> (Mv, f64) {
    node.nodes_evaluated = 0;
    node.tt_hits = 0;
    if is_terminal(&node) { return (Mv::null_move(), 0.0); }

    let mut m_depth: i32 = 1;
    let start_time: u128 = get_time_millis();
    let mut current_time: u128 = start_time;

    // initialize some values
    let mut best_move : Mv = Mv::null_move();
    let mut best_val: i32 = 0;

    let mut nodes_evaled = 0;
    let mut tt_hits = 0;

    // initialize persistent tables
    if !tt.valid {
        tt = TT::get_tt(24);
    }
    if !pht.valid {
        pht = PHT::get_pht(18);
    }

    if bh_mode == BH_HAND {
        // we need to verify that the piece we were given has legal moves
        if h_idx < 0 || h_idx > 63 {
            eprintln!("I need you to tell me a piece");
            return (Mv::null_move(), 0.0);
        }
        let moves = node.moves();
        let mut legal_move = false;
        for mv in moves {
            if mv.start != h_idx {continue;}
            node.do_move(&mv);
            legal_move = !node.is_check(maximize);
            node.undo_move(&mv);
            if legal_move {
                break;
            }
        }
        if !legal_move {
            eprintln!("There aren't any legal moves for that piece :/");
            return (Mv::null_move(), 0.0);
        }
    }

    evaled = Vec::new();
    hits = Vec::new();
    k_tables = Vec::new();
    h_tables = Vec::new();
    for i in 0..nthreads {
        evaled.push(0);
        hits.push(0);

        // killer moves table
        let k_table: [[(Mv, i32); 2]; 64] = [[(Mv::null_move(), 0); 2]; 64];
        // history table: h_table[s2m][piece][to]
        let h_table: [[[u64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
        k_tables.push(k_table);
        h_tables.push(h_table);
    }

    let mut aspire = 0;
    while (current_time - start_time) <= compute_time {
        let mut alpha = LB;
        let mut beta = UB;

        if aspire != 0 {
            if maximize {
                alpha = best_val - aspire;
                beta = best_val + aspire;
            } else {
                alpha = -best_val - aspire;
                beta = -best_val + aspire;
            }
        }

        // let (term_tx, term_rx) = mpsc::channel();
        let (move_tx, move_rx) = mpsc::channel();
        kill_threads = false;
        let mut threads = vec![];
        let mut extra_depth = 0;
        for t_num in 0..nthreads {
            // let extra_depth = if nthreads > 1 {
            //     if t_num > 0 {t_num.trailing_zeros() as i32} else {1}
            // } else {
            //     0
            // };
            if t_num % 2 == 0 && t_num >= 2 {
                extra_depth += 1;
            }
            // let extra_depth = 0;
            let mut node = node.copy();
            let move_tx = move_tx.clone();
            threads.push(thread::spawn(move || {
                thread_handler(node, t_num, move_tx, m_depth+extra_depth, alpha, beta, maximize, h_idx);
            }));
        }

        let mut ply_move = Mv::err_move();
        let mut ply_val = 0;
        let mut node_type = PV_NODE;
        loop {
            thread::sleep(Duration::from_millis(5));
            match move_rx.try_recv() {
                Ok((mv, score, n_type)) => {
                    ply_move = mv;
                    ply_val = score;
                    node_type = n_type;
                    kill_threads = true;
                    break;
                },
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => panic!("what.")
            }

            let current_time = get_time_millis();
            if current_time - start_time >= compute_time {
                if m_depth > 1 {
                    ply_move = Mv::err_move();
                    kill_threads = true;
                    break;
                }
            }
        }
        for t in threads {
            t.join();
        }

        // kind of a misnomer, this means we ran out of time
        if ply_move.is_err {break;}

        // we ended up outside of our aspiration window
        // let's widen it once.  Then if that fails
        // give up on aspiration for this iteration
        if ply_move.is_null || node_type != PV_NODE {
            if aspire == 250 {
                aspire = 1000;
            } else {
                aspire = 0;
            }
            continue;
        }

        // we maintain that positive is always white advantage here so we have to flip
        (best_move, best_val) = (ply_move, if maximize {ply_val} else {-ply_val});

        nodes_evaled = 0;
        tt_hits = 0;
        for i in 0..nthreads {
            nodes_evaled += evaled[i];
            tt_hits += hits[i];
        }

        if bh_mode == BH_NONE {
            // for debugging
            eprintln!(
                "{} eval'd {} this time got val {} and recommends {} with piece {}",
                m_depth,
                nodes_evaled,
                best_val,
                best_move,
                best_move.piece);
        }

        if best_val.abs() == 1000000 {
            // checkmate
            // no sense in continuing search
            break;
        }

        current_time = get_time_millis();
        aspire = 250;
        m_depth += 1;
    }

    nodes_evaled = 0;
    tt_hits = 0;
    for i in 0..nthreads {
        nodes_evaled += evaled[i];
        tt_hits += hits[i];
    }

    if bh_mode == BH_NONE {
        // all for debugging
        eprintln!("{} tt hits", tt_hits);
        eprintln!("{} ply evaluated", m_depth - 1);
        eprintln!("{} nodes evaluated", nodes_evaled);
        eprintln!("projected value: {}", best_val);
        eprintln!("elapsed time: {}", get_time_millis() - start_time);
        eprintln!("expected PV:");
        print_pv(node, m_depth - 1);

        // update info for spectators, &c
        println!("info depth {} score cp {}", m_depth - 1, (best_val / 10) * if maximize {1} else {-1});
    } else if bh_mode == BH_BRAIN {
        eprintln!("Piece to move: {} at {}", best_move.piece as char, BB::idx_to_str(best_move.start));
    } else if bh_mode == BH_HAND {
        eprintln!("Best move for chosen pieces is: {}", best_move);
    }
    return (best_move, best_val as f64 / 1000.);
}

fn is_terminal(node: &BB) -> bool {
    // no king check is now irrelevant since checkmate is explicitly detected
    // but we'll leave it in for now to be safe
    return node.king[0] == 0 || node.king[1] == 0 || node.is_threefold();
}

fn is_move_tactical(node: &BB, mv: &Mv) -> bool {
    // should be `is_move_other_tactical`
    // this should detect moves that are tactical but which aren't
    // captures or promotions (which are more trivially detectable)

    // currently is just pawn pushes past a certain rank
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
    let mut val = 0;

    // pawn values
    let pht_entry = pht.get(node.pawn_hash);
    if pht_entry.valid {
        val += node.get_tapered_eval(pht_entry.mg_val, pht_entry.eg_val);
    } else {
        let (mg_pv, eg_pv) = node.pawn_structure_value();
        val += node.get_tapered_eval(mg_pv, eg_pv);
        pht.set(node.pawn_hash, mg_pv, eg_pv);
    }
    val += node.material;
    val += node.pawn_defense_value();
    val += node.double_bishop_bonus();
    val += node.castled_bonus();
    val += node.pawn_advancement_value();
    val += node.get_all_pt_bonus();
    val += node.rook_on_seventh_bonus();
    val += node.rook_on_open_file_value();
    val += node.tempo_bonus();
    val += node.early_queen_penalty();
    val += node.material_advantage_bonus();
    val += node.bishop_color_value();

    let (mob, kdf) = node.mobility_kdf_combo();
    val += mob + kdf;

    return val * if node.white_turn {1} else {-1};
}

pub unsafe fn print_evaluate(node: &BB) {
    let (mob, kdf) = node.mobility_kdf_combo();

    eprintln!("Material: {}", node.material);
    eprintln!("Mobility: {}", mob);
    eprintln!("King danger value: {}", kdf);
    eprintln!("doubled p: {}", node.doubled_pawns_value(node.get_tapered_eval(node.eval_params.doubled_pawn, node.eval_params.eg_doubled_pawn)));
    eprintln!("isolated p: {}", node.isolated_pawns_value(node.get_tapered_eval(node.eval_params.isolated_pawn, node.eval_params.eg_isolated_pawn)));
    eprintln!("backwards p: {}", node.backwards_pawns_value(node.get_tapered_eval(node.eval_params.backwards_pawn, node.eval_params.eg_backwards_pawn)));
    eprintln!("passed p: {}", node.passed_pawns_value(node.get_tapered_eval(node.eval_params.passed_pawn, node.eval_params.eg_passed_pawn)));
    eprintln!("connected p: {}", node.connected_pawns_value(node.get_tapered_eval(node.eval_params.supported_bonus, node.eval_params.eg_supported_bonus), node.get_tapered_eval(node.eval_params.advancement_bonus, node.eval_params.eg_advancement_bonus)));
    eprintln!("pawn space control: {}", node.space_control_value(node.get_tapered_eval(node.eval_params.space, node.eval_params.eg_space)));
    eprintln!("bishop color value: {}", node.bishop_color_value());
    eprintln!("Center: {}", node.center_value(node.get_tapered_eval(node.eval_params.center_pawn, node.eval_params.eg_center_pawn)));
    eprintln!("Near Center: {}", node.near_center_value(node.get_tapered_eval(node.eval_params.near_center_pawn, node.eval_params.eg_near_center_pawn)));
    eprintln!("Double bishop: {}", node.double_bishop_bonus());
    eprintln!("Pawn Defense: {}", node.pawn_defense_value());
    eprintln!("Pawn advancement: {}", node.pawn_advancement_value());
    eprintln!("Castle Bonus: {}", node.castled_bonus());
    eprintln!("Early queen penalty: {}", node.early_queen_penalty());
    eprintln!("All pt bonus: {}", node.get_all_pt_bonus());
    eprintln!("Tempo: {}", node.tempo_bonus());
    eprintln!("Rook on 7th: {}", node.rook_on_seventh_bonus());
    eprintln!("Rook on (semi-)open file: {}", node.rook_on_open_file_value());
    eprintln!("Material lead bonus: {}", node.material_advantage_bonus());
    // if pht.valid {
    //     let pht_entry = pht.get(node.pawn_hash);
    //     if pht_entry.valid {
    //         eprintln!("pawn entry: {}", pht_entry.val);
    //     }
    // }
    eprintln!("Phase {} / 256", node.get_phase());
}

fn is_capture(node: &mut BB) -> bool {
    if node.cap_stack.len() > 0 {
        return node.cap_stack[node.cap_stack.len() - 1] != 0;
    }
    return false;
}

fn is_quiet(node: &mut BB) -> bool {
    return !node.side_to_move_has_capture();
}

fn moves_equivalent(mv1: &Mv, mv2: &Mv) -> bool {
    return  mv1.start == mv2.start
            && mv1.end == mv2.end
            && mv1.piece == mv2.piece
            && mv1.promote_to == mv2.promote_to
            && mv1.ep_tile == mv2.ep_tile
            && mv1.is_ep == mv2.is_ep;
}

fn order_moves(moves: Vec<(Mv, u64)>, best_move: Mv) -> Vec<(Mv, u64)> {
    // currently mostly just pushes the expected
    // "best move" to the front with a baked-in
    // legality check
    if best_move.is_null { return moves; }
    let mut found_move = false;
    let mut new_q: Vec<(Mv, u64)> = Vec::new();
    new_q.push((best_move, 0xFFFFFFFFFFFFFFFF));
    for mv in moves {
        if moves_equivalent(&mv.0, &best_move) {
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

fn is_killer(k_table: &[[(Mv, i32); 2]; 64], mv: Mv, ply: i32) -> bool {
    let k_row = k_table[ply as usize];
    return moves_equivalent(&k_row[0].0, &mv)
        || moves_equivalent(&k_row[1].0, &mv);
}

fn update_k_table(k_table: & mut [[(Mv, i32); 2]; 64], mv: Mv, depth: i32, ply: i32) {
    let ply = ply as usize;
    if moves_equivalent(&k_table[ply][0].0, &mv) {
        if depth > k_table[ply][0].1 {
            k_table[ply][0] = (mv, depth);
        }
        return;
    }

    k_table[ply][1] = k_table[ply][0];
    k_table[ply][0] = (mv, depth);
    // if moves_equivalent(&k_table[ply][1].0, &mv) {
    //     if depth > k_table[ply][1].1 {
    //         k_table[ply][1] = (mv, depth);
    //     }
    //     return;
    // }

    // if k_table[ply][0]

    // if k_table[ply][0].0.is_null {
    //     k_table[ply][0] = (mv, depth);
    //     return;
    // }
    // if k_table[ply][1].0.is_null {
    //     k_table[ply][1] = (mv, depth);
    //     return;
    // }

    // if k_table[ply][0].1 <= depth {
    //     k_table[ply][0] = (mv, depth);
    //     return;
    // } else { // if k_table[ply][1].1 <= depth {
    //     k_table[ply][1] = (mv, depth);
    // }
}

fn move_sort(mvs: & mut Vec<(Mv, u64)>, cur_i: usize) {
    let mut highest_i = cur_i;

    for i in (cur_i + 1)..mvs.len() {
        if mvs[i].1 > mvs[highest_i].1 {
            highest_i = i;
        }
    }

    // swap
    let (mv, score) = mvs[highest_i];
    mvs[highest_i] = mvs[cur_i];
    mvs[cur_i] = (mv, score);
}

unsafe fn score_root_moves(node: &mut BB, mvs: Vec<Mv>) -> Vec<(Mv, u64)>{
    let mut mv_q: Vec<(Mv, u64)> = Vec::new();
    for mv in mvs {
        let mut score = 0;
        node.do_move(&mv);
        let tt_entry = tt.get(node.hash);
        node.undo_move(&mv);
        if tt_entry.valid {
            score = (10000000 - tt_entry.value) as u64;
        }
        mv_q.push((mv, score));
    }
    return mv_q;
}


// TODO: can clean up all these args
unsafe fn negamax_search(node: &mut BB,
                         depth: i32,
                         ply: i32,
                         alpha: i32,
                         beta: i32,
                         maximize: bool,
                         nmr_ok: bool,
                         init: bool,
                         is_pv: bool,
                         k_table: & mut [[(Mv, i32); 2]; 64],
                         h_table: & mut [[[u64; 64]; 6]; 2],
                         sing_move: Mv,
                         h_index: usize,
                         forced_piece_idx: i32
) -> (Mv, i32, u8) {
    node.nodes_evaluated += 1;
    x86_64::_mm_prefetch(tt.get_ptr(node.hash), x86_64::_MM_HINT_NTA);
    x86_64::_mm_prefetch(pht.get_ptr(node.pawn_hash), x86_64::_MM_HINT_NTA);

    if !init {
        if node.history[h_index..].iter().filter(|&n| *n == node.hash).count() > 0 {
            return (Mv::null_move(), 0, PV_NODE);
        }
    }
    if is_terminal(&node) {
        if node.is_threefold() {
            return (Mv::null_move(), 0, PV_NODE);
        }
        return (Mv::null_move(), evaluate_position(&node), PV_NODE);
    }
    if depth <= 0 {
        let (val, node_type) = quiescence_search(node, 0, alpha, beta, maximize);
        return (Mv::null_move(), val, node_type);
    }

    if kill_threads {
        return (Mv::err_move(), 0, PV_NODE);
    }

    let mut first_move = Mv::null_move();
    let mut depth = depth;
    let mut tt_entry = tt.get(node.hash);
    if tt_entry.valid {
        node.tt_hits += 1;
        let mv = tt_entry.mv;
        first_move = mv;
        if (sing_move.is_null && !is_pv && tt_entry.depth >= depth) {
            if ((tt_entry.value <= alpha || tt_entry.value >= beta) && tt_entry.node_type == PV_NODE)
                || (tt_entry.value <= alpha && tt_entry.node_type == ALL_NODE) // all node is upper bound
                || (tt_entry.value >= beta && tt_entry.node_type == CUT_NODE) // cut node is lower bound
            {
                // we do a janky legality check just in case of freak collisions
                // where we use the fact that move ordering will make sure a move comes from this
                // position
                let mut moves = order_moves(node.get_scored_moves(node.moves(), &k_table[ply as usize], &h_table), first_move);
                if moves.len() > 0 {
                    let (fm, _) = moves[0];
                    if moves_equivalent(&first_move, &fm) {
                        // check 3 fold
                        if !node.is_repitition() {

                            // TODO: update history here
                            let node_type = if tt_entry.value > alpha {if tt_entry.value >= beta {CUT_NODE} else {PV_NODE}} else {ALL_NODE};
                            if node_type == CUT_NODE {
                                node.do_move(&mv);
                                let quiet = !is_capture(node) && mv.promote_to == 0 && !mv.is_ep;
                                node.undo_move(&mv);
                                if quiet {
                                    update_k_table(k_table, mv, depth, ply);
                                    h_table[maximize as usize][mv.get_piece_num()][mv.end as usize] += 1 << depth;
                                }
                            }
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
    } else if depth >= 6 && is_pv {
        // internal iterative deepening
        let (mv, val, _) = negamax_search(node, depth - 2, ply, alpha, beta, maximize, true, false, false, k_table, h_table, Mv::null_move(), h_index, -1);
        first_move = mv;
    }

    if first_move.is_err {
        return (first_move, 0, PV_NODE);
    }

    let is_check = node.is_check(maximize);
    if is_check {
        // check extension
        depth += 1;
    }

    let mut alpha = alpha;
    let beta = beta;

    let mut raised_alpha = false;
    let mut best_move = Mv::null_move();
    let mut val = LB;

    // RFP
    if (depth < 3) && !is_pv && !is_check && !init && nmr_ok {
        let cur_val = evaluate_position(&node);

        let margin = [0, 1300, 2500];
        if (cur_val - margin[depth as usize]) >= beta {
            return (Mv::null_move(), (cur_val - margin[depth as usize]), CUT_NODE);
        }
    }

    if !is_pv
        && depth >= 3
        && !is_check
        && nmr_ok
        && !init
        && node.phase < 240 {
            let static_eval = evaluate_position(&node);
            if static_eval >= beta {
                let mut r = if depth > 6 {3} else {2};
                r += ((static_eval - beta) / 1000) as i32;

                node.do_null_move();
                let nmr_val = -negamax_search(node, depth - 1 - r, ply+1, -beta, -beta + 1, !maximize, false, false, false, k_table, h_table, Mv::null_move(), h_index, -1).1;
                node.undo_null_move();

                if nmr_val >= beta {
                    // using the extended null move reductions
                    // idea from Eli David and Nathan S. Netanyahu
                    // in the paper of the same name
                    // return (Mv::null_move(), nmr_val, CUT_NODE)
                    // depth -= 4;
                    // if depth <= 0 {
                    //     return negamax_search(node, 0, ply, alpha, beta, maximize, false, false, false, k_table, h_table, Mv::null_move(), h_index);
                    // }
                    return (Mv::null_move(), beta, CUT_NODE);
                }
            }
        }

    let mut sing_extend = false;
    if  !init
        && !is_check
        && depth >= 6
        && tt_entry.valid
        && sing_move.is_null
        && tt_entry.value.abs() != 1000000
        && (tt_entry.node_type == CUT_NODE || tt_entry.node_type == PV_NODE)
        && tt_entry.depth >= depth - 3
    {
        // I've stolen stockfish's idea here to combine singular
        // move detection with multi-cut in the same search
        //
        // I can't afford to do the super-tight cutoffs stockfish does though
        let margin = if tt_entry.node_type == PV_NODE {25 * (depth / 2)} else {20 * (depth / 2)}; // 200
        let depth_to_search = if tt_entry.node_type == PV_NODE {(depth + 2) / 2} else {(depth - 1) / 2};
        // let depth_to_search = (2 * depth) / 3;
        let target = tt_entry.value - margin;

        let val = negamax_search(node, depth_to_search, ply, target - 1, target, maximize, false, false, false, k_table, h_table, tt_entry.mv, h_index, -1).1;

        if (val < target) {
            // singular extension
            depth += 1;
            sing_extend = true;
        } else if !is_pv && target >= beta {
            // if we're not doing pv node we might want to prune here with
            // multi-cut.  This indicates multiple moves failed high
            // so this is probably a cutnode
            return (Mv::null_move(), target, CUT_NODE);
        } else if !is_pv && tt_entry.value >= beta {
            let val = negamax_search(node, (depth + 3) / 2, ply, beta - 1, beta, maximize, true, false, false, k_table, h_table, tt_entry.mv, h_index, -1).1;
        }
    }

    let mut moves: Vec<(Mv, u64)>;
    if init {
        let mut unscored_moves = node.moves();
        if forced_piece_idx != -1 {
            let mut filtered_moves: Vec<Mv> = Vec::new();
            for mv in unscored_moves {
                if mv.start == forced_piece_idx {
                    filtered_moves.push(mv);
                }
            }
            unscored_moves = filtered_moves;
        }

        if depth > 4 {
            moves = score_root_moves(node, unscored_moves);
        } else {
            moves = order_moves(node.get_scored_moves(unscored_moves, &k_table[ply as usize], &h_table), first_move);
        }
    } else {
        moves = order_moves(node.get_scored_moves(node.moves(), &k_table[ply as usize], &h_table), first_move);
    }
    let mut num_moves = 0;
    let mut cur_i = 0;

    // futility pruning and extended futility pruning
    let mut is_futile = false;
    if !is_pv && !is_check && !init && depth <= 3 {
        let futile_margin = [0, 2200, 3200, 5300];
        let futile_val = evaluate_position(&node);
        if depth == 3 {
            if futile_val < alpha - futile_margin[3] {
                depth = 2;
            }
        }
        if futile_val <= (alpha - futile_margin[depth as usize]) {
            is_futile = true;
            val = futile_val;
        }
    }
    let mut legal_move = false;

    while cur_i < moves.len() {
        move_sort(&mut moves, cur_i);
        let (mv, score) = moves[cur_i];
        // println!("depth {} cur_i {} mov {} score {}", depth, cur_i, mv, score);
        cur_i += 1;

        if !sing_move.is_null && moves_equivalent(&mv, &sing_move) {
            continue;
        }

        let is_tactical_move = is_move_tactical(&node, &mv);
        node.do_move(&mv);
        if node.is_check(maximize) {
            node.undo_move(&mv);
            continue;
        } else if is_futile {
            legal_move = true;
            if !is_capture(node) &&
                !is_terminal(node) &&
                !node.is_check(node.white_turn) &&
                (mv.promote_to == 0) &&
                !is_tactical_move {
                    // don't prune out tactical moves even in a 'futile'
                    // position
                    node.undo_move(&mv);
                    continue;
                }
        }

        legal_move = true;
        let mut res: (Mv, i32, u8) = (Mv::null_move(), LB, PV_NODE);

        let quiet = !is_capture(node) && mv.promote_to == 0 && !mv.is_ep;
        // let terminal = is_terminal(node);

        let mut reduced = false;
        if num_moves == 0 {
            res = negamax_search(node, depth - 1, ply+1, -beta, -alpha, !maximize, true, false, true, k_table, h_table, Mv::null_move(), h_index, -1);
        } else {
            if depth > 3
            // && !is_pv
                && !init
                && (quiet || score < 11000)
                && num_moves >= 4
                && !is_check
                // && !is_terminal(node)
                && !node.is_check(node.white_turn)
                && !is_killer(&k_table, mv, ply)
                // && (ply < 2 || !is_killer(&k_table, mv, ply - 2))
                && !is_tactical_move {
                    // late move reductions
                    // let mut r = (0.75 + ((num_moves as f64).ln() * (depth as f64).ln()) / 2.25) as i32;
                    // r = (((num_moves - 1) as f64).sqrt() + ((depth - 1) as f64).sqrt()) as i32;
                    // r = 1;
                    let mut r = 1 + ((num_moves - 4) / 4) + (depth / 8);
                    // r = (0.75 + (((num_moves) as f64).ln() * ((depth) as f64).ln()) / 2.25) as i32;
                    // r = 1;
                    // r = (depth / 3);
                    // if is_pv {
                    //     r = (depth / 5);
                    // }

                    if score > (1 << 16) {
                        if score > (1 << (16 + (depth - 4))) {
                        // if score > ((1 << 16) + (depth-3) * (depth-3)) as u64 {
                            r -= 1;
                        }
                        // if score > ((1 << 16) + (depth-1) * (depth-1)) as u64 {
                        if score > (1 << (16 + (depth - 2))) {
                            r -= 1;
                        }
                    }

                    // let tte = tt.get(node.hash);
                    let tte = tt_entry;
                    if tte.valid && tte.node_type == PV_NODE {// && tte.value > (alpha - 500) {
                        r -= 1;
                    } else if tte.valid && tte.node_type != PV_NODE {
                        r += 1;
                    }
                    // else if tte.valid && tte.node_type != PV_NODE {
                    //     r += 1;
                    // }

                    if !quiet && score > 8000 {
                        // not very bad capture
                        r -= 1;
                    } else if !quiet && score < 8000 {
                        // very bad capture
                        r += 1;
                    }

                    // if sing_extend {
                    //     r -= 1;
                    // }

                    // if num_moves >= 10 {
                    //     // later move reductions
                    //     if !is_pv {
                    //         r = (depth / 3);
                    //     } else if depth >= 5 {
                    //         r = (depth / 5);
                    //     }
                    // }
                    // }
                    if is_pv && r > 0 { r = (r * 2) / 3; }
                    let depth_to_search = depth - 1 - r;

                    res = negamax_search(node, depth_to_search, ply + 1, -alpha - 1, -alpha, !maximize, true, false, false, k_table, h_table, Mv::null_move(), h_index, -1);

                    if -res.1 <= alpha {
                        // if we fail to raise alpha we can move on
                        // otherwise, we'll have to redo the search
                        reduced = true;
                    }
                } else {
                    // trick so that we get into the ZWS directly when we don't reduce
                    reduced = false;
                }

            if !reduced {
                res = negamax_search(node, depth - 1, ply + 1, -alpha - 1, -alpha, !maximize, true, false, false, k_table, h_table, Mv::null_move(), h_index, -1);
                if -res.1 > alpha && -res.1 < beta { // failed high
                    res = negamax_search(node, depth - 1, ply + 1, -beta, -alpha, !maximize, true, false, true, k_table, h_table, Mv::null_move(), h_index, -1);
                }
            }

        }
        node.undo_move(&mv);

        num_moves += 1;
        let ret_move = res.0;
        if ret_move.is_err {
            // misnomer.  We ran out of compute time
            return (Mv::err_move(), 0, PV_NODE);
        }
        let ret_val = -res.1;

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
                // killer moves
                // and history moves
                update_k_table(k_table, mv, depth, ply);
                h_table[maximize as usize][mv.get_piece_num()][mv.end as usize] += 1 << depth; // (depth * depth) as u64;
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

    if sing_move.is_null {
        tt.set(node.hash, best_move, val, if !raised_alpha {ALL_NODE} else {PV_NODE}, depth);
    }
    return (best_move, val, if !raised_alpha {ALL_NODE} else {PV_NODE});
}

pub unsafe fn q_eval(node: &mut BB, maximize: bool) -> i32 {
    if !pht.valid {
        pht = PHT::get_pht(18);
    }
    let (val, _) = quiescence_search(node, 0, -1000000, 1000000, maximize);
    return val;
}

unsafe fn quiescence_search(node: &mut BB, depth: i32, alpha: i32, beta: i32, maximize: bool) -> (i32, u8) {
    node.nodes_evaluated += 1;
    if depth <= -32 || is_terminal(node) || is_quiet(node) {
        return (evaluate_position(node), PV_NODE);
    }

    let mut alpha = alpha;
    let mut raised_alpha = false;

    let curr_val = evaluate_position(node);
    let is_check = node.is_check(maximize);
    // standing pat check so we *do* stop eventually
    if curr_val >= beta {
        // beta cutoff
        return (beta, CUT_NODE);
    }
    if curr_val > alpha {
        raised_alpha = true;
        alpha = curr_val;
    } else {
        if node.get_phase() < 180 && curr_val < alpha - 10000 {
            return (curr_val, ALL_NODE);
        }
    }


    let mut best_val = LB;
    let mut best_mv: Mv = Mv::null_move();

    let mut mv_q = node.get_scored_q_moves(node.q_moves());
    let mut cur_i = 0;
    if mv_q.len() == 0 {
        return (curr_val, if curr_val > alpha { if curr_val >= beta {CUT_NODE} else {PV_NODE} } else {ALL_NODE});
    }

    while cur_i < mv_q.len() {
        move_sort(&mut mv_q, cur_i);
        let (mv, score) = mv_q[cur_i];
        cur_i += 1;
        node.do_move(&mv);
        // check legality
        if node.is_check(maximize) {node.undo_move(&mv); continue;}

        // delta pruning
        // if we're very behind of where we could be (alpha)
        // we should only accept exceptionally good captures
        if node.phase <= 180 { //&& !is_check {
            let mut futile = false;
            match node.cap_stack[node.cap_stack.len() - 1] {
                b'p' => { if alpha > curr_val + 3000 { futile = true; }},
                b'n' => { if alpha > curr_val + 5000 { futile = true; }},
                b'b' => { if alpha > curr_val + 5000 { futile = true; }},
                b'r' => { if alpha > curr_val + 7000 { futile = true; }},
                b'q' => { if alpha > curr_val + 11000 { futile = true; }},
                _ => {}
            };
            if futile {
                node.undo_move(&mv);
                continue;
            }
        }

        if score < ((1 << 45) + 100) {
            // use see to see if this is viable capture
            let cap_piece = node.cap_stack[node.cap_stack.len() - 1];
            if cap_piece != 0 {
                // capture happened, let's SEE this
                node.undo_move(&mv);
                let see = node.see(mv.end, cap_piece, mv.start, mv.piece);
                if see < 0 {
                    continue;
                } else {
                    node.do_move(&mv);
                }
            }
        }
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
    let node_val = if !best_mv.is_null {best_val} else {curr_val};
    let node_type = if raised_alpha {PV_NODE} else {ALL_NODE};
    return (node_val, node_type);
}
