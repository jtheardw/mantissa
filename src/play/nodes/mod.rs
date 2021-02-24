use rand::Rng;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::vec::Vec;

const KING_VALUE: i32 = 200000;
const QUEEN_VALUE: i32 = 9000;
const ROOK_VALUE: i32 = 5000;
const BISHOP_VALUE: i32 = 3200;
const KNIGHT_VALUE: i32 = 3100;
const PAWN_VALUE: i32 = 1000;

pub struct Node {
    state: HashMap<(i32, i32), (u8, bool)>, // location -> piece
    piece_map: HashMap<(u8, bool), (i32, i32)>, // piece -> location
    pub white_turn: bool,
    ep: i32,                     // en passant file
    cr: (bool, bool, bool, bool), // Castling Rights: white qs, white ks, black qs, black ks
    ep_stack: Vec<i32>,          // en passant stack for undo
    pub cap_stack: Vec<u8>,  // capture stack for undo
    cr_stack: Vec<(bool, bool, bool, bool)>, // castling rights stack for undo
    promote_stack: Vec<bool>,                 // was there a promotion (for undoing)
    pub material: i32,
    zobrist_table: [[u64; 12]; 64],
    hash: u64,
    history: Vec<u64>
}

impl Node {
    pub fn print_node(&self) {
        for i in 0..8 {
            let mut ln = ['a'; 8];
            for j in 0..8 {
                let x: i32 = j;
                let y: i32 = 7-i;
                match self.state.get(&(x, y)) {
                    Some(p) => {ln[j as usize] = p.0 as char;},//if p.1 {(p.0 as char).to_uppercase().next()} else {(p.0 as char).to_string()};},
                    None => {ln[j as usize] = '.';},
                }
            }
            println!("{}{}{}{}{}{}{}{}", ln[0], ln[1], ln[2], ln[3], ln[4], ln[5], ln[6], ln[7]);
        }
    }

    pub fn get_str(&self) -> String {
        let mut s = String::new();
        for i in 0..8 {
            for j in 0..8 {
                let x: i32 = j;
                let y: i32 = 7 - i;
                match self.state.get(&(x, y)) {
                    Some(p) => {
                        if p.1 {
                            s.push_str(&(p.0 as char).to_uppercase().to_string());
                        } else {
                            s.push(p.0 as char);
                        }
                    },
                    None => {s.push('.');}
                }
            }
            s.push('\n');
        }
        return s;
    }

    fn init_zobrist() -> [[u64; 12]; 64] {
        let mut rng = rand::thread_rng();
        let mut zobrist_table : [[u64; 12]; 64] = [[0; 12]; 64];
        for i in 0..64 {
            for j in 0..12 {
                zobrist_table[i][j] = rng.gen();
            }
        }
        // println!("zob {:?}", zobrist_table);
        return zobrist_table;
    }

    pub fn get_hash(&self) -> u64 {
        return self.hash;
    }

    pub fn get_full_hash(&self) -> u64 {
        let mut h: u64 = 0;
        for i in 0..64 {
            let (x, y) = (i / 8, i % 8);
            let i = i as usize;
            let to_xor = match self.state.get(&(x, y)) {
                Some(p) => match p {
                    (b'k', true) => self.zobrist_table[i][0],
                    (b'q', true) => self.zobrist_table[i][1],
                    (b'b', true) => self.zobrist_table[i][2],
                    (b'n', true) => self.zobrist_table[i][3],
                    (b'r', true) => self.zobrist_table[i][4],
                    (b'p', true) => self.zobrist_table[i][5],
                    (b'k', false) => self.zobrist_table[i][6],
                    (b'q', false) => self.zobrist_table[i][7],
                    (b'b', false) => self.zobrist_table[i][8],
                    (b'n', false) => self.zobrist_table[i][9],
                    (b'r', false) => self.zobrist_table[i][10],
                    (b'p', false) => self.zobrist_table[i][11],
                    _ => 0
                },
                None => 0
            };
            h ^= to_xor
        }
        return h
    }

    fn pawn_moves(&self, coord: (i32, i32), white: bool) -> VecDeque<Move> {
        let (x, y) = coord;
        let mut moves : VecDeque<Move> = VecDeque::new();
        // capture
        for nx in [x - 1, x + 1].iter() {
            let nx: i32 = nx + 0;
            let ny = if white {y + 1} else {y - 1};
            if (nx >= 8 || nx < 0) || (ny >= 8 || ny < 0) { continue; }
            match self.state.get(&(nx, ny)) {
                Some(p) => {
                    if p.1 != white {
                        if (white && ny == 7) || (!white && ny == 0) {
                            for p in [b'q', b'r', b'n', b'b'].iter() {
                                moves.push_back(Move::pawn_promote_move(&self, (x,y), (nx,ny), *p));
                            }
                        } else {
                            moves.push_back(Move::pawn_move(&self, (x,y), (nx, ny)));
                        }
                    };
                },
                None => {}
            };
        };

        // move forward
        for adv in 1..3 {
            let (nx, ny) = (x, if white {y + adv} else {y - adv});
            if (adv == 2) && !((white && y == 1) || (!white && y == 6)) {break;}
            match self.state.get(&(nx, ny)) {
                Some(_) => {break;},
                None => {
                    if (white && ny == 7) || (!white && ny == 0) {
                        for p in [b'q', b'r', b'n', b'b'].iter() {
                            moves.push_back(Move::pawn_promote_move(&self, (x,y), (nx,ny), *p));
                        }
                    } else {
                        moves.push_back(Move::pawn_move(&self, (x,y), (nx, ny)));
                    }
                }
            };

        };

        // en passant
        if (white && y == 4) || (!white && y == 3) {
            if (self.ep >= 0) && (self.ep == (x - 1) || self.ep == (x + 1)) {
                let (nx, ny) = (self.ep, if white {y + 1} else {y - 1});
                moves.push_back(Move::ep_pawn_move(&self, (x,y), (nx as i32, ny)));
            }
        }

        return moves
    }

