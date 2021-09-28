use super::SquareSpec;
use crate::piece::PieceType;
use std::fmt;

/// The general type to represent moves.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Move {
    /// A "normal" move between two squares. This covers most moves,
    /// most notably en passant
    Normal { from: SquareSpec, to: SquareSpec },
    /// We have a special variant for castling as it moves two pieces
    Castling(Castling),
    /// Promotion also gets a special move, as it results in a new
    /// piece and we'd like to record the new piece type
    Promotion {
        from: SquareSpec,
        to: SquareSpec,
        target: PieceType,
    },
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Normal { from, to } => write!(f, "{}{}", from, to),
            Move::Castling(Castling::Short) => write!(f, "O-O"),
            Move::Castling(Castling::Long) => write!(f, "O-O-O"),
            Move::Promotion { from, to, target } => write!(f, "{}{}={}", from, to, target),
        }
    }
}

/// Enum for the two ways you can castle
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Castling {
    /// Castling king-side
    Short,
    /// Castling queen-side
    Long,
}
