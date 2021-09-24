//! This module contains the board and all related structs
use crate::error::Error;
use crate::piece::{Color, Piece, PieceType};
use bitflags::bitflags;
use std::fmt;

mod fen_parser;
mod legal_moves;
mod move_types;
mod squarespec;

pub use move_types::{Castling, Move};
pub use squarespec::{SquareDiff, SquareSpec};

bitflags! {
    /// [bitflags] struct
    pub struct CastlingFlags: u32 {
        #[allow(missing_docs)]
        const WHITE_SHORT = 0b0000_0001;
        #[allow(missing_docs)]
        const WHITE_LONG  = 0b0000_0010;
        #[allow(missing_docs)]
        const WHITE       = 0b0000_0011;
        #[allow(missing_docs)]
        const BLACK_SHORT = 0b0000_0100;
        #[allow(missing_docs)]
        const BLACK_LONG  = 0b0000_1000;
        #[allow(missing_docs)]
        const BLACK       = 0b0000_1100;

        #[allow(missing_docs)]
        const SHORT       = 0b0000_0101;
        #[allow(missing_docs)]
        const LONG        = 0b0000_1010;
        #[allow(missing_docs)]
        const DEFAULT     = 0b0000_1111;
    }
}

/// A struct containing all the information required to represent a position
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Board {
    board: [[Option<Piece>; 8]; 8],
    turn: Color,
    castling: CastlingFlags,
    en_passant: Option<SquareSpec>,
    halfmove: u32,
    fullmove: u32,
}

impl Board {
    /// Create a new empty `Board`
    pub fn new(turn: Color, castling: CastlingFlags) -> Board {
        Board {
            board: [[None; 8]; 8],
            turn,
            castling,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
        }
    }

    /// Get the current player's turn
    ///
    /// # Examples
    /// ```
    /// # use chess_engine::board::Board;
    /// # use chess_engine::piece::Color;
    /// let default = Board::default_board();
    /// assert_eq!(default.turn(), Color::White);
    /// ```
    pub fn turn(&self) -> Color {
        self.turn
    }

    /// Load a board from a string containing (FEN)[<https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation>]
    ///
    /// # Errors
    ///
    /// Will return an error if the string is not valid FEN
    pub fn load_fen(s: &str) -> Result<Board, Error> {
        fen_parser::parse(s)
    }

    /// Create a board initialised in the default chess starting
    /// position
    pub fn default_board() -> Board {
        Board {
            board: [
                //   a  b  c
                // 1 a1 b1 c1
                // 2 a2 b2 c2
                row![o; w r, w n, w b, w q, w k, w b, w n, w r],
                row![o; w p, w p, w p, w p, w p, w p, w p, w p],
                [None; 8],
                [None; 8],
                [None; 8],
                [None; 8],
                row![o; b p, b p, b p, b p, b p, b p, b p, b p],
                row![o; b r, b n, b b, b q, b k, b b, b n, b r],
            ],
            turn: Color::White,
            castling: CastlingFlags::DEFAULT,
            en_passant: None,
            halfmove: 0,
            fullmove: 1,
        }
    }

    // this function only checks if castling is at all allowed
    fn can_castle(&self, castle: Castling, color: Color) -> bool {
        (self.castling
            & match color {
                Color::White => CastlingFlags::WHITE,
                Color::Black => CastlingFlags::BLACK,
            }
            & match castle {
                Castling::Long => CastlingFlags::LONG,
                Castling::Short => CastlingFlags::SHORT,
            })
        .bits()
            != 0
    }

    /// Check if a certain move is legal to perform
    pub fn is_legal(&self, m: Move, side: Color) -> bool {
        match m {
            Move::Normal { from, .. } | Move::Promotion { from, .. } => {
                self[from].map_or(false, |piece| {
                    let legal_moves = legal_moves::enumerate_legal_moves(piece, from, self, true);
                    legal_moves.into_iter().any(|x| x == m)
                })
            }
            Move::Castling(c) => self.can_castle(c, side),
        }
    }

