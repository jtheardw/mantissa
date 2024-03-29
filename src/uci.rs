use std::io;
use std::str::SplitWhitespace;
use std::thread;
use std::time;

use crate::bitboard::*;
use crate::eval::*;
use crate::moveutil::*;
use crate::pht::*;
use crate::search::*;
use crate::searchutil::*;
use crate::syzygy::*;
use crate::util::*;
use crate::tt::*;

#[derive(Clone)]
pub struct UCIOptions {
    pub num_threads: u16,
    pub move_overhead: i32,
    pub hash: i32,
    pub bh_mode: u8,
    pub probe_depth: i32,
    pub syzygy_path: String
}

impl UCIOptions {
    pub fn default() -> UCIOptions {
        UCIOptions {
            num_threads: 1,
            move_overhead: 10,
            hash: 64,
            bh_mode: OFF,
            probe_depth: 0,
            syzygy_path: ("").to_string()
        }
    }
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

    let mut bh_piece = -1;

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
        } else if p == "piece" {
            bh_piece = match params.next() {
                Some(p) => { str_to_idx(p.trim().to_string()) },
                None => panic!("empty piece!")
            };
        }
    }

    if moves_to_go >= 0 {
        // moves to go move
        search_limit = SearchLimits::moves_to_go(clock_time, moves_to_go, options.move_overhead);
    } else if depth > 0 {
        search_limit = SearchLimits::depth(depth);
    } else if movetime > 0 {
        search_limit = SearchLimits::movetime(movetime);
    } else if clock_time > 0 {
        let ply = board.history.len() as i32;
        let mat = mg_score(simple_material_score(board)) / 1000;
        search_limit = SearchLimits::clock_with_inc(clock_time, inc, options.move_overhead, ply, mat);
    } else if infinite {
        search_limit = SearchLimits::infinite();
    }

    let mut thread_board = board.thread_copy();
    thread::spawn(move || {
        best_move(&mut thread_board, options.num_threads, search_limit, options, bh_piece);
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

fn stop() {
    if !ongoing_search() {
        std::thread::sleep(time::Duration::from_millis(100));
    }
    abort_search();
    while ongoing_search() {
        std::thread::sleep(time::Duration::from_millis(10));
        abort_search();
    }
    // emit_last_bestmove();
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
                        eprintln!("ERR: Cannot resize hash table during search");
                        return;
                    }
                    let hash_size = match value_str.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {println!("ERR: invalid value provided for Hash"); return;}
                    };
                    if hash_size > 65536 || hash_size < 1 {
                        eprintln!("ERR: invalid value provided for Hash");
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
                        eprintln!("ERR: invalid value provided for Threads");
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
                        eprintln!("ERR: invalid value provided for Move Overhead");
                        return;
                    }
                    options.move_overhead = overhead;
                    return;
                }

                // Syzygy
                else if option_name.as_str() == "SyzygyPath" {
                    let path: String = format!("{}", value_str.trim());
                    if !path.starts_with("<empty>") {
                        options.syzygy_path = path.clone();
                        unsafe {
                            let success = setup_tb(path.as_str());
                            if success {
                                println!("info string successfully read in TB that supports {} pieces", max_tb_pieces());
                            } else {
                                println!("info string failed to initialize TB.")
                            }
                        }
                    }
                }

                else if option_name.as_str() == "SyzygyProbeDepth" {
                    let depth: i32 = match value_str.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {eprintln!("ERR: invalide value provided for SyzygyProbeDepth"); return;}
                    };
                    if depth > 64 || depth < 0 {
                        eprintln!("ERR: invalid value provided for SyzygyProbeDepth");
                        return;
                    }
                    options.probe_depth = depth;
                }
            },
            _ => { return; }
        },
        None => { return; }
    };
}

const OFF: u8 = 0;
const BRAIN: u8 = 1;
const HAND: u8 = 2;

pub fn uci_loop() {
    let mut board = Bitboard::default_board();
    let mut options = UCIOptions::default();// {num_threads: 1, move_overhead: 10, hash: 64, bh_mode: OFF, probe_depth: 0};

    loop {
        let mut inp: String = String::new();
        io::stdin()
            .read_line(&mut inp)
            .expect("Failed to read line");

        let mut params = inp.trim().split_whitespace();
        let cmd = match params.next() {
            Some(p) => p,
            None => {break;}
        };
        if cmd == "quit" {
            break;
        } else if cmd == "uci" {
            println!("id name Mantissa v3.7.2");
            println!("id author jtwright");
            println!("option name Hash type spin default 64 min 1 max 65536");
            println!("option name Threads type spin default 1 min 1 max 256");
            println!("option name Move Overhead type spin default 10 min 1 max 1000");
            println!("option name SyzygyPath type string default <empty>");
            println!("option name SyzygyProbeDepth type spin default 0 min 0 max 64");
            println!("uciok");
        } else if cmd == "ucinewgame" {
            // clear the transposition table
            board = Bitboard::default_board();
            clear_tt();
            clear_info();
            unsafe {
                setup_tb(options.syzygy_path.as_str());
            }
        } else if cmd == "isready" {
            println!("readyok");
        } else if cmd == "setoption" {
            setoption(&mut params, &mut options);
        } else if cmd == "position" {
            board = set_position(&mut params);
        } else if cmd == "go" {
            uci_go(&mut board, options.clone(), &mut params);
        } else if cmd == "stop" {
            stop();
        } else if cmd == "eval" {
            println!("{}", static_eval(&mut board, &mut PHT::get_pht(1)) / 10);
        } else if cmd == "bhmode" || cmd == "bh_mode" || cmd == "bh" {
            let mode = match params.next() {
                Some(p) => p,
                None => {println!("You didn't say a mode"); continue;}
            };
            if mode == "brain" {
                eprintln!("Brain mode.");
                options.bh_mode = BRAIN;
            } else if mode == "hand" {
                eprintln!("Hand mode.");
                options.bh_mode = HAND;
            } else if mode == "off" {
                eprintln!("Normal mode.");
                options.bh_mode = OFF;
            } else {
                println!("Err: Invalid mode {}?", mode);
            }
        } else if cmd == "nnue" {
            let b = board.thread_copy();
            board.net.print_eval(&b);
            println!("Classical Eval (White View): {:+.2}", evaluate_position(&mut board, &mut PHT::get_pht(1)) as f32 / 1000.0);
        } else if cmd == "hce" {
            let mut b = board.thread_copy();
            print_eval(&mut b);
            println!("NNUE Eval (White View): {:+.2}", board.net.nnue_eval() as f32 / 1000.0)
        }
        else {
            println!("unrecognized command.");
        }
    }
}
