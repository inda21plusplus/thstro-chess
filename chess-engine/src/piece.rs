//! This module contains definitions and helper methods for pieces and their related data
use std::fmt;

/// The general piece type
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    /// Which kind of piece this is
    pub piece: PieceType,
    /// What color this piece is
    pub color: Color,
}

impl Piece {
    /// Creates a new Piece
    pub fn new(piece: PieceType, color: Color) -> Piece {
        Piece { piece, color }
    }
}

/// The different kinds of pieces representable in this backend
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum PieceType {
    Pawn,
    Rook,
    Bishop,
    Queen,
    Knight,
    King,
}

/// Enum representing the two colors in chess
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Get the opposite color
    /// ```
    /// # use chess_engine::piece::Color;
    /// assert_eq!(Color::White, Color::Black.opposite())
    /// ```
    pub fn opposite(&self) -> Color {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Gets the "board index" corresponding to the home rank of the
    /// color. I.e., a number intended to be used in a
    /// [`SquareSpec`]. See [`SquareSpec`]'s documentation for more
    /// information. Returns 0 for white and 7 for black.
    pub fn home_rank(&self) -> u32 {
        match *self {
            Color::White => 0,
            Color::Black => 7,
        }
    }

    /// Gets the board index corresponding to the color's pawn rank.
    /// Returns 1 for white and 6 for black.
    pub fn pawn_home_rank(&self) -> u32 {
        match *self {
            Color::White => 1,
            Color::Black => 6,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = format!("{}", self.piece);
        if self.color == Color::Black {
            s = s.to_lowercase();
        }
        write!(f, "{}", s)
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PieceType::*;
        write!(
            f,
            "{}",
            match self {
                Pawn => "P",
                Rook => "R",
                Bishop => "B",
                Queen => "Q",
                Knight => "N",
                King => "K",
            }
        )
    }
}

impl std::str::FromStr for PieceType {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> Result<PieceType, crate::error::Error> {
        use PieceType::*;
        Ok(match s {
            "P" => Pawn,
            "R" => Rook,
            "B" => Bishop,
            "Q" => Queen,
            "N" => Knight,
            "K" => King,
            _ => return Err(crate::error::Error::InvalidPiece(s.to_string())),
        })
    }
}
