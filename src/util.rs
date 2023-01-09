use std::{char::ParseCharError, fmt::Error};

struct FEN {
    board: [[char; 8]; 8],
    turn: char,
    castling: String,
    en_passant: String,
    halfmove: u32,
    fullmove: u32,
}

fn parse_fen(fen: &str) -> Result<FEN, &str> {
    let fen_parts: Vec<&str> = fen.split_whitespace().collect();
}