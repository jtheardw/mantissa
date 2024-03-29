// use std::arch::x86_64::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::prelude::*;
use std::simd::{num::SimdFloat, *};

use crate::bitboard::Bitboard;
use crate::default_nnue::*;
use crate::util::*;

#[derive(Clone)]
pub struct Matrix {
    pub data: Vec<f32>,
    pub rows: usize,
    pub cols: usize
}

pub struct SlowNetwork {
    pub weights: Vec<Matrix>,
    pub biases: Vec<Matrix>,
    pub activations: Vec<Matrix>,
}

fn input_number(piece: u8, white: bool, idx: i32) -> usize {
    let piece_num = if white { piece } else { 6 + piece };
    let num = (piece_num as i32 * 64 + idx) as i16;
    return num as usize;
}

pub fn region(idx: i8) -> i8 {
  if idx >= 24 { return 3; }
  const MAP : usize = 193491139974165;
  return ((MAP >> (idx*2)) & 3) as i8;
}

fn flip_input(input: i16) -> i16 {
    // need to flip "color" and "rank"
    // flip square
    if input == 768 { return 768; }
    let idx = (input % 64) ^ 56;
    let orig_piece_num = input / 64;
    let piece_type = orig_piece_num % 6;
    // >= 6 meant it was black, so we need to undo that and vice versa
    let piece_color = orig_piece_num >= 6;
    let piece_num = if piece_color {piece_type} else {6 + piece_type};

    return piece_num * 64 + idx;
}

fn relu(x: f32) -> f32 {
    x.max(0.0)
}

