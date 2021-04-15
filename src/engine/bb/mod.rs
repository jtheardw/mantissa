use rand::Rng;
use std::fmt;
use std::vec::Vec;

pub mod board_eval;

// LSB bit is A1, second is B1, ninth is A2, ... MSB is H8

const KING_VALUE: i32 = 200000;
const QUEEN_VALUE: i32 = 9000;
const ROOK_VALUE: i32 = 5000;
const BISHOP_VALUE: i32 = 3200;
const KNIGHT_VALUE: i32 = 3000;
const PAWN_VALUE: i32 = 1000;

// Credit to Pradyumna Kannan for magic numbers
const ROOK_MAGIC_SHIFTS: [u32; 64] =
[
	52, 53, 53, 53, 53, 53, 53, 52,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 54, 54, 54, 54, 53,
	53, 54, 54, 53, 53, 53, 53, 53
];

const ROOK_MAGIC_NUMBERS: [u64; 64] =
[
	0x0080001020400080, 0x0040001000200040, 0x0080081000200080, 0x0080040800100080,
	0x0080020400080080, 0x0080010200040080, 0x0080008001000200, 0x0080002040800100,
	0x0000800020400080, 0x0000400020005000, 0x0000801000200080, 0x0000800800100080,
	0x0000800400080080, 0x0000800200040080, 0x0000800100020080, 0x0000800040800100,
	0x0000208000400080, 0x0000404000201000, 0x0000808010002000, 0x0000808008001000,
	0x0000808004000800, 0x0000808002000400, 0x0000010100020004, 0x0000020000408104,
	0x0000208080004000, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
	0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
	0x0000204000800080, 0x0000200040401000, 0x0000100080802000, 0x0000080080801000,
	0x0000040080800800, 0x0000020080800400, 0x0000020001010004, 0x0000800040800100,
	0x0000204000808000, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
	0x0000040008008080, 0x0000020004008080, 0x0000010002008080, 0x0000004081020004,
	0x0000204000800080, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
	0x0000040008008080, 0x0000020004008080, 0x0000800100020080, 0x0000800041000080,
	0x00FFFCDDFCED714A, 0x007FFCDDFCED714A, 0x003FFFCDFFD88096, 0x0000040810002101,
	0x0001000204080011, 0x0001000204000801, 0x0001000082000401, 0x0001FFFAABFAD1A2
];

const BISHOP_MAGIC_SHIFTS: [u32; 64] =
[
	58, 59, 59, 59, 59, 59, 59, 58,
	59, 59, 59, 59, 59, 59, 59, 59,
	59, 59, 57, 57, 57, 57, 59, 59,
	59, 59, 57, 55, 55, 57, 59, 59,
	59, 59, 57, 55, 55, 57, 59, 59,
	59, 59, 57, 57, 57, 57, 59, 59,
	59, 59, 59, 59, 59, 59, 59, 59,
	58, 59, 59, 59, 59, 59, 59, 58
];

const BISHOP_MAGIC_NUMBERS: [u64; 64] =
[
	0x0002020202020200, 0x0002020202020000, 0x0004010202000000, 0x0004040080000000,
	0x0001104000000000, 0x0000821040000000, 0x0000410410400000, 0x0000104104104000,
	0x0000040404040400, 0x0000020202020200, 0x0000040102020000, 0x0000040400800000,
	0x0000011040000000, 0x0000008210400000, 0x0000004104104000, 0x0000002082082000,
	0x0004000808080800, 0x0002000404040400, 0x0001000202020200, 0x0000800802004000,
	0x0000800400A00000, 0x0000200100884000, 0x0000400082082000, 0x0000200041041000,
	0x0002080010101000, 0x0001040008080800, 0x0000208004010400, 0x0000404004010200,
	0x0000840000802000, 0x0000404002011000, 0x0000808001041000, 0x0000404000820800,
	0x0001041000202000, 0x0000820800101000, 0x0000104400080800, 0x0000020080080080,
	0x0000404040040100, 0x0000808100020100, 0x0001010100020800, 0x0000808080010400,
	0x0000820820004000, 0x0000410410002000, 0x0000082088001000, 0x0000002011000800,
	0x0000080100400400, 0x0001010101000200, 0x0002020202000400, 0x0001010101000200,
	0x0000410410400000, 0x0000208208200000, 0x0000002084100000, 0x0000000020880000,
	0x0000001002020000, 0x0000040408020000, 0x0004040404040000, 0x0002020202020000,
	0x0000104104104000, 0x0000002082082000, 0x0000000020841000, 0x0000000000208800,
	0x0000000010020200, 0x0000000404080200, 0x0000040404040400, 0x0002020202020200
];

const FILE_MASKS: [u64; 8] =
    [
        0x0101010101010101,     // A FILE
        0x0202020202020202,     // B FILE
        0x0404040404040404,     // ...
        0x0808080808080808,
        0x1010101010101010,
        0x2020202020202020,
        0x4040404040404040,
        0x8080808080808080      // H FILE
    ];

const RANK_MASKS: [u64; 8] =
    [
        0x00000000000000FF,     // 1ST RANK
        0x000000000000FF00,     // 2ND RANK
        0x0000000000FF0000,     // ...
        0x00000000FF000000,
        0x000000FF00000000,
        0x0000FF0000000000,
        0x00FF000000000000,
        0xFF00000000000000      // 8TH RANK
    ];

const CENTER_MASK: u64 = 0x0000001818000000;
const NEAR_CENTER_MASK: u64 = 0x00003C24243C0000;
const AHEAD_RANKS: [[u64; 8]; 2] =
    [
        [
            0x0000000000000000,
            0x00000000000000FF,
            0x000000000000FFFF,
            0x0000000000FFFFFF,
            0x00000000FFFFFFFF,
            0x000000FFFFFFFFFF,
            0x0000FFFFFFFFFFFF,
            0x00FFFFFFFFFFFFFF,
        ],
        [
            0xFFFFFFFFFFFFFF00,
            0xFFFFFFFFFFFF0000,
            0xFFFFFFFFFF000000,
            0xFFFFFFFF00000000,
            0xFFFFFF0000000000,
            0xFFFF000000000000,
            0xFF00000000000000,
            0x0000000000000000
        ],
    ];

// modifiable scaling factors for the sub-evaluators
#[derive(Copy, Clone)]
pub struct EvalParams {
    pub mobility: i32,
    pub pdf: i32,         // pawn defense
    pub dbb: i32,         // double-bishop
    pub castle: i32,
    pub pav: i32,         // pawn advancement
    pub rook_on_seventh: i32,
    pub rook_on_open: i32,
    pub early_queen_penalty: i32,
    pub king_danger: i32,

    // pawn structure
    pub passed_pawn: i32,
    pub center_pawn: i32,
    pub near_center_pawn: i32,
    pub isolated_pawn: i32,
    pub doubled_pawn: i32,
    pub backwards_pawn: i32,

    // piece tables
    pub pawn_pt_offset: i32,
    pub pawn_pt_scale: i32,

    pub bishop_pt_offset: i32,
    pub bishop_pt_scale: i32,

    pub knight_pt_offset: i32,
    pub knight_pt_scale: i32,

    pub king_mg_pt_offset: i32,
    pub king_mg_pt_scale: i32,

    pub king_eg_pt_offset: i32,
    pub king_eg_pt_scale: i32,

    // tempo
    pub tempo_bonus: i32,

    // slight bonus to leader in material
    pub material_advantage: i32
}

impl EvalParams {
    pub fn default_params() -> EvalParams {
        EvalParams {
            mobility: 15,
            pdf: 70,
            dbb: 550,
            castle: 441,
            pav: 36,
            rook_on_seventh: 140,
            rook_on_open: 77,
            early_queen_penalty: -254,

            passed_pawn: 531,
            center_pawn: 165,
            near_center_pawn: 30,
            isolated_pawn: -170,
            doubled_pawn: -263,
            backwards_pawn: -260,

            pawn_pt_offset: -52,
            pawn_pt_scale: 102,

            bishop_pt_offset: -16,
            bishop_pt_scale: 104,

            knight_pt_offset: 80,
            knight_pt_scale: 127,

            king_mg_pt_offset: 12,
            king_mg_pt_scale: 97,

            king_eg_pt_offset: -17,
            king_eg_pt_scale: 114,

            tempo_bonus: 152,
            material_advantage: 212,
            king_danger: -65,
        }
    }
}

pub struct BB {
    pub white_turn: bool,

    pub king: [u64; 2],
    queen: [u64; 2],
    pub rook: [u64; 2],
    bishop: [u64; 2],
    knight: [u64; 2],
    pawn: [u64; 2],
    pub composite: [u64; 2],

    knight_mask: [u64; 64],
    rook_mask: [u64; 64],
    bishop_mask: [u64; 64],
    king_mask: [u64; 64],

    // won't need a pawn mask
    castling_rights: u64,
    castled: [bool; 2],
    rook_magic_table: Vec<Vec<u64>>,//[[u64; 4096]; 64],
    bishop_magic_table: Vec<Vec<u64>>,//[[u64; 512]; 64],

    // for moves and such
    pub ep: i32,

    // stacks
    ep_stack: Vec<i32>,
    pub cap_stack: Vec<u8>,
    cr_stack: Vec<u64>,
    pub history: Vec<u64>,
    pawn_history: Vec<u64>,

    // eval
    pub material: i32,
    pub hash: u64,
    pub pawn_hash: u64,
    pub zobrist_table: ([[u64; 12]; 64], (u64, u64)),
    pub phase: i32,

    // incrementally updated piece table scores
    pub king_mg_pt_score: i32,
    pub king_eg_pt_score: i32,
    pub pawn_pt_score: i32,
    pub knight_pt_score: i32,
    pub bishop_pt_score: i32,

    pub eval_params: EvalParams,

    // for info reporting
    pub nodes_evaluated: u64,
    pub tt_hits: u64
}

impl BB {
    // convenience functions.  Mostly for single time generation and debugging
    #[inline]
    pub fn coord_to_idx(coord: (i32, i32)) -> i32 {
        (coord.1 << 3) + coord.0
    }

    #[inline]
    pub fn idx_to_coord(idx: i32) -> (i32, i32) {
        (idx % 8, idx >> 3)
    }

    #[inline]
    pub fn idx_to_bb(idx: i32) -> u64 {
        1 << idx
    }

    #[inline]
    pub fn coord_to_bb(coord: (i32, i32)) -> u64 {
        BB::idx_to_bb(BB::coord_to_idx(coord))
    }

    #[inline]
    fn at_idx(bb: u64, idx: i32) -> u64{
        bb & (1 << idx)
    }

    // constructors
    pub fn default_board(
        knight_mask: [u64; 64],
        rook_mask: [u64; 64],
        bishop_mask: [u64; 64],
        king_mask: [u64; 64],
        zobrist_table: ([[u64; 12]; 64], (u64, u64)),
        eval_params: EvalParams
    ) -> BB {
        let black_king = BB::coord_to_bb((4, 7));
        let white_king = BB::coord_to_bb((4, 0));
        let king = [black_king, white_king];

        let black_queen = BB::coord_to_bb((3, 7));
        let white_queen = BB::coord_to_bb((3, 0));
        let queen = [black_queen, white_queen];

        let black_rook = BB::coord_to_bb((0, 7)) | BB::coord_to_bb((7,7));
        let white_rook = BB::coord_to_bb((0, 0)) | BB::coord_to_bb((7,0));
        let rook = [black_rook, white_rook];

        let black_bishop = BB::coord_to_bb((2, 7)) | BB::coord_to_bb((5,7));
        let white_bishop = BB::coord_to_bb((2, 0)) | BB::coord_to_bb((5,0));
        let bishop = [black_bishop, white_bishop];

        let black_knight = BB::coord_to_bb((1, 7)) | BB::coord_to_bb((6,7));
        let white_knight = BB::coord_to_bb((1, 0)) | BB::coord_to_bb((6,0));
        let knight = [black_knight, white_knight];

        let black_pawn = {
            let mut bp = 0;
            for x in 0..8 {
                bp |= BB::coord_to_bb((x, 6));
            }
            bp
        };
        let white_pawn = {
            let mut wp = 0;
            for x in 0..8 {
                wp |= BB::coord_to_bb((x, 1));
            }
            wp
        };
        let pawn = [black_pawn, white_pawn];

        let black_composite = black_king | black_queen | black_rook | black_bishop | black_knight | black_pawn;
        let white_composite = white_king | white_queen | white_rook | white_bishop | white_knight | white_pawn;
        let composite = [black_composite, white_composite];

        let rook_magic_table = BB::gen_rook_magic_table();
        let bishop_magic_table = BB::gen_bishop_magic_table();

        let mut bb = BB{
            white_turn: true,

            king: king,
            queen: queen,
            rook: rook,
            bishop: bishop,
            knight: knight,
            pawn: pawn,
            composite: composite,

            knight_mask: knight_mask,
            rook_mask: rook_mask,
            bishop_mask: bishop_mask,
            king_mask: king_mask,

            castling_rights: 0xFFFFFFFFFFFFFFFF,
            castled: [false, false],
            rook_magic_table: rook_magic_table,
            bishop_magic_table: bishop_magic_table,

            ep: -1,

            cr_stack: Vec::new(),
            cap_stack: Vec::new(),
            ep_stack: Vec::new(),
            history: Vec::new(),
            pawn_history: Vec::new(),

            material: 0,
            hash: 0,
            pawn_hash: 0,
            zobrist_table: zobrist_table,
            phase: 0,

            eval_params: eval_params,
            king_mg_pt_score: 0,
            king_eg_pt_score: 0,
            pawn_pt_score: 0,
            knight_pt_score: 0,
            bishop_pt_score: 0,

            nodes_evaluated: 0,
            tt_hits: 0
        };
        bb.hash = bb.get_full_hash();
        bb.pawn_hash = bb.get_full_pawn_hash();
        bb.king_mg_pt_score = bb.get_king_mg_pt_bonus();
        bb.king_eg_pt_score = bb.get_king_eg_pt_bonus();
        bb.pawn_pt_score = bb.get_pawn_pt_bonus();
        bb.knight_pt_score = bb.get_knight_pt_bonus();
        bb.bishop_pt_score = bb.get_bishop_pt_bonus();

        return bb;
    }

