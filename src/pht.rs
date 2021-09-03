use crate::eval::*;
use crate::util::*;

pub static mut PHT: PHT = PHT {pht: Vec::new(), bits: 0, mask: 0};

#[derive(Copy, Clone)]
pub struct PHTEntry {
    pub hash: u64,
    pub value: Score,
    pub valid: bool
}

pub struct PHT {
    pub pht: Vec<PHTEntry>,
    pub bits: usize,
    pub mask: u64,
}

impl PHTEntry {
    pub fn invalid_entry() -> PHTEntry {
        PHTEntry {
            hash: 0,
            value: 0,
            valid: false
        }
    }
}

impl PHT {
    pub fn get_pht(bits: usize) -> PHT {
        let mut v: Vec<PHTEntry> = Vec::new();
        for _ in 0..(1 << bits) {
            v.push(PHTEntry::invalid_entry());
        }
        PHT {
            pht: v,
            bits: bits,
            mask: (1 << bits) - 1
        }
    }

    pub fn get(&self, hash: u64) -> PHTEntry {
        let idx: usize = (hash & self.mask) as usize;
        let e = self.pht[idx];
        if !e.valid { return e; }
        return if e.hash == hash {e} else {PHTEntry::invalid_entry()};
    }

    pub fn set(& mut self, hash: u64, value: Score) {
        let idx: usize = (hash & self.mask) as usize;
        self.pht[idx] = PHTEntry {
            hash: hash,
            value: value,
            valid: true
        };
    }
}

pub fn initialize_pht() {
    unsafe {
        PHT = PHT::get_pht(18);
    }
}
