我想开发一个 Windows 本地桌面翻译工具，目标是做成一个“本地运行、API 可配置、尽量保留 PDF 原格式”的文档翻译器。请你作为资深全栈工程师、Tauri 桌面应用工程师、Python 文档处理工程师，帮我从 0 到 1 设计并实现这个项目。

一、参考项目

请参考以下两个开源项目的设计思路，但不要盲目复制代码：

BabelDOC
GitHub: https://github.com/funstory-ai/BabelDOC
参考点：
使用它作为保留 PDF 格式翻译的核心能力参考。
优先研究它的 CLI 和 Python API。
支持 PDF 翻译、双语输出、保留版式、调用 OpenAI-compatible API。
需要评估如何在 Windows 本地桌面应用中调用它：作为 Python 库、CLI 子进程，或独立 sidecar。
ai-toolbox
GitHub: https://github.com/coulsontl/ai-toolbox
参考点：
参考它的 AI API 配置管理界面。
参考它的 Tauri + React + TypeScript 桌面架构。
参考供应商、模型、API Key、Base URL、代理、配置导入导出、本地备份、主题切换等设计。
这个项目的配置体验很好，我希望我的翻译器也能方便地配置不同 AI API。
二、项目目标

开发一个 Windows 本地桌面应用，暂定名为：

Local Format Translator

核心目标：

用户可以在 Windows 上本地打开应用。
用户可以拖入 PDF 文件。
用户可以选择源语言和目标语言，例如英文到中文、日文到中文、中文到英文。
用户可以选择 AI 服务商、模型、Base URL、API Key。
工具调用 BabelDOC 或类似能力完成 PDF 翻译。
输出结果尽量保留原 PDF 的排版、图片、表格、公式、段落结构。
支持单语译文 PDF 和双语对照 PDF。
支持批量翻译多个 PDF。
API Key 只保存在本地，不上传到任何自建服务器。
整个工具优先服务个人本地使用，后续再考虑打包分发。
三、重要要求

请先完成“技术方案设计”，不要一上来就直接写代码。
你需要先分析这两个参考项目的结构、许可证、可复用方式和集成风险，然后给出推荐架构。

尤其要注意：

BabelDOC 是 AGPL-3.0 许可证，ai-toolbox 是 MIT 许可证。不过放心我的项目是开源项目无需担心。

四、推荐技术栈

请优先采用以下技术栈：

桌面端：

Tauri 2.x
React
TypeScript
Ant Design
Zustand 或 Jotai 做状态管理
i18next 做中英文界面
Vite 构建

后端/本地能力：

Tauri Rust 后端负责文件选择、进程调用、本地配置读写、安全存储、日志流转。
Python 侧负责调用 BabelDOC，或者由 Rust 直接调用 BabelDOC CLI。
MVP 可以采用“Rust 调用 BabelDOC CLI 子进程”的方式，降低集成复杂度。
后续再抽象成 Python service 或 Rust command wrapper。

配置存储：

普通配置可以存储在本地 SQLite、SurrealKV、JSON 或 Tauri store 中。
API Key 必须加密保存，优先使用 Windows Credential Manager、系统 Keyring 或 Tauri 安全存储插件。
支持导入导出配置，但导出时默认不导出 API Key，除非用户主动选择加密导出。
五、核心功能模块

请把项目拆成以下模块设计。

1. 首页 / 翻译任务页

功能：

拖拽上传 PDF。
支持选择多个 PDF。
显示文件名、大小、页数、状态。
支持选择输出目录。
支持选择源语言和目标语言。
支持选择输出模式：
仅译文 PDF
双语对照 PDF
原文 + 译文分开输出
支持选择翻译配置：
服务商
模型
Base URL
API Key
并发数
超时时间
重试次数
开始翻译按钮。
暂停、取消、重试按钮。
翻译进度条。
实时日志面板。
完成后显示“打开文件”“打开所在文件夹”。
2. AI API 配置页

参考 ai-toolbox 的配置体验，设计一个清晰的供应商管理界面。

功能：

新增 / 编辑 / 删除供应商。
内置常见 OpenAI-compatible 预设：
OpenAI
DeepSeek
硅基流动
阿里云百炼 / Qwen
Moonshot / Kimi
Zhipu / GLM
Gemini-compatible 或自定义
Ollama / LM Studio 本地模型
每个供应商字段：
名称
Base URL
API Key
默认模型
模型列表
是否启用
代理设置
请求超时
最大并发
支持“测试连接”。
支持“拉取模型列表”，如果供应商兼容 /models 接口。
支持手动添加模型。
支持设置默认供应商和默认模型。
支持导入 / 导出配置。
API Key 输入框默认隐藏，可点击显示。
3. 翻译参数页

功能：

源语言 / 目标语言。
是否启用术语表。
术语表导入 CSV。
是否启用上下文增强。
是否跳过已翻译缓存。
最大并发数。
每个 PDF 分片页数。
是否生成双语 PDF。
是否保留中间文件。
是否开启 debug 日志。
是否启用 OCR workaround。
是否跳过扫描 PDF 检测。
自定义 BabelDOC 参数输入框，高级用户可手动追加参数。
4. 任务管理页

功能：

显示历史任务。
字段包括：
文件名
源语言
目标语言
使用模型
开始时间
结束时间
状态
输出路径
错误信息
支持重新运行任务。
支持打开输出文件。
支持删除历史记录。
支持复制错误日志。
5. 设置页

功能：

主题：浅色、深色、跟随系统。
语言：中文、英文。
默认输出目录。
默认源语言和目标语言。
默认翻译供应商。
默认模型。
自动检查更新。
日志保存天数。
缓存目录管理。
清理缓存。
备份和恢复配置。
关于页面：显示版本、许可证、依赖项目说明。
六、MVP 范围

请优先实现 MVP，不要一开始做太复杂。

MVP 必须包含：

Windows 桌面应用壳子。
PDF 文件选择和拖拽。
AI API 配置管理：
Base URL
API Key
Model
测试连接
调用 BabelDOC CLI 完成一个 PDF 的翻译。
显示实时日志。
显示进度状态。
输出译文 PDF。
打开输出文件夹。
本地保存配置。
基础错误提示。

MVP 可以暂时不做：

DOCX / PPTX 翻译。
复杂任务队列。
云同步。
用户账号。
在线支付。
插件系统。
OCR 深度优化。
多语言 UI 的完整翻译。
自动更新。