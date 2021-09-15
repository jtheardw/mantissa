use std::cmp;
use std::iter::FromIterator;
use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};

use crate::bitboard::*;
use crate::tuning_eval::*;
use crate::rand::*;
use crate::search::get_time_millis;
use crate::see::*;
use crate::moveorder::*;
use crate::tuning_psqt::*;
use crate::util::*;

const LB: i32 = -10000000;
const UB: i32 = -10000000;

macro_rules! S {
    ($a:expr, $b:expr) => {
        make_score($a, $b)
    }
}

// queen_value
// rook_value
// bishop_value
// knight_value
// pawn_value

// knight_mobility
// bishop_mobility
// rook_mobility
// queen_mobility

// queen_king_danger
// rook_king_danger
// bishop_king_danger
// knight_king_danger
// king danger scale

// double_bishop_bonus

// passed_pawn_value
// center_pawn_value
// isolated_pawn_value
// doubled_pawn_value
// backwards_pawn_value
// advanced_pawn_value
// supported_pawn_bonus
// space_value

// bishop_color

// rook_on_seventh
// rook_on_open

// tempo_bonus

fn set_mg_score(score: Score, val: i32) -> i64 {
    S!(val, eg_score(score))
}

fn set_eg_score(score: Score, val: i32) -> i64 {
    S!(mg_score(score), val)
}

pub fn get_params_vector() -> Vec<i32> {
    let mut v = Vec::new();

    for i in 0..584 {
        v.push(get_param(i));
    }

    return v;
}

fn load_params_vector(v: &Vec<i32>) {
    for i in 0..584 {
        set_param(i, v[i]);
    }
}

fn print_score(name: &str, mg: i32, eg: i32) {
    println!("{} S!({}, {})", name, mg, eg);
}

fn print_score_vector(name: &str, v_mg: &Vec<i32>, v_eg: &Vec<i32>) {
    let mut s = format!("");
    for i in 0..v_mg.len() {
        if s.len() > 0 {
            s = format!("{}, S!({}, {})", s, v_mg[i], v_eg[i]);
        } else {
            s = format!("S!({}, {})", v_mg[i], v_eg[i]);
        }
    }
    println!("{} [{}]", name, s);
}

fn v_copy(v: &Vec<i32>, l: usize, u: usize) -> Vec<i32> {
    Vec::from_iter(v[l..u].iter().cloned())
}

