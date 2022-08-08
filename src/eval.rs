use std::cmp;

use crate::bitboard::*;
use crate::evalutil::*;
use crate::movegen::*;
use crate::pht::*;
use crate::psqt::*;
use crate::util::*;

pub type Score = i64;

// This is an idea I'm stealing from Stockfish's source
// and an older version of Ethereal
// essentially you store 2 scores
pub const fn make_score(mg_value: i32, eg_value: i32) -> Score {
    ((mg_value as i64) << 32) + (eg_value as i64)
}

pub fn mg_score(score: Score) -> i32 {
    // this is a quirk required to handle the way the
    // eg value was added in (particularly if it was a negative number)
    ((score + 0x80000000) >> 32) as i32
}

pub fn eg_score(score: Score) -> i32 {
    ((score << 32) >> 32) as i32
}

pub fn taper_score(s: Score, phase: i32) -> i32 {
    ((256 - phase) * mg_score(s) + phase * eg_score(s)) >> 8
}

// we're going to have large arrays and so on
// where "make_score" will become cumbersome
// we'll use the longer form where we can (e.g. in functions)
// but use "S" when it makes things easier to read
macro_rules! S {
    ($a:expr, $b:expr) => {
        make_score($a, $b)
    }
}

pub const QUEEN_VALUE: Score = S!(10949, 16252);
pub const ROOK_VALUE: Score = S!(4919, 8583);
pub const BISHOP_VALUE: Score = S!(3523, 5315);
pub const KNIGHT_VALUE: Score = S!(3466, 4834);
pub const PAWN_VALUE: Score = S!(841, 1440);

pub const KNIGHT_MOBILITY: [Score; 9] = [S!(-507, 133), S!(-112, 2), S!(-5, 326), S!(151, 520), S!(224, 528), S!(226, 538), S!(272, 528), S!(320, 427), S!(313, 359)];
pub const BISHOP_MOBILITY: [Score; 14] = [S!(-108, 52), S!(-26, -198), S!(275, -808), S!(271, -251), S!(448, -61), S!(440, 190), S!(474, 373), S!(571, 478), S!(559, 590), S!(553, 661), S!(551, 686), S!(567, 627), S!(449, 664), S!(1135, 336)];
pub const ROOK_MOBILITY: [Score; 15] = [S!(147, -2), S!(519, 255), S!(-154, -376), S!(-54, -73), S!(-33, 177), S!(9, 446), S!(-23, 627), S!(-24, 788), S!(19, 832), S!(60, 903), S!(113, 962), S!(170, 988), S!(262, 968), S!(288, 975), S!(541, 780)];
pub const QUEEN_MOBILITY: [Score; 28] = [S!(30, -158), S!(-204, 340), S!(-361, -96), S!(-231, -84), S!(6, -217), S!(89, -183), S!(87, 4), S!(145, -33), S!(193, -169), S!(191, 100), S!(225, 231), S!(222, 436), S!(239, 594), S!(239, 705), S!(299, 673), S!(260, 780), S!(284, 868), S!(265, 904), S!(223, 929), S!(257, 874), S!(159, 981), S!(260, 741), S!(224, 792), S!(-71, 938), S!(46, 830), S!(320, 565), S!(84, 759), S!(208, 699)];
pub const KNIGHT_OUTPOST_VALUE: Score = S!(318, 334);
pub const BISHOP_OUTPOST_VALUE: Score = S!(374, 73);
pub const BISHOP_LONG_DIAGONAL_VALUE: Score = S!(118, 120);

pub const PAWN_PROXIMITY_VALUE: [Score; 8] = [S!(110, 130), S!(-61, -71), S!(-124, 136), S!(-35, -10), S!(-44, -17), S!(59, -5), S!(-74, -270), S!(30, -15)];
pub const PAWN_SHELTER_VALUE: [[Score; 8]; 2] = [
    [S!(240, 25), S!(88, -11), S!(0, 57), S!(-120, -61), S!(-6, -251), S!(-13, -328), S!(-41, -70), S!(-268, 31)],
    [S!(-48, -119), S!(188, 31), S!(155, 34), S!(25, -55), S!(-80, -37), S!(19, -242), S!(-334, -176), S!(-266, 29)]
];
pub const PAWN_STORM_VALUE: [[Score; 8]; 2] = [
    [S!(516, 602), S!(871, 555), S!(-341, 140), S!(-154, -12), S!(6, -55), S!(43, -94), S!(-5, -66), S!(-17, -107)],
    [S!(2, -70), S!(16, 22), S!(-398, -195), S!(43, -190), S!(92, -110), S!(8, -154), S!(104, 142), S!(-55, -235)]
];

pub const QUEEN_KING_DANGER: [Score; 6] = [
    S!(0, 0), S!(95, 1), S!(314, 64), S!(662, 152), S!(938, 1005), S!(473, 563)
];
pub const ROOK_KING_DANGER: [Score; 6] = [
    S!(0, 0), S!(4, 113), S!(167, 48), S!(184, 147), S!(336, 211), S!(652, 668)
];
pub const BISHOP_KING_DANGER: [Score; 6] = [
    S!(0, 0), S!(51, 301), S!(207, 88), S!(235, 34), S!(114, 71), S!(171, 1)
];
pub const KNIGHT_KING_DANGER: [Score; 6] = [
    S!(0, 0), S!(1, 392), S!(178, 81), S!(332, 1), S!(62, 321), S!(22, 151)
];
pub const NO_QUEEN_ATTACK_VALUE: Score = S!(-276, -462);
pub const WEAK_SQUARE_VALUE: Score = S!(0, 0);
pub const QUEEN_CHECK_VALUE: Score = S!(241, 256);
pub const ROOK_CHECK_VALUE: Score = S!(436, 112);
pub const BISHOP_CHECK_VALUE: Score = S!(118, 131);
pub const KNIGHT_CHECK_VALUE: Score = S!(1000, 21);

