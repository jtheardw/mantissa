use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};

use crate::bitboard::*;
use crate::eval::*;
use crate::movegen::*;
use crate::moveutil::*;
use crate::search::*;
use crate::searchutil::*;
use crate::util::*;

pub fn san_to_move(pos: &mut Bitboard, san: String) -> Move {
    let mut san_bytes = san.trim_end_matches(|x| x == '+' || x == '#').as_bytes();

    if san_bytes == "0-0".as_bytes() || san_bytes == "O-O".as_bytes() {
        return if pos.side_to_move == Color::White {
            Move::piece_move(coord_to_idx((4, 0)), coord_to_idx((6, 0)), b'k')
        } else {
            Move::piece_move(coord_to_idx((4, 7)), coord_to_idx((6, 7)), b'k')
        }
    }

    if san_bytes == "0-0-0".as_bytes() || san_bytes == "O-O-O".as_bytes() {
        return if pos.side_to_move == Color::White {
            Move::piece_move(coord_to_idx((4, 0)), coord_to_idx((2, 0)), b'k')
        } else {
            Move::piece_move(coord_to_idx((4, 7)), coord_to_idx((2, 7)), b'k')
        }
    }

    let pseudolegal_moves = moves(pos);
    let mut moves = Vec::new();
    for mv in pseudolegal_moves {
        pos.do_move(&mv);
        if !pos.is_check(!pos.side_to_move) {
            // legal
            moves.push(mv);
        }
        pos.undo_move(&mv);
    }

    let promote_to = if san_bytes[san_bytes.len() - 2] == b'=' {
        let promote = san_bytes[san_bytes.len() - 1] + 32; // lowercase
        san_bytes = &san_bytes[..san_bytes.len() - 2];
        promote
    } else {
        0
    };

    // start by filtering by destination tile
    let dest = str_to_idx(format!("{}{}", san_bytes[san_bytes.len() - 2] as char, san_bytes[san_bytes.len() - 1] as char));
    moves.retain(|&m| m.end == dest && m.promote_to == promote_to);

    if moves.len() == 1 { return moves[0]; }

    let piece_type = if san_bytes[0] >= b'A' && san_bytes[0] <= b'Z' {
        let piece = san_bytes[0] + 32; // + 32 to lowercase the ascii
        san_bytes = &san_bytes[1..];
        piece
    } else {
        b'p'
    };

    moves.retain(|&m| m.piece == piece_type);
    if moves.len() == 1 { return moves[0]; }

    let file = if san_bytes[0] >= b'a' && san_bytes[0] <= b'z' {
        let f = san_bytes[0] - b'a';
        san_bytes = &san_bytes[1..];
        f as i8
    } else {
        -1
    };

    if file != -1 {
        moves.retain(|&m| m.start % 8 == file);
        if moves.len() == 1 { return moves[0]; }
    }

    let rank = if san_bytes[0] >= b'1' && san_bytes[0] <= b'9' {
        let r = san_bytes[0] - b'1';
        r as i8
    } else {
        -1
    };

    if rank != -1 {
        moves.retain(|&m| m.start / 8 == rank);
        if moves.len() == 1 { return moves[0]; }
    }

    return Move::null_move();
}


fn extract_positions(pos: &mut Bitboard, buf: &String) {
    let mut in_annotation = false;
    let mut cur_fen = pos.fen();
    let mut record_pos = false;
    let mut pos_stm = Color::White;
    let mut pos_eval = 0;
    let mut pos_qeval = 0;
    let mut lines: Vec<String> = Vec::new();

    let tokens = buf.split_ascii_whitespace();
    for token in tokens {
        if in_annotation {
            if token.ends_with("}") {
                in_annotation = false;
            }
            continue;
        }

        if token == "1-0" || token == "0-1" || token == "1/2-1/2" || token == "*" {
            let outcome = {
                if token == "1-0" {
                    "1.0"
                } else if token == "1/2-1/2" {
                    "0.5"
                } else if token == "0-1" {
                    "0.0"
                } else {
                    return;
                }
            };
            for line in lines {
                println!("{};outcome:{}", line, outcome);
            }
            return;
        }

        match &token[..1] {
            "{" => {
                // entering annotation
                in_annotation = true;
                if record_pos {
                    let end_idx = token.find("/").unwrap();
                    let score: f64 = match token[1..end_idx].parse() {
                        Ok(s) => s,
                        Err(_e) => {
                            // generally a mate score. ignore
                            record_pos = false;
                            continue;
                        }
                    };
                    let mul = if pos_stm == Color::White { 1 } else { -1 };
                    let output_score = mul * ((score * 100.0) as i32);

                    lines.push(format!("{};score:{};eval:{};qs:{}", cur_fen, output_score, pos_eval, pos_qeval));
                    record_pos = false;
                }
            },
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {},
            _ => {
                // for well-formatted file, by process of elinimation, this
                // should be a move
                let mv = san_to_move(pos, token.to_string());
                let mul = if pos.side_to_move == Color::White { 1 } else { -1 };
                let qeval = mul * qsearch(pos, -1000000, 1000000, 0) / 10;
                let eval = mul * static_eval(pos, unsafe {&mut TI[0].pht}) / 10;

                if qeval == eval {
                    cur_fen = pos.fen();
                    pos_stm = pos.side_to_move;
                    pos_eval = eval;
                    pos_qeval = qeval;
                    record_pos = true;
                }

                pos.do_move(&mv);
            }
        }
    }
}

pub fn convert_pgn(fname: &str) {
    unsafe {
        TI = vec![ThreadInfo::new()];
    }

    let f = match File::open(fname) {
        Ok(f) => f,
        Err(e) => panic!("unable to open file {}.  err: {}", fname, e)
    };

    let mut r = BufReader::new(f);

    let mut buf = String::new();

    const GET_STARTING_POSITION: i32 = 0;
    const FIND_MOVES: i32 = 1;
    const GATHER_MOVES: i32 = 2;
    const EXTRACT_POSITIONS: i32 = 3;

    let mut state = GET_STARTING_POSITION;
    let mut board = Bitboard::default_board();
    loop {
        match state {
            GET_STARTING_POSITION => {
                loop {
                    buf.clear();
                    let num_bytes = match r.read_line(&mut buf) {
                        Ok(n) => n, Err(e) => panic!("unable to read line. err: {}", e)
                    };

                    if num_bytes == 0 { return; }

                    match buf.get(..4) {
                        Some("[FEN") => {
                            board = Bitboard::from_position(format!("{}", buf.get(6..(buf.len() - 2)).unwrap()));
                            state = FIND_MOVES;
                            break;
                        },
                        _ => {}
                    }
                }
            },
            FIND_MOVES => {
                loop {
                    buf.clear();
                    let num_bytes = match r.read_line(&mut buf) {
                        Ok(n) => n, Err(e) => panic!("unable to read line. err: {}", e)
                    };
                    if num_bytes == 0 { return; }

                    if buf.trim().len() == 0 {
                        // was an empty line
                        state = GATHER_MOVES;
                        break;
                    }
                }
            },
            GATHER_MOVES => {
                buf.clear();
                // gathering loop
                loop {
                    let num_bytes = match r.read_line(&mut buf) {
                        Ok(n) => n, Err(e) => panic!("unable to read line. err: {}", e)
                    };
                    if num_bytes <= 1 { state = EXTRACT_POSITIONS; break; }
                }
            },
            EXTRACT_POSITIONS => {
                // the buffer should contain the whole moveslist at this point, with annotations.
                extract_positions(&mut board, &buf);
                state = GET_STARTING_POSITION;
            },
            _ => {return;}
        }
    }
}
