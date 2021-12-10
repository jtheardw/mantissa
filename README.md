# Mantissa

<p align="center">
<img src="logos/mantissa_logo.png" style="width: 45%">
</p>

## About

Mantissa is a young UCI-compatible chess engine written as a side project to learn both chess programming and also rust.  Because of this, there are many things to improve both in terms of code organization and in terms of performance and correctness for Mantissa, but there are a lot of ideas packed in already.

Mantissa is named after the "significand" (or mantissa) in the floating point spec (and math in general).  Ironically there are next-to-no floats in Mantissa's code.

## Play Against Mantissa

Mantissa is hosted on lichess here: https://lichess.org/@/AKS-Mantissa

She accepts challenges from humans and bots in any time format except ultrabullet (bots aren't allowed to play ultra-bullet on lichess) or correspondence/unlimited (a few bad apples have clogged up Mantissa's match queue before by leaving once they were in a bad spot).  Mantissa accepts both casual and rated challenges.

## Strength

### Lichess
As of this writing, Mantissa is currently between 2050 and 2300 lichess ELO in most formats and is slowly getting stronger.  Lichess ELO does not seem to have a straightforward conversion into CCRL/CEGT/etc ratings, however.

### CCRL

Mantissa currently holds a blitz ELO rating of 2686 on CCRL (as of version 2.1.1).  This score may fluctuate in the future, as this is its first entry on the list.

## Features

Mantissa is UCI-compatible and can be used with most UCI GUIs (e.g. cutechess, arena).  She supports both clock+inc time and x moves in y time formats.

In terms of search, Mantissa uses some form of the following ideas:

### Board Representation
- Magic Bitboards
- Zobrist keys used to calculate a board's hash.

### Handwritten Evaluation
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

### NNUE
- 769 -> 128 -> 1 Simple architecture trained on Mantissa self-play games

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

## Build

**For the current version (3.0.0), Mantissa is only compatible with `x86_64` hardware.  This is my first experience working with NNUE so my ability to have it be efficient is fairly limited.  I'll be researching ways to increase portability while keeping strength as I become more familiar.  For now, if you're trying to run Mantissa on non `x86_64` hardware, consider the 2.5.0 release.  I apologize for any inconvenience**

In order to build Mantissa, you'll need rust installed.  From there, you can use whatever build method is easiest for you.  Typically, this is navigating to the directory of the project (in this case, Mantissa), and running `RUSTFLAGS='-C target-cpu=native' cargo build --release`.

I also provide some build scripts located in the main directory.  They attempt to build with certain optimizations, most notably using the `popcnt` instruction if available.  At the moment, all the build scripts were written with a unix-like shell in mind (including the `build_windows` script, which I use for cross-compiling), but I'll be adding windows cmd line compatible ones soon.

NOTE: If you're having trouble installing the Windows build using the `build_windows` script, because you don't have the `popcnt` instruction or any other reason, use the `build_portable` script instead.  I encourage you to use the normal windows build if possible though, as the difference in playing strength is noticeable

## Credit

### Engine Design

I am not very good at chess and am still relatively new to chess programming, so many of the ideas implemented in Mantissa have been very much informed by or taken from those who have come before.  I have to give a big thanks to the Chess Programming Wiki in general, which is an excellent jumping-off point for new chess programmers.  While I have looked at a lot of engines recently, there are a few I will directly credit for having been particularly influential of my understanding of chess programming:

- [Stockfish](https://github.com/official-stockfish/Stockfish)
- [Ethereal](https://github.com/AndyGrant/Ethereal)
- [CPW-Engine](https://www.chessprogramming.org/CPW-Engine)
- [Cheng-4](https://github.com/kmar/cheng4)
- [Zahak](https://github.com/amanjpro/zahak)

### Data

- Andrew Grant for some of the positions used in the tuning process for Mantissa's handwritten evaluator.
- Kade Phillips and Thomas Ahle for their two analyses of data to model moves remaining in a game.

### NNUE

- Amanj was incredibly helpful in getting me started with NNUE.  `tissa-trainer`, which is the program I wrote to be able to train Mantissa nets, is basically a port of `zahak-trainer` to rust with some slight modifications.  He also helped explain a lot of the resources and information needed to understand the process.

### Other

- Kade Phillips for his work on Mantissa's logos and for the net visualization code.
