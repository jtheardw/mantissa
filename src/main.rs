#![feature(exclusive_range_pattern)]
#![feature(destructuring_assignment)]

use std::io;
mod engine;

const BH_NONE: u8 = 0;
const BH_BRAIN: u8 = 1;
const BH_HAND: u8 = 2;

struct Game {
    pub board: engine::bb::BB,
    pub eval_params: engine::bb::EvalParams,
    white_turn: bool,
    num_threads: usize,
    pub bh_mode: u8
}

impl Game {
    fn get_basic_game(
        num_threads: usize,
        knight_mask: [u64; 64],
        rook_mask: [u64; 64],
        bishop_mask: [u64; 64],
        king_mask: [u64; 64],
        zobrist_table: ([[u64; 12]; 64], (u64, u64))
    ) -> Game {

        let eval_params = engine::bb::EvalParams::default_params();
        let mut board = engine::bb::BB::default_board(
            knight_mask,
            rook_mask,
            bishop_mask,
            king_mask,
            zobrist_table,
            eval_params
        );

        Game{
            board: board,
            eval_params: eval_params,
            white_turn: true,
            num_threads: num_threads,
            bh_mode: BH_NONE
        }
    }

    fn update_param(& mut self, option: &str, value_str : &str) {
        let value : i32 = match value_str.trim().parse() {
            Ok(num) => num,
            Err(_) => {eprintln!("invalid param"); return;}
        };

        match option {
            "queen_mobility" => { self.eval_params.queen_mobility = value; },
            "rook_mobility" => { self.eval_params.rook_mobility = value; },
            "bishop_mobility" => { self.eval_params.bishop_mobility = value; },
            "knight_mobility" => { self.eval_params.knight_mobility = value; },
            "pdf" => { self.eval_params.pdf = value; },
            "dbb" => { self.eval_params.dbb = value; },
            "castle" => { self.eval_params.castle = value; },
            "pav" => { self.eval_params.pav = value; },
            "rook_on_seventh" => { self.eval_params.rook_on_seventh = value; },
            "rook_on_open" => { self.eval_params.rook_on_open = value; },
            "early_queen_penalty" => { self.eval_params.early_queen_penalty = value; },

            "eg_queen_mobility" => { self.eval_params.eg_queen_mobility = value; },
            "eg_rook_mobility" => { self.eval_params.eg_rook_mobility = value; },
            "eg_bishop_mobility" => { self.eval_params.eg_bishop_mobility = value; },
            "eg_knight_mobility" => { self.eval_params.eg_knight_mobility = value; },
            "eg_pdf" => { self.eval_params.eg_pdf = value; },
            "eg_dbb" => { self.eval_params.eg_dbb = value; },
            "eg_castle" => { self.eval_params.eg_castle = value; },
            "eg_pav" => { self.eval_params.eg_pav = value; },
            "eg_rook_on_seventh" => { self.eval_params.eg_rook_on_seventh = value; },
            "eg_rook_on_open" => { self.eval_params.eg_rook_on_open = value; },

            "passed_pawn" => { self.eval_params.passed_pawn = value; },
            "center_pawn" => { self.eval_params.center_pawn = value; },
            "near_center_pawn" => { self.eval_params.near_center_pawn = value; },
            "isolated_pawn" => { self.eval_params.isolated_pawn = value; },
            "doubled_pawn" => { self.eval_params.doubled_pawn = value; },
            "backwards_pawn" => { self.eval_params.backwards_pawn = value; },
            "supported_bonus" => { self.eval_params.supported_bonus = value; },
            "advancement_bonus" => { self.eval_params.advancement_bonus = value; },
            "space" => { self.eval_params.space = value; },
            "bishop_color" => { self.eval_params.bishop_color = value; }

            "eg_passed_pawn" => { self.eval_params.eg_passed_pawn = value; },
            "eg_center_pawn" => { self.eval_params.eg_center_pawn = value; },
            "eg_near_center_pawn" => { self.eval_params.eg_near_center_pawn = value; },
            "eg_isolated_pawn" => { self.eval_params.eg_isolated_pawn = value; },
            "eg_doubled_pawn" => { self.eval_params.eg_doubled_pawn = value; },
            "eg_backwards_pawn" => { self.eval_params.eg_backwards_pawn = value; },
            "eg_supported_bonus" => { self.eval_params.eg_supported_bonus = value; },
            "eg_advancement_bonus" => { self.eval_params.eg_advancement_bonus = value; },
            "eg_space" => { self.eval_params.eg_space = value; },
            "eg_bishop_color" => { self.eval_params.eg_bishop_color = value; }

            "pawn_pt_offset" => { self.eval_params.pawn_pt_offset = value; },
            "pawn_pt_scale" => { self.eval_params.pawn_pt_scale = value; },

            "bishop_pt_offset" => { self.eval_params.bishop_pt_offset = value; },
            "bishop_pt_scale" => { self.eval_params.bishop_pt_scale = value; },

            "knight_pt_offset" => { self.eval_params.knight_pt_offset = value; },
            "knight_pt_scale" => { self.eval_params.knight_pt_scale = value; },

            "eg_pawn_pt_offset" => { self.eval_params.eg_pawn_pt_offset = value; },
            "eg_pawn_pt_scale" => { self.eval_params.eg_pawn_pt_scale = value; },

            "eg_bishop_pt_offset" => { self.eval_params.eg_bishop_pt_offset = value; },
            "eg_bishop_pt_scale" => { self.eval_params.eg_bishop_pt_scale = value; },

            "eg_knight_pt_offset" => { self.eval_params.eg_knight_pt_offset = value; },
            "eg_knight_pt_scale" => { self.eval_params.eg_knight_pt_scale = value; },

            "king_mg_pt_offset" => { self.eval_params.king_mg_pt_offset = value; },
            "king_mg_pt_scale" => { self.eval_params.king_mg_pt_scale = value; },

            "king_eg_pt_offset" => { self.eval_params.king_eg_pt_offset = value; },
            "king_eg_pt_scale" => { self.eval_params.king_eg_pt_scale = value; },

            "tempo_bonus" => { self.eval_params.tempo_bonus = value; },
            "material_advantage" => { self.eval_params.material_advantage = value; },
            "king_danger" => { self.eval_params.king_danger = value; },
            "eg_king_danger" => { self.eval_params.eg_king_danger = value; },
            _ => {}
        }

        self.board.eval_params = self.eval_params;
    }