pub fn print_params_vector(v: &Vec<i32>) {
    print_score("queen value", v[0], v[1]);
    print_score("rook value", v[2], v[3]);
    print_score("bishop value", v[4], v[5]);
    print_score("knight value", v[6], v[7]);
    print_score("pawn value", v[8], v[9]);

    print_score_vector("knight mobility", &v_copy(v, 10, 19), &v_copy(v, 19, 28));
    print_score_vector("bishop mobility", &v_copy(v, 28, 42), &v_copy(v, 42, 56));
    print_score_vector("rook mobility", &v_copy(v, 56, 71), &v_copy(v, 71, 86));
    print_score_vector("queen mobility", &v_copy(v, 86, 114), &v_copy(v, 114, 142));

    print_score_vector("queen king danger", &v_copy(v, 142, 150), &v_copy(v, 142, 150));
    print_score_vector("rook king danger", &v_copy(v, 150, 158), &v_copy(v, 150, 158));
    print_score_vector("bishop king danger", &v_copy(v, 158, 166), &v_copy(v, 158, 166));
    print_score_vector("knight king danger", &v_copy(v, 166, 174), &v_copy(v, 166, 174));

    print_score("double bishop", v[174], v[175]);

    print_score_vector("passed pawn", &v_copy(v, 176, 182), &v_copy(v, 182, 188));
    print_score("center pawn", v[188], v[189]);
    print_score("isolated pawn", v[190], v[191]);
    print_score("doubled pawn", v[192], v[193]);
    print_score("backwards pawn", v[194], v[195]);
    print_score_vector("advanced pawn", &v_copy(v, 196, 202), &v_copy(v, 202, 208));
    print_score("supported pawn", v[208], v[209]);
    print_score("space", v[210], v[211]);

    print_score("rook on 7th", v[212], v[213]);
    print_score("rook on open", v[214], v[215]);

    print_score_vector("pawn psqt 2", &v_copy(v, 216, 220), &v_copy(v, 240, 244));
    print_score_vector("pawn psqt 3", &v_copy(v, 220, 224), &v_copy(v, 244, 248));
    print_score_vector("pawn psqt 4", &v_copy(v, 224, 228), &v_copy(v, 248, 252));
    print_score_vector("pawn psqt 5", &v_copy(v, 228, 232), &v_copy(v, 252, 256));
    print_score_vector("pawn psqt 6", &v_copy(v, 232, 236), &v_copy(v, 256, 260));
    print_score_vector("pawn psqt 7", &v_copy(v, 236, 240), &v_copy(v, 260, 264));

    print_score_vector("knight psqt 1", &v_copy(v, 264, 268), &v_copy(v, 296, 300));
    print_score_vector("knight psqt 2", &v_copy(v, 268, 272), &v_copy(v, 300, 304));
    print_score_vector("knight psqt 3", &v_copy(v, 272, 276), &v_copy(v, 304, 308));
    print_score_vector("knight psqt 4", &v_copy(v, 276, 280), &v_copy(v, 308, 312));
    print_score_vector("knight psqt 5", &v_copy(v, 280, 284), &v_copy(v, 312, 316));
    print_score_vector("knight psqt 6", &v_copy(v, 284, 288), &v_copy(v, 316, 320));
    print_score_vector("knight psqt 7", &v_copy(v, 288, 292), &v_copy(v, 320, 324));
    print_score_vector("knight psqt 8", &v_copy(v, 292, 296), &v_copy(v, 324, 328));

    print_score_vector("bishop psqt 1", &v_copy(v, 328, 332), &v_copy(v, 360, 364));
    print_score_vector("bishop psqt 2", &v_copy(v, 332, 336), &v_copy(v, 364, 368));
    print_score_vector("bishop psqt 3", &v_copy(v, 336, 340), &v_copy(v, 368, 372));
    print_score_vector("bishop psqt 4", &v_copy(v, 340, 344), &v_copy(v, 372, 376));
    print_score_vector("bishop psqt 5", &v_copy(v, 344, 348), &v_copy(v, 376, 380));
    print_score_vector("bishop psqt 6", &v_copy(v, 348, 352), &v_copy(v, 380, 384));
    print_score_vector("bishop psqt 7", &v_copy(v, 352, 356), &v_copy(v, 384, 388));
    print_score_vector("bishop psqt 8", &v_copy(v, 356, 360), &v_copy(v, 388, 392));

    print_score_vector("rook psqt 1", &v_copy(v, 392, 396), &v_copy(v, 424, 428));
    print_score_vector("rook psqt 4", &v_copy(v, 396, 400), &v_copy(v, 428, 432));
    print_score_vector("rook psqt 4", &v_copy(v, 400, 404), &v_copy(v, 432, 436));
    print_score_vector("rook psqt 4", &v_copy(v, 404, 408), &v_copy(v, 436, 440));
    print_score_vector("rook psqt 5", &v_copy(v, 408, 412), &v_copy(v, 440, 444));
    print_score_vector("rook psqt 6", &v_copy(v, 412, 416), &v_copy(v, 444, 448));
    print_score_vector("rook psqt 7", &v_copy(v, 416, 420), &v_copy(v, 448, 452));
    print_score_vector("rook psqt 8", &v_copy(v, 420, 424), &v_copy(v, 452, 456));

    print_score_vector("queen psqt 1", &v_copy(v, 456, 460), &v_copy(v, 488, 492));
    print_score_vector("queen psqt 4", &v_copy(v, 460, 464), &v_copy(v, 492, 496));
    print_score_vector("queen psqt 3", &v_copy(v, 464, 468), &v_copy(v, 496, 500));
    print_score_vector("queen psqt 4", &v_copy(v, 468, 472), &v_copy(v, 500, 504));
    print_score_vector("queen psqt 5", &v_copy(v, 472, 476), &v_copy(v, 504, 508));
    print_score_vector("queen psqt 6", &v_copy(v, 476, 480), &v_copy(v, 508, 512));
    print_score_vector("queen psqt 7", &v_copy(v, 480, 484), &v_copy(v, 512, 516));
    print_score_vector("queen psqt 8", &v_copy(v, 484, 488), &v_copy(v, 516, 520));

    print_score_vector("king psqt 1", &v_copy(v, 520, 524), &v_copy(v, 552, 556));
    print_score_vector("king psqt 4", &v_copy(v, 524, 528), &v_copy(v, 556, 560));
    print_score_vector("king psqt 3", &v_copy(v, 528, 532), &v_copy(v, 560, 564));
    print_score_vector("king psqt 4", &v_copy(v, 532, 536), &v_copy(v, 564, 568));
    print_score_vector("king psqt 5", &v_copy(v, 536, 540), &v_copy(v, 568, 572));
    print_score_vector("king psqt 6", &v_copy(v, 540, 544), &v_copy(v, 572, 576));
    print_score_vector("king psqt 7", &v_copy(v, 544, 548), &v_copy(v, 576, 580));
    print_score_vector("king psqt 8", &v_copy(v, 548, 552), &v_copy(v, 580, 584));
}

