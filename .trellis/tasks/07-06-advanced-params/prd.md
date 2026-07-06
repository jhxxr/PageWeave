# 高级翻译参数页落地

## Goal

把 `/params` 占位页变成可用的高级翻译参数编辑器，所选参数经 `TranslateRequest.advanced` 透传到 Rust `build_args`，最终拼进 BabelDOC sidecar 命令行。默认状态（未设置任何高级参数）下生成的 CLI 参数与现状逐字节一致。

## Background

- 现状：`src/features/params/ParamsPage.tsx` 是 `Empty` 占位；`build_args`（`src-tauri/src/translate/args.rs`）硬编码 `--enhance-compatibility`、`--auto-enable-ocr-workaround`、`--watermark-output-mode no_watermark`、`--report-interval 0.1` + output_mode 分支。
- BabelDOC sidecar `--help` 已核实，存在大量可调 flag（pages、min-text-length、glossary-files、primary-font-family、ocr 三态、兼容 bundle 子项、pools、openai 调参等）。
- README 与 i18n 都把"术语表/上下文/缓存/OCR 调优"标为"MVP 之后扩展"——本次落地这项债。

## Requirements

### R1 Rust 契约
- 新增 `OcrMode` 枚举（`auto/off/force`，serde lowercase）与 `AdvancedParams` 结构体（全字段 `Option<T>` + `#[serde(default)]`，derive `Serialize+Deserialize`）。
- `TranslateRequest` 加 `#[serde(default)] pub advanced: Option<AdvancedParams>`，老前端不传该字段时反序列化为 `None`。

### R2 build_args 重构（无回归）
- 默认（`advanced=None` 或全 `None`）输出与今天逐字节一致。
- 兼容 bundle：`enhance_compatibility=None`→`true`→发 `--enhance-compatibility` 并忽略三个子 flag；`Some(false)`→按子 flag 单独发。
- OCR 三态：`None`/`Auto`→`--auto-enable-ocr-workaround`；`Off`→`--skip-scanned-detection`；`Force`→`--ocr-workaround`，三者只发一个。
- dual-only：`use_alternating_pages_dual`、`dual_translate_first` 仅 `output_mode != Mono` 时发。
- 其余可选 flag 逐字段 `if let Some` 发出；`primary_font_family="auto"`=omit；`glossary_files` 用逗号 join。

### R3 校验
- `start_translate` 在 `req.advanced=Some(a)` 时校验：pages 正则、各数值 ≥1、glossary 文件存在且路径不含逗号、`custom_system_prompt` ≤8000 字符。

### R4 前端契约与 store
- `src/types/index.ts` 加 `OcrMode`/`FontFamily`/`AdvancedParams`，`TranslateRequest.advanced?`。
- `translateStore` 加 `advanced: AdvancedParams`（默认 `{}`）+ `setAdvanced(patch)` 浅合并 + `resetAdvanced()`。`resetTask` 不动（高级参数随会话保留，与 `langIn/providerId` 一致）。不加 persist 中间件。

### R5 ParamsPage UI
- `Card`+`Collapse` 六面板：Scope / Glossary / Layout / OCR&Compat / Cache&Pools / OpenAI。控件双向绑定 store。重置按钮调 `resetAdvanced`。可选 CLI 预览（纯展示，不发给 Rust）。
- Layout/OCR 面板的 dual-only 与 bundle 禁用态按计划文档约束实现。

### R6 TranslatePage 接线
- `start()` 的 `req` 加 `advanced: st.advanced`。可选 appliedCount Tag 跳 `/params`。

### R7 i18n
- `params.*` 键补全（zh+en），`placeholder`→`intro`。

## Acceptance Criteria

- [ ] `cargo test -p pageweave` 全绿，含 `default_advanced_matches_baseline`（无回归锁）与其余 args 测试。
- [ ] `pnpm exec tsc --noEmit` 无新错。
- [ ] `pnpm tauri dev`：`/params` 页面六面板渲染、折叠、重置清空、预览合理。
- [ ] 端到端：设 `pages=1-3`+glossary.csv+`ocr_mode=force`+`enhance_compatibility=false`+`skip_clean=true` 翻译，日志确认 sidecar 收到 `--pages 1-3 --glossary-files <path> --ocr-workaround --skip-clean` 且无 `--enhance-compatibility`。
- [ ] 重置高级参数后再翻一次，日志确认只有 `--enhance-compatibility --auto-enable-ocr-workaround`（与改动前一致）。
- [ ] `pages=foo` 等非法输入被 `start_translate` 拒绝。
- [ ] 切换语言，ParamsPage 标签正确翻译。

## Constraints

- 默认状态 CLI 参数逐字节不变（无回归不变式）。
- 不引入 DB 迁移、不引入 persist 中间件（会话内有效）。
- `resetTask` 不得清空 `advanced`。
- 不动 `--watermark-output-mode`/`--report-interval`（仍硬编码）。

## Out of Scope

- 批量/多文件翻译、任务历史持久化、`--config` toml 改造（已知债）、sidecar flag 漂移检测。
