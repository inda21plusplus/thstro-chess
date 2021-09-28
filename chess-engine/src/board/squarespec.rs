use crate::error::Error;
use std::fmt;
use std::ops;

/// A struct representing a particular square on the board
/// ```
/// # use chess_engine::board::SquareSpec;
/// let a1 = SquareSpec { rank: 0, file: 0 };
/// assert_eq!(a1, "a1".parse::<SquareSpec>().unwrap());
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SquareSpec {
    /// The rank of this square, with 0 being rank 1, and so on
    pub rank: u32,
    /// The file of this square, with 0 being the "a" file, etc.
    pub file: u32,
}

impl SquareSpec {
    /// Create a new [`SquareSpec`]
    pub fn new(rank: u32, file: u32) -> SquareSpec {
        SquareSpec { rank, file }
    }

    /// Checked addition with a [`SquareDiff`], making sure that the
    /// result remains in bounds.
    ///
    /// # Examples
    /// ```
    /// # use chess_engine::board::{SquareDiff, SquareSpec};
    /// let diff = SquareDiff { d_rank: -1, d_file: -1 };
    /// let a1 = "a1".parse::<SquareSpec>().unwrap();
    /// assert!(a1.checked_add(diff).is_none());
    /// ```
    /// ```
    /// # use chess_engine::board::{SquareDiff, SquareSpec};
    /// let diff = SquareDiff { d_rank: 1, d_file: 1 };
    /// let h8 = "h8".parse::<SquareSpec>().unwrap();
    /// assert!(h8.checked_add(diff).is_none());
    /// ```
    /// ```
    /// # use chess_engine::board::{SquareDiff, SquareSpec};
    /// let diff = SquareDiff { d_rank: 1, d_file: 1 };
    /// let e5 = "e5".parse::<SquareSpec>().unwrap();
    /// let f6 = "f6".parse::<SquareSpec>().unwrap();
    /// assert_eq!(e5.checked_add(diff), Some(f6));
    /// ```
    pub fn checked_add(self, rhs: SquareDiff) -> Option<SquareSpec> {
        use std::convert::TryInto;

        let rank = (self.rank as i32 + rhs.d_rank).try_into().ok()?;
        let file = (self.file as i32 + rhs.d_file).try_into().ok()?;

        if rank > 7 || file > 7 {
            return None;
        }

        Some(SquareSpec { rank, file })
    }
}

/// A struct representing a difference between two squares, mainly
/// created as [`SquareSpec`] can't contain negative numbers
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SquareDiff {
    /// The rank difference
    pub d_rank: i32,
    /// The file difference
    pub d_file: i32,
}

fn sgn(x: i32) -> i32 {
    match x {
        i32::MIN..=-1 => -1,
        0 => 0,
        1..=i32::MAX => 1,
    }
}

impl SquareDiff {
    /// Creates a new [`SquareDiff`]
    pub fn new(d_rank: i32, d_file: i32) -> SquareDiff {
        SquareDiff { d_rank, d_file }
    }

    /// Get the absolute of this [`SquareDiff`]
    ///
    /// # Example
    /// ```
    /// # use chess_engine::board::{SquareDiff, SquareSpec};
    /// let diff = SquareDiff { d_rank: -1, d_file: -1 };
    ///
    /// assert_eq!(diff.abs(), SquareDiff { d_rank: 1, d_file: 1 });
    /// ```
    pub fn abs(self) -> SquareDiff {
        let d_rank = self.d_rank.abs();
        let d_file = self.d_file.abs();

        SquareDiff { d_rank, d_file }
    }

    /// Check if this [`SquareDiff`] represents a diagonal
    ///
    /// # Example
    /// ```
    /// # use chess_engine::board::{SquareDiff, SquareSpec};
    /// let a1 = "a1".parse::<SquareSpec>().unwrap();
    /// let h8 = "h8".parse::<SquareSpec>().unwrap();
    ///
    /// assert!((a1 - h8).is_diag());
    /// ```
    pub fn is_diag(&self) -> bool {
        let abs = self.abs();
        abs.d_rank == abs.d_file
    }

