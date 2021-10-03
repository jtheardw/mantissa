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
    [S!(-104, 97), S!(16, 212), S!(33, 303), S!(-82, 294)],
    [S!(-190, 71), S!(-152, 86), S!(-140, 127), S!(-84, 116)],
    [S!(-111, 99), S!(-67, 130), S!(11, 3), S!(-46, -68)],
    [S!(27, 299), S!(104, 227), S!(163, 63), S!(247, -145)],
    [S!(62, 683), S!(452, 606), S!(832, 142), S!(684, -140)],
    [S!(163, 954), S!(-125, 1000), S!(714, 498), S!(1000, 124)],
    [S!(   0,    0), S!(   0,   0), S!(  0,   0), S!(  0,    0)]  // last rank
];


pub const KNIGHT_PSQT: [[Score; 4]; 8] = [
    [S!(-549, -1000), S!(-249, -699), S!(-333, -359), S!(-329, -202)],
    [S!(-307, -229), S!(-496, -101), S!(-243, -183), S!(-117, -77)],
    [S!(-307, -483), S!(-159, -85), S!(-63, 297), S!(6, 470)],
    [S!(-47, -156), S!(148, 210), S!(135, 597), S!(151, 693)],
    [S!(14, -97), S!(128, 363), S!(346, 645), S!(352, 765)],
    [S!(-57, -158), S!(505, 41), S!(704, 480), S!(729, 456)],
    [S!(-228, -442), S!(-300, -31), S!(246, -18), S!(663, 118)],
    [S!(-1000, -1000), S!(-1000, -91), S!(-1000, 119), S!(-356, 54)],
];

pub const BISHOP_PSQT: [[Score; 4]; 8] = [
    [S!(160, 6), S!(-44, 21), S!(-32, 97), S!(-103, 7)],
    [S!(157, -310), S!(128, -131), S!(2, -67), S!(-33, -7)],
    [S!(25, -27), S!(52, 5), S!(-26, 109), S!(8, 119)],
    [S!(-96, 102), S!(-31, 155), S!(14, 276), S!(138, 239)],
    [S!(-127, 168), S!(91, 246), S!(-19, 237), S!(245, 280)],
    [S!(-39, 291), S!(228, 258), S!(263, 183), S!(315, 119)],
    [S!(-628, 455), S!(-336, 355), S!(-63, 293), S!(-200, 174)],
    [S!(-884, 511), S!(-947, 431), S!(-1000, 462), S!(-1000, 424)],
];


pub const ROOK_PSQT: [[Score; 4]; 8] = [
    [S!(-286, -108), S!(-164, -53), S!(-142, -43), S!(-118, -102)],
    [S!(-838, -145), S!(-272, -250), S!(-199, -263), S!(-183, -220)],
    [S!(-427, -133), S!(-175, -80), S!(-337, -35), S!(-229, -109)],
    [S!(-393, 106), S!(-152, 146), S!(-349, 253), S!(-296, 210)],
    [S!(18, 242), S!(118, 227), S!(165, 210), S!(336, 173)],
    [S!(83, 264), S!(499, 161), S!(211, 319), S!(652, 84)],
    [S!(10, 416), S!(20, 434), S!(204, 404), S!(388, 428)],
    [S!(591, 398), S!(444, 492), S!(35, 614), S!(315, 444)],
];

pub const QUEEN_PSQT: [[Score; 4]; 8] = [
    [S!(130, -527), S!(77, -753), S!(93, -716), S!(128, -501)],
    [S!(55, -616), S!(74, -624), S!(169, -861), S!(63, -300)],
    [S!(61, -497), S!(67, -283), S!(-88, 42), S!(-68, -83)],
    [S!(135, -129), S!(47, 250), S!(-156, 368), S!(-204, 689)],
    [S!(-44, 368), S!(11, 652), S!(-201, 513), S!(-247, 815)],
    [S!(-20, 462), S!(-23, 357), S!(-91, 609), S!(-152, 695)],
    [S!(-200, 315), S!(-748, 600), S!(-331, 600), S!(-602, 1000)],
    [S!(270, -49), S!(-82, 310), S!(-397, 789), S!(-96, 456)],
];

pub const KING_PSQT: [[Score; 4]; 8] = [
    [S!(122, -1000), S!(282, -442), S!(-172, -286), S!(-277, -466)],
    [S!(181, -294), S!(66, -21), S!(-242, 139), S!(-680, 237)],
    [S!(-314, -131), S!(-1, 58), S!(171, 225), S!(-36, 426)],
    [S!(-541, -129), S!(714, 131), S!(791, 367), S!(547, 499)],
    [S!(-244, 77), S!(431, 455), S!(1000, 496), S!(920, 485)],
    [S!(-137, 273), S!(1000, 533), S!(1000, 500), S!(855, 334)],
    [S!(-233, -285), S!(767, 462), S!(1000, 97), S!(1000, 15)],
    [S!(-1000, -1000), S!(1000, -713), S!(579, -24), S!(-178, -37)],
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