pub const DOUBLE_BISHOP_BONUS: Score = S!(149, 978);

pub const PASSED_PAWN_VALUE: [Score; 8] = [
    S!(  0,   0), S!(0, 188), S!(0, 214), S!(0, 537), S!(260, 823), S!(661, 1261), S!(663, 1188), S!(0,   0)
];

pub const CANDIDATE_PASSED_PAWN_VALUE: [Score; 8] = [
    S!(  0,   0),S!(0, 0), S!(0, 0), S!(0, 191), S!(410, 506), S!(315, 604), S!(175, 245), S!(0,   0)
];
pub const CENTER_PAWN_VALUE: Score = S!(0, 1);
pub const ISOLATED_PAWN_VALUE: Score = S!(-33, -80);
pub const DOUBLED_PAWN_VALUE: Score = S!(0, -252);
pub const BACKWARDS_PAWN_VALUE: Score = S!(-45, -62);
pub const ADVANCED_PAWN_VALUE: [Score; 8] = [
    S!( 0,   0), S!(25, 0), S!(63, 43), S!(68, 69), S!(104, 156), S!(353, 302), S!(478, 414), S!( 0,  0)
];
pub const SUPPORTED_PAWN_BONUS: Score = S!(137, 64);
pub const SPACE_VALUE: Score = S!(16, 0);

pub const BISHOP_COLOR: Score = S!(-33, -111);

pub const TEMPO_BONUS: Score = S!(130, 130);

pub const ROOK_ON_SEVENTH: Score = S!(0, 74);
pub const ROOK_ON_OPEN: Score = S!(118, 90);

pub fn static_eval(pos: &mut Bitboard, pht: &mut PHT) -> i32 {
    let score = if pos.net.is_valid() {
            pos.net.nnue_eval()
    } else {
        evaluate_position(pos, pht)
    };

    return if pos.side_to_move == Color::White {score} else {-score};
}

fn halfmove_scale(score: i32, pos: &Bitboard) -> i32 {
    return ((100 - pos.halfmove as i32) * score) / 100;
}

pub fn evaluate_position(pos: &Bitboard, pht: &mut PHT) -> i32 {
    // positive is white-favored, negative black-favored
    let mut score: Score = make_score(0, 0);
    score += material_score(pos);
    score += mobility_and_king_danger(pos);
    score += pawn_value(pos, pht);
    score += double_bishop_bonus(pos);
    score += bishop_color_value(pos);
    score += rook_on_seventh_value(pos);
    score += rook_on_open_value(pos);
    score += nonpawn_psqt_value(pos);
    score += outpost_value(pos);
    score += king_pawns_value(pos);
    score += if pos.side_to_move == Color::White {TEMPO_BONUS} else {-TEMPO_BONUS};
    return halfmove_scale(taper_score(score, pos.get_phase()), pos);
}

fn pawnless_endgame_drawish(pos: &Bitboard) -> bool {
    // some endgames are known to be drawish
    // for these, until I can produce more intelligent
    // heuristics, the first pass is to *discourage*
    // mantissa from trading into these if she is otherwise
    // winning and encourage trading into them if she
    // is losing by supressing the material score in
    // these endgames, but leaving everything else functional
    // so that positional play still occurs.
    if pos.pawn[0] != 0 || pos.pawn[1] != 0 {
        return false;
    }

    if pos.queen[0] != 0 || pos.queen[1] != 0 {
        return false;
    }

    let kingless_composite = [pos.composite[0] & !pos.king[0], pos.composite[1] & !pos.king[1]];

    if kingless_composite[0].count_ones() > 2 || kingless_composite[1].count_ones() > 2 {
        return false;
    }

    if kingless_composite[0].count_ones() == 1 && kingless_composite[1].count_ones() == 1 {
        // one-on-one endgames
        // KR v K(B/N)
        for side in [Color::White, Color::Black] {
            let us = side as usize;
            let them = !side as usize;
            if kingless_composite[us] == pos.rook[us] &&
                (kingless_composite[them] == pos.bishop[them] | pos.knight[them]) {
                    return true;
                }
        }
    } else {
        for side in [Color::White, Color::Black] {
            let us = side as usize;
            let them = !side as usize;
            // KRN v KR
            // KRB v KR
            if pos.rook[us].count_ones() == 1 && pos.rook[them].count_ones() == 1 {
                if kingless_composite[them] == pos.rook[them] &&
                    kingless_composite[us] == pos.rook[us] | pos.bishop[us] | pos.knight[us] {
                        return true;
                    }
            }
        }
    }
    return false;
}

