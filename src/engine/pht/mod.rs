#[derive(Copy, Clone)]
pub struct PHTEntry {
    pub hash: u64,
    pub mg_val: i32,
    pub eg_val: i32,
    pub valid: bool
}

pub struct PHT {
    pub pht: Vec<PHTEntry>,
    pub bits: usize,
    pub mask: u64,
    pub valid: bool
}

impl PHTEntry {
    pub fn invalid_entry() -> PHTEntry {
        PHTEntry {
            hash: 0,
            mg_val: 0,
            eg_val: 0,
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
            mask: (1 << bits) - 1,
            valid: true
        }
    }

    pub fn get_ptr(&self, hash: u64) -> * const i8 {
        let idx: usize = (hash & self.mask) as usize;
        return &self.pht[idx] as *const PHTEntry as *const i8;
    }

    pub fn get(&self, hash: u64) -> PHTEntry {
        let idx: usize = (hash & self.mask) as usize;
        let e = self.pht[idx];
        if !e.valid { return e; }
        // if e.valid && e.hash == hash { return e; }
        return if e.hash == hash {e} else {PHTEntry::invalid_entry()};
    }

    pub fn set(& mut self, hash: u64, mg_val: i32, eg_val: i32) {
        let idx: usize = (hash & self.mask) as usize;
        self.pht[idx] = PHTEntry {
            hash: hash,
            mg_val: mg_val,
            eg_val: eg_val,
            valid: true
        };
    }
}
