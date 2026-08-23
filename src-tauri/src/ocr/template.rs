//! 传统 CV 模板匹配识别器（确定性、无外部权重）。
//!
//! 步骤：解码图片 → 按棋盘底色检测网格区域 → 按方向枚举 90 格 → 每格与
//! 16 枚棋子模板 + 空格模板做匹配 → 输出格子分类、方向与置信度。
//! 本实现**只负责识别**，规则校验在 `super::recognize` 管线中由本地 Rust 完成。
//!
//! 局限：模板为程序生成的字母圆盘，真实截图（汉字棋子、不同配色）识别率有限，
//! 需人工校正；真实模型迭代见 docs/ocr.md §3（`NEEDS_VERIFICATION`）。

use image::{imageops, Rgba, RgbaImage};

use crate::board::types::{Color, Piece, PieceKind};

use super::render::{self, BOARD_BG};
use super::{BoardOrientation, OcrError, OcrInput, RawRecognition, RecognizedCell};

/// 探测区占格子的比例（避开网格线）。
const PROBE_RATIO: f32 = 0.6;
/// 圆盘半径与格子边长的比例（与 render.rs 一致）。
const DISC_RATIO: f32 = render::DISC_RATIO;
/// 判定为空格的匹配阈值。
const EMPTY_THRESHOLD: f32 = 0.88;
/// 判定为某棋子的匹配阈值。
const PIECE_THRESHOLD: f32 = 0.70;
/// 背景色容差（每通道）。
const BG_TOLERANCE: i32 = 18;
/// 棋盘包围盒外扩（抵消网格线覆盖导致的收缩），像素。
const BBOX_EXPAND: u32 = 2;
/// 分类阶段的小偏移搜索半径（抵消包围盒估计误差）。
const CLASSIFY_SHIFT: i32 = 2;
/// 方向判定阶段的偏移搜索半径（只需粗对齐）。
const ORIENT_SHIFT: i32 = 1;

/// 模板匹配识别器。
pub struct TemplateRecognizer;

impl Default for TemplateRecognizer {
    fn default() -> Self {
        Self
    }
}

impl TemplateRecognizer {
    pub fn new() -> Self {
        Self
    }
}

/// 颜色是否接近棋盘底色。
fn is_board_bg(c: &[u8; 3]) -> bool {
    let bg = BOARD_BG.0;
    (c[0] as i32 - bg[0] as i32).abs() <= BG_TOLERANCE
        && (c[1] as i32 - bg[1] as i32).abs() <= BG_TOLERANCE
        && (c[2] as i32 - bg[2] as i32).abs() <= BG_TOLERANCE
}

/// 检测棋盘区域（棋盘底色像素包围盒，并向外扩以抵消网格线收缩）。
fn detect_board_rect(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = (img.width(), img.height());
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut count = 0u64;
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y).0;
            if is_board_bg(&[p[0], p[1], p[2]]) {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                count += 1;
            }
        }
    }
    if count == 0 || max_x <= min_x || max_y <= min_y {
        return None;
    }
    let bw = max_x - min_x + 1;
    let bh = max_y - min_y + 1;
    if bw < 9 * 8 || bh < 10 * 8 {
        return None;
    }
    let area = bw as u64 * bh as u64;
    if count * 10 < area * 3 {
        return None;
    }
    let x0 = min_x.saturating_sub(BBOX_EXPAND);
    let y0 = min_y.saturating_sub(BBOX_EXPAND);
    let x1 = (max_x + BBOX_EXPAND).min(w - 1);
    let y1 = (max_y + BBOX_EXPAND).min(h - 1);
    Some((x0, y0, x1, y1))
}

/// 生成某棋子的模板（棋盘底色 + 圆盘字母），尺寸与探测区一致。
fn make_piece_template(size: u32, piece: Piece) -> RgbaImage {
    let bg = BOARD_BG.0;
    let mut img = RgbaImage::from_pixel(size, size, Rgba([bg[0], bg[1], bg[2], 255]));
    let radius = size as f32 * DISC_RATIO / PROBE_RATIO;
    render::draw_piece(
        &mut img,
        size as f32 / 2.0,
        size as f32 / 2.0,
        radius,
        piece,
    );
    img
}

/// 空格模板（纯棋盘底色）。
fn make_empty_template(size: u32) -> RgbaImage {
    let bg = BOARD_BG.0;
    RgbaImage::from_pixel(size, size, Rgba([bg[0], bg[1], bg[2], 255]))
}