    fn reset(& mut self) {
        self.board.reset();
        self.board.eval_params = self.eval_params;
        self.white_turn = true;
    }

    fn reset_fen(& mut self, fen: String) {
        self.board.reset_from_position(fen);
        self.board.eval_params = self.eval_params;
        self.white_turn = self.board.white_turn;
    }

    fn receive_move(& mut self, mv: String) {
        // translate
        let move_bytes = mv.as_bytes();
        let start = engine::bb::BB::coord_to_idx(((move_bytes[0] - b'a') as i32, (move_bytes[1] - b'1') as i32));
        let end = engine::bb::BB::coord_to_idx(((move_bytes[2] - b'a') as i32, (move_bytes[3] - b'1') as i32));
        let mv: engine::bb::Mv;
        if move_bytes.len() == 5 {
            // pawn promotion
            let promote_to = move_bytes[4];
            mv = engine::bb::Mv::pawn_promote_move(start, end, promote_to);
        } else {
            let piece = self.board.get_piece_at_idx(start);
            if piece == b'p' {
                if self.board.is_ep(start, end) {
                    mv = engine::bb::Mv::pawn_ep_move(start, end);
                } else {
                    mv = engine::bb::Mv::pawn_move(start, end);
                }
            } else if piece > 0 {
                mv = engine::bb::Mv::normal_move(start, end, piece);
            } else {
                panic!("no piece at that location!");
            }
        }
        self.board.do_move(&mv);
        self.white_turn = !self.white_turn;
    }

    unsafe fn make_move(&mut self, compute_time: u128, hand_piece: i32) -> engine::bb::Mv {
        if self.bh_mode == BH_NONE {
            eprintln!("current eval:");
            engine::print_evaluate(&self.board);
        }
        let (best_move, _) = engine::best_move(& mut self.board, self.white_turn, compute_time, self.num_threads, self.bh_mode, hand_piece);
        if self.bh_mode == BH_NONE {
            eprintln!("best move is {}", best_move);
        }
        return best_move;
    }

    unsafe fn eval(& mut self) -> i32 {
        let val = engine::q_eval(& mut self.board, self.white_turn);
        return val
    }
}

fn main() {
    unsafe {
        play()
    }
}

