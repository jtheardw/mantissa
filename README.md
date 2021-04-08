# Mantissa

Mantissa is a young UCI-compatible chess engine written as a side project to learn both chess programming and also rust.  Because of this, there are many things to improve both in terms of code clarity and in terms of performance and correctness for Mantissa, but there are a lot of ideas packed in already.

Mantissa is named after the "significand" in mathematics but especially as a reference to the floating point spec.  Ironically there are next-to-no floats in Mantissa's code.

## Play Against Mantissa

Mantissa is hosted on lichess here: https://lichess.org/@/AKS-Mantissa

It accepts challenges from humans and bots in any time format except ultrabullet (bots aren't allowed) or correspondence (a few bad apples have clogged up Mantissa's match queue before by leaving once they were in a bad spot).  It accepts both casual and rated challenges.

As of this writing it's currently between 1900 and 2000 ELO in most formats and is slowly getting stronger.

## Features

A few nights obsessively pouring over the chessprogramming wiki (https://www.chessprogramming.org/Main_Page) have imbued Mantissa with a number of features, including many staples like:  Late Move Reductions, Principal Variation Search, Transposition Tables, and so on...

Many of these are currently fairly unsophisticated applications of the concepts.  They will see improvements in the coming future.

Mantissa can be used with any UCI interface in theory.  It currently doesn't support much in terms of customization, that is to say that it mostly ignores "setoption".

## Build

You can build Mantissa so long as you have rust installed with `cargo build --release` (for the optimized version).  If you have a machine with native popcount and bitscan/tzcnt/etc. instructions (most x86-64 machines qualify these days), you can compile with:

```
RUSTFLAGS='-C target-cpu=native' cargo build --release
```

for a major performance boost.  As I get more familiar with the Cargo build system I'll try to automate this step.
