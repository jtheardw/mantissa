use crate::bitboard::*;
use crate::moveorder::*;
use crate::moveutil::*;

pub static mut PERFT_NODES : [usize; 16] = [0; 16];

pub fn perft(pos: &mut Bitboard, depth: i8, ply: usize) {
    unsafe { PERFT_NODES[ply] += 1; }
    if depth < 0 { return; }
    let mut movepicker = MovePicker::perft_new();
    loop {
        let (mv, _score) = movepicker.next(pos);
        if mv.is_null {
            break;
        }
        pos.do_move(&mv);
        if !pos.is_check(!pos.side_to_move) {
            perft(pos, depth - 1, ply + 1);
        }
        pos.undo_move(&mv);
    }
}