fn get_param(idx: usize) -> i32 {
    unsafe {
        return match idx {
            0 => { mg_score(QUEEN_VALUE) },
            1 => { eg_score(QUEEN_VALUE) },
            2 => { mg_score(ROOK_VALUE) },
            3 => { eg_score(ROOK_VALUE) },
            4 => { mg_score(BISHOP_VALUE) },
            5 => { eg_score(BISHOP_VALUE) },
            6 => { mg_score(KNIGHT_VALUE) },
            7 => { eg_score(KNIGHT_VALUE) },
            8 => { mg_score(PAWN_VALUE) },
            9 => { eg_score(PAWN_VALUE) },

            10..19 => { mg_score(KNIGHT_MOBILITY[idx - 10]) },
            19..28 => { eg_score(KNIGHT_MOBILITY[idx - 19]) },
            28..42 => { mg_score(BISHOP_MOBILITY[idx - 28]) },
            42..56 => { eg_score(BISHOP_MOBILITY[idx - 42]) },
            56..71 => { mg_score(ROOK_MOBILITY[idx - 56]) },
            71..86 => { eg_score(ROOK_MOBILITY[idx - 71]) },
            86..114 => { mg_score(QUEEN_MOBILITY[idx - 86]) },
            114..142 => { eg_score(QUEEN_MOBILITY[idx - 114]) },

            142..150 => { QUEEN_KING_DANGER[idx - 142] },
            150..158 => { ROOK_KING_DANGER[idx - 150] },
            158..166 => { BISHOP_KING_DANGER[idx - 158] },
            166..174 => { KNIGHT_KING_DANGER[idx - 166] },

            174 => { mg_score(DOUBLE_BISHOP_BONUS) },
            175 => { eg_score(DOUBLE_BISHOP_BONUS) },

            176..182 => { mg_score(PASSED_PAWN_VALUE[idx - 175]) },
            182..188 => { eg_score(PASSED_PAWN_VALUE[idx - 181]) },
            188 => { mg_score(CENTER_PAWN_VALUE) },
            189 => { eg_score(CENTER_PAWN_VALUE) },
            190 => { mg_score(ISOLATED_PAWN_VALUE) },
            191 => { eg_score(ISOLATED_PAWN_VALUE) },
            192 => { mg_score(DOUBLED_PAWN_VALUE) },
            193 => { eg_score(DOUBLED_PAWN_VALUE) },
            194 => { mg_score(BACKWARDS_PAWN_VALUE) },
            195 => { eg_score(BACKWARDS_PAWN_VALUE) },
            196..202 => { mg_score(ADVANCED_PAWN_VALUE[idx - 195]) },
            202..208 => { eg_score(ADVANCED_PAWN_VALUE[idx - 201]) },
            208 => { mg_score(SUPPORTED_PAWN_BONUS) },
            209 => { eg_score(SUPPORTED_PAWN_BONUS) },
            210 => { mg_score(SPACE_VALUE) },
            211 => { eg_score(SPACE_VALUE) },

            212 => { mg_score(ROOK_ON_SEVENTH) },
            213 => { eg_score(ROOK_ON_SEVENTH) },
            214 => { mg_score(ROOK_ON_OPEN) },
            215 => { eg_score(ROOK_ON_OPEN) },

            // pawn table is special, we only care about the middle 6
            216..240 => {
                let pr = (idx + 4 - 216) / 4;
                let pc = (idx + 4 - 216) % 4;
                mg_score(PAWN_PSQT[pr][pc])
            },
            240..264 => {
                let pr = (idx + 4 - 240) / 4;
                let pc = (idx + 4 - 240) % 4;
                eg_score(PAWN_PSQT[pr][pc])
            },
            264..296 => {
                let pr = (idx - 264) / 4;
                let pc = (idx - 264) % 4;
                mg_score(KNIGHT_PSQT[pr][pc])
            },
            296..328 => {
                let pr = (idx - 296) / 4;
                let pc = (idx - 296) % 4;
                eg_score(KNIGHT_PSQT[pr][pc])
            },
            328..360 => {
                let pr = (idx - 328) / 4;
                let pc = (idx - 328) % 4;
                mg_score(BISHOP_PSQT[pr][pc])
            },
            360..392 => {
                let pr = (idx - 360) / 4;
                let pc = (idx - 360) % 4;
                eg_score(BISHOP_PSQT[pr][pc])
            },
            392..424 => {
                let pr = (idx - 392) / 4;
                let pc = (idx - 392) % 4;
                mg_score(ROOK_PSQT[pr][pc])
            },
            424..456 => {
                let pr = (idx - 424) / 4;
                let pc = (idx - 424) % 4;
                eg_score(ROOK_PSQT[pr][pc])
            },
            456..488 => {
                let pr = (idx - 456) / 4;
                let pc = (idx - 456) % 4;
                mg_score(QUEEN_PSQT[pr][pc])
            },
            488..520 => {
                let pr = (idx - 488) / 4;
                let pc = (idx - 488) % 4;
                eg_score(QUEEN_PSQT[pr][pc])
            },
            520..552 => {
                let pr = (idx - 520) / 4;
                let pc = (idx - 520) % 4;
                mg_score(KING_PSQT[pr][pc])
            },
            552..584 => {
                let pr = (idx - 552) / 4;
                let pc = (idx - 552) % 4;
                eg_score(KING_PSQT[pr][pc])
            },
            _ => {0}
        }
    }
}

