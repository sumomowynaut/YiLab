//! GIF 导出：把选定的着法序列渲染成棋盘动画 GIF。
//!
//! - 来源：当前局面（单帧）/ 主线 / 指定变例（由命令层把「棋谱树 → startpos + moves」传入）。
//! - 帧渲染复用 `ocr::render`（同一套棋盘/棋子渲染），并在其上叠加：
//!   坐标（a-i / 0-9）、棋步高亮（最后一步的起止格）与棋步标注。
//! - 编码：`gif` crate；固定调色板（本渲染只使用有限颜色集合）+ 就近量化，支持帧间隔（毫秒）。
//! - 已知局限：棋子为程序生成的字母圆盘（无中文字体），真实棋子图形留待后续迭代。

use image::RgbaImage;

use crate::board::fen::parse_fen;
use crate::board::rules::apply_move;
use crate::board::types::{Move, Position, Square};
use crate::ocr::font::draw_char;
use crate::ocr::render::render_screenshot;
use crate::ocr::BoardOrientation;

/// GIF 导出请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GifRequest {
    /// 起始局面 FEN。
    pub startpos: String,
    /// 依次播放的着法（UCI）；空 = 仅当前局面单帧。
    pub moves: Vec<String>,
    /// 帧间隔（毫秒）。
    pub frame_delay_ms: u64,
    /// 棋盘格子像素边长（棋盘尺寸）。
    pub cell_size: u32,
    /// 是否显示坐标（a-i / 0-9）。
    pub show_coordinates: bool,
    /// 是否显示棋步（最后一步高亮 + 标注）。
    pub show_moves: bool,
}

/// 页面边距（用于坐标/标注）。
const MARGIN: u32 = 24;
/// 标注文字缩放（5×7 字库放大倍数）。
const LABEL_SCALE: usize = 2;

/// 固定调色板（本渲染只使用这些颜色）。
const PALETTE: [[u8; 3]; 8] = [
    [64, 64, 70],    // 页面背景
    [232, 196, 125], // 棋盘底色
    [96, 64, 32],    // 网格线
    [198, 60, 45],   // 红方棋子
    [40, 40, 48],    // 黑方棋子
    [255, 240, 220], // 棋子文字/坐标/标注
    [70, 44, 22],    // 棋子描边
    [255, 220, 0],   // 棋步高亮
];

/// 导出 GIF 字节。
pub fn export_gif(req: &GifRequest) -> Result<Vec<u8>, String> {
    // cap board size to avoid huge allocation from abnormal input
    let cell = req.cell_size.clamp(16, 256);
    let width = 9 * cell + 2 * MARGIN;
    let height = 10 * cell + 2 * MARGIN;

    // 构建局面序列：startpos + 每步之后
    let start = parse_fen(&req.startpos).map_err(|e| format!("FEN 无效：{e}"))?;
    let mut positions = Vec::with_capacity(req.moves.len() + 1);
    let mut last_moves: Vec<Option<Move>> = vec![None];
    let mut pos = start.clone();
    positions.push(start);
    for uci in &req.moves {
        let m = Move::parse_uci(uci).ok_or_else(|| format!("非法着法：{uci}"))?;
        pos = apply_move(&pos, m).ok_or_else(|| format!("非法着法：{uci}"))?;
        positions.push(pos.clone());
        last_moves.push(Some(m));
    }

    let mut out = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut out, width as u16, height as u16, &[])
            .map_err(|e| format!("GIF 编码器初始化失败：{e}"))?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| format!("设置循环失败：{e}"))?;
        let delay_cs = ((req.frame_delay_ms.max(10)) / 10) as u16; // 厘秒，至少 1
        let palette: Vec<u8> = PALETTE.iter().flatten().copied().collect();

        for (i, p) in positions.iter().enumerate() {
            let label = if req.show_moves && i > 0 {
                Some(format!("{}. {}", i, req.moves[i - 1]))
            } else {
                None
            };
            let img = render_frame(
                p,
                cell,
                req.show_coordinates,
                last_moves[i],
                label.as_deref(),
            );
            let raw = img.into_raw(); // RGBA
            let indexed = quantize(&raw, &PALETTE);
            let mut frame =
                gif::Frame::from_indexed_pixels(width as u16, height as u16, indexed, None);
            frame.palette = Some(palette.clone());
            frame.delay = delay_cs;
            encoder
                .write_frame(&frame)
                .map_err(|e| format!("写入帧失败：{e}"))?;
        }
    }
    Ok(out)
}

/// 渲染一帧：棋盘 + 坐标 + 棋步高亮/标注。
fn render_frame(
    pos: &Position,
    cell: u32,
    show_coordinates: bool,
    last_move: Option<Move>,
    label: Option<&str>,
) -> RgbaImage {
    let mut img = render_screenshot(pos, BoardOrientation::Normal, cell, MARGIN);
    if show_coordinates {
        draw_coordinates(&mut img, cell);
    }
    if let Some(label) = label {
        draw_label(&mut img, label);
    }
    if let Some(mv) = last_move {
        highlight_square(&mut img, mv.from, cell);
        highlight_square(&mut img, mv.to, cell);
    }
    img
}

/// 在 RGBA 图像上写一行文字（逐字符推进）。
fn draw_text_on(img: &mut RgbaImage, x: usize, y: usize, scale: usize, text: &str) {
    let stride = img.width() as usize * 4;
    let raw = img.as_mut();
    let mut cx = x;
    for c in text.chars() {
        draw_char(raw, stride, 4, cx, y, scale, c, [255, 240, 220]);
        cx += 5 * scale + 1;
    }
}

