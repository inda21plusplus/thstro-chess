//! this module is responsible for checking all the low level rules and whatnot

use super::{Board, Castling, Move, SquareDiff, SquareSpec};
use crate::piece::{Color, Piece, PieceType};

const DIAGONALS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const AXES: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

// Enumerate all possible legal moves for a certain pieces. We use a
// boolean flag for whether this function should filter out moves that
// result in the king being threatened, and it has to be done this way
// as we will call this function recursively to see if the king is threatened
pub(crate) fn enumerate_legal_moves(
    piece: Piece,
    location: SquareSpec,
    board: &Board,
    account_for_check: bool,
) -> Vec<Move> {
    let diagonals = DIAGONALS
        .iter()
        .map(|&(d_rank, d_file)| SquareDiff { d_rank, d_file });

    let axes = AXES
        .iter()
        .map(|&(d_rank, d_file)| SquareDiff { d_rank, d_file });

    let mut moves = match piece.piece {
        PieceType::Pawn => {
            let mut moves = Vec::new();
            for to in get_moves_pawn(piece.color, board, location) {
                match to {
                    PawnMove::EnPassant(to) => {
                        moves.push(Move::Normal { from: location, to });
                    }
                    PawnMove::Normal(to) => moves.push(Move::Normal { from: location, to }),
                    PawnMove::Promotion(to) => {
                        for piece in [
                            PieceType::Queen,
                            PieceType::Knight,
                            PieceType::Bishop,
                            PieceType::Rook,
                        ] {
                            moves.push(Move::Promotion {
                                from: location,
                                to,
                                target: piece,
                            });
                        }
                    }
                }
            }
            moves
        }
        PieceType::King => get_moves_king(piece.color, board, location, account_for_check),
        PieceType::Knight => get_moves_knight(piece.color, board, location)
            .into_iter()
            .map(|to| Move::Normal { from: location, to })
            .collect(),
        PieceType::Rook => {
            get_moves_directions(piece.color, board, location, &axes.collect::<Vec<_>>())
                .into_iter()
                .map(|to| Move::Normal { from: location, to })
                .collect()
        }
        PieceType::Bishop => {
            get_moves_directions(piece.color, board, location, &diagonals.collect::<Vec<_>>())
                .into_iter()
                .map(|to| Move::Normal { from: location, to })
                .collect()
        }
        PieceType::Queen => get_moves_directions(
            piece.color,
            board,
            location,
            &axes.chain(diagonals).collect::<Vec<_>>(),
        )
        .into_iter()
        .map(|to| Move::Normal { from: location, to })
        .collect(),
    };

    if account_for_check {
        moves.retain(|m| {
            let new_board = board.unchecked_perform_move(*m);
            let king = match new_board.king(board.turn()) {
                Some(k) => k,
                _ => return true,
            };

            for (rank, row) in new_board.board.iter().enumerate() {
                for (file, p) in row.iter().enumerate() {
                    if let Some(p) = p {
                        if p.color != piece.color {
                            for m_other in enumerate_legal_moves(
                                *p,
                                SquareSpec {
                                    rank: rank as u32,
                                    file: file as u32,
                                },
                                &new_board,
                                false,
                            ) {
                                if let Move::Normal { to, .. } = m_other {
                                    if to == king {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            true
        });
    }

    moves
}

pub(crate) fn get_moves_king(
    k_col: Color,
    board: &Board,
    orig_sq: SquareSpec,
    check_castling: bool,
) -> Vec<Move> {
    let mut moves = Vec::new();

    let diagonals = DIAGONALS
        .iter()
        .map(|&(d_rank, d_file)| SquareDiff { d_rank, d_file });

    let axes = AXES
        .iter()
        .map(|&(d_rank, d_file)| SquareDiff { d_rank, d_file });

    for dir in axes.chain(diagonals) {
        if let Some(sq) = orig_sq.checked_add(dir) {
            match board[sq] {
                Some(Piece { color, .. }) if color == k_col => (),
                _ => moves.push(Move::Normal {
                    from: orig_sq,
                    to: sq,
                }),
            }
        }
    }
    if check_castling {
        'castle: {
            if board.is_threatened(k_col, orig_sq) {
                break 'castle;
            }
            'long: {
                if board.can_castle(Castling::Long, k_col) {
                    let (bn, cn, dn) = {
                        let rank = k_col.home_rank();
                        (
                            SquareSpec { rank, file: 1 },
                            SquareSpec { rank, file: 2 },
                            SquareSpec { rank, file: 3 },
                        )
                    };
                    match (board[bn], board[cn], board[dn]) {
                        (None, None, None) => (),
                        _ => break 'long,
                    };

                    // we only need to check the intermediate square as the
                    // other check is handled by enumerate_legal_moves
                    if board.is_threatened(
                        k_col,
                        orig_sq
                            + SquareDiff {
                                d_rank: 0,
                                d_file: -1,
                            },
                    ) {
                        break 'long;
                    }

                    moves.push(Move::Castling(Castling::Long));
                }
            }
            'short: {
                if board.can_castle(Castling::Short, k_col) {
                    let (r#fn, gn) = {
                        let rank = k_col.home_rank();
                        (SquareSpec { rank, file: 5 }, SquareSpec { rank, file: 6 })
                    };
                    match (board[r#fn], board[gn]) {
                        (None, None) => (),
                        _ => break 'short,
                    };

                    // once again, we only need to check the intermediate
                    // square as the other check is handled by
                    // enumerate_legal_moves
                    if board.is_threatened(
                        k_col,
                        orig_sq
                            + SquareDiff {
                                d_rank: 0,
                                d_file: 1,
                            },
                    ) {
                        break 'short;
                    }

                    moves.push(Move::Castling(Castling::Short));
                }
            }
        }
    }

    moves
}

enum PawnMove {
    Normal(SquareSpec),
    EnPassant(SquareSpec),
    Promotion(SquareSpec),
}

fn get_moves_pawn(p_col: Color, board: &Board, orig_sq: SquareSpec) -> Vec<PawnMove> {
    use PawnMove::*;

    let mut moves = Vec::new();

    let pawn_direction = SquareDiff {
        d_rank: match p_col {
            Color::White => 1,
            Color::Black => -1,
        },
        d_file: 0,
    };

    // whether we can move forward once
    if let Some((sq, None)) = orig_sq
        .checked_add(pawn_direction)
        .map(|sq| (sq, board[sq]))
    {
        // check for promotion
        if sq.rank == p_col.opposite().home_rank() {
            moves.push(Promotion(sq));
        } else {
            moves.push(Normal(sq));
            // if we can move twice
            if orig_sq.rank == p_col.pawn_home_rank() {
                if let Some((sq2, None)) = sq.checked_add(pawn_direction).map(|sq| (sq, board[sq]))
                {
                    moves.push(Normal(sq2));
                }
            }
        }
    }

    // from white's perspective, remember that 0 is the "a" file
    const LEFT: SquareDiff = SquareDiff {
        d_rank: 0,
        d_file: -1,
    };
    const RIGHT: SquareDiff = SquareDiff {
        d_rank: 0,
        d_file: 1,
    };

    let left_diag = orig_sq
        .checked_add(pawn_direction + LEFT)
        .map(|sq| (sq, board[sq]));
    let right_diag = orig_sq
        .checked_add(pawn_direction + RIGHT)
        .map(|sq| (sq, board[sq]));

    // check en passants
    if let Some(en_passant) = board.en_passant {
        if let Some((sq, _)) = left_diag {
            if sq == en_passant {
                moves.push(EnPassant(sq));
            }
        }
        if let Some((sq, _)) = right_diag {
            if sq == en_passant {
                moves.push(EnPassant(sq));
            }
        }
    }

    // we don't need to double check the en passant stuff as its
    // impossible for the en passant square to contain a takeable
    // piece

    // check left diagonal
    if let Some((sq, Some(Piece { color, .. }))) = left_diag {
        if p_col != color {
            moves.push(Normal(sq));
        }
    }

    // check right diagonal
    if let Some((sq, Some(Piece { color, .. }))) = right_diag {
        if p_col != color {
            moves.push(Normal(sq));
        }
    }

    moves
}

fn get_moves_knight(k_col: Color, board: &Board, orig_sq: SquareSpec) -> Vec<SquareSpec> {
    let mut moves = [
        (2, 1),
        (2, -1),
        (-2, 1),
        (-2, -1),
        (1, 2),
        (1, -2),
        (-1, 2),
        (-1, -2),
    ]
    .iter()
    .map(|&(d_rank, d_file)| SquareDiff { d_rank, d_file })
    .filter_map(|sd| orig_sq.checked_add(sd))
    .collect::<Vec<_>>();

    moves.retain(|x| !matches!(board[*x], Some(Piece { color, .. }) if k_col == color));

    moves
}

fn get_moves_directions(
    piece_col: Color,
    board: &Board,
    orig_sq: SquareSpec,
    directions: &[SquareDiff],
) -> Vec<SquareSpec> {
    // assumes all of the directions are unit vectors

    let mut moves = Vec::new();

    'dir: for direction in directions {
        let mut sq_i = orig_sq;
        while let Some(sq) = sq_i.checked_add(*direction) {
            sq_i = sq;
            match board[sq_i] {
                Some(Piece { color, .. }) if color == piece_col => continue 'dir,
                Some(Piece { .. }) => {
                    moves.push(sq_i);
                    continue 'dir;
                }
                None => {
                    moves.push(sq_i);
                }
            }
        }
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::{super::Castling, Board, Move, SquareSpec};

    /// Macro to make writing tests wayyy easier, by simply writing
    /// out a board state in FEN, saying which piece we check, and
    /// then writing out all the legal moves without any quotes or
    /// anything
    macro_rules! basic_test {
        {
            fen: $fen:expr,
            piece: $spot:ident,
            legal_moves: [$($token:tt)*],
        } => {
            {
                let board = Board::load_fen($fen).unwrap();
                let $spot = stringify!($spot).parse::<SquareSpec>().unwrap();
                let piece = board[$spot].unwrap();
                let legal_moves = move_list![$spot; $($token)*].iter().map(|x|*x).collect::<Vec<_>>();
                let moves = super::enumerate_legal_moves(piece, $spot, &board, true);

                compare_moves(moves, legal_moves);
            }
        }
    }

    macro_rules! move_list {
        [$location:ident; $($token:tt),*] => {
            [$(
                move_list!(stringify!($location).parse::<SquareSpec>().unwrap(); $token)
            ),*]
        };
        ($location:expr; $spot:ident) => {
            Move::Normal {
                to: stringify!($spot).parse::<SquareSpec>().unwrap(),
                from: $location,
            }
        };
        // same deal here as castling, it kinda sucks
        ($location:expr; [$spot:ident=$pt:ident]) => {
            Move::Promotion {
                from: $location,
                to: stringify!($spot).parse::<SquareSpec>().unwrap(),
                target: stringify!($pt).parse::<crate::piece::PieceType>().unwrap()
            }
        };
        ($location:expr; [$spot:ident=*]) => {
            move_list!($location; [$spot=N]),
            move_list!($location; [$spot=Q]),
            move_list!($location; [$spot=K]),
            move_list!($location; [$spot=R])
        };
        // Basically, the macros have to be this way because we match
        // on `tokentree,*`, and in o-o the o will be a tokentree,
        // then the - will be a tokentree, etc. Trying to match on
        // just $($tokens:tt)* will make the commas ambiguous, and so
        // that's not possible either. In the end, I chose to simply
        // wrap the thing in brackets to make it one token tree.
        ($location:expr; [o-o]) => {
            Move::Castling(Castling::Short)
        };
        ($location:expr; [o-o-o]) => {
            Move::Castling(Castling::Long)
        }
    }

    fn compare_moves(generated_moves: Vec<Move>, possible_moves: Vec<Move>) {
        use std::collections::HashSet;

        let possible = possible_moves.into_iter().collect::<HashSet<_>>();
        let generated = generated_moves.into_iter().collect::<HashSet<_>>();

        let mut found_diff = false;

        for m in possible.symmetric_difference(&generated) {
            found_diff = true;
            if possible.contains(m) {
                eprintln!("move {} should be possible, but wasn't generated", m);
            } else {
                eprintln!("move {} was generated, when it shouldn't be possible", m)
            }
        }
        if found_diff {
            panic!("Found a difference between generated and possible moves");
        }
    }

    #[test]
    fn get_move_directions_stops_for_same_color() {
        basic_test! {
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            piece: a1,
            legal_moves: [],
        }
    }

    #[test]
    fn get_move_directions_can_capture() {
        basic_test! {
            fen: "8/8/8/3p4/2pRp3/3p4/8/8 w - - 0 1",
            piece: d4,
            legal_moves: [d5, e4, d3, c4],
        }
    }

    #[test]
    fn get_move_directions_capture_and_empty() {
        basic_test! {
            fen: "8/8/3p4/8/8/B7/8/8 w - - 0 1",
            piece: a3,
            legal_moves: [b2, c1, b4, c5, d6],
        }
    }

    #[test]
    fn pawn_normal() {
        basic_test! {
            fen: "8/8/8/8/3P4/8/8/8 w - - 0 1",
            piece: d4,
            legal_moves: [d5],
        }
    }

    #[test]
    fn pawn_two_steps() {
        basic_test! {
            fen: "8/8/8/8/8/8/3P4/8 w - - 0 1",
            piece: d2,
            legal_moves: [d3, d4],
        }
    }

    #[test]
    fn pawn_two_blocked() {
        basic_test! {
            fen: "8/8/8/8/3p4/8/3P4/8 w - - 0 1",
            piece: d2,
            legal_moves: [d3],
        }
    }

    #[test]
    fn pawn_en_passant() {
        basic_test! {
            fen: "8/8/8/5Pp1/8/8/8/8 w - g6 0 1",
            piece: f5,
            legal_moves: [f6, g6],
        }
    }

    #[test]
    fn pawn_promotion() {
        basic_test! {
            fen: "8/4P3/8/8/8/8/8/8 w - - 0 1",
            piece: e7,
            legal_moves: [
                [e8=B],
                [e8=R],
                [e8=N],
                [e8=Q]
            ],
        }
    }

    #[test]
    fn pawn_take() {
        basic_test! {
            fen: "8/8/4ppp1/5P2/8/8/8/8 w - - 0 1",
            piece: f5,
            legal_moves: [g6, e6],
        }
    }

    #[test]
    fn pawn_all_at_once() {
        basic_test! {
            fen: "8/8/4p3/5Pp1/8/8/8/8 w - g6 0 1",
            piece: f5,
            legal_moves: [g6, f6, e6],
        }
    }

    #[test]
    fn queen() {
        // hate
        basic_test! {
            fen: "8/8/8/8/8/8/1Q6/8 w - - 0 1",
            piece: b2,
            legal_moves: [a1, a2, a3, b1, c1, c2, d2, e2, f2, g2, h2,
                          c3, d4, e5, f6, g7, h8,
                          b3, b4, b5, b6, b7, b8
            ],
        }
    }

    #[test]
    fn knight() {
        basic_test! {
            fen: "8/8/8/8/8/2N5/8/8 w - - 0 1",
            piece: c3,
            legal_moves: [b1, a2, a4, b5, d5, e4, e2, d1],
        }
    }

    #[test]
    fn castling() {
        basic_test! {
            fen: "8/8/8/8/8/8/3PPP2/R3K2R w KQ - 0 1",
            piece: e1,
            legal_moves: [f1, d1, [o-o], [o-o-o]],
        }
    }

    #[test]
    fn kings_next_to_each_other() {
        basic_test! {
            fen: "8/8/8/8/8/k7/8/K7 w - - 0 1",
            piece: a1,
            legal_moves: [b1],
        }
    }

    #[test]
    fn king_in_check() {
        basic_test! {
            fen: "8/8/8/8/r7/8/8/K7 w - - 0 1",
            piece: a1,
            legal_moves: [b1, b2],
        }
    }

    #[test]
    fn castling_blocked_intermediate() {
        basic_test! {
            fen: "8/8/8/5r2/8/8/8/4K2R w K - 0 1",
            piece: e1,
            legal_moves: [d1, d2, e2],
        }
    }

    #[test]
    fn castling_blocked_final() {
        basic_test! {
            fen: "8/8/8/6r1/8/8/8/4K2R w K - 0 1",
            piece: e1,
            legal_moves: [d1, d2, e2, f1, f2],
        }
    }

    #[test]
    fn castling_while_in_check() {
        basic_test! {
            fen: "8/8/8/4r3/8/8/8/4K2R w K - 0 1",
            piece: e1,
            legal_moves: [d1, d2, f1, f2],
        }
    }

    #[test]
    fn castling_blocked_by_king() {
        basic_test! {
            fen: "8/8/8/8/8/8/2kPPP2/R3KB1R w Q - 0 1",
            piece: e1,
            legal_moves: [],
        }
    }

    #[test]
    fn cant_move_pinned_piece() {
        basic_test! {
            fen: "8/8/8/8/8/r7/R7/K7 w - - 0 1",
            piece: a2,
            legal_moves: [a3],
        }
    }
}