fn set_param(idx: usize, val: i32) {
    unsafe {
        // ...
        match idx {
            0 => { QUEEN_VALUE = set_mg_score(QUEEN_VALUE, val); },
            1 => { QUEEN_VALUE = set_eg_score(QUEEN_VALUE, val); },
            2 => { ROOK_VALUE = set_mg_score(ROOK_VALUE, val); },
            3 => { ROOK_VALUE = set_eg_score(ROOK_VALUE, val); },
            4 => { BISHOP_VALUE = set_mg_score(BISHOP_VALUE, val); },
            5 => { BISHOP_VALUE = set_eg_score(BISHOP_VALUE, val); },
            6 => { KNIGHT_VALUE = set_mg_score(KNIGHT_VALUE, val); },
            7 => { KNIGHT_VALUE = set_eg_score(KNIGHT_VALUE, val); },
            8 => { PAWN_VALUE = set_mg_score(PAWN_VALUE, val); },
            9 => { PAWN_VALUE = set_eg_score(PAWN_VALUE, val); },

            10..19 => { KNIGHT_MOBILITY[idx - 10] = set_mg_score(KNIGHT_MOBILITY[idx - 10], val); },
            19..28 => { KNIGHT_MOBILITY[idx - 19] = set_eg_score(KNIGHT_MOBILITY[idx - 19], val); },
            28..42 => { BISHOP_MOBILITY[idx - 28] = set_mg_score(BISHOP_MOBILITY[idx - 28], val); },
            42..56 => { BISHOP_MOBILITY[idx - 42] = set_eg_score(BISHOP_MOBILITY[idx - 42], val); },
            56..71 => { ROOK_MOBILITY[idx - 56] = set_mg_score(ROOK_MOBILITY[idx - 56], val); },
            71..86 => { ROOK_MOBILITY[idx - 71] = set_eg_score(ROOK_MOBILITY[idx - 71], val); },
            86..114 => { QUEEN_MOBILITY[idx - 86] = set_mg_score(QUEEN_MOBILITY[idx - 86], val); },
            114..142 => { QUEEN_MOBILITY[idx - 114] = set_eg_score(QUEEN_MOBILITY[idx - 114], val); },

            142..150 => { QUEEN_KING_DANGER[idx - 142] = val; },
            150..158 => { ROOK_KING_DANGER[idx - 150] = val; },
            158..166 => { BISHOP_KING_DANGER[idx - 158] = val; },
            166..174 => { KNIGHT_KING_DANGER[idx - 166] = val; },

            174 => { DOUBLE_BISHOP_BONUS = set_mg_score(DOUBLE_BISHOP_BONUS, val); },
            175 => { DOUBLE_BISHOP_BONUS = set_eg_score(DOUBLE_BISHOP_BONUS, val); },

            176..182 => { PASSED_PAWN_VALUE[idx - 175] = set_mg_score(PASSED_PAWN_VALUE[idx - 175], val); }, // skip 0th entry
            182..188 => { PASSED_PAWN_VALUE[idx - 181] = set_eg_score(PASSED_PAWN_VALUE[idx - 181], val); },
            188 => { CENTER_PAWN_VALUE = set_mg_score(CENTER_PAWN_VALUE, val); },
            189 => { CENTER_PAWN_VALUE = set_eg_score(CENTER_PAWN_VALUE, val); },
            190 => { ISOLATED_PAWN_VALUE = set_mg_score(ISOLATED_PAWN_VALUE, val); },
            191 => { ISOLATED_PAWN_VALUE = set_eg_score(ISOLATED_PAWN_VALUE, val); },
            192 => { DOUBLED_PAWN_VALUE = set_mg_score(DOUBLED_PAWN_VALUE, val); },
            193 => { DOUBLED_PAWN_VALUE = set_eg_score(DOUBLED_PAWN_VALUE, val); },
            194 => { BACKWARDS_PAWN_VALUE = set_mg_score(BACKWARDS_PAWN_VALUE, val); },
            195 => { BACKWARDS_PAWN_VALUE = set_eg_score(BACKWARDS_PAWN_VALUE, val); },
            196..202 => { ADVANCED_PAWN_VALUE[idx - 195] = set_mg_score(ADVANCED_PAWN_VALUE[idx - 195], val); }, // skip 0th entry
            202..208 => { ADVANCED_PAWN_VALUE[idx - 201] = set_eg_score(ADVANCED_PAWN_VALUE[idx - 201], val); },
            208 => { SUPPORTED_PAWN_BONUS = set_mg_score(SUPPORTED_PAWN_BONUS, val); },
            209 => { SUPPORTED_PAWN_BONUS = set_eg_score(SUPPORTED_PAWN_BONUS, val); },
            210 => { SPACE_VALUE = set_mg_score(SPACE_VALUE, val); },
            211 => { SPACE_VALUE = set_eg_score(SPACE_VALUE, val); },

            212 => { ROOK_ON_SEVENTH = set_mg_score(ROOK_ON_SEVENTH, val); },
            213 => { ROOK_ON_SEVENTH = set_eg_score(ROOK_ON_SEVENTH, val); },
            214 => { ROOK_ON_OPEN = set_mg_score(ROOK_ON_OPEN, val); },
            215 => { ROOK_ON_OPEN = set_eg_score(ROOK_ON_OPEN, val); },

            // pawn table is special, we only care about the middle 6
            216..240 => {
                let pr = (idx + 4 - 216) / 4;
                let pc = (idx + 4 - 216) % 4;
                PAWN_PSQT[pr][pc] = set_mg_score(PAWN_PSQT[pr][pc], val);
            },
            240..264 => {
                let pr = (idx + 4 - 240) / 4;
                let pc = (idx + 4 - 240) % 4;
                PAWN_PSQT[pr][pc] = set_eg_score(PAWN_PSQT[pr][pc], val);
            },
            264..296 => {
                let pr = (idx - 264) / 4;
                let pc = (idx - 264) % 4;
                KNIGHT_PSQT[pr][pc] = set_mg_score(KNIGHT_PSQT[pr][pc], val);
            },
            296..328 => {
                let pr = (idx - 296) / 4;
                let pc = (idx - 296) % 4;
                KNIGHT_PSQT[pr][pc] = set_eg_score(KNIGHT_PSQT[pr][pc], val);
            },
            328..360 => {
                let pr = (idx - 328) / 4;
                let pc = (idx - 328) % 4;
                BISHOP_PSQT[pr][pc] = set_mg_score(BISHOP_PSQT[pr][pc], val);
            },
            360..392 => {
                let pr = (idx - 360) / 4;
                let pc = (idx - 360) % 4;
                BISHOP_PSQT[pr][pc] = set_eg_score(BISHOP_PSQT[pr][pc], val);
            },
            392..424 => {
                let pr = (idx - 392) / 4;
                let pc = (idx - 392) % 4;
                ROOK_PSQT[pr][pc] = set_mg_score(ROOK_PSQT[pr][pc], val);
            },
            424..456 => {
                let pr = (idx - 424) / 4;
                let pc = (idx - 424) % 4;
                ROOK_PSQT[pr][pc] = set_eg_score(ROOK_PSQT[pr][pc], val);
            },
            456..488 => {
                let pr = (idx - 456) / 4;
                let pc = (idx - 456) % 4;
                QUEEN_PSQT[pr][pc] = set_mg_score(QUEEN_PSQT[pr][pc], val);
            },
            488..520 => {
                let pr = (idx - 488) / 4;
                let pc = (idx - 488) % 4;
                QUEEN_PSQT[pr][pc] = set_eg_score(QUEEN_PSQT[pr][pc], val);
            },
            520..552 => {
                let pr = (idx - 520) / 4;
                let pc = (idx - 520) % 4;
                KING_PSQT[pr][pc] = set_mg_score(KING_PSQT[pr][pc], val);
            },
            552..584 => {
                let pr = (idx - 552) / 4;
                let pc = (idx - 552) % 4;
                KING_PSQT[pr][pc] = set_eg_score(KING_PSQT[pr][pc], val);
            },
            _ => {}
        }
    }
}

