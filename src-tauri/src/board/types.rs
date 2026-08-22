//! 基础类型：颜色、棋子、坐标、着法与局面。

use serde::{Deserialize, Serialize};

/// 棋盘行数（红方底线 rank 0 → 黑方底线 rank 9）。
pub const NUM_RANKS: u8 = 10;
/// 棋盘列数（file a..i，a 为红方左侧）。
pub const NUM_FILES: u8 = 9;

/// 中国象棋起始局面 FEN。
pub const START_FEN: &str = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";

/// 行棋方。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Red,
    Black,
}

impl Color {
    pub fn opponent(self) -> Color {
        match self {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }

    /// FEN 走子方字符：红 'w' / 黑 'b'。
    pub fn fen_char(self) -> char {
        match self {
            Color::Red => 'w',
            Color::Black => 'b',
        }
    }

    pub fn from_fen_char(c: char) -> Option<Color> {
        match c {
            'w' => Some(Color::Red),
            'b' => Some(Color::Black),
            _ => None,
        }
    }

    /// 前端 DTO 使用的名称。
    pub fn from_name(s: &str) -> Option<Color> {
        match s {
            "red" => Some(Color::Red),
            "black" => Some(Color::Black),
            _ => None,
        }
    }
}

/// 棋子种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PieceKind {
    King,
    Advisor,
    Elephant,
    Horse,
    Rook,
    Cannon,
    Pawn,
}

impl PieceKind {
    /// FEN 棋子字符：红方大写、黑方小写（k/a/b/n/r/c/p）。
    pub fn fen_char(self, color: Color) -> char {
        let c = match self {
            PieceKind::King => 'k',
            PieceKind::Advisor => 'a',
            PieceKind::Elephant => 'b',
            PieceKind::Horse => 'n',
            PieceKind::Rook => 'r',
            PieceKind::Cannon => 'c',
            PieceKind::Pawn => 'p',
        };
        match color {
            Color::Red => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    pub fn from_fen_char(c: char) -> Option<(PieceKind, Color)> {
        let (kind, red) = match c {
            'k' => (PieceKind::King, false),
            'K' => (PieceKind::King, true),
            'a' => (PieceKind::Advisor, false),
            'A' => (PieceKind::Advisor, true),
            'b' => (PieceKind::Elephant, false),
            'B' => (PieceKind::Elephant, true),
            'n' => (PieceKind::Horse, false),
            'N' => (PieceKind::Horse, true),
            'r' => (PieceKind::Rook, false),
            'R' => (PieceKind::Rook, true),
            'c' => (PieceKind::Cannon, false),
            'C' => (PieceKind::Cannon, true),
            'p' => (PieceKind::Pawn, false),
            'P' => (PieceKind::Pawn, true),
            _ => return None,
        };
        let color = if red { Color::Red } else { Color::Black };
        Some((kind, color))
    }

    /// 前端 DTO 使用的名称。
    pub fn from_name(s: &str) -> Option<PieceKind> {
        match s {
            "king" => Some(PieceKind::King),
            "advisor" => Some(PieceKind::Advisor),
            "elephant" => Some(PieceKind::Elephant),
            "horse" => Some(PieceKind::Horse),
            "rook" => Some(PieceKind::Rook),
            "cannon" => Some(PieceKind::Cannon),
            "pawn" => Some(PieceKind::Pawn),
            _ => None,
        }
    }
}

/// 一枚棋子。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

/// 棋盘格：rank 0..10（红方底线为 0），file 0..9（a..i）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub rank: u8,
    pub file: u8,
}

impl Square {
    pub fn new(rank: u8, file: u8) -> Option<Square> {
        if rank < NUM_RANKS && file < NUM_FILES {
            Some(Square { rank, file })
        } else {
            None
        }
    }

    /// UCI-Cyclone 坐标，如 "h2"。
    pub fn uci(self) -> String {
        format!("{}{}", (b'a' + self.file) as char, self.rank)
    }

    pub fn parse_uci(s: &str) -> Option<Square> {
        let b = s.as_bytes();
        if b.len() != 2 {
            return None;
        }
        let file = b[0].checked_sub(b'a')?;
        let rank = b[1].checked_sub(b'0')?;
        Square::new(rank, file)
    }
}

/// 一步着法：起点 → 终点。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

impl Move {
    /// UCI-Cyclone 四字符，如 "h2e2"。
    pub fn uci(self) -> String {
        format!("{}{}", self.from.uci(), self.to.uci())
    }

    pub fn parse_uci(s: &str) -> Option<Move> {
        if s.len() != 4 {
            return None;
        }
        let from = Square::parse_uci(&s[0..2])?;
        let to = Square::parse_uci(&s[2..4])?;
        Some(Move { from, to })
    }
}

/// 局面棋盘：`board[rank][file]`。
pub type BoardArray = [[Option<Piece>; NUM_FILES as usize]; NUM_RANKS as usize];

/// 中国象棋局面。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub board: BoardArray,
    pub side_to_move: Color,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Default for Position {
    /// 空棋盘，红方先行。
    fn default() -> Self {
        Position {
            board: [[None; NUM_FILES as usize]; NUM_RANKS as usize],
            side_to_move: Color::Red,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}
