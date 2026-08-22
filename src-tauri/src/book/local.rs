//! 本地开局库（完全离线）。
//!
//! 存储：内存 `HashMap<u64, Vec<BookMove>>`（键为局面 Zobrist 哈希），可选 JSON 持久化。
//! SQLite 存储随 DB 阶段落地（见 docs/book.md §2），与本实现共享 `BookProvider` 接口，
//! 上层（`BookChain`/命令层）不感知存储后端。

use std::collections::HashMap;
use std::io::{Read, Write};

use crate::board::rules::legal_moves;
use crate::board::types::{Move, Position};
use crate::board::zobrist::zobrist_key;
use serde::{Deserialize, Serialize};

use super::{BookError, BookMove, BookProvider, BookStats};
use std::cmp::Ordering;

/// 本地开局库：内存 + JSON 持久化。
#[derive(Default)]
pub struct LocalBookProvider {
    entries: HashMap<u64, Vec<BookMove>>,
}

/// JSON 文件格式（version 1）。
#[derive(Serialize, Deserialize)]
struct BookFile {
    version: u32,
    entries: Vec<BookEntryJson>,
}

#[derive(Serialize, Deserialize)]
struct BookEntryJson {
    key: u64,
    mv: String,
    count: u32,
    stats: Option<StatsJson>,
}

#[derive(Serialize, Deserialize)]
struct StatsJson {
    wins: u32,
    draws: u32,
    losses: u32,
}

impl LocalBookProvider {
    pub fn new() -> Self {
        LocalBookProvider {
            entries: HashMap::new(),
        }
    }

    /// 收录的局面数（不同 Zobrist 键数量）。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 添加一条候选着法（以局面为键）；重复着法合并计数与统计。
    pub fn add_entry(&mut self, pos: &Position, mv: Move, count: u32, stats: Option<BookStats>) {
        self.add_entry_by_key(zobrist_key(pos), mv, count, stats);
    }

    /// 按哈希键添加（JSON 加载等无局面场景使用）。
    pub fn add_entry_by_key(&mut self, key: u64, mv: Move, count: u32, stats: Option<BookStats>) {
        let list = self.entries.entry(key).or_default();
        if let Some(existing) = list.iter_mut().find(|e| e.mv == mv) {
            existing.count = existing.count.saturating_add(count);
            if let (Some(a), Some(b)) = (existing.stats.as_mut(), stats) {
                a.wins = a.wins.saturating_add(b.wins);
                a.draws = a.draws.saturating_add(b.draws);
                a.losses = a.losses.saturating_add(b.losses);
            }
            return;
        }
        list.push(BookMove::new(mv, count, stats));
    }

    /// 序列化为 JSON 文本。
    pub fn to_json(&self) -> Result<String, BookError> {
        let entries = self
            .entries
            .iter()
            .flat_map(|(key, list)| {
                list.iter().map(move |bm| BookEntryJson {
                    key: *key,
                    mv: bm.mv.uci(),
                    count: bm.count,
                    stats: bm.stats.map(|s| StatsJson {
                        wins: s.wins,
                        draws: s.draws,
                        losses: s.losses,
                    }),
                })
            })
            .collect();
        let file = BookFile {
            version: 1,
            entries,
        };
        serde_json::to_string_pretty(&file).map_err(|e| BookError::Corrupt(e.to_string()))
    }

    /// 从 JSON 文本加载（合并进当前数据）。
    pub fn load_json(&mut self, text: &str) -> Result<(), BookError> {
        let file: BookFile =
            serde_json::from_str(text).map_err(|e| BookError::Corrupt(e.to_string()))?;
        if file.version != 1 {
            return Err(BookError::Corrupt(format!(
                "不支持的开局库版本：{}",
                file.version
            )));
        }
        for e in file.entries {
            let mv = Move::parse_uci(&e.mv)
                .ok_or_else(|| BookError::Corrupt(format!("非法着法：{}", e.mv)))?;
            let stats = e.stats.map(|s| BookStats {
                wins: s.wins,
                draws: s.draws,
                losses: s.losses,
            });
            self.add_entry_by_key(e.key, mv, e.count, stats);
        }
        Ok(())
    }

    /// 写入 JSON 到 writer。
    pub fn save_to<W: Write>(&self, w: &mut W) -> Result<(), BookError> {
        let json = self.to_json()?;
        w.write_all(json.as_bytes())
            .map_err(|e| BookError::Corrupt(e.to_string()))
    }

    /// 从 reader 读取 JSON 并合并。
    pub fn load_from<R: Read>(&mut self, r: &mut R) -> Result<(), BookError> {
        let mut text = String::new();
        r.read_to_string(&mut text)
            .map_err(|e| BookError::Corrupt(e.to_string()))?;
        self.load_json(&text)
    }
}

