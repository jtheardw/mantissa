mod magic; use crate::magic::*;
mod util; use crate::util::*;
mod zobrist; use crate::zobrist::*;

fn init() {
    initialize_magic_tables();
    initialize_zobrist_table();
    // initialize_transposition_table();
}

fn main() {
    init();
}
