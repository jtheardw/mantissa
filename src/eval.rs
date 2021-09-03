use crate::bitboard::*;
use crate::movegen::*;
use crate::pht::*;
use crate::util::*;

pub type Score = i64;

// This is an idea I'm stealing from Stockfish's source
// and an older version of Ethereal
// essentially you store 2 scores
const fn make_score(mg_value: i32, eg_value: i32) -> Score {
    ((mg_value as i64) << 32) + (eg_value as i64)
}

fn mg_score(score: Score) -> i32 {
    // this is a quirk required to handle the way the
    // eg value was added in (particularly if it was a negative number)
    ((score + 0x80000000) >> 32) as i32
}

fn eg_score(score: Score) -> i32 {
    ((score << 32) >> 32) as i32
}

fn taper_score(s: Score, phase: i32) -> i32 {
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

// TODO might have to make these mutable
// if tuning needs to mess with them.
const QUEEN_VALUE: Score = S!(9200, 9200);
const ROOK_VALUE: Score = S!(5000, 5000);
const BISHOP_VALUE: Score = S!(3200, 3200);
const KNIGHT_VALUE: Score = S!(3000, 3000);
const PAWN_VALUE: Score = S!(1000, 1000);

const KNIGHT_MOBILITY: [Score; 9] = [
    S!(  0,   0), S!( 25,  27), S!( 50,  54), S!( 75,  81), S!(100, 108),
    S!(125, 135), S!(150, 162), S!(175, 189), S!(200, 216)
];
const BISHOP_MOBILITY: [Score; 14] = [
    S!(  0,   0), S!( 68,  10), S!(136,  20), S!(204,  30), S!(272,  40),
    S!(340,  50), S!(408,  60), S!(476,  70), S!(544,  80), S!(612,  90),
    S!(680, 100), S!(748, 110), S!(816, 120), S!(884, 130)
];
const ROOK_MOBILITY: [Score; 15] = [
    S!(  0,   0), S!( 10,  58), S!( 20, 116), S!( 30, 174), S!( 40, 232),
    S!( 50, 290), S!( 60, 348), S!( 70, 406), S!( 80, 464), S!( 90, 522),
    S!(100, 580), S!(110, 638), S!(120, 696), S!(130, 754), S!(140, 812)
];
const QUEEN_MOBILITY: [Score; 28] = [
    S!(  0,   0), S!( 16,  36), S!( 32,  72), S!( 48, 108), S!( 64, 144),
    S!( 80, 180), S!( 96, 216), S!(112, 252), S!(128, 288), S!(144, 324),
    S!(160, 360), S!(176, 396), S!(192, 432), S!(208, 468), S!(224, 504),
    S!(240, 540), S!(256, 576), S!(272, 612), S!(288, 648), S!(304, 684),
    S!(320, 720), S!(336, 756), S!(352, 792), S!(368, 828), S!(384, 864),
    S!(400, 900), S!(416, 936), S!(432, 972)
];

const DOUBLE_BISHOP_BONUS: Score = S!(500, 500);

const PASSED_PAWN_VALUE: [Score; 8] = [
    S!(   0,    0), S!( 116,  199), S!( 232,  398), S!( 348,  597), S!( 464,  796),
    S!( 580,  995), S!( 696, 1194), S!(   0,    0)
];
const CENTER_PAWN_VALUE: Score = S!(256, 282);
const ISOLATED_PAWN_VALUE: Score = S!(-181, -181);
const DOUBLED_PAWN_VALUE: Score = S!(-162, -344);
const BACKWARDS_PAWN_VALUE: Score = S!(-215, -233);
const ADVANCED_PAWN_VALUE: [Score; 8] = [
    S!(  0,   0), S!( 15,  21), S!( 30,  42), S!( 45,  63), S!( 60,  84),
    S!( 75, 105), S!( 90, 126), S!(  0,   0)
];
const SUPPORTED_PAWN_BONUS: Score = S!(26, 88);
const SPACE_VALUE: Score = S!(43, 19);

const BISHOP_COLOR_PENALTY: Score = S!(-50, -30);

const TEMPO_BONUS: Score = S!(130, 130);

pub fn static_eval(pos: &Bitboard) -> i32 {
    let score = evaluate_position(pos);
    return if pos.side_to_move == Color::White {score} else {-score};
}

pub fn evaluate_position(pos: &Bitboard) -> i32 {
    // positive is white-favored, negative black-favored
    let mut score: Score = make_score(0, 0);
    score += material_score(pos);
    score += mobility(pos);
    score += pawn_structure_value(pos);
    score += double_bishop_bonus(pos);
    score += if pos.side_to_move == Color::White {TEMPO_BONUS} else {-TEMPO_BONUS};
    return taper_score(score, pos.get_phase());
}

fn material_score(pos: &Bitboard) -> Score {
    // TODO this will probably be handled incrementally
    let mut score: Score = make_score(0, 0);
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

fn pawn_attacks(pawn_bb: u64, side_to_move: Color) -> u64 {
    if side_to_move == Color::White {
        ((pawn_bb & !FILE_MASKS[0]) << 7) | ((pawn_bb & !FILE_MASKS[7]) << 9)
    } else {
        ((pawn_bb & !FILE_MASKS[0]) >> 9) | ((pawn_bb & !FILE_MASKS[7]) >> 7)
    }
}

fn mobility(pos: &Bitboard) -> Score {
    let mut score: Score = make_score(0, 0);
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let occ = pos.composite[white] | pos.composite[black];

    for side in [white, black] {
        let multiplier = if side == white {1} else {-1};

        let mut board = pos.queen[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i32;
            let move_board = queen_moves_board(start_idx, occ);
            let moves = move_board.count_ones() as usize;
            score += multiplier * QUEEN_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.rook[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i32;
            let move_board = rook_moves_board(start_idx, occ & !(pos.queen[side] | pos.rook[side]));
            let moves = move_board.count_ones() as usize;
            score += multiplier * ROOK_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.bishop[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i32;
            let move_board = bishop_moves_board(start_idx, occ & !(pos.queen[side] | pos.bishop[side]));
            let moves = move_board.count_ones() as usize;
            score += multiplier * BISHOP_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.knight[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i32;
            let enemy = if side == white {black} else {white};
            let move_board = knight_moves_board(start_idx) & !pawn_attacks(pos.pawn[enemy], !pos.side_to_move);
            let moves = move_board.count_ones() as usize;
            score += multiplier * KNIGHT_MOBILITY[moves];
            board &= board - 1;
        }
    }

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

fn pawn_structure_value(pos: &Bitboard) -> Score {
    let pht;
    unsafe {
        pht = &mut PHT;
    }
    let mut val: Score = 0;
    let pht_entry = pht.get(pos.pawn_hash);
    if pht_entry.valid {
        val = pht_entry.value;
    } else {
        val += passed_pawns_value(pos);
        val += center_pawns_value(pos);
        val += isolated_pawns_value(pos);
        val += doubled_pawns_value(pos);
        val += backwards_pawns_value(pos);
        val += connected_pawns_value(pos);
        val += space_control_value(pos);

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
            }

            if neighbor_files & pos.pawn[side] == 0 {
                isolated_pawns[side] += 1;
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
            let idx = pawn_bb.trailing_zeros() as i32;
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
    let bonus_black_space = (pos.pawn[black] >> 8 | pos.pawn[black] >> 16) & base_black_space;

    space_control[white] = (base_white_space.count_ones() + bonus_white_space.count_ones()) as i32;
    space_control[black] = (base_black_space.count_ones() + bonus_black_space.count_ones()) as i32;

    return SPACE_VALUE * (space_control[white] - space_control[black]) as i64;
}
