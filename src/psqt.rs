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
    [S!(-156, 94), S!(-85, 150), S!(-102, 219), S!(-181, 223)],
    [S!(-165, 57), S!(-159, 39), S!(-162, 85), S!(-154, 79)],
    [S!(-143, 69), S!(-96, 38), S!(-91, 27), S!(-100, -54)],
    [S!(-83, 187), S!(5, 130), S!(-10, 65), S!(45, -65)],
    [S!(-134, 529), S!(27, 457), S!(178, 310), S!(190, 109)],
    [S!(753, 1125), S!(691, 1054), S!(1065, 943), S!(1393, 520)],
    [S!(   0,    0), S!(   0,   0), S!(  0,   0), S!(  0,    0)]  // last rank
];

pub const KNIGHT_PSQT: [[Score; 4]; 8] = [
    [S!(-936, -713), S!(-287, -823), S!(-315, -312), S!(-316, -212)],
    [S!(-256, -425), S!(-354, -243), S!(-224, -218), S!(-173, 57)],
    [S!(-312, -607), S!(-137, 36), S!(-106, 320), S!(35, 521)],
    [S!(-148, -102), S!(6, 234), S!(24, 639), S!(97, 632)],
    [S!(76, -107), S!(130, 270), S!(318, 647), S!(211, 724)],
    [S!(5, -330), S!(238, 50), S!(427, 569), S!(590, 350)],
    [S!(-159, -441), S!(-333, -109), S!(365, -157), S!(491, 206)],
    [S!(-1420, -966), S!(-511, -313), S!(-905, 115), S!(163, -125)],
];

pub const BISHOP_PSQT: [[Score; 4]; 8] = [
    [S!(84, -124), S!(-28, -73), S!(-183, 9), S!(-251, 6)],
    [S!(51, -241), S!(69, -149), S!(-6, -152), S!(-126, -18)],
    [S!(-43, 10), S!(-2, 6), S!(-63, 103), S!(-81, 135)],
    [S!(-133, 45), S!(1, -13), S!(-109, 199), S!(31, 248)],
    [S!(-183, 146), S!(6, 275), S!(64, 146), S!(215, 240)],
    [S!(-43, 188), S!(65, 313), S!(383, 134), S!(252, 65)],
    [S!(-305, 143), S!(-337, 332), S!(-173, 210), S!(-199, 136)],
    [S!(-455, 278), S!(-5, 166), S!(-946, 306), S!(-874, 406)]
];


pub const ROOK_PSQT: [[Score; 4]; 8] = [
    [S!(-229, -30), S!(-157, -20), S!(-153, 64), S!(-80, -56)],
    [S!(-605, -33), S!(-210, -142), S!(-289, -139), S!(-199, -137)],
    [S!(-380, -33), S!(-238, 58), S!(-381, 23), S!(-204, -23)],
    [S!(-293, 143), S!(-186, 245), S!(-394, 369), S!(-188, 234)],
    [S!(-40, 311), S!(74, 319), S!(103, 330), S!(286, 282)],
    [S!(63, 356), S!(503, 164), S!(332, 299), S!(528, 185)],
    [S!(313, 332), S!(65, 426), S!(591, 199), S!(489, 404)],
    [S!(664, 403), S!(716, 392), S!(220, 581), S!(576, 393)]
];

pub const QUEEN_PSQT: [[Score; 4]; 8] = [
    [S!(163, -663), S!(79, -914), S!(50, -699), S!(151, -501)],
    [S!(168, -545), S!(231, -814), S!(238, -961), S!(104, -367)],
    [S!(82, -507), S!(212, -405), S!(-74, -50), S!(-57, -96)],
    [S!(-29, 121), S!(54, 115), S!(-116, 212), S!(-157, 455)],
    [S!(-76, 226), S!(-142, 621), S!(-251, 550), S!(-203, 847)],
    [S!(43, 137), S!(120, 131), S!(-57, 685), S!(-263, 783)],
    [S!(-51, 581), S!(-621, 714), S!(-105, 558), S!(-464, 964)],
    [S!(431, 4), S!(485, 430), S!(306, 427), S!(135, 472)]
];

pub const KING_PSQT: [[Score; 4]; 8] = [
    [S!(219, -1031), S!(250, -431), S!(-361, -227), S!(-313, -511)],
    [S!(87, -310), S!(-103, -77), S!(-605, 169), S!(-761, 231)],
    [S!(-350, -86), S!(-74, 23), S!(-104, 223), S!(-237, 417)],
    [S!(-353, -158), S!(409, 115), S!(379, 364), S!(343, 502)],
    [S!(-244, -30), S!(538, 300), S!(282, 518), S!(286, 503)],
    [S!(-53, -122), S!(910, 431), S!(723, 475), S!(400, 281)],
    [S!(-287, -287), S!(-189, 471), S!(410, 159), S!(-82, 95)],
    [S!(-78, -1534), S!(-278, -465), S!(-9, -316), S!(-51, -314)]
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