fn get_value(node: &mut Bitboard) -> i32 {
    let multiplier = if node.side_to_move == Color::White {1} else {-1};
    return multiplier * static_eval(node);
}



fn tuning_qsearch(node: &mut Bitboard, alpha: i32, beta: i32) -> (i32, Bitboard) {
    if node.is_quiet() { return (static_eval(node), node.thread_copy()); }

    let mut alpha = alpha;

    let stand_pat = static_eval(node);

    let is_check = node.is_check(node.side_to_move);
    // standing pat check so we *do* stop eventually
    if !is_check {
        if stand_pat >= beta {
            return (stand_pat, node.thread_copy());
        } else if stand_pat > alpha {
            alpha = stand_pat;
        }
    }
    if stand_pat < alpha - 11000 && node.has_non_pawn_material() {
        return (stand_pat, node.thread_copy());
    }

    let mut best_val = stand_pat;
    let mut best_bb = node.thread_copy();

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
        let qresult = tuning_qsearch(node, -beta, -alpha);
        let val = -qresult.0;
        if val > best_val {
            best_val = val;
            best_bb = qresult.1
        }
        if val > alpha {
            alpha = val;
        }
        node.undo_move(&mv);
        if val >= beta {
            break;
        }
    }

    return (best_val, best_bb);
}

