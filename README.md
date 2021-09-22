# Mantissa

<img src="mantissa_logo_seal.png" style="width: 300px">

## About

Mantissa is a young UCI-compatible chess engine written as a side project to learn both chess programming and also rust.  Because of this, there are many things to improve both in terms of code organization and in terms of performance and correctness for Mantissa, but there are a lot of ideas packed in already.

Mantissa is named after the "significand" (or mantissa) in the floating point spec (and math in general).  Ironically there are next-to-no floats in Mantissa's code.

## Play Against Mantissa

Mantissa is hosted on lichess here: https://lichess.org/@/AKS-Mantissa

She accepts challenges from humans and bots in any time format except ultrabullet (bots aren't allowed to play ultra-bullet on lichess) or correspondence/unlimited (a few bad apples have clogged up Mantissa's match queue before by leaving once they were in a bad spot).  Mantissa accepts both casual and rated challenges.

As of this writing, Mantissa is currently between 2000 and 2100 lichess ELO in most formats and is slowly getting stronger.

## Features

Mantissa is UCI-compatible and can be used with most UCI GUIs (e.g. cutechess, arena).  She supports both clock+inc time and x moves in y time formats.

In terms of search, Mantissa uses some form of the following ideas:

### Board Representation
- Magic Bitboards
- Zobrist keys used to calculate a board's hash.

### Evaluation
- Material balance
- Piece Mobility
- King Danger based on incoming attacks
- Pawn Structure
  - Isolated/Doubled/Backwards pawns
  - Passed Pawns
  - Advanced Connected Pawns
  - Space control
- Bishop Pair
- Rook positioning
  - Rook on seventh
  - Rook on (semi-)open files
- Piece Square Tables
- Pawn hash tables
- Tapered Evaluation

### Search Basics
- Negamax search with alpha-beta pruning
- Principal Variation (PV) and Zero-window search
- Quiescence search in the leaves
- Aspiration Windows
- Transposition Table

### Selectivity
- Null Move Pruning (NMP)
- Late Move Reduction (LMR)
- Late Move Pruning (LMP)
- Futility Pruning
- Reverse Futility Pruning
- Singular Move Extensions
- Multi-cut
- Delta Pruning
- SEE Pruning in Quiescence search

### Move Ordering
- Undefended bonus
- Victim-Attacker relative value
- Killer heuristic
- History tables
- Countermove heuristic
- Followup-history tables

## Build

In order to build Mantissa, you'll need rust installed.  From there, you can use whatever build method is easiest for you.  Typically, this is navigating to the directory of the project (in this case, Mantissa), and running `cargo build --release`.

I also provide some build scripts located in the main directory.  They attempt to build with certain optimizations, most notably using the `popcnt` instruction if available.  At the moment, all the build scripts were written with a unix-like shell in mind (including the `build_windows` script, which I use for cross-compiling), but I'll be adding windows cmd line compatible ones soon.

NOTE: If you're having trouble installing the Windows build using the `build_windows` script, because you don't have the `popcnt` instruction or any other reason, use the `build_portable` script instead.

## Credit

- The chess programming community for many of the ideas used in Mantissa.
- Kade Phillips for the logo, optimization code in the tuner, and the "plies remaining" estimation code for Mantissa's time control.
- Andy Grant (Ethereal creator) for some of the positions used in tuning Mantissa's evaluator, as well as a lot of ideas found pouring through Ethereal's source.
