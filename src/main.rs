#![feature(exclusive_range_pattern)]
#![feature(destructuring_assignment)]

use std::collections::VecDeque;
use std::io;
mod play;

fn print_deq(deq : &VecDeque<play::nodes::Move>) {
    for mv in deq.iter() {
        println!("{}", mv);
    }
}

struct Game {
    pub board: play::nodes::Node,
    white_turn: bool,
    zobrist_table: ([[u64; 12]; 64], (u64, u64))
}

impl Game {
    fn get_basic_game(zobrist_table: ([[u64; 12]; 64], (u64, u64))) -> Game {
        let mut board = play::nodes::Node::default_board();
        board.zobrist_table = zobrist_table;
        Game{
            board: board,
            white_turn: true,
            zobrist_table: zobrist_table
        }
    }

    fn file_to_number(file: char) -> i32 {
        return match file {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => -1
        }
    }

    fn receive_move(& mut self, mv: String) {
        // translate
        let move_bytes = mv.as_bytes();
        let start = ((move_bytes[0] - b'a') as i32, (move_bytes[1] - b'1') as i32);
        let end = ((move_bytes[2] - b'a') as i32, (move_bytes[3] - b'1') as i32);
        let mut mv : play::nodes::Move;
        if move_bytes.len() == 5 {
            // pawn promotion
            let promote_to = move_bytes[4];
            mv = play::nodes::Move::pawn_promote_move(&self.board, start, end, promote_to);
        } else {
            match self.board.state.get(&start) {
                Some(p) => {
                    if p.0 == b'p' {
                        if self.board.is_ep(start, end) {
                            mv = play::nodes::Move::ep_pawn_move(&self.board, start, end);
                        } else {
                            mv = play::nodes::Move::pawn_move(&self.board, start, end);
                        }
                    } else {
                        mv = play::nodes::Move::sliding_move(&self.board, start, end);
                    }
                },
                None => panic!("no piece at location to move!")
            };
        }
        self.board.do_move(&mv);
        self.white_turn = !self.white_turn;
        eprintln!("received move {}", mv);
        eprintln!("board state:\n{}", self.board.get_str());
        eprintln!("ep file: {}", self.board.ep);
    }

    unsafe fn make_move(& mut self, compute_time: u128) -> play::nodes::Move {
        let (best_move, val) = play::choose_move(& mut self.board, self.white_turn, compute_time);
        return best_move;
    }
}

fn main() {
    // let mut node: play::nodes::Node = play::nodes::Node::default_board();
    // let mut moves: VecDeque<play::nodes::Move> = node.moves();
    // // println!("{}", moves.len().to_string());
    // // print_deq(&moves);
    // // for mv in moves.drain(0..) {
    // //     let s: String = mv.repr.to_string();
    // //     println!("{}", s);
    // // }
    // let move1 = match moves.pop_front() {Some(m) => m, None => panic!("Out of moves!")};
    // node.do_move(&move1);
    // let moves2: VecDeque<play::nodes::Move> = node.moves();
    // // println!("{}", moves2.len().to_string());
    // // print_deq(&moves2);

    // node.undo_move(&move1);
    // let moves3: VecDeque<play::nodes::Move> = node.moves();
    // // println!("{}", moves3.len().to_string());
    // // print_deq(&moves3);

    // // println!("node mat {}", node.material);
    // unsafe {
    //     let (best_move, val) = play::choose_move(& mut node, true, 4000);
    //     println!("MV {}, {}", best_move, val);
    // }
    unsafe {
        play()
    }
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

unsafe fn play() {
    let zobrist = play::nodes::Node::init_zobrist();
    let mut game : Game = Game::get_basic_game(zobrist);
    loop {
        let mut inp : String = String::new();
        io::stdin()
            .read_line(&mut inp)
            .expect("Failed to read line");

        let mut params = inp.trim().split_whitespace();
        let cmd = match params.next() {
            Some(p) => p,
            None => {continue;}
        };
        // println!("CMD {}", cmd);
        if cmd == "quit" {
            break;
        }
        if cmd == "uci" {
            println!("id name Mantissa");
            println!("id author jtwright");
            println!("uciok");
        }
        if cmd == "isready" {
            println!("readyok");
        }
        if cmd == "setoption" {}
        if cmd == "ucinewgame" {
            game = Game::get_basic_game(zobrist);
        }
        if cmd == "position" {
            match params.next() {
                Some(p) => {
                    if p == "startpos" {
                        game = Game::get_basic_game(zobrist);
                        if params.next() == Some("moves") {
                            loop {
                                match params.next() {
                                    Some(mv) => {game.receive_move(mv.to_string());},
                                    None => {break;}
                                };
                            }
                        }
                    }
                },
                None => { continue; }
            };
        }
        if cmd == "go" {
            let clock_key = if game.board.white_turn {"wtime"} else {"btime"};
            let mut time = 10000;
            loop {
                let p = match params.next() {
                    Some(p) => p,
                    None => { break; }
                };
                if p == clock_key {
                    let clock_time = match params.next() {
                        Some(p) => match p.trim().parse() {
                            Ok(num) => num,
                            Err(_) => panic!("error in time parse")
                        },
                        None => panic!("empty time!")
                    };
                    time = get_calc_time(clock_time);
                }
            }
            let mv = game.make_move(time);
            println!("bestmove {}", mv);
        }
    }
}
