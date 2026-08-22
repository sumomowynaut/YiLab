//! 中国象棋 PGN 解析与导出。
//!
//! - 导出：标准 PGN 头（Event/Site/Date/Round/White/Black/Result + 自定义 FEN/PikaXiangqiTitle）+ 主变/变例/注释/NAG。
//! - 导入：解析 PGN 头、着法（UCI-Cyclone）、变例 `(...)`、注释 `{...}`、NAG（符号或 `$n`）。
//!   着法经棋谱树 `insert_main_at`/`insert_move_at` 校验合法性；变例首着按回合前缀（`N.` 红 / `N...` 黑）
//!   沿祖先链定位分支点，前缀缺失或定位失败时回退到当前节点/祖先（处理根级变例与常见 PGN 习惯）。
//! - 走法记谱：本项目导出/导入使用 UCI-Cyclone（如 `h2e2`）；中文纵线制导入暂不支持（`NEEDS_VERIFICATION`）。
//!
//! 往返保证：`import(export(tree))` 与 `tree` 在文档状态（主变/变例/注释/NAG/头信息）上等价。

use crate::board::types::{Color, Move, START_FEN};
use crate::game::nag::Nag;
use crate::game::tree::{GameError, GameHeaders, GameTree, NodeId};

/// 回合前缀：`N.` 为红方第 N 回合，`N...` 为黑方第 N 回合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveNumber {
    Red(u32),
    Black(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Move(String, Option<MoveNumber>),
    Nag(String),
    Comment(String),
    OpenParen,
    CloseParen,
}

/// 导出整棵棋谱树为 PGN 文本。
pub fn export(tree: &GameTree) -> String {
    let mut out = String::new();
    // 头部
    push_header(&mut out, "Event", &tree.headers.event);
    push_header(&mut out, "Site", "");
    push_header(&mut out, "Date", &tree.headers.date);
    push_header(&mut out, "Round", "");
    push_header(&mut out, "White", &tree.headers.red);
    push_header(&mut out, "Black", &tree.headers.black);
    push_header(&mut out, "Result", &tree.headers.result);
    if tree.startpos != START_FEN {
        push_header(&mut out, "FEN", &tree.startpos);
    }
    if !tree.headers.title.is_empty() {
        push_header(&mut out, "PikaXiangqiTitle", &tree.headers.title);
    }
    out.push('\n');

    let root = &tree.nodes[&tree.root];
    // 根节点注释/NAG（无着法，写在首着之前）
    if !root.comment.is_empty() {
        out.push_str(&format!("{{{} }}", root.comment));
    }
    if let Some(first) = root.children.first() {
        let tail: Vec<NodeId> = root.children.iter().skip(1).copied().collect();
        write_move(tree, *first, &tail, &mut out);
    }
    out.push(' ');
    out.push_str(result_token(&tree.headers.result));
    out.push('\n');
    out
}

fn push_header(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!("[{key} \"{value}\"]\n"));
}

fn result_token(result: &str) -> &str {
    if result.is_empty() {
        "*"
    } else {
        result
    }
}

/// 写出一个着法节点：着法 + NAG + 注释 + 父节点的变例（tail）+ 本节点变例 + 主线续着。
fn write_move(tree: &GameTree, node: NodeId, tail: &[NodeId], out: &mut String) {
    let n = &tree.nodes[&node];
    let prefix = if n.is_red() {
        format!("{}.", n.move_number())
    } else {
        format!("{}...", n.move_number())
    };
    out.push_str(&format!(
        " {prefix} {}",
        n.mv.map(|m| m.uci()).unwrap_or_default()
    ));
    for nag in &n.nags {
        out.push(' ');
        out.push_str(nag.symbol());
    }
    if !n.comment.is_empty() {
        out.push_str(&format!(" {{{}}}", n.comment));
    }
    // 父节点的变例（根级变例紧随首着）
    for v in tail {
        out.push_str(" (");
        write_move(tree, *v, &[], out);
        out.push(')');
    }
    // 本节点的变例
    for v in n.children.iter().skip(1) {
        out.push_str(" (");
        write_move(tree, *v, &[], out);
        out.push(')');
    }
    if let Some(c) = n.children.first() {
        write_move(tree, *c, &[], out);
    }
}

