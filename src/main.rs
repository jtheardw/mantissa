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
    let mut starting_board = Bitboard::from_position(format!("rnbqkbn1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB2 w KQq - 0 1"));
    // let moves = moves(&board);
    // for mv in moves {
    //     println!("{}", mv);
    // }

    // let mut starting_board = Bitboard::default_board();
    println!("SCORE {}", evaluate_position(&starting_board, 0));
    println!("SCORE {}", evaluate_position(&starting_board, 256));
    // perft(&mut starting_board, 5, 0);
    // let mut i = 0;
    // unsafe {
    //     for n in PERFT_NODES {
    //         println!("{} {}", i, n);
    //         i += 1;
    //     }
    // }
}
