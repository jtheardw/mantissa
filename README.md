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
As of this writing, Mantissa is currently between 2300 and 2550 lichess Elo in most formats and is slowly getting stronger.  Lichess Elo does not seem to have a straightforward conversion into CCRL/CEGT/etc ratings, however, due to several reasons, most notably the wall of Stockfish Clones around 2600 Elo.

### Rating Lists

All ratings listed her are for the 1CPU version.

| **Version** | **CCRL Blitz (40/4)** | **CCRL 40/40** | **CEGT 40/4** | **CEGT 40/20** | **SPCC** |
|-------------|-----------------------|----------------|---------------|----------------|----------|
| 3.7.2       | 3270                  | N/a            | ~3186         | 3193           | 3281     |
| 3.3.0       | 3115                  | 3093           | 2993          | 3044           | N/a      |
| 3.0.0       | 2999                  | 2996           | 2877          | N/a            | N/a      |
| 2.5.0       | N/a                   | N/a            | N/a           | N/a            | N/a      |
| 2.1.x       | 2685                  | N/a            | N/a           | N/a            | N/a      |

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
- 768 -> 256 x 2 -> 1 Simple architecture trained on Mantissa self-play games based on a combination of Zahak and Koivisto topologies

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
- Countermove History
- Followup History
- Countermove heuristic

## Build

In order to build Mantissa, you'll need rust nightly > 1.63.0 installed.  From there, you can use whatever build method is easiest for you.  Typically, this is navigating to the directory of the project (in this case, Mantissa), and running `RUSTFLAGS='-C target-cpu=native' cargo build --release`.

## Credit

### Engine Design

I am not very good at chess and am still relatively new to chess programming, so many of the ideas implemented in Mantissa have been very much informed by or taken from those who have come before.  I have to give a big thanks to the Chess Programming Wiki in general, which is an excellent jumping-off point for new chess programmers.  While I have looked at a lot of engines recently, there are a few I will directly credit for having been particularly influential of my understanding of chess programming:

- [Stockfish](https://github.com/official-stockfish/Stockfish)
- [Ethereal](https://github.com/AndyGrant/Ethereal)
- [CPW-Engine](https://www.chessprogramming.org/CPW-Engine)
- [Cheng-4](https://github.com/kmar/cheng4)
- [Zahak](https://github.com/amanjpro/zahak)
- [Asymptote](https://github.com/malu/asymptote)

### Data

- Andrew Grant for some of the positions used in the tuning process for Mantissa's handwritten evaluator.
- Kade and Thomas Ahle for their two analyses of data to model moves remaining in a game.

### NNUE

- Amanj was incredibly helpful in getting me started with NNUE.  `tissa-trainer`, which is the program I wrote to be able to train Mantissa nets, is basically a port of `zahak-trainer` to rust with some slight modifications.  He also helped explain a lot of the resources and information needed to understand the process.  `tissa-trainer` has since been changed ot train a different type of net, but still has many roots there.
- Koivisto's authors, as I found a way to improve my NNUE topology from what I found in Koivisto's source.
- Kade for net visualization code and the NNUE-derived piece value display

### Other

- Kade for his work on Mantissa's logos.
