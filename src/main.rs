#![feature(exclusive_range_pattern)]
#![feature(destructuring_assignment)]

use std::io;
mod play;

struct Game {
    pub board: play::bb::BB,
    white_turn: bool,
}

impl Game {
    fn get_basic_game(
        knight_mask: [u64; 64],
        rook_mask: [u64; 64],
        bishop_mask: [u64; 64],
        king_mask: [u64; 64],
        zobrist_table: ([[u64; 12]; 64], (u64, u64))
    ) -> Game {

        let board = play::bb::BB::default_board(
            knight_mask,
            rook_mask,
            bishop_mask,
            king_mask,
            zobrist_table
        );
        Game{
            board: board,
            white_turn: true,
        }
    }

    fn reset(& mut self) {
        self.board.reset();
        self.white_turn = true;
    }

    fn receive_move(& mut self, mv: String) {
        // translate
        let move_bytes = mv.as_bytes();
        let start = play::bb::BB::coord_to_idx(((move_bytes[0] - b'a') as i32, (move_bytes[1] - b'1') as i32));
        let end = play::bb::BB::coord_to_idx(((move_bytes[2] - b'a') as i32, (move_bytes[3] - b'1') as i32));
        let mv: play::bb::Mv;
        if move_bytes.len() == 5 {
            // pawn promotion
            let promote_to = move_bytes[4];
            mv = play::bb::Mv::pawn_promote_move(start, end, promote_to);
        } else {
            let piece = self.board.get_piece_at_idx(start);
            if piece == b'p' {
                if self.board.is_ep(start, end) {
                    mv = play::bb::Mv::pawn_ep_move(start, end);
                } else {
                    mv = play::bb::Mv::pawn_move(start, end);
                }
            } else if piece > 0 {
                mv = play::bb::Mv::normal_move(start, end, piece);
            } else {
                panic!("no piece at that location!");
            }
        }
        self.board.do_move(&mv);
        self.white_turn = !self.white_turn;
    }

    unsafe fn make_move(& mut self, compute_time: u128) -> play::bb::Mv {
        eprintln!("current eval:");
        play::print_evaluate(&self.board);
        let (best_move, _) = play::best_move(& mut self.board, self.white_turn, compute_time);
        eprintln!("best move is {}", best_move);
        return best_move;
    }
}

fn main() {
    unsafe {
        play()
    }
}

fn get_calc_time(time: i32) -> u128 {
    let mut calc_time = 30000;
     // ten minutes
    calc_time = time / 30;
    if calc_time > 30000 {
        calc_time = 30000;
    }
    if calc_time < 250 {
        calc_time = 250;
    }
    return calc_time as u128;
}

unsafe fn play() {
    let nm = play::bb::BB::gen_knight_mask();
    let rm = play::bb::BB::gen_rook_mask();
    let bm = play::bb::BB::gen_bishop_mask();
    let km = play::bb::BB::gen_king_mask();

    let zobrist = play::bb::BB::init_zobrist();

    let mut game : Game = Game::get_basic_game(
        nm,
        rm,
        bm,
        km,
        zobrist
    );

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
            game.reset();
        }
        if cmd == "position" {
            match params.next() {
                Some(p) => {
                    if p == "startpos" {
                        game.reset();
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
