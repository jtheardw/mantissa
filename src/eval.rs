use crate::bitboard::*;
use crate::movegen::*;
use crate::util::*;

type Score = i64;

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

pub fn static_eval(pos: &Bitboard) -> i32 {
    let score = evaluate_position(pos, 0); // pos.get_phase()
    return if pos.side_to_move == Color::White {score} else {-score};
}

pub fn evaluate_position(pos: &Bitboard, phase: i32) -> i32 {
    // positive is white-favored, negative black-favored
    let mut score: Score = make_score(0, 0);
    score += material_score(pos);
    score += mobility(pos);
    return taper_score(score, phase);
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
