# Mantissa


<p align="center">
<img src="mantissa_logo_seal_simple.png" style="width: 45%">
</p>

## About

Mantissa is a young UCI-compatible chess engine written as a side project to learn both chess programming and also rust.  Because of this, there are many things to improve both in terms of code organization and in terms of performance and correctness for Mantissa, but there are a lot of ideas packed in already.

Mantissa is named after the "significand" (or mantissa) in the floating point spec (and math in general).  Ironically there are next-to-no floats in Mantissa's code.

## Play Against Mantissa

Mantissa is hosted on lichess here: https://lichess.org/@/AKS-Mantissa

She accepts challenges from humans and bots in any time format except ultrabullet (bots aren't allowed to play ultra-bullet on lichess) or correspondence/unlimited (a few bad apples have clogged up Mantissa's match queue before by leaving once they were in a bad spot).  Mantissa accepts both casual and rated challenges.

## Strength

### Lichess
As of this writing, Mantissa is currently between 2050 and 2200 lichess ELO in most formats and is slowly getting stronger.  Lichess ELO does not seem to have a straightforward conversion into CCRL/CEGT/etc ratings, however.

### CCRL

Mantissa is currently receiving some testing from the CCRL testers.  As numbers get reported, I'll update this section to present the numbers.

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

### Engine Design

I am not very good at chess and am still relatively new to chess programming, so many of the ideas implemented in Mantissa have been very much informed by or taken from those who have come before.  I have to give a big thanks to the Chess Programming Wiki in general, which is an excellent jumping-off point for new chess programmers.  While I have looked at a lot of engines recently, there are a few I will directly credit for having been particularly influential of my understanding of chess programming:

- [Stockfish](https://github.com/official-stockfish/Stockfish)
- [Ethereal](https://github.com/AndyGrant/Ethereal)
- [CPW-Engine](https://www.chessprogramming.org/CPW-Engine)
- [Cheng-4](https://github.com/kmar/cheng4)
- [Zahak](https://github.com/amanjpro/zahak)

### Data

- Andrew Grant for some of the positions used in the tuning process for Mantissa's evaluator.
- Kade Phillips and Thomas Ahle for their two analyses of data to model moves remaining in a game.

### Other

- Kade Phillips for his work on Mantissa's logos.