    /// Perform a move and return the next board. Returns [None] if
    /// the move was illegal.
    #[allow(clippy::missing_panics_doc)]
    pub fn perform_move(&self, m: Move) -> Option<Board> {
        // local function because this snippet occurs 3 times
        fn rook_taken_castling(flags: &mut CastlingFlags, file: u32, color: Color) {
            if file == 0 {
                *flags &= !match color {
                    Color::White => CastlingFlags::WHITE_LONG,
                    Color::Black => CastlingFlags::BLACK_LONG,
                };
            } else if file == 7 {
                *flags &= !match color {
                    Color::White => CastlingFlags::WHITE_SHORT,
                    Color::Black => CastlingFlags::BLACK_SHORT,
                };
            }
        }

        if !self.is_legal(m, self.turn) {
            return None;
        }

        let mut new_board = *self;
        let mut new_en_passant = None;
        let mut reset_halfmove = false;

        match m {
            Move::Normal { from, to } => {
                // the move has already been found to be legal, so we can unwrap
                match self[from].unwrap() {
                    Piece {
                        piece: PieceType::Rook,
                        color,
                    } => {
                        // disable castling in one direction
                        rook_taken_castling(&mut new_board.castling, from.file, color);
                    }
                    Piece {
                        piece: PieceType::King,
                        color,
                    } => {
                        // disable castling in both directions
                        new_board.castling &= !match color {
                            Color::White => CastlingFlags::WHITE,
                            Color::Black => CastlingFlags::BLACK,
                        }
                    }
                    Piece {
                        piece: PieceType::Pawn,
                        color,
                    } => {
                        reset_halfmove = true;
                        let dir = match color {
                            Color::White => SquareDiff::new(1, 0),
                            Color::Black => SquareDiff::new(-1, 0),
                        };
                        if let Some(en_passant) = self.en_passant {
                            if en_passant == to {
                                debug_assert!(
                                    new_board[to + dir] == Some(Piece::new(PieceType::Pawn, color)),
                                    "The piece taken by en passant wasn't a pawn, this is most likely a bug"
                                );
                                new_board[to + dir] = None;
                            }
                        } else if (to - from).abs().d_rank == 2 {
                            // if a pawn moved two squares, we need to
                            // set the new en passant square
                            new_en_passant = Some(from + dir);
                        }
                    }
                    _ => (),
                };

                if self[to].is_some() {
                    reset_halfmove = true;
                }

                // disable castling in that direction if the rook was taken
                if let Some(Piece {
                    piece: PieceType::Rook,
                    color,
                }) = self[to]
                {
                    rook_taken_castling(&mut new_board.castling, to.file, color);
                }

                new_board[to] = self[from];
                new_board[from] = None;
            }
            Move::Castling(c) => {
                use Castling::{Long, Short};

                let color = self.turn;
                let rank = color.home_rank();
                let king_from = SquareSpec::new(rank, 4);

                let (rf, kt, rt) = match c {
                    Short => (7, 6, 5),
                    Long => (0, 2, 3),
                };

                let rook_from = SquareSpec::new(rank, rf);
                let king_to = SquareSpec::new(rank, kt);
                let rook_to = SquareSpec::new(rank, rt);

                new_board.castling &= !match color {
                    Color::White => CastlingFlags::WHITE,
                    Color::Black => CastlingFlags::BLACK,
                };

                new_board[king_to] = self[king_from];
                new_board[king_from] = None;
                new_board[rook_to] = self[rook_from];
                new_board[rook_from] = None;
            }
            Move::Promotion { from, to, target } => {
                // since promotions are always pawn moves, this must
                // result in resetting the halfmove counter
                reset_halfmove = true;

                // yet again have to double check if either of the
                // rooks were taken
                if let Some(Piece {
                    piece: PieceType::Rook,
                    color,
                }) = self[to]
                {
                    rook_taken_castling(&mut new_board.castling, to.file, color);
                }

                // again, the move is guaranteed to be valid, so this
                // unwrap can't panic
                new_board[to] = Some(Piece::new(target, self[from].unwrap().color));
                new_board[from] = None;
            }
        }

        new_board.en_passant = new_en_passant;
        new_board.turn = self.turn.opposite();
        if self.turn == Color::Black {
            new_board.fullmove += 1;
        }
        if reset_halfmove {
            new_board.halfmove = 0;
        } else {
            new_board.halfmove += 1;
        }

        Some(new_board)
    }

