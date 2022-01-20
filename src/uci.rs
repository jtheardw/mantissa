use std::io;
use std::str::SplitWhitespace;
use std::thread;

use crate::bitboard::*;
use crate::eval::*;
use crate::moveutil::*;
use crate::pht::*;
use crate::search::*;
use crate::searchutil::*;
use crate::util::*;
use crate::tt::*;

#[derive(Copy, Clone)]
struct UCIOptions {
    pub num_threads: u16,
    pub move_overhead: i32,
    pub hash: i32
}

fn uci_go(board: &Bitboard, options: UCIOptions, params: &mut SplitWhitespace) {
    if ongoing_search() { println!("ERR: Search still ongoing."); return; }

    let mut search_limit = SearchLimits::movetime(10000);
    let clock_key = if board.side_to_move == Color::White {"wtime"} else {"btime"};
    let inc_key = if board.side_to_move == Color::White {"wtime"} else {"btime"};

    let mut clock_time = 0;
    let mut inc = 0;
    let mut moves_to_go = -1;

    let mut movetime = 0;
    let mut depth = -1;

    let mut infinite = false;

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
        } else if p == inc_key {
            inc = match params.next() {
                Some(p) => match p.trim().parse() {
                    Ok(num) => num,
                    Err(_) => panic!("error in time parse")
                },
                None => panic!("empty time!")
            };
        } else if p == "movetime" {
            movetime = match params.next() {
                Some(p) => match p.trim().parse() {
                    Ok(num) => num,
                    Err(_) => panic!("error in time parse")
                },
                None => panic!("empty time!")
            };
        } else if p == "movestogo" {
            moves_to_go = match params.next() {
                Some(p) => match p.trim().parse() {
                    Ok(num) => num,
                    Err(_) => panic!("error in time parse")
                },
                None => panic!("empty time!")
            };
        } else if p == "depth" {
            depth = match params.next() {
                Some(p) => match p.trim().parse() {
                    Ok(num) => num,
                    Err(_) => panic!("error in time parse")
                },
                None => panic!("empty time!")
            };
        } else if p == "infinite" {
            infinite = true;
        }
    }

    if moves_to_go >= 0 {
        // moves to go move
        search_limit = SearchLimits::moves_to_go(clock_time, moves_to_go, options.move_overhead);
    } else if movetime > 0 {
        search_limit = SearchLimits::movetime(movetime);
    } else if clock_time > 0 {
        let ply = board.history.len() as i32;
        let mat = mg_score(material_score(board)) / 1000;
        search_limit = SearchLimits::clock_with_inc(clock_time, inc, options.move_overhead, ply, mat);
    } else if depth > 0 {
        search_limit = SearchLimits::depth(depth);
    } else if infinite {
        search_limit = SearchLimits::infinite();
    }

    let mut thread_board = board.thread_copy();
    thread::spawn(move || {
        best_move(&mut thread_board, options.num_threads, search_limit);
    });
}

fn apply_uci_move(board: &mut Bitboard, move_str: String) {
    let move_bytes = move_str.as_bytes();
    let start = bytes_to_idx(move_bytes[0], move_bytes[1]) as i8;
    let end = bytes_to_idx(move_bytes[2], move_bytes[3]) as i8;
    let mv: Move;
    if move_bytes.len() == 5 {
        // promotion
        let promote_to = move_bytes[4];
        mv = Move::promotion(start, end, promote_to);
    } else {
        let piece = board.piece_at_square(start as i8, board.side_to_move);
        if piece == b'p' {
            let is_sixth = if board.side_to_move == Color::White {
                (end / 8) == 5
            } else {
                (end / 8) == 2
            };
            if board.ep_file == (end % 8) as i32 && is_sixth {
                mv = Move::ep_capture(start, end);
            } else {
                mv = Move::pawn_move(start, end);
            }
        } else if piece > 0 {
            mv = Move::piece_move(start, end, piece);
        } else {
            panic!("no piece at that location!");
        }
    }

    board.do_move(&mv);
}