pub fn print_eval(board: &mut Bitboard) {
    let base_score = evaluate_position(board, &mut PHT::get_pht(1)) as f32 / 1000.0;

    eprint!("\x1B[0m");
    for _ in 0..24 { eprint!("\n"); }
    eprint!("\x1B[24A");
    for rank in (0..8).rev() {
        for file in 0..8 {
            let idx = rank*8 + file;

            let parity = (rank + file) % 2 == 0;
            let bg = if parity { (181, 135, 99) } else { (212, 190, 154) };
            eprint!("\x1B[48;2;{};{};{}m", bg.0, bg.1, bg.2);
            eprint!("       \x1B[7D\x1B[B");

            let mut piece = board.piece_at_square(idx, Color::White);
            let mut color = Color::White;
            if piece == 0 {
                piece = board.piece_at_square(idx, Color::Black);
                color = Color::Black;
            }
            if piece == 0 {
                eprint!("       \x1B[7D\x1B[B");
                eprint!("       \x1B[7D");
                eprint!("\x1B[2A\x1B[7C");
                continue;
            }

            match color {
                Color::White => eprint!("\x1B[97m"),
                Color::Black => eprint!("\x1B[30m")
            };
            eprint!("   {}   \x1B[7D\x1B[B", (piece - 32) as char);
            if piece == b'k' {
                eprint!("       \x1B[7D");
            } else {
                let side = color as usize;
                board.composite[side] ^= idx_to_bb(idx);
                match piece {
                    b'p' => {board.pawn[side] ^= idx_to_bb(idx);},
                    b'n' => {board.knight[side] ^= idx_to_bb(idx);},
                    b'b' => {board.bishop[side] ^= idx_to_bb(idx);},
                    b'r' => {board.rook[side] ^= idx_to_bb(idx);},
                    b'q' => {board.queen[side] ^= idx_to_bb(idx);},
                    _ => {panic!("there is no piece here")}
                }

                // self.deactivate(piece_num, color, idx);
                let hypothetical_score = evaluate_position(board, &mut PHT::get_pht(1)) as f32 / 1000.0;
                let ofs = base_score - hypothetical_score;

                board.composite[side] ^= idx_to_bb(idx);
                match piece {
                    b'p' => {board.pawn[side] ^= idx_to_bb(idx);},
                    b'n' => {board.knight[side] ^= idx_to_bb(idx);},
                    b'b' => {board.bishop[side] ^= idx_to_bb(idx);},
                    b'r' => {board.rook[side] ^= idx_to_bb(idx);},
                    b'q' => {board.queen[side] ^= idx_to_bb(idx);},
                    _ => {panic!("there is no piece here")}
                }

                if ofs.abs() >= 10.0 {
                    eprint!(" {:+5.1} \x1B[7D", ofs);
                }
                else {
                    eprint!(" {:+5.2} \x1B[7D", ofs);
                }
            }
            eprint!("\x1B[2A\x1B[7C");
        }
        eprint!("\x1B[0m\r\x1B[3B");
    }
    eprintln!("Classical evaluation (White View): {:+.2}", base_score);
}
// fn king_distance(k1: i8, k2: i8) -> i32 {
//     let (k1f, k1r) = idx_to_coord(k1);
//     let (k2f, k2r) = idx_to_coord(k2);
//     return cmp::max((k1f - k2f).abs(), (k1r - k2f).abs()) as i32;
// }

// fn edge_distance(idx: i8) -> i32 {
//     let (f, r) = idx_to_coord(idx);
//     return cmp::min(7 - f, f, 7 - r, r) as i32;
// }

// fn corner_manhattan_distance(idx: i8) -> i32 {
//     let (f, r) = idx_to_coord(idx);
//     let f_dist = cmp::min(7 - f, f);
//     let r_dist = cmp::min(7 - r, r);
//     return (f_dist + r_dist) as i32;
// }

// fn corner_distance(idx: i8) -> i32 {
//     let (f, r) = idx_to_coord(idx);
//     let f_dist = cmp::min(7 - f, f);
//     let r_dist = cmp::min(7 - r, r);
//     return cmp::max(f_dist, r_dist) as i32;
// }

// pub fn is_known_win_endgame(pos: &Bitboard) -> i32 {
//     // for now, we'll only add KX v K endgames
// }

// pub fn known_win_endgame(pos: &Bitboard) -> i32 {

// }

pub fn material_score(pos: &Bitboard) -> Score {
    let mut score: Score = make_score(0, 0);
    if pawnless_endgame_drawish(pos) { return score; }
    let white = Color::White as usize;
    let black = Color::Black as usize;

    score += QUEEN_VALUE * pos.queen[white].count_ones() as Score;
    score -= QUEEN_VALUE * pos.queen[black].count_ones() as Score;

    score += ROOK_VALUE * pos.rook[white].count_ones() as Score;
    score -= ROOK_VALUE * pos.rook[black].count_ones() as Score;

    score += BISHOP_VALUE * pos.bishop[white].count_ones() as Score;
    score -= BISHOP_VALUE * pos.bishop[black].count_ones() as Score;

    score += KNIGHT_VALUE * pos.knight[white].count_ones() as Score;
    score -= KNIGHT_VALUE * pos.knight[black].count_ones() as Score;

    score += PAWN_VALUE * pos.pawn[white].count_ones() as Score;
    score -= PAWN_VALUE * pos.pawn[black].count_ones() as Score;

    return score;
}

pub fn simple_material_score(pos: &Bitboard) -> Score {
    let mut score = 0;
    let white = Color::White as usize;
    let black = Color::Black as usize;

    score += 9000 * pos.queen[white].count_ones() as Score;
    score -= 9000 * pos.queen[black].count_ones() as Score;

    score += 5000 * pos.rook[white].count_ones() as Score;
    score -= 5000 * pos.rook[black].count_ones() as Score;

    score += 3000 * pos.bishop[white].count_ones() as Score;
    score -= 3000 * pos.bishop[black].count_ones() as Score;

    score += 3000 * pos.knight[white].count_ones() as Score;
    score -= 3000 * pos.knight[black].count_ones() as Score;

    score += 1000 * pos.pawn[white].count_ones() as Score;
    score -= 1000 * pos.pawn[black].count_ones() as Score;

    return score;
}

