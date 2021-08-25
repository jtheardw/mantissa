mod bitboard; use crate::bitboard::*;
mod magic; use crate::magic::*;
mod movegen; use crate::movegen::*;
mod perft; use crate::perft::*;
mod util; use crate::util::*;
mod zobrist; use crate::zobrist::*;

fn init() {
    initialize_masks();
    initialize_magic_tables();
    initialize_zobrist_table();
    // initialize_transposition_table();
}

fn main() {
    init();
    // let mut board = Bitboard::from_position(format!("rnbqkbnr/ppp3pp/3p1p2/4p3/4P3/3B1N2/PPPP1PPP/RNBQK2R w KQkq - 0 4"));
    // let moves = moves(&board);
    // for mv in moves {
    //     println!("{}", mv);
    // }

    let mut starting_board = Bitboard::default_board();
    perft(&mut starting_board, 6, 0);
    let mut i = 0;
    unsafe {
        for n in PERFT_NODES {
            println!("{} {}", i, n);
            i += 1;
        }
    }
}
