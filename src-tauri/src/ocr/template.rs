//! 传统 CV 截图识别器（确定性、无外部权重）。
//!
//! 与早期「字母圆盘模板」不同，本实现改为对真实象棋截图更友好的策略：
//! 1. 通过棋盘底色检测网格区域；
//! 2. 在每个格子的中心圆形探测区内，用「相对局部中位色」提取前景（棋子文字）；
//! 3. 由前景颜色判断红/黑，并用内嵌汉字字形模板做原分辨率掩码匹配判断兵种；
//! 4. 同时枚举正立/旋转 180° 两种方向，取匹配更好者作为棋盘方向。
//!
//! 规则校验仍在 `super::recognize` 管线中由本地 Rust 完成，本模块不做棋规判断。

use image::{imageops, RgbaImage};

use crate::board::types::{Color, Piece, PieceKind};

use super::glyphs::{self, Glyph};
use super::render::BOARD_BG;
use super::{BoardOrientation, OcrError, OcrInput, RawRecognition, RecognizedCell};

/// 中心探测区占格子的比例（避开网格线）。
const PROBE_RATIO: f32 = 0.64;
/// 棋盘底色容差（每通道）。
const BG_TOLERANCE: i32 = 18;
/// 棋盘包围盒外扩（抵消网格线覆盖导致的收缩），像素。
const BBOX_EXPAND: u32 = 2;
/// 前景判定阈值：与局部中位色的 RGB 平均距离。
const FG_DISTANCE: i32 = 28;
/// 中心探测区内前景像素占比超过该值即认为有棋子。
const OCCUPIED_RATIO: f32 = 0.10;
/// 字形匹配分数低于该值才标记为「不确定」。
const TYPE_THRESHOLD: f32 = 0.42;
/// 中心圆形掩码半径占探测区半边的比例（避开棋子圆盘外圈描边）。
const MASK_RATIO: f32 = 0.90;
/// 字形模板宽度相对探测区边长的候选比例（覆盖不同截图缩放/留白）。
const GLYPH_SIDE_RATIOS: [f32; 3] = [0.78, 0.825, 0.86];

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

fn is_board_bg(c: &[u8; 3], bg: [u8; 3]) -> bool {
    (c[0] as i32 - bg[0] as i32).abs() <= BG_TOLERANCE
        && (c[1] as i32 - bg[1] as i32).abs() <= BG_TOLERANCE
        && (c[2] as i32 - bg[2] as i32).abs() <= BG_TOLERANCE
}

fn dominant_bg_color(img: &RgbaImage) -> [u8; 3] {
    use std::collections::HashMap;
    let mut buckets: HashMap<(u8, u8, u8), u32> = HashMap::new();
    for y in (0..img.height()).step_by(3) {
        for x in (0..img.width()).step_by(3) {
            let p = img.get_pixel(x, y).0;
            let (r, g, b) = (p[0], p[1], p[2]);
            if r < 24 && g < 24 && b < 24 {
                continue;
            }
            if r > 235 && g > 235 && b > 235 {
                continue;
            }
            let key = ((r / 12) * 12, (g / 12) * 12, (b / 12) * 12);
            *buckets.entry(key).or_insert(0) += 1;
        }
    }
    buckets
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|((r, g, b), _)| [r, g, b])
        .unwrap_or(BOARD_BG.0)
}

fn detect_board_rect(img: &RgbaImage, bg: [u8; 3]) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = (img.width(), img.height());
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut count = 0u64;
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y).0;
            if is_board_bg(&[p[0], p[1], p[2]], bg) {
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
    let non_bg = area.saturating_sub(count);
    if non_bg < (area / 100).max(1) {
        return None;
    }
    let x0 = min_x.saturating_sub(BBOX_EXPAND);
    let y0 = min_y.saturating_sub(BBOX_EXPAND);
    let x1 = (max_x + BBOX_EXPAND).min(w - 1);
    let y1 = (max_y + BBOX_EXPAND).min(h - 1);
    Some((x0, y0, x1, y1))
}

fn median_color(img: &RgbaImage) -> [u8; 3] {
    let n = (img.width() * img.height()) as usize;
    if n == 0 {
        return [0, 0, 0];
    }
    let mut r = Vec::with_capacity(n);
    let mut g = Vec::with_capacity(n);
    let mut b = Vec::with_capacity(n);
    for p in img.pixels() {
        r.push(p.0[0]);
        g.push(p.0[1]);
        b.push(p.0[2]);
    }
    r.sort_unstable();
    g.sort_unstable();
    b.sort_unstable();
    let mid = n / 2;
    [r[mid], g[mid], b[mid]]
}