pub fn err(v: &mut Vec<(Bitboard, f64)>, k: f64) -> f64 {
    let mut err = 0.0;
    let n = v.len();
    for e in v {
        let score = static_eval(&mut e.0);
        let sigma = sigmoid(k, score);
        err += (e.1 - sigma).powi(2);
    }
    err /= n as f64;

    return err;
}

pub fn sigmoid(k: f64, eval: i32) -> f64 {
    let e: f64 = 2.71828;
    return 1.0 / (1.0 + e.powf(-k * eval as f64));
}

pub fn find_optimal_k(v: &mut Vec<(Bitboard, f64)>) -> f64 {
    // As of writing this comment, it appeared to be roughtly 0.000388
    let mut k = 0.000001;
    let mut best_k = k;
    let mut min_err = 1000000.;
    loop {
        let err = err(v, k);
        println!("{} {}", k, err);
        if err <= min_err {
            min_err = err;
            best_k = k;
        } else {
            break;
        }
        k += 0.000001;
    }
    return best_k;
}

const K: f64 = 0.000388;

pub fn score_positions(v: &mut Vec<(Bitboard, f64)>) -> i32 {
    let mut s = LB;
    for e in v {
        s = cmp::max(s, static_eval(&mut e.0));
    }
    return s;
}

pub fn neighbor(param_vec: &Vec<i32>, reach: f64) -> Vec<i32> {
    let mut new_params = param_vec.clone();

    // (minimum, maximum, scale @ zero reach, scale @ full reach)
    let mut dimens: Vec<(f64, f64, f64, f64)> = Vec::new();
    for i in 0..584 {
        let d = match i {
            0..2   => {(8000.0, 18000.0,  100.0, 2000.0)},
            2..4   => {(4000.0, 12000.0,   50.0, 1000.0)},
            4..6   => {(2500.0,  8000.0,   30.0, 1000.0)},
            6..8   => {(2500.0,  8000.0,   30.0, 1000.0)},
            8..10  => {( 800.0,  3000.0,   10.0,  200.0)},

            10..28  => {(-1000.0, 1500.0,   15.0,  200.0)},
            28..56  => {(-1000.0, 1500.0,   15.0,  200.0)},
            56..86  => {(-1000.0, 1500.0,   15.0,  200.0)},
            86..142 => {(-1000.0, 1500.0,   15.0,  200.0)},

            142..150 => {(   0.0, 1400.0,   15.0,  500.0)},
            150..158 => {(   0.0, 1400.0,   15.0,  500.0)},
            158..166 => {(   0.0, 1400.0,   15.0,  500.0)},
            166..174 => {(   0.0, 1400.0,   15.0,  500.0)},

            174..176 => {(   0.0, 1500.0,   30.0,  300.0)},

            176..188 => {(   0.0, 2000.0,   20.0,  200.0)},

            188..190 => {(    0.0, 1000.0,   20.0,  150.0)},
            190..196 => {(-1000.0,    0.0,   20.0,  150.0)},

            196..208 => {(    0.0,  500.0,   20.0,   100.0)},
            208..210 => {(    0.0,  300.0,   20.0,    70.0)},
            210..212 => {(    0.0,  300.0,   10.0,   100.0)},

            212..216 => {(    0.0,  500.0,   20.0,   100.0)},

            216..584 => {(-1000.0, 1000.0,   15.0,   200.0)},
            _ => {(0.0, 0.0, 0.0, 0.0)}
        };
        dimens.push(d);
    }

    let mut delta = [0f64; 584];
    let axes = (rand() % 8) + 1;
    for _ in 0..axes {
        delta[(rand() % 584) as usize] = symunif();
    }

    for idx in 0..584 {
        let (min, max, zero_scale, full_scale) = dimens[idx];
        let scale = zero_scale + (full_scale - zero_scale) * reach;
        let next = new_params[idx] + (delta[idx] * scale) as i32;
        new_params[idx] = cmp::min(max as i32, cmp::max(min as i32, next));
    }
    return new_params;
}

