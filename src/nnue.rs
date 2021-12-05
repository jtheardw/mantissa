use std::fs::File;
use std::io::prelude::*;

use crate::bitboard::Bitboard;
use crate::util::*;

#[derive(Clone)]
pub struct Matrix {
    pub data: Vec<f32>,
    pub rows: usize,
    pub cols: usize
}

pub struct Network {
    pub weights: Vec<Matrix>,
    pub biases: Vec<Matrix>,
    pub activations: Vec<Matrix>,
}

pub const PAWN: i32 = 0;
pub const KNIGHT: i32 = 1;
pub const BISHOP: i32 = 2;
pub const ROOK: i32 = 3;
pub const QUEEN: i32 = 4;
pub const KING: i32 = 5;

static mut DEFAULT_NET: Network = Network {
    weights: Vec::new(),
    biases: Vec::new(),
    activations: Vec::new()
};

fn input_number(piece: i32, white: bool, idx: i32) -> usize {
    let piece_num = if white { piece } else { 6 + piece };
    let num = (piece_num * 64 + idx) as i16;
    // if num > 769 {
    //     panic!("num too big {}, {} {} {} {}", num, piece, white, rank, file);
    // }
    return num as usize;
}

fn relu(x: f32) -> f32 {
    x.max(0.0)
}

pub fn set_default_net(net: &Network) {
    unsafe {
        DEFAULT_NET = net.copy();
    }
}

pub fn get_default_net() -> Network {
    unsafe {
        DEFAULT_NET.copy()
    }
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<f32>) -> Matrix {
        Matrix {
            data: data,
            rows: rows,
            cols: cols
        }
    }

    pub fn size(&self) -> usize {
        self.rows * self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[col*self.rows + row]
    }

    pub fn copy(&self) -> Matrix {
        let cp = Matrix {
            data: self.data.to_vec(),
            cols: self.cols,
            rows: self.rows
        };
        return cp;
    }

    pub fn clear(&mut self) {
        for i in 0..self.size() {
            self.data[i] = 0.0;
        }
    }
}

impl Network {
    pub fn empty_net() -> Network {
        Network {
            weights: Vec::new(),
            biases: Vec::new(),
            activations: Vec::new()
        }
    }

    pub fn is_valid(&self) -> bool {
        return self.weights.len() > 0;
    }

    pub fn load(fname: &str) -> std::io::Result<Network> {
        let mut file = File::open(fname)?;

        // We'll read one 4-byte "word" at a time
        let mut buf: [u8; 4] = [0; 4];

        // magic number
        file.read(&mut buf)?;
        if buf[0..2] != [66, 90] { panic!("Magic word does not match expected 'BZ'"); }
        if buf[2..4] != [2, 0] { panic!("Network binary format version is not supported."); }

        // network id
        file.read(&mut buf)?;
        let id = u32::from_le_bytes(buf);
        println!("loading network with id {}", id);

        // topology information
        file.read(&mut buf)?;
        let _inputs = u32::from_le_bytes(buf);
        file.read(&mut buf)?;
        let _outputs = u32::from_le_bytes(buf);
        file.read(&mut buf)?;
        let layers = u32::from_le_bytes(buf);

        // hidden neuron counts in layers
        let mut hidden: Vec<u32> = Vec::new();
        for _ in 0..layers {
            file.read(&mut buf)?;
            hidden.push(u32::from_le_bytes(buf));
        }

        let mut weights = Vec::new();
        let mut biases = Vec::new();
        let mut activations = Vec::new();
        let mut input_size = 769;
        for l in 0..(layers + 1) {
            let output_size = if l == layers {
                1
            } else {
                hidden[l as usize]
            } as usize;
            weights.push(Matrix::new(output_size, input_size, vec![0.0; output_size * input_size]));
            biases.push(Matrix::new(output_size, 1, vec![0.0; output_size]));
            activations.push(Matrix::new(output_size, 1, vec![0.0; output_size]));

            input_size = output_size;
        }

        for i in 0..activations.len() {
            for j in 0..weights[i].size() {
                file.read(&mut buf)?;
                weights[i].data[j] = f32::from_le_bytes(buf);
            }

            for j in 0..biases[i].size() {
                file.read(&mut buf)?;
                biases[i].data[j] = f32::from_le_bytes(buf);
            }
        }

        let network = Network {
            weights: weights,
            biases: biases,
            activations: activations
        };

        return Ok(network);
    }