fn pawn_attacks(pawn_bb: u64, side_to_move: Color) -> u64 {
    if side_to_move == Color::White {
        ((pawn_bb & !FILE_MASKS[0]) << 7) | ((pawn_bb & !FILE_MASKS[7]) << 9)
    } else {
        ((pawn_bb & !FILE_MASKS[0]) >> 9) | ((pawn_bb & !FILE_MASKS[7]) >> 7)
    }
}

pub fn pawn_structure_value(pos: &Bitboard) -> Score {
    let white = Color::White;
    let black = Color::Black;
    let mut pawn_score: [Score; 2] = [0, 0];

    for side in [white, black] {
        let me = side as usize;
        let them = !side as usize;
        let forward = if side == Color::White {8} else {-8};

        let my_pawns = pos.pawn[me];
        let their_pawns = pos.pawn[them];

        let mut pawn_bb = my_pawns;

        while pawn_bb != 0 {
            let idx = pawn_bb.trailing_zeros() as i8;
            let this_pawn = idx_to_bb(idx);
            let this_pawn_pushed = idx_to_bb(idx + forward);

            let r = (idx / 8) as usize;
            let f = (idx % 8) as usize;
            let score_r = if side == Color::White {r} else {7-r};

            // This particular organization of bb's I learned from Ethereal
            // I'll try to give explanations to the importance of each though

            // friendly pawns on adjacent files
            let neighbors = my_pawns & unsafe{ADJACENT_FILE_MASKS[f]};
            // friendly pawns that could conceivably walk up to aid in this pawn's advance
            let backup = my_pawns & unsafe{PASSED_PAWN_MASKS[them][idx as usize]};
            // enemy pawns preventing this pawn from being considered passed
            let stoppers = their_pawns & unsafe{PASSED_PAWN_MASKS[me][idx as usize]};
            // pawns that attack this pawn
            let threats = their_pawns & pawn_attacks(this_pawn, side);
            // pawns that defend this pawn
            let support = my_pawns & pawn_attacks(this_pawn, !side);
            // pawns that attack the stop square of this pawn
            let push_threats = their_pawns & pawn_attacks(this_pawn_pushed, side);
            // pawns that would defend this pawn if it pushed
            let push_support = my_pawns & pawn_attacks(this_pawn_pushed, !side);
            // pawns that are stoppers which aren't one of the immediate threats
            let leftover_stoppers = stoppers & !(threats | push_threats);
            // friendly pawns in front of this pawn
            let own_blockers = my_pawns & unsafe{FILE_MASKS[f] & AHEAD_RANK_MASKS[me][r]};
            // enemy pawns in front of this pawn
            let enemy_blockers = their_pawns & unsafe{FILE_MASKS[f] & AHEAD_RANK_MASKS[me][r]};


            // passed pawns
            if stoppers == 0 { pawn_score[me] += PASSED_PAWN_VALUE[score_r];}

            // candidate passed pawns
            // we do a brief calculation to see if we have enough supporters
            // to push by any final, immediate threats by trading off all
            // supporters and threateners and seeing if we come out on top.
            else if leftover_stoppers == 0 && push_support.count_ones() >= push_threats.count_ones() {
                if support.count_ones() >= threats.count_ones() {
                    pawn_score[me] += CANDIDATE_PASSED_PAWN_VALUE[score_r];
                }
            }

            // Isolated pawns
            if neighbors == 0 && threats == 0 {
                pawn_score[me] += ISOLATED_PAWN_VALUE;
            }

            // doubled pawns
            if own_blockers != 0 {
                pawn_score[me] += DOUBLED_PAWN_VALUE;
            }

            // backwards pawns
            if push_threats != 0 && backup == 0 && push_support == 0 {
                pawn_score[me] += BACKWARDS_PAWN_VALUE;
            }

            // connected pawns
            // Specific implementation here from the SF evaluation guide
            let supported_count = support.count_ones() as i32;
            let phalanx = if push_support != 0 {1} else {0};
            let opposed = if enemy_blockers != 0 {1} else {0};

            if supported_count != 0 || phalanx != 0 {
                pawn_score[me] += ADVANCED_PAWN_VALUE[score_r] * (2 + phalanx - opposed) as i64;
                pawn_score[me] += SUPPORTED_PAWN_BONUS * supported_count as i64;
            }

            // pop off this pawn
            pawn_bb &= pawn_bb - 1;
        }
    }

    return pawn_score[1] - pawn_score[0];
}

fn is_outpost(pos: &Bitboard, idx: i8, side: Color) -> bool {
    let bounds = if side == Color::White {(4, 6)} else {(1, 3)};
    let r = idx / 8;
    let f = idx % 8;
    if r < bounds.0 || r > bounds.1 { return false; }

    let enemy_pawns = pos.pawn[(!side) as usize];
    let my_pawns = pos.pawn[side as usize];
    let support = my_pawns & pawn_attacks(idx_to_bb(idx), !side);
    if support == 0 { return false; }

    let atk_span = unsafe{ADJACENT_FILE_MASKS[f as usize] & AHEAD_RANK_MASKS[side as usize][r as usize]};
    return (atk_span & enemy_pawns) == 0;
}

fn outpost_value(pos: &Bitboard) -> Score {
    let mut outpost = [0, 0];
    for side in [Color::White, Color::Black] {
        let me = side as usize;

        let mut bishop_bb = pos.bishop[me];
        let mut knight_bb = pos.knight[me];

        while bishop_bb != 0 {
            let idx = bishop_bb.trailing_zeros() as i8;
            if is_outpost(pos, idx, side) {
                // println!("bishop outpost at {}", idx_to_str(idx));
                outpost[me] += BISHOP_OUTPOST_VALUE;
            }

            // pop bishop
            bishop_bb &= bishop_bb - 1;
        }

        while knight_bb != 0 {
            let idx = knight_bb.trailing_zeros() as i8;
            if is_outpost(pos, idx, side) {
                // println!("knight outpost at {}", idx_to_str(idx));
                outpost[me] += KNIGHT_OUTPOST_VALUE;
            }

            // pop knight
            knight_bb &= knight_bb - 1;
        }
    }
    return outpost[1] - outpost[0];
}

