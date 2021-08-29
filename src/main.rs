mod bitboard; use crate::bitboard::*;
mod eval; use crate::eval::*;
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
    // let mut starting_board = Bitboard::default_board();
    println!("{}", bb_str(starting_board.composite[0] | starting_board.composite[1]));

    println!("SCORE {}", evaluate_position(&starting_board, 0));
    println!("SCORE {}", evaluate_position(&starting_board, 128));
    println!("SCORE {}", evaluate_position(&starting_board, 256));
    perft(&mut starting_board, 4, 0);
    let mut i = 0;
    unsafe {
        for n in PERFT_NODES {
            println!("{} {}", i, n);
            i += 1;
        }
    }
}