/// 匹配分数 [0,1]：1 = 完全相同（RGB 逐像素）。
fn match_score(a: &RgbaImage, b: &RgbaImage) -> f32 {
    debug_assert_eq!(a.dimensions(), b.dimensions());
    let mut sum = 0u64;
    let mut n = 0u64;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let (ra, ga, ba) = (pa.0[0] as i32, pa.0[1] as i32, pa.0[2] as i32);
        let (rb, gb, bb) = (pb.0[0] as i32, pb.0[1] as i32, pb.0[2] as i32);
        sum += ((ra - rb).abs() + (ga - gb).abs() + (ba - bb).abs()) as u64;
        n += 1;
    }
    1.0 - (sum as f32 / (n as f32 * 3.0 * 255.0))
}

/// 所有棋子的遍历顺序。
fn all_pieces() -> Vec<Piece> {
    let mut out = Vec::with_capacity(16);
    for color in [Color::Red, Color::Black] {
        for kind in [
            PieceKind::King,
            PieceKind::Advisor,
            PieceKind::Elephant,
            PieceKind::Horse,
            PieceKind::Rook,
            PieceKind::Cannon,
            PieceKind::Pawn,
        ] {
            out.push(Piece { color, kind });
        }
    }
    out
}

/// 提取格子中心探测区（原生尺寸裁剪，不做缩放）。
fn crop_probe(img: &RgbaImage, cx: f32, cy: f32, side: u32) -> RgbaImage {
    let x0 = (cx - side as f32 / 2.0).round().max(0.0) as u32;
    let y0 = (cy - side as f32 / 2.0).round().max(0.0) as u32;
    let x0 = x0.min(img.width().saturating_sub(side));
    let y0 = y0.min(img.height().saturating_sub(side));
    imageops::crop_imm(img, x0, y0, side, side).to_image()
}

/// 棋盘网格几何（检测得到）。
#[derive(Clone, Copy)]
struct Grid {
    cell: f32,
    left: f32,
    top: f32,
    side: u32,
}

/// 在给定方向下，把 (rank, file) 映射到图像坐标（格中心）。
fn cell_center(rank: u8, file: u8, g: Grid, orientation: BoardOrientation) -> (f32, f32) {
    let (r, f) = match orientation {
        BoardOrientation::Normal => (9 - rank, file),
        BoardOrientation::Flipped180 => (rank, 8 - file),
    };
    (
        g.left + (f as f32 + 0.5) * g.cell,
        g.top + (r as f32 + 0.5) * g.cell,
    )
}

/// 返回（最佳模板：None=空格 / Some(棋子), 分数）。
fn best_score(
    probe: &RgbaImage,
    empty: &RgbaImage,
    pieces: &[(Piece, RgbaImage)],
) -> (Option<Piece>, f32) {
    let empty_score = match_score(probe, empty);
    let mut best = (None, empty_score);
    for (piece, tpl) in pieces {
        let s = match_score(probe, tpl);
        if s > best.1 {
            best = (Some(*piece), s);
        }
    }
    best
}

/// 带小偏移搜索的最佳匹配：在小窗口内取最高分，抵消包围盒估计误差。
fn best_score_shifted(
    img: &RgbaImage,
    cx: f32,
    cy: f32,
    side: u32,
    empty: &RgbaImage,
    pieces: &[(Piece, RgbaImage)],
    shift: i32,
) -> (Option<Piece>, f32) {
    let mut best: (Option<Piece>, f32) = (None, f32::MIN);
    for dy in -shift..=shift {
        for dx in -shift..=shift {
            let probe = crop_probe(img, cx + dx as f32, cy + dy as f32, side);
            let (p, s) = best_score(&probe, empty, pieces);
            if s > best.1 {
                best = (p, s);
            }
        }
    }
    best
}

/// 计算某方向 + 模板集合下的总分（方向判定用）。
fn total_score(
    img: &RgbaImage,
    g: Grid,
    orientation: BoardOrientation,
    empty: &RgbaImage,
    pieces: &[(Piece, RgbaImage)],
    shift: i32,
) -> f32 {
    let mut total = 0.0f32;
    for rank in 0..10u8 {
        for file in 0..9u8 {
            let (cx, cy) = cell_center(rank, file, g, orientation);
            total += best_score_shifted(img, cx, cy, g.side, empty, pieces, shift).1;
        }
    }
    total
}

