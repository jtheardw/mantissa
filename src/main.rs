#![feature(exclusive_range_pattern)]
#![feature(destructuring_assignment)]

// use std::collections::VecDeque;
// use std::io;
mod play;

// fn print_deq(deq : &VecDeque<play::nodes::Move>) {
//     for mv in deq.iter() {
//         println!("{}", mv);
//     }
// }

// struct Game {
//     pub board: play::nodes::Node,
//     white_turn: bool,
//     zobrist_table: ([[u64; 12]; 64], (u64, u64))
// }

// impl Game {
//     fn get_basic_game(zobrist_table: ([[u64; 12]; 64], (u64, u64))) -> Game {
//         let mut board = play::nodes::Node::default_board();
//         board.zobrist_table = zobrist_table;
//         Game{
//             board: board,
//             white_turn: true,
//             zobrist_table: zobrist_table
//         }
//     }

//     fn file_to_number(file: char) -> i32 {
//         return match file {
//             'a' => 0,
//             'b' => 1,
//             'c' => 2,
//             'd' => 3,
//             'e' => 4,
//             'f' => 5,
//             'g' => 6,
//             'h' => 7,
//             _ => -1
//         }
//     }

//     fn receive_move(& mut self, mv: String) {
//         // translate
//         let move_bytes = mv.as_bytes();
//         let start = ((move_bytes[0] - b'a') as i32, (move_bytes[1] - b'1') as i32);
//         let end = ((move_bytes[2] - b'a') as i32, (move_bytes[3] - b'1') as i32);
//         let mut mv : play::nodes::Move;
//         if move_bytes.len() == 5 {
//             // pawn promotion
//             let promote_to = move_bytes[4];
//             mv = play::nodes::Move::pawn_promote_move(&self.board, start, end, promote_to);
//         } else {
//             match self.board.state.get(&start) {
//                 Some(p) => {
//                     if p.0 == b'p' {
//                         if self.board.is_ep(start, end) {
//                             mv = play::nodes::Move::ep_pawn_move(&self.board, start, end);
//                         } else {
//                             mv = play::nodes::Move::pawn_move(&self.board, start, end);
//                         }
//                     } else {
//                         mv = play::nodes::Move::sliding_move(&self.board, start, end);
//                     }
//                 },
//                 None => panic!("no piece at location to move!")
//             };
//         }
//         self.board.do_move(&mv);
//         self.white_turn = !self.white_turn;
//     }

//     unsafe fn make_move(& mut self, compute_time: u128) -> play::nodes::Move {
//         eprintln!("board state:\n{}", self.board.get_str());
//         let (best_move, val) = play::choose_move(& mut self.board, self.white_turn, compute_time, false);
//         self.board.do_move(&best_move);
//         let is_check = self.board.is_check(self.white_turn);
//         self.board.undo_move(&best_move);
//         if is_check {
//             return play::choose_move(& mut self.board, self.white_turn, compute_time, true).0;
//         }
//         eprintln!("best move is {}", best_move);
//         return best_move;
//     }
// }

fn main() {
    // unsafe {
    //     play()
    // }
    unsafe {
        bb_test();
    }
}

unsafe fn bb_test() {
    let nm = play::bb::BB::gen_knight_mask();
    let rm = play::bb::BB::gen_rook_mask();
    let bm = play::bb::BB::gen_bishop_mask();
    let km = play::bb::BB::gen_king_mask();

    // let bb = play::bb::BB::default_board(nm, rm, bm, km);
    let r_magic = play::bb::BB::gen_rook_magic_table();
    let b_magic = play::bb::BB::gen_bishop_magic_table();

    let mut db = play::bb::BB::default_board(nm, rm, bm, km, r_magic, b_magic);

    let mut nm = db.knight_moves(true);
    println!("knight moves");
    for mv in nm.drain(0..) {
        println!("{}", mv);
    }

    let mut pm = db.pawn_moves(true);
    println!("pawn moves");
    for mv in pm.drain(0..) {
        println!("{}", mv);
    }

    let mut rm = db.rook_moves(true);
    println!("rook moves");
    for mv in rm.drain(0..) {
        println!("{}", mv);
    }

    let mut bm = db.bishop_moves(true);
    println!("bishop moves");
    for mv in bm.drain(0..) {
        println!("{}", mv);
    }

    let mut km = db.king_moves(true);
    println!("king moves");
    for mv in km.drain(0..) {
        println!("{}", mv);
    }

    let mut qm = db.queen_moves(true);
    println!("queen moves");
    for mv in qm.drain(0..) {
        println!("{}", mv);
    }

    // let mut nm = db.knight_moves(true);
    // println!("knight moves");
    // for mv in nm.drain(0..) {
    //     db.do_move(&mv);
    //     let mut nm = db.knight_moves(true);
    //     db.undo_move(&mv);
    //     for mv in nm.drain(0..) {
    //         println!("{}", mv);
    //     }
    //     break;
    // }

    let (mv, val) = play::best_move(&mut db, true, 10000);
    println!("best move {} val {}", mv, val);
}

fn get_calc_time(time: i32) -> u128 {
    let mut calc_time = 10000;
     // ten minutes
    if time < 60 * 10 * 1000 {
        calc_time = time / 40;
    }
    if calc_time > 10000 {
        calc_time = 10000;
    }
    if calc_time < 1000 {
        calc_time = 1000;
    }
    return calc_time as u128;
}

// unsafe fn play() {
//     let zobrist = play::nodes::Node::init_zobrist();
//     let mut game : Game = Game::get_basic_game(zobrist);
//     loop {
//         let mut inp : String = String::new();
//         io::stdin()
//             .read_line(&mut inp)
//             .expect("Failed to read line");

//         let mut params = inp.trim().split_whitespace();
//         let cmd = match params.next() {
//             Some(p) => p,
//             None => {continue;}
//         };
//         // println!("CMD {}", cmd);
//         if cmd == "quit" {
//             break;
//         }
//         if cmd == "uci" {
//             println!("id name Mantissa");
//             println!("id author jtwright");
//             println!("uciok");
//         }
//         if cmd == "isready" {
//             println!("readyok");
//         }
//         if cmd == "setoption" {}
//         if cmd == "ucinewgame" {
//             game = Game::get_basic_game(zobrist);
//         }
//         if cmd == "position" {
//             match params.next() {
//                 Some(p) => {
//                     if p == "startpos" {
//                         game = Game::get_basic_game(zobrist);
//                         if params.next() == Some("moves") {
//                             loop {
//                                 match params.next() {
//                                     Some(mv) => {game.receive_move(mv.to_string());},
//                                     None => {break;}
//                                 };
//                             }
//                         }
//                     }
//                 },
//                 None => { continue; }
//             };
//         }
//         if cmd == "go" {
//             let clock_key = if game.board.white_turn {"wtime"} else {"btime"};
//             let mut time = 10000;
//             loop {
//                 let p = match params.next() {
//                     Some(p) => p,
//                     None => { break; }
//                 };
//                 if p == clock_key {
//                     let clock_time = match params.next() {
//                         Some(p) => match p.trim().parse() {
//                             Ok(num) => num,
//                             Err(_) => panic!("error in time parse")
//                         },
//                         None => panic!("empty time!")
//                     };
//                     time = get_calc_time(clock_time);
//                 }
//             }
//             let mv = game.make_move(time);
//             println!("bestmove {}", mv);
//         }
//     }
// }