    fn rook_moves(&self, coord: (i32, i32), white: bool) -> VecDeque<Move> {
        let (x, y) = coord;
        let mut moves : VecDeque<Move> = VecDeque::new();

        // up, down, left, right
        for (sx, sy) in [(0, 1), (0, -1), (-1, 0), (1, 0)].iter() {
            let mut d = 1;
            while d < 8 {
                let (nx, ny) = (x + (sx * d), y + (sy * d));
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    break;
                }
                match self.state.get(&(nx, ny)) {
                    Some(p) => {
                        if p.1 != white {
                            moves.push_back(Move::sliding_move(&self, (x,y), (nx, ny)));
                        };
                        break;
                    },
                    None => {moves.push_back(Move::sliding_move(&self, (x,y), (nx, ny)));}
                }
                d += 1;
            }
        }
        return moves;
    }

    fn bishop_moves(&self, coord: (i32, i32), white: bool) -> VecDeque<Move> {
        let (x, y) = coord;
        let mut moves : VecDeque<Move> = VecDeque::new();

        // up=left, up-right, dn-left, dn-right
        for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)].iter() {
            let mut d = 1;
            while d < 8 {
                let (nx, ny) = (x + (sx * d), y + (sy * d));
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    break;
                }
                match self.state.get(&(nx, ny)) {
                    Some(p) => {
                        if p.1 != white {
                            moves.push_back(Move::sliding_move(&self, (x,y), (nx, ny)));
                        };
                        break;
                    },
                    None => moves.push_back(Move::sliding_move(&self, (x,y), (nx, ny)))
                }
                d += 1;
            }
        }
        return moves;
    }

    fn queen_moves(&self, coord: (i32, i32), white: bool) -> VecDeque<Move> {
        let mut moves : VecDeque<Move> = VecDeque::new();
        moves.append(& mut self.bishop_moves(coord, white));
        moves.append(& mut self.rook_moves(coord, white));
        return moves;
    }

    fn knight_moves(&self, coord: (i32, i32), white: bool) -> VecDeque<Move> {
        let (x, y) = coord;
        let mut moves : VecDeque<Move> = VecDeque::new();

        for (dx, dy) in [
            (-1, 2),
            ( 1, 2),
            (-2, 1),
            ( 2, 1),
            (-2,-1),
            ( 2,-1),
            (-1,-2),
            ( 1,-2)
        ].iter() {
            let (nx, ny) = (x + dx, y + dy);
            if (nx >= 8 || nx < 0) || (ny >= 8 || ny < 0) { continue; }
            match self.state.get(&(nx, ny)) {
                Some(p) => if p.1 != white {
                    moves.push_back(Move::knight_move(&self, (x,y), (nx, ny)))
                },
                None => moves.push_back(Move::knight_move(&self, (x,y), (nx, ny)))
            };
        };

        return moves;
    }

    fn king_moves(&self, coord: (i32, i32), white: bool) -> VecDeque<Move> {
        let (x, y) = coord;
        let mut moves: VecDeque<Move> = VecDeque::new();

        for (dx, dy) in [
            ( 0, 1),
            ( 0,-1),
            (-1, 0),
            ( 1, 0),
            (-1, 1),
            ( 1, 1),
            (-1,-1),
            ( 1, -1)
        ].iter() {
            let (nx, ny) = (x + dx, y + dy);
            if (nx >= 8 || nx < 0) || (ny >= 8 || ny < 0) { continue; }
            // println!("{} {} {} {} {}", white, x, y, nx, ny);
            match self.state.get(&(nx, ny)) {
                Some(p) => if p.1 != white {
                    moves.push_back(Move::king_move(&self, (x,y), (nx, ny)))
                },
                None => moves.push_back(Move::king_move(&self, (x,y), (nx, ny)))
            };
        }
        // castling
        // queenside
        if self.can_castle(white, true) {
            moves.push_back(Move::castle_move(&self, (x,y), (x-2, y)))
        }
        if self.can_castle(white, false) {
            moves.push_back(Move::castle_move(&self, (x,y), (x+2, y)))
        }

        return moves;
    }

    pub fn is_check(&self, white: bool) -> bool {
        for (coord, (p, w)) in self.state.iter() {
            if *p == b'k' && *w == white {
                return self.is_attacked(*coord, white);
            }
        }
        return false;
    }

    fn is_attacked(&self, coord: (i32, i32), white: bool) -> bool {
        // is a `white` piece on this square threatened
        let (x, y) = coord;
        let pawn_dy = if white {1} else {-1};
        // check all potential attackers

        // slidy bois
        for (sx, sy) in [(0,1), (0,-1), (-1,0), (1,0), (-1,1), (1,1), (-1,-1), (1,-1)].iter() {
            let mut d = 1;
            while d < 8 {
                let (nx, ny) = (x + (sx * d), y + (sy * d));
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    break;
                }
                match self.state.get(&(nx, ny)) {
                    Some(p) => {
                        if p.1 != white {
                            if p.0 == b'q' {
                                return true;
                            } else if d == 1 && (p.0 == b'k') {
                                return true;
                            } else if ((*sx == 0) != (*sy == 0)) && (p.0 == b'r') {
                                return true;
                            } else if ((*sx == 0) == (*sy == 0)) && (p.0 == b'b') {
                                return true;
                            } else if d == 1 && ((*sx, *sy) == (-1, pawn_dy) || (*sx, *sy) == (1, pawn_dy)) && p.0 == b'p' {
                                return true;
                            }
                        }
                        break;
                    },
                    None => {}
                };
                d += 1;
            }
        }

        // knights
        for (dx, dy) in [(-1, 2), (1, 2), (-2, 1), (2, 1), (-2, -1), (2, -1), (-1, -2), (1, -2)].iter() {
            let (nx, ny) = (x + dx, y + dy);
            if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                continue;
            }
            match self.state.get(&(nx, ny)) {
                Some(p) => {
                    if p.1 != white && p.0 == b'n' {
                        return true;
                    }
                },
                None => {}
            };
        }

        return false;
    }

    fn can_castle(&self, white: bool, queenside: bool) -> bool {
        // castling rights?
        let rights = match (white, queenside) {
            (false, false) => self.cr.0,
            (false, true) => self.cr.1,
            (true, false) => self.cr.2,
            (true, true) => self.cr.3
        };
        if !rights { return false; }

        // check squares on the way
        let (kx, ky) = if white {(4, 0)} else {(4, 7)};
        let crossover_square = if queenside {(kx - 1, ky)} else {(kx + 1, ky)};
        let target_square = if queenside {(kx - 2, ky)} else {(kx + 2, ky)};

        if self.state.contains_key(&crossover_square) || self.state.contains_key(&target_square) {
            return false;
        }

        if queenside {
            let extra_square = (kx - 3, ky);
            if self.state.contains_key(&extra_square) {
                return false;
            }
        }

        if self.is_attacked((kx, ky), white) || self.is_attacked(crossover_square, white) {
            return false;
        }

        return true;
    }

    pub fn moves(&mut self) -> VecDeque<Move>{
        let mut move_queue : VecDeque<Move> = VecDeque::new();
        for (coord, (p, w)) in self.state.iter() {
            let coord = *coord;
            let w = *w;
            if w != self.white_turn {continue;}
            let mut piece_moves = match p {
                b'k' => self.king_moves(coord, w),
                b'q' => self.queen_moves(coord, w),
                b'b' => self.bishop_moves(coord, w),
                b'n' => self.knight_moves(coord, w),
                b'r' => self.rook_moves(coord, w),
                b'p' => self.pawn_moves(coord, w),
                _ => panic!("erroneous piece! {}", p.to_string())
            };
            move_queue.append(& mut piece_moves)
        }
        // for cmove in move_queue.iter() {
        //     let before = self.get_str();
        //     let before_turn = self.white_turn;
        //     self.do_move(&cmove);
        //     let inter = self.get_str();
        //     let inter_turn = self.white_turn;
        //     self.undo_move(&cmove);
        //     if self.get_str() != before {
        //         panic!("BAD MOVE REWIND ON MOVE {} BEFORE:\n{}\nW {}\nINTER\n{}\nW {}\nAFTER\n{}\nW {}", cmove, before, before_turn, inter, inter_turn, self.get_str(), self.white_turn);
        //     }
        // }

        return move_queue;
    }

    fn get_zr_xor(&self, c : usize, p : u8, w: bool) -> u64 {
        match (p, w) {
            (b'k', true) => self.zobrist_table[c][0],
            (b'q', true) => self.zobrist_table[c][1],
            (b'b', true) => self.zobrist_table[c][2],
            (b'n', true) => self.zobrist_table[c][3],
            (b'r', true) => self.zobrist_table[c][4],
            (b'p', true) => self.zobrist_table[c][5],
            (b'k', false) => self.zobrist_table[c][6],
            (b'q', false) => self.zobrist_table[c][7],
            (b'b', false) => self.zobrist_table[c][8],
            (b'n', false) => self.zobrist_table[c][9],
            (b'r', false) => self.zobrist_table[c][10],
            (b'p', false) => self.zobrist_table[c][11],
            _ => 0
        }
    }

    fn update_hash(&mut self, piece: u8, white: bool, start: (i32, i32), end: (i32, i32), capture: u8, promotion: u8, is_ep: bool) {
        // remember to call this twice for castling
        let start_coord = start.0 * 8 + start.1;
        self.hash ^= self.get_zr_xor(start_coord as usize, piece, white);
        let end_coord = end.0 * 8 + end.1;
        self.hash ^= self.get_zr_xor(end_coord as usize, if promotion != 0 {promotion} else {piece}, white);

        if capture != 0 {
            if !is_ep {
                self.hash ^= self.get_zr_xor(end_coord as usize, capture, !white);
            } else {
                let cap_coord = end.0 * 8 + start.1;
                self.hash ^= self.get_zr_xor(cap_coord as usize, capture, !white);
            }
        }
    }

    pub fn undo_move(&mut self, cmove: &Move) {
        // put the piece back where it was (may need to demote)
        // de_castle
        // restore captured piece
        // restore ep
        // restore cr
        // restore material
        // flip whose turn
        self.history.pop();
        let mut material_delta = 0;
        let mut moved_piece = match self.state.remove(&cmove.end) {
            Some(tup) => tup,
            None => {
                println!("move: {}, \nboard: \n{}", cmove, self.get_str());
                panic!("move can't be reversed tried to look at nonexistant piece.");
            }
        };
        let promoted_result = moved_piece;

        // demote
        let promoted = match self.promote_stack.pop() {
            Some(p) => p,
            None => panic!("Nothing in promoted stack!")
        };
        if promoted {
            material_delta += match moved_piece.0 {
                b'q' => QUEEN_VALUE,
                b'r' => ROOK_VALUE,
                b'b' => BISHOP_VALUE,
                b'n' => KNIGHT_VALUE,
                _ => 0
            };
            material_delta -= PAWN_VALUE;
            moved_piece = (b'p', moved_piece.1);
        }

        // put piece back
        self.state.insert(cmove.start, moved_piece);

        let mut decastling = false;
        // de_castle :/
        if moved_piece.0 == b'k' {
            if (cmove.end.0 - cmove.start.0).abs() == 2 {
                // castled boys!
                let old_rook_x: i32 = if cmove.end.0 > cmove.start.0 {7} else {0};
                let new_rook_x: i32 = if cmove.end.0 > cmove.start.0 {5} else {3};
                self.state.insert((old_rook_x, cmove.start.1), (b'r', !self.white_turn));
                self.state.remove(&(new_rook_x, cmove.start.1));
                decastling = true;
                self.update_hash(b'r', !self.white_turn, (old_rook_x, cmove.start.1), (new_rook_x, cmove.start.1), 0, 0, false);
            }
        }

        let captured_piece = match self.cap_stack.pop() {
            Some(p) => p,
            None => panic!("Empty capture stack!")
        };
        // restore captured piece
        if cmove.is_ep {
            self.state.insert((cmove.end.0, cmove.start.1), (b'p', self.white_turn));
            material_delta += PAWN_VALUE;
        } else if captured_piece != 0 {
            self.state.insert(cmove.end, (captured_piece, self.white_turn));
            material_delta += match captured_piece {
                b'k' => KING_VALUE,
                b'q' => QUEEN_VALUE,
                b'p' => PAWN_VALUE,
                b'r' => ROOK_VALUE,
                b'b' => BISHOP_VALUE,
                b'n' => KNIGHT_VALUE,
                _ => 0
            };
        }

        self.update_hash(moved_piece.0, !self.white_turn, cmove.start, cmove.end, captured_piece, if promoted {promoted_result.0} else {0}, cmove.is_ep);
        self.ep = match self.ep_stack.pop() {Some(p) => p, None => panic!("empty ep stack!")};
        self.cr = match self.cr_stack.pop() {Some(p) => p, None => panic!("empty cr stack!")};
        self.material += if self.white_turn {material_delta} else {-material_delta};
        self.white_turn = !self.white_turn;
    }

    pub fn do_move(&mut self, cmove: &Move) {
        self.history.push(self.get_hash());
        let mut ep_status = -1;
        let cap_status;
        let mut promote_status = false;
        let mut material_delta = 0;

        if cmove.is_double {
            ep_status = cmove.start.0
        }

        if cmove.promote_to != 0 {
            promote_status = true;
            material_delta -= PAWN_VALUE;
            material_delta += match cmove.promote_to {
                b'q' => QUEEN_VALUE,
                b'r' => ROOK_VALUE,
                b'b' => BISHOP_VALUE,
                b'n' => KNIGHT_VALUE,
                _ => 0
            };
        }

        // is this a capture
        if cmove.is_ep {
            // en passant
            cap_status = b'p';
            self.state.remove(&(cmove.end.0, cmove.start.1));
            material_delta = PAWN_VALUE;
        } else {
            cap_status = match self.state.remove(&cmove.end) {
                Some(tup) => tup.0,
                None => 0
            };
            material_delta += match cap_status {
                b'k' => KING_VALUE,
                b'q' => QUEEN_VALUE,
                b'p' => PAWN_VALUE,
                b'r' => ROOK_VALUE,
                b'b' => BISHOP_VALUE,
                b'n' => KNIGHT_VALUE,
                _ => 0
            };
        }

        // castle
        let start_piece = match self.state.get(&cmove.start) {
            Some(p) => p.0 + 0,
            None => {println!("move: {}, \nboard: \n{}", cmove, self.get_str()); panic!("Move started on empty tile!")}};
        let mut castling = false;
        if start_piece == b'k' {
            if (cmove.end.0 - cmove.start.0).abs() == 2 {
                if cmove.start != (4, 7) && cmove.start != (4, 0) {panic!("bad castle start move {}\nboard\n{} cr {} {} {} {}", cmove, self.get_str(), self.cr.0, self.cr.1, self.cr.2, self.cr.3)}
                // castling boys!
                let old_rook_x: i32 = if cmove.end.0 > cmove.start.0 {7} else {0};
                let new_rook_x: i32 = if cmove.end.0 > cmove.start.0 {5} else {3};
                self.state.insert((new_rook_x, cmove.start.1), (b'r', self.white_turn));
                match self.state.get(&(old_rook_x, cmove.start.1)) {
                    Some(p) => {},
                    None => panic!("Where's the rook?  move {}\nboard\n{} cr {} {} {} {}", cmove, self.get_str(), self.cr.0, self.cr.1, self.cr.2, self.cr.3)
                };
                self.state.remove(&(old_rook_x, cmove.start.1));
                self.update_hash(b'r', self.white_turn, (old_rook_x, cmove.start.1), (new_rook_x, cmove.start.1), 0, 0, false);
            }
        }

        self.update_hash(start_piece, self.white_turn, cmove.start, cmove.end, cap_status, cmove.promote_to, cmove.is_ep);

        // move the thing
        let mut piece = match self.state.remove(&cmove.start) {
            Some(tup) => tup,
            None => panic!("There was no piece here!")
        };

        if cmove.promote_to != 0 {
            piece = (cmove.promote_to, self.white_turn)
        }
        self.state.insert(cmove.end, piece);

        self.cap_stack.push(cap_status);
        self.ep_stack.push(self.ep);
        self.cr_stack.push(self.cr);
        self.promote_stack.push(promote_status);

        let mut ncr = self.cr;
        self.ep = ep_status;

        match cmove.start {
            (0, 0) => {ncr = (ncr.0, ncr.1, ncr.2, false);},
            (7, 0) => {ncr = (ncr.0, ncr.1, false, ncr.3);},
            (0, 7) => {ncr = (ncr.0, false, ncr.2, ncr.3);},
            (7, 7) => {ncr = (false, ncr.1, ncr.2, ncr.3);},
            (4, 0) => {ncr = (ncr.0, ncr.1, false, false);},
            (4, 7) => {ncr = (false, false, ncr.2, ncr.3);},
            _ => {}
        };

        match cmove.end {
            (0, 0) => {ncr = (ncr.0, ncr.1, ncr.2, false);},
            (7, 0) => {ncr = (ncr.0, ncr.1, false, ncr.3);},
            (0, 7) => {ncr = (ncr.0, false, ncr.2, ncr.3);},
            (7, 7) => {ncr = (false, ncr.1, ncr.2, ncr.3);},
            (4, 0) => {ncr = (ncr.0, ncr.1, false, false);},
            (4, 7) => {ncr = (false, false, ncr.2, ncr.3);},
            _ => {}
        };
        self.cr = ncr;
        self.material += if self.white_turn {material_delta} else {-material_delta};
        self.white_turn = !self.white_turn;
    }

    pub fn is_ep(&self, start: (i32, i32), end: (i32, i32)) -> bool {
        let pawn = match self.state.get(&start) {
            Some(p) => p,
            None => {return false;}
        };
        return pawn.0 == b'p' && end.0 == self.ep && ((pawn.1 && start.1 == 4) || (!pawn.1 && start.1 == 3));
    }

    fn king_moves_value(&self, coord: (i32, i32), w: bool) -> i32 {
        let (x, y) = coord;
        let mut moves = 0;
        for (dx, dy) in [
            ( 0, 1),
            ( 0,-1),
            (-1, 0),
            ( 1, 0),
            (-1, 1),
            ( 1, 1),
            (-1,-1),
            ( 1, -1)
        ].iter() {
            let (nx, ny) = (x + dx, y + dy);
            if (nx >= 8 || nx < 0) || (ny >= 8 || ny < 0) { continue; }
            // println!("{} {} {} {} {}", white, x, y, nx, ny);
            match self.state.get(&(nx, ny)) {
                Some(p) => {
                    moves += ((p.1 != w) as i32);
                },
                None => {moves += 1;}
            };
        }
        return moves;
    }

    fn knight_moves_value(&self, coord: (i32, i32), w: bool) -> i32 {
        let (x, y) = coord;
        let mut moves = 8;

        for (dx, dy) in [
            (-1, 2),
            ( 1, 2),
            (-2, 1),
            ( 2, 1),
            (-2,-1),
            ( 2,-1),
            (-1,-2),
            ( 1,-2)
        ].iter() {
            let (nx, ny) = (x + dx, y + dy);
            if (nx >= 8 || nx < 0) || (ny >= 8 || ny < 0) { moves -= 1; }
        };

        return moves;
    }

    fn bishop_moves_value(&self, coord: (i32, i32), w: bool) -> i32 {
        let (x, y) = coord;
        let mut moves = 0;

        // up=left, up-right, dn-left, dn-right
        for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)].iter() {
            let mut d = 1;
            while d < 3 {
                let (nx, ny) = (x + (sx * d), y + (sy * d));
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    break;
                }
                match self.state.get(&(nx, ny)) {
                    Some(p) => {
                        moves += (p.1 != w) as i32;
                        break;
                    },
                    None => { moves += 1; }
                }
                d += 1;
            }
        }
        return moves;
    }

    fn rook_moves_value(&self, coord: (i32, i32), w: bool) -> i32 {
        let (x, y) = coord;
        let mut moves = 0;

        // up=left, up-right, dn-left, dn-right
        for (sx, sy) in [(-1, 0), (1, 0), (0, -1), (0, -1)].iter() {
            let mut d = 1;
            while d < 3 {
                let (nx, ny) = (x + (sx * d), y + (sy * d));
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    break;
                }
                match self.state.get(&(nx, ny)) {
                    Some(p) => {
                        moves += (p.1 != w) as i32;
                        break;
                    },
                    None => { moves += 1; }
                }
                d += 1;
            }
        }
        return moves;
    }

    pub fn mobility_value(&self) -> i32 {
        let mut move_val = 0;
        for (coord, (p, w)) in self.state.iter() {
            let coord = *coord;
            let w = *w;
            let piece_moves = match p {
                b'k' => self.king_moves_value(coord, w),
                b'q' => self.bishop_moves_value(coord, w),
                b'b' => self.bishop_moves_value(coord, w) + self.rook_moves_value(coord, w),
                b'n' => self.knight_moves_value(coord, w),
                b'r' => self.rook_moves_value(coord, w),
                b'p' => 0,
                _ => panic!("erroneous piece! {}", p.to_string())
            };
            let moves = piece_moves;
            move_val += if w {moves * PAWN_VALUE} else {-moves * PAWN_VALUE};
        }
        return move_val
    }

    pub fn piece_synergy_values(&self) -> i32 {
        let mut piece_bonus: i32 = 0;
        // king, queen, rook, bishop, knight, pawn
        let order = [b'k', b'q', b'r', b'b', b'n', b'p'];
        let mut w_piece_count = [0; 6];
        let mut b_piece_count = [0; 6];

        for ((x, y), (p, w)) in self.state.iter() {
            let idx = match p {
                b'k' => 0,
                b'q' => 1,
                b'r' => 2,
                b'b' => 3,
                b'n' => 4,
                b'p' => 5,
                _ => 0
            };
            if *w {
                w_piece_count[idx] += 1;
            } else {
                b_piece_count[idx] += 1;
            }
            if *p == b'q' {
                if self.history.len() < 12 {
                    // queen moved early
                    let qpl = if *w {*y > 0} else {*y < 7};
                    if qpl {
                        piece_bonus -= if *w {1} else {-1} * (PAWN_VALUE / 3);
                    }
                }
            }
            if *p == b'r' {
                if self.history.len() < 12 {
                    // rook moved early
                    let rpl = if *w {*y > 0} else {*y < 7};
                    if rpl {
                        piece_bonus -= if *w {1} else {-1} * (PAWN_VALUE / 3);
                    }
                }
            }
        }

        // synergies
        for (w, arr) in [(true, w_piece_count), (false, b_piece_count)].iter() {
            let scale = if *w {1} else {-1};
            for i in 0..arr.len() {
                let p = order[i];
                if p == b'b' {
                    if arr[i] >= 2 {
                        piece_bonus += (PAWN_VALUE / 2);
                    }
                }
                if p == b'k' {
                    // few friendly pawns means weak knights
                    if arr[5] < 3 {
                        piece_bonus -= (PAWN_VALUE / 2);
                    }
                }
            }
        }
        return piece_bonus;
    }

    pub fn backwards_pawns_value(&self) -> i32 {
        let mut white_pawns: Vec<i32> = Vec::new();
        let mut black_pawns: Vec<i32> = Vec::new();
        for ((x, y), (p, w)) in self.state.iter() {
            if *p == b'p' {
                if *w {white_pawns.push(*y);}  else {black_pawns.push(*y);}
            }
        }

        let white_min = match white_pawns.iter().min() {
            Some(min) => *min,
            None => 0
        };
        let black_min = match black_pawns.iter().min() {
            Some(min) => *min,
            None => 0
        };

        return PAWN_VALUE *
            (white_pawns.iter().filter(|&n| *n == white_min).count() as i32 -
             black_pawns.iter().filter(|&n| *n == black_min).count() as i32);
    }

    pub fn doubled_pawns_value(&self) -> i32 {
        let mut val = 0;
        let mut pawns_in_file_black = [false; 8];
        let mut pawns_in_file_white = [false; 8];
        for ((x, y), (p, w)) in self.state.iter() {
            if *p == b'p' {
                if *w {
                    if pawns_in_file_white[*x as usize] {
                        val += PAWN_VALUE;
                    }
                    pawns_in_file_white[*x as usize] = true;
                } else {
                    if pawns_in_file_black[*x as usize] {
                        val -= PAWN_VALUE;
                    }
                    pawns_in_file_black[*x as usize] = true;
                }
            }
        }
        return val;
    }

    pub fn isolated_pawns_value(&self) -> i32 {
        let mut val = 0;
        let mut pawns_in_file_black = [false; 8];
        let mut pawns_in_file_white = [false; 8];
        for ((x, y), (p, w)) in self.state.iter() {
            if *p == b'p' {
                if *w {
                    pawns_in_file_white[*x as usize] = true;
                } else {
                    pawns_in_file_black[*x as usize] = true;
                }
            }
        }

        for i in 0..8 {
            let mut has_nb_black = false;
            let mut has_nb_white = false;
            let mut n = i - 1;
            if n >= 0 {
                if pawns_in_file_white[n as usize] {
                    has_nb_white = true;
                }
                if pawns_in_file_black[n as usize] {
                    has_nb_black = true;
                }
            }
            n = i + 1;
            if n < 7 {
                if pawns_in_file_white[n as usize] {
                    has_nb_white = true;
                }
                if pawns_in_file_black[n as usize] {
                    has_nb_black = true;
                }
            }

            if !has_nb_black {val -= PAWN_VALUE;}
            if !has_nb_white {val += PAWN_VALUE;}
        }
        return val;
    }

    pub fn center_value(&self) -> i32 {
        let mut val = 0;
        for ((x, y), (p, w)) in self.state.iter() {
            if *p == b'p' || *p == b'n' {
                let (x, y) = (*x, *y);
                if (1 < x && x < 6) && (1 < y && y < 6) {
                    if (x == 3 || x == 4) && (y == 3 || y == 4) {
                        val += if *w {PAWN_VALUE} else {-PAWN_VALUE}
                    } else if *p == b'p' {
                        let factor = (150) * if *w {y - 2} else {5 - y};
                        val += factor;
                    }
                }
            }
        }
        return val;
    }

    pub fn default_board() -> Node {
        let ep_stack: Vec<i32> = Vec::new();
        let cap_stack: Vec<u8> = Vec::new();
        let cr_stack: Vec<(bool, bool, bool, bool)> = Vec::new();
        let promote_stack: Vec<bool> = Vec::new();
        let zobrist_table: [[u64; 12]; 64] = Node::init_zobrist();

        let state: HashMap<(i32, i32), (u8, bool)> = [
            ((0, 0), (b'r', true)),
            ((1, 0), (b'n', true)),
            ((2, 0), (b'b', true)),
            ((3, 0), (b'q', true)),
            ((4, 0), (b'k', true)),
            ((5, 0), (b'b', true)),
            ((6, 0), (b'n', true)),
            ((7, 0), (b'r', true)),

            ((0, 1), (b'p', true)),
            ((1, 1), (b'p', true)),
            ((2, 1), (b'p', true)),
            ((3, 1), (b'p', true)),
            ((4, 1), (b'p', true)),
            ((5, 1), (b'p', true)),
            ((6, 1), (b'p', true)),
            ((7, 1), (b'p', true)),

            ((0, 7), (b'r', false)),
            ((1, 7), (b'n', false)),
            ((2, 7), (b'b', false)),
            ((3, 7), (b'q', false)),
            ((4, 7), (b'k', false)),
            ((5, 7), (b'b', false)),
            ((6, 7), (b'n', false)),
            ((7, 7), (b'r', false)),

            ((0, 6), (b'p', false)),
            ((1, 6), (b'p', false)),
            ((2, 6), (b'p', false)),
            ((3, 6), (b'p', false)),
            ((4, 6), (b'p', false)),
            ((5, 6), (b'p', false)),
            ((6, 6), (b'p', false)),
            ((7, 6), (b'p', false)),
        ].iter().cloned().collect();

        let piece_map: HashMap<(u8, bool), (i32, i32)> = [
            ((b'r', true), (0, 0)),
            ((b'n', true), (0, 1)),
            ((b'b', true), (0, 2)),
            ((b'q', true), (0, 3)),
            ((b'k', true), (0, 4)),
            ((b'b', true), (0, 5)),
            ((b'n', true), (0, 6)),
            ((b'r', true), (0, 7)),

            ((b'p', true), (1, 0)),
            ((b'p', true), (1, 1)),
            ((b'p', true), (1, 2)),
            ((b'p', true), (1, 3)),
            ((b'p', true), (1, 4)),
            ((b'p', true), (1, 5)),
            ((b'p', true), (1, 6)),
            ((b'p', true), (1, 7)),

            ((b'r', true), (7, 0)),
            ((b'n', true), (7, 1)),
            ((b'b', true), (7, 2)),
            ((b'q', true), (7, 3)),
            ((b'k', true), (7, 4)),
            ((b'b', true), (7, 5)),
            ((b'n', true), (7, 6)),
            ((b'r', true), (7, 7)),

            ((b'p', true), (6, 0)),
            ((b'p', true), (6, 1)),
            ((b'p', true), (6, 2)),
            ((b'p', true), (6, 3)),
            ((b'p', true), (6, 4)),
            ((b'p', true), (6, 5)),
            ((b'p', true), (6, 6)),
            ((b'p', true), (6, 7)),
        ].iter().cloned().collect();

        let mut n = Node {
            state: state,
            piece_map: piece_map,
            white_turn: true,
            ep: -1,
            cr: (true, true, true, true),
            ep_stack: ep_stack,
            cap_stack: cap_stack,
            cr_stack: cr_stack,
            promote_stack: promote_stack,
            material: 0,
            zobrist_table: zobrist_table,
            history: Vec::new(),
            hash: 0
        };
        n.hash = n.get_full_hash();
        return n;
    }
}