fn get_error(v: &mut Vec<(Bitboard, f64)>, params: &Vec<i32>) -> f64 {
    let old_params = get_params_vector();
    load_params_vector(params);
    let e = err(v, K);
    load_params_vector(&old_params);
    return e;
}

pub fn tune(v: &mut Vec<(Bitboard, f64)>) -> Vec<i32> {
    // returns the scoring vec
    let mut best = get_params_vector();
    let mut best_err = get_error(v, &best);
    let mut last_print = get_time_millis();

    let mut reach = 0.00;
    let mut counter: usize = 0;
    loop {
        let limit = (8192.0 * (1.0 + 7.0*reach*reach)) as usize;
        if counter > limit {
            counter = 0;
            reach += 0.25;
            if reach > 1.0 { break; }
        }
        if counter % 8 == 1 {
            println!("\x1B[1G{:>5}/{:<5}", counter, limit);
        }
        let next = neighbor(&best, reach);
        let next_err = get_error(v, &next);
        if next_err < best_err {
            counter = 0;
            best = next;
            best_err = next_err;
            println!("new best {}", best_err);
            if get_time_millis() - last_print > 5000 {
                print_params_vector(&best);
                last_print = get_time_millis();
            }
            if reach != 0.00 {reach = 0.0;}
        }
        counter += 1;
    }
    println!("DONE\n\n");
    print_params_vector(&best);
    return best;
}