    pub fn copy(&self) -> Network {
        let mut activations: Vec<Matrix> = Vec::new();

        let num_layers = self.activations.len();

        for i in 0..num_layers {
            activations.push(self.activations[i].copy());
        }

        let mut weights = Vec::new();
        let mut biases = Vec::new();

        for w in &self.weights {
            weights.push(w.copy());
        }
        for b in &self.biases {
            biases.push(b.copy());
        }
        Network {
            weights: weights,
            biases: biases,
            activations: activations,
        }
    }

    fn set_bb_activations(&mut self, bb: u64, piece_num: i32, is_white: bool) {
        let mut bb = bb;
        while bb != 0 {
            let idx = bb.trailing_zeros() as i32;
            bb &= bb - 1;
            let input_number = input_number(piece_num, is_white, idx);
            for j in 0..self.activations[0].size() {
                self.activations[0].data[j] += self.weights[0].get(j, input_number);
            }
        }
    }

    pub fn set_activations(&mut self, pos: &Bitboard) {
        for i in 0..self.activations[0].data.len() {
            self.activations[0].data[i] = self.biases[0].data[i];
        }

        for side in [Color::White, Color::Black] {
            let is_white = side == Color::White;
            let me = side as usize;

            self.set_bb_activations(pos.pawn[me], PAWN, is_white);
            self.set_bb_activations(pos.knight[me], KNIGHT, is_white);
            self.set_bb_activations(pos.bishop[me], BISHOP, is_white);
            self.set_bb_activations(pos.rook[me], ROOK, is_white);
            self.set_bb_activations(pos.queen[me], QUEEN, is_white);
            self.set_bb_activations(pos.king[me], KING, is_white);
        }

        if pos.side_to_move == Color::White {
            for j in 0..self.activations[0].size() {
                self.activations[0].data[j] += self.weights[0].get(j, 768);
            }
        }
    }

    pub fn activate(&mut self, piece: i32, color: Color, idx: i8) {
        let feature_idx = input_number(piece, color == Color::White, idx as i32);
        for j in 0..self.activations[0].size() {
            self.activations[0].data[j] += self.weights[0].get(j, feature_idx);
        }
    }

    pub fn deactivate(&mut self, piece: i32, color: Color, idx: i8) {
        let feature_idx = input_number(piece, color == Color::White, idx as i32);
        for j in 0..self.activations[0].size() {
            self.activations[0].data[j] -= self.weights[0].get(j, feature_idx);
        }
    }

    pub fn move_piece(&mut self, piece: i32, color: Color, start: i8, end: i8) {
        self.deactivate(piece, color, start);
        self.activate(piece, color, end);
    }

    pub fn black_turn(&mut self) {
        for j in 0..self.activations[0].size() {
            self.activations[0].data[j] -= self.weights[0].get(j, 768);
        }
    }

    pub fn white_turn(&mut self) {
        for j in 0..self.activations[0].size() {
            self.activations[0].data[j] += self.weights[0].get(j, 768);
        }
    }

    pub fn nnue_eval(&mut self) -> i32 {
        // assumes that inputs already updated and so on
        let mut output = 0.0;
        for layer in 1..self.activations.len() {
            for i in 0..self.activations[layer].size() {
                self.activations[layer].data[i] = 0.0;
                for j in 0..self.activations[layer-1].size() {
                    self.activations[layer].data[i] += relu(self.activations[layer-1].data[j]) * self.weights[layer].get(i, j);
                }

                if layer != self.activations.len() - 1 {
                    self.activations[layer].data[i] = self.activations[layer].data[i] + self.biases[layer].data[i];
                } else {
                    output = self.activations[layer].data[i] + self.biases[layer].data[i];
                }
            }
        }
        return (output * 10.0).floor() as i32;
    }

    pub fn full_eval(&mut self, pos: &Bitboard) -> i32 {
        self.set_activations(pos);
        return self.nnue_eval();
    }
}