#![feature(portable_simd)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(non_upper_case_globals)]
// #![feature(exclusive_range_pattern)]


mod bitboard;
mod default_nnue;
mod eval;
mod evalutil;
mod magic;
mod movegen;
mod moveorder;
mod moveutil;
mod nnue;
mod pgn;
mod pht;
mod psqt;
mod search;
mod searchparams;
mod searchutil;
mod see;
mod time;
mod tt;
mod uci;
mod util;
mod zobrist;

use crate::bitboard::*;
use crate::evalutil::*;
use crate::magic::*;
use crate::movegen::*;
use crate::nnue::*;
use crate::pgn::*;
use crate::pht::*;
use crate::searchutil::*;
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
}