/// 从 PGN 文本导入棋谱树。
pub fn import(text: &str) -> Result<GameTree, String> {
    let (headers, startpos, tokens) = parse(text)?;
    let mut tree = GameTree::new(&startpos).map_err(|e| e.to_string())?;
    tree.headers = headers;
    let mut stack: Vec<NodeId> = Vec::new();
    let mut current = tree.root;
    let mut pending_var = false;
    for tok in tokens {
        match tok {
            Token::Move(uci, num) => {
                let m = Move::parse_uci(&uci).ok_or_else(|| format!("无法识别的着法：{uci}"))?;
                if pending_var {
                    // 变例首着：按回合前缀定位分支点；失败时沿祖先链回退（常见 PGN 习惯）
                    let branch = branch_point(&tree, current, num);
                    let node = tree
                        .insert_move_at(branch, m)
                        .or_else(|_| insert_along_ancestors(&mut tree, current, m))
                        .map_err(|e| e.to_string())?;
                    current = node;
                    pending_var = false;
                } else {
                    current = tree.insert_main_at(current, m).map_err(|e| e.to_string())?;
                }
            }
            Token::Comment(c) => {
                tree.set_comment_at(current, c).map_err(|e| e.to_string())?;
            }
            Token::Nag(sym) => {
                let nag =
                    Nag::from_symbol(&sym).ok_or_else(|| format!("无法识别的注释符号：{sym}"))?;
                tree.set_nag_at(current, nag, true)
                    .map_err(|e| e.to_string())?;
            }
            Token::OpenParen => {
                stack.push(current);
                pending_var = true;
            }
            Token::CloseParen => {
                current = stack.pop().ok_or("括号不匹配：多余的 )")?;
                pending_var = false;
            }
        }
    }
    if !stack.is_empty() {
        return Err("括号不匹配：缺少 )".to_string());
    }
    Ok(tree)
}

/// 根据回合前缀在祖先链上定位变例分支点（前缀缺失时取当前节点）。
fn branch_point(tree: &GameTree, current: NodeId, num: Option<MoveNumber>) -> NodeId {
    let Some(num) = num else {
        return current;
    };
    let mut id = Some(current);
    while let Some(nid) = id {
        let Some(n) = tree.node(nid).ok() else {
            break;
        };
        let matches = match num {
            MoveNumber::Red(mn) => n.side_to_move == Color::Red && n.fullmove_number == mn,
            MoveNumber::Black(mn) => n.side_to_move == Color::Black && n.fullmove_number == mn,
        };
        if matches {
            return nid;
        }
        id = n.parent;
    }
    current
}

/// 从 `from` 沿祖先链向上依次尝试插入（处理根级变例等 PGN 习惯）。
fn insert_along_ancestors(
    tree: &mut GameTree,
    from: NodeId,
    mv: Move,
) -> Result<NodeId, GameError> {
    let mut id = Some(from);
    while let Some(nid) = id {
        if let Ok(node) = tree.insert_move_at(nid, mv) {
            return Ok(node);
        }
        id = tree.node(nid).ok().and_then(|n| n.parent);
    }
    Err(GameError::IllegalMove(mv.uci()))
}

/// 解析头部与着法文本。
fn parse(text: &str) -> Result<(GameHeaders, String, Vec<Token>), String> {
    let mut headers = GameHeaders::default();
    let mut startpos = START_FEN.to_string();
    let mut movetext = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            if let Some((key, value)) = parse_header_tag(line) {
                match key.as_str() {
                    "White" => headers.red = value,
                    "Black" => headers.black = value,
                    "Event" => headers.event = value,
                    "Date" => headers.date = value,
                    "Result" => headers.result = value,
                    "FEN" => startpos = value,
                    "PikaXiangqiTitle" => headers.title = value,
                    _ => {}
                }
            }
        } else if !line.is_empty() {
            movetext.push_str(line);
            movetext.push(' ');
        }
    }
    let tokens = tokenize(&movetext)?;
    Ok((headers, startpos, tokens))
}

fn parse_header_tag(line: &str) -> Option<(String, String)> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?;
    let (key, rest) = inner.split_once(' ')?;
    let value = rest.trim().trim_start_matches('"').trim_end_matches('"');
    Some((key.to_string(), value.to_string()))
}

