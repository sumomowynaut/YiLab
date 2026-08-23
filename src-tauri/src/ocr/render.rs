//! 共享棋子渲染 + 合成截图生成器。
//!
//! 传统 CV 识别器（template）与测试合成截图共用同一渲染，保证模板匹配自洽。
//! 真实截图使用汉字棋子，识别率有限属已知局限（`NEEDS_VERIFICATION`：真实模型的选型与权重许可，
//! 见 docs/ocr.md §3），本版本以「确定性模板匹配 + 置信度 + 人工校正」为最低可用实现。

use image::{Rgb, Rgba, RgbaImage};

use crate::board::types::{Color, Piece, Position, NUM_FILES, NUM_RANKS};

use super::glyphs::{self, Glyph};
use super::BoardOrientation;

/// 棋盘底色（浅木色）。
pub const BOARD_BG: Rgb<u8> = Rgb([232, 196, 125]);
/// 棋盘网格线颜色。
pub const GRID: Rgb<u8> = Rgb([96, 64, 32]);
/// 页面（棋盘外）背景色。
pub const PAGE_BG: Rgb<u8> = Rgb([64, 64, 70]);
/// 红方棋子圆盘色。
pub const RED_PIECE: Rgb<u8> = Rgb([198, 60, 45]);
/// 黑方棋子圆盘色。
pub const BLACK_PIECE: Rgb<u8> = Rgb([40, 40, 48]);
/// 棋子文字色（米白）。
pub const PIECE_TEXT: Rgb<u8> = Rgb([255, 240, 220]);
/// 圆盘半径与格子边长的比例（识别器模板必须一致，见 template.rs）。
pub const DISC_RATIO: f32 = 0.40;

fn piece_color_rgb(piece: Piece) -> Rgb<u8> {
    match piece.color {
        Color::Red => RED_PIECE,
        Color::Black => BLACK_PIECE,
    }
}

/// 在 RGBA 画布上绘制一枚棋子圆盘（圆心 + 半径），盘面用 FEN 字母。
pub fn draw_piece(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, piece: Piece) {
    let w = img.width() as i32;
    let h = img.height() as i32;
    let r = radius as i32;
    let color = piece_color_rgb(piece).0;
    // 圆盘（含抗锯齿近似：逐像素距离判定）
    for dy in -r..=r {
        for dx in -r..=r {
            let px = cx as i32 + dx;
            let py = cy as i32 + dy;
            if px < 0 || py < 0 || px >= w || py >= h {
                continue;
            }
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 <= radius * radius {
                img.put_pixel(
                    px as u32,
                    py as u32,
                    Rgba([color[0], color[1], color[2], 255]),
                );
            }
        }
    }
    // 圆环描边（深色）
    let rim = radius * 0.78;
    for dy in -r..=r {
        for dx in -r..=r {
            let px = cx as i32 + dx;
            let py = cy as i32 + dy;
            if px < 0 || py < 0 || px >= w || py >= h {
                continue;
            }
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 <= radius * radius && d2 >= rim * rim {
                img.put_pixel(px as u32, py as u32, Rgba([70, 44, 22, 255]));
            }
        }
    }
    // 盘面汉字（字形模板与识别器共享，保证自洽）
    let glyph = glyphs::first_glyph(piece.color, piece.kind);
    draw_glyph(img, glyph, cx, cy, radius * 1.1, PIECE_TEXT.0);
}

/// 在 RGBA 画布上以左下角/中心缩放绘制一个 20 点阵字形（圆心居中）。
fn draw_glyph(
    img: &mut RgbaImage,
    glyph: &Glyph,
    cx: f32,
    cy: f32,
    half_size: f32,
    color: [u8; 3],
) {
    let grid = glyphs::GLYPH_GRID as f32;
    // 字形占棋子直径约 60%，故半宽取 half_size 的 0.6 倍
    let side = half_size * 1.2f32;
    let scale = (side / grid).max(1.0);
    let x0 = (cx - side / 2.0).round() as i32;
    let y0 = (cy - side / 2.0).round() as i32;
    let w = img.width() as i32;
    let h = img.height() as i32;
    for gy in 0..glyphs::GLYPH_GRID {
        for gx in 0..glyphs::GLYPH_GRID {
            if !glyph.get(gx, gy) {
                continue;
            }
            for sy in 0..(scale as i32) {
                for sx in 0..(scale as i32) {
                    let px = x0 + gx as i32 * scale as i32 + sx;
                    let py = y0 + gy as i32 * scale as i32 + sy;
                    if px >= 0 && py >= 0 && px < w && py < h {
                        img.put_pixel(
                            px as u32,
                            py as u32,
                            Rgba([color[0], color[1], color[2], 255]),
                        );
                    }
                }
            }
        }
    }
}

