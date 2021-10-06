//! General errors that can happen by the chess engine
use crate::board::Move;
use std::io;
use thiserror::Error;

/// The general error type
#[derive(Error, Debug)]
pub enum Error {
    /// Error for illegal moves
    #[error("The move {1} is illegal for the board {0}")]
    IllegalMove(String, Move),
    /// Error for if a string wasn't an valid square
    #[error("`{0}` is not a valid square coordinate")]
    InvalidSquare(String),
    /// Error for trying to parse erroneous FEN
    #[error("`{0}` is invalid FEN")]
    InvalidFen(String),
    /// Error for parsing an invalid piece
    #[error("`{0}` is not a valid piece designator")]
    InvalidPiece(String),
    /// Error for generic IO errors
    #[error(transparent)]
    Io(#[from] io::Error),
}