impl BookProvider for LocalBookProvider {
    fn name(&self) -> &'static str {
        "local"
    }

    fn lookup(&self, pos: &Position) -> Result<Vec<BookMove>, BookError> {
        let key = zobrist_key(pos);
        let Some(entries) = self.entries.get(&key) else {
            return Ok(Vec::new());
        };
        let legal: Vec<Move> = legal_moves(pos);
        let mut out: Vec<BookMove> = entries
            .iter()
            .filter(|e| legal.contains(&e.mv))
            .cloned()
            .collect();
        // 按推荐度降序：得分 → 出现次数 → 着法字典序（确定性排序）。
        out.sort_by(|a, b| {
            b.score()
                .partial_cmp(&a.score())
                .unwrap_or(Ordering::Equal)
                .then_with(|| b.count.cmp(&a.count))
                .then_with(|| a.mv.uci().cmp(&b.mv.uci()))
        });
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::fen::parse_fen;
    use crate::board::types::START_FEN;

    fn mv(uci: &str) -> Move {
        Move::parse_uci(uci).unwrap()
    }

    fn stats(w: u32, d: u32, l: u32) -> BookStats {
        BookStats {
            wins: w,
            draws: d,
            losses: l,
        }
    }

    fn start() -> Position {
        parse_fen(START_FEN).unwrap()
    }

    #[test]
    fn lookup_returns_sorted_candidates_with_stats() {
        let pos = start();
        let mut book = LocalBookProvider::new();
        book.add_entry(&pos, mv("h2e2"), 100, Some(stats(40, 30, 30))); // score 0.55
        book.add_entry(&pos, mv("b0c2"), 10, Some(stats(9, 0, 1))); // score 0.9
        book.add_entry(&pos, mv("h0g2"), 50, None); // score 0，count 50

        let moves = book.lookup(&pos).unwrap();
        assert_eq!(moves.len(), 3);
        // 排序：b0c2(0.9) > h2e2(0.55) > h0g2(0, 50)
        assert_eq!(moves[0].mv, mv("b0c2"));
        assert_eq!(moves[1].mv, mv("h2e2"));
        assert_eq!(moves[2].mv, mv("h0g2"));
        assert_eq!(moves[0].stats.unwrap().total(), 10);
        assert_eq!(moves[2].count, 50);
    }

    #[test]
    fn lookup_unknown_position_is_empty() {
        let pos = start();
        let book = LocalBookProvider::new();
        assert!(book.lookup(&pos).unwrap().is_empty());
    }

    #[test]
    fn lookup_filters_illegal_moves() {
        let pos = start();
        let mut book = LocalBookProvider::new();
        // h7e7 是黑方着法，红先局面下非法 → 查询时应被过滤
        book.add_entry(&pos, mv("h7e7"), 10, Some(stats(9, 0, 1)));
        book.add_entry(&pos, mv("h2e2"), 10, Some(stats(9, 0, 1)));
        let moves = book.lookup(&pos).unwrap();
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].mv, mv("h2e2"));
    }

    #[test]
    fn add_entry_merges_duplicates() {
        let pos = start();
        let mut book = LocalBookProvider::new();
        book.add_entry(&pos, mv("h2e2"), 10, Some(stats(5, 3, 2)));
        book.add_entry(&pos, mv("h2e2"), 5, Some(stats(2, 1, 2)));
        let moves = book.lookup(&pos).unwrap();
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].count, 15);
        assert_eq!(moves[0].stats.unwrap(), stats(7, 4, 4));
    }

    #[test]
    fn json_roundtrip_preserves_entries() {
        let pos = start();
        let mut book = LocalBookProvider::new();
        book.add_entry(&pos, mv("h2e2"), 100, Some(stats(40, 30, 30)));
        book.add_entry(&pos, mv("b0c2"), 10, None);

        let json = book.to_json().unwrap();
        let mut loaded = LocalBookProvider::new();
        loaded.load_json(&json).unwrap();

        assert_eq!(book.lookup(&pos).unwrap(), loaded.lookup(&pos).unwrap());
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn json_rejects_corrupt_data() {
        let mut book = LocalBookProvider::new();
        assert!(book.load_json("not json").is_err());
        assert!(book.load_json(r#"{"version": 99, "entries": []}"#).is_err());
        assert!(book
            .load_json(r#"{"version": 1, "entries": [{"key": 1, "mv": "xyz", "count": 1}]}"#)
            .is_err());
    }

    #[test]
    fn load_from_reader_merges() {
        let pos = start();
        let mut book = LocalBookProvider::new();
        book.add_entry(&pos, mv("h2e2"), 10, None);
        let mut buf = Vec::new();
        book.save_to(&mut buf).unwrap();

        let mut other = LocalBookProvider::new();
        other.load_from(&mut buf.as_slice()).unwrap();
        assert_eq!(other.lookup(&pos).unwrap(), book.lookup(&pos).unwrap());
    }
}
