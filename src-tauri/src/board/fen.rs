//! 中国象棋 FEN 解析与序列化。
//!
//! 格式：`<10行局面> <走子方 w/b> - - <半步钟> <回合数>`
//! 局面从黑方底线（rank 9）到红方底线（rank 0），红子大写、黑子小写。

use super::types::{Color, Piece, PieceKind, Position, NUM_FILES, NUM_RANKS};

/// 序列化局面为规范 FEN。
pub fn to_fen(pos: &Position) -> String {
    let mut rows = Vec::with_capacity(NUM_RANKS as usize);
    for rank in (0..NUM_RANKS).rev() {
        let mut row = String::new();
        let mut empty = 0u8;
        for file in 0..NUM_FILES {
            match pos.board[rank as usize][file as usize] {
                None => empty += 1,
                Some(piece) => {
                    if empty > 0 {
                        row.push(char::from_digit(u32::from(empty), 10).expect("digit"));
                        empty = 0;
                    }
                    row.push(piece.kind.fen_char(piece.color));
                }
            }
        }
        if empty > 0 {
            row.push(char::from_digit(u32::from(empty), 10).expect("digit"));
        }
        rows.push(row);
    }
    format!(
        "{} {} - - {} {}",
        rows.join("/"),
        pos.side_to_move.fen_char(),
        pos.halfmove_clock,
        pos.fullmove_number
    )
}

/// 解析 FEN 字符串。仅做语法解析，规则合法性由 `validate` 判定。
pub fn parse_fen(s: &str) -> Result<Position, String> {
    let fields: Vec<&str> = s.split_whitespace().collect();
    if fields.len() < 2 {
        return Err("FEN 至少需要两个字段：局面与走子方".to_string());
    }
    if fields.len() >= 3 && fields[2] != "-" {
        return Err("中国象棋 FEN 第 3 字段必须为 '-'".to_string());
    }
    if fields.len() >= 4 && fields[3] != "-" {
        return Err("中国象棋 FEN 第 4 字段必须为 '-'".to_string());
    }

    let rows: Vec<&str> = fields[0].split('/').collect();
    if rows.len() != NUM_RANKS as usize {
        return Err(format!("局面应为 {} 行，实际 {} 行", NUM_RANKS, rows.len()));
    }

    let mut board = [[None; NUM_FILES as usize]; NUM_RANKS as usize];
    for (i, row) in rows.iter().enumerate() {
        let rank = (NUM_RANKS - 1 - i as u8) as usize; // 第一行是黑方底线 rank 9
        let mut file = 0u8;
        for ch in row.chars() {
            if file >= NUM_FILES {
                return Err(format!("rank {} 的棋子超过 9 格", rank));
            }
            if let Some(d) = ch.to_digit(10) {
                if d == 0 {
                    return Err("局面中不允许出现数字 0".to_string());
                }
                file += d as u8;
                if file > NUM_FILES {
                    return Err(format!("rank {} 的格子数超过 9", rank));
                }
            } else if let Some((kind, color)) = PieceKind::from_fen_char(ch) {
                board[rank][file as usize] = Some(Piece { color, kind });
                file += 1;
            } else {
                return Err(format!("无法识别的棋子字符：{ch}"));
            }
        }
        if file != NUM_FILES {
            return Err(format!("rank {} 应合计 9 格，实际 {}", rank, file));
        }
    }

    let side_to_move = fields[1]
        .chars()
        .next()
        .and_then(Color::from_fen_char)
        .ok_or_else(|| "走子方必须为 w 或 b".to_string())?;
    let halfmove_clock = fields.get(4).and_then(|f| f.parse().ok()).unwrap_or(0);
    let fullmove_number = fields.get(5).and_then(|f| f.parse().ok()).unwrap_or(1);

    Ok(Position {
        board,
        side_to_move,
        halfmove_clock,
        fullmove_number,
    })
}