impl super::OcrEngine for TemplateRecognizer {
    fn name(&self) -> &'static str {
        "template"
    }

    fn recognize_cells(&self, input: &OcrInput) -> Result<RawRecognition, OcrError> {
        let img = image::load_from_memory(&input.image)
            .map_err(|e| OcrError::ImageDecode(e.to_string()))?
            .to_rgba8();
        if img.width() < 9 * 8 || img.height() < 10 * 8 {
            return Err(OcrError::InvalidImage("图片尺寸过小".to_string()));
        }
        let (bx0, by0, bx1, by1) = detect_board_rect(&img)
            .ok_or_else(|| OcrError::BoardNotFound("未找到棋盘底色区域".to_string()))?;
        let cell_w = (bx1 - bx0 + 1) as f32 / 9.0;
        let cell_h = (by1 - by0 + 1) as f32 / 10.0;
        if cell_w < 8.0 || cell_h < 8.0 {
            return Err(OcrError::BoardNotFound("棋盘格过小".to_string()));
        }
        let cell = cell_w.max(cell_h);
        let grid = Grid {
            cell,
            left: bx0 as f32,
            top: by0 as f32,
            side: ((cell * PROBE_RATIO).round() as u32).max(8),
        };
        let probe_side = grid.side;

        // 模板：正立 + 旋转 180°（识别倒置截图用）
        let empty_tpl = make_empty_template(probe_side);
        let upright: Vec<(Piece, RgbaImage)> = all_pieces()
            .into_iter()
            .map(|p| (p, make_piece_template(probe_side, p)))
            .collect();
        let rotated: Vec<(Piece, RgbaImage)> = upright
            .iter()
            .map(|(p, t)| (*p, imageops::rotate180(t)))
            .collect();

        // 方向判定：正立模板(Normal) vs 旋转模板(Flipped180)
        let upright_total = total_score(
            &img,
            grid,
            BoardOrientation::Normal,
            &empty_tpl,
            &upright,
            ORIENT_SHIFT,
        );
        let rotated_total = total_score(
            &img,
            grid,
            BoardOrientation::Flipped180,
            &empty_tpl,
            &rotated,
            ORIENT_SHIFT,
        );
        let orientation = if rotated_total > upright_total {
            BoardOrientation::Flipped180
        } else {
            BoardOrientation::Normal
        };
        let (pieces, orientation) = match orientation {
            BoardOrientation::Normal => (&upright, BoardOrientation::Normal),
            BoardOrientation::Flipped180 => (&rotated, BoardOrientation::Flipped180),
        };

        // 最终分类
        let mut cells = Vec::with_capacity(90);
        let mut total_conf = 0.0f32;
        for rank in 0..10u8 {
            for file in 0..9u8 {
                let (cx, cy) = cell_center(rank, file, grid, orientation);
                let (best_piece, score) =
                    best_score_shifted(&img, cx, cy, grid.side, &empty_tpl, pieces, CLASSIFY_SHIFT);
                total_conf += score;
                let (piece, uncertain) = match best_piece {
                    None => (None, score < EMPTY_THRESHOLD),
                    Some(p) => {
                        if score >= PIECE_THRESHOLD {
                            (Some(p), false)
                        } else {
                            (None, true)
                        }
                    }
                };
                cells.push(RecognizedCell {
                    rank,
                    file,
                    piece,
                    confidence: score,
                    uncertain,
                });
            }
        }

        Ok(RawRecognition {
            cells,
            orientation,
            side_to_move: None,
            overall_confidence: total_conf / 90.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::fen::parse_fen;
    use crate::board::types::{Position, START_FEN};
    use crate::ocr::{recognize, OcrInput};

    #[test]
    fn recognize_startpos_normal() {
        let start = parse_fen(START_FEN).unwrap();
        let png = render::render_screenshot_png(&start, BoardOrientation::Normal, 48, 24);
        let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();
        assert_eq!(out.orientation, BoardOrientation::Normal);
        assert!(out.confidence > 0.9);
        assert_eq!(out.fen, START_FEN);
        assert!(out.cells.iter().all(|c| !c.uncertain));
    }

    #[test]
    fn recognize_startpos_flipped180() {
        let start = parse_fen(START_FEN).unwrap();
        let png = render::render_screenshot_png(&start, BoardOrientation::Flipped180, 48, 24);
        let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();
        assert_eq!(out.orientation, BoardOrientation::Flipped180);
        assert_eq!(out.fen, START_FEN);
        assert!(out.confidence > 0.9);
    }

    #[test]
    fn empty_board_recognized_as_empty() {
        let pos = Position::default();
        let png = render::render_screenshot_png(&pos, BoardOrientation::Normal, 48, 24);
        let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();
        assert!(out.cells.iter().all(|c| c.piece.is_none()));
        // 空棋盘缺将帅 → 规则校验问题
        assert!(out.issues.iter().any(|i| i.message.contains("缺少将/帅")));
    }
}