    /// Returns whether the current player is in check
    pub fn in_check(&self) -> bool {
        self.is_threatened(
            self.turn,
            match self.king(self.turn) {
                Some(king) => king,
                // we can't be checked if there's no king to check
                _ => return false,
            },
        )
    }

    /// Get the current halfmove
    pub fn halfmove(&self) -> u32 {
        self.halfmove
    }

    /// Performs a move with wanton abandon for the rules, effectively
    /// taking any piece on the resulting squares regardless of color.
    /// Moving an empty piece will also result in a phantom take.
    /// Needless to say, this function shouldn't really be used by
    /// anyone except internally, but if you need it, it's there.  Oh
    /// yeah, castling is also unchecked and will produce wildly wrong
    /// results if used illegally
    pub fn unchecked_perform_move(&self, m: Move) -> Board {
        let mut new_board = *self;

        match m {
            Move::Normal { from, to } => {
                new_board[to] = self[from];
                new_board[from] = None;
            }
            Move::Castling(c) => {
                let rank = self.turn.home_rank();
                let kf = 4;
                let (rf, kt, rt) = match c {
                    Castling::Long => (0, 2, 3),
                    Castling::Short => (7, 6, 5),
                };

                let (king_from, rook_from, king_to, rook_to) = (
                    SquareSpec::new(rank, kf),
                    SquareSpec::new(rank, rf),
                    SquareSpec::new(rank, kt),
                    SquareSpec::new(rank, rt),
                );

                new_board[king_to] = self[king_from];
                new_board[king_from] = None;
                new_board[rook_to] = self[rook_from];
                new_board[rook_from] = None;
            }
            Move::Promotion { from, to, target } => {
                new_board[to] = self[from];
                new_board[from] = None;
                if let Some(Piece { color, .. }) = new_board[to] {
                    new_board[to] = Some(Piece {
                        color,
                        piece: target,
                    });
                }
            }
        }
        if let Move::Castling(_) = m {
            new_board.castling &= !match self.turn {
                Color::White => CastlingFlags::WHITE,
                Color::Black => CastlingFlags::BLACK,
            };
        }

        new_board.turn = self.turn.opposite();

        new_board
    }

    /// Get all the legal moves for the piece on this square. If the
    /// square is empty, or if the selected piece is unavailable this
    /// turn, this will return an empty vector.
    pub fn get_legal_moves(&self, piece_location: SquareSpec) -> Vec<Move> {
        if let Some(piece) = self[piece_location] {
            if piece.color != self.turn {
                let f = |x| match x {
                    Color::White => "white",
                    Color::Black => "black",
                };

                return vec![];
            }
            legal_moves::enumerate_legal_moves(piece, piece_location, self, true)
        } else {
            vec![]
        }
    }

    /// Like [`get_legal_moves`], but for getting all the legal moves possible on this turn
    pub fn get_all_legal_moves(&self) -> Vec<Move> {
        let mut all_moves = Vec::new();

        for (rank, row) in self.board.iter().enumerate() {
            for (file, piece) in row.iter().enumerate() {
                let sq = SquareSpec::new(rank as u32, file as u32);
                if let Some(Piece { color, .. }) = piece {
                    if *color == self.turn {
                        all_moves.append(&mut self.get_legal_moves(sq));
                    }
                }
            }
        }

        all_moves
    }

    /// Get a particular color's king's square (if there is one)
    ///
    /// # Example
    /// ```
    /// # use chess_engine::board::{Board, SquareSpec};
    /// # use chess_engine::piece::Color;
    /// let king_square = Board::default_board().king(Color::White).unwrap();
    ///
    /// assert_eq!(king_square, "e1".parse::<SquareSpec>().unwrap());
    /// ```
    pub fn king(&self, king: Color) -> Option<SquareSpec> {
        for (rank, arr) in self.board.iter().enumerate() {
            for (file, piece) in arr.iter().enumerate() {
                match piece {
                    Some(Piece {
                        piece: PieceType::King,
                        color,
                    }) if color == &king => {
                        return Some(SquareSpec {
                            rank: rank as u32,
                            file: file as u32,
                        })
                    }
                    _ => continue,
                }
            }
        }
        None
    }

