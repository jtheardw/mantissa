// Credit to Kade Phillips, Don Knuth

static mut RAND_STATE : u64 = 0;

pub fn set_rand(ofs : u64)
{
  // This is an arbitrary number (randomly chosen).
  //
  unsafe { RAND_STATE = ofs.wrapping_add(1761173596); }
  for _ in 0..4 { rand(); }
}

pub fn reset_rand() { set_rand(0); }

pub fn rand() -> u32
{
  let a : u64 = 8093;
  let m : u64 = 0x0001000000000000;
  let c : u64 = 1;

  unsafe { RAND_STATE = RAND_STATE.wrapping_mul(a).wrapping_add(c) % m; }
  return (unsafe{RAND_STATE} >> 16) as u32;
}

// 0.0 to +1.0
//
pub fn unif() -> f64
{
  return rand() as f64 / 4294967295.0;
}

// -1.0 to +1.0
//
pub fn symunif() -> f64
{
  return (rand() as f64 / 2147483647.5) - 1.0;
}

// -1.0 to +1.0 but PDF is triangular not constant
//
pub fn triang() -> f64
{
  let x = symunif();
  return if x >= 0.0 {
    1.0 - (1.0 - x).sqrt()
  }
  else {
    (1.0 + x).sqrt() - 1.0
  };
}

// Randomly select k numbers from 0..N
//
#[allow(non_snake_case)]
pub fn choose(k : usize, N : usize) -> Vec<usize>
{
  debug_assert!(N >= k, "cannot choose {} from 0..{}", k, N);

  let mut sel = Vec::new();
  sel.reserve(N);
  for x in 0..N { sel.push(x); }

  for x in 0..k {
    let idx = rand() as usize % (N - x);
    sel.swap(x, x + idx);
  }

  sel.truncate(k);
  return sel;
}
