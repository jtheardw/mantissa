set RUSTFLAGS=-C target-feature=+popcnt,+avx,+avx2,+sse3
cargo rustc --release --target=x86_64-pc-windows-gnu
