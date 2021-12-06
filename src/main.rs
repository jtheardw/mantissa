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
mod nnue;
mod pgn;
mod pht;
mod psqt;
mod search;
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
    unsafe {
        let mut nn = Network::load("/home/jtwright/chess/mantissa/epoch-209.nnue").unwrap();
        let mut slow_nn = SlowNetwork::load("/home/jtwright/chess/mantissa/epoch-209.nnue").unwrap();
        let b = Bitboard::default_board();
        // nn.set_activations(&b);
        // slow_nn.set_activations(&b);
        // println!("BLAH");

        // nn.nnue_eval();
        // slow_nn.nnue_eval();
    }
    // set_default_net(nn);
    uci_loop();
}
