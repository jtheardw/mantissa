use super::bb::Mv;

const PV_NODE: u8 = 1;
const CUT_NODE: u8 = 2;
const ALL_NODE: u8 = 3;

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub hash: u64,
    pub mv: Mv,
    pub node_type: u8,
    pub depth: i32,
    pub value: i32,
    pub valid: bool
}

// first entry is depth preferred, second is always replace
pub struct TT {
    pub tt: Vec<(TTEntry, TTEntry)>,
    pub bits: usize,
    pub mask: u64,
    pub valid: bool
}

impl TTEntry {
    pub fn invalid_entry() -> TTEntry {
        TTEntry {
            hash: 0,
            mv: Mv::null_move(),
            node_type: PV_NODE,
            depth: 0,
            value: 0,
            valid: false
        }
    }
}

impl TT {
    pub fn get_tt(bits: usize) -> TT {
        let mut v: Vec<(TTEntry, TTEntry)> = Vec::new();
        for _ in 0..(1 << bits) {
            v.push((TTEntry::invalid_entry(), TTEntry::invalid_entry()));
        }
        TT {
            tt: v,
            bits: bits,
            mask: (1 << bits) - 1,
            valid: true
        }
    }

    pub fn get_ptr(&self, hash: u64) -> * const i8 {
        let idx: usize = (hash & self.mask) as usize;
        return &self.tt[idx] as *const (TTEntry, TTEntry) as *const i8;
    }

    pub fn get(&self, hash: u64) -> TTEntry {
        let idx: usize = (hash & self.mask) as usize;
        let (e1, e2) = self.tt[idx];
        if e1.valid && e1.hash == hash {
            // match
            return e1;
        }
        if e2.valid && e2.hash == hash {
            return e2;
        }
        return TTEntry::invalid_entry();
    }

    pub fn set(& mut self, hash: u64, mv: Mv, value: i32, node_type: u8, depth: i32) {
        let idx: usize = (hash & self.mask) as usize;
        let (e1, e2) = self.tt[idx];
        let mut to_insert = (e1, e2);
        let entry = TTEntry {
            hash: hash,
            mv: mv,
            node_type: node_type,
            depth: depth,
            value: value,
            valid: true
        };
        if !e1.valid || e1.depth <= depth {
            // replace the depth preferred
            to_insert = (entry, e2);
        } else if hash != e1.hash {
            to_insert = (e1, entry);
        }
        self.tt[idx] = to_insert;
    }
}
