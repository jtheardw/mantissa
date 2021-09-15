#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![feature(exclusive_range_pattern)]

mod bitboard; use crate::bitboard::*;
mod eval; use crate::eval::*;
mod magic; use crate::magic::*;
mod movegen; use crate::movegen::*;
mod moveorder; use crate::moveorder::*;
mod moveutil; use crate::moveutil::*;
mod perft; use crate::perft::*;
mod pht; use crate::pht::*;
mod psqt; use crate::psqt::*;
mod rand;
mod search; use crate::search::*;
mod searchutil; use crate::searchutil::*;
mod see; use crate::see::*;
mod tuning_eval;
mod tuning_psqt;
mod tuning; use crate::tuning::*;
mod time; use crate::time::*;
mod tt; use crate::tt::*;
mod uci; use crate::uci::*;
mod util; use crate::util::*;
mod zobrist; use crate::zobrist::*;

fn init() {
    initialize_masks();
    initialize_magic_tables();
    initialize_pht();
    allocate_tt(64);
}

fn main() {
    init();
    let mut v = get_position_vector("positions.fen");
    // let k = find_optimal_k(&mut v);
    // print_params_vector(&get_params_vector());
    // uci_loop();
    tune(&mut v);
}