#[derive(PartialEq)]
pub struct Move {
    pub repr: String,
    start: (i32, i32),
    end: (i32, i32),
    is_ep: bool,
    is_double: bool,
    is_castle: bool,
    promote_to: u8,
    new_castling_rights: (bool, bool, bool, bool),
    pub is_null: bool,
    pub is_err: bool
}

impl Default for Move {
    fn default() -> Move {
        Move {
            repr: "".to_string(),
            start: (0, 0),
            end: (0, 0),
            is_ep: false,
            is_double: false,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: (true, true, true, true),
            is_null: false,
            is_err: false
        }
    }
}

impl Move {
    pub fn get_repr(start: (i32, i32), end: (i32, i32), promote: u8) -> String {
        let f1 = "abcdefgh".as_bytes()[start.0 as usize] as char;
        let r1 = (start.1 + 1).to_string();
        let f2 = "abcdefgh".as_bytes()[end.0 as usize] as char;
        let r2 = (end.1 + 1).to_string();
        let p = if promote != 0 {(promote as char).to_string()} else {"".to_string()};

        return format!("{}{}{}{}{}", f1.to_string(), r1, f2.to_string(), r2, p);
    }

    pub fn copy_move(mv: &Move) -> Move {
        Move {
            repr: Move::get_repr(mv.start, mv.end, mv.promote_to),
            start: mv.start,
            end: mv.end,
            is_ep: mv.is_ep,
            is_double: mv.is_double,
            is_castle: mv.is_castle,
            promote_to: mv.promote_to,
            new_castling_rights: mv.new_castling_rights,
            is_null: mv.is_null,
            is_err: mv.is_err
        }
    }

