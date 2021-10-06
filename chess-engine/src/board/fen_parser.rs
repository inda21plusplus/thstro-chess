use super::CastlingFlags;
use super::{Board, SquareSpec};
use crate::error::Error;
use crate::piece::{Color, Piece, PieceType};
use std::convert::TryInto;

pub(crate) fn parse(s: &str) -> Result<Board, Error> {
    let mut parts = s.split(' ');

    let board = parse_boardstate(
        parts
            .next()
            .ok_or_else(|| Error::InvalidFen(s.to_string()))?,
    )?;
    let turn = match parts.next() {
        Some("w") => Color::White,
        Some("b") => Color::Black,
        _ => return Err(Error::InvalidFen(s.to_string())),
    };
    let castling = {
        let c_str = parts
            .next()
            .ok_or_else(|| Error::InvalidFen(s.to_string()))?;
        let mut flags = CastlingFlags::empty();
        if c_str.contains('K') {
            flags |= CastlingFlags::WHITE_SHORT;
        }
        if c_str.contains('k') {
            flags |= CastlingFlags::BLACK_SHORT;
        }
        if c_str.contains('Q') {
            flags |= CastlingFlags::WHITE_LONG;
        }
        if c_str.contains('q') {
            flags |= CastlingFlags::BLACK_LONG;
        }
        flags
    };
    let en_passant = {
        let en_passant_str = parts
            .next()
            .ok_or_else(|| Error::InvalidFen(s.to_string()))?;
        match en_passant_str {
            "-" => None,
            x => Some(
                x.parse::<SquareSpec>()
                    .map_err(|_| Error::InvalidFen(s.to_string()))?,
            ),
        }
    };

    let halfmove = parts
        .next()
        .ok_or_else(|| Error::InvalidFen(s.to_string()))?
        .parse::<u32>()
        .map_err(|_| Error::InvalidFen(s.to_string()))?;
    let fullmove = parts
        .next()
        .ok_or_else(|| Error::InvalidFen(s.to_string()))?
        .parse::<u32>()
        .map_err(|_| Error::InvalidFen(s.to_string()))?;

    Ok(Board {
        board,
        turn,
        castling,
        en_passant,
        halfmove,
        fullmove,
    })
}

fn parse_boardstate(s: &str) -> Result<[[Option<Piece>; 8]; 8], Error> {
    let mut lines = vec![];
    for row in s.split('/') {
        let mut cur_line = vec![];
        for c in row.chars() {
            match parse_piece(c).ok_or_else(|| Error::InvalidFen(s.to_string()))? {
                PieceResult::Piece(p) => cur_line.push(Some(p)),
                PieceResult::Empty(n) => cur_line.extend(std::iter::repeat(None).take(n as usize)),
            }
        }
        if cur_line.len() == 8 {
            lines.push(cur_line.try_into().unwrap());
        } else {
            return Err(Error::InvalidFen(s.to_string()));
        }
    }
    lines.reverse();
    lines
        .try_into()
        .map_err(|_| Error::InvalidFen(s.to_string()))
}

#[allow(variant_size_differences)]
enum PieceResult {
    Piece(Piece),
    Empty(u32),
}

fn parse_piece(c: char) -> Option<PieceResult> {
    use PieceType::*;

    if c.is_ascii_digit() {
        return Some(PieceResult::Empty(c as u32 - '0' as u32));
    }

    let color = if "PNBRQK".contains(c) {
        Color::White
    } else if "pnbrqk".contains(c) {
        Color::Black
    } else {
        return None;
    };

    let piece = match c.to_ascii_lowercase() {
        'p' => Pawn,
        'n' => Knight,
        'b' => Bishop,
        'r' => Rook,
        'q' => Queen,
        'k' => King,
        _ => unreachable!(),
    };

    Some(PieceResult::Piece(Piece { piece, color }))
}
