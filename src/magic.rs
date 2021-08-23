use crate::util::*;

// magic table generation and usage

// Credit to Pradyumna Kannan for magic numbers
const ROOK_MAGIC_SHIFTS: [u32; 64] =
[
	52, 53, 53, 53, 53, 53, 53, 52,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 53, 53, 53, 53, 53
];

const ROOK_MAGIC_NUMBERS: [u64; 64] =
[
	0x0080001020400080, 0x0040001000200040, 0x0080081000200080, 0x0080040800100080,
	0x0080020400080080, 0x0080010200040080, 0x0080008001000200, 0x0080002040800100,
	0x0000800020400080, 0x0000400020005000, 0x0000801000200080, 0x0000800800100080,
	0x0000800400080080, 0x0000800200040080, 0x0000800100020080, 0x0000800040800100,
	0x0000208000400080, 0x0000404000201000, 0x0000808010002000, 0x0000808008001000,
	0x0000808004000800, 0x0000808002000400, 0x0000010100020004, 0x0000020000408104,
	0x0000208080004000, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
	0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
	0x0000204000800080, 0x0000200040401000, 0x0000100080802000, 0x0000080080801000,
	0x0000040080800800, 0x0000020080800400, 0x0000020001010004, 0x0000800040800100,
	0x0000204000808000, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
	0x0000040008008080, 0x0000020004008080, 0x0000010002008080, 0x0000004081020004,
	0x0000204000800080, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
	0x0000040008008080, 0x0000020004008080, 0x0000800100020080, 0x0000800041000080,
	0x00FFFCDDFCED714A, 0x007FFCDDFCED714A, 0x003FFFCDFFD88096, 0x0000040810002101,
	0x0001000204080011, 0x0001000204000801, 0x0001000082000401, 0x0001FFFAABFAD1A2
];

const BISHOP_MAGIC_SHIFTS: [u32; 64] =
[
	58, 59, 59, 59, 59, 59, 59, 58,
	59, 59, 59, 59, 59, 59, 59, 59,
	59, 59, 57, 57, 57, 57, 59, 59,
	59, 59, 57, 55, 55, 57, 59, 59,
	59, 59, 57, 55, 55, 57, 59, 59,
	59, 59, 57, 57, 57, 57, 59, 59,
	59, 59, 59, 59, 59, 59, 59, 59,
	58, 59, 59, 59, 59, 59, 59, 58
];

const BISHOP_MAGIC_NUMBERS: [u64; 64] =
[
	0x0002020202020200, 0x0002020202020000, 0x0004010202000000, 0x0004040080000000,
	0x0001104000000000, 0x0000821040000000, 0x0000410410400000, 0x0000104104104000,
	0x0000040404040400, 0x0000020202020200, 0x0000040102020000, 0x0000040400800000,
	0x0000011040000000, 0x0000008210400000, 0x0000004104104000, 0x0000002082082000,
	0x0004000808080800, 0x0002000404040400, 0x0001000202020200, 0x0000800802004000,
	0x0000800400A00000, 0x0000200100884000, 0x0000400082082000, 0x0000200041041000,
	0x0002080010101000, 0x0001040008080800, 0x0000208004010400, 0x0000404004010200,
	0x0000840000802000, 0x0000404002011000, 0x0000808001041000, 0x0000404000820800,
	0x0001041000202000, 0x0000820800101000, 0x0000104400080800, 0x0000020080080080,
	0x0000404040040100, 0x0000808100020100, 0x0001010100020800, 0x0000808080010400,
	0x0000820820004000, 0x0000410410002000, 0x0000082088001000, 0x0000002011000800,
	0x0000080100400400, 0x0001010101000200, 0x0002020202000400, 0x0001010101000200,
	0x0000410410400000, 0x0000208208200000, 0x0000002084100000, 0x0000000020880000,
	0x0000001002020000, 0x0000040408020000, 0x0004040404040000, 0x0002020202020000,
	0x0000104104104000, 0x0000002082082000, 0x0000000020841000, 0x0000000000208800,
	0x0000000010020200, 0x0000000404080200, 0x0000040404040400, 0x0002020202020200
];

pub static mut ROOK_MAGIC_TABLE: [[u64; 4096]; 64] = [[0; 4096]; 64];
pub static mut BISHOP_MAGIC_TABLE: [[u64; 512]; 64] = [[0; 512]; 64];

fn slow_gen_rook_mask() -> [u64; 64] {
    let mut rook_mask: [u64; 64] = [0; 64];
    for idx in 0..64 {
        let mut bb: u64 = 0;
        let (rx, ry) = idx_to_coord(idx);
        for (sx, sy) in [(0, 1), (0, -1), (-1, 0), (1, 0)] {
            for d in 1..8 {
                let (nx, ny) = (rx + (sx * d), ry + (sy * d));
                if ((nx < 1 || nx >= 7) && (nx != rx)) ||
                   ((ny < 1 || ny >= 7) && (ny != ry)) {
                        break;
                }
                bb |= coord_to_bb((nx, ny));
            }
        }

        rook_mask[idx as usize] = bb;
    }
    return rook_mask;
}

fn slow_gen_bishop_mask() -> [u64; 64] {
    let mut bishop_mask: [u64; 64] = [0; 64];
    for idx in 0..64 {
        let mut bb: u64 = 0;
        let (bx, by) = idx_to_coord(idx);
        for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)] {
            for d in 1..8 {
                let (nx, ny) = (bx + (sx * d), by + (sy * d));
                if ((nx < 1 || nx >= 7) && (nx != bx)) ||
                   ((ny < 1 || ny >= 7) && (ny != by)) {
                        break;
                   }
                bb |= coord_to_bb((nx, ny));
            }
        }
        bishop_mask[idx as usize] = bb;
    }
    return bishop_mask;
}

