#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(non_upper_case_globals)]
// #![feature(exclusive_range_pattern)]


mod bitboard;
mod eval;
mod evalutil;
mod magic;
mod movegen;
mod moveorder;
mod moveutil;
mod pgn;
mod pht;
mod psqt;
// mod rand;
mod search;
mod searchutil;
mod see;
// mod tuning;
// mod tuning_eval;
// mod tuning_psqt;
mod time;
mod tt;
mod uci;
mod util;
mod zobrist;

use crate::bitboard::*;
use crate::evalutil::*;
use crate::magic::*;
use crate::movegen::*;
// use crate::perft::*;
use crate::pgn::*;
use crate::pht::*;
use crate::searchutil::*;
// use crate::tuning::*;
use crate::tt::*;
use crate::uci::*;
use crate::util::*;

fn init() {
    initialize_masks();
    initialize_magic_tables();
    initialize_pht();
    initialize_eval_masks();
    lmr_table_gen();
    allocate_tt(64);
}

fn main() {
    init();
    // let mut board = Bitboard::from_position(format!("rnb1qbn1/pppPpppp/2Np4/3N4/1PRQ2QR/1PP5/PPK4P/1BBQ2rk w - - 0 1"));
    // let board = Bitboard::default_board();
    // board.fen();
    // println!("{}", san_to_move(&mut board, format!("dxe8=N")));
    convert_pgn("../pgns/self-play-data.pgn");
    // println!("{}", san_to_move(&mut board, format!("Rhd4")));
    // uci_loop();
    // let mut v = get_position_vector("eth2.book");
    // let _ = tune(&mut v);
}