    pub fn from_position(
        fen: String,
        knight_mask: [u64; 64],
        rook_mask: [u64; 64],
        bishop_mask: [u64; 64],
        king_mask: [u64; 64],
        zobrist_table: ([[u64; 12]; 64], (u64, u64)),
        eval_params: EvalParams
    ) -> BB {
        let mut black_king: u64 = 0;
        let mut white_king: u64 = 0;

        let mut black_queen: u64 = 0;
        let mut white_queen: u64 = 0;

        let mut black_rook: u64 = 0;
        let mut white_rook: u64 = 0;

        let mut black_bishop: u64 = 0;
        let mut white_bishop: u64 = 0;

        let mut black_knight: u64 = 0;
        let mut white_knight: u64 = 0;

        let mut black_pawn: u64 = 0;
        let mut white_pawn: u64 = 0;

        let mut rank: i32 = 7;
        let mut file: i32 = 0;

        let mut fen_split = fen.split(' ');
        let positions = match fen_split.next() {
            Some(s) => String::from(s),
            None => panic!("bad FEN string")
        };

        for c in positions.as_bytes().iter() {
            let c = *c;
            match c {
                b'k' => {black_king |= BB::coord_to_bb((file, rank)); file += 1;},
                b'K' => {white_king |= BB::coord_to_bb((file, rank)); file += 1;},

                b'q' => {black_queen |= BB::coord_to_bb((file, rank)); file += 1;},
                b'Q' => {white_queen |= BB::coord_to_bb((file, rank)); file += 1;},

                b'r' => {black_rook |= BB::coord_to_bb((file, rank)); file += 1;},
                b'R' => {white_rook |= BB::coord_to_bb((file, rank)); file += 1;},

                b'b' => {black_bishop |= BB::coord_to_bb((file, rank)); file += 1;},
                b'B' => {white_bishop |= BB::coord_to_bb((file, rank)); file += 1;},

                b'n' => {black_knight |= BB::coord_to_bb((file, rank)); file += 1;},
                b'N' => {white_knight |= BB::coord_to_bb((file, rank)); file += 1;},

                b'p' => {black_pawn |= BB::coord_to_bb((file, rank)); file += 1;},
                b'P' => {white_pawn |= BB::coord_to_bb((file, rank)); file += 1;},

                b'/' => {rank -= 1; file = 0;},
                b'1' => {file += 1},
                b'2' => {file += 2},
                b'3' => {file += 3},
                b'4' => {file += 4},
                b'5' => {file += 5},
                b'6' => {file += 6},
                b'7' => {file += 7},
                b'8' => {file += 8},
                _ => {}
            };
        }

        let mut white_turn = true;

        match fen_split.next() {
            Some(s) => {if s == "b" {white_turn = false;}},
            None => panic!("bad FEN string")
        };

        let cr_str = match fen_split.next() {
            Some(s) => s,
            None => panic!("bad FEN string")
        };

        let mut cr: u64 = 0;
        let wk_mask = BB::coord_to_bb((4, 0));
        let bk_mask = BB::coord_to_bb((4, 7));
        let wrk_mask = BB::coord_to_bb((7, 0));
        let brk_mask = BB::coord_to_bb((7, 7));
        let wrq_mask = BB::coord_to_bb((0, 0));
        let brq_mask = BB::coord_to_bb((0, 7));

        if cr_str != "-" {
            for c in String::from(cr_str).as_bytes().iter() {
                let c = *c;
                match c {
                    b'k' => {cr |= bk_mask | brk_mask;},
                    b'K' => {cr |= wk_mask | wrk_mask;},
                    b'q' => {cr |= bk_mask | brq_mask;},
                    b'Q' => {cr |= wk_mask | wrq_mask;},
                    _ => panic!("bad CR")
                };
            }
        }

        let ep_str = match fen_split.next() {
            Some(s) => s,
            None => panic!("bad fen str")
        };

        let mut ep: i32 = -1;
        if ep_str != "-" {
            let ep_str = String::from(ep_str);
            let ep_chars = ep_str.as_bytes();
            let file = ep_chars[0] - b'a';
            let rank = ep_chars[1] - b'1';
            ep = BB::coord_to_idx((file as i32, rank as i32));
        }

        let king = [black_king, white_king];
        let queen = [black_queen, white_queen];
        let rook = [black_rook, white_rook];
        let bishop = [black_bishop, white_bishop];
        let knight = [black_knight, white_knight];
        let pawn = [black_pawn, white_pawn];

        let black_composite = black_king | black_queen | black_rook | black_bishop | black_knight | black_pawn;
        let white_composite = white_king | white_queen | white_rook | white_bishop | white_knight | white_pawn;
        let composite = [black_composite, white_composite];

        let rook_magic_table = BB::gen_rook_magic_table();
        let bishop_magic_table = BB::gen_bishop_magic_table();

        let mut bb = BB{
            white_turn: white_turn,

            king: king,
            queen: queen,
            rook: rook,
            bishop: bishop,
            knight: knight,
            pawn: pawn,
            composite: composite,

            knight_mask: knight_mask,
            rook_mask: rook_mask,
            bishop_mask: bishop_mask,
            king_mask: king_mask,

            castling_rights: cr,
            castled: [false, false],
            rook_magic_table: rook_magic_table,
            bishop_magic_table: bishop_magic_table,

            ep: ep,

            cr_stack: Vec::new(),
            cap_stack: Vec::new(),
            ep_stack: Vec::new(),
            history: Vec::new(),
            pawn_history: Vec::new(),

            material: 0,
            hash: 0,
            pawn_hash: 0,
            zobrist_table: zobrist_table,
            eval_params: eval_params,
            phase: 0,

            king_mg_pt_score: 0,
            king_eg_pt_score: 0,
            pawn_pt_score: 0,
            knight_pt_score: 0,
            bishop_pt_score: 0,

            nodes_evaluated: 0,
            tt_hits: 0
        };

        bb.material = bb.get_material();
        bb.hash = bb.get_full_hash();
        bb.pawn_hash = bb.get_full_pawn_hash();
        bb.phase = bb.slow_get_phase();
        bb.king_mg_pt_score = bb.get_king_mg_pt_bonus();
        bb.king_eg_pt_score = bb.get_king_eg_pt_bonus();
        bb.pawn_pt_score = bb.get_pawn_pt_bonus();
        bb.knight_pt_score = bb.get_knight_pt_bonus();
        bb.bishop_pt_score = bb.get_bishop_pt_bonus();
        return bb;
    }

    pub fn reset(& mut self) {
        let black_king = BB::coord_to_bb((4, 7));
        let white_king = BB::coord_to_bb((4, 0));
        self.king = [black_king, white_king];

        let black_queen = BB::coord_to_bb((3, 7));
        let white_queen = BB::coord_to_bb((3, 0));
        self.queen = [black_queen, white_queen];

        let black_rook = BB::coord_to_bb((0, 7)) | BB::coord_to_bb((7,7));
        let white_rook = BB::coord_to_bb((0, 0)) | BB::coord_to_bb((7,0));
        self.rook = [black_rook, white_rook];

        let black_bishop = BB::coord_to_bb((2, 7)) | BB::coord_to_bb((5,7));
        let white_bishop = BB::coord_to_bb((2, 0)) | BB::coord_to_bb((5,0));
        self.bishop = [black_bishop, white_bishop];

        let black_knight = BB::coord_to_bb((1, 7)) | BB::coord_to_bb((6,7));
        let white_knight = BB::coord_to_bb((1, 0)) | BB::coord_to_bb((6,0));
        self.knight = [black_knight, white_knight];

        let black_pawn = {
            let mut bp = 0;
            for x in 0..8 {
                bp |= BB::coord_to_bb((x, 6));
            }
            bp
        };
        let white_pawn = {
            let mut wp = 0;
            for x in 0..8 {
                wp |= BB::coord_to_bb((x, 1));
            }
            wp
        };
        self.pawn = [black_pawn, white_pawn];

        let black_composite = black_king | black_queen | black_rook | black_bishop | black_knight | black_pawn;
        let white_composite = white_king | white_queen | white_rook | white_bishop | white_knight | white_pawn;
        self.composite = [black_composite, white_composite];

        self.white_turn = true;
        self.castling_rights = 0xFFFFFFFFFFFFFFFF;
        self.castled = [false, false];
        self.ep = -1;

        // stacks
        self.ep_stack = Vec::new();
        self.cap_stack = Vec::new();
        self.cr_stack = Vec::new();
        self.history = Vec::new();
        self.pawn_history = Vec::new();

        // eval
        self.material = 0;
        self.hash = self.get_full_hash();
        self.pawn_hash = self.get_full_pawn_hash();

        self.phase = 0;
        self.king_mg_pt_score = self.get_king_mg_pt_bonus();
        self.king_eg_pt_score = self.get_king_eg_pt_bonus();
        self.pawn_pt_score = self.get_pawn_pt_bonus();
        self.knight_pt_score = self.get_knight_pt_bonus();
        self.bishop_pt_score = self.get_bishop_pt_bonus();

        self.nodes_evaluated = 0;
        self.tt_hits = 0;
    }

    pub fn reset_from_position(
        & mut self,
        fen: String,
    ) {
        let mut black_king: u64 = 0;
        let mut white_king: u64 = 0;

        let mut black_queen: u64 = 0;
        let mut white_queen: u64 = 0;

        let mut black_rook: u64 = 0;
        let mut white_rook: u64 = 0;

        let mut black_bishop: u64 = 0;
        let mut white_bishop: u64 = 0;

        let mut black_knight: u64 = 0;
        let mut white_knight: u64 = 0;

        let mut black_pawn: u64 = 0;
        let mut white_pawn: u64 = 0;

        let mut rank: i32 = 7;
        let mut file: i32 = 0;

        let mut fen_split = fen.split(' ');
        let positions = match fen_split.next() {
            Some(s) => String::from(s),
            None => panic!("bad FEN string")
        };

        for c in positions.as_bytes().iter() {
            let c = *c;
            match c {
                b'k' => {black_king |= BB::coord_to_bb((file, rank)); file += 1;},
                b'K' => {white_king |= BB::coord_to_bb((file, rank)); file += 1;},

                b'q' => {black_queen |= BB::coord_to_bb((file, rank)); file += 1;},
                b'Q' => {white_queen |= BB::coord_to_bb((file, rank)); file += 1;},

                b'r' => {black_rook |= BB::coord_to_bb((file, rank)); file += 1;},
                b'R' => {white_rook |= BB::coord_to_bb((file, rank)); file += 1;},

                b'b' => {black_bishop |= BB::coord_to_bb((file, rank)); file += 1;},
                b'B' => {white_bishop |= BB::coord_to_bb((file, rank)); file += 1;},

                b'n' => {black_knight |= BB::coord_to_bb((file, rank)); file += 1;},
                b'N' => {white_knight |= BB::coord_to_bb((file, rank)); file += 1;},

                b'p' => {black_pawn |= BB::coord_to_bb((file, rank)); file += 1;},
                b'P' => {white_pawn |= BB::coord_to_bb((file, rank)); file += 1;},

                b'/' => {rank -= 1; file = 0;},
                b'1' => {file += 1},
                b'2' => {file += 2},
                b'3' => {file += 3},
                b'4' => {file += 4},
                b'5' => {file += 5},
                b'6' => {file += 6},
                b'7' => {file += 7},
                b'8' => {file += 8},
                _ => {}
            };
        }

        let mut white_turn = true;

        match fen_split.next() {
            Some(s) => {if s == "b" {white_turn = false;}},
            None => panic!("bad FEN string")
        };

        let cr_str = match fen_split.next() {
            Some(s) => s,
            None => panic!("bad FEN string")
        };

        let mut cr: u64 = 0;
        let wk_mask = BB::coord_to_bb((4, 0));
        let bk_mask = BB::coord_to_bb((4, 7));
        let wrk_mask = BB::coord_to_bb((7, 0));
        let brk_mask = BB::coord_to_bb((7, 7));
        let wrq_mask = BB::coord_to_bb((0, 0));
        let brq_mask = BB::coord_to_bb((0, 7));

        if cr_str != "-" {
            for c in String::from(cr_str).as_bytes().iter() {
                let c = *c;
                match c {
                    b'k' => {cr |= bk_mask | brk_mask;},
                    b'K' => {cr |= wk_mask | wrk_mask;},
                    b'q' => {cr |= bk_mask | brq_mask;},
                    b'Q' => {cr |= wk_mask | wrq_mask;},
                    _ => panic!("bad CR")
                };
            }
        }

        let ep_str = match fen_split.next() {
            Some(s) => s,
            None => panic!("bad fen str")
        };

        let mut ep: i32 = -1;
        if ep_str != "-" {
            let ep_str = String::from(ep_str);
            let ep_chars = ep_str.as_bytes();
            let file = ep_chars[0] - b'a';
            let rank = ep_chars[1] - b'1';
            ep = BB::coord_to_idx((file as i32, rank as i32));
        }

        let king = [black_king, white_king];
        let queen = [black_queen, white_queen];
        let rook = [black_rook, white_rook];
        let bishop = [black_bishop, white_bishop];
        let knight = [black_knight, white_knight];
        let pawn = [black_pawn, white_pawn];

        let black_composite = black_king | black_queen | black_rook | black_bishop | black_knight | black_pawn;
        let white_composite = white_king | white_queen | white_rook | white_bishop | white_knight | white_pawn;
        let composite = [black_composite, white_composite];

        self.white_turn = white_turn;
        self.king = king;
        self.queen = queen;
        self.rook = rook;
        self.bishop = bishop;
        self.knight = knight;
        self.pawn = pawn;
        self.composite = composite;

        self.castling_rights = cr;
        self.castled = [false, false];

        self.ep = ep;

        // stacks
        self.ep_stack = Vec::new();
        self.cap_stack = Vec::new();
        self.cr_stack = Vec::new();
        self.history = Vec::new();
        self.pawn_history = Vec::new();

        self.material = self.get_material();
        self.hash = self.get_full_hash();
        self.pawn_hash = self.get_full_pawn_hash();

        self.phase = self.slow_get_phase();

        self.king_mg_pt_score = self.get_king_mg_pt_bonus();
        self.king_eg_pt_score = self.get_king_eg_pt_bonus();
        self.pawn_pt_score = self.get_pawn_pt_bonus();
        self.knight_pt_score = self.get_knight_pt_bonus();
        self.bishop_pt_score = self.get_bishop_pt_bonus();

        self.nodes_evaluated = 0;
        self.tt_hits = 0;
    }

