# Chess Oxide

A chess engine written in Rust.

## Features

- **Move Generation**: Efficient move generation for all pieces using mailbox notation.
- **Board Representation**: Uses a 64-square array board representation.
- **Zobrist Hashing**: Implements Zobrist hashing for fast position comparison.
- **Perft Testing**: Performance testing for move generation.
- **Error Handling**: Error handling for user facing functions.
- **GUI Integration**: Basic GUI for visualizing the board and moves using Slint.

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

NOTE: GUI IS VERY BASIC AND JUST USED FOR TESTING

## License
This project is licensed under the MPL-2.0 License.