fn get_rook_moves_from_occ(coord: (i32, i32), ob: u64) -> u64 {
    // slow generate the rook moves when given an occupancy bitboard and a starting point
    let (rx, ry) = coord;
    let mut ob = ob;
    let mut bb = 0;
    for (sx, sy) in [(0, 1), (0, -1), (-1, 0), (1, 0)].iter() {
        let mut d = 1;
        while d < 8 {
            let (nx, ny) = (rx + (sx * d), ry + (sy * d));
            if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                break;
            }
            let new_idx = coord_to_idx((nx, ny));
            let old_ob = ob;
            bb |= 1 << new_idx;
            ob |= 1 << new_idx;
            if old_ob == ob {
                // reached a piece
                break;
            }
            d += 1;
        }
    }
    return bb;
}

fn get_bishop_moves_from_occ(coord: (i32, i32), ob: u64) -> u64 {
    // slow generate the bishop moves when given an occupancy bitboard and a starting point
    let (bx, by) = coord;
    let mut ob = ob;
    let mut bb = 0;
    for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)].iter() {
        let mut d = 1;
        while d < 8 {
            let (nx, ny) = (bx + (sx * d), by + (sy * d));
            if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                break;
            }
            let new_idx = coord_to_idx((nx, ny));
            let old_ob = ob;
            bb |= 1 << new_idx;
            ob |= 1 << new_idx;
            if old_ob == ob {
                // reached a piece
                break;
            }
            d += 1;
        }
    }
    return bb;
}

fn get_bishop_start_coords(coord: (i32, i32)) -> ((i32, i32), (i32, i32)) {
    // go from the coordinate of a bishop to the lowest point in the board
    // along its diagonals
    let (x, y) = coord;
    let mut lx = x;
    let mut ly = y;

    while lx - 1 >= 0 && ly - 1 >= 0 {
        lx -= 1;
        ly -= 1;
    }

    let mut rx = x;
    let mut ry = y;
    while rx + 1 < 8 && ry - 1 >= 0 {
        rx += 1;
        ry -= 1;
    }

    return ((lx, ly), (rx, ry));
}

pub fn rook_magic_hash(masked_composite: u64, square: usize) -> u64 {
    return (masked_composite * ROOK_MAGIC_NUMBERS[square]) >> ROOK_MAGIC_SHIFTS[square];
}

pub fn bishop_magic_hash(masked_composite: u64, square: usize) -> u64 {
    return (masked_composite * BISHOP_MAGIC_NUMBERS[square]) >> BISHOP_MAGIC_SHIFTS[square];
}

fn initialize_rook_magic_table() {
    let rook_mask = slow_gen_rook_mask();
    for idx in 0..64 {
        let (rx, ry) = idx_to_coord(idx);
        for r_num in 0..256 {
            // the idea here is jank but simple.  We can represent
            // every possible occupancy on a rank and file (and then some)
            // by interpreting all 8-bit numbers in binary as the occupied
            // spots
            for f_num in 0..256 {
                let mut occ_board = 0;
                // r_num fills a rank, f_num fills a file
                let mut r_num: u64 = r_num;
                let mut f_num: u64 = f_num;

                for f in 0..8 {
                    let v = r_num % 2;
                    r_num = r_num >> 1;
                    if v == 1 {
                        occ_board |= coord_to_bb((f, ry))
                    }
                }

                for r in 0..8 {
                    let v = f_num % 2;
                    f_num = f_num >> 1;
                    if v == 1 {
                        occ_board |= coord_to_bb((rx, r));
                    }
                }

                // get rid of the rook and mask
                occ_board &= rook_mask[idx as usize];

                let hash = rook_magic_hash(occ_board, idx as usize);
                let move_map = get_rook_moves_from_occ((rx, ry), occ_board);
                unsafe {
                    ROOK_MAGIC_TABLE[idx as usize][hash as usize] = move_map;
                }
            }
        }
    }
}

fn initialize_bishop_magic_table() {
    let bishop_mask = slow_gen_bishop_mask();
    for idx in 0..64 {
        let (bx, by) = idx_to_coord(idx);
        for i in 0..256 {
            for j in 0..256 {
                // the idea here is similar to the rook one.
                // we instead populate the two diagonals
                let mut occ_board = 0;
                // i takes up one diagonal, j takes up the other
                // get the bottom-most coordinates of the diagonals
                let start_coords = get_bishop_start_coords((bx, by));

                let mut i_coord = start_coords.0;
                let mut j_coord = start_coords.1;
                let mut i_num: u64 = i;
                let mut j_num: u64 = j;

                while i_num != 0 && i_coord.0 < 8 && i_coord.1 < 8 {
                    let v = i_num % 2;
                    i_num = i_num >> 1;
                    if v == 1 {
                        occ_board |= coord_to_bb(i_coord);
                    }
                    i_coord = (i_coord.0 + 1, i_coord.1 + 1)
                }

                while j_num != 0 && j_coord.0 < 8 && j_coord.1 < 8 {
                    let v = j_num % 2;
                    j_num = j_num >> 1;
                    if v == 1 {
                        occ_board |= coord_to_bb(j_coord);
                    }
                    j_coord = (j_coord.0 - 1, j_coord.1 + 1)
                }

                // get rid of bishop and mask
                occ_board &= bishop_mask[idx as usize];

                let hash = bishop_magic_hash(occ_board, idx as usize);
                let move_map = get_bishop_moves_from_occ((bx, by), occ_board);
                unsafe {
                    BISHOP_MAGIC_TABLE[idx as usize][hash as usize] = move_map;
                }
            }
        }
    }
}

pub fn initialize_magic_tables() {
    initialize_rook_magic_table();
    initialize_bishop_magic_table();
}
