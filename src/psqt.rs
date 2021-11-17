use crate::bitboard::*;
use crate::eval::*;
use crate::util::*;

macro_rules! S {
    ($a:expr, $b:expr) => {
        make_score($a, $b)
    }
}

pub const PAWN_PSQT: [[Score; 4]; 8] = [
    [S!(  0,     0), S!(   0,   0), S!(  0,   0), S!(  0,    0)], // first rank
    [S!(-227, 37), S!(-40, 131), S!(-134, 269), S!(-220, 229)],
    [S!(-209, -5), S!(-220, 20), S!(-193, 73), S!(-150, 51)],
    [S!(-164, 61), S!(-54, 57), S!(-72, -47), S!(-21, -111)],
    [S!(-92, 247), S!(48, 139), S!(18, 47), S!(169, -157)],
    [S!(-214, 816), S!(212, 680), S!(524, 349), S!(199, 108)],
    [S!(1086, 1493), S!(954, 1454), S!(1365, 1065), S!(1701, 662)],
    [S!(   0,    0), S!(   0,   0), S!(  0,   0), S!(  0,    0)]  // last rank
];

pub const KNIGHT_PSQT: [[Score; 4]; 8] = [
    [S!(-955, -783), S!(-288, -708), S!(-309, -347), S!(-318, -213)],
    [S!(-214, -402), S!(-417, -147), S!(-265, -118), S!(-199, 73)],
    [S!(-372, -471), S!(-123, 7), S!(-134, 291), S!(21, 496)],
    [S!(-122, -106), S!(162, 232), S!(16, 657), S!(121, 674)],
    [S!(81, -133), S!(25, 167), S!(307, 450), S!(129, 662)],
    [S!(-74, -357), S!(90, 25), S!(332, 412), S!(443, 369)],
    [S!(-123, -342), S!(-422, -18), S!(360, -85), S!(410, 282)],
    [S!(-1512, -1098), S!(-427, -256), S!(-939, 234), S!(-21, 28)]
];

pub const BISHOP_PSQT: [[Score; 4]; 8] = [
    [S!(8, -169), S!(-26, -55), S!(-160, -52), S!(-278, -12)],
    [S!(1, -242), S!(-26, -262), S!(-18, -198), S!(-183, -15)],
    [S!(-116, 41), S!(-62, 3), S!(-176, -55), S!(-63, 103)],
    [S!(-139, 47), S!(-68, 95), S!(-101, 206), S!(78, 236)],
    [S!(-297, 194), S!(-3, 159), S!(-120, 144), S!(43, 254)],
    [S!(-144, 202), S!(80, 161), S!(26, 112), S!(64, 60)],
    [S!(-530, 234), S!(-414, 113), S!(-143, 237), S!(-286, 128)],
    [S!(-575, 113), S!(-62, 201), S!(-1113, 314), S!(-735, 319)]
];

pub const ROOK_PSQT: [[Score; 4]; 8] = [
    [S!(-255, -103), S!(-236, -29), S!(-160, -19), S!(-141, -91)],
    [S!(-644, -111), S!(-265, -167), S!(-222, -178), S!(-216, -185)],
    [S!(-368, -71), S!(-201, -17), S!(-441, 76), S!(-262, -35)],
    [S!(-345, 132), S!(-234, 260), S!(-270, 264), S!(-215, 186)],
    [S!(-36, 242), S!(106, 232), S!(195, 258), S!(260, 210)],
    [S!(-31, 359), S!(547, 135), S!(351, 294), S!(629, 138)],
    [S!(211, 335), S!(65, 418), S!(452, 281), S!(531, 357)],
    [S!(733, 301), S!(511, 394), S!(224, 535), S!(372, 420)]
];

pub const QUEEN_PSQT: [[Score; 4]; 8] = [
    [S!(197, -561), S!(49, -694), S!(75, -745), S!(109, -469)],
    [S!(68, -694), S!(192, -770), S!(194, -848), S!(123, -402)],
    [S!(112, -518), S!(124, -311), S!(23, -93), S!(7, -131)],
    [S!(69, 3), S!(114, 64), S!(-41, 348), S!(-128, 487)],
    [S!(88, 182), S!(49, 683), S!(-124, 633), S!(-159, 672)],
    [S!(-22, 235), S!(208, 223), S!(-19, 673), S!(33, 590)],
    [S!(-187, 570), S!(-622, 561), S!(-170, 649), S!(-459, 959)],
    [S!(202, -10), S!(287, 140), S!(-23, 589), S!(247, 280)]
];

pub const KING_PSQT: [[Score; 4]; 8] = [
    [S!(178, -1242), S!(337, -382), S!(-163, -140), S!(-53, -437)],
    [S!(73, -436), S!(-55, -36), S!(-434, 261), S!(-524, 297)],
    [S!(-359, -285), S!(200, -31), S!(104, 236), S!(151, 397)],
    [S!(-684, -375), S!(790, 20), S!(597, 274), S!(349, 480)],
    [S!(-346, -158), S!(481, 131), S!(717, 322), S!(165, 474)],
    [S!(-8, -28), S!(678, 406), S!(865, 366), S!(746, 321)],
    [S!(-165, -468), S!(17, 519), S!(546, 145), S!(-187, 170)],
    [S!(-444, -1606), S!(48, -437), S!(41, -264), S!(-302, -287)]
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
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&PAWN_PSQT, pos.pawn[white], Color::White) - get_psqt_bonus(&PAWN_PSQT, pos.pawn[black], Color::Black);
}

fn get_knight_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&KNIGHT_PSQT, pos.knight[white], Color::White) - get_psqt_bonus(&KNIGHT_PSQT, pos.knight[black], Color::Black);
}

fn get_bishop_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&BISHOP_PSQT, pos.bishop[white], Color::White) - get_psqt_bonus(&BISHOP_PSQT, pos.bishop[black], Color::Black);
}

fn get_rook_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&ROOK_PSQT, pos.rook[white], Color::White) - get_psqt_bonus(&ROOK_PSQT, pos.rook[black], Color::Black);
}

fn get_queen_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&QUEEN_PSQT, pos.queen[white], Color::White) - get_psqt_bonus(&QUEEN_PSQT, pos.queen[black], Color::Black);
}

fn get_king_psqt(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    return get_psqt_bonus(&KING_PSQT, pos.king[white], Color::White) - get_psqt_bonus(&KING_PSQT, pos.king[black], Color::Black);
}

pub fn nonpawn_psqt_value(pos: &Bitboard) -> Score {
    return get_knight_psqt(pos) + get_bishop_psqt(pos)
         + get_rook_psqt(pos) + get_queen_psqt(pos) + get_king_psqt(pos);
}
