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

pub fn evaluate_position(pos: &Bitboard, phase: i32) -> i32 {
    // positive is white-favored, negative black-favored
    let mut score: Score = make_score(0, 0);
    score += material_score(pos);
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
