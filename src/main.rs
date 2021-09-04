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
    // let mut starting_board = Bitboard::from_position(format!("1k6/4R3/1p6/p1p3p1/qnBn2Qp/5P2/1P6/1K6 w - - 0 1"));
    // let mut starting_board = Bitboard::from_position(format!("8/p3b1kN/bp4p1/4P3/1P4Q1/P1qr2R1/5PP1/1K2R3 b - - 0 1"));
    // let mut starting_board = Bitboard::from_position(format!("r2q2k1/p5bp/bp4p1/4P3/4B3/P4N2/1PPQ1PP1/2K1R2R w - - 0 1"));
    // let mut starting_board = Bitboard::from_position(format!("r1b3k1/pp4pp/4p3/3p4/3N4/8/PPPq1rPP/2K2B1R w - - 0 19"));
    // let mut starting_board = Bitboard::from_position(format!("8/1p6/8/6k1/8/3r2p1/3p2Kp/3R4 w - - 2 65"));
    let mut starting_board = Bitboard::from_position(format!("r1b1k2r/5pp1/p3p2p/q1pnP3/1b1B2PP/P1NB1P2/1PPQ4/R4RK1 b kq - 0 1"));
    println!("{}", starting_board.get_phase());
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
    print_value(&starting_board);
    best_move(&mut starting_board, 10000);
}
