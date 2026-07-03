# BabelDOC 调研报告

> 调研对象：`funstory-ai/BabelDOC` (GitHub)
> 调研目的：评估如何在一个 Windows 本地 Tauri 桌面翻译工具中集成 BabelDOC。
> 调研时间：2026-07-01

## 一、项目概览

| 项目 | 信息 |
|---|---|
| 仓库 | https://github.com/funstory-ai/BabelDOC |
| 定位 | "PDF scientific paper translation and bilingual comparison library"——**库优先**，"主要设计为被嵌入到其他程序中，也可直接用于简单翻译任务" |
| 最新版本 | v0.6.3（PyPI 同步） |
| 活跃度 | 8.8k stars、711 forks、1,827 commits、212 个 release；funstory.ai 团队维护 |
| 构建系统 | hatchling；`requires-python = ">=3.10,<3.14"` |
| CLI 入口 | `babeldoc = "babeldoc.main:cli"`（pip 安装后即生成 `babeldoc` 命令） |

**核心能力**：
- PDF 解析 → 中间表示（IR）→ 翻译 → 重新排版输出（保留原文档结构）
- 双语 PDF（dual）+ 单语 PDF（mono）双输出，默认同时产出
- 自动术语提取 + 自定义术语表（CSV：`source,target,tgt_lng`）
- 扫描文档检测 + OCR workaround（白块覆盖原文）
- 插件化 pipeline（模型 / OCR / renderer 可替换）
- 大文档自动分片（split）+ 合并
- 离线资源包（`--generate-offline-assets` / `--restore-offline-assets`），适配内网/气隙环境

**翻译服务商**：当前仅支持 OpenAI 兼容 LLM（任何 base_url + api_key + model 的 OpenAI 兼容端点，含 Ollama、DeepSeek、GLM 等）。

## 二、许可证（关键风险点）

**确切许可证：AGPL-3.0-or-later**（LICENSE 文件头部明确，Copyright (C) 2024 funstory.ai limited）。

### AGPL-3.0 传染性对本项目的影响

本项目（PageWeave）是开源项目：

1. **派生作品必须开源且同样 AGPL-3.0**：只要 BabelDOC 被静态/动态链接、修改、或作为派生作品分发，整个派生物须以 AGPL-3.0 开源。本开源项目可满足此约束（但若 PageWeave 想用更宽松的 MIT/Apache，则不能集成 BabelDOC——会被强制降为 AGPL）。
2. **网络交互触发源码披露（Section 13）**：即使不分发二进制，只要用户通过网络与服务交互（如 SaaS 形态），也必须向终端用户提供源码。对**纯本地桌面工具**（Tauri 本地运行、不提供网络服务）此条**不触发**——这是桌面场景的利好。
3. **API Key 等业务逻辑**：AGPL 不要求披露配置数据，仅限源码。

**结论**：PageWeave 作为开源本地桌面工具，集成 BabelDOC 在许可证上**可行**，但需将 PageWeave 整体以 AGPL-3.0（或兼容协议如 GPL-3.0）开源。若 PageWeave 未来想闭源商业化，则**不可集成**。

## 三、三种集成方式对比

### (a) 作为 Python 库直接 import（`import babeldoc`）

- 可行性：技术上可行但有重大障碍。BabelDOC 官方声明"All APIs of BabelDOC should be considered as internal APIs"。
- 复杂度：高。需在 Tauri 进程内嵌 Python（PyO3）或调起 Python 子进程跑脚本。
- Windows 适配：**重大障碍**——依赖 `hyperscan>=0.7.13`，hyperscan 在 PyPI 上没有 Windows wheel。
- 打包难度：极高（CPython + ~30 个重型依赖，200MB+）。
- 维护成本：高（内部 API 不稳定，TranslationConfig 字段频繁变动）。

### (b) 作为 CLI 子进程由 Rust/Tauri 调用（`Command::new("babeldoc")`）

- 可行性：较高，但**仍受 Windows 安装障碍制约**。
- 复杂度：中。`tokio::process::Command` + 管道读取 stdout/stderr，解析 progress 输出。
- Windows 适配：仍受 hyperscan 阻塞。
- 打包难度：低-中。要求用户预装 Python+BabelDOC（或随安装包附带便携 Python + 预装 wheel）。
- 维护成本：低。CLI 参数比内部 API 稳定得多。
- 进度获取：rich/tqdm 进度条输出到 stderr，需解析 ANSI；或用 `--report-interval 0.1` + `--debug` 拿结构化日志。

### (c) 打包成独立 sidecar 可执行文件