pub fn get_position_vector(fname: &str) -> Vec<(Bitboard, f64)> {
    let f = match File::open(fname) {
        Ok(f) => f,
        Err(e) => panic!("unable to open file {}", fname)
    };

    let mut r = BufReader::new(f);

    let mut v: Vec<(Bitboard, f64)> = Vec::new();
    let mut buf = String::new();
    let mut idx = 0;
    loop {
        let num_bytes = match r.read_line(&mut buf) {
            Ok(n) => n, Err(e) => panic!("unable to read line")
        };
        if num_bytes == 0 { break; }
        if buf.len() > 0 {
            // fen winner
            if idx % 10 != 0 {
                idx += 1;
                buf.clear();
                continue;
            }

            idx += 1;
            let mut params = buf.trim().split_whitespace();
            let mut position_str = String::from("");
            for i in 0..6 {
                match params.next() {
                    Some(param) => {
                        if i != 0 {
                            position_str = format!("{} {}", position_str, param);
                        } else {
                            position_str = format!("{}", param);
                        }
                    },
                    None => {panic!("bad fen {}", idx)}
                }
            }
            let mut board = Bitboard::from_position(format!("{}", position_str));

            let winner: f64 = match params.next() {
                Some(p) => match p.trim().parse() {
                    Ok(num) => num,
                    Err(_) => panic!("error in winner parse")
                },
                None => panic!("empty winner!")
            };
            let qresult = tuning_qsearch(&mut board, LB, UB);
            let qboard = qresult.1;

            v.push((qboard, winner));
            buf.clear();
        }
    }

    return v;
}
