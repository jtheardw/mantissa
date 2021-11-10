use crate::bitboard::*;
use crate::tuning_eval::*;
use crate::util::*;

macro_rules! S {
    ($a:expr, $b:expr) => {
        make_score($a, $b)
    }
}

pub static mut PAWN_PSQT: [[Score; 4]; 8] = [
    [S!(  0,     0), S!(   0,   0), S!(  0,   0), S!(  0,    0)], // first rank
    [S!(-247, -65), S!(-42, 122), S!(-151, 173), S!(-296, 98)],
    [S!(-213, -96), S!(-89, 35), S!(-203, -66), S!(-158, -32)],
    [S!(-242, -12), S!(-18, 34), S!(-120, -117), S!(-58, -168)],
    [S!(-158, 230), S!(111, 221), S!(47, 86), S!(184, -60)],
    [S!(-172, 1091), S!(390, 1066), S!(685, 767), S!(662, 680)],
    [S!(1296, 1747), S!(1159, 1772), S!(1607, 1404), S!(1914, 1079)],
    [S!(   0,    0), S!(   0,   0), S!(  0,   0), S!(  0,    0)]  // last rank
];

pub static mut KNIGHT_PSQT: [[Score; 4]; 8] = [
    [S!(-984, -930), S!(-304, -939), S!(-386, -425), S!(-411, -269)],
    [S!(-293, -538), S!(-398, -256), S!(-254, -206), S!(-182, -24)],
    [S!(-267, -557), S!(-138, 17), S!(-101, 120), S!(-10, 329)],
    [S!(-143, -84), S!(160, 217), S!(22, 523), S!(15, 582)],
    [S!(136, -53), S!(90, 361), S!(341, 545), S!(217, 704)],
    [S!(112, -194), S!(201, 212), S!(518, 489), S!(656, 420)],
    [S!(-226, -376), S!(-408, 25), S!(331, 29), S!(587, 305)],
    [S!(-1511, -1036), S!(-481, -123), S!(-712, 243), S!(12, 186)]
];

pub static mut BISHOP_PSQT: [[Score; 4]; 8] = [
    [S!(-55, -329), S!(-76, -260), S!(-287, -351), S!(-502, -133)],
    [S!(49, -467), S!(-22, -209), S!(-49, -111), S!(-155, -4)],
    [S!(-66, -115), S!(-51, 30), S!(-62, 153), S!(-31, 193)],
    [S!(-108, -61), S!(-5, 158), S!(-20, 299), S!(116, 324)],
    [S!(-88, 87), S!(57, 338), S!(236, 261), S!(318, 378)],
    [S!(39, 78), S!(248, 230), S!(387, 291), S!(440, 194)],
    [S!(-300, -11), S!(-214, 319), S!(-37, 271), S!(-44, 257)],
    [S!(-487, 269), S!(24, 92), S!(-867, 240), S!(-708, 303)]
];

pub static mut ROOK_PSQT: [[Score; 4]; 8] = [
    [S!(-282, -240), S!(-189, -84), S!(-189, -22), S!(-117, -60)],
    [S!(-746, -229), S!(-265, -254), S!(-284, -144), S!(-301, -134)],
    [S!(-373, -226), S!(-238, -7), S!(-432, 30), S!(-325, -5)],
    [S!(-318, 53), S!(-202, 223), S!(-300, 298), S!(-218, 263)],
    [S!(-33, 243), S!(102, 277), S!(214, 329), S!(307, 312)],
    [S!(112, 349), S!(564, 228), S!(450, 401), S!(742, 251)],
    [S!(346, 403), S!(268, 448), S!(707, 363), S!(774, 459)],
    [S!(849, 332), S!(807, 384), S!(447, 569), S!(644, 465)]
];

pub static mut QUEEN_PSQT: [[Score; 4]; 8] = [
    [S!(-29, -617), S!(-48, -892), S!(-97, -951), S!(-29, -839)],
    [S!(-49, -723), S!(99, -913), S!(77, -866), S!(-19, -617)],
    [S!(66, -666), S!(94, -423), S!(-27, -145), S!(-36, -302)],
    [S!(64, -167), S!(85, 12), S!(-16, 196), S!(-128, 447)],
    [S!(230, 8), S!(14, 571), S!(14, 648), S!(-109, 807)],
    [S!(173, 256), S!(351, 146), S!(77, 772), S!(138, 812)],
    [S!(-227, 449), S!(-600, 728), S!(-81, 906), S!(-232, 1125)],
    [S!(341, 221), S!(412, 446), S!(324, 730), S!(551, 542)]
];

pub static mut KING_PSQT: [[Score; 4]; 8] = [
    [S!(241, -907), S!(223, -344), S!(-396, -238), S!(-229, -691)],
    [S!(172, -310), S!(26, -122), S!(-470, 88), S!(-680, 105)],
    [S!(-238, -150), S!(63, -72), S!(-110, 136), S!(-113, 251)],
    [S!(-517, -83), S!(338, 139), S!(377, 289), S!(187, 402)],
    [S!(-238, 105), S!(521, 362), S!(250, 469), S!(145, 506)],
    [S!(-62, 155), S!(628, 530), S!(616, 514), S!(513, 327)],
    [S!(-92, -142), S!(115, 562), S!(488, 240), S!(-247, 168)],
    [S!(-333, -1559), S!(-122, -283), S!(-46, -249), S!(-162, -309)]
];

fn get_psqt_bonus(psqt: &[[Score; 4]; 8], bb: u64, side_to_move: Color) -> Score {
    let mut bb = bb;
    let mut score = make_score(0, 0);
    while bb != 0 {
        let bb_idx = bb.trailing_zeros() as i32;
        bb &= bb - 1;
        // let psqt_idx = if side_to_move == Color::White {bb_idx} else {63 - bb_idx};
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

pub fn psqt_value(pos: &Bitboard) -> Score {
    unsafe {
        return get_knight_psqt(pos) + get_bishop_psqt(pos)
            + get_rook_psqt(pos) + get_queen_psqt(pos) + get_king_psqt(pos) + pawn_psqt_value(pos);
    }
}