fn is_red(r: u8, g: u8, b: u8) -> bool {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    r > 140 && g * 4 < r * 3 && b * 4 < r * 3
}

fn is_dark_neutral(r: u8, g: u8, b: u8) -> bool {
    let (r, g, b) = (r as i32, g as i32, b as i32);
    r.max(g).max(b) < 130 && (r - g).abs() < 50 && (r - b).abs() < 50 && (g - b).abs() < 50
}

struct CropInfo {
    occupied: bool,
    color: Color,
    fg_ratio: f32,
    mask: Vec<bool>,
}

fn analyze_crop(crop: &RgbaImage) -> CropInfo {
    let med = median_color(crop);
    let (mr, mg, mb) = (med[0] as i32, med[1] as i32, med[2] as i32);
    let cw = crop.width();
    let ch = crop.height();
    let n = (cw * ch) as usize;
    let cx = cw as f32 / 2.0;
    let cy = ch as f32 / 2.0;
    // 只取中心圆形区域，避免棋子圆盘外圈描边被当成文字前景。
    let mask_radius = (cw.min(ch) as f32 * MASK_RATIO / 2.0).max(1.0);
    let mask_radius_sq = mask_radius * mask_radius;
    let mut fg = 0usize;
    let mut masked = 0usize;
    let mut red_votes = 0usize;
    let mut dark_votes = 0usize;
    let mut sum_r = 0u32;
    let mut sum_g = 0u32;
    let mut sum_b = 0u32;
    let mut fg_cx_sum = 0f32;
    let mut fg_cy_sum = 0f32;
    let mut mask = vec![false; n];

    for (i, p) in crop.pixels().enumerate() {
        let px = (i % cw as usize) as f32;
        let py = (i / cw as usize) as f32;
        let dx = px - cx;
        let dy = py - cy;
        if dx * dx + dy * dy > mask_radius_sq {
            continue;
        }
        masked += 1;
        let (r, g, b) = (p.0[0] as i32, p.0[1] as i32, p.0[2] as i32);
        let d = ((r - mr).abs() + (g - mg).abs() + (b - mb).abs()) / 3;
        if d > FG_DISTANCE {
            fg += 1;
            mask[i] = true;
            sum_r += p.0[0] as u32;
            sum_g += p.0[1] as u32;
            sum_b += p.0[2] as u32;
            fg_cx_sum += px;
            fg_cy_sum += py;
            if is_red(p.0[0], p.0[1], p.0[2]) {
                red_votes += 1;
            } else if is_dark_neutral(p.0[0], p.0[1], p.0[2]) {
                dark_votes += 1;
            }
        }
    }

    let fg_ratio = if masked == 0 {
        0.0
    } else {
        fg as f32 / masked as f32
    };
    // 真实棋子的前景（文字）应集中在探测区中心；坐标数字/网格线等噪声会偏向外侧。
    let centered = if fg == 0 {
        true
    } else {
        let fcx = fg_cx_sum / fg as f32;
        let fcy = fg_cy_sum / fg as f32;
        let dist = ((fcx - cx).powi(2) + (fcy - cy).powi(2)).sqrt();
        dist <= mask_radius * 0.55
    };
    // 前景颜色必须与「红」或「暗中性（黑）」之一吻合，或中位色本身是棋子圆盘色，
    // 否则视为网格线/噪声，而不是棋子。
    let color_votes_ok = fg > 0 && (red_votes.max(dark_votes) as f32) >= fg as f32 * 0.5;
    let med_is_piece = is_red(med[0], med[1], med[2]) || is_dark_neutral(med[0], med[1], med[2]);
    let occupied = fg_ratio > OCCUPIED_RATIO && centered && (med_is_piece || color_votes_ok);

    let color = if occupied {
        if is_red(med[0], med[1], med[2]) {
            Color::Red
        } else if is_dark_neutral(med[0], med[1], med[2]) {
            Color::Black
        } else if fg > 0 {
            let avg = [sum_r / fg as u32, sum_g / fg as u32, sum_b / fg as u32];
            let avg = [
                avg[0].min(255) as u8,
                avg[1].min(255) as u8,
                avg[2].min(255) as u8,
            ];
            if is_red(avg[0], avg[1], avg[2]) {
                Color::Red
            } else if is_dark_neutral(avg[0], avg[1], avg[2]) {
                Color::Black
            } else if red_votes >= dark_votes {
                Color::Red
            } else {
                Color::Black
            }
        } else {
            Color::Red
        }
    } else {
        Color::Red
    };

    CropInfo {
        occupied,
        color,
        fg_ratio,
        mask,
    }
}

