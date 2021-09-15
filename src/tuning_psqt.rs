use crate::bitboard::*;
use crate::tuning_eval::*;
use crate::util::*;

macro_rules! S {
    ($a:expr, $b:expr) => {
        make_score($a, $b)
    }
}

pub static mut PAWN_PSQT: [[Score; 4]; 8] = [
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)], // first rank
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(-50, -50)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!( 50,  50)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(100, 100)],
    [S!(  0,   0), S!(  0,   0), S!( 50,  50), S!(100, 125)],
    [S!( 25,  50), S!( 25,  50), S!( 75, 100), S!(100, 150)],
    [S!(300, 700), S!(300, 700), S!(300, 700), S!(300, 700)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)]  // last rank
];

pub static mut KNIGHT_PSQT: [[Score; 4]; 8] = [
    [S!(-250, -250), S!(-200, -200), S!(-150, -150), S!(-150, -150)],
    [S!(-200, -200), S!(   0,    0), S!(   0,    0), S!(   0,    0)],
    [S!(-200, -200), S!(   0,    0), S!( 100,  100), S!( 150,  150)],
    [S!(-150, -150), S!(   0,    0), S!( 150,  150), S!( 200,  200)],
    [S!(-150, -150), S!(   0,    0), S!( 150,  150), S!( 200,  200)],
    [S!(-200, -200), S!(   0,    0), S!( 100,  100), S!( 150,  150)],
    [S!(-200, -200), S!(   0,    0), S!(   0,    0), S!(   0,    0)],
    [S!(-250, -250), S!(-200, -200), S!(-150, -150), S!(-150, -150)]
];

pub static mut BISHOP_PSQT: [[Score; 4]; 8] = [
    [S!(-150, -150), S!( -75,  -75), S!(-200, -200), S!( -75,  -75)],
    [S!( -75,  -75), S!(  50,   50), S!(   0,    0), S!(   0,    0)],
    [S!( -75,  -75), S!(   0,    0), S!(   0,    0), S!(   0,    0)],
    [S!( -75,  -75), S!(   0,    0), S!(  50,   50), S!(  75,   75)],
    [S!( -75,  -75), S!(   0,    0), S!(   0,    0), S!(  75,   75)],
    [S!( -75,  -75), S!(   0,    0), S!(  50,   50), S!(  75,   75)],
    [S!( -75,  -75), S!(  50,   50), S!(   0,    0), S!(   0,    0)],
    [S!(-150, -150), S!( -75,  -75), S!( -75,  -75), S!( -75,  -75)]
];

pub static mut ROOK_PSQT: [[Score; 4]; 8] = [
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)]
];

pub static mut QUEEN_PSQT: [[Score; 4]; 8] = [
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)],
    [S!(  0,   0), S!(  0,   0), S!(  0,   0), S!(  0,   0)]
];

pub static mut KING_PSQT: [[Score; 4]; 8] = [
    [S!( 100, -400), S!( 200, -300), S!(  75, -200), S!(   0, -200)],
    [S!( 100, -300), S!( 100, -200), S!(   0, -100), S!(   0,    0)],
    [S!( -75, -300), S!(-100,  200), S!(-100,  200), S!(-100,  300)],
    [S!( -75, -300), S!(-100, -100), S!(-100,  300), S!(-200,  400)],
    [S!( -75, -300), S!(-100, -100), S!(-150,  300), S!(-250,  400)],
    [S!( -75, -300), S!(-100, -100), S!(-150,  200), S!(-250,  300)],
    [S!( -75, -300), S!(-100, -200), S!(-150, -100), S!(-250,    0)],
    [S!( -75, -400), S!(-100, -300), S!(-150, -200), S!(-250, -100)]
];

fn get_psqt_bonus(psqt: &[[Score; 4]; 8], bb: u64, side_to_move: Color) -> Score {
    let mut bb = bb;
    let mut score = make_score(0, 0);
    while bb != 0 {
        let bb_idx = bb.trailing_zeros() as i32;
        bb &= bb - 1;
        let psqt_idx = if side_to_move == Color::White {bb_idx} else {63 - bb_idx};
        let r = (psqt_idx / 8) as usize;
        let potential_f = psqt_idx % 8;
        // psqt tables are mirrored horizontally
        // file 4 (zero-indexed) should get the 3th entry
        // file 5 should get the 2th entry and so on...
        let f = (if potential_f > 3 { 7 - potential_f } else { potential_f }) as usize;

        score += psqt[r][f];
    }
    return score;
}

pub fn pawn_psqt_value(pos: &Bitboard) -> Score {
    unsafe {
        let white = Color::White as usize;
        let black = Color::Black as usize;
        return get_psqt_bonus(&PAWN_PSQT, pos.pawn[white], Color::White) - get_psqt_bonus(&PAWN_PSQT, pos.pawn[black], Color::Black);
    }
}

unsafe fn get_knight_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&KNIGHT_PSQT, pos.knight[white], Color::White) - get_psqt_bonus(&KNIGHT_PSQT, pos.knight[black], Color::Black);
}

unsafe fn get_bishop_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&BISHOP_PSQT, pos.bishop[white], Color::White) - get_psqt_bonus(&BISHOP_PSQT, pos.bishop[black], Color::Black);
}

unsafe fn get_rook_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&ROOK_PSQT, pos.rook[white], Color::White) - get_psqt_bonus(&ROOK_PSQT, pos.rook[black], Color::Black);
}

unsafe fn get_queen_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&QUEEN_PSQT, pos.queen[white], Color::White) - get_psqt_bonus(&QUEEN_PSQT, pos.queen[black], Color::Black);
}

unsafe fn get_king_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&KING_PSQT, pos.king[white], Color::White) - get_psqt_bonus(&KING_PSQT, pos.king[black], Color::Black);
}

pub fn nonpawn_psqt_value(pos: &Bitboard) -> Score {
    unsafe {
        return get_knight_psqt(pos) + get_bishop_psqt(pos)
            + get_rook_psqt(pos) + get_queen_psqt(pos) + get_king_psqt(pos);
    }
}
