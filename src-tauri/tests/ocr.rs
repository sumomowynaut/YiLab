//! 截图识别集成测试：合成截图 → 模板识别 → 本地规则校验。

use pikaxiangqi_lib::board::fen::{parse_fen, to_fen};
use pikaxiangqi_lib::board::rules::apply_move;
use pikaxiangqi_lib::board::types::{Move, Position, START_FEN};
use pikaxiangqi_lib::ocr::render::{render_screenshot, render_screenshot_png};
use pikaxiangqi_lib::ocr::template::TemplateRecognizer;
use pikaxiangqi_lib::ocr::{recognize, BoardOrientation, OcrError, OcrInput};

fn mv(uci: &str) -> Move {
    Move::parse_uci(uci).unwrap()
}

#[test]
fn recognize_startpos_normal() {
    let start = parse_fen(START_FEN).unwrap();
    let png = render_screenshot_png(&start, BoardOrientation::Normal, 48, 24);
    let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();

    assert_eq!(out.orientation, BoardOrientation::Normal);
    assert_eq!(out.fen, START_FEN);
    assert!(out.confidence > 0.9, "置信度 {:.2}", out.confidence);
    assert!(out.cells.iter().all(|c| !c.uncertain), "不应有不确定格");
    // 起始局面合法（除「行棋方未知」提示外无规则问题）
    let rule_issues: Vec<_> = out
        .issues
        .iter()
        .filter(|i| !i.message.contains("行棋方"))
        .collect();
    assert!(rule_issues.is_empty(), "规则问题：{:?}", rule_issues);
}

#[test]
fn recognize_startpos_flipped180() {
    let start = parse_fen(START_FEN).unwrap();
    let png = render_screenshot_png(&start, BoardOrientation::Flipped180, 48, 24);
    let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();

    assert_eq!(out.orientation, BoardOrientation::Flipped180);
    assert_eq!(out.fen, START_FEN, "翻转后仍应识别出相同局面");
    assert!(out.confidence > 0.9);
}

#[test]
fn recognize_custom_position_after_move() {
    let start = parse_fen(START_FEN).unwrap();
    let pos = apply_move(&start, mv("h2e2")).unwrap();
    let png = render_screenshot_png(&pos, BoardOrientation::Normal, 48, 24);
    let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();

    // 截图无法判断行棋方/计数，仅比较棋盘部分
    let board = out.fen.split(' ').next().unwrap().to_string();
    let full = to_fen(&pos);
    let expected = full.split(' ').next().unwrap().to_string();
    assert_eq!(board, expected, "识别局面不一致：{}", out.fen);
    assert!(out.cells.iter().all(|c| !c.uncertain));
}

#[test]
fn recognize_rejects_corrupted_image() {
    let err = recognize(
        &TemplateRecognizer::new(),
        &OcrInput {
            image: b"this is not an image".to_vec(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, OcrError::ImageDecode(_)), "{err}");
}

#[test]
fn recognize_rejects_image_without_board() {
    // 纯色图片（无棋盘底色）
    let img = image::RgbaImage::from_pixel(480, 520, image::Rgba([64, 64, 70, 255]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    let err = recognize(&TemplateRecognizer::new(), &OcrInput { image: bytes }).unwrap_err();
    assert!(matches!(err, OcrError::BoardNotFound(_)), "{err}");
}

#[test]
fn recognize_missing_king_flags_rule_violation() {
    // 只有红帅（缺黑将）→ 规则校验应报「黑方缺少将/帅」
    let fen = "9/9/9/9/9/9/9/9/9/4K4 w - - 0 1";
    let pos = parse_fen(fen).unwrap();
    let png = render_screenshot_png(&pos, BoardOrientation::Normal, 48, 24);
    let out = recognize(&TemplateRecognizer::new(), &OcrInput { image: png }).unwrap();

    assert!(!out.valid, "缺将局面不应被判为有效");
    assert!(
        out.issues.iter().any(|i| i.message.contains("缺少将/帅")),
        "issues: {:?}",
        out.issues
    );
    // 仍输出结构化局面（人工校正路径）
    assert!(out.fen.contains('K'));
}

#[test]
fn recognize_tiny_image_errors() {
    let img = render_screenshot(&Position::default(), BoardOrientation::Normal, 4, 2);
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    let err = recognize(&TemplateRecognizer::new(), &OcrInput { image: bytes }).unwrap_err();
    assert!(
        matches!(err, OcrError::InvalidImage(_) | OcrError::BoardNotFound(_)),
        "{err}"
    );
}
