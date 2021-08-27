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
    let mut starting_board = Bitboard::from_position(format!("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"));
    // let moves = moves(&board);
    // for mv in moves {
    //     println!("{}", mv);
    // }

    // let mut starting_board = Bitboard::default_board();
    perft(&mut starting_board, 4, 0);
    let mut i = 0;
    unsafe {
        for n in PERFT_NODES {
            println!("{} {}", i, n);
            i += 1;
        }
    }
}
