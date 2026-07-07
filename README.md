<p align="center">
  <a href="https://github.com/jhxxr/PageWeave">
    <img src="src-tauri/icons/128x128@2x.png" alt="PageWeave" width="128" height="128">
  </a>
</p>

<h1 align="center">PageWeave</h1>

<p align="center">
  Windows 本地 PDF 翻译桌面工具<br>
  拖入 PDF → 选择语言与 AI 服务商 → 输出保留原排版的译文 PDF
</p>
<p align="center">
  <a href="https://github.com/jhxxr/PageWeave/releases">
    <img src="https://img.shields.io/github/v/release/jhxxr/PageWeave?style=flat-square" alt="Release">
  </a>
  <a href="https://github.com/jhxxr/PageWeave/releases">
    <img src="https://img.shields.io/github/downloads/jhxxr/PageWeave/total?style=flat-square" alt="Downloads">
  </a>
</p>


---

## 简介

PageWeave 是一款 Windows 本地 PDF 翻译桌面应用。拖入 PDF 文件，选择源/目标语言、AI 服务商和模型，即可调用 [BabelDOC](https://github.com/funstory-ai/BabelDOC) 生成尽量保留原排版、图片与表格结构的译文 PDF。

- **纯本地运行**：翻译引擎以内置 sidecar 形式随应用分发，用户无需安装 Python。
- **API Key 本地加密**：Key 经 Windows Credential Manager（DPAPI）加密保存，SQLite 只存引用，导出配置默认不含 Key。
- **多服务商预设**：内置 OpenAI、DeepSeek、硅基流动、阿里云百炼、Moonshot、智谱、Ollama / LM Studio 等 OpenAI-compatible 预设。
- **灵活的输出模式**：仅译文、原文+译文双语对照、全部输出可选。
- **高级翻译参数**：页码范围、术语表 CSV、OCR 模式、并发池、自定义 system prompt 等。

## 功能

| 模块 | 能力 |
|------|------|
| PDF 翻译 | 拖拽或点击选择 PDF，实时进度、日志、取消、完成后打开文件/文件夹 |
| AI API 配置 | 新增/编辑/删除服务商，测试连接，拉取 `/models` 模型列表，导出配置（不含 Key） |
| 翻译参数 | 页码范围、术语表、字体与排版、OCR 与兼容性、缓存与并发、OpenAI 调参 |
| 任务管理 | 查看当前任务状态、输出文件、最近日志 |
| 设置 | 主题/语言、默认输出目录与语言、默认服务商、离线资源包安装、自动更新 |

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面端 | Tauri 2.x + React 19 + TypeScript + Ant Design + Zustand + i18next + Vite |
| 后端 | Rust（SQLite via rusqlite，API Key via keyring → Windows Credential Manager + DPAPI） |
| 翻译引擎 | BabelDOC，以内置 sidecar 形式随应用分发 |

## 快速开始

```bash
pnpm install
pnpm tauri dev
```

环境要求：Node ≥ 20、pnpm ≥ 9、Rust 工具链（rustup + stable + MSVC build tools）。

首次翻译前，请在「设置」页安装 BabelDOC 离线资源包（模型 + 字体）。

## 构建 sidecar（内置翻译引擎）

sidecar 不随 `tauri dev` 自动构建，首次需单独执行：

```bash
bash sidecar/build_sidecar.sh [/path/to/venv/python]
# 默认使用 /tmp/bd_venv/Scripts/python.exe
```

脚本会：安装 PyInstaller、预下载模型/字体、PyInstaller 冻结、补拷 MSVC 运行时、生成离线资源包、复制并重命名 sidecar、烟测 `--version`。

开发期可直接使用 `sidecar/dist/babeldoc-sidecar/`；手动拷贝 exe 时务必连同同级 `_internal/` 一起拷。

## 打包与分发

```bash
pnpm tauri build
```

`tauri.conf.json` 通过 `externalBin` 引入 sidecar exe，并通过 `resources` 打包完整 `sidecar/dist/babeldoc-sidecar/` 目录，安装后无需 Python 前置依赖。

## 安全

- API Key 存入 Windows Credential Manager（DPAPI 加密），SQLite 仅保存 `api_key_id` 引用。
- 导出配置默认不含 API Key。
- 日志对 `sk-...` 类令牌做掩码。
- CSP 显式白名单。

## 许可证

AGPL-3.0-or-later。本项目集成 BabelDOC（AGPL-3.0），依其条款须整体开源。
