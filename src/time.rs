use std::cmp;

fn estimate_plies_remaining(ply: i32, material_delta: i32) -> f64 {
    // Credit to Kade Phillips for this final set of math
    // and to Thomas Ahle for a similar idea that evolved into this

    // there is no clearer variable names here because there
    // isn't really a priori reasoning to this formula
    // it is purely an attempt to fit data.
    let x = ply as f64;
    let mut m = material_delta.abs() as usize;
    if m < 4 {
        let bk1 = [ 32.0,  31.0,  34.0,  51.0];
        let bk2 = [114.0, 124.0, 121.0, 124.0];
        let a1  = [ 81.5,  80.0,  75.0,  82.0];
        let b1  = [  0.9,   1.1,  1.08,   1.0];
        let a2  = [ 65.0,  54.5,  46.5,  40.0];
        let b2  = [ 0.38,  0.28,  0.22,  0.17];
        let c   = [ 22.0,  20.0,  20.0,  19.0];
        if x >= bk2[m] {
            return c[m];
        }
        else if x >= bk1[m] {
            return a2[m] - x * b2[m];
        } else {
            return a1[m] - x * b1[m];
        }
    }

    if m > 19 { m = 19; }
    m -= 4;

    let bk = [ 0.0,  0.0,  0.0, 29.0, 21.0,  76.0,  67.0,  71.0, 57.0,  0.0, 157.0, 140.0, 138.0, 138.0, 150.0, 150.0];
    let a  = [ 0.0,  0.0,  0.0, 76.0, 88.0,  87.0,  76.0,  57.0, 46.0,  0.0,  17.0,  14.0,  12.5,  11.5,   9.5,   8.0];
    let b  = [ 0.0,  0.0,  0.0, 1.30, 1.36,  0.87,  0.81,  0.55, 0.45,  0.0,  0.07,  0.05,  0.04,  0.04,  0.03,  0.02];
    let c  = [16.0, 12.0, 16.0, 12.0, 10.5,   9.0,  10.0,  10.0,  9.0,  8.5,   6.0,   7.0,   7.0,   6.0,   5.0,   5.0];
    let d  = [64.0, 56.0, 56.0, 61.0, 89.5, 106.0, 130.0, 100.0, 86.0, 38.5,   0.0,   0.0,   0.0,   0.0,   0.0,   0.0];
    let p  = [35.0, 49.0, 35.0, 35.0, 35.0,  35.0,  28.0,  28.0, 28.0, 35.0,  35.0,  35.0,  35.0,  35.0,  35.0,  35.0];
    if x >= bk[m] {
        return c[m] + d[m] * (x * -1.0 / p[m]).exp();
    } else {
        return a[m] - x * b[m];
    }
}

pub fn get_time_bounds_clock_inc(clock: i32, inc: i32, overhead: i32, ply: i32, material: i32) -> (u128, u128) {
    let ply_remaining = estimate_plies_remaining(ply, material);
    let moves_remaining = ply_remaining / 2.;
    let est_time_per_move = (((clock - inc) as f64 / moves_remaining) + inc as f64) - overhead as f64;

    let optimum_time = cmp::min((est_time_per_move * 0.2).ceil() as u128, cmp::max(0, clock - overhead) as u128);
    let maximum_time = cmp::min((est_time_per_move * 2.0).ceil() as u128, cmp::max(0, clock - overhead) as u128);

    return (optimum_time, maximum_time);
}

pub fn get_time_bounds_moves_to_go(clock: i32, moves_to_go: i32, overhead: i32) -> (u128, u128) {
    let moves_remaining = moves_to_go as f64;
    let est_time_per_move = (clock as f64 / moves_remaining) - overhead as f64;

    let optimum_time = cmp::min((est_time_per_move * 0.2).ceil() as u128, cmp::max(0, clock - overhead) as u128);
    let maximum_time = cmp::min((est_time_per_move * 2.0).ceil() as u128, cmp::max(0, clock - overhead) as u128);

    return (optimum_time, maximum_time);
}