- 可行性：理论可行，工作量大（PyInstaller/Nuitka 打成单 exe；Tauri externalBin 分发）。
- Windows 适配：需在 Windows 构建机上成功 `pip install BabelDOC` 才能 PyInstaller 打包——而 hyperscan 阻塞了这一步。**必须先解决 hyperscan 问题**。
- 打包难度：极高。单 exe 体积 150–300MB。
- 用户体验：最佳——开箱即用，无需用户装 Python。

### 三者综合

| 方式 | 可行性 | 复杂度 | Windows 友好度 | 打包难度 | 维护成本 | 用户体验 |
|---|---|---|---|---|---|---|
| (a) Python 库 import | 中 | 高 | 差（hyperscan） | 极高 | 高 | 中 |
| (b) CLI 子进程 | 高 | 中 | 差（hyperscan） | 中 | 低 | 中（需用户装 Python） |
| (c) Sidecar exe | 中（需先绕过 hyperscan） | 高 | 差→需 fork | 极高 | 高 | 最佳 |

## 四、CLI 关键参数

**输入/输出**：`--files`（可重复）、`--output`/`-o`、`--pages`/`-p`、`--working-dir`
**语言**：`--lang-in`/`-li`（默认 en）、`--lang-out`/`-lo`（默认 zh）
**翻译服务（OpenAI 兼容）**：`--openai`、`--openai-model`（默认 gpt-4o-mini）、`--openai-base-url`、`--openai-api-key`、`--qps`（默认 4）、`--pool-max-workers`、`--ignore-cache`
**双语/单语输出**：`--no-dual`、`--no-mono`、`--dual-translate-first`、`--use-alternating-pages-dual`、`--watermark-output-mode`、`--only-include-translated-page`
**PDF 处理**：`--skip-clean`、`--disable-rich-text-translate`、`--enhance-compatibility`（三者组合）、`--split-short-lines`、`--translate-table-text`（实验性）、`--formular-font-pattern`、`--skip-form-render`、`--primary-font-family`、`--max-pages-per-part`
**OCR**：`--ocr-workaround`、`--auto-enable-ocr-workaround`（>80% 扫描页自动启用）、`--skip-scanned-detection`
**术语表**：`--glossary-files`（CSV，可重复）、`--no-auto-extract-glossary`、`--save-auto-extracted-glossary`
**配置/调试**：`--config`/`-c`（TOML）、`--debug`、`--report-interval`、`--warmup`、`--generate-offline-assets`/`--restore-offline-assets`

### 真实可跑示例命令

```bash
babeldoc \
  --files input.pdf \
  --output ./out \
  --lang-in en --lang-out zh \
  --openai \
  --openai-model "gpt-4o-mini" \
  --openai-base-url "https://api.openai.com/v1" \
  --openai-api-key "sk-xxxxxxxx" \
  --qps 4 \
  --enhance-compatibility \
  --auto-enable-ocr-workaround \
  --report-interval 0.1
```

输出：`./out/input-mono.pdf`（单语）、`./out/input-dual.pdf`（双语）。

### Windows 适配的致命问题（⚠️ 已被实证推翻 — 见下方"关键更正"）

> 历史结论（基于网络调研，**已过时**）：BabelDOC 将 `hyperscan>=0.7.13` 列为硬依赖，而 hyperscan 在 PyPI 上"仅提供 Linux/macOS wheel，没有 Windows wheel"，因此 Windows 上 `pip install BabelDOC` 会因从 sdist 编译 hyperscan 而失败。

### ⚠️ 关键更正（2026-07-01 本机实证）

在本机（Windows 11，Python 3.11.15）一个**全新 venv** 中实测：

```
python -m venv /tmp/bd_venv
/tmp/bd_venv/Scripts/python.exe -m pip install BabelDOC
→ Successfully installed BabelDOC-0.6.3 hyperscan-0.8.2 ...（含全部 ~30 个依赖，无编译失败）
```

随后验证：
```
import hyperscan            → OK，version 0.8.2
hyperscan.Database()        → OK
HS_FLAG_CASELESS / SINGLEMATCH → 可用
import babeldoc             → OK，version 0.6.3
from babeldoc.format.pdf.high_level import translate, TranslationConfig, async_translate → OK
from babeldoc.glossary import Glossary → OK
babeldoc --help             → CLI 正常输出全部参数
```

**结论更正**：截至 2026-07，`hyperscan 0.8.2` 在 Windows 上**可正常安装并工作**（pip 拉到了可用的 Windows 安装产物，无需用户自备 C++ 工具链）。`pip install BabelDOC` 在 Windows 上**开箱即装**。原调研报告的"致命阻塞"判断**不成立**。

