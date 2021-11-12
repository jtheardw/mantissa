#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![feature(exclusive_range_pattern)]

mod bitboard;
mod eval;
mod evalutil;
mod magic;
mod movegen;
mod moveorder;
mod moveutil;
mod pht;
mod psqt;
mod rand;
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
    uci_loop();
    // let mut v = get_position_vector("eth2.book");
    // let _ = tune(&mut v);
}
