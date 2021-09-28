macro_rules! row {
    [ $($s:ident $p:ident),* ] => {
        [ $(p!($s $p)),* ]
    };

    [ o; $($s:ident $p:ident),* ] => {
        [ $(Some(p!($s $p))),* ]
    };
}

macro_rules! p {
    (w $piece:ident) => {
        Piece {
            color: crate::piece::Color::White,
            piece: p!([$piece]),
        }
    };
    (b $piece:ident) => {
        Piece {
            color: crate::piece::Color::Black,
            piece: p!([$piece]),
        }
    };

    ([p]) => {
        crate::piece::PieceType::Pawn
    };

    ([r]) => {
        crate::piece::PieceType::Rook
    };
    ([b]) => {
        crate::piece::PieceType::Bishop
    };
    ([q]) => {
        crate::piece::PieceType::Queen
    };
    ([k]) => {
        crate::piece::PieceType::King
    };
    ([n]) => {
        crate::piece::PieceType::Knight
    };
}