fn king_pawns_value(pos: &Bitboard) -> Score {
    // credit to SF Evaluation Guide
    // and Ethereal Source
    let mut pawn_proximity: [Score; 2] = [0, 0];
    let mut pawn_shelter: [Score; 2] = [0, 0];
    let mut pawn_storm: [Score; 2] = [0, 0];

    for side in [Color::White, Color::Black] {
        let me = side as usize;
        let them = !side as usize;
        let king_idx = pos.king[me].trailing_zeros() as i8;
        let king_rank = king_idx / 8;
        let king_file = king_idx % 8;

        // first find the nearest file-wise pawn
        let pawns = pos.pawn[me] | pos.pawn[them];
        if pawns != 0 {
            let mut pawn_distance = 0;
            for i in 0..7 {
                let mut mask = 0;
                if king_file - i >= 0 {
                    mask |= FILE_MASKS[(king_file - i) as usize];
                }
                if king_file + i < 8 {
                    mask |= FILE_MASKS[(king_file + i) as usize];
                }
                if mask & pawns != 0 {
                    pawn_distance = i as usize;
                    // println!("pawn distance for side {} is {}", side as i8, pawn_distance);
                    break;
                }
            }

            pawn_proximity[me] += PAWN_PROXIMITY_VALUE[pawn_distance];
        }

        // king shelter and storm
        let bounds = (cmp::max(king_file - 1, 0), cmp::min(king_file + 2, 8));
        let at_or_above_mask = unsafe{AHEAD_RANK_MASKS[me][king_rank as usize] | RANK_MASKS[king_rank as usize]};
        let my_pawns = pos.pawn[me];
        let their_pawns = pos.pawn[them];

        for file in bounds.0..bounds.1 {
            let mask = FILE_MASKS[file as usize] & at_or_above_mask;
            // println!("checking file {}", file);

            // closest friendly pawn at or above
            let friendly_pawn = my_pawns & mask;
            let friendly_distance = if friendly_pawn != 0 {
                (king_rank - (friendly_pawn.trailing_zeros() as i8 / 8)).abs()
            } else {
                7
            };

            // println!("friendly distance {}", friendly_distance);

            // closest enemy pawn at or above
            let enemy_pawn = their_pawns & mask;
            let enemy_distance = if enemy_pawn != 0 {
                (king_rank - (enemy_pawn.trailing_zeros() as i8 / 8)).abs()
            } else {
                7
            };

            // println!("enemy distance {}", enemy_distance);

            // going a bit more basic than Ethereal for now
            // just consider whether this is the king file and the distance
            pawn_shelter[me] += PAWN_SHELTER_VALUE[(file == king_file) as usize][friendly_distance as usize];

            // check if we have a pawn in the way before updating storm score
            let blocked = friendly_distance == enemy_distance - 1;
            // println!("blocked {}", blocked);
            pawn_storm[me] += PAWN_STORM_VALUE[blocked as usize][enemy_distance as usize];
        }
    }

    return (pawn_shelter[1] + pawn_storm[1]) - (pawn_shelter[0] + pawn_storm[0]);
}