/// 用 `orientation` 把 (rank, file) 映射到图像坐标（像素，格中心）。
fn cell_center(
    rank: u8,
    file: u8,
    cell: f32,
    margin: f32,
    orientation: BoardOrientation,
) -> (f32, f32) {
    let (r, f) = match orientation {
        BoardOrientation::Normal => (9 - rank, file),
        BoardOrientation::Flipped180 => (rank, 8 - file),
    };
    let x = margin + (f as f32 + 0.5) * cell;
    let y = margin + (r as f32 + 0.5) * cell;
    (x, y)
}

/// 合成一张棋盘截图（PNG 字节），用于测试与模板自洽验证。
///
/// - `cell`：每个格子的像素边长（不含边距）。
/// - `margin`：棋盘四周留白（像素）。
pub fn render_screenshot(
    pos: &Position,
    orientation: BoardOrientation,
    cell: u32,
    margin: u32,
) -> RgbaImage {
    let width = 9 * cell + 2 * margin;
    let height = 10 * cell + 2 * margin;
    let mut img = RgbaImage::from_pixel(
        width,
        height,
        Rgba([PAGE_BG.0[0], PAGE_BG.0[1], PAGE_BG.0[2], 255]),
    );
    let bg = Rgba([BOARD_BG.0[0], BOARD_BG.0[1], BOARD_BG.0[2], 255]);
    let grid = Rgba([GRID.0[0], GRID.0[1], GRID.0[2], 255]);

    // 棋盘底色
    for y in margin..margin + 10 * cell {
        for x in margin..margin + 9 * cell {
            img.put_pixel(x, y, bg);
        }
    }

    // 网格：9 条竖线（完整）+ 10 条横线（河界处中间断开）
    let thickness = 2u32;
    for f in 0..=9u32 {
        let x = margin + f * cell;
        for t in 0..thickness {
            for y in margin..margin + 10 * cell {
                img.put_pixel(x + t, y, grid);
            }
        }
    }
    for r in 0..=10u32 {
        let y = margin + r * cell;
        let (x0, x1) = if r == 5 {
            // 河界：两端保留短横线
            (margin + cell, margin + 8 * cell)
        } else {
            (margin, margin + 9 * cell)
        };
        for t in 0..thickness {
            for x in x0..x1 {
                img.put_pixel(x, y + t, grid);
            }
        }
    }

    // 棋子先按 Normal 摆放；Flipped180 通过整图旋转 180° 得到（避免二次翻转）
    let radius = cell as f32 * DISC_RATIO;
    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            let Some(piece) = pos.board[rank as usize][file as usize] else {
                continue;
            };
            let (cx, cy) = cell_center(
                rank,
                file,
                cell as f32,
                margin as f32,
                BoardOrientation::Normal,
            );
            draw_piece(&mut img, cx, cy, radius, piece);
        }
    }
    // Flipped180 = 整图真实旋转 180°（字母倒置），与「拍倒」的截图一致
    if orientation == BoardOrientation::Flipped180 {
        return image::imageops::rotate180(&img);
    }
    img
}

/// 把合成截图编码为 PNG 字节。
pub fn render_screenshot_png(
    pos: &Position,
    orientation: BoardOrientation,
    cell: u32,
    margin: u32,
) -> Vec<u8> {
    let img = render_screenshot(pos, orientation, cell, margin);
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("PNG 编码不应失败");
    out.into_inner()
}
