use crate::movegen::*;
use crate::util::*;

const white: usize = 1;
const black: usize = 0;

pub static mut ADJACENT_FILE_MASKS: [u64; 8] = [0; 8];
pub static mut AHEAD_RANK_MASKS: [[u64; 8]; 2] = [[0; 8]; 2];
pub static mut PASSED_PAWN_MASKS: [[u64; 64]; 2] = [[0; 64]; 2];

pub static mut KING_RING_MASKS: [u64; 64] = [0; 64];

pub fn initialize_eval_masks() {
    unsafe {
        adjacent_file_mask_init();
        ahead_rank_mask_init();
        passed_pawn_mask_init();
        king_ring_mask_init();
    }
}

unsafe fn adjacent_file_mask_init() {
    for i in 0..8 {
        ADJACENT_FILE_MASKS[i] = if i > 0 {FILE_MASKS[i-1]} else {0} | if i < 7 {FILE_MASKS[i+1]} else {0};
    }
}

unsafe fn ahead_rank_mask_init() {
    for i in 0..8 {
        let mut white_mask = 0;
        let mut black_mask = 0;
        for j in (i+1)..8 {
            // white
            white_mask |= RANK_MASKS[j];
        }
        for j in 0..i {
            // black
            black_mask |= RANK_MASKS[j];
        }

        AHEAD_RANK_MASKS[black][i] = black_mask;
        AHEAD_RANK_MASKS[white][i] = white_mask;

    }
}

unsafe fn passed_pawn_mask_init() {
    for i in 0..64 {
        let r = i / 8;
        let f = i % 8;
        PASSED_PAWN_MASKS[white][i] = AHEAD_RANK_MASKS[white][r] & (FILE_MASKS[f] | ADJACENT_FILE_MASKS[f]);
        PASSED_PAWN_MASKS[black][i] = AHEAD_RANK_MASKS[black][r] & (FILE_MASKS[f] | ADJACENT_FILE_MASKS[f]);
    }
}

unsafe fn king_ring_mask_init() {
    for i in 0..64 {
        let mut king_virt_idx = i;
        if i % 8 == 7 {
            king_virt_idx -= 1;
        } else if i % 8 == 0 {
            king_virt_idx += 1;
        }

        if i / 8 == 7 {
            king_virt_idx /= 2;
        } else if i / 8 == 0 {
            king_virt_idx *= 2;
        }

        KING_RING_MASKS[i] == KING_MASK[king_virt_idx];
    }
}
