//! GIF 导出集成测试：树 → startpos + moves → 渲染编码 → GIF 回读校验。

use pikaxiangqi_lib::board::types::{Move, START_FEN};
use pikaxiangqi_lib::game::tree::GameTree;
use pikaxiangqi_lib::gif_export::{export_gif, GifRequest};

fn mv(uci: &str) -> Move {
    Move::parse_uci(uci).unwrap()
}

/// 回读 GIF：帧数、尺寸、每帧延迟。
fn decode(bytes: &[u8]) -> (u16, u16, Vec<u16>) {
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
fn exports_mainline_animation_from_tree() {
    let mut tree = GameTree::new(START_FEN).unwrap();
    tree.insert_move(mv("h2e2")).unwrap();
    tree.insert_move(mv("h7e7")).unwrap();
    tree.insert_move(mv("h0g2")).unwrap();
    tree.insert_move(mv("b9c7")).unwrap();

    let moves: Vec<String> = tree
        .main_line()
        .iter()
        .skip(1)
        .map(|id| tree.node(*id).unwrap().mv.unwrap().uci())
        .collect();
    assert_eq!(moves.len(), 4);

    let req = GifRequest {
        startpos: tree.startpos.clone(),
        moves,
        frame_delay_ms: 300,
        cell_size: 48,
        show_coordinates: true,
        show_moves: true,
    };
    let bytes = export_gif(&req).expect("export");
    assert!(bytes.starts_with(b"GIF"), "GIF 魔数");
    let (w, h, delays) = decode(&bytes);
    assert_eq!(delays.len(), 5, "4 步棋 = 5 帧");
    assert_eq!((w, h), (9 * 48 + 48, 10 * 48 + 48), "含边距（24*2）");
    for d in &delays {
        assert_eq!(*d, 30, "300ms = 30 厘秒");
    }
}

#[test]
fn exports_single_frame_current_position() {
    let req = GifRequest {
        startpos: START_FEN.to_string(),
        moves: Vec::new(),
        frame_delay_ms: 500,
        cell_size: 32,
        show_coordinates: false,
        show_moves: false,
    };
    let bytes = export_gif(&req).expect("export");
    let (_, _, delays) = decode(&bytes);
    assert_eq!(delays.len(), 1);
}

#[test]
fn exports_variation_line() {
    let mut tree = GameTree::new(START_FEN).unwrap();
    tree.insert_move(mv("h2e2")).unwrap();
    let n1 = tree.current_id();
    tree.insert_move(mv("h7e7")).unwrap();
    // h2e2 下的变例：b9c7 → h0g2
    tree.set_current(n1).unwrap();
    let var = tree.insert_move(mv("b9c7")).unwrap();
    tree.insert_move(mv("h0g2")).unwrap();

    // 变例路径 = 到分支点（h2e2，含其着法） + 变例自身着法
    let parent = tree.node(var).unwrap().parent.unwrap();
    let mut prefix: Vec<String> = vec![];
    let mut cur = Some(parent);
    while let Some(id) = cur {
        if let Some(m) = tree.node(id).unwrap().mv {
            prefix.push(m.uci());
        }
        cur = tree.node(id).unwrap().parent;
    }
    prefix.reverse();
    let mut line: Vec<String> = vec![];
    let mut cur = Some(var);
    while let Some(id) = cur {
        if let Some(m) = tree.node(id).unwrap().mv {
            line.push(m.uci());
        }
        cur = tree.node(id).unwrap().children.first().copied();
    }
    let mut moves = prefix;
    moves.extend(line);
    assert_eq!(moves, vec!["h2e2", "b9c7", "h0g2"]);

    let req = GifRequest {
        startpos: tree.startpos.clone(),
        moves,
        frame_delay_ms: 400,
        cell_size: 32,
        show_coordinates: true,
        show_moves: true,
    };
    let bytes = export_gif(&req).expect("export");
    let (_, _, delays) = decode(&bytes);
    assert_eq!(delays.len(), 4, "h2e2 + 变例 2 步 = 4 帧");
}