fn mobility_and_king_danger(pos: &Bitboard) -> Score {
    let mut mobility: Score = make_score(0, 0);
    let mut piece_bonus: [Score; 2] = [0, 0];
    let mut king_danger: [Score; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let occ = pos.composite[white] | pos.composite[black];

    let center_diagonal_1 = idx_to_bb(27) | idx_to_bb(36);
    let center_diagonal_2 = idx_to_bb(28) | idx_to_bb(35);

    let mut attacked: [u64; 2] = [pawn_attacks(pos.pawn[black], Color::Black), pawn_attacks(pos.pawn[white], Color::White)];
    let mut attacked_by_queens: [u64; 2] = [0, 0];
    let mut attacked_by_rooks: [u64; 2] = [0, 0];
    let mut attacked_by_bishops: [u64; 2] = [0, 0];
    let mut attacked_by_knights: [u64; 2] = [0, 0];
    let mut attacked_by_two: [u64; 2] = [0, 0];

    for side in [white, black] {
        let other_side = if side == white {black} else {white};
        let multiplier = if side == white {1} else {-1};

        let king_bb = pos.king[other_side];
        let mut attackers = 0;
        let mut attack_value: i64 = 0;
        let king_idx = king_bb.trailing_zeros() as i8;
        let king_zone = king_bb | unsafe{ KING_MASK[king_idx as usize] };

        let mut queen_attacks = 0;
        let mut rook_attacks = 0;
        let mut bishop_attacks = 0;
        let mut knight_attacks = 0;

        let mut board = pos.queen[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let move_board = queen_moves_board(start_idx, occ);
            let moves = move_board.count_ones() as usize;

            attacked_by_two[side] |= attacked[side] & move_board;
            attacked[side] |= move_board;
            attacked_by_queens[side] |= move_board;
            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                queen_attacks += attacks.count_ones();
            }
            mobility += multiplier * QUEEN_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.rook[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let move_board = rook_moves_board(start_idx, occ & !(pos.queen[side] | pos.rook[side]));
            let moves = move_board.count_ones() as usize;

            attacked_by_two[side] |= attacked[side] & move_board;
            attacked[side] |= move_board;
            attacked_by_rooks[side] |= move_board;
            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                rook_attacks += attacks.count_ones();
            }
            mobility += multiplier * ROOK_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.bishop[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let move_board = bishop_moves_board(start_idx, occ & !(pos.queen[side] | pos.bishop[side]));

            if move_board & center_diagonal_1 == center_diagonal_1 { piece_bonus[side] += BISHOP_LONG_DIAGONAL_VALUE; }
            else if move_board & center_diagonal_2 == center_diagonal_2 { piece_bonus[side] += BISHOP_LONG_DIAGONAL_VALUE; }

            let moves = move_board.count_ones() as usize;
            attacked_by_two[side] |= attacked[side] & move_board;
            attacked[side] |= move_board;
            attacked_by_bishops[side] |= move_board;

            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                bishop_attacks += attacks.count_ones();
            }
            mobility += multiplier * BISHOP_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.knight[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let enemy = if side == white {black} else {white};
            let move_board = knight_moves_board(start_idx) & !pawn_attacks(pos.pawn[enemy], !pos.side_to_move);
            let moves = move_board.count_ones() as usize;
            attacked_by_two[side] |= attacked[side] & move_board;
            attacked[side] |= move_board;
            attacked_by_knights[side] |= move_board;
            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                knight_attacks += attacks.count_ones();
            }
            mobility += multiplier * KNIGHT_MOBILITY[moves];
            board &= board - 1;
        }

        if attackers > 5 { attackers = 5; }
        // println!("num attackers side {}: {}", side, attackers);
        if attackers >= if pos.queen[side] != 0 {1} else {2} {
            attack_value += QUEEN_KING_DANGER[attackers] * queen_attacks as i64;
            attack_value += ROOK_KING_DANGER[attackers] * rook_attacks as i64;
            attack_value += BISHOP_KING_DANGER[attackers] * bishop_attacks as i64;
            attack_value += KNIGHT_KING_DANGER[attackers] * knight_attacks as i64;

            king_danger[side] = attack_value;
        }

        // if piece_bonus[side] != 0 {println!("LONG DIAGONAL");}

    }

    let total_occ = pos.composite[1] | pos.composite[0];
    // safe checks.
    for side in [white, black] {
        let me = side;
        let them = if side == white {black} else {white};
        let weak_squares = attacked[me] & !attacked_by_two[them] & (!attacked[them] | attacked_by_queens[them]);
        let safe_squares = !pos.composite[me] & (!attacked[them] | (attacked_by_two[me] & weak_squares));

        let king_bb = pos.king[them];
        let king_idx = king_bb.trailing_zeros() as i8;

        let king_rook_threats = rook_moves_board(king_idx, total_occ);
        let king_bishop_threats = bishop_moves_board(king_idx, total_occ);
        let king_queen_threats = king_rook_threats | king_bishop_threats;
        let king_knight_threats = knight_moves_board(king_idx);

        let queen_checks = attacked_by_queens[me] & safe_squares & king_queen_threats;
        let rook_checks = attacked_by_rooks[me] & safe_squares & king_rook_threats;
        let bishop_checks = attacked_by_bishops[me] & safe_squares & king_bishop_threats;
        let knight_checks = attacked_by_knights[me] & safe_squares & king_knight_threats;

        if king_danger[side] != 0 {
            // println!("safe queen checks side {}: {}", side, queen_checks.count_ones());
            // println!("safe rook checks side {}: {}", side, rook_checks.count_ones());
            // println!("safe bishop checks side {}: {}", side, bishop_checks.count_ones());
            // println!("safe knight checks side {}: {}", side, knight_checks.count_ones());
            // println!("enemy weak squares side {}: {}", side, weak_squares.count_ones());

            king_danger[side] += queen_checks.count_ones() as i64 * QUEEN_CHECK_VALUE;
            king_danger[side] += rook_checks.count_ones() as i64 * ROOK_CHECK_VALUE;
            king_danger[side] += bishop_checks.count_ones() as i64 * BISHOP_CHECK_VALUE;
            king_danger[side] += knight_checks.count_ones() as i64 * KNIGHT_CHECK_VALUE;

            king_danger[side] += WEAK_SQUARE_VALUE * weak_squares.count_ones() as i64;
            if pos.queen[side] == 0 {
                king_danger[side] += NO_QUEEN_ATTACK_VALUE;
            }
            // println!("king danger! {}", mg_score(king_danger[side]));
        }
    }

    let score = mobility + (piece_bonus[white] - piece_bonus[black]) + (king_danger[white] - king_danger[black]);
    return score;
}

fn double_bishop_bonus(pos: &Bitboard) -> Score {
    let mut score: Score = make_score(0, 0);
    let white = Color::White as usize;
    let black = Color::Black as usize;
    if pos.bishop[white].count_ones() >= 2 {
        score += DOUBLE_BISHOP_BONUS;
    }
    if pos.bishop[black].count_ones() >= 2 {
        score -= DOUBLE_BISHOP_BONUS;
    }

    return score;
}

