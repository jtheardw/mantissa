#! /usr/bin/env sh

export VERSION="3.7.2"

mkdir -p release

# linux AVX2
RUSTFLAGS='-C target-feature=+popcnt,+avx2' cargo rustc --release --target=x86_64-unknown-linux-gnu
cp ./target/x86_64-unknown-linux-gnu/release/mantissa release/mantissa-$VERSION-linux-avx2

# linux AVX
RUSTFLAGS='-C target-feature=+popcnt,+avx' cargo rustc --release --target=x86_64-unknown-linux-gnu
cp ./target/x86_64-unknown-linux-gnu/release/mantissa release/mantissa-$VERSION-linux-avx

# linux SSE3
RUSTFLAGS='-C target-feature=+popcnt,+sse3' cargo rustc --release --target=x86_64-unknown-linux-gnu
cp ./target/x86_64-unknown-linux-gnu/release/mantissa release/mantissa-$VERSION-linux-sse3

# linux portable
RUSTFLAGS='-C target-feature=+sse3' cargo rustc --target=x86_64-unknown-linux-gnu
cp ./target/x86_64-unknown-linux-gnu/release/mantissa release/mantissa-$VERSION-linux-sse3-no-popcnt

# Windows
# windows AVX2
RUSTFLAGS='-C target-feature=+popcnt,+avx2' cargo rustc --release --target=x86_64-pc-windows-gnu
cp ./target/x86_64-pc-windows-gnu/release/mantissa.exe release/mantissa-$VERSION-windows-avx2.exe

# windows AVX
RUSTFLAGS='-C target-feature=+popcnt,+avx' cargo rustc --release --target=x86_64-pc-windows-gnu
cp ./target/x86_64-pc-windows-gnu/release/mantissa.exe release/mantissa-$VERSION-windows-avx.exe

# windows SSE3
RUSTFLAGS='-C target-feature=+popcnt,+sse3' cargo rustc --release --target=x86_64-pc-windows-gnu
cp ./target/x86_64-pc-windows-gnu/release/mantissa.exe release/mantissa-$VERSION-windows-sse3.exe

# windows portable
RUSTFLAGS='-C target-feature=+sse3' cargo rustc --target=x86_64-pc-windows-gnu
cp ./target/x86_64-pc-windows-gnu/release/mantissa.exe release/mantissa-$VERSION-windows-sse3-no-popcnt.exe

# Android
cargo rustc --release --target=aarch64-linux-android
cp ./target/aarch64-linux-android/release/mantissa release/mantissa-$VERSION-aarch64-android
