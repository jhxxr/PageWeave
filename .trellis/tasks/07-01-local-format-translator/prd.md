# PageWeave — 本地 PDF 翻译桌面工具（MVP）

> 项目代号 PageWeave。guide.md 暂定名 "Local Format Translator"，采用工作目录名 PageWeave 作为正式名。

## Goal

做一个 Windows 本地运行的桌面 PDF 翻译工具：用户拖入 PDF → 选源/目标语言 → 选 AI 服务商/模型/Base URL/API Key → 调用 BabelDOC 完成翻译 → 输出尽量保留原排版的译文 PDF。API Key 只存本地、加密保存。优先服务个人本地使用。

本轮交付范围 = guide.md 定义的**严格 MVP**。

## 背景 / 关键决策（来自调研）

- **集成 BabelDOC 的方式**：Rust 后端用 `tokio::process::Command` 调用 `babeldoc` CLI 子进程（不走 Python 库 import，也不打 sidecar exe）。理由：CLI 参数稳定、进程隔离、升级成本低。详见 `research/babeldoc.md`。
- **Windows 可行性已实证**：本机 Python 3.11 实测 `pip install BabelDOC` 开箱即装（hyperscan 0.8.2 在 Windows 可用），无需 fork。用户前置依赖 = Python 3.10–3.13 + `pip install BabelDOC`。
- **许可证**：BabelDOC 是 AGPL-3.0 → PageWeave 须整体 AGPL-3.0 开源（本项目开源，可接受；纯本地桌面不触发 Section 13）。
- **API Key 存储**：用 `keyring` crate（Windows Credential Manager + DPAPI），不照搬 ai-toolbox 的明文 JSON。SQLite 存配置元数据 + `api_key_id` 引用。
- **架构借鉴**：ai-toolbox 的 feature-driven 前端、JSON-blob SQLite 表、services 分层、antd ProviderFormModal 交互范式（借鉴设计，不抄代码；ai-toolbox 是 MIT）。

## MVP 必须包含（Requirements）

### R1 桌面应用壳子
- Tauri 2.x + React + TypeScript + Ant Design + Zustand + i18next + Vite。
- 单窗口，左侧导航 5 个入口（首页/翻译、API 配置、翻译参数、任务管理、设置）；MVP 阶段后三个页面可仅做占位 + 基本可用，重点实现前两个。
- 中文/英文 i18n 框架就位（MVP 只需中文完整、英文键就位即可）。

### R2 PDF 文件选择与拖拽
- 支持拖拽上传 PDF（Tauri `onDragDropEvent`）+ 文件选择对话框（`tauri-plugin-dialog`）。
- 支持选择多个 PDF；列表显示文件名、大小、状态。
- 选择输出目录（默认读设置页的默认输出目录）。

### R3 AI API 配置管理
- 供应商 CRUD（新增/编辑/删除）：字段 = 名称、Base URL、API Key、默认模型、模型列表、是否启用。
- 内置 OpenAI-compatible 预设：OpenAI、DeepSeek、硅基流动、阿里云百炼/Qwen、Moonshot/Kimi、Zhipu/GLM、Ollama/LM Studio、自定义。
- "测试连接"：发一个最小 chat 请求验证 Base URL + Key + Model 可用，返回成功/失败 + 错误信息。
- "拉取模型列表"：对兼容 `/v1/models` 的供应商拉取模型填入列表（失败可降级为手填）。
- 设置默认供应商和默认模型。
- API Key 输入框默认隐藏、可点击显示。
- API Key 加密保存（`keyring` crate），SQLite 只存引用 id。
- 配置导入/导出：导出默认不含 API Key。

### R4 调用 BabelDOC 完成翻译
- Rust command 接收：PDF 路径、输出目录、源语言、目标语言、供应商配置（base_url/api_key/model）、输出模式（仅译文/双语）、qps 等参数。
- Rust 用 `tokio::process::Command` 调 `babeldoc` CLI，构造参数：`--files <pdf> --output <dir> --lang-in <li> --lang-out <lo> --openai --openai-model <m> --openai-base-url <u> --openai-api-key <k> --qps <q> --enhance-compatibility --auto-enable-ocr-workaround --report-interval 0.1`，按输出模式加 `--no-dual`/`--no-mono`。
- **API Key 传递**：MVP 用 `--openai-api-key`（命令行参数，标注"进程命令行可见"风险，后续迭代改 `--config` toml）。
- **进度**：实时捕获 stdout/stderr，通过 Tauri event 推送到前端日志面板；用正则从 rich/tqdm 进度行解析 `overall_progress` 百分比推送进度条。
- **取消**：前端按钮 → Rust `kill` 子进程。
- **输出**：`<stem>-mono.pdf`（单语）/`<stem>-dual.pdf`（双语）。