    /// Get the [`SquareDiff`] as a unit "vector", i.e. a one square
    /// difference in any direction. Returns [`None`] if it's impossible
    /// to construct an integer unit "vector".
    /// ```
    /// # use chess_engine::board::{SquareDiff, SquareSpec};
    /// let a1 = "a1".parse::<SquareSpec>().unwrap();
    /// let h8 = "h8".parse::<SquareSpec>().unwrap();
    /// let diff = h8 - a1;
    ///
    /// assert_eq!(diff.as_unit(), Some(SquareDiff { d_rank: 1, d_file: 1 }));
    /// ```
    /// ```
    /// # use chess_engine::board::{SquareDiff, SquareSpec};
    /// let a1 = "a1".parse::<SquareSpec>().unwrap();
    /// let h7 = "h7".parse::<SquareSpec>().unwrap();
    /// let diff = h7 - a1;
    ///
    /// assert!(diff.as_unit().is_none());
    /// ```
    pub fn as_unit(self) -> Option<SquareDiff> {
        match (self.d_rank, self.d_file) {
            (d_rank, d_file) if d_rank.abs() == d_file.abs() => Some(SquareDiff {
                d_rank: sgn(d_rank),
                d_file: sgn(d_file),
            }),
            (d_rank, 0) => Some(SquareDiff {
                d_rank: sgn(d_rank),
                d_file: 0,
            }),
            (0, d_file) => Some(SquareDiff {
                d_rank: 0,
                d_file: sgn(d_file),
            }),
            _ => None,
        }
    }
}

impl Default for SquareDiff {
    fn default() -> SquareDiff {
        SquareDiff {
            d_rank: 0,
            d_file: 0,
        }
    }
}

impl ops::Sub<Self> for SquareSpec {
    type Output = SquareDiff;

    fn sub(self, rhs: SquareSpec) -> SquareDiff {
        SquareDiff {
            d_rank: self.rank as i32 - rhs.rank as i32,
            d_file: self.file as i32 - rhs.file as i32,
        }
    }
}

impl ops::Add<SquareDiff> for SquareSpec {
    type Output = SquareSpec;

    fn add(self, rhs: SquareDiff) -> SquareSpec {
        SquareSpec {
            rank: (self.rank as i32 + rhs.d_rank) as u32,
            file: (self.file as i32 + rhs.d_file) as u32,
        }
    }
}

impl ops::Add<SquareDiff> for SquareDiff {
    type Output = SquareDiff;

    fn add(self, rhs: SquareDiff) -> SquareDiff {
        SquareDiff {
            d_rank: self.d_rank + rhs.d_rank,
            d_file: self.d_file + rhs.d_file,
        }
    }
}

impl ops::AddAssign<SquareDiff> for SquareSpec {
    fn add_assign(&mut self, rhs: SquareDiff) {
        *self = *self + rhs;
    }
}

impl fmt::Display for SquareSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            match self.file {
                x @ 0..=7 => (x as u8 + b'a') as char,
                _ => '?',
            },
            match self.rank {
                x @ 0..=7 => (x as u8 + b'1') as char,
                _ => '?',
            }
        )
    }
}

impl std::str::FromStr for SquareSpec {
    type Err = Error;
    fn from_str(s: &str) -> Result<SquareSpec, Error> {
        let mut chars = s.chars();
        let file = match chars.next() {
            Some(c @ 'a'..='h') => c as u32 - 'a' as u32,
            _ => return Err(Error::InvalidSquare(s.to_string())),
        };
        let rank = match chars.next() {
            Some(c @ '1'..='8') => c as u32 - '1' as u32,
            _ => return Err(Error::InvalidSquare(s.to_string())),
        };
        if chars.next().is_some() {
            Err(Error::InvalidSquare(s.to_string()))
        } else {
            Ok(SquareSpec { rank, file })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Board, SquareSpec};
    use crate::piece::{Color, Piece, PieceType};

    #[test]
    fn parsing_works() {
        let constructed = SquareSpec { rank: 0, file: 0 };
        let parsed = "a1".parse::<SquareSpec>().unwrap();

        assert_eq!(constructed, parsed);
    }

    #[test]
    fn printing_works() {
        let constructed = "a1";
        let printed = format!("{}", SquareSpec { rank: 0, file: 0 });

        assert_eq!(&printed, constructed);
    }

    #[test]
    fn squarespec_refers_to_right_square() {
        let d8 = SquareSpec { rank: 7, file: 3 };

        assert_eq!(d8, "d8".parse::<SquareSpec>().unwrap());

        let default_board = Board::default_board();

        assert_eq!(
            default_board[d8],
            Some(Piece {
                piece: PieceType::Queen,
                color: Color::Black
            })
        );
    }

    #[test]
    fn parse_printed_is_noop() {
        let constructed = SquareSpec { rank: 0, file: 0 };
        let parsed = format!("{}", constructed).parse::<SquareSpec>().unwrap();

        assert_eq!(constructed, parsed);
    }
}
