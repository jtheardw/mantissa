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
mod perft;
mod pgn;
mod pht;
mod psqt;
mod search;
mod searchparams;
mod searchutil;
mod see;
mod syzygy;
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
use crate::perft::*;
use crate::pgn::*;
use crate::pht::*;
use crate::syzygy::*;
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
    // let n = SlowNetwork::load("/home/jtwright/chess/mantissa/epoch-65.nnue").unwrap();
    // n.save_image("net");
    // n.print();

    // let n = SlowNetwork::load("/home/jtwright/nets_50_50/epoch-235.nnue").unwrap();
    // n.print();
    // unsafe {
    //     setup_tb("/home/jtwright/chess/tablebase/3-4-5/");
    // }
    uci_loop();
}
