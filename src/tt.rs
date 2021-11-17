use std::mem;
use std::sync::Mutex;

use crate::moveutil::*;
use crate::util::*;

pub static mut TT: TT = TT {tt: Vec::new(), bits: 0, mask: 0, locks: Vec::new()};

pub fn allocate_tt(size_mb: usize) {
    let entry_size = mem::size_of::<TTEntry>() * 2;
    let mut pow = 1;
    while (entry_size << pow) <= (size_mb * 1024 * 1024) {
        pow += 1;
    }
    // afterwards we know the right power is one less than that.
    unsafe {
        TT = TT::new(0);
        TT = TT::new(pow - 1);
    }
}

pub fn clear_tt() {
    unsafe {
        TT.clear();
    }
}

fn age_threshold(old_depth: i8, new_depth: i8) -> i32 {
    // how old can an entry be and still not be replaced by an entry with lower depth
    return ((old_depth - new_depth) as i32) << 4;
}

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub hash: u64,
    pub mv: Move,
    pub node_type: u8,
    pub depth: i8,
    pub value: i32,
    pub ply: i8
} // 8 + 4 + 1 + 1 + 4 + 1 = 19 bytes

pub struct TT {
    pub tt: Vec<(TTEntry, TTEntry)>,
    pub bits: usize,
    pub mask: u64,
    locks: Vec<Mutex<u64>>
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
        }
    }

    pub fn valid(&self) -> bool {
        self.hash != 0 && !self.mv.is_null()
    }

    pub fn make_tt_score(score: i32, ply: i32) -> i32 {
        if score > MATE_SCORE - 100000 {
            return score + ply;
        } else if score < (-MATE_SCORE + 100000) {
            return score - ply;
        } else {
            return score;
        }
    }

    pub fn read_tt_score(score: i32, ply: i32) -> i32 {
        if score > MATE_SCORE - 100000 {
            return score - ply;
        } else if score < (-MATE_SCORE + 100000) {
            return score + ply;
        } else {
            return score
        }
    }
}

impl TT {
    pub fn new(bits: usize) -> TT {
        let mut v: Vec<(TTEntry, TTEntry)> = Vec::new();
        for _ in 0..(1 << bits) {
            v.push((TTEntry::invalid_entry(), TTEntry::invalid_entry()));
        }

        let mut locks: Vec<Mutex<u64>> = Vec::new();
        for _ in 0..1024 {
            locks.push(Mutex::new(0));
        }
        TT {
            tt: v,
            bits: bits,
            mask: (1 << bits) - 1,
            locks: locks
        }
    }

    pub fn get(&self, hash: u64) -> TTEntry {
        let mut l = self.locks[(hash % 1024) as usize].lock().unwrap();
        let idx: usize = (hash & self.mask) as usize;
        let (e1, e2) = self.tt[idx];
        *l = hash | e1.hash | e2.hash;
        if e1.valid() && e1.hash == hash {
            return e1;
        }
        if e2.valid() && e2.hash == hash {
            return e2
        }
        return TTEntry::invalid_entry();
    }

    pub fn set(&mut self, hash: u64, mv: Move, value: i32, node_type: u8, depth: i32, ply: i32) {
        let idx: usize = (hash & self.mask) as usize;
        let depth = depth as i8;
        let ply = ply as i8;
        let mut l = self.locks[(hash % 1024) as usize].lock().unwrap();
        let (e1, e2) = self.tt[idx];
        let to_insert;

        let entry = TTEntry {
            hash: hash,
            mv: mv,
            node_type: node_type,
            depth: depth as i8,
            value: value,
            ply: ply as i8,
        };

        if e1.valid() && e1.hash == hash {
            // always replace with more recent search of the same position
            to_insert = (entry, e2);
        } else if e2.valid() && e2.hash == hash {
            to_insert = (e1, entry);
        } else if !e1.valid() || e1.depth <= depth || (ply - e1.ply) as i32 > age_threshold(e1.depth, depth) {
            // first bucket is depth-preferred (though ages out)
            to_insert = (entry, e2);
        } else {
            to_insert = (e1, entry);
        }
        self.tt[idx] = to_insert;
        *l = hash | to_insert.0.hash | to_insert.1.hash;
    }

    pub fn clear(&mut self) {
        for i in 0..self.tt.len() {
            self.tt[i] = (TTEntry::invalid_entry(), TTEntry::invalid_entry());
        }
    }
}