    // it's fine for the gen map fns to be
    // suboptimal in performance time since
    // we only do it once
    pub fn gen_knight_mask() -> [u64; 64] {
        let mut knight_mask: [u64; 64] = [0; 64];
        for idx in 0..64 {
            let mut bb: u64 = 0;
            let (kx, ky) = BB::idx_to_coord(idx);
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
                let (nx, ny) = (kx + dx, ky + dy);
                if (nx >= 8 || nx < 0) || (ny >= 8 || ny < 0) { continue; }
                let new_idx = BB::coord_to_idx((nx, ny));
                bb |= 1 << new_idx;
            }
            knight_mask[idx as usize] = bb;
        }
        return knight_mask;
    }

    // it's fine for the gen map fns to be
    // suboptimal in performance time since
    // we only do it once
    pub fn gen_rook_mask() -> [u64; 64] {
        let mut rook_mask: [u64; 64] = [0; 64];
        for idx in 0..64 {
            let mut bb: u64 = 0;
            let (rx, ry) = BB::idx_to_coord(idx);
            for (sx, sy) in [(0, 1), (0, -1), (-1, 0), (1, 0)].iter() {
                let mut d = 1;
                while d < 8 {
                    let (nx, ny) = (rx + (sx * d), ry + (sy * d));
                    if ((nx < 1 || nx >= 7) && (nx != rx)) || ((ny < 1 || ny >= 7) && (ny != ry)) {
                        break;
                    }
                    let new_idx = BB::coord_to_idx((nx, ny));
                    bb |= 1 << new_idx;
                    d += 1;
                }
            }

            rook_mask[idx as usize] = bb;
        }
        return rook_mask;
    }

    // it's fine for the gen map fns to be
    // suboptimal in performance time since
    // we only do it once
    pub fn gen_bishop_mask() -> [u64; 64] {
        let mut bishop_mask: [u64; 64] = [0; 64];
        for idx in 0..64 {
            let mut bb: u64 = 0;
            let (bx, by) = BB::idx_to_coord(idx);
            for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)].iter() {
                let mut d = 1;
                while d < 8 {
                    let (nx, ny) = (bx + (sx * d), by + (sy * d));
                    if ((nx < 1 || nx >= 7) && (nx != bx)) || ((ny < 1 || ny >= 7) && (ny != by)) {
                        break;
                    }
                    let new_idx = BB::coord_to_idx((nx, ny));
                    bb |= 1 << new_idx;
                    d += 1;
                }
            }
            bishop_mask[idx as usize] = bb;
        }
        return bishop_mask;
    }

    pub fn gen_king_mask() -> [u64; 64] {
        let mut king_mask: [u64; 64] = [0; 64];
        for idx in 0..64 {
            let mut bb: u64 = 0;
            let (kx, ky) = BB::idx_to_coord(idx);
            for (dx, dy) in [
                (-1, 1), (1, 1), (-1, -1), (1, -1),
                (0, 1), (0, -1), (-1, 0), (1, 0)
            ].iter() {
                let (nx, ny) = (kx + dx, ky + dy);
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    continue;
                }
                let new_idx = BB::coord_to_idx((nx, ny));
                bb |= 1 << new_idx;
            }
            king_mask[idx as usize] = bb;
        }
        return king_mask;
    }

    pub fn gen_rook_magic_table() -> Vec<Vec<u64>> { //[[u64; 4096]; 64] {
        let mut rook_magic: Vec<Vec<u64>> = vec![vec![0; 4096]; 64];// [[u64; 4096]; 64] = [[0; 4096]; 64];
        let rook_mask = BB::gen_rook_mask();
        for idx in 0..64 {
            let (rx, ry) = BB::idx_to_coord(idx);
            for i in 0..256 {
                for j in 0..256 {
                    let mut ob = 0;
                    // i should take up the rank, j the file
                    let mut i_file = 0;
                    let mut j_rank = 0;
                    let mut i_num: u64 = i;
                    let mut j_num: u64 = j;
                    while i_num != 0 {
                        let v = i_num % 2;
                        i_num = i_num >> 1;
                        if v == 1 {
                            ob |= BB::coord_to_bb((i_file, ry));
                        }
                        i_file += 1;
                    }

                    while j_num != 0 {
                        let v = j_num % 2;
                        j_num = j_num >> 1;
                        if v == 1 {
                            ob |= BB::coord_to_bb((rx, j_rank));
                        }
                        j_rank += 1;
                    }

                    // get rid of the rook and mask
                    ob &= rook_mask[BB::coord_to_idx((rx, ry)) as usize];

                    let hash = BB::rook_magic_hash(ob, idx as usize);
                    let move_map = BB::get_rook_moves_from_occ((rx, ry), ob);
                    rook_magic[idx as usize][hash as usize] = move_map;
                }
            }
        }
        return rook_magic;
    }

    fn get_rook_moves_from_occ(coord: (i32, i32), ob: u64) -> u64 {
        let (rx, ry) = coord;
        let mut ob = ob;
        let mut bb = 0;
        for (sx, sy) in [(0, 1), (0, -1), (-1, 0), (1, 0)].iter() {
            let mut d = 1;
            while d < 8 {
                let (nx, ny) = (rx + (sx * d), ry + (sy * d));
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    break;
                }
                let new_idx = BB::coord_to_idx((nx, ny));
                let old_ob = ob;
                bb |= 1 << new_idx;
                ob |= 1 << new_idx;
                if old_ob == ob {
                    // reached a piece
                    break;
                }
                d += 1;
            }
        }
        return bb;
    }

    pub fn gen_bishop_magic_table() -> Vec<Vec<u64>> {// [[u64; 512]; 64] {
        let mut bishop_magic: Vec<Vec<u64>> = vec![vec![0; 512]; 64];// [[u64; 512]; 64] = [[0; 512]; 64];
        let bishop_mask = BB::gen_bishop_mask();
        for idx in 0..64 {
            let (bx, by) = BB::idx_to_coord(idx);
            for i in 0..256 {
                for j in 0..256 {
                    let mut ob = 0;
                    // i should take up the rank, j the file
                    let start_coords = BB::get_bishop_start_coords((bx, by));
                    let mut i_coord = start_coords.0; // bottom left
                    let mut j_coord = start_coords.1; // bottom right
                    let mut i_num: u64 = i;
                    let mut j_num: u64 = j;
                    while i_num != 0 && i_coord.0 < 8 && i_coord.1 < 8 {
                        let v = i_num % 2;
                        i_num = i_num >> 1;
                        if v == 1 {
                            ob |= BB::coord_to_bb(i_coord);
                        }
                        i_coord = (i_coord.0 + 1, i_coord.1 + 1)
                    }

                    while j_num != 0 && j_coord.0 >= 0 && j_coord.1 < 8 {
                        let v = j_num % 2;
                        j_num = j_num >> 1;
                        if v == 1 {
                            ob |= BB::coord_to_bb(j_coord);
                        }
                        j_coord = (j_coord.0 - 1, j_coord.1 + 1)
                    }

                    // get rid of the bishop and mask
                    ob &= bishop_mask[BB::coord_to_idx((bx, by)) as usize];

                    let hash = BB::bishop_magic_hash(ob, idx as usize);
                    let move_map = BB::get_bishop_moves_from_occ((bx, by), ob);
                    bishop_magic[idx as usize][hash as usize] = move_map;
                }

            }
        }
        return bishop_magic;
    }

    fn get_bishop_start_coords(coord: (i32, i32)) -> ((i32, i32), (i32, i32)) {
        // go from the coordinate of a bishop to the lowest point in the board
        // along its diagonals
        let (x, y) = coord;
        let mut lx = x;
        let mut ly = y;

        while lx - 1 >= 0 && ly - 1 >= 0 {
            lx -= 1;
            ly -= 1;
        }

        let mut rx = x;
        let mut ry = y;
        while rx + 1 < 8 && ry - 1 >= 0 {
            rx += 1;
            ry -= 1;
        }

        return ((lx, ly), (rx, ry));
    }

    fn get_bishop_moves_from_occ(coord: (i32, i32), ob: u64) -> u64 {
        let (bx, by) = coord;
        let mut ob = ob;
        let mut bb = 0;
        for (sx, sy) in [(-1, 1), (1, 1), (-1, -1), (1, -1)].iter() {
            let mut d = 1;
            while d < 8 {
                let (nx, ny) = (bx + (sx * d), by + (sy * d));
                if (nx < 0 || nx >= 8) || (ny < 0 || ny >= 8) {
                    break;
                }
                let new_idx = BB::coord_to_idx((nx, ny));
                let old_ob = ob;
                bb |= 1 << new_idx;
                ob |= 1 << new_idx;
                if old_ob == ob {
                    // reached a piece
                    break;
                }
                d += 1;
            }
        }
        return bb;
    }

    pub fn rook_magic_hash(masked_composite: u64, square: usize) -> u64 {
        return (masked_composite * ROOK_MAGIC_NUMBERS[square]) >> ROOK_MAGIC_SHIFTS[square];
    }

    pub fn bishop_magic_hash(masked_composite: u64, square: usize) -> u64 {
        return (masked_composite * BISHOP_MAGIC_NUMBERS[square]) >> BISHOP_MAGIC_SHIFTS[square];
    }

    pub fn bb_str(bb: u64) -> String {
        let mut s = String::new();
        let mut b = bb;
        for _ in 0..8 {
            let rank = b & ((1 << 8) - 1);
            b = b >> 8;
            s.push_str(format!("{:08b}\n", rank).as_str());
        }
        return s.chars().rev().collect();
    }


    // these functions are useful for resetting and debugging
    // but their values are usually handled incrementally for speed
    pub fn get_material(&self) -> i32 {
        let mut mat = 0;
        mat += (self.king[1].count_ones() as i32 - self.king[0].count_ones() as i32) * KING_VALUE;
        mat += (self.queen[1].count_ones() as i32 - self.queen[0].count_ones() as i32) * QUEEN_VALUE;
        mat += (self.rook[1].count_ones() as i32 - self.rook[0].count_ones() as i32) * ROOK_VALUE;
        mat += (self.bishop[1].count_ones() as i32 - self.bishop[0].count_ones() as i32) * BISHOP_VALUE;
        mat += (self.knight[1].count_ones() as i32 - self.knight[0].count_ones() as i32) * KNIGHT_VALUE;
        mat += (self.pawn[1].count_ones() as i32 - self.pawn[0].count_ones() as i32) * PAWN_VALUE;

        return mat;
    }

    pub fn slow_get_phase(&self) -> i32 {
        let mut phase = 240;
        phase -= (self.queen[1].count_ones() as i32) * board_eval::QUEEN_PHASE;
        phase -= (self.queen[0].count_ones() as i32) * board_eval::QUEEN_PHASE;
        phase -= (self.rook[1].count_ones() as i32) * board_eval::ROOK_PHASE;
        phase -= (self.rook[0].count_ones() as i32) * board_eval::ROOK_PHASE;
        phase -= (self.bishop[1].count_ones() as i32) * board_eval::BISHOP_PHASE;
        phase -= (self.bishop[0].count_ones() as i32) * board_eval::BISHOP_PHASE;
        phase -= (self.knight[1].count_ones() as i32) * board_eval::KNIGHT_PHASE;
        phase -= (self.knight[0].count_ones() as i32) * board_eval::KNIGHT_PHASE;

        if phase < 0 {
            phase = 0;
        }

        return phase;
    }

    pub fn init_zobrist() -> ([[u64; 12]; 64], (u64, u64)) {
        let mut rng = rand::thread_rng();
        let mut zobrist_table : [[u64; 12]; 64] = [[0; 12]; 64];
        for i in 0..64 {
            for j in 0..12 {
                zobrist_table[i][j] = rng.gen();
            }
        }
        let whose_turn: (u64, u64) = (rng.gen(), rng.gen());
        return (zobrist_table, whose_turn);
    }

    pub fn get_phase(&self) -> i32 {
        if self.phase < 0 {
            return 0;
        } else if self.phase > 256 {
            return 256;
        }
        return self.phase;
    }

    fn get_bb_hash(&self, bb: u64, piece: u8, white: bool) -> u64 {
        let mut h: u64 = 0;
        let mut c = 0;
        let mut idx: i32 = -1;
        let mut bb = bb;

        while bb != 0 && c < 64 {
            c = bb.trailing_zeros() as i32 + 1;
            idx += c;
            bb = bb >> c;

            let i = idx as usize;

            let to_xor = match (piece, white) {
                (b'k', true) => self.zobrist_table.0[i][0],
                (b'q', true) => self.zobrist_table.0[i][1],
                (b'b', true) => self.zobrist_table.0[i][2],
                (b'n', true) => self.zobrist_table.0[i][3],
                (b'r', true) => self.zobrist_table.0[i][4],
                (b'p', true) => self.zobrist_table.0[i][5],
                (b'k', false) => self.zobrist_table.0[i][6],
                (b'q', false) => self.zobrist_table.0[i][7],
                (b'b', false) => self.zobrist_table.0[i][8],
                (b'n', false) => self.zobrist_table.0[i][9],
                (b'r', false) => self.zobrist_table.0[i][10],
                (b'p', false) => self.zobrist_table.0[i][11],
                _ => 0
            };
            h ^= to_xor;
        }
        return h
    }

    pub fn get_full_hash(&self) -> u64 {
        let mut h: u64 = 0;
        for color in 0..2 {
            let side = color as usize;
            let white = color != 0;
            h ^= self.get_bb_hash(self.king[side], b'k', white);
            h ^= self.get_bb_hash(self.queen[side], b'q', white);
            h ^= self.get_bb_hash(self.rook[side], b'r', white);
            h ^= self.get_bb_hash(self.bishop[side], b'b', white);
            h ^= self.get_bb_hash(self.knight[side], b'n', white);
            h ^= self.get_bb_hash(self.pawn[side], b'p', white);
        }
        h ^= if self.white_turn {self.zobrist_table.1.0} else {self.zobrist_table.1.1};
        return h;
    }

    pub fn get_full_pawn_hash(&self) -> u64 {
        let mut h: u64 = 0;
        for color in 0..2 {
            let side = color as usize;
            let white = color != 0;
            h ^= self.get_bb_hash(self.pawn[side], b'p', white);
        }
        return h;
    }

    // get a single xor value from the zobrist
    fn get_zr_xor(&self, c : usize, p : u8, w: bool) -> u64 {
        match (p, w) {
            (b'k', true) => self.zobrist_table.0[c][0],
            (b'q', true) => self.zobrist_table.0[c][1],
            (b'b', true) => self.zobrist_table.0[c][2],
            (b'n', true) => self.zobrist_table.0[c][3],
            (b'r', true) => self.zobrist_table.0[c][4],
            (b'p', true) => self.zobrist_table.0[c][5],
            (b'k', false) => self.zobrist_table.0[c][6],
            (b'q', false) => self.zobrist_table.0[c][7],
            (b'b', false) => self.zobrist_table.0[c][8],
            (b'n', false) => self.zobrist_table.0[c][9],
            (b'r', false) => self.zobrist_table.0[c][10],
            (b'p', false) => self.zobrist_table.0[c][11],
            _ => 0
        }
    }

    fn get_pawn_table_val(&self, idx: i32, white: bool) -> i32 {
        let entry_value = (self.eval_params.pawn_pt_scale * board_eval::PAWN_TABLE[if white {idx} else {63-idx} as usize]) / 100;
        return entry_value + self.eval_params.pawn_pt_offset;
    }

    fn get_knight_table_val(&self, idx: i32, white: bool) -> i32 {
        let entry_value = (self.eval_params.knight_pt_scale * board_eval::KNIGHT_TABLE[if white {idx} else {63-idx} as usize]) / 100;
        return entry_value + self.eval_params.knight_pt_offset;
    }

    fn get_bishop_table_val(&self, idx: i32, white: bool) -> i32 {
        let entry_value = (self.eval_params.bishop_pt_scale * board_eval::BISHOP_TABLE[if white {idx} else {63-idx} as usize]) / 100;
        return entry_value + self.eval_params.bishop_pt_offset;
    }

    fn get_king_mg_table_val(&self, idx: i32, white: bool) -> i32 {
        let entry_value = (self.eval_params.king_mg_pt_scale * board_eval::KING_MG_TABLE[if white {idx} else {63-idx} as usize]) / 100;
        return entry_value + self.eval_params.king_mg_pt_offset;
    }

    fn get_king_eg_table_val(&self, idx: i32, white: bool) -> i32 {
        let entry_value = (self.eval_params.king_eg_pt_scale * board_eval::KING_EG_TABLE[if white {idx} else {63-idx} as usize]) / 100;
        return entry_value + self.eval_params.king_eg_pt_offset;
    }

    fn update_pt_score(&mut self, piece: u8, white: bool, start: i32, end: i32, capture: u8, promotion: u8) {
        let side_factor = if white {1} else {-1};
        match piece {
            b'p' => {
                // remove score from before
                self.pawn_pt_score -= side_factor * self.get_pawn_table_val(start, white);
                match promotion {
                    b'n' => {self.knight_pt_score += side_factor * self.get_knight_table_val(end, white);},
                    b'b' => {self.bishop_pt_score += side_factor * self.get_bishop_table_val(end, white);},
                    0 => {self.pawn_pt_score += side_factor * self.get_pawn_table_val(end, white)},
                    _ => {}
                }
            },
            b'n' => {
                self.knight_pt_score -= side_factor * self.get_knight_table_val(start, white);
                self.knight_pt_score += side_factor * self.get_knight_table_val(end, white);
            },
            b'b' => {
                self.bishop_pt_score -= side_factor * self.get_bishop_table_val(start, white);
                self.bishop_pt_score += side_factor * self.get_bishop_table_val(end, white);
            },
            b'k' => {
                self.king_mg_pt_score -= side_factor * self.get_king_mg_table_val(start, white);
                self.king_mg_pt_score += side_factor * self.get_king_mg_table_val(end, white);
                self.king_eg_pt_score -= side_factor * self.get_king_eg_table_val(start, white);
                self.king_eg_pt_score += side_factor * self.get_king_eg_table_val(end, white);
            },
            _ => {}
        };

        match capture {
            b'p' => {
                self.pawn_pt_score += side_factor * self.get_pawn_table_val(end, !white);
            },
            b'n' => {
                self.knight_pt_score += side_factor * self.get_knight_table_val(end, !white);
            },
            b'b' => {
                self.bishop_pt_score += side_factor * self.get_bishop_table_val(end, !white);
            },
            _ => {}
        };
    }

    fn deupdate_pt_score(&mut self, piece: u8, white: bool, start: i32, end: i32, capture: u8, promotion: u8) {
        let side_factor = if white {-1} else {1};
        match piece {
            b'p' => {
                // remove score from before
                self.pawn_pt_score -= side_factor * self.get_pawn_table_val(start, white);
                match promotion {
                    b'n' => {self.knight_pt_score += side_factor * self.get_knight_table_val(end, white);},
                    b'b' => {self.bishop_pt_score += side_factor * self.get_bishop_table_val(end, white);},
                    0 => {self.pawn_pt_score += side_factor * self.get_pawn_table_val(end, white)},
                    _ => {}
                }
            },
            b'n' => {
                self.knight_pt_score -= side_factor * self.get_knight_table_val(start, white);
                self.knight_pt_score += side_factor * self.get_knight_table_val(end, white);
            },
            b'b' => {
                self.bishop_pt_score -= side_factor * self.get_bishop_table_val(start, white);
                self.bishop_pt_score += side_factor * self.get_bishop_table_val(end, white);
            },
            b'k' => {
                self.king_mg_pt_score -= side_factor * self.get_king_mg_table_val(start, white);
                self.king_mg_pt_score += side_factor * self.get_king_mg_table_val(end, white);
                self.king_eg_pt_score -= side_factor * self.get_king_eg_table_val(start, white);
                self.king_eg_pt_score += side_factor * self.get_king_eg_table_val(end, white);
            },
            _ => {}
        };

        match capture {
            b'p' => {
                self.pawn_pt_score += side_factor * self.get_pawn_table_val(end, !white);
            },
            b'n' => {
                self.knight_pt_score += side_factor * self.get_knight_table_val(end, !white);
            },
            b'b' => {
                self.bishop_pt_score += side_factor * self.get_bishop_table_val(end, !white);
            },
            _ => {}
        };
    }

    fn update_hash(&mut self, piece: u8, white: bool, start: i32, end: i32, capture: u8, promotion: u8) {
        let start_h = self.get_zr_xor(start as usize, piece, white);
        self.hash ^= start_h;
        let end_h = self.get_zr_xor(end as usize, if promotion != 0 {promotion} else {piece}, white);
        self.hash ^= end_h;
        if capture != 0 {
            let capture_h = self.get_zr_xor(end as usize, capture, !white);
            self.hash ^= capture_h;
            if capture == b'p' {
                self.pawn_hash ^= capture_h;
            }
        }

        if piece == b'p' {
            // update the pawn hash
            self.pawn_hash ^= start_h;
            if promotion == 0 {
                self.pawn_hash ^= end_h;
            }
        }
    }

    pub fn is_check(&self, white: bool) -> bool {
        let king_idx = self.king[white as usize].trailing_zeros() as i32;
        return self.is_idx_attacked(king_idx, white);
    }

    fn is_idx_attacked(&self, idx: i32, white: bool) -> bool {
        // process:
        // create "virtual pieces" at the idx and see
        // if they can attack enemy pieces of the appropriate
        // types.  If so, there is an attack on this square
        //
        // should provide an alternative function to get all attacks
        // by a side, but this is optimized for checking *just* a single
        // square
        let all_composite = self.composite[0] | self.composite[1];
        let self_bb = BB::idx_to_bb(idx);

        // knights
        let virt_knight_bb = self.knight_mask[idx as usize];
        if (virt_knight_bb & self.knight[!white as usize]) != 0 {
            // a knight is attacking
            return true;
        }

        // king
        // normally this is irrelevant for check since a king can't legally
        // end up there but this may be relevant in other cases, e.g. castling
        let virt_king_bb = self.king_mask[idx as usize];
        if (virt_king_bb & self.king[!white as usize]) != 0 {
            // a king is attacking
            return true;
        }

        // pawns
        let virt_pawn_bb = if white {
            // move up
            ((self_bb & !FILE_MASKS[0]) << 7) | ((self_bb & !FILE_MASKS[7]) << 9)
        } else {
            // move down
            ((self_bb & !FILE_MASKS[0]) >> 9) | ((self_bb & !FILE_MASKS[7]) >> 7)
        };
        if (virt_pawn_bb & self.pawn[!white as usize]) != 0 {
            // a pawn is attacking
            return true;
        }

        // bishops & queens
        let virt_bishop_occ_bb = self.bishop_mask[idx as usize] & all_composite;
        let hash = BB::bishop_magic_hash(virt_bishop_occ_bb, idx as usize);
        let virt_bishop_bb = self.bishop_magic_table[idx as usize][hash as usize];
        if (virt_bishop_bb & (self.bishop[!white as usize] | self.queen[!white as usize])) != 0 {
            // bishop or queen is attacking
            return true;
        }

        // rooks & queens
        let virt_rook_occ_bb = self.rook_mask[idx as usize] & all_composite;
        let hash = BB::rook_magic_hash(virt_rook_occ_bb, idx as usize);
        let virt_rook_bb = self.rook_magic_table[idx as usize][hash as usize];
        if (virt_rook_bb & (self.rook[!white as usize] | self.queen[!white as usize])) != 0 {
            // rook or queen is attacking
            return true;
        }

        return false;
    }

    // condition checks
    fn can_castle(&self, white: bool, queenside: bool) -> bool {
        // check castling rights

        // white vs. black
        let king_idx = if white {4} else {60};
        if (self.castling_rights & BB::idx_to_bb(king_idx)) == 0 {
            // no king castling rights
            return false;
        }

        // kingside vs. queenside
        let rook_idx = if queenside {king_idx - 4} else {king_idx + 3};
        if (self.castling_rights & BB::idx_to_bb(rook_idx)) == 0 {
            // no castling rights at the relevant rook
            return false;
        }

        // check empty space
        let occ_mask = if queenside {
            0b111 << (king_idx - 3)
        } else {
            0b11 << (king_idx + 1)
        };

        if ((self.composite[0] | self.composite[1]) & occ_mask) != 0 {
            // there's something in the way
            return false;
        }

        // check if in check at relevant positions
        let attacked = self.is_idx_attacked(king_idx, white) ||
            if queenside {
                self.is_idx_attacked(king_idx - 1, white)
            } else {
                self.is_idx_attacked(king_idx + 1, white)
            };

        return !attacked;
    }

    // move generation
    pub fn king_moves(&self, white: bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx: i32 = -1;
        let mut king_loc_bb = self.king[white as usize];

        let mut kc = 0;
        while king_loc_bb != 0 && kc < 64 {
            kc = king_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            king_loc_bb = king_loc_bb >> kc;

            // only include moves that don't hit your own guys
            let mut king_bb = self.king_mask[start_idx as usize] & !self.composite[white as usize];
            let mut end_idx: i32 = -1;

            // normal moves
            let mut c = 0;
            while king_bb != 0 && c < 64 {
                c = king_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                king_bb = king_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'k'));
            }

            // kingside-castling
            if self.can_castle(white, false) {
                moves.push(Mv::normal_move(start_idx, start_idx + 2, b'k'));
            }

            // queenside-castling
            if self.can_castle(white, true) {
                moves.push(Mv::normal_move(start_idx, start_idx - 2, b'k'));
            }
        }
        return moves;
    }

    // *_q_moves refers to moves actually checked in
    // quiescence search
    //
    // having a separate move generation for them (despite the repeated code)
    // is noticably quicker
    pub fn king_q_moves(&self, white: bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx: i32 = -1;
        let mut king_loc_bb = self.king[white as usize];

        let mut kc = 0;
        while king_loc_bb != 0 && kc < 64 {
            kc = king_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            king_loc_bb = king_loc_bb >> kc;

            // only include moves that capture
            let mut king_bb = self.king_mask[start_idx as usize] & self.composite[!white as usize];
            let mut end_idx: i32 = -1;

            // normal moves
            let mut c = 0;
            while king_bb != 0 && c < 64 {
                c = king_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                king_bb = king_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'k'));
            }
        }
        return moves;
    }

    pub fn knight_q_moves(&self, white: bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx: i32 = -1;
        let mut knight_loc_bb = self.knight[white as usize];

        let mut kc = 0;
        while knight_loc_bb != 0 && kc < 64 {
            kc = knight_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            knight_loc_bb = knight_loc_bb >> kc;

            // only include moves that capture
            let mut knight_bb = self.knight_mask[start_idx as usize] & self.composite[!white as usize];
            let mut end_idx: i32 = -1;

            let mut c = 0;
            while knight_bb != 0 && c < 64 {
                c = knight_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                knight_bb = knight_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'n'));
            }
        }
        return moves;
    }

    pub fn knight_moves(&self, white: bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx: i32 = -1;
        let mut knight_loc_bb = self.knight[white as usize];

        let mut kc = 0;
        while knight_loc_bb != 0 && kc < 64 {
            kc = knight_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            knight_loc_bb = knight_loc_bb >> kc;

            // only include moves that don't hit your own guys
            let mut knight_bb = self.knight_mask[start_idx as usize] & !self.composite[white as usize];
            let mut end_idx: i32 = -1;

            let mut c = 0;
            while knight_bb != 0 && c < 64 {
                c = knight_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                knight_bb = knight_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'n'));
            }
        }
        return moves;
    }

    pub fn pawn_q_moves(&self, white:bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut pawn_loc_bb = self.pawn[white as usize];
        let mut capture_composite = self.composite[!white as usize];
        if self.ep != -1 {
            // include a "ghost" entry in the capture composite
            // if an ep-able move happened
            capture_composite |= BB::idx_to_bb(self.ep);
        }

        while pawn_loc_bb != 0 {
            let pc = pawn_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += pc;
            pawn_loc_bb = pawn_loc_bb >> pc;
            let pawn_point_bb = BB::idx_to_bb(start_idx as i32);

            // normal captures and en passant
            let mut pawn_capture_bb: u64 = if white {
                ((pawn_point_bb & !FILE_MASKS[0]) << 7) | ((pawn_point_bb & !FILE_MASKS[7]) << 9)
            } else {
                ((pawn_point_bb & !FILE_MASKS[0]) >> 9) | ((pawn_point_bb & !FILE_MASKS[7]) >> 7)
            } & capture_composite;

            // enqueue
            let mut end_idx = -1;
            let mut c = 0;
            while pawn_capture_bb != 0 && c < 64 {
                c = pawn_capture_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                pawn_capture_bb = pawn_capture_bb >> c;

                if end_idx == self.ep {
                    moves.push(Mv::pawn_ep_move(start_idx, end_idx));
                } else {
                    if (white && end_idx >= 56) || (!white && end_idx < 8) {
                        // promotion
                        for p in [b'q', b'r', b'b', b'n'].iter() {
                            moves.push(Mv::pawn_promote_move(start_idx, end_idx, *p));
                        }
                    } else {
                        moves.push(Mv::pawn_move(start_idx, end_idx));
                    }
                }
            }
        }

        return moves;
    }

    pub fn pawn_moves(&self, white: bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut pawn_loc_bb = self.pawn[white as usize];
        let all_composite = self.composite[0] | self.composite[1];
        let mut capture_composite = self.composite[!white as usize];
        if self.ep != -1 {
            // include a "ghost" entry in the capture composite
            // if an ep-able move happened
            capture_composite |= BB::idx_to_bb(self.ep);
        }

        while pawn_loc_bb != 0 {
            let pc = pawn_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += pc;
            pawn_loc_bb = pawn_loc_bb >> pc;
            let pawn_point_bb = BB::idx_to_bb(start_idx as i32);

            // normal captures and en passant
            let mut pawn_capture_bb: u64 = if white {
                ((pawn_point_bb & !FILE_MASKS[0]) << 7) | ((pawn_point_bb & !FILE_MASKS[7]) << 9)
            } else {
                ((pawn_point_bb & !FILE_MASKS[0]) >> 9) | ((pawn_point_bb & !FILE_MASKS[7]) >> 7)
            } & capture_composite;

            // walk forward once
            let mut pawn_walk_bb: u64 = if white {
                pawn_point_bb << 8
            } else {
                pawn_point_bb >> 8
            } & !all_composite;

            // walk forward twice
            if pawn_walk_bb != 0 {
                // we can (maybe) walk forward twice
                if (white && (start_idx >> 3) == 1) || (!white && (start_idx >> 3) == 6) {
                    pawn_walk_bb |= if white {
                        pawn_point_bb << 16
                    } else {
                        pawn_point_bb >> 16
                    } & !all_composite;
                }
            }

            // time to enqueue the moves

            // captures
            let mut end_idx = -1;
            let mut c = 0;
            while pawn_capture_bb != 0 && c < 64 {
                c = pawn_capture_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                pawn_capture_bb = pawn_capture_bb >> c;

                if end_idx == self.ep {
                    moves.push(Mv::pawn_ep_move(start_idx, end_idx));
                } else {
                    if (white && end_idx >= 56) || (!white && end_idx < 8) {
                        // promotion
                        for p in [b'q', b'r', b'b', b'n'].iter() {
                            moves.push(Mv::pawn_promote_move(start_idx, end_idx, *p));
                        }
                    } else {
                        moves.push(Mv::pawn_move(start_idx, end_idx));
                    }
                }
            }

            // walk forward
            end_idx = -1;
            c = 0;
            while pawn_walk_bb != 0 && c < 64 {
                c = pawn_walk_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                pawn_walk_bb = pawn_walk_bb >> c;

                if (white && end_idx >= 56) || (!white && end_idx < 8) {
                    // promotion
                    for p in [b'q', b'r', b'b', b'n'].iter() {
                        moves.push(Mv::pawn_promote_move(start_idx, end_idx, *p));
                    }
                } else {
                    moves.push(Mv::pawn_move(start_idx, end_idx));
                }
            }
        }

        return moves;
    }

    pub fn rook_q_moves(&self, white:bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut rook_loc_bb = self.rook[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut rc = 0;
        while rook_loc_bb != 0  && rc < 64 {
            rc = rook_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += rc;
            rook_loc_bb = rook_loc_bb >> rc;

            // lookup from the magic table
            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let mut rook_bb = self.rook_magic_table[start_idx as usize][hash as usize];

            // only captures
            rook_bb = rook_bb & self.composite[!white as usize];
            let mut end_idx = -1;

            let mut c = 0;
            while rook_bb != 0 && c < 64 {
                c = rook_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                rook_bb = rook_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'r'));
            }
        }
        return moves;
    }

    pub fn rook_moves(&self, white:bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut rook_loc_bb = self.rook[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut rc = 0;
        while rook_loc_bb != 0  && rc < 64 {
            rc = rook_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += rc;
            rook_loc_bb = rook_loc_bb >> rc;

            // lookup from the magic table
            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let mut rook_bb = self.rook_magic_table[start_idx as usize][hash as usize];

            // can't hit your own guys
            rook_bb = rook_bb & !self.composite[white as usize];
            let mut end_idx = -1;

            let mut c = 0;
            while rook_bb != 0 && c < 64 {
                c = rook_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                rook_bb = rook_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'r'));
            }
        }
        return moves;
    }

    pub fn bishop_q_moves(&self, white:bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut bishop_loc_bb = self.bishop[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut bc = 0;
        while bishop_loc_bb != 0  && bc < 64 {
            bc = bishop_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += bc;
            bishop_loc_bb = bishop_loc_bb >> bc;

            // lookup from the magic table
            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let mut bishop_bb = self.bishop_magic_table[start_idx as usize][hash as usize];

            // only captures
            bishop_bb = bishop_bb & self.composite[!white as usize];
            let mut end_idx = -1;

            let mut c = 0;
            while bishop_bb != 0 && c < 64 {
                c = bishop_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                bishop_bb = bishop_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'b'));
            }
        }
        return moves;
    }

    pub fn bishop_moves(&self, white:bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut bishop_loc_bb = self.bishop[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut bc = 0;
        while bishop_loc_bb != 0  && bc < 64 {
            bc = bishop_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += bc;
            bishop_loc_bb = bishop_loc_bb >> bc;

            // lookup from the magic table
            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let mut bishop_bb = self.bishop_magic_table[start_idx as usize][hash as usize];

            // can't hit your own guys
            bishop_bb = bishop_bb & !self.composite[white as usize];
            let mut end_idx = -1;

            let mut c = 0;
            while bishop_bb != 0 && c < 64 {
                c = bishop_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                bishop_bb = bishop_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'b'));
            }
        }
        return moves;
    }

    pub fn queen_q_moves(&self, white:bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut queen_loc_bb = self.queen[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut qc = 0;
        while queen_loc_bb != 0 && qc < 64 {
            qc = queen_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += qc;
            queen_loc_bb = queen_loc_bb >> qc;

            // treat a queen as a rook bishop combo
            // lookup from the magic table
            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let rook_hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let rook_bb = self.rook_magic_table[start_idx as usize][rook_hash as usize];

            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let bishop_hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let bishop_bb = self.bishop_magic_table[start_idx as usize][bishop_hash as usize];

            let mut queen_bb = rook_bb | bishop_bb;
            // only captures
            queen_bb = queen_bb & self.composite[!white as usize];
            let mut end_idx = -1;

            let mut c = 0;
            while queen_bb != 0 && c < 64 {
                c = queen_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                queen_bb = queen_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'q'));
            }
        }
        return moves;
    }

    pub fn queen_moves(&self, white:bool) -> Vec<Mv> {
        let mut moves: Vec<Mv> = Vec::new();
        let mut start_idx = -1;
        let mut queen_loc_bb = self.queen[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut qc = 0;
        while queen_loc_bb != 0 && qc < 64 {
            qc = queen_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += qc;
            queen_loc_bb = queen_loc_bb >> qc;

            // treat a queen as a rook bishop combo
            // lookup from the magic table
            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let rook_hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let rook_bb = self.rook_magic_table[start_idx as usize][rook_hash as usize];

            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let bishop_hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let bishop_bb = self.bishop_magic_table[start_idx as usize][bishop_hash as usize];

            let mut queen_bb = rook_bb | bishop_bb;
            // can't hit your own guys
            queen_bb = queen_bb & !self.composite[white as usize];
            let mut end_idx = -1;

            let mut c = 0;
            while queen_bb != 0 && c < 64 {
                c = queen_bb.trailing_zeros() as i32 + 1;
                end_idx += c;
                queen_bb = queen_bb >> c;

                moves.push(Mv::normal_move(start_idx, end_idx, b'q'));
            }
        }
        return moves;
    }

    pub fn q_moves(&self) -> Vec<Mv> {
        let mut move_queue : Vec<Mv> = Vec::new();

        move_queue.append(& mut self.pawn_q_moves(self.white_turn));
        move_queue.append(& mut self.king_q_moves(self.white_turn));
        move_queue.append(& mut self.queen_q_moves(self.white_turn));
        move_queue.append(& mut self.rook_q_moves(self.white_turn));
        move_queue.append(& mut self.bishop_q_moves(self.white_turn));
        move_queue.append(& mut self.knight_q_moves(self.white_turn));

        return move_queue;
    }

    pub fn moves(&self) -> Vec<Mv> {
        let mut move_queue : Vec<Mv> = Vec::new();

        move_queue.append(& mut self.pawn_moves(self.white_turn));
        move_queue.append(& mut self.king_moves(self.white_turn));
        move_queue.append(& mut self.queen_moves(self.white_turn));
        move_queue.append(& mut self.rook_moves(self.white_turn));
        move_queue.append(& mut self.bishop_moves(self.white_turn));
        move_queue.append(& mut self.knight_moves(self.white_turn));

        return move_queue;
    }

    fn rook_attacks(&self, white: bool) -> u64 {
        let mut attacks = 0;
        let mut start_idx = -1;
        let mut rook_loc_bb = self.rook[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut rc = 0;
        while rook_loc_bb != 0  && rc < 64 {
            rc = rook_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += rc;
            rook_loc_bb = rook_loc_bb >> rc;

            // lookup from the magic table
            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let rook_bb = self.rook_magic_table[start_idx as usize][hash as usize];

            // can't hit your own guys
            attacks |= rook_bb;
        }
        return attacks;
    }

    fn bishop_attacks(&self, white: bool) -> u64 {
        let mut attacks: u64 = 0;
        let mut start_idx = -1;
        let mut bishop_loc_bb = self.bishop[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut bc = 0;
        while bishop_loc_bb != 0  && bc < 64 {
            bc = bishop_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += bc;
            bishop_loc_bb = bishop_loc_bb >> bc;

            // lookup from the magic table
            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let bishop_bb = self.bishop_magic_table[start_idx as usize][hash as usize];

            attacks |= bishop_bb;
        }
        return attacks;
    }

    fn knight_attacks(&self, white: bool) -> u64 {
        let mut attacks: u64 = 0;
        let mut start_idx: i32 = -1;
        let mut knight_loc_bb = self.knight[white as usize];

        let mut kc = 0;
        while knight_loc_bb != 0 && kc < 64 {
            kc = knight_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            knight_loc_bb = knight_loc_bb >> kc;

            // only include moves that don't hit your own guys
            attacks |= self.knight_mask[start_idx as usize];
        }
        return attacks;
    }

    fn pawn_attacks(&self, white: bool) -> u64 {
        // as a first pass not going to count promotions as 4 moves
        // and ignoring ep
        // this will also miss counting some due to overlap
        // but I think slight inaccuracy is worth the perf improvement
        let pawn_loc_bb = self.pawn[white as usize];
        let capture_composite = self.composite[!white as usize];

        let pawn_capture_bb = if white {
            ((pawn_loc_bb & !FILE_MASKS[0]) << 7) | ((pawn_loc_bb & !FILE_MASKS[7]) << 9)
        } else {
            ((pawn_loc_bb & !FILE_MASKS[0]) >> 9) | ((pawn_loc_bb & !FILE_MASKS[7]) >> 7)
        } & capture_composite;

        return pawn_capture_bb;
    }

    fn king_attacks(&self, white: bool) -> u64 {
        // ignores castling
        let mut start_idx: i32 = -1;
        let mut king_loc_bb = self.king[white as usize];

        let mut kc = 0;
        while king_loc_bb != 0 && kc < 64 {
            kc = king_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            king_loc_bb = king_loc_bb >> kc;

            // only include moves that don't hit your own guys
            return self.king_mask[start_idx as usize];
        }
        return 0;
    }

    fn queen_attacks(&self, white: bool) -> u64 {
        let mut attacks: u64 = 0;
        let mut start_idx: i32 = -1;
        let mut queen_loc_bb = self.queen[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];

        let mut qc = 0;
        while queen_loc_bb != 0 && qc < 64 {
            qc = queen_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += qc;
            queen_loc_bb = queen_loc_bb >> qc;

            // treat a queen as a rook bishop combo
            // lookup from the magic table
            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let rook_hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let rook_bb = self.rook_magic_table[start_idx as usize][rook_hash as usize];

            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let bishop_hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let bishop_bb = self.bishop_magic_table[start_idx as usize][bishop_hash as usize];
            attacks |= rook_bb | bishop_bb;
        }
        return attacks;
    }

    fn get_defended_pieces(&self, white: bool) -> u64 {
        let mut atked_squares: u64 = 0;
        atked_squares |= self.king_attacks(white);
        atked_squares |= self.queen_attacks(white);
        atked_squares |= self.bishop_attacks(white);
        atked_squares |= self.rook_attacks(white);
        atked_squares |= self.knight_attacks(white);
        atked_squares |= self.pawn_attacks(white);

        return atked_squares & self.composite[white as usize];
    }

    pub fn order_capture_moves(&self, mvs: Vec<Mv>, k_array: &[Mv; 3], h_table: &[[[u64; 64]; 6]; 2]) -> Vec<Mv> {
        let side = self.white_turn as usize;
        let enemy_side = !self.white_turn as usize;
        let enemy_occ = self.composite[enemy_side];
        let def_pieces = self.get_defended_pieces(!self.white_turn);

        let mut free_caps: Vec<Mv> = Vec::new();
        let mut pawn_caps: Vec<Mv> = Vec::new();
        let mut winning_caps: Vec<Mv> = Vec::new();
        let mut equal_caps: Vec<Mv> = Vec::new();
        let mut losing_caps: Vec<Mv> = Vec::new();
        let mut killer_moves: Vec<Mv> = Vec::new();
        let mut no_caps: Vec<Mv> = Vec::new();
        let mut mv_q: Vec<Mv> = Vec::new();

        for mv in mvs {
            let dst_bb = BB::idx_to_bb(mv.end);
            if (dst_bb & enemy_occ) != 0 {
                // capture of some sort
                if dst_bb & def_pieces == 0 {
                    // undefended piece.  That's good
                    free_caps.push(mv);
                    continue;
                }
                match mv.piece {
                    b'p' => {
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            equal_caps.push(mv);
                        } else {
                            // must be capturing non-pawn
                            pawn_caps.push(mv);
                        }
                    },
                    b'n' => {
                        // losing
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            losing_caps.push(mv);
                        } else if (dst_bb & (self.knight[enemy_side] | self.bishop[enemy_side])) != 0 {
                            equal_caps.push(mv);
                        } else {
                            winning_caps.push(mv);
                        }
                    },
                    b'b' => {
                        // losing
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            losing_caps.push(mv);
                        } else if (dst_bb & (self.knight[enemy_side] | self.bishop[enemy_side])) != 0 {
                            equal_caps.push(mv);
                        } else {
                            winning_caps.push(mv);
                        }
                    },
                    b'r' => {
                        // winning
                        if (dst_bb & (self.queen[enemy_side] | self.king[enemy_side])) != 0 {
                            winning_caps.push(mv);
                        } else if (dst_bb & self.rook[enemy_side]) != 0 {
                            equal_caps.push(mv);
                        } else {
                            losing_caps.push(mv);
                        }
                    },
                    b'q' => {
                        // winning
                        if (dst_bb & self.king[enemy_side]) != 0 {
                            winning_caps.push(mv);
                        } else if (dst_bb & self.queen[enemy_side]) != 0 {
                            equal_caps.push(mv);
                        } else {
                            losing_caps.push(mv);
                        }
                    },
                    _ => {no_caps.push(mv);}
                }
            } else {
                if BB::moves_equivalent(&mv, &k_array[0]) {
                    killer_moves.push(mv);
                } else if BB::moves_equivalent(&mv, &k_array[1]) {
                    killer_moves.push(mv);
                } else if BB::moves_equivalent(&mv, &k_array[2]) {
                    killer_moves.push(mv);
                } else {
                    let history_value = h_table[side][mv.get_piece_num()][mv.end as usize];
                    if history_value == 0 || no_caps.len() == 0 {
                        no_caps.push(mv);
                        continue;
                    }
                    let mut pushed = false;
                    let mut i = 0;
                    for other_mv in no_caps.iter() {
                        if h_table[side][other_mv.get_piece_num()][other_mv.end as usize] <= history_value {
                            no_caps.insert(i, mv);
                            pushed = true;
                            break;
                        }
                        i += 1;
                    }
                    if !pushed {
                        no_caps.push(mv);
                    }
                }
            }
        }

        mv_q.append(& mut free_caps);
        mv_q.append(& mut pawn_caps);
        mv_q.append(& mut winning_caps);
        mv_q.append(& mut equal_caps);
        mv_q.append(& mut killer_moves);
        mv_q.append(& mut no_caps);
        mv_q.append(& mut losing_caps);
        return mv_q;
    }

    pub fn get_scored_moves(&self, mvs: Vec<Mv>, k_array: &[Mv; 3], h_table: &[[[u64; 64]; 6]; 2]) -> Vec<(Mv, u64)> {
        let side = self.white_turn as usize;
        let enemy_side = !self.white_turn as usize;
        let enemy_occ = self.composite[enemy_side];
        let def_pieces = self.get_defended_pieces(!self.white_turn);

        let cap_offset: u64 = 1 << 42;
        let free_cap_offset: u64 = 1 << 45;
        let killer_score_offset: u64 = 1 << 40;
        let no_cap_offset: u64 = 1 << 10;
        let bad_cap_offset: u64 = 0; // TODO move sophistication
        let mut mv_q: Vec<(Mv, u64)> = Vec::new();

        for mv in mvs {
            let mut mv_score: u64 = 0;
            let dst_bb = BB::idx_to_bb(mv.end);
            if (dst_bb & enemy_occ) == 0 {
                // not a capture
                if BB::moves_equivalent(&mv, &k_array[0]) ||
                    BB::moves_equivalent(&mv, &k_array[1]) ||
                    BB::moves_equivalent(&mv, &k_array[2]) {
                    mv_score = killer_score_offset;
                } else {
                    mv_score = no_cap_offset + h_table[side][mv.get_piece_num()][mv.end as usize];
                }
            } else {
                let my_val = match mv.piece {
                    b'p' => 1000,
                    b'n' => 3000,
                    b'b' => 3000,
                    b'r' => 5000,
                    b'q' => 9000,
                    b'k' => 10000,
                    _ => 0
                };

                if dst_bb & def_pieces == 0 {
                    // free capture.  That's good
                    mv_score = free_cap_offset - my_val;
                } else {
                    if mv.piece == b'k' {
                        // not legal
                        continue;
                    }
                    // defended
                    let mut other_val: u64 = 0;
                    if dst_bb & self.pawn[enemy_side] != 0 {
                        other_val = 1000;
                    } else if dst_bb & self.knight[enemy_side] != 0 {
                        other_val = 3000;
                    } else if dst_bb & self.bishop[enemy_side] != 0 {
                        other_val = 3000;
                    } else if dst_bb & self.rook[enemy_side] != 0 {
                        other_val = 5000;
                    } else if dst_bb & self.queen[enemy_side] != 0 {
                        other_val = 9000;
                    } else if dst_bb & self.king[enemy_side] != 0 {
                        // shouldn't be possible
                        panic!("king capture?");
                    }
                    if my_val > other_val {
                        // "losing" capture
                        mv_score = 0;
                    } else {
                        mv_score = cap_offset + (other_val - my_val);
                    }
                }
            }

            mv_q.push((mv, mv_score));
        }
        return mv_q;
    }

    pub fn get_scored_q_moves(&self, mvs: Vec<Mv>) -> Vec<(Mv, u64)> {
        let side = self.white_turn as usize;
        let enemy_side = !self.white_turn as usize;
        let enemy_occ = self.composite[enemy_side];
        let def_pieces = self.get_defended_pieces(!self.white_turn);

        let cap_offset: u64 = 1 << 42;
        let free_cap_offset: u64 = 1 << 45;
        let killer_score_offset: u64 = 1 << 40;
        let no_cap_offset: u64 = 1 << 10;
        let bad_cap_offset: u64 = 0; // TODO move sophistication
        let mut mv_q: Vec<(Mv, u64)> = Vec::new();
        let mut vlc_q: Vec<(Mv, u64)> = Vec::new();

        for mv in mvs {
            let mut mv_score: u64 = 0;
            let dst_bb = BB::idx_to_bb(mv.end);
            if (dst_bb & enemy_occ) == 0 {
                // not a capture
                continue        // for now
            } else {
                let my_val = match mv.piece {
                    b'p' => 1000,
                    b'n' => 3000,
                    b'b' => 3000,
                    b'r' => 5000,
                    b'q' => 9000,
                    b'k' => 10000,
                    _ => 0
                };

                if dst_bb & def_pieces == 0 {
                    // free capture.  That's good
                    mv_score = free_cap_offset - my_val;
                } else {
                    if mv.piece == b'k' {
                        // not legal
                        continue;
                    }
                    // defended
                    let mut other_val: u64 = 0;
                    if dst_bb & self.pawn[enemy_side] != 0 {
                        other_val = 1000;
                    } else if dst_bb & self.knight[enemy_side] != 0 {
                        other_val = 3000;
                    } else if dst_bb & self.bishop[enemy_side] != 0 {
                        other_val = 3000;
                    } else if dst_bb & self.rook[enemy_side] != 0 {
                        other_val = 5000;
                    } else if dst_bb & self.queen[enemy_side] != 0 {
                        other_val = 9000;
                    } else if dst_bb & self.king[enemy_side] != 0 {
                        // shouldn't be possible
                        panic!("king capture?");
                    }
                    if my_val > other_val {
                        // "losing" capture
                        mv_score = 0;
                    } else {
                        mv_score = cap_offset + (other_val - my_val);
                    }
                }
            }

            mv_q.push((mv, mv_score));
        }
        return mv_q;
    }

    pub fn order_and_filter_capture_moves(&self, mvs: Vec<Mv>) -> (Vec<Mv>, Vec<Mv>) {
        let enemy_side = !self.white_turn as usize;
        let enemy_occ = self.composite[enemy_side];
        let def_pieces = self.get_defended_pieces(!self.white_turn);
        let pawn_def_pieces = self.pawn_attacks(!self.white_turn) & self.composite[!self.white_turn as usize];

        let mut free_caps: Vec<Mv> = Vec::new();
        let mut pawn_caps: Vec<Mv> = Vec::new();
        let mut winning_caps: Vec<Mv> = Vec::new();
        let mut equal_caps: Vec<Mv> = Vec::new();
        let mut losing_caps: Vec<Mv> = Vec::new();
        let mut very_losing_caps: Vec<Mv> = Vec::new();
        let mut mv_q: Vec<Mv> = Vec::new();

        for mv in mvs {
            let dst_bb = BB::idx_to_bb(mv.end);
            if (dst_bb & enemy_occ) != 0 {
                // capture of some sort
                if dst_bb & def_pieces == 0 {
                    // undefended piece.  That's good
                    free_caps.push(mv);
                    continue;
                }
                if dst_bb & pawn_def_pieces == 0 {
                    // might be very bad
                    let mut skip = false;
                    if mv.piece == b'r' {
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            very_losing_caps.push(mv);
                            skip = true;
                        }
                    } else if mv.piece == b'b' || mv.piece == b'n' {
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            very_losing_caps.push(mv);
                            skip = true;
                        }
                    } else if mv.piece == b'q' {
                        if (dst_bb & self.queen[enemy_side]) == 0 {
                            very_losing_caps.push(mv);
                            skip = true;
                        }
                    }
                    if skip {
                        continue;
                    }
                }
                match mv.piece {
                    b'p' => {
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            equal_caps.push(mv);
                        } else {
                            // must be capturing non-pawn
                            pawn_caps.push(mv);
                        }
                    },
                    b'n' => {
                        // losing
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            losing_caps.push(mv);
                        } else if (dst_bb & (self.knight[enemy_side] | self.bishop[enemy_side])) != 0 {
                            equal_caps.push(mv);
                        } else {
                            winning_caps.push(mv);
                        }
                    },
                    b'b' => {
                        // losing
                        if (dst_bb & self.pawn[enemy_side]) != 0 {
                            losing_caps.push(mv);
                        } else if (dst_bb & (self.knight[enemy_side] | self.bishop[enemy_side])) != 0 {
                            equal_caps.push(mv);
                        } else {
                            winning_caps.push(mv);
                        }
                    },
                    b'r' => {
                        // winning
                        if (dst_bb & (self.queen[enemy_side] | self.king[enemy_side])) != 0 {
                            winning_caps.push(mv);
                        } else if (dst_bb & self.rook[enemy_side]) != 0 {
                            equal_caps.push(mv);
                        } else {
                            losing_caps.push(mv);
                        }
                    },
                    b'q' => {
                        // winning
                        if (dst_bb & self.king[enemy_side]) != 0 {
                            winning_caps.push(mv);
                        } else if (dst_bb & self.queen[enemy_side]) != 0 {
                            equal_caps.push(mv);
                        } else {
                            losing_caps.push(mv);
                        }
                    },
                    _ => {}
                }
            }
        }
        mv_q.append(& mut free_caps);
        mv_q.append(& mut pawn_caps);
        mv_q.append(& mut winning_caps);
        mv_q.append(& mut equal_caps);
        mv_q.append(& mut losing_caps);
        return (mv_q, very_losing_caps);
    }

    pub fn moves_equivalent(mv1: &Mv, mv2: &Mv) -> bool {
        return mv1.start == mv2.start
            && mv1.end == mv2.end
            && mv1.piece == mv2.piece
            && mv1.promote_to == mv2.promote_to
            && mv1.ep_tile == mv2.ep_tile
            && mv1.is_ep == mv2.is_ep;
    }

    pub fn do_null_move(&mut self) {
        self.white_turn = !self.white_turn;
        self.hash ^= self.zobrist_table.1.0 ^ self.zobrist_table.1.1;
        self.ep_stack.push(self.ep);
        self.ep = -1;
    }

    pub fn undo_null_move(&mut self) {
        self.white_turn = !self.white_turn;
        self.hash ^= self.zobrist_table.1.0 ^ self.zobrist_table.1.1;
        self.ep = match self.ep_stack.pop() {Some(p) => p, None => panic!("empty ep stack")};
    }

    // let's make moves
    pub fn do_move(&mut self, mv: &Mv) {
        // push history
        // push ep
        // push castling rights

        // check capture
        // push capture

        // check castling
        // do castling

        // check promotion
        // move piece

        // update material

        // update castling rights
        // update composites
        // TODO: update hash
        // flip turn

        self.ep_stack.push(self.ep);
        self.cr_stack.push(self.castling_rights);
        self.history.push(self.hash);
        self.pawn_history.push(self.pawn_hash);

        let start_point: u64 = BB::idx_to_bb(mv.start);
        let end_point: u64 = BB::idx_to_bb(mv.end);
        let self_side: usize = self.white_turn as usize;
        let enemy_side: usize = !self.white_turn as usize;
        // positive favors side making move
        let mut material_delta: i32 = 0;
        let mut captured_piece: u8 = 0;

        // check en passant first because it's quick
        if mv.is_ep {
            captured_piece = b'p';
            let actual_pawn_idx: i32 = match self.ep - mv.start {
                7 => { mv.start - 1 }, // up and left
                9 => { mv.start + 1 }, // up and right
                -7 => { mv.start + 1 },// down and right
                -9 => { mv.start - 1 },// down and left
                _ => {panic!("impossible ep move {} s {} e {} ep coord {} piece {}!", mv, mv.start, mv.end, self.ep, mv.piece);}
            };

            // remove the enemy pawn
            self.pawn[enemy_side] ^= BB::idx_to_bb(actual_pawn_idx);
            self.hash ^= self.get_zr_xor(actual_pawn_idx as usize, b'p', !self.white_turn);
            self.pawn_hash ^= self.get_zr_xor(actual_pawn_idx as usize, b'p', !self.white_turn);
            self.pawn_pt_score += if self.white_turn {1} else {-1} * self.get_pawn_table_val(actual_pawn_idx, !self.white_turn);
            material_delta += PAWN_VALUE;

        } else if (end_point & self.composite[enemy_side]) != 0 {
            // capture happened.  *sigh*, we need to find out what got captured

            // go in order of likelihood?
            // pawn
            if (end_point & self.pawn[enemy_side]) != 0 {
                // pawn captured
                captured_piece = b'p';
                self.pawn[enemy_side] ^= end_point;
                material_delta += PAWN_VALUE;
                self.phase += board_eval::PAWN_PHASE;
            } else if (end_point & self.bishop[enemy_side]) != 0 {
                // bishop captured
                captured_piece = b'b';
                self.bishop[enemy_side] ^= end_point;
                material_delta += BISHOP_VALUE;
                self.phase += board_eval::BISHOP_PHASE;
            } else if (end_point & self.rook[enemy_side]) != 0 {
                // rook captured
                captured_piece = b'r';
                self.rook[enemy_side] ^= end_point;
                material_delta += ROOK_VALUE;
                self.phase += board_eval::ROOK_PHASE;
            } else if (end_point & self.knight[enemy_side]) != 0 {
                // knight captured
                captured_piece = b'n';
                self.knight[enemy_side] ^= end_point;
                material_delta += KNIGHT_VALUE;
                self.phase += board_eval::KNIGHT_PHASE;
            } else if (end_point & self.queen[enemy_side]) != 0 {
                // queen captured
                captured_piece = b'q';
                self.queen[enemy_side] ^= end_point;
                material_delta += QUEEN_VALUE;
                self.phase += board_eval::QUEEN_PHASE;
            } else if (end_point & self.king[enemy_side]) != 0 {
                // king captured
                captured_piece = b'k';
                self.king[enemy_side] ^= end_point;
                material_delta += KING_VALUE;
            }
        }

        // gotta keep track of the captured piece
        self.cap_stack.push(captured_piece);

        // castling
        if mv.piece == b'k' && (mv.end - mv.start).abs() == 2 {
            let old_rook_idx: i32 = if mv.end > mv.start { mv.start + 3 } else { mv.start - 4 };
            let new_rook_idx: i32 = if mv.end > mv.start { mv.start + 1 } else { mv.start - 1 };

            // move the rook
            let rook_mask = BB::idx_to_bb(old_rook_idx) | BB::idx_to_bb(new_rook_idx);
            self.rook[self_side] ^= rook_mask;
            self.update_hash(b'r', self.white_turn, old_rook_idx, new_rook_idx, 0, 0);
            // self.update_pt_score(b'r')
            self.castled[self_side] = true;
        }

        // move piece
        if mv.promote_to != 0 {
            self.pawn[self_side] ^= start_point;
            let mut material_gain;
            match mv.promote_to {
                b'q' => { self.queen[self_side] |= end_point; material_gain = QUEEN_VALUE; },
                b'r' => { self.rook[self_side] |= end_point; material_gain = ROOK_VALUE; },
                b'b' => { self.bishop[self_side] |= end_point; material_gain = BISHOP_VALUE; },
                b'n' => { self.knight[self_side] |= end_point; material_gain = KNIGHT_VALUE; },
                _ => { panic!("illegal promotion on mv {}", mv); }
            }
            material_delta += material_gain - PAWN_VALUE;
        } else {
            let move_mask = start_point | end_point;
            match mv.piece {
                b'k' => { self.king[self_side] ^= move_mask; },
                b'q' => { self.queen[self_side] ^= move_mask; },
                b'r' => { self.rook[self_side] ^= move_mask; },
                b'b' => { self.bishop[self_side] ^= move_mask; },
                b'n' => { self.knight[self_side] ^= move_mask; },
                b'p' => { self.pawn[self_side] ^= move_mask; },
                _ => { panic!("moved nonexistent piece {} in mv {}", mv.piece, mv); }
            }
        }

        // main update hash
        self.update_hash(mv.piece, self.white_turn, mv.start, mv.end, if mv.is_ep {0} else {captured_piece}, mv.promote_to);
        self.update_pt_score(mv.piece, self.white_turn, mv.start, mv.end, if mv.is_ep {0} else {captured_piece}, mv.promote_to);

        // update material
        self.material += material_delta * if self.white_turn {1} else {-1};

        // update castling rights
        self.castling_rights &= !(start_point | end_point);

        // update ep tile
        self.ep = mv.ep_tile;

        // update composites
        // TODO this can be done as part of the other operations
        self.composite[self_side] = (
            self.pawn[self_side] | self.knight[self_side] | self.bishop[self_side] |
            self.rook[self_side] | self.queen[self_side] | self.king[self_side]
        );

        self.composite[enemy_side] = (
            self.pawn[enemy_side] | self.knight[enemy_side] | self.bishop[enemy_side] |
            self.rook[enemy_side] | self.queen[enemy_side] | self.king[enemy_side]
        );

        // TODO update hash
        self.white_turn = !self.white_turn;
        self.hash ^= self.zobrist_table.1.0 ^ self.zobrist_table.1.1;
    }

    pub fn undo_move(&mut self, mv: &Mv) {
        // flip turn

        // pop ep
        // pop castling rights

        // undo capture
        // pop capture

        // undo castling

        // undo promotion
        // move piece

        // update material

        // pop history
        self.white_turn = !self.white_turn;
        let start_point: u64 = BB::idx_to_bb(mv.start);
        let end_point: u64 = BB::idx_to_bb(mv.end);
        let self_side: usize = self.white_turn as usize;
        let enemy_side: usize = !self.white_turn as usize;
        // positive is against side making move
        let mut material_delta: i32 = 0;
        let captured_piece = match self.cap_stack.pop() {
            Some(p) => p,
            None => panic!("Empty capture stack!")
        };

        self.ep = match self.ep_stack.pop() {Some(p) => p, None => panic!("empty ep stack!")};
        self.castling_rights = match self.cr_stack.pop() {Some(p) => p, None => panic!("empty cr stack!")};

        // restore captured_piece
        if mv.is_ep {
            let actual_pawn_idx: i32 = match self.ep - mv.start {
                7 => { mv.start - 1 }, // up and left
                9 => { mv.start + 1 }, // up and right
                -7 => { mv.start + 1 },// down and right
                -9 => { mv.start - 1 },// down and left
                _ => {panic!("impossible ep move {}!", mv);}
            };
            // remove the enemy pawn
            self.pawn[enemy_side] ^= BB::idx_to_bb(actual_pawn_idx);
            self.pawn_pt_score -= if self.white_turn {1} else {-1} * self.get_pawn_table_val(actual_pawn_idx, !self.white_turn);
            material_delta += PAWN_VALUE;
        } else if captured_piece != 0 {
            match captured_piece {
                b'k' => {
                    self.king[enemy_side] ^= end_point;
                    material_delta += KING_VALUE;
                },
                b'q' => {
                    self.queen[enemy_side] ^= end_point;
                    material_delta += QUEEN_VALUE;
                    self.phase -= board_eval::QUEEN_PHASE;
                },
                b'r' => {
                    self.rook[enemy_side] ^= end_point;
                    material_delta += ROOK_VALUE;
                    self.phase -= board_eval::ROOK_PHASE;
                },
                b'b' => {
                    self.bishop[enemy_side] ^= end_point;
                    material_delta += BISHOP_VALUE;
                    self.phase -= board_eval::BISHOP_PHASE;
                },
                b'n' => {
                    self.knight[enemy_side] ^= end_point;
                    material_delta += KNIGHT_VALUE;
                    self.phase -= board_eval::KNIGHT_PHASE;
                },
                b'p' => {
                    self.pawn[enemy_side] ^= end_point;
                    material_delta += PAWN_VALUE;
                    self.phase -= board_eval::PAWN_PHASE;
                },
                _ => panic!("captured piece type that doesn't exist!")
            };
        }

        // castling
        if mv.piece == b'k' && (mv.end - mv.start).abs() == 2 {
            let old_rook_idx: i32 = if mv.end > mv.start { mv.start + 3 } else { mv.start - 4 };
            let new_rook_idx: i32 = if mv.end > mv.start { mv.start + 1 } else { mv.start - 1 };

            // move the rook
            let rook_mask = BB::idx_to_bb(old_rook_idx) | BB::idx_to_bb(new_rook_idx);
            self.rook[self_side] ^= rook_mask;
            self.castled[self_side] = false;
        }

        // move piece
        if mv.promote_to != 0 {
            self.pawn[self_side] ^= start_point;
            let mut material_gain;
            match mv.promote_to {
                b'q' => { self.queen[self_side] ^= end_point; material_gain = QUEEN_VALUE; },
                b'r' => { self.rook[self_side] ^= end_point; material_gain = ROOK_VALUE; },
                b'b' => { self.bishop[self_side] ^= end_point; material_gain = BISHOP_VALUE; },
                b'n' => { self.knight[self_side] ^= end_point; material_gain = KNIGHT_VALUE; },
                _ => { panic!("illegal promotion on mv {}", mv); }
            }
            material_delta += material_gain - PAWN_VALUE;
        } else {
            let move_mask = start_point | end_point;
            match mv.piece {
                b'k' => { self.king[self_side] ^= move_mask; },
                b'q' => { self.queen[self_side] ^= move_mask; },
                b'r' => { self.rook[self_side] ^= move_mask; },
                b'b' => { self.bishop[self_side] ^= move_mask; },
                b'n' => { self.knight[self_side] ^= move_mask; },
                b'p' => { self.pawn[self_side] ^= move_mask; },
                _ => { panic!("moved nonexistent piece {} in mv {}", mv.piece, mv);}
            }
        }

        // update material
        self.material -= material_delta * if self.white_turn {1} else {-1};
        self.deupdate_pt_score(mv.piece, self.white_turn, mv.start, mv.end, if mv.is_ep {0} else {captured_piece}, mv.promote_to);

        // update composites
        // TODO this can be done as part of the other operations
        self.composite[self_side] = (
            self.pawn[self_side] | self.knight[self_side] | self.bishop[self_side] |
            self.rook[self_side] | self.queen[self_side] | self.king[self_side]
        );

        self.composite[enemy_side] = (
            self.pawn[enemy_side] | self.knight[enemy_side] | self.bishop[enemy_side] |
            self.rook[enemy_side] | self.queen[enemy_side] | self.king[enemy_side]
        );

        self.hash = match self.history.pop() {
            Some(p) => p,
            None => panic!("History stack empty!")
        };
        self.pawn_hash = match self.pawn_history.pop() {
            Some(p) => p,
            None => panic!("Pawn History stack empty!")
        }
    }

    pub fn side_to_move_has_capture(&self) -> bool {
        let white = self.white_turn;
        let mut atked_squares: u64 = 0;
        atked_squares |= self.king_attacks(white);
        atked_squares |= self.queen_attacks(white);
        atked_squares |= self.bishop_attacks(white);
        atked_squares |= self.rook_attacks(white);
        atked_squares |= self.knight_attacks(white);
        atked_squares |= self.pawn_attacks(white);

        return (atked_squares & self.composite[!white as usize]) != 0;
    }

    fn queen_mobility_kdf_attacks(&self, white: bool, king_bb: u64) -> (i32, i32, i32) {
        let mut attacks: i32 = 0;
        let mut attackers: i32 = 0;
        let mut start_idx: i32 = -1;
        let mut loc_bb = self.queen[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];
        let mut moves = 0;

        let mut c = 0;
        while loc_bb != 0 && c < 64 {
            c = loc_bb.trailing_zeros() as i32 + 1;
            start_idx += c;
            loc_bb = loc_bb >> c;

            // treat a queen as a rook bishop combo
            // lookup from the magic table
            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let rook_hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let rook_bb = self.rook_magic_table[start_idx as usize][rook_hash as usize];

            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let bishop_hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let bishop_bb = self.bishop_magic_table[start_idx as usize][bishop_hash as usize];

            moves += ((rook_bb | bishop_bb) & !self.composite[white as usize]).count_ones() as i32;
            let piece_attacks = ((rook_bb | bishop_bb) & king_bb).count_ones() as i32;
            if piece_attacks > 0 {
                attackers += 1;
            }
            attacks += piece_attacks;
        }
        return (moves, attackers, attacks);
    }

    fn bishop_mobility_kdf_attacks(&self, white: bool, king_bb: u64) -> (i32, i32, i32) {
        let mut attacks: i32 = 0;
        let mut attackers: i32 = 0;
        let mut start_idx: i32 = -1;
        let mut loc_bb = self.bishop[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];
        let mut moves = 0;

        let mut c = 0;
        while loc_bb != 0 && c < 64 {
            c = loc_bb.trailing_zeros() as i32 + 1;
            start_idx += c;
            loc_bb = loc_bb >> c;

            let bishop_occ_bb = self.bishop_mask[start_idx as usize] & all_composite;
            let bishop_hash = BB::bishop_magic_hash(bishop_occ_bb, start_idx as usize);
            let bishop_bb = self.bishop_magic_table[start_idx as usize][bishop_hash as usize];

            moves += (bishop_bb & !self.composite[white as usize]).count_ones() as i32;
            let piece_attacks = (bishop_bb & king_bb).count_ones() as i32;
            if piece_attacks > 0 {
                attackers += 1;
            }
            attacks += piece_attacks

        }
        return (moves, attackers, attacks);
    }

    fn rook_mobility_kdf_attacks(&self, white: bool, king_bb: u64) -> (i32, i32, i32) {
        let mut attacks: i32 = 0;
        let mut attackers: i32 = 0;
        let mut start_idx: i32 = -1;
        let mut loc_bb = self.rook[white as usize];
        let all_composite = self.composite[!white as usize] | self.composite[white as usize];
        let mut moves = 0;

        let mut c = 0;
        while loc_bb != 0 && c < 64 {
            c = loc_bb.trailing_zeros() as i32 + 1;
            start_idx += c;
            loc_bb = loc_bb >> c;

            let rook_occ_bb = self.rook_mask[start_idx as usize] & all_composite;
            let rook_hash = BB::rook_magic_hash(rook_occ_bb, start_idx as usize);
            let rook_bb = self.rook_magic_table[start_idx as usize][rook_hash as usize];

            moves += (rook_bb & !self.composite[white as usize]).count_ones() as i32;
            let piece_attacks = (rook_bb & king_bb).count_ones() as i32;
            if piece_attacks > 0 {
                attackers += 1;
            }
            attacks += piece_attacks;
        }
        return (moves, attackers, attacks);
    }

    fn knight_mobility_kdf_attacks(&self, white: bool, king_bb: u64) -> (i32, i32, i32) {
        let mut attacks: i32 = 0;
        let mut attackers: i32 = 0;
        let mut start_idx: i32 = -1;
        let mut knight_loc_bb = self.knight[white as usize];
        let mut moves = 0;

        let mut kc = 0;
        while knight_loc_bb != 0 && kc < 64 {
            kc = knight_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            knight_loc_bb = knight_loc_bb >> kc;

            let knight_bb = self.knight_mask[start_idx as usize];
            moves += (knight_bb & !self.composite[white as usize]).count_ones() as i32;
            let piece_attacks = (knight_bb & king_bb).count_ones() as i32;
            if piece_attacks > 0 {
                attackers += 1;
            }
            attacks += piece_attacks;
        }
        return (moves, attackers, attacks);
    }


    fn pawn_mobility(&self, white: bool) -> i32 {
        // as a first pass not going to count promotions as 4 moves
        // and ignoring ep
        // this will also miss counting some due to overlap
        // but I think slight inaccuracy is worth the perf improvement
        let pawn_loc_bb = self.pawn[white as usize];
        let all_composite = self.composite[0] | self.composite[1];
        let capture_composite = self.composite[!white as usize];

        let pawn_capture_bb = if white {
            ((pawn_loc_bb & !FILE_MASKS[0]) << 7) | ((pawn_loc_bb & !FILE_MASKS[7]) << 9)
        } else {
            ((pawn_loc_bb & !FILE_MASKS[0]) >> 9) | ((pawn_loc_bb & !FILE_MASKS[7]) >> 7)
        } & capture_composite;

        let pawn_walk_bb = if white {
            pawn_loc_bb << 8
        } else {
            pawn_loc_bb >> 8
        } & !all_composite;

        return (pawn_capture_bb.count_ones() + pawn_walk_bb.count_ones()) as i32;
    }

    fn king_mobility(&self, white: bool) -> i32 {
        // ignores castling
        let mut moves: u32 = 0;
        let mut start_idx: i32 = -1;
        let mut king_loc_bb = self.king[white as usize];

        let mut kc = 0;
        while king_loc_bb != 0 && kc < 64 {
            kc = king_loc_bb.trailing_zeros() as i32 + 1;
            start_idx += kc;
            king_loc_bb = king_loc_bb >> kc;

            // only include moves that don't hit your own guys
            let king_bb = self.king_mask[start_idx as usize] & !self.composite[white as usize];
            moves += king_bb.count_ones();
        }
        return moves as i32;
    }

    pub fn mobility_kdf_combo(&self) -> (i32, i32) {
        // mobility + king danger mashed together
        // for efficiency's sake
        let mut mobility: [i32; 2] = [0; 2];
        let mut king_danger: [i32; 2] = [0; 2];
        for side in 0..2 {
            let white = side != 0;

            let king_loc_bb = self.king[white as usize];
            if king_loc_bb == 0 {
                return (0, 0);
            }
            let mut attack_value: i32 = 0;

            let king_idx = king_loc_bb.trailing_zeros() as i32;

            let mut king_bb = king_loc_bb | self.king_mask[king_idx as usize];
            // eprintln!("{}", BB::bb_str(king_bb));
            // if white {
            //     king_bb |= king_bb << 8;
            // } else {
            //     king_bb |= king_bb >> 8;
            // }

            let (queen_moves, queen_attackers, queen_attacks) = self.queen_mobility_kdf_attacks(!white, king_bb);
            let (knight_moves, knight_attackers, knight_attacks) = self.knight_mobility_kdf_attacks(!white, king_bb);
            let (bishop_moves, bishop_attackers, bishop_attacks) = self.bishop_mobility_kdf_attacks(!white, king_bb);
            let (rook_moves, rook_attackers, rook_attacks) = self.rook_mobility_kdf_attacks(!white, king_bb);
            let pawn_moves = self.pawn_mobility(!white);
            let king_moves = self.king_mobility(!white);

            mobility[!white as usize] = queen_moves + knight_moves + bishop_moves + rook_moves + pawn_moves + king_moves;
            let mut num_attackers = queen_attackers + knight_attackers + bishop_attackers + rook_attackers;

            if num_attackers > 7 { num_attackers = 7; }

            attack_value += 800 * queen_attacks;
            attack_value += 400 * rook_attacks;
            attack_value += 200 * knight_attacks;
            attack_value += 200 * bishop_attacks;

            let atk_weights = [0, 5, 50, 75, 85, 90, 95, 100];
            king_danger[side] = ((atk_weights[num_attackers as usize] * attack_value) / 100) as i32;
        }

        let king_danger_value = (self.eval_params.king_danger * (king_danger[1] - king_danger[0])) / 100;
        let mobility = self.eval_params.mobility * (mobility[1] - mobility[0]);
        return (mobility, king_danger_value);
    }

    pub fn rook_on_seventh_bonus(&self) -> i32 {
        let white_bonus = (self.rook[1] & RANK_MASKS[6]).count_ones() as i32;
        // has to be a good reason to bonus 7th rank
        let white_cond = (self.king[0] & RANK_MASKS[7]) != 0 || (self.pawn[0] & RANK_MASKS[6]) != 0;

        let black_bonus = (self.rook[0] & RANK_MASKS[1]).count_ones() as i32;
        let black_cond = (self.king[1] & RANK_MASKS[0]) != 0 || (self.pawn[1] & RANK_MASKS[1] != 0);
        return self.eval_params.rook_on_seventh * (if white_cond {white_bonus} else {0} - if black_cond {black_bonus} else {0});
    }

    pub fn doubled_pawns_value(&self) -> i32 {
        let mut doubled_pawns: [i32; 2] = [0, 0];
        for i in 0..2 {
            let side = i as usize;
            for f in 0..8 {
                let mask = FILE_MASKS[f as usize];
                let file_pawns = mask & self.pawn[side];
                if file_pawns.count_ones() > 1 {
                    doubled_pawns[side] += 1;
                }
            }
        }
        return self.eval_params.doubled_pawn * (doubled_pawns[1] - doubled_pawns[0]);
    }

    pub fn isolated_pawns_value(&self) -> i32 {
        let mut isolated_pawns: [i32; 2] = [0, 0];
        for i in 0..2 {
            let side = i as usize;
            for f in 0..8 {
                let mask = FILE_MASKS[f as usize];
                let file_pawns = mask & self.pawn[side];
                if file_pawns != 0 {
                    let mut neighbor_files: u64 = 0;
                    if f > 0 {
                        neighbor_files |= FILE_MASKS[(f - 1) as usize];
                    }
                    if f < 7 {
                        neighbor_files |= FILE_MASKS[(f + 1) as usize];
                    }

                    if neighbor_files & self.pawn[side] == 0 {
                        isolated_pawns[side] += 1;
                    }
                }
            }
        }
        return self.eval_params.isolated_pawn * (isolated_pawns[1] - isolated_pawns[0]);
    }

    pub fn passed_pawns_value(&self) -> i32 {
        let mut passed_pawns: [i32; 2] = [0, 0];
        for i in 0..2 {
            let white = i != 0;
            let side = i as usize;
            for f in 0..8 {
                let mask = FILE_MASKS[f as usize];
                let file_pawns = mask & self.pawn[side];
                if file_pawns != 0 {
                    let mut enemy_mask: u64 = 0;
                    if f > 0 {
                        enemy_mask |= FILE_MASKS[(f - 1) as usize];
                    }
                    if f < 7 {
                        enemy_mask |= FILE_MASKS[(f + 1) as usize];
                    }
                    enemy_mask |= mask;
                    let mut rank_mask = 0;
                    for r in 0..8 {
                        let r = if white {7-r} else {r};
                        if RANK_MASKS[r] & file_pawns == 0 {
                            rank_mask |= RANK_MASKS[r]
                        } else {
                            break;
                        }
                    }
                    enemy_mask &= rank_mask;

                    if enemy_mask & self.pawn[!white as usize] == 0 {
                        passed_pawns[side] += 1;
                    }
                }
            }
        }
        return self.eval_params.passed_pawn * (passed_pawns[1] - passed_pawns[0]);
    }

    pub fn backwards_pawns_value(&self) -> i32 {
        let white_pawn_attacks = ((self.pawn[1] & !FILE_MASKS[0]) << 7) | ((self.pawn[1] & !FILE_MASKS[7]) << 9);

        let black_pawn_attacks = ((self.pawn[0] & !FILE_MASKS[0]) >> 9) | ((self.pawn[0] & !FILE_MASKS[7]) >> 7);

        // brute force but we cache it anyway
        let mut white_pawn_attack_proj = white_pawn_attacks;
        let mut black_pawn_attack_proj = black_pawn_attacks;
        for _r in 0..5 {
            white_pawn_attack_proj |= white_pawn_attack_proj << 8;
            black_pawn_attack_proj |= black_pawn_attack_proj >> 8;
        }

        let white_backwards_pawns = (self.pawn[1] << 8) & black_pawn_attacks & !white_pawn_attack_proj;
        let black_backwards_pawns = (self.pawn[0] >> 8) & white_pawn_attacks & !black_pawn_attack_proj;

        return self.eval_params.backwards_pawn * (white_backwards_pawns.count_ones() as i32 - black_backwards_pawns.count_ones() as i32);
    }

    pub fn pawn_advancement_value(&self) -> i32 {
        let mut pawn_advancement: [i32; 2] = [0, 0];
        // white
        for r in 0..8 {
            let mask = RANK_MASKS[r as usize];
            let rank_pawns = mask & self.pawn[1];
            if rank_pawns != 0 {
                pawn_advancement[1] += (r) * rank_pawns.count_ones() as i32;
            }
        }

        // black
        for r in 0..8 {
            let mask = RANK_MASKS[r as usize];
            let rank_pawns = mask & self.pawn[0];
            if rank_pawns != 0 {
                pawn_advancement[0] += (7-r) * rank_pawns.count_ones() as i32;
            }
        }

        let scale_numerator = 256 + self.get_phase();
        return self.eval_params.pav * (((pawn_advancement[1] - pawn_advancement[0]) * scale_numerator) / 256);
    }

    pub fn center_value(&self) -> i32 {
        let mut center_pieces: [i32; 2] = [0, 0];
        for i in 0..2 {
            let side = i as usize;
            center_pieces[side] = (CENTER_MASK & self.pawn[side]).count_ones() as i32;
        }
        return self.eval_params.center_pawn * (center_pieces[1] - center_pieces[0]);
    }

    pub fn near_center_value(&self) -> i32 {
        let mut center_pieces: [i32; 2] = [0, 0];
        for i in 0..2 {
            let side = i as usize;
            center_pieces[side] = (NEAR_CENTER_MASK & (self.pawn[side])).count_ones() as i32;
        }
        return self.eval_params.near_center_pawn * (center_pieces[1] - center_pieces[0]);
    }

    pub fn pawn_defense_value(&self) -> i32 {
        let mut pdf: i32 = 0;

        // white
        let pawn_capture_mask = ((self.pawn[1] & !FILE_MASKS[0]) << 7) | ((self.pawn[1] & !FILE_MASKS[7]) << 9);
        pdf += (pawn_capture_mask & self.composite[1]).count_ones() as i32;

        // black
        let pawn_capture_mask = ((self.pawn[0] & !FILE_MASKS[0]) >> 9) | ((self.pawn[0] & !FILE_MASKS[7]) >> 7);
        pdf -= (pawn_capture_mask & self.composite[0]).count_ones() as i32;

        return self.eval_params.pdf * pdf;
    }

    pub fn rook_on_open_file_value(&self) -> i32 {
        let mut open_file_rooks: [i32; 2] = [0, 0];
        let mut semi_open_file_rooks: [i32; 2] = [0, 0];
        for i in 0..2 {
            let side = i as usize;
            let enemy_side = if i == 1 {0} else {1};
            let rook_bb = self.rook[side];
            for f in 0..8 {
                let rooks = (FILE_MASKS[f] & rook_bb).count_ones();
                if rooks > 0 && ((FILE_MASKS[f] & self.pawn[side]) == 0) {
                    // at least semi-open
                    if (FILE_MASKS[f] & self.pawn[enemy_side]) == 0 {
                        open_file_rooks[side] += rooks as i32;
                    } else {
                        semi_open_file_rooks[side] += rooks as i32;
                    }
                }
            }
        }
        return self.eval_params.rook_on_open * (2 * (open_file_rooks[1] - open_file_rooks[0]) + (semi_open_file_rooks[1] - semi_open_file_rooks[0]));
    }

    pub fn double_bishop_bonus(&self) -> i32 {
        return self.eval_params.dbb * ((self.bishop[1].count_ones() >= 2) as i32 - (self.bishop[0].count_ones() >= 2) as i32);
    }

    pub fn early_queen_penalty(&self) -> i32 {
        if self.history.len() >= 10 { return 0; }
        let mut early_queen_white = 0;
        let mut early_queen_black = 0;

        // white
        if self.queen[1] != 0 && self.queen[1].trailing_zeros() != 3 {
            early_queen_white = 1;
        }

        // black
        if self.queen[0] != 0 && self.queen[0].trailing_zeros() != (56 + 3) {
            early_queen_black = 1;
        }

        return self.eval_params.early_queen_penalty * (early_queen_white - early_queen_black);
    }

    pub fn tempo_bonus(&self) -> i32 {
        return self.eval_params.tempo_bonus * if self.white_turn {1} else {-1};
    }

    pub fn material_advantage_bonus(&self) -> i32 {
        if self.material.abs() < 500 {
            return 0;
        }
        return (self.eval_params.material_advantage * self.phase * if self.material > 0 {1} else {-1}) / 100;
    }

    pub fn castled_bonus(&self) -> i32 {
        let castled_diff = self.castled[1] as i32 - self.castled[0] as i32;
        return castled_diff * (self.eval_params.castle * (256 - self.get_phase()) / 256)
    }

    pub fn get_all_pt_bonus(&self) -> i32 {
        let pawn_pt_bonus = self.pawn_pt_score;
        let bishop_pt_bonus = self.bishop_pt_score;
        let knight_pt_bonus = self.knight_pt_score;
        let king_mg_pt_bonus = self.king_mg_pt_score;
        let king_eg_pt_bonus = self.king_eg_pt_score;

        let phase = self.get_phase();
        let king_pt_bonus = ((phase * king_eg_pt_bonus) + ((256-phase) * king_mg_pt_bonus)) / 256;

        return pawn_pt_bonus + knight_pt_bonus + bishop_pt_bonus + king_pt_bonus;
    }

    fn get_pt_bonus(&self, bb: u64, pt: &[i32; 64], offset: i32, white: bool) -> i32 {
        let mut bonus = 0;
        let mut idx: i32 = -1;
        let mut bb = bb;
        let mut c = 0;
        while bb != 0 && c < 64 {
            c = bb.trailing_zeros() as i32 + 1;
            idx += c;
            bb = bb >> c;

            // this relies on pts being horizontally symmetric
            bonus += pt[if white {idx} else {63-idx} as usize] + offset;
        }

        return bonus;
    }

    pub fn get_pawn_pt_bonus(&self) -> i32 {
        let offset = self.eval_params.pawn_pt_offset;
        let white_bonus = self.get_pt_bonus(self.pawn[1], &board_eval::PAWN_TABLE, offset, true);
        let white_bonus = (white_bonus * self.eval_params.pawn_pt_scale) / 100;
        let black_bonus = self.get_pt_bonus(self.pawn[0], &board_eval::PAWN_TABLE, offset, false);
        let black_bonus = (black_bonus * self.eval_params.pawn_pt_scale) / 100;

        return white_bonus - black_bonus;
    }

    pub fn get_knight_pt_bonus(&self) -> i32 {
        let offset = self.eval_params.knight_pt_offset;
        let white_bonus = self.get_pt_bonus(self.knight[1], &board_eval::KNIGHT_TABLE, offset, true);
        let white_bonus = (white_bonus * self.eval_params.knight_pt_scale) / 100;
        let black_bonus = self.get_pt_bonus(self.knight[0], &board_eval::KNIGHT_TABLE, offset, false);
        let black_bonus = (black_bonus * self.eval_params.knight_pt_scale) / 100;

        return white_bonus - black_bonus;
    }

    pub fn get_bishop_pt_bonus(&self) -> i32 {
        let offset = self.eval_params.bishop_pt_offset;
        let white_bonus = self.get_pt_bonus(self.bishop[1], &board_eval::BISHOP_TABLE, offset, true);
        let white_bonus = (white_bonus * self.eval_params.bishop_pt_scale) / 100;
        let black_bonus = self.get_pt_bonus(self.bishop[0], &board_eval::BISHOP_TABLE, offset, false);
        let black_bonus = (black_bonus * self.eval_params.bishop_pt_scale) / 100;

        return white_bonus - black_bonus;
    }

    pub fn get_king_mg_pt_bonus(&self) -> i32 {
        let offset = self.eval_params.king_mg_pt_offset;
        let white_bonus = self.get_pt_bonus(self.king[1], &board_eval::KING_MG_TABLE, offset, true);
        let white_bonus = (white_bonus * self.eval_params.king_mg_pt_scale) / 100;
        let black_bonus = self.get_pt_bonus(self.king[0], &board_eval::KING_MG_TABLE, offset, false);
        let black_bonus = (black_bonus * self.eval_params.king_mg_pt_scale) / 100;

        return white_bonus - black_bonus;
    }

    pub fn get_king_eg_pt_bonus(&self) -> i32 {
        let offset = self.eval_params.king_eg_pt_offset;
        let white_bonus = self.get_pt_bonus(self.king[1], &board_eval::KING_EG_TABLE, offset, true);
        let white_bonus = (white_bonus * self.eval_params.king_eg_pt_scale) / 100;
        let black_bonus = self.get_pt_bonus(self.king[0], &board_eval::KING_EG_TABLE, offset, false);
        let black_bonus = (black_bonus * self.eval_params.king_eg_pt_scale) / 100;

        return white_bonus - black_bonus;
    }

    pub fn is_threefold(&self) -> bool {
        return self.history.iter().filter(|&n| *n == self.hash).count() >= 2;
    }

    pub fn is_repitition(&self) -> bool {
        return self.history.iter().filter(|&n| *n == self.hash).count() > 0;
    }

    // for interface
    pub fn is_ep(&self, start: i32, end: i32) -> bool {
        let start_bb = BB::idx_to_bb(start);
        if (self.pawn[self.white_turn as usize] & start_bb) != 0 {
            return end == self.ep;
        } else {
            return false;
        }
    }

    pub fn get_piece_at_idx(&self, idx: i32) -> u8 {
        let point = BB::idx_to_bb(idx);
        // pawn?
        if (point & (self.pawn[0] | self.pawn[1])) != 0 { return b'p'; }
        if (point & (self.knight[0] | self.knight[1])) != 0 { return b'n'; }
        if (point & (self.bishop[0] | self.bishop[1])) != 0 { return b'b'; }
        if (point & (self.rook[0] | self.rook[1])) != 0 { return b'r'; }
        if (point & (self.queen[0] | self.queen[1])) != 0 { return b'q'; }
        if (point & (self.king[0] | self.king[1])) != 0 { return b'k'; }
        return 0;
    }
}

