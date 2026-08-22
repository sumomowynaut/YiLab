//! 注释符号（NAG）。

use serde::{Deserialize, Serialize};

/// 常用中国象棋注释符号（对应 PGN NAG）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nag {
    Good,        // !
    Mistake,     // ?
    Brilliant,   // !!
    Blunder,     // ??
    Interesting, // !?
    Dubious,     // ?!
    Equal,       // =
    Unclear,     // ~
}

impl Nag {
    pub fn symbol(self) -> &'static str {
        match self {
            Nag::Good => "!",
            Nag::Mistake => "?",
            Nag::Brilliant => "!!",
            Nag::Blunder => "??",
            Nag::Interesting => "!?",
            Nag::Dubious => "?!",
            Nag::Equal => "=",
            Nag::Unclear => "~",
        }
    }

    pub fn from_symbol(s: &str) -> Option<Nag> {
        match s {
            "!" => Some(Nag::Good),
            "?" => Some(Nag::Mistake),
            "!!" => Some(Nag::Brilliant),
            "??" => Some(Nag::Blunder),
            "!?" => Some(Nag::Interesting),
            "?!" => Some(Nag::Dubious),
            "=" => Some(Nag::Equal),
            "~" => Some(Nag::Unclear),
            _ => None,
        }
    }
}