/// 把 NAG 代码映射为符号；未映射的返回 None（忽略）。
fn nag_from_code(code: u32) -> Option<&'static str> {
    match code {
        1 => Some("!"),
        2 => Some("?"),
        3 => Some("!!"),
        4 => Some("??"),
        5 => Some("!?"),
        6 => Some("?!"),
        10 => Some("="),
        13 => Some("~"),
        _ => None,
    }
}

fn is_result(tok: &str) -> bool {
    matches!(tok, "1-0" | "0-1" | "1/2-1/2" | "*")
}

/// 尝试把 "12." / "12..." / "12...e5" 拆成（回合前缀, 剩余着法文本）。
fn split_move_number(tok: &str) -> Option<(MoveNumber, String)> {
    let first_dot = tok.find('.')?;
    let num_part = &tok[..first_dot];
    if num_part.is_empty() || !num_part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u32 = num_part.parse().ok()?;
    let rest = &tok[first_dot..];
    if rest == "…" {
        return Some((MoveNumber::Black(n), String::new()));
    }
    if let Some(tail) = rest.strip_prefix("...") {
        return Some((MoveNumber::Black(n), tail.to_string()));
    }
    if let Some(tail) = rest.strip_prefix('.') {
        return Some((MoveNumber::Red(n), tail.to_string()));
    }
    None
}

