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
    // let mut board = Bitboard::from_position(format!("r1bqk2r/pP2b1pp/n4p1n/6N1/2BP4/8/PPPQpPPP/RNB2RK1 b kq - 0 12"));
    // let moves = moves(&board);
    // for mv in moves {
    //     println!("{}", mv);
    // }

    let mut starting_board = Bitboard::default_board();
    perft(&mut starting_board, 6, 0);
    let mut i = 0;
    unsafe {
        for n in PERFT_NODES {
            println!("{} {}", i, n);
            i += 1;
        }
    }
}