/// 绘制坐标：底部文件 a-i，左侧行号 9..0。
fn draw_coordinates(img: &mut RgbaImage, cell: u32) {
    for f in 0..9u32 {
        let x = (MARGIN + f * cell + cell / 2) as usize - (5 * LABEL_SCALE) / 2;
        let y = (MARGIN + 10 * cell + 4) as usize;
        let c = (b'a' + f as u8) as char;
        draw_text_on(img, x, y, LABEL_SCALE, &c.to_string());
    }
    for r in 0..10u32 {
        let rank = (9 - r) as u8;
        let x = 4;
        let y = (MARGIN + r * cell + cell / 2) as usize - (7 * LABEL_SCALE) / 2;
        draw_text_on(img, x, y, LABEL_SCALE, &((b'0' + rank) as char).to_string());
    }
}

/// 顶部绘制棋步标注（如 "1. h2e2"）。
fn draw_label(img: &mut RgbaImage, label: &str) {
    draw_text_on(img, MARGIN as usize, 4, LABEL_SCALE, label);
}

/// 高亮一个格子（黄色圆环）。
fn highlight_square(img: &mut RgbaImage, sq: Square, cell: u32) {
    let (r, f) = (9 - sq.rank, sq.file);
    let cx = (MARGIN + f as u32 * cell + cell / 2) as i32;
    let cy = (MARGIN + r as u32 * cell + cell / 2) as i32;
    let radius = cell as f32 * 0.46;
    let inner = (radius - 3.0).max(1.0);
    let (w, h) = (img.width() as i32, img.height() as i32);
    for dy in -(radius as i32)..=(radius as i32) {
        for dx in -(radius as i32)..=(radius as i32) {
            let px = cx + dx;
            let py = cy + dy;
            if px < 0 || py < 0 || px >= w || py >= h {
                continue;
            }
            let d2 = (dx * dx + dy * dy) as f32;
            if d2 <= radius * radius && d2 >= inner * inner {
                img.put_pixel(px as u32, py as u32, image::Rgba([255, 220, 0, 255]));
            }
        }
    }
}

/// 就近量化到调色板（本渲染颜色集合有限，量化无损）。
fn quantize(rgba: &[u8], palette: &[[u8; 3]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgba.len() / 4);
    for px in rgba.as_chunks::<4>().0 {
        let (r, g, b) = (px[0] as i32, px[1] as i32, px[2] as i32);
        let mut best = 0usize;
        let mut best_d = i32::MAX;
        for (i, c) in palette.iter().enumerate() {
            let dr = r - c[0] as i32;
            let dg = g - c[1] as i32;
            let db = b - c[2] as i32;
            let d = dr * dr + dg * dg + db * db;
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        out.push(best as u8);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::START_FEN;

    fn req(moves: &[&str]) -> GifRequest {
        GifRequest {
            startpos: START_FEN.to_string(),
            moves: moves.iter().map(|s| s.to_string()).collect(),
            frame_delay_ms: 500,
            cell_size: 32,
            show_coordinates: true,
            show_moves: true,
        }
    }

    fn decode_frames(bytes: &[u8]) -> (u16, u16, Vec<u16>) {
        let mut opts = gif::DecodeOptions::new();
        opts.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = opts
            .read_info(std::io::Cursor::new(bytes))
            .expect("gif 可解码");
        let (w, h) = (decoder.width(), decoder.height());
        let mut delays = Vec::new();
        while let Some(frame) = decoder.read_next_frame().expect("frame") {
            delays.push(frame.delay);
        }
        (w, h, delays)
    }

    #[test]
    fn single_frame_gif_for_current_position() {
        let bytes = export_gif(&req(&[])).expect("export");
        assert!(bytes.starts_with(b"GIF"), "应含 GIF 魔数");
        let (w, h, delays) = decode_frames(&bytes);
        assert_eq!(delays.len(), 1, "单帧");
        assert_eq!(
            (w, h),
            (9 * 32 + 2 * MARGIN as u16, 10 * 32 + 2 * MARGIN as u16)
        );
        assert_eq!(delays[0], 50, "500ms = 50 厘秒");
    }

    #[test]
    fn multi_frame_gif_for_mainline() {
        let bytes = export_gif(&req(&["h2e2", "h7e7", "h0g2"])).expect("export");
        let (_, _, delays) = decode_frames(&bytes);
        assert_eq!(delays.len(), 4, "3 步棋 = 4 帧（含起始局面）");
        for d in &delays {
            assert_eq!(*d, 50);
        }
    }

    #[test]
    fn frame_delay_rounds_to_centiseconds() {
        let bytes = export_gif(&GifRequest {
            frame_delay_ms: 120,
            ..req(&["h2e2"])
        })
        .expect("export");
        let (_, _, delays) = decode_frames(&bytes);
        assert_eq!(delays[0], 12, "120ms = 12 厘秒");
    }

    #[test]
    fn cell_size_is_clamped_to_reasonable_range() {
        let bytes = export_gif(&GifRequest {
            cell_size: 100_000, // 应被钳制到上限，而不是导致内存放大
            ..req(&[])
        })
        .expect("export");
        let (w, h, _) = decode_frames(&bytes);
        // 上限 256：宽 9*256 + 2*MARGIN
        assert!(w <= 9 * 256 + 2 * MARGIN as u16, "宽度应被钳制");
        assert!(h <= 10 * 256 + 2 * MARGIN as u16, "高度应被钳制");
    }

    #[test]
    fn invalid_fen_or_move_errors() {
        let bad_fen = GifRequest {
            startpos: "garbage".to_string(),
            ..req(&[])
        };
        assert!(export_gif(&bad_fen).is_err());

        let bad_move = GifRequest {
            moves: vec!["e9e9".to_string()], // 非法着法
            ..req(&[])
        };
        assert!(export_gif(&bad_move).is_err());
    }
}
