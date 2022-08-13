use crate::bitboard::*;
use crate::movegen::*;
use crate::moveorder::*;
use crate::moveutil::*;

pub static mut PERFT_NODES : [usize; 16] = [0; 16];

pub fn perft(pos: &mut Bitboard, depth: i8, ply: usize) {
    unsafe { PERFT_NODES[ply] += 1; }
    if depth < 0 { return; }
    let moves = moves(pos);
    // let mut movepicker = MovePicker::perft_new();
    for mv in moves {
        if !pos.do_move_legal(&mv) {
            continue;
        }
        perft(pos, depth - 1, ply + 1);
        pos.undo_move(&mv);
    }
}