pub fn print_value(pos: &Bitboard) {
    eprintln!("HCE Score Breakdown:");
    eprintln!("material: {}", taper_score(material_score(pos), pos.get_phase()));
    eprintln!("mobility and king_danger: {}", taper_score(mobility_and_king_danger(pos), pos.get_phase()));
    eprintln!("passed_pawns: {}", taper_score(passed_pawns_value(pos), pos.get_phase()));
    eprintln!("center_pawns: {}", taper_score(center_pawns_value(pos), pos.get_phase()));
    eprintln!("isolated_pawns: {}", taper_score(isolated_pawns_value(pos), pos.get_phase()));
    eprintln!("doubled_pawns: {}", taper_score(doubled_pawns_value(pos), pos.get_phase()));
    eprintln!("backwards_pawns: {}", taper_score(backwards_pawns_value(pos), pos.get_phase()));
    eprintln!("connected_pawns: {}", taper_score(connected_pawns_value(pos), pos.get_phase()));
    eprintln!("space: {}", taper_score(space_control_value(pos), pos.get_phase()));
    eprintln!("rook on 7th: {}", taper_score(rook_on_seventh_value(pos), pos.get_phase()));
    eprintln!("rook on open: {}", taper_score(rook_on_open_value(pos), pos.get_phase()));
    eprintln!("double_bishop_bonus: {}", taper_score(double_bishop_bonus(pos), pos.get_phase()));
    eprintln!("bishop_color: {}", taper_score(bishop_color_value(pos), pos.get_phase()));
    eprintln!("psqt: {}", taper_score(nonpawn_psqt_value(pos) + pawn_psqt_value(pos), pos.get_phase()));
}

fn pawn_value(pos: &Bitboard, pht: &mut PHT) -> Score {
    // let pht;
    // unsafe {
    //     pht = &mut PHT;
    // }
    let mut val: Score = 0;
    let pht_entry = pht.get(pos.pawn_hash);
    if pht_entry.valid {
        val = pht_entry.value;
    } else {
        // val += passed_pawns_value(pos);
        val += center_pawns_value(pos);
        // val += isolated_pawns_value(pos);
        // val += doubled_pawns_value(pos);
        // val += backwards_pawns_value(pos);
        // val += connected_pawns_value(pos);
        // val += space_control_value(pos);
        val += pawn_structure_value(pos);
        val += pawn_psqt_value(pos);
        pht.set(pos.pawn_hash, val);
    }
    return val;
}

