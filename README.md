# Chess Oxide

A chess engine written in Rust.

## Features

- **Move Generation**: Efficient move generation for all pieces using a mailbox system.
- **Position Representation**: Uses a 64-square array for position representation.
- **Board Representation**: Data and functions required to run a chess game.
- **Zobrist Hashing**: Implements Zobrist hashing for fast position comparison.
- **Perft Testing**: Performance testing for move generation.
- **Error Handling**: Error handling for user facing functions.
- **GUI Integration**: Basic GUI for visualizing the board and moves using Slint.
- **FEN/PGN Implementations**: Implementation of FEN and PGN standards for import/export of board states.
- **Engine**: Implemented using a negamax algorithm implementing alpha/beta pruning.
- **Transposition Table**: Implementation of a Transposition Table to use with engine.
- **Engine Debug Feature**: Enabling 'debug_engine_logging' feature gives detailed breakdown of the nodes searched in engine.
- **Logging**: Library uses 'log' crate and frontends use 'env_logger'.

## Installation

To build and run the project, ensure you have Rust installed. Clone the repository and run:

```sh
cargo build [--bin] [--release]
```

## Usage

To run the chess engine with the GUI:

```sh
cargo run --bin chess-gui [--release]
```

To run the basic performance test:
```sh
cargo run --bin chess-perft [--release]
```

Example using environment variable RUST_LOG for env_logger configuration:
```sh
RUST_LOG=debug cargo run --bin chess-gui --release
```

## License
This project is licensed under the MPL-2.0 License.