**对集成方案的影响**：
- 方式 (b) CLI 子进程的"Windows 安装障碍"风险**解除**——用户只需 `pip install BabelDOC` 即可，无需 fork、无需 vectorscan、无需预编译。
- 方式 (c) sidecar exe 的可行性也**提升**——可在 Windows 构建机上直接 `pip install BabelDOC` 再用 PyInstaller 打包，无需先解决 hyperscan。
- 因此 MVP **不再需要**"fork BabelDOC 替换 hyperscan"这一前置阻塞。可降级为"建议在 README 注明用户需预装 Python 3.10–3.13 + `pip install BabelDOC`"。

**hyperscan 在 BabelDOC 中的实际用途**（已确认）：仅用于 `babeldoc/glossary.py`——`Glossary` 类用 `hyperscan.Database` 把术语表编译成多模式正则数据库（`HS_FLAG_CASELESS | HS_FLAG_SINGLEMATCH`，按 20000 条一批分块），再用 `hyperscan.Scratch` + `db.scan()` 在原文里快速找命中的术语条目。即仅在**启用术语表**时用到。这意味着：即便将来某天 hyperscan 在某 Windows 环境装不上，也可通过"不启用术语表"或"用 `re` 做降级匹配"来绕过——风险面比想象的小。

**遗留待验证项**（留到集成阶段做，不阻塞规划）：
- 首次运行会从 HuggingFace 下载 DocLayout-YOLO 模型与字体资源（需联网；可用 `--generate-offline-assets`/`--restore-offline-assets` 预打包）。
- 真实 PDF + API Key 的端到端翻译流程尚未在本机跑通（需 API Key，属集成测试范畴）。

## 五、Python API 关键入口

**异步入口（推荐，流式 progress）**：
```python
from babeldoc.format.pdf.high_level import async_translate
async for event in async_translate(translation_config):
    # event: dict，含 type=progress_start|progress_update|progress_end|finish|error
    # 字段: stage, stage_progress(0-100), overall_progress(0-100), part_index, total_parts
    # finish 事件带 translate_result
```

**`TranslationConfig` 关键字段**：`translator`、`input_file`、`lang_in`/`lang_out`、`doc_layout_model`、`pages`、`output_dir`、`no_dual`/`no_mono`、`qps`、`enhance_compatibility`、`dual_translate_first`、`use_alternating_pages_dual`、`ocr_workaround`/`auto_enable_ocr_workaround`、`skip_scanned_detection`、`glossaries`、`auto_extract_glossary`、`pool_max_workers`、`custom_system_prompt`、`working_dir`、`watermark_output_mode`、`report_interval`、`debug`。

**`OpenAITranslator` 构造**：`OpenAITranslator(lang_in, lang_out, model, base_url=None, api_key=None, ...)`。

**官方推荐的封装层：pdf2zh_next（PDFMathTranslate-next）**——稳定、文档化的公开 API `do_translate_async_stream`，事件类型 `stage_summary|progress_start|progress_update|progress_end|finish|error`，finish 含 `translate_result`（mono_pdf_path, dual_pdf_path 等）。`SettingsModel` 字段分层：`basic`/`translation`/`pdf`/`gui_settings`/`translate_engine_settings`。OpenAI 兼容引擎配置在 `OpenAICompatibleSettings`。

## 六、PDF 格式保留能力

- 用 `pymupdf` + 自研 `babeldoc.pdfminer` 解析 PDF → 抽取文本块、图片、表格、公式到 IR（`babeldoc.format.pdf.document_il`）
- `DocLayout-YOLO`（ONNX 模型）做版面分析
- pipeline：`LayoutParser` → `TableParser` → `ParagraphFinder` → `StylesAndFormulas` → `ILTranslator` → `Typesetting` → `PDFCreater`
- 译文按 IR 重新排版回 PDF，保留原字体映射、坐标、图片图层
- `fix_cmap` 修正字符映射；`migrate_toc` 迁移书签目录

**保留项**：段落结构、图片、表格（实验性）、公式、书签 TOC、字体族。
**局限**：作者信息/参考文献区解析有错；不支持线条、首字下沉；大页会被跳过；表格翻译实验性；复杂公式可能丢字。

## 七、Windows 部署要点

### Python 依赖（重型，约 30 个）
- PDF：`pymupdf>=1.26.7`（有 Windows wheel，OK）
- ML：`onnx`、`onnxruntime`、`numpy`（Windows wheel OK）
- CV：`opencv-python-headless`、`scikit-image`、`scipy`、`scikit-learn`（Windows wheel OK，体积大）
- 字体：`freetype-py`、`uharfbuzz`（Windows wheel OK）
- 正则：`hyperscan>=0.7.13`——**无 Windows wheel，致命阻塞**
- 其他：`httpx[socks]`、`openai`、`tiktoken`、`pydantic`、`peewee`、`psutil`、`rtree`、`pyzstd`、`cryptography`、`Levenshtein`