#[derive(Copy, Clone)]
pub struct Mv {
    pub start: i32,
    pub end: i32,
    pub piece: u8,
    pub is_ep: bool,
    pub ep_tile: i32,
    pub promote_to: u8,
    pub is_null: bool,
    pub is_err: bool
}

impl Mv {

    pub fn get_repr(&self) -> String {
        let start = (self.start % 8, self.start >> 3);
        let end = (self.end % 8, self.end >> 3);
        let f1 = "abcdefgh".as_bytes()[start.0 as usize] as char;
        let r1 = (start.1 + 1).to_string();
        let f2 = "abcdefgh".as_bytes()[end.0 as usize] as char;
        let r2 = (end.1 + 1).to_string();
        let p = if self.promote_to != 0 {(self.promote_to as char).to_string()} else {"".to_string()};

        return format!("{}{}{}{}{}", f1.to_string(), r1, f2.to_string(), r2, p);
    }

    pub fn get_piece_num(&self) -> usize {
        match self.piece {
            b'k'=> 0,
            b'q'=> 1,
            b'r'=> 2,
            b'b'=> 3,
            b'n'=> 4,
            b'p'=> 5,
            _ => panic!("bad piece for getting num")
        }
    }

    pub fn normal_move(start: i32, end: i32, piece: u8) -> Mv {
        Mv {
            start: start,
            end: end,
            piece: piece,
            ep_tile: -1,
            is_ep: false,
            promote_to: 0,
            is_null: false,
            is_err: false
        }
    }

