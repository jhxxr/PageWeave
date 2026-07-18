# Add markitdown document-to-Markdown conversion

## Goal

在 PageWeave 中以**可剥离模块**形式集成 [microsoft/markitdown](https://github.com/microsoft/markitdown)，提供「本地文档 → Markdown」能力；与 BabelDOC PDF 翻译链路平行、独立，便于后期改动或整块删除。

用户价值：拖入/选择办公文档，一键得到 LLM 友好的 `.md`，无需安装 Python，也无需 API Key。

## Background

- 产品：Tauri 2 + React + Rust；PDF 翻译经内置 `babeldoc-sidecar`（PyInstaller one-folder）子进程完成。
- markitdown：MIT；PyPI `0.1.6`；Python ≥ 3.10；CLI `markitdown <file> -o out.md`；定位为 LLM 文本抽取，非高保真排版。
- 现有模式可复用：sidecar 解析、子进程 spawn、stderr 日志、取消 kill、打开文件/文件夹。
- 许可：markitdown MIT 与项目 AGPL-3.0 共存无额外传染；项目仍整体 AGPL 开源。

## Decisions

| # | Decision | Choice |
|---|----------|--------|
| D1 | MVP 输入格式 | PDF + DOCX + PPTX + XLSX/XLS（`markitdown[pdf,docx,pptx,xlsx,xls]`） |
| D2 | 应用内 MD 预览 | 不做；成功后路径 + 打开文件/文件夹 |
| D3 | 批量 | 单文件；批量二期 |
| D4 | LLM 图片描述 | 不做；纯本地，不读 API Key |
| D5 | Feature flag | 不做；靠模块边界剥离 |
| D6 | 与翻译并发 | 模块内单任务；跨模块可并存 |
| D7 | 输出命名 | `{stem}.md`；重名则加时间戳后缀（不覆盖） |

## Requirements

- **R1** 用户可将本地文档转换为 Markdown 并落盘，成功后可打开文件/文件夹。
- **R2** 引擎以内置 sidecar 分发，终端用户无需 Python/pip。
- **R3** 与 BabelDOC 翻译解耦：独立 sidecar、构建脚本、Tauri 打包项、Rust 模块、前端路由/侧栏；design 含剥离清单。
- **R4** 仅本地离线转换；禁止远程 URI、Azure、第三方插件、网络依赖（MVP）。
- **R5** 可取消进行中的转换（子进程 kill，Windows 无僵尸进程）。
- **R6** 输出目录：应用默认输出目录或当次用户选择。
- **R7** 输入扩展名白名单：`.pdf` `.docx` `.pptx` `.xlsx` `.xls`；前后端一致校验。
- **R8** 无应用内 Markdown 预览（D2）。
- **R9** 单次任务仅一个输入文件（D3）。
- **R10** 不使用 LLM/API Key（D4）。
- **R11** 无 feature flag（D5）；功能默认可见可用。
- **R12** convert 同时最多 1 个运行中任务；不阻塞 translate（D6）。
- **R13** 输出文件名：`{源 stem}.md`；若目标已存在，改为 `{stem}-YYYYMMDD-HHMMSS.md`（本地时区或 UTC 需在 design 固定一种），**不静默覆盖**（D7）。
- **R14** 进度 UI：indeterminate / 状态文案 + 日志即可（markitdown 无 stage 百分比）。
- **R15** 文案标明：输出偏 LLM/文本分析，非印刷级排版还原。

## Acceptance Criteria

- [ ] **AC1** 对 PDF / DOCX / PPTX / XLSX / XLS 各至少一份样例成功写出非空 `.md`，且可打开。
- [ ] **AC2** 无系统 Python 时，内置 sidecar 仍可完成转换。
- [ ] **AC3** design 剥离清单所列路径删除后，翻译链路可编译运行（不引用 convert）。
- [ ] **AC4** 转换中取消成功；无残留 sidecar 子进程。
- [ ] **AC5** 侧栏/路由有独立「转 Markdown」入口，与 PDF 翻译页分离。
- [ ] **AC6** 非白名单扩展名：UI 禁用或明确错误，不启动 sidecar。
- [ ] **AC7** 成功态仅路径 + 打开文件/文件夹，无 MD 渲染预览。
- [ ] **AC8** 单文件任务；无多文件队列 UI。
- [ ] **AC9** 转换路径不读 API Key、不发起业务网络请求。
- [ ] **AC10** 无 feature flag 代码路径。
- [ ] **AC11** 第二路 convert 被拒绝并提示；同时 start translate 不被 convert 阻塞。
- [ ] **AC12** 目标 `{stem}.md` 已存在时，写入带时间戳的新文件且原文件保持不变。

## Out of Scope

- Azure Document Intelligence / Content Understanding
- markitdown 插件、YouTube、远程 URI
- 图片 / HTML / CSV / ZIP / EPub 等非 R7 格式
- 应用内 MD 预览、批量/并发 convert、LLM 描述/OCR
- 运行时或编译期 feature flag
- 全局 convert↔translate 互斥
- 与 BabelDOC 自动串联
- 高保真排版还原
- 完整任务历史库（MVP 可不落库；若实现须仍可随模块剥离）

## Non-goals / Product notes

- 不与「PDF 翻译」合并为同一页。
- 不向用户暗示「完美还原 Word/PPT 版式」。