### R5 实时日志与进度
- 翻译过程中实时日志面板（滚动、可复制）。
- 进度条（百分比 + 当前阶段名）。
- 完成后显示"打开文件""打开所在文件夹"（`tauri-plugin-opener`）。

### R6 本地配置保存
- 普通配置（UI 偏好、默认输出目录、默认语言、默认供应商）存 SQLite 或 Tauri store。
- API Key 存 `keyring`。
- 应用重启后配置恢复。

### R7 基础错误提示
- BabelDOC 子进程非零退出 → 解析 stderr 末尾错误 → 前端 message/Modal 提示。
- API Key 缺失/供应商未选 → 翻译前校验提示。
- Python/BabelDOC 未安装 → 检测 `babeldoc --version` 失败时给出安装引导。

## MVP 暂不做（Out of Scope）

- DOCX/PPTX 翻译；复杂任务队列/批量并发调度；云同步；用户账号；在线支付；插件系统；OCR 深度优化；多语言 UI 完整翻译；自动更新；术语表/上下文增强/缓存管理等高级参数页功能（参数页 MVP 仅占位 + 最常用几项）；任务管理页 MVP 仅展示当次任务（不做持久化历史）。

## Constraints

- 平台：Windows 10/11（MVP 不保证 macOS/Linux，但 Tauri 跨平台结构不破坏）。
- 用户前置依赖：自装 Python 3.10–3.13 + `pip install BabelDOC`（MVP 不内置 Python）。
- 许可证：PageWeave 须 AGPL-3.0 开源。
- API Key 不上传任何远端；只本地。
- 翻译不触网经自建服务器（直接用户 ↔ AI 供应商）。

## Acceptance Criteria

- [ ] AC1 `pnpm tauri dev` 能在 Windows 启动桌面窗口，左侧导航可见 5 个入口。
- [ ] AC2 拖拽一个 PDF 到首页 → 列表出现该文件（文件名 + 大小 + 状态"待翻译"）。
- [ ] AC3 在 API 配置页新增一个供应商（填 Base URL/API Key/Model）→ 保存 → 重启应用后该供应商仍在，且 API Key 从 keyring 正确恢复。
- [ ] AC4 点"测试连接" → 对合法配置返回成功；对错误 Key/URL 返回失败 + 可读错误信息。
- [ ] AC5 点"拉取模型列表" → 对兼容 `/v1/models` 的供应商能拉到模型并填入（对不兼容的给出降级提示）。
- [ ] AC6 在首页选好源/目标语言 + 供应商 + 输出模式 → 点"开始翻译" → BabelDOC 子进程被正确拉起并翻译该 PDF。
- [ ] AC7 翻译过程中日志面板实时滚动输出 BabelDOC 的 stderr，进度条推进。
- [ ] AC8 翻译完成后，输出目录下生成译文 PDF（`*-mono.pdf` 或 `*-dual.pdf`），且首页出现"打开文件""打开所在文件夹"按钮，点击有效。
- [ ] AC9 翻译中点"取消" → 子进程被 kill，状态变为"已取消"。
- [ ] AC10 BabelDOC 未安装时，启动翻译给出清晰的"请先 pip install BabelDOC"引导，而非崩溃。
- [ ] AC11 导出配置 → 导出文件不含 API Key 明文（仅含引用占位）。
- [ ] AC12 中英文 i18n 切换不报错（中文完整、英文键存在）。

## Notes

- 详细技术设计见 `design.md`；执行清单见 `implement.md`。
- 调研报告见 `research/babeldoc.md`、`research/ai-toolbox.md`。
- 本 PRD 仅含需求/约束/验收标准，不放技术设计。