fn rotate180_mask(mask: &[bool], side: u32) -> Vec<bool> {
    let s = side as usize;
    let mut out = vec![false; mask.len()];
    for y in 0..s {
        for x in 0..s {
            out[y * s + x] = mask[(s - 1 - y) * s + (s - 1 - x)];
        }
    }
    out
}

/// 生成某兵种字形模板掩码（原分辨率，含多个缩放比例）。
fn make_glyph_mask(side: u32, glyph: &Glyph, ratio: f32) -> Vec<bool> {
    let g = glyphs::GLYPH_GRID;
    let glyph_side = side as f32 * ratio;
    let scale = ((glyph_side / g as f32) as i32).max(1);
    let x0 = ((side as f32 - glyph_side) / 2.0).round() as i32;
    let y0 = x0;
    let mut mask = vec![false; (side * side) as usize];
    for gy in 0..g {
        for gx in 0..g {
            if !glyph.get(gx, gy) {
                continue;
            }
            for sy in 0..scale {
                for sx in 0..scale {
                    let px = x0 + gx as i32 * scale + sx;
                    let py = y0 + gy as i32 * scale + sy;
                    if px >= 0 && py >= 0 && px < side as i32 && py < side as i32 {
                        mask[py as usize * side as usize + px as usize] = true;
                    }
                }
            }
        }
    }
    mask
}

/// 允许 ±1 像素平移的掩码匹配，抵消包围盒估计带来的亚像素对齐误差。
fn mask_dice_shifted(a: &[bool], b: &[bool], side: u32) -> f32 {
    let s = side as i32;
    let mut best = 0.0f32;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            let mut inter = 0usize;
            let mut a_area = 0usize;
            let mut b_area = 0usize;
            for y in 0..s {
                for x in 0..s {
                    let sx = x + dx;
                    let sy = y + dy;
                    if sx < 0 || sy < 0 || sx >= s || sy >= s {
                        continue;
                    }
                    let av = a[(y * s + x) as usize];
                    let bv = b[(sy * s + sx) as usize];
                    if av {
                        a_area += 1;
                    }
                    if bv {
                        b_area += 1;
                    }
                    if av && bv {
                        inter += 1;
                    }
                }
            }
            if a_area == 0 || b_area == 0 {
                continue;
            }
            let score = 2.0 * inter as f32 / (a_area + b_area) as f32;
            best = best.max(score);
        }
    }
    best
}

const ALL_KINDS: [PieceKind; 7] = [
    PieceKind::King,
    PieceKind::Advisor,
    PieceKind::Elephant,
    PieceKind::Horse,
    PieceKind::Rook,
    PieceKind::Cannon,
    PieceKind::Pawn,
];

/// 返回按分数降序排列的全部兵种候选，供后续规则修复使用。
fn classify_type(mask: &[bool], side: u32, color: Color) -> Vec<(PieceKind, f32)> {
    let mut scores: Vec<(PieceKind, f32)> = ALL_KINDS
        .iter()
        .map(|kind| {
            let mut best = 0.0f32;
            for glyph in glyphs::glyphs_for(color, *kind) {
                for ratio in GLYPH_SIDE_RATIOS {
                    let tmpl = make_glyph_mask(side, glyph, ratio);
                    let s = mask_dice_shifted(mask, &tmpl, side);
                    best = best.max(s);
                }
            }
            (*kind, best)
        })
        .collect();
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    scores
}

#[derive(Clone, Copy)]
struct Grid {
    cell: f32,
    left: f32,
    top: f32,
    side: u32,
}

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