fn get_calc_time(time: i32, inc: i32, ply: i32) -> u128 {
    // All credit for this calculation goes to Kade Phillips and Thomas Ahle
    let p = ply as f64;
    let ply_remaining = 59.3 + (72830.0 - p*2330.0) / (p*p + p*10.0 + 2644.0);
    let moves_remaining = ply_remaining / 2.0;
    let mut calc_time = (((time - inc) as f64 / moves_remaining) as i32 + inc) as i32;

    if calc_time > 30000 {
        calc_time = 30000;
    }
    if calc_time > time - 100 {
        calc_time = time - 100;
    }
    if calc_time < 0 {
        calc_time = 0;          // single ply
    }
    return calc_time as u128;
}

unsafe fn play() {
    let nm = engine::bb::BB::gen_knight_mask();
    let rm = engine::bb::BB::gen_rook_mask();
    let bm = engine::bb::BB::gen_bishop_mask();
    let km = engine::bb::BB::gen_king_mask();

    let zobrist = engine::bb::BB::init_zobrist();
    let num_threads = 2;

    let mut game : Game = Game::get_basic_game(
        num_threads,
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
        else if cmd == "uci" {
            println!("id name Mantissa");
            println!("id author jtwright");
            println!("uciok");
        }
        else if cmd == "isready" {
            println!("readyok");
        }
        else if cmd == "setoption" {
            match params.next() {
                Some(o) => {
                    match params.next() {
                        Some(p) => {game.update_param(o, p);}
                        None => {}
                    }
                }
                None => {}
            }
        } else if cmd == "bh_mode" {
            match params.next() {
                Some(mode) => match mode {
                    "brain" => {game.bh_mode = BH_BRAIN;},
                    "hand" => {game.bh_mode = BH_HAND;},
                    "none" => {game.bh_mode = BH_NONE;},
                    _ => {}
                },
                None => {}
            };
        } else if cmd == "ucinewgame" {
            game.reset();
        }
        else if cmd == "position" {
            match params.next() {
                Some(p) => {
                    if p == "startpos" {
                        game.reset();
                    } else if p == "fen" {
                        let mut position_str: String = String::from("");
                        for i in 0..6 {
                            match params.next() {
                                Some(param) => {position_str.push_str(param); position_str.push_str(" ");}
                                None => {panic!("bad fen");}
                            }
                        }
                        game.reset_fen(position_str);
                    }
                    if params.next() == Some("moves") {
                        loop {
                            match params.next() {
                                Some(mv) => {game.receive_move(mv.to_string());},
                                None => {break;}
                            };
                        }
                    }
                },
                None => { continue; }
            };
        }
        else if cmd == "go" {
            let clock_key = if game.white_turn {"wtime"} else {"btime"};
            let inc_key = if game.white_turn {"winc"} else {"binc"};
            let mut on_clock = false;
            let mut clock_time = 0;
            let mut inc_time = 0;
            let mut time = 10000;
            let mut hand_piece: i32 = -1;
            loop {
                let p = match params.next() {
                    Some(p) => p,
                    None => { break; }
                };
                if p == clock_key {
                    clock_time = match params.next() {
                        Some(p) => match p.trim().parse() {
                            Ok(num) => num,
                            Err(_) => panic!("error in time parse")
                        },
                        None => panic!("empty time!")
                    };
                    on_clock = true;
                    // time = get_calc_time(clock_time);
                } else if p == inc_key {
                    inc_time = match params.next() {
                        Some(p) => match p.trim().parse() {
                            Ok(num) => num,
                            Err(_) => panic!("error in time parse")
                        },
                        None => panic!("empty time!")
                    };
                } else if p == "movetime" {
                    time = match params.next() {
                        Some(p) => match p.trim().parse() {
                            Ok(num) => num,
                            Err(_) => panic!("error in time parse")
                        },
                        None => panic!("empty time!")
                    };
                } else if p == "hand_piece" {
                    hand_piece = match params.next() {
                        Some(p) => engine::bb::BB::str_to_idx(String::from(p.trim())),
                        None => panic!("empty hand piece!")
                    };
                }
            }
            if on_clock { time = get_calc_time(clock_time, inc_time, game.board.history.len() as i32); }
            let mv = game.make_move(time, hand_piece);
            if game.bh_mode == BH_NONE {
                println!("bestmove {}", mv);
            }
        }
        else if cmd == "eval" {
            let val: i32 = game.eval();
            println!("value {}", val);
        }
    }
}