pub fn get_default_net() -> Network {
    unsafe {
        Network::load_default()
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

impl SlowNetwork {
    pub fn empty_net() -> SlowNetwork {
        SlowNetwork {
            weights: Vec::new(),
            biases: Vec::new(),
            activations: Vec::new()
        }
    }

    pub fn is_valid(&self) -> bool {
        return self.weights.len() > 0;
    }

    pub fn print(&self) {
        println!("pub static DEFAULT_NNUE_FEATURE_WEIGHTS: [f32; 786432] = [");
        for i in 0..(768*4) {
            for j in 0..32 {
                print!("\t");
                for k in 0..8 {
                    let idx = j * 8 + k;
                    print!("{}", self.weights[0].get(idx, i));
                    if i != (768*4 - 1) || j != 31 || k != 7 {
                        print!(", ");
                    }
                }
                print!("\n");
            }
        }
        println!("];");
        println!("");
        println!("pub const DEFAULT_NNUE_HIDDEN_BIAS: [f32; 256] = [");
        for i in 0..32 {
            print!("\t");
            for j in 0..8 {
                let idx = i * 8 + j;
                print!("{}", self.biases[0].data[idx]);
                if i != 31 || j != 7 {
                    print!(", ");
                }
            }
            print!("\n");
        }
        println!("];");
        println!("");
        println!("pub const DEFAULT_NNUE_HIDDEN_WEIGHTS: [f32; 512] = [");
        for i in 0..64 {
            print!("\t");
            for j in 0..8 {
                let idx = i * 8 + j;
                print!("{}", self.weights[1].data[idx]);
                if i != 63 || j != 7 {
                    print!(", ");
                }
            }
            print!("\n");
        }
        println!("];");
        println!("");
        println!("pub const DEFAULT_NNUE_OUTPUT_BIAS: f32 = {};", self.biases[1].data[0]);
    }

    pub fn load(fname: &str) -> std::io::Result<SlowNetwork> {
        let mut file = File::open(fname)?;

        // We'll read one 4-byte "word" at a time
        let mut buf: [u8; 4] = [0; 4];

        // magic number
        file.read(&mut buf)?;
        if buf[0..2] != [66, 90] { panic!("Magic word does not match expected 'BZ'"); }
        if buf[2..4] != [2, 0] { panic!("SlowNetwork binary format version is not supported."); }

        // network id
        file.read(&mut buf)?;
        let _id = u32::from_le_bytes(buf);
        // println!("loading network with id {}", id);

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
        let mut input_size = 768*4;
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

        let network = SlowNetwork {
            weights: weights,
            biases: biases,
            activations: activations
        };

        return Ok(network);
    }

    pub fn copy(&self) -> SlowNetwork {
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
        SlowNetwork {
            weights: weights,
            biases: biases,
            activations: activations,
        }
    }

    fn set_bb_activations(&mut self, bb: u64, piece_num: u8, is_white: bool) {
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
        for i in 0..self.activations[0].data.len() {
            println!("{} {}", i, self.activations[0].data[i]);
        }
    }

    #[inline]
    pub fn activate(&mut self, piece: u8, color: Color, idx: i8) {
        let feature_idx = input_number(piece, color == Color::White, idx as i32);
        for j in 0..self.activations[0].size() {
            self.activations[0].data[j] += self.weights[0].get(j, feature_idx);
        }
    }

    #[inline]
    pub fn deactivate(&mut self, piece: u8, color: Color, idx: i8) {
        let feature_idx = input_number(piece, color == Color::White, idx as i32);
        for j in 0..self.activations[0].size() {
            self.activations[0].data[j] -= self.weights[0].get(j, feature_idx);
        }
    }

    #[inline]
    pub fn move_piece(&mut self, piece: u8, color: Color, start: i8, end: i8) {
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

    pub fn save_image(&self, file: &str) -> std::io::Result<()> {
        let ss = 8; // supersampling
        let border = [0, 0, 0];
        let width  = (12*8*4)*ss + 11 * 4 + 3 * ss * 4;
        let neurons = self.activations[0].size();
        let height = 8*(neurons / 2)*ss + (neurons / 2)-1;
        let mut w = BufWriter::new(File::create(format!("{}.ppm", file))?);
        writeln!(&mut w, "P6")?;
        writeln!(&mut w, "{} {}", width, height)?;
        writeln!(&mut w, "255")?;
        // for n in 0..128 {
        // for n in 0..128 {
        for n in 0..neurons {
            if n != 0 { for _ in 0..width { w.write(&border)?; } }
            let mut upper = 0.0;
            let mut lower = 0.0;
            for rank in (0..8).rev() {
                for _ in 0..ss {
                    for kr in 0..4 {
                        if kr != 0 {
                            for _ in 0..(ss*4) { w.write(&border)?; }
                        }

                        let kr_offset = kr * 768;
                        for i in 0..768 {
                            upper = self.weights[0].get(n, kr_offset + i).max(upper);
                            lower = self.weights[0].get(n, kr_offset + i).min(lower);
                        }
                        let scale = upper.max(-lower);

                        for side in [true, false] {
                            for piece in (0..6).rev() {
                                if piece != 5 || !side { w.write(&border)?; }
                                for file in 0..8 {
                                    // let x : usize = piece*64 + rank*8 + file;
                                    let idx = rank * 8 + file;
                                    let inp = input_number(piece, side, idx) + kr_offset as usize;
                                    let normed = ((self.weights[0].get(n, inp as usize) / scale) + 1.0) / 2.0;
                                    // let normed = ((self.w1[n][x] / scale) + 1.0) / 2.0;
                                    debug_assert!(1.0 >= normed && normed >= 0.0, "out of range");
                                    let r = (normed * 255.0).round() as u8;
                                    let g = (normed * 255.0).round() as u8;
                                    let b = (32.0 + normed * 191.0).round() as u8;
                                    for _ in 0..ss { w.write(&[r, g, b])?; }
                                }
                            }
                        }
                    }
                }
            }
        }
        return Ok(());
    }
}

#[repr(align(32))]
pub struct Network {
    pub feature_weights: Vec<[f32x8; 64]>,
    pub hidden_weights: [f32x8; 64],
    pub hidden_biases: [f32x8; 64],
    pub hidden_activations: [f32x8; 64],
    pub output_bias: f32,
    pub flip: bool
}

fn all_zeros() -> f32x8 {
    f32x8::from_array([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
}

impl Network {
    pub fn empty_net() -> Network {
        Network {
            feature_weights: vec![[all_zeros(); 64]; 769],
            hidden_weights: [all_zeros(); 64],
            hidden_biases: [all_zeros(); 64],
            hidden_activations: [all_zeros(); 64],
            output_bias: 0.0,
            flip: false
        }
    }

    pub fn copy(&self) -> Network {
        Network {
            feature_weights: self.feature_weights.to_vec(),
            hidden_weights: self.hidden_weights,
            hidden_biases: self.hidden_biases,
            hidden_activations: self.hidden_activations,
            output_bias: self.output_bias,
            flip: self.flip
        }
    }

    pub fn is_valid(&self) -> bool {true}

    pub unsafe fn load_default() -> Network {
        // inputs
        let mut feature_weights = vec![[all_zeros(); 64]; 3072];
        let mut weights: Vec<[f32; 512]> = vec![[0.0; 512]; 3072];
        for k in 0..4 {
            for j in 0..768 {
                for i in 0..128 {
                    let weight = DEFAULT_NNUE_FEATURE_WEIGHTS[(k * 768 + j)*256 + i] as f32;
                    weights[k * 768 + j][i] = weight;
                    weights[k * 768 + flip_input(j as i16) as usize][i + 256] = weight;
                }
                for i in 128..256 {
                    let weight = DEFAULT_NNUE_FEATURE_WEIGHTS[j * 256 + i] as f32;
                    weights[k * 768 + j][i] = weight;
                    weights[k * 768 + flip_input(j as i16) as usize][i + 256] = weight;
                }
            }
        }

        for i in 0..3072 {
            for j in 0..64 {
                feature_weights[i][j] = f32x8::from_slice(&weights[i][j*8..j*8+8]);
            }
        }

        let mut hidden_biases = [all_zeros(); 64];
        let mut biases = [0.0; 512];
        for i in 0..512 {
            biases[i] = DEFAULT_NNUE_HIDDEN_BIAS[i % 256];
        }
        for i in 0..64 {
            hidden_biases[i] = f32x8::from_slice(&biases[i*8..i*8+8]);
        }

        let mut hidden_weights = [all_zeros(); 64];
        let mut weights = [0.0; 512];
        for i in 0..512 {
            weights[i] = DEFAULT_NNUE_HIDDEN_WEIGHTS[i];
        }
        for i in 0..64 {
            hidden_weights[i] = f32x8::from_slice(&weights[i*8..i*8+8]);
        }

        let output_bias = DEFAULT_NNUE_OUTPUT_BIAS;

        let network = Network {
            feature_weights: feature_weights,
            hidden_weights: hidden_weights,
            hidden_biases: hidden_biases,
            hidden_activations: [all_zeros(); 64],
            output_bias: output_bias,
            flip: false
        };

        return network;
    }

    pub unsafe fn load(fname: &str) -> std::io::Result<Network> {
        let mut file = File::open(fname)?;

        // We'll read one 4-byte "word" at a time
        let mut buf: [u8; 4] = [0; 4];

        // magic number
        file.read(&mut buf)?;
        if buf[0..2] != [66, 90] { panic!("Magic word does not match expected 'BZ'"); }
        if buf[2..4] != [2, 0] { panic!("SlowNetwork binary format version is not supported."); }

        // network id
        file.read(&mut buf)?;
        let _id = u32::from_le_bytes(buf);
        // println!("loading network with id {}", id);

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

        // inputs
        let mut feature_weights = vec![[all_zeros(); 64]; 3072];
        let mut weights = vec![[0.0; 512]; 3072];
        for k in 0..4 {
            for j in 0..768 {
                for i in 0..512 {
                    file.read(&mut buf)?;
                    if i < 256 {
                        weights[k * 768 + j][i] = f32::from_le_bytes(buf);
                        weights[k * 768 + flip_input(j as i16) as usize][i + 256] = f32::from_le_bytes(buf);
                    }
                }
            }
        }

        for i in 0..3072 {
            for j in 0..64 {
                feature_weights[i][j] = f32x8::from_slice(&weights[i][j*8..j*8+8]);
            }
        }

        let mut hidden_biases = [all_zeros(); 64];
        let mut biases = [0.0; 512];
        for i in 0..512 {
            file.read(&mut buf)?;
            if i < 256 {
                biases[i] = f32::from_le_bytes(buf);
                biases[i+256] = f32::from_le_bytes(buf);
            }
        }
        for i in 0..64 {
            hidden_biases[i] = f32x8::from_slice(&biases[i*8..i*8+8]);
        }

        let mut hidden_weights = [all_zeros(); 64];
        let mut weights = [0.0; 512];
        for i in 0..512 {
            file.read(&mut buf)?;
            weights[i] = f32::from_le_bytes(buf);
        }
        for i in 0..64 {
            hidden_weights[i] = f32x8::from_slice(&weights[i*8..i*8+8]);
        }

        file.read(&mut buf)?;
        let output_bias = f32::from_le_bytes(buf);

        let network = Network {
            feature_weights: feature_weights,
            hidden_weights: hidden_weights,
            hidden_biases: hidden_biases,
            hidden_activations: [all_zeros(); 64],
            output_bias: output_bias,
            flip: false
        };

        return Ok(network);
    }

    #[inline]
    pub fn activate(&mut self, piece: u8, color: Color, idx: i8, wkr: usize, bkr: usize) {
        let inp = input_number(piece, color == Color::White, idx as i32);
        // TODO note to self: make sure that flipping happens _within_ each kr
        let feature_idx = inp + wkr as usize;
        let flipped_idx = inp + bkr as usize;
        for j in 0..8 {
            let j = j * 4;
            self.hidden_activations[j] += self.feature_weights[feature_idx][j];
            self.hidden_activations[j+1] += self.feature_weights[feature_idx][j+1];
            self.hidden_activations[j+2] += self.feature_weights[feature_idx][j+2];
            self.hidden_activations[j+3] += self.feature_weights[feature_idx][j+3];
            self.hidden_activations[j+32] += self.feature_weights[flipped_idx][j+32];
            self.hidden_activations[j+33] += self.feature_weights[flipped_idx][j+33];
            self.hidden_activations[j+34] += self.feature_weights[flipped_idx][j+34];
            self.hidden_activations[j+35] += self.feature_weights[flipped_idx][j+35];
        }
    }

    #[inline]
    pub fn deactivate(&mut self, piece: u8, color: Color, idx: i8, wkr: usize, bkr: usize) {
        let inp = input_number(piece, color == Color::White, idx as i32);
        // TODO note to self: make sure that flipping happens _within_ each kr
        let feature_idx = inp + wkr as usize;
        let flipped_idx = inp + bkr as usize;
        for j in 0..8 {
            let j = j * 4;
            self.hidden_activations[j] -= self.feature_weights[feature_idx][j];
            self.hidden_activations[j+1] -= self.feature_weights[feature_idx][j+1];
            self.hidden_activations[j+2] -= self.feature_weights[feature_idx][j+2];
            self.hidden_activations[j+3] -= self.feature_weights[feature_idx][j+3];
            self.hidden_activations[j+32] -= self.feature_weights[flipped_idx][j+32];
            self.hidden_activations[j+33] -= self.feature_weights[flipped_idx][j+33];
            self.hidden_activations[j+34] -= self.feature_weights[flipped_idx][j+34];
            self.hidden_activations[j+35] -= self.feature_weights[flipped_idx][j+35];
        }
    }

    pub fn clear_kr(&mut self) {
        for i in 0..16 {
            self.hidden_activations[i] = self.hidden_biases[i];
        }
        for i in 32..48 {
            self.hidden_activations[i] = self.hidden_biases[i];
        }
    }

    pub fn set_kr_bb_activations(&mut self, bb: u64, piece_num: u8, is_white: bool, wkr: usize, bkr: usize) {
        let mut bb = bb;
        while bb != 0 {
            let idx = bb.trailing_zeros() as i8;
            bb &= bb - 1;
            self.activate_kr(piece_num, if is_white {Color::White} else {Color::Black}, idx, wkr, bkr);
        }
    }

    #[inline]
    pub fn activate_kr(&mut self, piece: u8, color: Color, idx: i8, wkr: usize, bkr: usize) {
        let inp = input_number(piece, color == Color::White, idx as i32);
        // TODO note to self: make sure that flipping happens _within_ each kr
        let feature_idx = inp + wkr as usize;
        let flipped_idx = inp + bkr as usize;
        for j in 0..4 {
            let j = j * 4;
            self.hidden_activations[j] += self.feature_weights[feature_idx][j];
            self.hidden_activations[j+1] += self.feature_weights[feature_idx][j+1];
            self.hidden_activations[j+2] += self.feature_weights[feature_idx][j+2];
            self.hidden_activations[j+3] += self.feature_weights[feature_idx][j+3];
            self.hidden_activations[j+32] += self.feature_weights[flipped_idx][j+32];
            self.hidden_activations[j+33] += self.feature_weights[flipped_idx][j+33];
            self.hidden_activations[j+34] += self.feature_weights[flipped_idx][j+34];
            self.hidden_activations[j+35] += self.feature_weights[flipped_idx][j+35];
        }
    }

    #[inline]
    pub fn deactivate_kr(&mut self, piece: u8, color: Color, idx: i8, wkr: usize, bkr: usize) {
        let inp = input_number(piece, color == Color::White, idx as i32);
        // TODO note to self: make sure that flipping happens _within_ each kr
        let feature_idx = inp + wkr as usize;
        let flipped_idx = inp + bkr as usize;
        for j in 0..4 {
            let j = j * 4;
            self.hidden_activations[j] -= self.feature_weights[feature_idx][j];
            self.hidden_activations[j+1] -= self.feature_weights[feature_idx][j+1];
            self.hidden_activations[j+2] -= self.feature_weights[feature_idx][j+2];
            self.hidden_activations[j+3] -= self.feature_weights[feature_idx][j+3];
            self.hidden_activations[j+32] -= self.feature_weights[flipped_idx][j+32];
            self.hidden_activations[j+33] -= self.feature_weights[flipped_idx][j+33];
            self.hidden_activations[j+34] -= self.feature_weights[flipped_idx][j+34];
            self.hidden_activations[j+35] -= self.feature_weights[flipped_idx][j+35];
        }
    }

    #[inline]
    pub fn move_piece(&mut self, piece: u8, color: Color, start: i8, end: i8, wkr: usize, bkr: usize) {
        self.deactivate(piece, color, start, wkr, bkr);
        self.activate(piece, color, end, wkr, bkr);
    }

    pub fn black_turn(&mut self) {
        self.flip = true;
    }

    pub fn white_turn(&mut self) {
        self.flip = false;
    }

    pub fn set_bb_activations(&mut self, bb: u64, piece_num: u8, is_white: bool, wkr: usize, bkr: usize) {
        let mut bb = bb;
        while bb != 0 {
            let idx = bb.trailing_zeros() as i8;
            bb &= bb - 1;
            self.activate(piece_num, if is_white {Color::White} else {Color::Black}, idx, wkr, bkr);
        }
    }

    pub fn set_activations(&mut self, pos: &Bitboard) {
        for i in 0..64 {
            self.hidden_activations[i] = self.hidden_biases[i];
        }

        let wkr = region(pos.king[1].trailing_zeros() as i8) as usize * 768;
        let bkr = region((pos.king[0].trailing_zeros() ^ 56) as i8) as usize * 768;
        for side in [Color::White, Color::Black] {
            let is_white = side == Color::White;
            let me = side as usize;

            self.set_bb_activations(pos.pawn[me], PAWN, is_white, wkr, bkr);
            self.set_bb_activations(pos.knight[me], KNIGHT, is_white, wkr, bkr);
            self.set_bb_activations(pos.bishop[me], BISHOP, is_white, wkr, bkr);
            self.set_bb_activations(pos.rook[me], ROOK, is_white, wkr, bkr);
            self.set_bb_activations(pos.queen[me], QUEEN, is_white, wkr, bkr);
            self.set_bb_activations(pos.king[me], KING, is_white, wkr, bkr);
        }

        if pos.side_to_move == Color::White {
            self.white_turn();
        } else {
            self.black_turn();
        }
    }

    pub fn nnue_eval(&mut self) -> i32 {
        // assumes that inputs already updated and so on
        let mut output = self.output_bias;
        let mut total = all_zeros();
        let relu_zeros = all_zeros();
        for i in 0..64 {
            let idx = if self.flip { i ^ 32 } else { i };
            let relud = self.hidden_activations[idx].simd_max(relu_zeros);

            total += relud * self.hidden_weights[i];
        }
        output += total.reduce_sum();

        if self.flip {output *= -1.0;}
        return (output * 9.0).floor() as i32;
    }

    pub fn print_eval(&mut self, board: &Bitboard) {
        let base_score = self.nnue_eval() as f32 / 1000.0;
        let wkr = region(board.king[1].trailing_zeros() as i8) as usize * 768;
        let bkr = region((board.king[0].trailing_zeros() ^ 56) as i8) as usize * 768;
        // let wkr = region(board.king[1].trailing_zeros() as i8);
        // let bkr = region((board.king[0].trailing_zeros() ^ 56) as i8);
        eprint!("\x1B[0m");
        for _ in 0..24 { eprint!("\n"); }
        eprint!("\x1B[24A");
        for rank in (0..8).rev() {
            for file in 0..8 {
                let idx = rank*8 + file;

                let parity = (rank + file) % 2 == 0;
                let bg = if parity { (181, 135, 99) } else { (212, 190, 154) };
                eprint!("\x1B[48;2;{};{};{}m", bg.0, bg.1, bg.2);
                eprint!("       \x1B[7D\x1B[B");

                let mut piece = board.piece_at_square(idx, Color::White);
                let mut color = Color::White;
                if piece == 0 {
                    piece = board.piece_at_square(idx, Color::Black);
                    color = Color::Black;
                }
                if piece == 0 {
                    eprint!("       \x1B[7D\x1B[B");
                    eprint!("       \x1B[7D");
                    eprint!("\x1B[2A\x1B[7C");
                    continue;
                }

                match color {
                    Color::White => eprint!("\x1B[97m"),
                    Color::Black => eprint!("\x1B[30m")
                };
                eprint!("   {}   \x1B[7D\x1B[B", (piece - 32) as char);
                if piece == b'k' {
                    eprint!("       \x1B[7D");
                } else {
                    let piece_num = match piece {
                        b'p' => PAWN,
                        b'n' => KNIGHT,
                        b'b' => BISHOP,
                        b'r' => ROOK,
                        b'q' => QUEEN,
                        _ => {panic!("there is no piece here")}
                    };
                    self.deactivate(piece_num, color, idx, wkr, bkr);
                    let hypothetical_score = self.nnue_eval() as f32 / 1000.0;
                    let ofs = base_score - hypothetical_score;
                    self.activate(piece_num, color, idx, wkr, bkr);
                    if ofs.abs() >= 10.0 {
                        eprint!(" {:+5.1} \x1B[7D", ofs);
                    }
                    else {
                        eprint!(" {:+5.2} \x1B[7D", ofs);
                    }
                }
                eprint!("\x1B[2A\x1B[7C");
            }
            eprint!("\x1B[0m\r\x1B[3B");
        }
        eprintln!("NNUE evaluation (White View): {:+.2}", base_score);
    }
}