fn crop_center(img: &RgbaImage, cx: f32, cy: f32, side: u32) -> RgbaImage {
    let x0 = (cx - side as f32 / 2.0).round().max(0.0) as u32;
    let y0 = (cy - side as f32 / 2.0).round().max(0.0) as u32;
    let x0 = x0.min(img.width().saturating_sub(side));
    let y0 = y0.min(img.height().saturating_sub(side));
    imageops::crop_imm(img, x0, y0, side, side).to_image()
}

/// 给定方向下，用所有格子的「占用格平均字形匹配分」评估方向是否合理。
fn orientation_quality(img: &RgbaImage, grid: Grid, orientation: BoardOrientation) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0usize;
    for rank in 0..10u8 {
        for file in 0..9u8 {
            let (cx, cy) = cell_center(rank, file, grid, orientation);
            let crop = crop_center(img, cx, cy, grid.side);
            let info = analyze_crop(&crop);
            if !info.occupied {
                continue;
            }
            let mask = if orientation == BoardOrientation::Flipped180 {
                rotate180_mask(&info.mask, grid.side)
            } else {
                info.mask.clone()
            };
            let scores = classify_type(&mask, grid.side, info.color);
            let score = scores.first().map(|(_, s)| *s).unwrap_or(0.0);
            total += score;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
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
        let bg = dominant_bg_color(&img);
        let (bx0, by0, bx1, by1) = detect_board_rect(&img, bg)
            .ok_or_else(|| OcrError::BoardNotFound("未找到棋盘底色区域".to_string()))?;
        let cell_w = (bx1 - bx0 + 1) as f32 / 9.0;
        let cell_h = (by1 - by0 + 1) as f32 / 10.0;
        if cell_w < 8.0 || cell_h < 8.0 {
            return Err(OcrError::BoardNotFound("棋盘格过小".to_string()));
        }
        let cell = (cell_w + cell_h) / 2.0;
        let grid = Grid {
            cell,
            left: bx0 as f32,
            top: by0 as f32,
            side: ((cell * PROBE_RATIO).round() as u32).max(8),
        };

        let normal_q = orientation_quality(&img, grid, BoardOrientation::Normal);
        let flipped_q = orientation_quality(&img, grid, BoardOrientation::Flipped180);
        let orientation = if flipped_q > normal_q {
            BoardOrientation::Flipped180
        } else {
            BoardOrientation::Normal
        };

        let mut cells = Vec::with_capacity(90);
        let mut total_conf = 0.0f32;
        for rank in 0..10u8 {
            for file in 0..9u8 {
                let (cx, cy) = cell_center(rank, file, grid, orientation);
                let crop = crop_center(&img, cx, cy, grid.side);
                let info = analyze_crop(&crop);
                if !info.occupied {
                    total_conf += 1.0;
                    cells.push(RecognizedCell {
                        rank,
                        file,
                        piece: None,
                        confidence: if info.fg_ratio < OCCUPIED_RATIO * 0.5 {
                            1.0
                        } else {
                            0.5
                        },
                        uncertain: false,
                        alternatives: Vec::new(),
                    });
                    continue;
                }

                let mask = if orientation == BoardOrientation::Flipped180 {
                    rotate180_mask(&info.mask, grid.side)
                } else {
                    info.mask
                };
                let scores = classify_type(&mask, grid.side, info.color);
                let (kind, score) = scores[0];
                total_conf += score;
                cells.push(RecognizedCell {
                    rank,
                    file,
                    piece: Some(Piece {
                        color: info.color,
                        kind,
                    }),
                    confidence: score,
                    uncertain: score < TYPE_THRESHOLD,
                    alternatives: scores,
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
    use crate::ocr::render;
    use crate::ocr::{recognize, OcrInput};

    #[test]
    fn recognize_startpos_normal() {
        let start = parse_fen(START_FEN).unwrap();
        let png = render::render_screenshot_png(&start, BoardOrientation::Normal, 48, 24);
        let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();
        assert_eq!(out.orientation, BoardOrientation::Normal);
        assert!(out.confidence > 0.8);
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
        assert!(out.confidence > 0.8);
        assert!(out.cells.iter().all(|c| !c.uncertain));
    }

    #[test]
    fn empty_board_recognized_as_empty() {
        let pos = Position::default();
        let png = render::render_screenshot_png(&pos, BoardOrientation::Normal, 48, 24);
        let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();
        assert!(out.cells.iter().all(|c| c.piece.is_none()));
        assert!(out.issues.iter().any(|i| i.message.contains("缺少将/帅")));
    }
}