    pub fn pawn_move(node: &Node, start: (i32, i32), end: (i32, i32)) -> Move {
        Move {
            repr: Move::get_repr(start, end, 0),
            start: start,
            end: end,
            is_ep: false,
            is_double: ((start.1) - (end.1)).abs() == 2,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: node.cr,
            is_null: false,
            is_err: false
        }
    }

    pub fn ep_pawn_move(node: &Node, start: (i32, i32), end: (i32, i32)) -> Move {
        Move {
            repr: Move::get_repr(start, end, 0),
            start: start,
            end: end,
            is_ep: true,
            is_double: false,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: node.cr,
            is_null: false,
            is_err: false
        }
    }

    pub fn pawn_promote_move(node: &Node, start: (i32, i32), end: (i32, i32), target_piece: u8) -> Move {
        Move {
            repr: Move::get_repr(start, end, target_piece),
            start: start,
            end: end,
            is_ep: false,
            is_double: false,
            is_castle: false,
            promote_to: target_piece,
            new_castling_rights: node.cr,
            is_null: false,
            is_err: false
        }
    }

    pub fn sliding_move(node: &Node, start: (i32, i32), end: (i32, i32)) -> Move {
        Move {
            repr: Move::get_repr(start, end, 0),
            start: start,
            end: end,
            is_ep: false,
            is_double: false,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: node.cr,
            is_null: false,
            is_err: false
        }
    }

    pub fn king_move(node: &Node, start: (i32, i32), end: (i32, i32)) -> Move {
        Move::sliding_move(node, start, end)
    }

    pub fn castle_move(node: &Node, start: (i32, i32), end: (i32, i32)) -> Move {
        let mut mv = Move::king_move(node, start, end);
        mv.is_castle = true;
        return mv
    }

    pub fn knight_move(node: &Node, start: (i32, i32), end: (i32, i32)) -> Move {
        Move::sliding_move(node, start, end)
    }

    pub fn null_move() -> Move {
        Move {
            repr: "".to_string(),
            start: (0, 0),
            end: (0, 0),
            is_ep: false,
            is_double: false,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: (true, true, true, true),
            is_null: true,
            is_err: false
        }
    }

    pub fn err_move() -> Move {
        Move {
            repr: "".to_string(),
            start: (0, 0),
            end: (0, 0),
            is_ep: false,
            is_double: false,
            is_castle: false,
            promote_to: 0,
            new_castling_rights: (true, true, true, true),
            is_null: true,
            is_err: true
        }
    }
}

// impl std::cmp::PartialEq for Move {
//     fn eq(&self, other: &Self) -> bool {
//         self.repr == other.repr;

//     }
// }
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}
