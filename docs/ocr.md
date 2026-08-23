# 截图识别架构（OCR）

> 实现状态（2026-08-23）：`src-tauri/src/ocr/` 已落地——`OcrEngine` trait（视觉模型抽象）、
> `TemplateRecognizer`（传统 CV，确定性模板匹配）、合成截图生成器（`render`）、
> 识别管线（本地棋规校验 + 置信度 + 问题清单 + FEN），以及 Tauri 命令 `ocr_recognize` 与前端 OcrPanel。
> 真实模型（ONNX 等）仍为后续迭代（`NEEDS_VERIFICATION`，见 §3）。

## 1. 目标与范围

从一张棋盘图片（截图/拍照）识别出局面，生成 FEN 并载入局面编辑器，供用户校正后继续。

对照网页版基线：网页版「棋盘图片识别」明确说明「建议清晰无干扰且垂直棋盘，识别率不是很高，识别有误可手动摆棋」。本项目把**手动校正**作为一等公民，而非追求完美识别。

## 2. 管线

```
图片输入 → 预处理 → 棋盘检测 → 网格切分 → 棋子分类 → FEN 合成 → 人工校正
```

| 步骤 | 职责 | 备注 |
|------|------|------|
| 预处理 | 灰度/去噪/透视校正 | 提升后续鲁棒性 |
| 棋盘检测 | 定位 10×9 网格角点 | 支持透视变形校正 |
| 网格切分 | 得到 90 个格子的子图 | |
| 棋子分类 | 识别每格：空/红黑 7 种棋子 | 核心难点：汉字 + 红黑区分 |
| FEN 合成 | 分类结果 → 局面 | 缺失/不确定格置空并标记 |
| 人工校正 | 在局面编辑器中修正 | 必选，保证最终正确性 |

## 3. 技术选型

- 本地优先：在 Rust 侧做识别，不上传图片。
- **首版（已实现）**：传统 CV——按棋盘底色定位网格 + 程序生成字母模板逐格匹配（`TemplateRecognizer`）。
  合成截图（`render`）与识别共用同一渲染，保证测试自洽；方向判定用「正立 vs 旋转 180° 模板」比较。
- 候选方案（`NEEDS_VERIFICATION`，需验证识别率与许可）：
  1. 轻量本地模型（如 ONNX 小分类网络）——识别率更好，需选型与权重许可确认。
  2. 云端 OCR/识别 API——仅作可选增强（违背本地优先）。
- 已知局限：模板为程序生成字母圆盘，真实截图（汉字棋子、不同配色/透视）识别率有限；
  识别不确定的格子**置空并标记**，引导用户手动校正（`docs/ocr.md` 基线：校正为一等公民）。

## 4. 接口（已实现）

```rust
pub struct OcrInput { pub image: Vec<u8> }          // PNG/JPEG 字节

pub struct RecognizedCell {
    pub rank: u8, pub file: u8,
    pub piece: Option<Piece>,    // None = 空格
    pub confidence: f32,         // [0,1]
    pub uncertain: bool,         // 低置信度 → 置空并标记
}

pub enum BoardOrientation { Normal, Flipped180 }

pub trait OcrEngine: Send + Sync {
    /// 视觉模型：只做识别，不做规则判断
    fn recognize_cells(&self, input: &OcrInput) -> Result<RawRecognition, OcrError>;
}

/// 识别管线：引擎识别 → 本地 `validate_position` 棋规校验 → FEN 合成 → 问题汇总
pub fn recognize(engine: &dyn OcrEngine, input: &OcrInput) -> Result<RecognitionOutput, OcrError>;
```

- 输出始终带 `confidence`、`orientation`、`issues` 与 `valid`；不确定格在 FEN 中**置空并标记**，不静默接受。
- 静态截图无法判断行棋方 → `side_to_move = None` + 提示（默认按红方先行）。
- OCR 目前为同步阻塞调用（图片小、单次 <1s）；后台线程化留给图片较大/批量场景。

## 5. 依赖与许可

- 图片解码：Tauri/Rust 侧标准图像库（如 `image` crate）。
- 若引入本地模型，权重与运行时许可须纳入 `docs/licensing.md` 风险清单。
- 不将用户图片上传到第三方，除非用户显式选择云端方案。

## 6. 测试（已实现）

- 合成截图测试集：起始局面（正立/翻转 180°）、自定义局面、空棋盘、缺将残局——验证棋盘检测、方向判定、
  分类与置信度（Rust 单元 8 + 集成 7）。
- 错误路径：损坏图片（解码失败）、无棋盘底色（定位失败）、过小图片。
- 前端 OcrPanel 4 用例：选择截图 → 识别 → 展示置信度/问题/FEN → 载入人工修正。
- 最低验收线：不崩溃 + 输出合法结构 + 可人工校正，全部满足。