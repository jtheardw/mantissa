#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unreachable_code)]

mod bitboard; use crate::bitboard::*;
mod eval; use crate::eval::*;
mod magic; use crate::magic::*;
mod movegen; use crate::movegen::*;
mod moveorder; use crate::moveorder::*;
mod moveutil; use crate::moveutil::*;
mod perft; use crate::perft::*;
mod pht; use crate::pht::*;
mod search; use crate::search::*;
mod searchutil; use crate::searchutil::*;
mod tt; use crate::tt::*;
mod util; use crate::util::*;
mod zobrist; use crate::zobrist::*;

fn init() {
    initialize_masks();
    initialize_magic_tables();
    initialize_zobrist_table();
    initialize_pht();
    allocate_tt(1024);
}

fn main() {
    init();
    // let mut starting_board = Bitboard::default_board();
    // let mut starting_board = Bitboard::from_position(format!("1k6/4R3/1p6/p1p3p1/qn4Qp/1n3P2/1P6/1K6 w - - 0 24"));
    let mut starting_board = Bitboard::from_position(format!("1k6/4R3/1p6/p1p3p1/qnBn2Qp/5P2/1P6/1K6 w - - 0 1"));
    // let mut starting_board = Bitboard::default_board();
    println!("{}", bb_str(starting_board.composite[0] | starting_board.composite[1]));

    // println!("SCORE {}", evaluate_position(&starting_board, 0));
    // println!("SCORE {}", evaluate_position(&starting_board, 128));
    // println!("SCORE {}", evaluate_position(&starting_board, 256));
    // perft(&mut starting_board, 4, 0);
    // let mut i = 0;
    // unsafe {
    //     for n in PERFT_NODES {
    //         println!("{} {}", i, n);
    //         i += 1;
    //     }
    // }

    best_move(&mut starting_board, 30000);
}
