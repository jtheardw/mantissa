use crate::bitboard::*;
use crate::movegen::*;
use crate::pht::*;
use crate::psqt::*;
use crate::util::*;

pub type Score = i64;

// This is an idea I'm stealing from Stockfish's source
// and an older version of Ethereal
// essentially you store 2 scores
pub const fn make_score(mg_value: i32, eg_value: i32) -> Score {
    ((mg_value as i64) << 32) + (eg_value as i64)
}

pub fn mg_score(score: Score) -> i32 {
    // this is a quirk required to handle the way the
    // eg value was added in (particularly if it was a negative number)
    ((score + 0x80000000) >> 32) as i32
}

pub fn eg_score(score: Score) -> i32 {
    ((score << 32) >> 32) as i32
}

pub fn taper_score(s: Score, phase: i32) -> i32 {
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

pub const QUEEN_VALUE: Score = S!(10947, 16706);
pub const ROOK_VALUE: Score = S!(4865, 8652);
pub const BISHOP_VALUE: Score = S!(3523, 5211);
pub const KNIGHT_VALUE: Score = S!(3466, 4880);
pub const PAWN_VALUE: Score = S!(800, 1414);

pub const KNIGHT_MOBILITY: [Score; 9] = [
    S!(-275, 21), S!(-36,  10), S!(145, 148), S!(132, 528),
    S!(191, 535), S!(183, 563), S!(223, 551), S!(295, 453),
    S!(297, 333)
];
pub const BISHOP_MOBILITY: [Score; 14] = [
    S!(-139,  115), S!( 227, -209), S!(288, -843), S!(259, -339),
    S!( 374, -108), S!( 364,   96), S!(472,  310), S!(559,  389),
    S!( 572,  558), S!( 580,  634), S!(552,  682), S!(579,  621),
    S!( 440,  651), S!(1141,  511)
];
pub const ROOK_MOBILITY: [Score; 15] = [
    S!( 66,  -86), S!( 55,  97), S!(-221, -254), S!(-78,  -74),
    S!(-20,  301), S!( 13, 468), S!(  15,  756), S!( -4,  833),
    S!( -8,  901), S!( 66, 935), S!( 139,  967), S!(190, 1001),
    S!(277, 1005), S!(335, 969), S!( 569,  767)
];
pub const QUEEN_MOBILITY: [Score; 28] = [
    S!( 42,  -24), S!( 41, 121), S!(-241,   81), S!(-42,  76),
    S!( -3,  134), S!(302, -76), S!( -45,  -89), S!( 92,  47),
    S!(123, -119), S!(151,  64), S!( 227,  178), S!(184, 423),
    S!(244,  500), S!(249, 627), S!( 269,  800), S!(224, 883),
    S!(250,  910), S!(282, 801), S!( 234,  970), S!(334, 811),
    S!(191,  961), S!(200, 912), S!( 192,  935), S!( 71, 822),
    S!(235,  994), S!(443, 631), S!( 278, 1125), S!(448, 955)
];

pub const QUEEN_KING_DANGER: [i32; 8] = [66, 216, 464, 824, 983, 647, 753, 910];
pub const ROOK_KING_DANGER: [i32; 8] = [19, 35, 98, 177, 362, 502, 400, 293];
pub const BISHOP_KING_DANGER: [i32; 8] = [0, 60, 41, 87, 55, 181, 124, 176];
pub const KNIGHT_KING_DANGER: [i32; 8] = [25, 0, 48, 103, 517, 115, 427, 154];

pub const DOUBLE_BISHOP_BONUS: Score = S!(109, 972);

pub const PASSED_PAWN_VALUE: [Score; 8] = [
    S!(  0,   0), S!( 45,  179), S!(   0,  239), S!(0, 511),
    S!(234, 794), S!(388, 1465), S!(1316, 1732), S!(0,   0)
];
pub const CENTER_PAWN_VALUE: Score = S!(84, 0);
pub const ISOLATED_PAWN_VALUE: Score = S!(-53, -67);
pub const DOUBLED_PAWN_VALUE: Score = S!(-64, -284);
pub const BACKWARDS_PAWN_VALUE: Score = S!(-61, -19);
pub const ADVANCED_PAWN_VALUE: [Score; 8] = [
    S!( 0,   0), S!( 32,  16), S!( 63,  54), S!(68, 82),
    S!(97, 201), S!(360, 244), S!(289, 500), S!( 0,  0)
];
pub const SUPPORTED_PAWN_BONUS: Score = S!(158, 65);
pub const SPACE_VALUE: Score = S!(16, 5);

pub const BISHOP_COLOR: Score = S!(-50, -30);

pub const TEMPO_BONUS: Score = S!(130, 130);

pub const ROOK_ON_SEVENTH: Score = S!(0, 24);
pub const ROOK_ON_OPEN: Score = S!(124, 95);

pub fn static_eval(pos: &Bitboard, pht: &mut PHT) -> i32 {
    let score = evaluate_position(pos, pht);
    return if pos.side_to_move == Color::White {score} else {-score};
}

fn halfmove_scale(score: i32, pos: &Bitboard) -> i32 {
    return ((100 - pos.halfmove as i32) * score) / 100;
}

pub fn evaluate_position(pos: &Bitboard, pht: &mut PHT) -> i32 {
    // positive is white-favored, negative black-favored
    let mut score: Score = make_score(0, 0);
    score += material_score(pos);
    score += mobility_and_king_danger(pos);
    score += pawn_structure_value(pos, pht);
    score += double_bishop_bonus(pos);
    score += bishop_color_value(pos);
    score += rook_on_seventh_value(pos);
    score += rook_on_open_value(pos);
    score += nonpawn_psqt_value(pos);
    score += if pos.side_to_move == Color::White {TEMPO_BONUS} else {-TEMPO_BONUS};
    return halfmove_scale(taper_score(score, pos.get_phase()), pos);
}

fn pawnless_endgame_drawish(pos: &Bitboard) -> bool {
    // some endgames are known to be drawish
    // for these, until I can produce more intelligent
    // heuristics, the first pass is to *discourage*
    // mantissa from trading into these if she is otherwise
    // winning and encourage trading into them if she
    // is losing by supressing the material score in
    // these endgames, but leaving everything else functional
    // so that positional play still occurs.
    if pos.pawn[0] != 0 || pos.pawn[1] != 0 {
        return false;
    }

    if pos.queen[0] != 0 || pos.queen[1] != 0 {
        return false;
    }

    let kingless_composite = [pos.composite[0] & !pos.king[0], pos.composite[1] & !pos.king[1]];

    if kingless_composite[0].count_ones() > 2 || kingless_composite[1].count_ones() > 2 {
        return false;
    }

    if kingless_composite[0].count_ones() == 1 && kingless_composite[1].count_ones() == 1 {
        // one-on-one endgames
        // KR v K(B/N)
        for side in [Color::White, Color::Black] {
            let us = side as usize;
            let them = !side as usize;
            if kingless_composite[us] == pos.rook[us] &&
                (kingless_composite[them] == pos.bishop[them] | pos.knight[them]) {
                    return true;
                }
        }
    } else {
        for side in [Color::White, Color::Black] {
            let us = side as usize;
            let them = !side as usize;
            // KRN v KR
            // KRB v KR
            if pos.rook[us].count_ones() == 1 && pos.rook[them].count_ones() == 1 {
                if kingless_composite[them] == pos.rook[them] &&
                    kingless_composite[us] == pos.rook[us] | pos.bishop[us] | pos.knight[us] {
                        return true;
                    }
            }
        }
    }
    return false;
}

pub fn material_score(pos: &Bitboard) -> Score {
    // TODO this will probably be handled incrementally
    let mut score: Score = make_score(0, 0);
    if pawnless_endgame_drawish(pos) { return score; }
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

fn mobility_and_king_danger(pos: &Bitboard) -> Score {
    let mut mobility: Score = make_score(0, 0);
    let mut king_danger: [i32; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let occ = pos.composite[white] | pos.composite[black];

    for side in [white, black] {
        let other_side = if side == white {black} else {white};
        let multiplier = if side == white {1} else {-1};

        let king_bb = pos.king[other_side];
        let mut attackers = 0;
        let mut attack_value: i32 = 0;
        let king_idx = king_bb.trailing_zeros() as i8;
        let king_zone = king_bb | unsafe{ KING_MASK[king_idx as usize] };

        let mut queen_attacks = 0;
        let mut rook_attacks = 0;
        let mut bishop_attacks = 0;
        let mut knight_attacks = 0;

        let mut board = pos.queen[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let move_board = queen_moves_board(start_idx, occ);
            let moves = move_board.count_ones() as usize;
            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                queen_attacks += attacks.count_ones();
            }
            mobility += multiplier * QUEEN_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.rook[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let move_board = rook_moves_board(start_idx, occ & !(pos.queen[side] | pos.rook[side]));
            let moves = move_board.count_ones() as usize;
            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                rook_attacks += attacks.count_ones();
            }
            mobility += multiplier * ROOK_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.bishop[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let move_board = bishop_moves_board(start_idx, occ & !(pos.queen[side] | pos.bishop[side]));
            let moves = move_board.count_ones() as usize;
            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                bishop_attacks += attacks.count_ones();
            }
            mobility += multiplier * BISHOP_MOBILITY[moves];
            board &= board - 1;
        }

        board = pos.knight[side];
        while board != 0 {
            let start_idx = board.trailing_zeros() as i8;
            let enemy = if side == white {black} else {white};
            let move_board = knight_moves_board(start_idx) & !pawn_attacks(pos.pawn[enemy], !pos.side_to_move);
            let moves = move_board.count_ones() as usize;
            let attacks = move_board & king_zone;
            if attacks != 0 {
                attackers += 1;
                knight_attacks += attacks.count_ones();
            }
            mobility += multiplier * KNIGHT_MOBILITY[moves];
            board &= board - 1;
        }

        if attackers > 7 { attackers = 7; }
        attack_value += QUEEN_KING_DANGER[attackers] * queen_attacks as i32;
        attack_value += ROOK_KING_DANGER[attackers] * rook_attacks as i32;
        attack_value += BISHOP_KING_DANGER[attackers] * bishop_attacks as i32;
        attack_value += KNIGHT_KING_DANGER[attackers] * knight_attacks as i32;
        king_danger[side] = attack_value;
    }

    let score = mobility + (make_score(1, 1) * (king_danger[white] - king_danger[black]) as i64);
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

pub fn print_value(pos: &Bitboard) {
    println!("material: {}", taper_score(material_score(pos), pos.get_phase()));
    println!("mobility and king_danger: {}", taper_score(mobility_and_king_danger(pos), pos.get_phase()));
    println!("passed_pawns: {}", taper_score(passed_pawns_value(pos), pos.get_phase()));
    println!("center_pawns: {}", taper_score(center_pawns_value(pos), pos.get_phase()));
    println!("isolated_pawns: {}", taper_score(isolated_pawns_value(pos), pos.get_phase()));
    println!("doubled_pawns: {}", taper_score(doubled_pawns_value(pos), pos.get_phase()));
    println!("backwards_pawns: {}", taper_score(backwards_pawns_value(pos), pos.get_phase()));
    println!("connected_pawns: {}", taper_score(connected_pawns_value(pos), pos.get_phase()));
    println!("space: {}", taper_score(space_control_value(pos), pos.get_phase()));
    println!("rook on 7th: {}", taper_score(rook_on_seventh_value(pos), pos.get_phase()));
    println!("rook on open: {}", taper_score(rook_on_open_value(pos), pos.get_phase()));
    println!("double_bishop_bonus: {}", taper_score(double_bishop_bonus(pos), pos.get_phase()));
    println!("bishop_color: {}", taper_score(bishop_color_value(pos), pos.get_phase()));
    println!("psqt: {}", taper_score(nonpawn_psqt_value(pos) + pawn_psqt_value(pos), pos.get_phase()));
}

fn pawn_structure_value(pos: &Bitboard, pht: &mut PHT) -> Score {
    // let pht;
    // unsafe {
    //     pht = &mut PHT;
    // }
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
        val += pawn_psqt_value(pos);
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
                if neighbor_files & pos.pawn[side] == 0 {
                    isolated_pawns[side] += 1;
                }
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
            let idx = pawn_bb.trailing_zeros() as i8;
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
    let bonus_black_space = (pos.pawn[black] << 8 | pos.pawn[black] << 16) & base_black_space;

    space_control[white] = (base_white_space.count_ones() + bonus_white_space.count_ones()) as i32;
    space_control[black] = (base_black_space.count_ones() + bonus_black_space.count_ones()) as i32;

    return SPACE_VALUE * (space_control[white] - space_control[black]) as i64;
}

fn rook_on_seventh_value(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let white_seventh_rooks = (pos.rook[white] & RANK_MASKS[6]).count_ones() as i32;
    let black_seventh_rooks = (pos.rook[black] & RANK_MASKS[1]).count_ones() as i32;

    let white_condition = (pos.king[black] & RANK_MASKS[7]) != 0 || (pos.pawn[black] & RANK_MASKS[6]) != 0;
    let black_condition = (pos.king[white] & RANK_MASKS[7]) != 0 || (pos.pawn[white] & RANK_MASKS[6]) != 0;

    return ROOK_ON_SEVENTH
        * (if white_condition {white_seventh_rooks} else {0}
         - if black_condition {black_seventh_rooks} else {0}) as i64;
}

fn rook_on_open_value(pos: &Bitboard) -> Score {
    let white = Color::White as usize;
    let black = Color::Black as usize;
    let mut openish_file_rooks: [i32; 2] = [0, 0];
    for side in [white, black] {
        let enemy_side = if side == white {black} else {white};
        let mut rook_bb = pos.rook[side];
        while rook_bb != 0 {
            let idx = rook_bb.trailing_zeros() as i8;
            rook_bb &= rook_bb - 1;

            let f = (idx % 8) as usize;
            let pawns = FILE_MASKS[f] & pos.pawn[side];
            if pawns != 0 { continue; }

            if (FILE_MASKS[f] & pos.pawn[enemy_side]) == 0 {
                // open
                openish_file_rooks[side] += 2;
            } else {
                openish_file_rooks[side] += 1;
            }
        }
    }

    return ROOK_ON_OPEN * (openish_file_rooks[white] - openish_file_rooks[black]) as i64;
}

pub fn bishop_color_value(pos: &Bitboard) -> Score {
    let mut bishop_color: [i32; 2] = [0, 0];
    let white = Color::White as usize;
    let black = Color::Black as usize;

    let dark_tiles_mask =
        0b10101010_01010101_10101010_01010101_10101010_01010101_10101010_01010101;
    let light_tiles_mask = !dark_tiles_mask;

    for side in [white, black] {
        let bishops_on_dark = (pos.bishop[side] & dark_tiles_mask).count_ones() as i32;
        let bishops_on_light = (pos.bishop[side] & light_tiles_mask).count_ones() as i32;

        let pawns_on_dark = (pos.pawn[side] & dark_tiles_mask).count_ones() as i32;
        let pawns_on_light = (pos.pawn[side] & light_tiles_mask).count_ones() as i32;

        bishop_color[side] = bishops_on_dark * pawns_on_dark + bishops_on_light * pawns_on_light;
    }

    return BISHOP_COLOR * (bishop_color[white] - bishop_color[black]) as i64;
}
