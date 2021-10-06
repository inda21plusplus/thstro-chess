//! Module containing the [`Game`] type, the main way for an application
//! to create and run a chess game.

use crate::board::{Board, Move};
use crate::piece::Color;

/// The struct representing a chess game, starting in the default
/// position with white going first.
#[derive(Debug, Clone)]
pub struct Game {
    boards: Vec<Board>,
    moves: Vec<Move>,
    board_state: BoardState,
}

/// Enum to represent the various different board states, most
/// importantly the final states.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BoardState {
    /// The game is in a normal state, and you can play as normal
    Normal,
    /// The current player is in check
    Check,
    /// The current player is in checkmate
    Checkmate,
    /// The game has been drawn
    Draw,
    /// The current player has no legal moves and the game has been
    /// drawn
    Stalemate,
}

impl Game {
    /// Create a new board initialised to the default chess position
    pub fn new() -> Self {
        Self {
            boards: vec![Board::default_board()],
            moves: vec![],
            board_state: BoardState::Normal,
        }
    }

    /// Get the current board state
    pub fn board_state(&self) -> BoardState {
        self.board_state
    }

    /// Get a list of all boards so far
    ///
    /// # Examples
    ///
    /// ```
    /// # use chess_engine::game::Game;
    /// # use chess_engine::board::Board;
    /// let default = Board::default_board();
    /// let game = Game::new();
    ///
    /// assert_eq!(game.get_boards(), &[default]);
    /// ```
    pub fn get_boards(&self) -> &[Board] {
        &self.boards[..]
    }

    /// Get a list of all moves so far
    pub fn get_moves(&self) -> &[Move] {
        &self.moves[..]
    }

    /// Make a move, if it is legal, returns a reference to the new
    /// board.  If the move was illegal, [None] is returned
    pub fn make_move(&mut self, next_move: Move) -> Option<&Board> {
        match self.board_state {
            BoardState::Draw | BoardState::Stalemate | BoardState::Checkmate => return None,
            _ => (),
        }

        let last_board = self.boards[self.boards.len() - 1];
        let next_board = match last_board.perform_move(next_move) {
            Some(board) => board,
            None => return None,
        };
        self.boards.push(next_board);
        self.moves.push(next_move);
        self.update_boardstate();
        Some(&self.boards[self.boards.len() - 1])
    }

    fn update_boardstate(&mut self) {
        let board = self.current_board();
        let legal_moves = self.get_all_legal_moves();
        if legal_moves.is_empty() && board.in_check() {
            self.board_state = BoardState::Checkmate;
        } else if legal_moves.is_empty() {
            self.board_state = BoardState::Stalemate;
        } else if board.in_check() {
            self.board_state = BoardState::Check;
        } else if board.halfmove() == 50 {
            self.board_state = BoardState::Draw;
        }
    }

    /// Get which player is supposed to go next
    ///
    /// # Panics
    ///
    /// This function should be unable to panic as self must at least
    /// contain one board.
    pub fn next_player(&self) -> Color {
        debug_assert!(!self.boards.is_empty());
        self.boards.last().unwrap().turn()
    }

    /// Get all legal moves for the current player
    fn get_all_legal_moves(&self) -> Vec<Move> {
        self.current_board().get_all_legal_moves()
    }

    /// Get a reference to the current (latest) board
    ///
    /// # Examples
    ///
    /// ```
    /// # use chess_engine::game::Game;
    /// # use chess_engine::board::Board;
    /// let default = Board::default_board();
    /// let game = Game::new();
    ///
    /// assert_eq!(game.current_board(), &default);
    /// ```
    ///
    /// # Panics
    ///
    /// This function should be unable to panic as self must at least
    /// contain one board.
    pub fn current_board(&self) -> &Board {
        // there should at least be a default board
        debug_assert!(!self.boards.is_empty());
        self.boards.last().unwrap()
    }

    /// Undo the last move, returning `None` if there was no last
    /// move, and the Board/Move combination if there was.
    ///
    /// # Panics
    ///
    /// This function should be unable to panic as self must at least
    /// contain one board.
    pub fn undo_move(&mut self) -> Option<(Board, Move)> {
        self.moves.pop().map(|m| (self.boards.pop().unwrap(), m))
    }
}

impl Default for Game {
    fn default() -> Game {
        Game::new()
    }
}