    /// Check if a certain square on the board is threatened
    pub fn is_threatened(&self, color: Color, sq: SquareSpec) -> bool {
        for (rank, row) in self.board.iter().enumerate().map(|(c, i)| (c as u32, i)) {
            for (file, piece) in row
                .iter()
                .enumerate()
                .filter_map(|(c, p)| p.map(|x| (c as u32, x)))
            {
                if piece.color == color.opposite() {
                    let legal_moves = legal_moves::enumerate_legal_moves(
                        piece,
                        SquareSpec { rank, file },
                        self,
                        false,
                    );
                    if legal_moves.into_iter().any(|m| match m {
                        Move::Normal { to, .. } => to == sq,
                        _ => false,
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl std::ops::Index<SquareSpec> for Board {
    type Output = Option<Piece>;
    fn index(&self, s: SquareSpec) -> &Option<Piece> {
        &self.board[s.rank as usize][s.file as usize]
    }
}

impl std::ops::Index<&str> for Board {
    type Output = Option<Piece>;
    fn index(&self, s: &str) -> &Option<Piece> {
        &self[s
            .parse::<SquareSpec>()
            .expect("Tried indexing with an invalid square")]
    }
}

impl std::ops::IndexMut<SquareSpec> for Board {
    fn index_mut(&mut self, s: SquareSpec) -> &mut Option<Piece> {
        &mut self.board[s.rank as usize][s.file as usize]
    }
}

impl fmt::Display for CastlingFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        if self.contains(CastlingFlags::WHITE_SHORT) {
            s.push('K');
        }
        if self.contains(CastlingFlags::WHITE_LONG) {
            s.push('Q');
        }
        if self.contains(CastlingFlags::BLACK_SHORT) {
            s.push('k');
        }
        if self.contains(CastlingFlags::BLACK_SHORT) {
            s.push('q');
        }
        write!(f, "{}", s)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write;

        let mut board = String::new();
        for rank in self.board.iter().rev() {
            let mut empty_squares = 0;
            for piece in rank.iter() {
                if let Some(piece) = piece {
                    if empty_squares != 0 {
                        write!(&mut board, "{}", empty_squares)?;
                        empty_squares = 0;
                    }
                    write!(&mut board, "{}", piece)?;
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares != 0 {
                write!(&mut board, "{}", empty_squares)?;
            }
            board.push('/');
        }
        // we added one too many slashes
        let _ = board.pop();
        write!(
            f,
            "{board} {turn} {castling} {en_passant} {halfmove} {fullmove}",
            board = board,
            turn = match self.turn {
                Color::White => 'w',
                Color::Black => 'b',
            },
            castling = self.castling,
            en_passant = match self.en_passant {
                Some(sq) => format!("{}", sq),
                None => "-".to_string(),
            },
            halfmove = self.halfmove,
            fullmove = self.fullmove
        )
    }
}

impl Default for Board {
    fn default() -> Board {
        Board::default_board()
    }
}

#[cfg(test)]
mod tests {
    static DEFAULT_BOARD: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    use super::*;

    #[test]
    fn default_board_display() {
        let default = Board::default_board();
        let s = format!("{}", default);

        assert_eq!(&s, DEFAULT_BOARD);
    }

    #[test]
    fn parsing_fen_of_default() {
        let parsed = Board::load_fen(DEFAULT_BOARD).unwrap();
        let constructed = Board::default_board();

        assert_eq!(parsed, constructed);
    }

    #[test]
    fn parsing_en_passant() {
        let parsed = Board::load_fen("8/8/8/5Pp1/8/8/8/8 w - g6 0 1").unwrap();

        assert!(parsed.en_passant.is_some());
        assert_eq!(
            parsed.en_passant.unwrap(),
            "g6".parse::<SquareSpec>().unwrap()
        );
    }

    // TODO: Tests that need to be written:
    // - pawn moves work
    // - promotion works
    // - en passant works
    // - pawn moves reset halfmove correctly
    // - other moves don't reset halfmove
    // - a bunch of kinds of moves correctly place their piece
    // - castling rights are updated when rooks move
    // - castling rights are updated when rooks are taken
    // - castling rights are updated when king moves
    // - castling rights are updated when castling
    // - the legality assumption made by perform_move isn't somehow
    //   violated
    // - the king shouldn't be possible to take
    // - fullmove is updated correctly and according to spec
}