fn set_position(params: &mut SplitWhitespace) -> Bitboard {
    let mut board = Bitboard::default_board();
    match params.next() {
        Some(p) => {
            if p == "startpos" {}
            else if p == "fen" {
                let mut position_str: String = String::from("");
                for i in 0..6 {
                    match params.next() {
                        Some(param) => {
                            if i != 0 {
                                position_str = format!("{} {}", position_str, param);
                            } else {
                                position_str = format!("{}", param);
                            }
                        },
                        None => {println!("bad fen"); return board;}
                    }
                }
                board = Bitboard::from_position(format!("{}", position_str));
            }

            if params.next() == Some("moves") {
                loop {
                    match params.next() {
                        Some(mv) => { apply_uci_move(&mut board, mv.to_string()); },
                        None => { return board; }
                    }
                }
            }
        },
        None => { return board; }
    };
    return board;
}

fn setoption(params: &mut SplitWhitespace, options: &mut UCIOptions) {
    match params.next() {
        Some(p) => match p {
            "name" => {
                let mut option_name: String = String::from("");
                loop {
                    match params.next() {
                        Some(param) => {if param == "value" {break;}; option_name.push_str(param);}
                        None => { return; }
                    }
                    option_name.push_str(" ");
                }

                option_name = option_name.trim().to_string();
                // value
                let value_str = match params.next() {
                    Some(param) => param,
                    None => { return; }
                };

                // Hash
                if option_name.as_str() == "Hash" {
                    if ongoing_search() {
                        println!("ERR: Cannot resize hash table during search");
                        return;
                    }
                    let hash_size = match value_str.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {println!("ERR: invalid value provided for Hash"); return;}
                    };
                    if hash_size > 65535 || hash_size < 1 {
                        println!("ERR: invalid value provided for Hash");
                        return;
                    }
                    allocate_tt(hash_size as usize);
                    options.hash = hash_size;
                    return;
                }

                // Threads
                else if option_name.as_str() == "Threads" {
                    let num_threads: i32 = match value_str.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {println!("ERR: invalid value provided for Threads"); return;}
                    };
                    if num_threads > 256 || num_threads < 1 {
                        println!("ERR: invalid value provided for Threads");
                        return;
                    }
                    options.num_threads = num_threads as u16;
                    return;
                }
                // Move Overhead

                else if option_name.as_str() == "Move Overhead" {
                    let overhead: i32 = match value_str.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {println!("ERR: invalid value provided for Move Overhead"); return;}
                    };
                    if overhead > 1000 || overhead < 0 {
                        println!("ERR: invalid value provided for Move Overhead");
                        return;
                    }
                    options.move_overhead = overhead;
                    return;
                }
            },
            _ => { return; }
        },
        None => { return; }
    };
}

pub fn uci_loop() {
    let mut board = Bitboard::default_board();
    let mut options = UCIOptions {num_threads: 1, move_overhead: 10, hash: 64};

    loop {
        let mut inp: String = String::new();
        io::stdin()
            .read_line(&mut inp)
            .expect("Failed to read line");

        let mut params = inp.trim().split_whitespace();
        let cmd = match params.next() {
            Some(p) => p,
            None => {continue;}
        };

        if cmd == "quit" {
            break;
        } else if cmd == "uci" {
            println!("id name Mantissa v3.3.0-tcec-0");
            println!("id author jtwright");
            println!("option name Hash type spin default 64 min 1 max 65535");
            println!("option name Threads type spin default 1 min 1 max 256");
            println!("option name Move Overhead type spin default 10 min 1 max 1000");
            println!("uciok");
        } else if cmd == "ucinewgame" {
            // clear the transposition table
            board = Bitboard::default_board();
            clear_tt();
            clear_info();
        } else if cmd == "isready" {
            println!("readyok");
        } else if cmd == "setoption" {
            setoption(&mut params, &mut options);
        } else if cmd == "position" {
            board = set_position(&mut params);
        } else if cmd == "go" {
            uci_go(&mut board, options, &mut params);
        } else if cmd == "stop" {
            abort_search();
        } else if cmd == "eval" {
            println!("{}", static_eval(&mut board, &mut PHT::get_pht(1)) / 10);
        } else if cmd == "nnue" {
            let b = board.thread_copy();
            board.net.print_eval(&b);
            println!("Classical Eval (White View): {:+.2}", evaluate_position(&mut board, &mut PHT::get_pht(1)) as f32 / 1000.0);
        } else {
            println!("unrecognized command.");
        }
    }
}
