use std::mem;

use crate::moveutil::*;
use crate::util::*;

pub static mut TT: TT = TT {tt: Vec::new(), bits: 0, mask: 0};

pub fn allocate_tt(size_mb: usize) {
    let entry_size = mem::size_of::<TTEntry>() * 2;
    let mut pow = 1;
    while (entry_size << pow) <= (size_mb * 1024 * 1024) {
        pow += 1;
    }
    // afterwards we know the right power is one less than that.
    unsafe {
        TT = TT::new(pow - 1);
    }
}

fn age_threshold(old_depth: i32, new_depth: i32) -> i32 {
    // how old can an entry be and still not be replaced by an entry with lower depth
    return (old_depth - new_depth) << 2;
}

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub hash: u64,
    pub mv: Move,
    pub node_type: u8,
    pub depth: i32,
    pub value: i32,
    pub ply: i32,
    pub valid: bool
}

pub struct TT {
    pub tt: Vec<(TTEntry, TTEntry)>,
    pub bits: usize,
    pub mask: u64
}

impl TTEntry {
    pub fn invalid_entry() -> TTEntry {
        TTEntry {
            hash: 0,
            mv: Move::null_move(),
            node_type: PV_NODE,
            depth: 0,
            value: 0,
            ply: 0,
            valid: false
        }
    }

    pub fn make_tt_score(score: i32, ply: i32) -> i32 {
        if score > MATE_SCORE - 100000 {
            score + ply
        } else if score < (-MATE_SCORE + 100000) {
            score - ply
        } else {
            score
        }
    }

    pub fn read_tt_score(score: i32, ply: i32) -> i32 {
        if score > MATE_SCORE - 100000 {
            score - ply
        } else if score < (-MATE_SCORE + 100000) {
            score + ply
        } else {
            score
        }
    }
}

impl TT {
    pub fn new(bits: usize) -> TT {
        let mut v: Vec<(TTEntry, TTEntry)> = Vec::new();
        for _ in 0..(1 << bits) {
            v.push((TTEntry::invalid_entry(), TTEntry::invalid_entry()));
        }
        TT {
            tt: v,
            bits: bits,
            mask: (1 << bits) - 1,
        }
    }

    pub fn get(&self, hash: u64) -> TTEntry {
        let idx: usize = (hash & self.mask) as usize;
        let (e1, e2) = self.tt[idx];
        if e1.valid && e1.hash == hash {
            return e1;
        }
        if e2.valid && e2.hash == hash {
            return e2
        }
        return TTEntry::invalid_entry();
    }

    pub fn set(&mut self, hash: u64, mv: Move, value: i32, node_type: u8, depth: i32, ply: i32) {
        let idx: usize = (hash & self.mask) as usize;
        let (e1, e2) = self.tt[idx];
        let mut to_insert = (e1, e2);

        let entry = TTEntry {
            hash: hash,
            mv: mv,
            node_type: node_type,
            depth: depth,
            value: value,
            ply: ply,
            valid: true
        };

        if !e1.valid || e1.depth <= depth || (ply - e1.ply) > age_threshold(e1.depth, depth) {
            // first bucket is depth-preferred (though ages out)
            to_insert = (entry, e2);
        } else if hash != e1.hash {
            to_insert = (e1, entry);
        }
        self.tt[idx] = to_insert;
    }
}