/// 把着法文本拆成 token（保留回合前缀，供变例分支定位使用）。
fn tokenize(s: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    let mut pending_num: Option<MoveNumber> = None;
    while i < chars.len() {
        let c = chars[i];
        match c {
            c if c.is_whitespace() => i += 1,
            '{' => {
                let end = chars[i + 1..]
                    .iter()
                    .position(|&x| x == '}')
                    .map(|p| i + 1 + p)
                    .ok_or("注释未闭合：缺少 }")?;
                let comment: String = chars[i + 1..end].iter().collect();
                out.push(Token::Comment(comment.trim().to_string()));
                i = end + 1;
            }
            ';' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '(' => {
                out.push(Token::OpenParen);
                i += 1;
            }
            ')' => {
                out.push(Token::CloseParen);
                i += 1;
            }
            '$' => {
                let mut code = 0u32;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    code = code * 10 + (chars[i] as u32 - '0' as u32);
                    i += 1;
                }
                if let Some(sym) = nag_from_code(code) {
                    out.push(Token::Nag(sym.to_string()));
                }
            }
            '!' | '?' | '=' | '~' => {
                let start = i;
                while i < chars.len() && matches!(chars[i], '!' | '?' | '=' | '~') {
                    i += 1;
                }
                out.push(Token::Nag(chars[start..i].iter().collect()));
            }
            _ => {
                let start = i;
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && !matches!(chars[i], '(' | ')' | '{' | '}' | ';')
                {
                    i += 1;
                }
                let tok: String = chars[start..i].iter().collect();
                if let Some((num, rest)) = split_move_number(&tok) {
                    pending_num = Some(num);
                    if !rest.is_empty() {
                        let mut mv = rest;
                        while mv.len() > 4 && mv.ends_with(['!', '?', '=', '~']) {
                            mv.pop();
                        }
                        out.push(Token::Move(mv, pending_num.take()));
                    }
                    continue;
                }
                if tok == "..." || tok == "…" {
                    continue;
                }
                if tok.chars().all(|c| c.is_ascii_digit()) {
                    // 裸回合数（无点号），忽略
                    continue;
                }
                if is_result(&tok) {
                    continue;
                }
                let mut mv = tok;
                // 剥离着法尾部粘连的 NAG 符号（如 h2e2!?）
                while mv.len() > 4 && mv.ends_with(['!', '?', '=', '~']) {
                    mv.pop();
                }
                out.push(Token::Move(mv, pending_num.take()));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::Move;
    use crate::game::tree::GameTree;

    #[test]
    fn tokenize_simple_moves() {
        let toks = tokenize("1. h2e2 h7e7 2. h0g2 1-0").unwrap();
        let moves: Vec<String> = toks
            .iter()
            .filter_map(|t| match t {
                Token::Move(m, _) => Some(m.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(moves, vec!["h2e2", "h7e7", "h0g2"]);
    }

    #[test]
    fn tokenize_preserves_move_numbers() {
        let toks = tokenize("1. h2e2 (1... b9c7 2. h0g2) 1... h7e7").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::Move("h2e2".to_string(), Some(MoveNumber::Red(1))),
                Token::OpenParen,
                Token::Move("b9c7".to_string(), Some(MoveNumber::Black(1))),
                Token::Move("h0g2".to_string(), Some(MoveNumber::Red(2))),
                Token::CloseParen,
                Token::Move("h7e7".to_string(), Some(MoveNumber::Black(1))),
            ]
        );
    }

    #[test]
    fn tokenize_variations_comments_nags() {
        let toks = tokenize("1. h2e2 ! (1... h7h8 {黑方变例} ??) 1... h7e7 $1").unwrap();
        assert!(toks.contains(&Token::OpenParen));
        assert!(toks.contains(&Token::CloseParen));
        assert!(toks.contains(&Token::Comment("黑方变例".to_string())));
        assert!(toks.contains(&Token::Nag("!".to_string())));
        assert!(toks.contains(&Token::Nag("??".to_string())));
    }

    #[test]
    fn tokenize_rejects_unclosed_comment() {
        assert!(tokenize("1. h2e2 {未闭合").is_err());
    }

    #[test]
    fn move_number_splitting() {
        assert_eq!(
            split_move_number("1."),
            Some((MoveNumber::Red(1), String::new()))
        );
        assert_eq!(
            split_move_number("12..."),
            Some((MoveNumber::Black(12), String::new()))
        );
        assert_eq!(
            split_move_number("1...e5"),
            Some((MoveNumber::Black(1), "e5".to_string()))
        );
        assert_eq!(
            split_move_number("1.h2e2"),
            Some((MoveNumber::Red(1), "h2e2".to_string()))
        );
        assert_eq!(split_move_number("h2e2"), None);
        assert_eq!(split_move_number("1-0"), None);
        assert!(is_result("1-0"));
        assert!(is_result("1/2-1/2"));
    }

    #[test]
    fn nag_code_mapping() {
        assert_eq!(nag_from_code(1), Some("!"));
        assert_eq!(nag_from_code(5), Some("!?"));
        assert_eq!(nag_from_code(10), Some("="));
        assert_eq!(nag_from_code(99), None);
    }

    #[test]
    fn export_import_roundtrip_basic() {
        let mut tree = GameTree::new(START_FEN).unwrap();
        tree.headers.red = "红方".to_string();
        tree.headers.black = "黑方".to_string();
        tree.headers.event = "测试对局".to_string();
        tree.headers.date = "2026-08-22".to_string();
        tree.headers.result = "1-0".to_string();
        tree.headers.title = "标题".to_string();
        let n1 = tree.insert_move(Move::parse_uci("h2e2").unwrap()).unwrap();
        tree.set_comment_at(n1, "中炮".to_string()).unwrap();
        tree.set_nag_at(n1, Nag::Good, true).unwrap();
        tree.insert_move(Move::parse_uci("h7e7").unwrap()).unwrap();
        tree.go_to_start().unwrap();
        let v = tree.insert_move(Move::parse_uci("b0c2").unwrap()).unwrap();
        tree.set_comment_at(v, "马二进三变例".to_string()).unwrap();

        let pgn = export(&tree);
        let imported = import(&pgn).expect("import");
        assert_eq!(imported.headers.red, "红方");
        assert_eq!(imported.headers.title, "标题");
        assert_eq!(imported.headers.result, "1-0");
        let main = imported.main_line();
        assert_eq!(main.len(), 3);
        let root_children = imported.node(imported.root).unwrap().children.clone();
        assert_eq!(root_children.len(), 2);
        let a = imported.node(root_children[0]).unwrap();
        assert_eq!(a.comment, "中炮");
        assert_eq!(a.nags, vec![Nag::Good]);
        let vnode = imported.node(root_children[1]).unwrap();
        assert_eq!(vnode.comment, "马二进三变例");
        assert_eq!(vnode.mv, Some(Move::parse_uci("b0c2").unwrap()));
    }
}
