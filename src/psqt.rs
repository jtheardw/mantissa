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
    [S!(-167, 73), S!(-81, 89), S!(-117, 178), S!(-182, 141)],
    [S!(-192, 26), S!(-167, -8), S!(-179, 36), S!(-162, 22)],
    [S!(-172, 50), S!(-85, 5), S!(-113, -15), S!(-108, -124)],
    [S!(-114, 146), S!(-10, 78), S!(-26, 19), S!(19, -144)],
    [S!(-136, 411), S!(34, 345), S!(233, 200), S!(81, 47)],
    [S!(734, 1074), S!(663, 1054), S!(897, 886), S!(1061, 658)],
    [S!(   0,    0), S!(   0,   0), S!(  0,   0), S!(  0,    0)]  // last rank
];

pub const KNIGHT_PSQT: [[Score; 4]; 8] = [
    [S!(-916, -942), S!(-307, -716), S!(-351, -336), S!(-358, -262)],
    [S!(-293, -401), S!(-457, -198), S!(-274, -212), S!(-186, 19)],
    [S!(-337, -625), S!(-167, -7), S!(-136, 326), S!(2, 530)],
    [S!(-167, -116), S!(179, 157), S!(-10, 669), S!(54, 710)],
    [S!(119, -133), S!(92, 289), S!(327, 634), S!(197, 758)],
    [S!(31, -308), S!(180, 114), S!(403, 598), S!(539, 474)],
    [S!(-204, -369), S!(-352, -47), S!(324, -87), S!(504, 206)],
    [S!(-1514, -1011), S!(-414, -235), S!(-934, 269), S!(131, 29)]
];

pub const BISHOP_PSQT: [[Score; 4]; 8] = [
    [S!(34, -133), S!(-17, -19), S!(-206, 11), S!(-369, 10)],
    [S!(4, -220), S!(-13, -163), S!(-92, -148), S!(-208, -39)],
    [S!(-111, 39), S!(-111, -23), S!(-146, 37), S!(-147, 94)],
    [S!(-141, -2), S!(-101, 29), S!(-167, 182), S!(-13, 188)],
    [S!(-239, 193), S!(-28, 200), S!(15, 100), S!(138, 215)],
    [S!(-125, 232), S!(110, 209), S!(176, 205), S!(117, 37)],
    [S!(-493, 249), S!(-348, 258), S!(-220, 203), S!(-336, 173)],
    [S!(-668, 354), S!(-302, 232), S!(-1318, 371), S!(-883, 340)]
];


pub const ROOK_PSQT: [[Score; 4]; 8] = [
    [S!(-216, -71), S!(-143, -49), S!(-111, -14), S!(-56, -105)],
    [S!(-615, -90), S!(-245, -171), S!(-223, -117), S!(-131, -190)],
    [S!(-351, -39), S!(-152, 46), S!(-373, 50), S!(-233, 4)],
    [S!(-344, 221), S!(-157, 279), S!(-269, 288), S!(-150, 213)],
    [S!(9, 326), S!(141, 279), S!(222, 288), S!(309, 273)],
    [S!(33, 406), S!(574, 166), S!(365, 371), S!(655, 142)],
    [S!(307, 331), S!(80, 441), S!(479, 292), S!(512, 412)],
    [S!(715, 402), S!(741, 389), S!(217, 580), S!(453, 444)]
];

pub const QUEEN_PSQT: [[Score; 4]; 8] = [
    [S!(117, -617), S!(85, -813), S!(57, -738), S!(97, -517)],
    [S!(102, -613), S!(149, -752), S!(183, -812), S!(78, -380)],
    [S!(76, -569), S!(130, -342), S!(-58, -46), S!(-72, -54)],
    [S!(52, 11), S!(63, 83), S!(-125, 245), S!(-183, 502)],
    [S!(80, 146), S!(-6, 591), S!(-219, 620), S!(-279, 794)],
    [S!(-67, 236), S!(166, 142), S!(-117, 669), S!(-144, 741)],
    [S!(-170, 367), S!(-672, 665), S!(-249, 617), S!(-657, 1012)],
    [S!(180, 45), S!(235, 190), S!(129, 365), S!(25, 337)]
];

pub const KING_PSQT: [[Score; 4]; 8] = [
    [S!(133, -1142), S!(189, -430), S!(-434, -212), S!(-323, -569)],
    [S!(106, -351), S!(25, -81), S!(-520, 220), S!(-678, 230)],
    [S!(-332, -143), S!(-4, 27), S!(-70, 254), S!(-21, 386)],
    [S!(-469, -152), S!(423, 168), S!(438, 390), S!(391, 506)],
    [S!(-237, 30), S!(618, 365), S!(814, 460), S!(407, 535)],
    [S!(-37, 49), S!(974, 445), S!(1070, 460), S!(523, 367)],
    [S!(-197, -309), S!(402, 476), S!(683, 153), S!(417, 0)],
    [S!(-18, -1724), S!(-82, -439), S!(58, -437), S!(-149, -303)]
];

fn get_psqt_bonus(psqt: &[[Score; 4]; 8], bb: u64, side_to_move: Color) -> Score {
    let mut bb = bb;
    let mut score = make_score(0, 0);
    while bb != 0 {
        let psqt_idx = bb.trailing_zeros() as i32;
        bb &= bb - 1;

        let r = if side_to_move == Color::White {(psqt_idx / 8)} else {7 - (psqt_idx / 8)} as usize;
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
