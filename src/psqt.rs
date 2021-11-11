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
    [S!(-53,    55), S!(  23, 162), S!( 70, 286), S!(-78,  270)],
    [S!(-91,    51), S!(-147,  82), S!(-90,  94), S!(-47,  100)],
    [S!(-111,  118), S!( -52, 119), S!(-16,   2), S!(-39, -61)],
    [S!(41,    271), S!( 165, 175), S!(109,  28), S!(208, -187)],
    [S!(79,    664), S!( 296, 523), S!(772, 171), S!(489, -160)],
    [S!( 385, 1000), S!( 241, 840), S!(465, 571), S!(822,  150)],
    [S!(   0,    0), S!(   0,   0), S!(  0,   0), S!(  0,    0)]  // last rank
];

pub const KNIGHT_PSQT: [[Score; 4]; 8] = [
    [S!(-632, -516), S!(-258,-687), S!( -127, -442), S!(-220, -338)],
    [S!(-226, -442), S!(-299,-233), S!( -280, -215), S!(-127,   -4)],
    [S!(-336, -567), S!( -69, -75), S!( -78,   222), S!(  84,  460)],
    [S!(-155, -201), S!( 281, 131), S!(  15,   727), S!( 118,  661)],
    [S!( 246, -183), S!( 124, 356), S!( 381,   609), S!( 250,  754)],
    [S!( 123, -340), S!( 174, 153), S!( 341,   719), S!( 502,  592)],
    [S!(-94,  -281), S!(-289, -88), S!( 191,   -36), S!( 324,  279)],
    [S!(-916, -901), S!(-177,   0), S!(-273,    88), S!(  16,  -49)]
];

pub const BISHOP_PSQT: [[Score; 4]; 8] = [
    [S!( 119,  -48), S!( 253, -62), S!( -89,   33), S!(-303,  29)],
    [S!(  93, -379), S!( 126, -71), S!(   7, -186), S!(-128,  -4)],
    [S!(  20,   15), S!(  45, -52), S!( -45,   53), S!( -31,  58)],
    [S!( 157,    1), S!( -38,  87), S!( -87,  142), S!( 126, 202)],
    [S!(-178,  193), S!( 100, 217), S!(  -1,  165), S!( 178, 226)],
    [S!(  17,  237), S!(  83, 356), S!( 328,  168), S!( 175, 154)],
    [S!(-585,  379), S!(-291, 167), S!(-161,  210), S!(-138, 193)],
    [S!(-122,  230), S!( 101, 213), S!(-437,  265), S!(-407, 353)]
];

pub const ROOK_PSQT: [[Score; 4]; 8] = [
    [S!(-215, -102), S!(-126, -115), S!( -88, -107), S!( -85, -133)],
    [S!(-650,   40), S!(-262, -260), S!(-210, -217), S!( -96, -242)],
    [S!(-374,  -33), S!(-165,  -29), S!(-412,  -36), S!(-185, -150)],
    [S!(-258,   70), S!(-319,  258), S!(-341,  284), S!(-193,  127)],
    [S!(  30,  161), S!( 272,  110), S!( 154,  243), S!( 251,  182)],
    [S!( -43,  271), S!( 521,   90), S!( 380,  246), S!( 567,   97)],
    [S!( 348,  228), S!(  -2,  444), S!( 278,  313), S!( 499,  364)],
    [S!( 472,  396), S!( 230,  490), S!( 329,  473), S!( 396,  381)]
];

pub const QUEEN_PSQT: [[Score; 4]; 8] = [
    [S!( 147, -495), S!( 111, -578), S!(  76, -741), S!(  75, -440)],
    [S!( -16, -375), S!( 152, -566), S!( 191, -801), S!(  56, -240)],
    [S!( -97, -432), S!( 140, -166), S!( -61,    5), S!( -60,  -16)],
    [S!( -43,  438), S!( 157,  -91), S!( -28,  104), S!(-190,  567)],
    [S!(  53,  321), S!( -76,  819), S!(-205,  557), S!(-330,  838)],
    [S!( -71,  280), S!( 117,  432), S!(-142,  483), S!( -92,  528)],
    [S!(-270,  454), S!(-632,  396), S!( -97,  349), S!(-164,  543)],
    [S!( 190,   70), S!( 161,  164), S!( 384,  370), S!(  85,   62)]
];

pub const KING_PSQT: [[Score; 4]; 8] = [
    [S!(  59, -1000), S!(290, -478), S!(-352, -299), S!(-253, -616)],
    [S!( 185,  -375), S!( 29,  -58), S!(-496,  212), S!(-669,  201)],
    [S!(-368,  -145), S!( 40,   35), S!(  71,  204), S!( -58,  417)],
    [S!(-493,  -111), S!(313,  203), S!( 372,  401), S!( 307,  547)],
    [S!(-318,   -11), S!(207,  401), S!( 205,  576), S!( 165,  611)],
    [S!( -61,    71), S!(193,  531), S!(  92,  732), S!(  32,  459)],
    [S!(-154,  -413), S!( 21,  597), S!( 185,  387), S!(-172,  235)],
    [S!(-258,  -844), S!( 62, -445), S!( -58, -390), S!(-187, -100)]
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
