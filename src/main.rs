#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![feature(exclusive_range_pattern)]

mod bitboard;
mod eval;
mod magic;
mod movegen;
mod moveorder;
mod moveutil;
mod perft;
mod pht;
mod psqt;
mod rand;
mod search;
mod searchutil;
mod see;
mod tuning_eval;
mod tuning_psqt;
mod tuning;
mod time;
mod tt;
mod uci;
mod util;
mod zobrist;

use crate::magic::*;
use crate::movegen::*;
use crate::pht::*;
use crate::tt::*;
use crate::uci::*;
use crate::util::*;

fn init() {
    initialize_masks();
    initialize_magic_tables();
    initialize_pht();
    allocate_tt(64);
}

fn main() {
    init();
    uci_loop();
}