    pub fn pawn_ep_move(start: i32, end: i32) -> Mv {
        Mv {
            start: start,
            end: end,
            ep_tile: -1,
            piece: b'p',
            is_ep: true,
            promote_to: 0,
            is_null: false,
            is_err: false
        }
    }

    pub fn pawn_move(start: i32, end: i32) -> Mv {
        let mut ep_tile = -1;
        if (end - start).abs() == 16 {
            if start > end {
                ep_tile = start - 8;
            } else {
                ep_tile = start + 8;
            }
        }
        Mv {
            start: start,
            end: end,
            piece: b'p',
            is_ep: false,
            ep_tile: ep_tile,
            promote_to: 0,
            is_null: false,
            is_err: false
        }
    }

    pub fn pawn_promote_move(start: i32, end: i32, piece: u8) -> Mv {
        Mv {
            start: start,
            end: end,
            piece: b'p',
            is_ep: false,
            ep_tile: -1,
            promote_to: piece,
            is_null: false,
            is_err: false
        }
    }

    pub fn null_move() -> Mv {
        Mv {
            start: 0,
            end: 0,
            piece: 0,
            is_ep: false,
            ep_tile: -1,
            promote_to: 0,
            is_null: true,
            is_err: false
        }
    }

    pub fn err_move() -> Mv {
        Mv {
            start: 0,
            end: 0,
            piece: 0,
            is_ep: false,
            ep_tile: -1,
            promote_to: 0,
            is_null: false,
            is_err: true
        }
    }
}

impl fmt::Display for Mv {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_repr())
    }
}