fn passed_pawns_value(pos: &Bitboard) -> Score {
    let mut passed_pawns: [Score; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;

    for side in [white, black] {
        for f in 0..8 {
            let f = f as usize;
            let mask = FILE_MASKS[f];
            let file_pawns = mask & pos.pawn[side];
            if file_pawns != 0 {
                let mut enemy_mask: u64 = 0;
                if f > 0 {
                    enemy_mask |= FILE_MASKS[(f - 1) as usize];
                }
                if f < 7 {
                    enemy_mask |= FILE_MASKS[(f + 1) as usize];
                }
                enemy_mask |= mask;
                let mut rank_mask = 0;
                let mut pp_rank = 0;

                for r in 0..8 {
                    let r: usize = if side == white {7-r} else {r};
                    if RANK_MASKS[r] & file_pawns == 0 {
                        rank_mask |= RANK_MASKS[r];
                    } else {
                        if side == white {
                            pp_rank = r;
                        } else {
                            pp_rank = 7 - r;
                        }
                        break;
                    }
                }
                enemy_mask &= rank_mask;

                if enemy_mask & pos.pawn[if side == white {black} else {white}] == 0 {
                    passed_pawns[side] += PASSED_PAWN_VALUE[pp_rank as usize];
                }
            }
        }
    }
    return passed_pawns[white] - passed_pawns[black];
}

fn center_pawns_value(pos: &Bitboard) -> Score {
    let white_center = (CENTER_MASK & pos.pawn[Color::White as usize]).count_ones() as i32;
    let black_center = (CENTER_MASK & pos.pawn[Color::Black as usize]).count_ones() as i32;
    return CENTER_PAWN_VALUE * (white_center - black_center) as i64;
}

fn isolated_pawns_value(pos: &Bitboard) -> Score {
    let mut isolated_pawns: [i32; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;

    for side in [white, black] {
        for f in 0..8 {
            let mask = FILE_MASKS[f as usize];
            let file_pawns = mask & pos.pawn[side];
            let mut neighbor_files: u64 = 0;
            if file_pawns != 0 {
                if f > 0 {
                    neighbor_files |= FILE_MASKS[(f - 1) as usize];
                }
                if f < 7 {
                    neighbor_files |= FILE_MASKS[(f + 1) as usize];
                }
                if neighbor_files & pos.pawn[side] == 0 {
                    isolated_pawns[side] += 1;
                }
            }
        }
    }
    return ISOLATED_PAWN_VALUE * (isolated_pawns[white] - isolated_pawns[black]) as i64;
}

fn doubled_pawns_value(pos: &Bitboard) -> Score {
    let mut doubled_pawns: [i32; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;

    for side in [white, black] {
        for f in 0..8 {
            let f = f as usize;
            let mask = FILE_MASKS[f];
            let file_pawns = mask & pos.pawn[side];
            if file_pawns.count_ones() > 1 {
                doubled_pawns[side] += file_pawns.count_ones() as i32 - 1;
            }
        }
    }

    return DOUBLED_PAWN_VALUE * (doubled_pawns[white] - doubled_pawns[black]) as i64;
}

fn backwards_pawns_value(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let white_atks = pawn_attacks(pos.pawn[white], Color::White);
    let black_atks = pawn_attacks(pos.pawn[black], Color::Black);

    let mut white_atk_proj = white_atks;
    let mut black_atk_proj = black_atks;
    for _ in 0..5 {
        white_atk_proj |= white_atk_proj << 8;
        black_atk_proj |= black_atk_proj >> 8;
    }

    let white_backwards_pawns = (pos.pawn[white] << 8) & black_atks & !white_atk_proj;
    let black_backwards_pawns = (pos.pawn[black] << 8) & white_atks & !black_atk_proj;
    let backwards_pawn_balance = white_backwards_pawns.count_ones() as i32 - black_backwards_pawns.count_ones() as i32;

    return BACKWARDS_PAWN_VALUE * backwards_pawn_balance as i64;
}

fn connected_pawns_value(pos: &Bitboard) -> Score {
    let mut connected_pawns: [Score; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;

    for color in [Color::White, Color::Black] {
        let me = color as usize;
        let them = !color as usize;
        let mut pawn_bb = pos.pawn[me];
        let my_atks = pawn_attacks(pos.pawn[me], color);
        while pawn_bb != 0 {
            let idx = pawn_bb.trailing_zeros() as i8;
            pawn_bb &= pawn_bb - 1;

            // supported?
            // aka is it protected by another pawn
            let supported = idx_to_bb(idx) & my_atks != 0;
            if supported {
                connected_pawns[me] += SUPPORTED_PAWN_BONUS;
            }

            // part of a phalanx?
            // aka is its stop square covered by one of its neighbors?
            let phalanx = if color == Color::White {idx_to_bb(idx) << 8} else {idx_to_bb(idx) >> 8} & my_atks != 0;
            if phalanx || supported {
                let rank = if color == Color::White { idx / 8 } else { 7 - (idx / 8) } as usize;
                let file = (idx % 8) as usize;
                connected_pawns[me] += ADVANCED_PAWN_VALUE[rank];
                if phalanx {
                    connected_pawns[me] += ADVANCED_PAWN_VALUE[rank];
                }
                if FILE_MASKS[file] & pos.pawn[them] == 0 {
                    connected_pawns[me] += ADVANCED_PAWN_VALUE[rank];
                }
            }
        }
    }

    return connected_pawns[white] - connected_pawns[black];
}

fn space_control_value(pos: &Bitboard) -> Score {
    let mut space_control: [i32; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let white_atks = pawn_attacks(pos.pawn[white], Color::White);
    let black_atks = pawn_attacks(pos.pawn[black], Color::Black);

    let white_space_mask = (FILE_MASKS[2] | FILE_MASKS[3] | FILE_MASKS[4] | FILE_MASKS[5])
                         & (RANK_MASKS[1] | RANK_MASKS[2] | RANK_MASKS[3]);

    let black_space_mask = (FILE_MASKS[2] | FILE_MASKS[3] | FILE_MASKS[4] | FILE_MASKS[5])
                         & (RANK_MASKS[6] | RANK_MASKS[5] | RANK_MASKS[4]);

    let base_white_space = white_space_mask & !black_atks & !pos.pawn[white];
    let base_black_space = black_space_mask & !white_atks & !pos.pawn[black];

    // bonus for sheltering behing a pawn
    let bonus_white_space = (pos.pawn[white] >> 8 | pos.pawn[white] >> 16) & base_white_space;
    let bonus_black_space = (pos.pawn[black] << 8 | pos.pawn[black] << 16) & base_black_space;

    space_control[white] = (base_white_space.count_ones() + bonus_white_space.count_ones()) as i32;
    space_control[black] = (base_black_space.count_ones() + bonus_black_space.count_ones()) as i32;

    return SPACE_VALUE * (space_control[white] - space_control[black]) as i64;
}

fn rook_on_seventh_value(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let white_seventh_rooks = (pos.rook[white] & RANK_MASKS[6]).count_ones() as i32;
    let black_seventh_rooks = (pos.rook[black] & RANK_MASKS[1]).count_ones() as i32;

    let white_condition = (pos.king[black] & RANK_MASKS[7]) != 0 || (pos.pawn[black] & RANK_MASKS[6]) != 0;
    let black_condition = (pos.king[white] & RANK_MASKS[7]) != 0 || (pos.pawn[white] & RANK_MASKS[6]) != 0;

    return ROOK_ON_SEVENTH
        * (if white_condition {white_seventh_rooks} else {0}
         - if black_condition {black_seventh_rooks} else {0}) as i64;
}

fn rook_on_open_value(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let mut openish_file_rooks: [i32; 2] = [0, 0];
    for side in [white, black] {
        let enemy_side = if side == white {black} else {white};
        let mut rook_bb = pos.rook[side];
        while rook_bb != 0 {
            let idx = rook_bb.trailing_zeros() as i8;
            rook_bb &= rook_bb - 1;

            let f = (idx % 8) as usize;
            let pawns = FILE_MASKS[f] & pos.pawn[side];
            if pawns != 0 { continue; }

            if (FILE_MASKS[f] & pos.pawn[enemy_side]) == 0 {
                // open
                openish_file_rooks[side] += 2;
            } else {
                openish_file_rooks[side] += 1;
            }
        }
    }

    return ROOK_ON_OPEN * (openish_file_rooks[white] - openish_file_rooks[black]) as i64;
}

pub fn bishop_color_value(pos: &Bitboard) -> Score {
    let mut bishop_color: [i32; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;

    let dark_tiles_mask =
        0b10101010_01010101_10101010_01010101_10101010_01010101_10101010_01010101;
    let light_tiles_mask = !dark_tiles_mask;

    for side in [white, black] {
        let bishops_on_dark = (pos.bishop[side] & dark_tiles_mask).count_ones() as i32;
        let bishops_on_light = (pos.bishop[side] & light_tiles_mask).count_ones() as i32;

        let pawns_on_dark = (pos.pawn[side] & dark_tiles_mask).count_ones() as i32;
        let pawns_on_light = (pos.pawn[side] & light_tiles_mask).count_ones() as i32;

        bishop_color[side] = bishops_on_dark * pawns_on_dark + bishops_on_light * pawns_on_light;
    }

    return BISHOP_COLOR * (bishop_color[white] - bishop_color[black]) as i64;
}
