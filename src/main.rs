mod bitboard; use crate::bitboard::*;
mod magic; use crate::magic::*;
mod movegen; use crate::movegen::*;
mod perft; use crate::perft::*;
mod util; use crate::util::*;
mod zobrist; use crate::zobrist::*;

fn init() {
    initialize_masks();
    initialize_magic_tables();
    initialize_zobrist_table();
    // initialize_transposition_table();
}

fn main() {
    init();
    let mut starting_board = Bitboard::default_board();
    perft(&mut starting_board, 5, 0);
    let mut i = 1;
    unsafe {
        for n in PERFT_NODES {
            println!("{} {}", i, n);
            i += 1;
        }
    }
}
