//! 5×7 位图字库：用于把棋子渲染成可识别的模板（合成截图与识别共用同一渲染）。
//!
//! 仅需 7 个大写字母（K/A/B/N/R/C/P，对应 FEN 棋子字符）。每行 5 bit，低位在左。

/// 字母 → 7 行 × 5 bit（bit=1 表示填充）。
fn glyph(c: char) -> Option<[u8; 7]> {
    match c {
        // K
        'K' => Some([
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ]),
        // A
        'A' => Some([
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ]),
        // B
        'B' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ]),
        // N
        'N' => Some([
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ]),
        // R
        'R' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ]),
        // C
        'C' => Some([
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ]),
        // P
        'P' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ]),
        // 数字 0-9（坐标/棋步标注）
        '0' => Some([
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ]),
        '1' => Some([
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ]),
        '2' => Some([
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ]),
        '3' => Some([
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ]),
        '4' => Some([
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ]),
        '5' => Some([
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ]),
        '6' => Some([
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ]),
        '7' => Some([
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ]),
        '8' => Some([
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ]),
        '9' => Some([
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ]),
        // 小写 a-i（坐标文件字母）
        'a' => Some([
            0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111,
        ]),
        'b' => Some([
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b11110,
        ]),
        'c' => Some([
            0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110,
        ]),
        'd' => Some([
            0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111,
        ]),
        'e' => Some([
            0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110,
        ]),
        'f' => Some([
            0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000,
        ]),
        'g' => Some([
            0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110,
        ]),
        'h' => Some([
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ]),
        'i' => Some([
            0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110,
        ]),
        _ => None,
    }
}

/// 在像素缓冲中按 `scale` 缩放绘制字母（左上角对齐）。
///
/// - `buf`：行主序像素缓冲（RGB 或 RGBA 均可）。
/// - `row_stride`：一行的字节数（宽 × 每像素字节数）。
/// - `bpp`：每像素字节数（3 或 4）。
/// - `x`/`y`：字母左上角（像素）。
/// - `scale`：放大倍数（≥1）。
#[allow(clippy::too_many_arguments)] // 底层像素绘制函数，参数本就多
pub fn draw_char(
    buf: &mut [u8],
    row_stride: usize,
    bpp: usize,
    x: usize,
    y: usize,
    scale: usize,
    c: char,
    color: [u8; 3],
) {
    let Some(rows) = glyph(c) else {
        return;
    };
    let h = buf.len() / row_stride;
    for (ry, row) in rows.iter().enumerate() {
        for rx in 0..5 {
            if row & (1 << (4 - rx)) != 0 {
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = x + rx * scale + sx;
                        let py = y + ry * scale + sy;
                        if px * bpp + 2 < row_stride && py < h {
                            let idx = py * row_stride + px * bpp;
                            buf[idx..idx + 3].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_piece_letters_have_glyphs() {
        for c in ['K', 'A', 'B', 'N', 'R', 'C', 'P'] {
            assert!(glyph(c).is_some(), "missing glyph for {c}");
        }
        assert!(glyph('X').is_none());
    }

    #[test]
    fn glyphs_are_distinct() {
        let letters = ['K', 'A', 'B', 'N', 'R', 'C', 'P'];
        for (i, a) in letters.iter().enumerate() {
            for b in letters.iter().skip(i + 1) {
                assert_ne!(glyph(*a), glyph(*b), "{a} and {b} collide");
            }
        }
    }

    #[test]
    fn coordinate_glyphs_exist() {
        for c in '0'..='9' {
            assert!(glyph(c).is_some(), "missing digit {c}");
        }
        for c in 'a'..='i' {
            assert!(glyph(c).is_some(), "missing letter {c}");
        }
        // UCI 着法字符集（文件 a-i + 行 0-9）全覆盖
        for c in "h2e2".chars() {
            assert!(glyph(c).is_some(), "missing uci char {c}");
        }
    }

    #[test]
    fn draw_char_writes_pixels_scaled() {
        let mut buf = vec![0u8; 5 * 7 * 4];
        // A 第一行 01110 → 第 1/2/3 列填充
        draw_char(&mut buf, 5 * 4, 4, 0, 0, 1, 'A', [255, 0, 0]);
        assert_eq!(&buf[4..7], &[255, 0, 0]);
        assert_eq!(&buf[0..3], &[0, 0, 0]);
    }
}