### 原生依赖风险
1. **hyperscan**（最高风险）：无 Windows wheel。
2. **zstd/pyzstd**：issue #359 报告过编译失败。
3. **DocLayout-YOLO 模型**：首次运行需从 HuggingFace 下载，需联网或用 `--generate-offline-assets` 预打包。
4. **字体资源**：`download_font_assets()` 会下载字体。

## 八、集成风险

| 风险类别 | 具体内容 | 严重度 |
|---|---|---|
| 许可证 | AGPL-3.0 传染性。PageWeave 必须整体 AGPL-3.0 开源 | 高（开源可接受） |
| Windows 安装阻塞 | `hyperscan` 无 Windows wheel，`pip install BabelDOC` 在 Windows 默认环境失败 | **致命** |
| 版本绑定 | BabelDOC 内部 API 不稳定；TranslationConfig 字段常变 | 高（走 CLI 或 pdf2zh_next 封装层更稳） |
| 子进程进度获取 | CLI 模式 progress 走 rich/tqdm stderr ANSI 输出，解析脏 | 中 |
| API Key 传递安全 | CLI `--openai-api-key` 会出现在进程命令行，其他进程可见；应改用环境变量 `OPENAI_API_KEY` 或 `--config` TOML 文件（权限 600） | 中-高 |
| 大文件/多页性能 | 长文档耗时数十分钟；`--max-pages-per-part` 分片 + `--qps` 限流；需 Tauri 侧做取消（kill）和断点续传（BabelDOC 自带翻译缓存 peewee） | 中 |
| 模型下载 | 首次运行下载 DocLayout-YOLO 模型，需联网或预置 | 低-中 |

## 九、推荐结论（MVP 阶段）

### 推荐方式：(b) CLI 子进程 + pdf2zh_next 配置层

> 2026-07-01 更正：原推荐结论里"必须先解决 hyperscan"的前置阻塞**已解除**——本机实测 `pip install BabelDOC` 在 Windows 上开箱即装即用（hyperscan 0.8.2 现可在 Windows 正常安装）。详见第二节"关键更正"。

**理由**：
1. 许可证可行（开源 + 本地桌面不触发 Section 13）。
2. API 稳定性：CLI 参数和 `pdf2zh_next.do_translate_async_stream` 是文档化的稳定入口，升级成本最低。
3. 解耦：Tauri（Rust）与 BabelDOC（Python）进程隔离，一方崩溃不影响另一方。
4. 进度与取消：子进程模式可用 `--report-interval` + stderr 解析拿进度；取消直接 kill。
5. (c) sidecar exe 是 (b) 的演化方向：MVP 先用 (b) 验证流程，待 hyperscan 解决后再用 PyInstaller 打 sidecar exe。

### 必须解决的前置阻塞
- ~~**Fork BabelDOC 移除/替换 hyperscan**~~ **已解除**（2026-07-01 本机实测 hyperscan 0.8.2 在 Windows 可正常安装）。降级为：README 注明用户需预装 Python 3.10–3.13 并 `pip install BabelDOC`；未来若某环境装不上，可"不启用术语表"绕过（hyperscan 仅 `glossary.py` 用到）。
- **预置模型与字体资源**：用 `--generate-offline-assets` 在构建期打包，避免用户首次运行下载（待集成阶段验证）。

### API Key 安全传递
不用 CLI `--openai-api-key`，改用环境变量：Tauri 侧 `Command::env("OPENAI_API_KEY", key)` 启动子进程，BabelDOC 的 `OpenAITranslator` 会读取。或在 `--config` TOML 中放 `openai_api_key`，文件权限 600。

### MVP 实施路径
1. ~~在 Windows 构建机上验证 `pip install BabelDOC` 是否真的因 hyperscan 失败~~ **已验证：不失败，开箱即装**（2026-07-01 本机实测）。
2. ~~Fork BabelDOC 替换 hyperscan~~ **不再需要**。
3. Tauri 侧实现 `babeldoc` 子进程调用 + stderr 进度解析 + 环境变量传 Key。
4. 用 `--enhance-compatibility --auto-enable-ocr-workaround` 作为 MVP 默认参数。
5. 跑通端到端翻译后，再评估是否用 PyInstaller 打 sidecar exe（演进到方式 (c)），实现"用户免装 Python"。
