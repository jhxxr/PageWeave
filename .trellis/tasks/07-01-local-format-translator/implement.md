# PageWeave 实施计划（implement.md）

> 对应 `prd.md` / `design.md` 的 MVP。按顺序执行，每步含验证命令与回滚点。

## 阶段 0：脚手架与工具链（gate G0）

- [ ] 0.1 在仓库根 `git init`（当前非 git 仓库；guide.md 不要求 git，但便于回滚——征得默认后初始化，首提交含现有 .trellis/.claude/.codex/.opencode/.agents/AGENTS.md/guide.md）。
- [ ] 0.2 `pnpm create tauri-app` 选 React + TS（pnpm），生成到当前目录（`./`）。产物：`package.json`/`vite.config.ts`/`tsconfig.json`/`index.html`/`src/`/`src-tauri/`。
- [ ] 0.3 调整 `tauri.conf.json`：productName=`PageWeave`，identifier=`com.pageweave.app`，窗口标题/尺寸（1100×760），`fileDropEnabled=true`，CSP 白名单，`app.security.csp` 显式设置。
- [ ] 0.4 装 Rust 依赖：`cd src-tauri && cargo add` 各 crate（见 design §9）；装前端依赖：`pnpm add antd @ant-design/icons zustand i18next react-i18next react-router-dom`、`pnpm add @tauri-apps/plugin-dialog @tauri-apps/plugin-opener`、`pnpm add -D @types/node`。
- [ ] 0.5 `pnpm install`。
- **G0 验证**：`pnpm tauri dev` 能弹出空 Tauri 窗口（默认 React 页）。`cd src-tauri && cargo check` 通过。
- **回滚点 R0**：脚手架生成失败 → 删除生成的 `src/`/`src-tauri`/配置，回到 0.2 重选模板参数。

## 阶段 1：Rust 后端骨架（gate G1）

- [ ] 1.1 `src-tauri/src/lib.rs`：`run()` + `tauri::Builder` 挂 `dialog`/`opener`/`fs` 插件 + `setup` 里开 SQLite（`rusqlite` bundled，db 文件落 `app_data_dir`）+ `app.manage(DbState)`。
- [ ] 1.2 `db/`：`schema.rs`（建 provider/app_settings/task_record 表 + json_extract 索引）、`migrations.rs`（IF NOT EXISTS 建表）、`helpers.rs`（json blob CRUD 通用函数）。
- [ ] 1.3 `error.rs`：`AppError`（thiserror）+ `impl Serialize`（前端可读 `{kind,message}`）。
- [ ] 1.4 `secrets.rs`：`keyring::Entry` 封装 `set/get/delete/has`，服务名 `PageWeave`。
- **G1 验证**：`cargo check` + `cargo test --lib`（如写 db/helpers 单测）通过；`pnpm tauri dev` 启动不崩（setup 日志可见 SQLite 已开）。
- **回滚点 R1**：db 初始化失败 → 检查 `app_data_dir` 权限与 rusqlite features。

## 阶段 2：provider 模块（gate G2）

- [ ] 2.1 `provider/model.rs`：`ProviderRecord`/`NewProvider`/`ProviderData` struct（serde，字段见 design §3.1）。前端 `src/types/provider.ts` 镜像。
- [ ] 2.2 `provider/presets.rs`：内置 8 个预设（OpenAI/DeepSeek/硅基流动/Qwen/Moonshot/Zhipu/Ollama/自定义）的默认 base_url + 常见 model 列表。
- [ ] 2.3 `provider/commands.rs`：`list_providers`/`get_provider`/`create_provider`/`update_provider`/`delete_provider`/`set_default_provider`/`reveal_api_key`。CRUD 联动 keyring。
- [ ] 2.4 `provider/connectivity.rs`：`test_provider_connection`（reqwest POST `<base_url>/chat/completions`，model=传入，messages=[{role:user,content:"ping"}]，max_tokens=1，超时 15s，返回 ok/message/latency）；`fetch_provider_models`（reqwest GET `<base_url>/models`，Bearer 头，解析 `data[].id`）。
- [ ] 2.5 `lib.rs` `generate_handler!` 注册 provider command。
- **G2 验证**：`cargo check`；前端临时调 `list_providers` 返回空数组；用真实 OpenAI key `test_provider_connection` 返回 ok。
- **回滚点 R2**：reqwest TLS 问题 → 切 `rustls-tls` feature。

## 阶段 3：前端 provider 页（gate G3）

- [ ] 3.1 `src/services/providerApi.ts`：每个 command 包 async 函数。
- [ ] 3.2 `src/stores/providerStore.ts`：Zustand，list/默认/CRUD action。
- [ ] 3.3 `src/features/provider/`：列表页（antd Table + 新增/编辑/删除按钮）、`ProviderFormModal`（category Select 预设、base_url Input、api_key Input+眼睛、model AutoComplete + 测试连接 + 拉取模型按钮、启用开关）、`StatusBadge`。
- [ ] 3.4 i18n key（zh/en）补齐 provider 页文案。
- **G3 验证**：`pnpm tauri dev` → provider 页能新增/保存/删除供应商，重启后仍在，API Key 眼睛能 reveal，测试连接有反馈。
- **回滚点 R3**：表单交互问题 → 缩到最小字段先跑通再补。

## 阶段 4：translate 模块（gate G4）

- [ ] 4.1 `translate/args.rs`：`build_babeldoc_args(req) -> Vec<String>`（design §6.1）。
- [ ] 4.2 `translate/progress.rs`：`ProgressParser`（状态机）吃字节 → 剥 ANSI → 正则取 `%` 与 stage，返回 `Option<ProgressEvent>` + 原始日志行。
- [ ] 4.3 `translate/runner.rs`：`run_translate(app, task_id, req)` —— 取 keyring key、探测 babeldoc、`Command::new("babeldoc")`、env 设 UTF8、spawn、存 `ChildHandle` 进 `AppState.tasks`、读 stderr（按 `\r`/`\n` 切逻辑行喂 parser + emit log/progress）、`wait` 拿退出码 emit status、扫输出文件。日志脱敏。
- [ ] 4.4 `translate/commands.rs`：`start_translate`(返回 task_id 后 spawn runner)/`cancel_translate`/`get_babeldoc_info`。`AppState` 用 `Mutex<HashMap<String, ChildHandle>>` + `tokio::sync`。
- [ ] 4.5 注册 translate command。
- **G4 验证**：`cargo check`；前端临时 `start_translate` 一个小 PDF（用 mock key）→ 看到 `translate://progress` 的 log 事件滚动（即使 key 错也能看到 babeldoc 起来 + 报错日志）。
- **回滚点 R4**：子进程找不到 babeldoc → `get_babeldoc_info` 提示安装；路径问题用 `which` 排查。

## 阶段 5：前端翻译首页（gate G5）

- [ ] 5.1 `src/services/translateApi.ts`：`startTranslate`/`cancelTranslate`/`getBabeldocInfo` + `listenProgress(cb)`（`@tauri-apps/api/event.listen`）。
- [ ] 5.2 `src/stores/translateStore.ts`：files 列表、currentTask{task_id,status,logs[],progress,stage}、action；`subscribeProgress()` 在 App 挂载调一次。
- [ ] 5.3 `src/features/translate/`：拖拽区（`onDragDropEvent`）、文件列表（文件名/大小/状态）、语言对 Select、供应商/模型 Select（读 providerStore 默认 + 可切换）、输出模式 Radio、输出目录选择（`plugin-dialog` open）、开始/取消按钮、日志面板（antd `Typography.Text` 滚动 + 复制）、进度条（antd Progress）、完成态"打开文件/打开文件夹"按钮（`plugin-opener`）。
- [ ] 5.4 `get_babeldoc_info` 未装 → 顶部 Alert 引导安装。
- [ ] 5.5 i18n 补齐。
- **G5 验证**：拖入 PDF → 选供应商（真实 key）→ 开始翻译 → 日志滚动 + 进度推进 → 完成后输出目录有 `*-mono.pdf`，"打开文件"有效；点取消能中断。
- **回滚点 R5**：进度解析全失败 → 退化为纯日志面板（功能仍可用），进度条置灰，记债后修 parser。

## 阶段 6：settings + 其余页面骨架（gate G6）

- [ ] 6.1 `settings/commands.rs`：`get_settings`/`save_settings`（app_settings 表 KV）。
- [ ] 6.2 `src/features/settings/`：主题（浅/深/系统）、语言（中/英）、默认输出目录、默认语言对、默认供应商、关于页（版本/许可证/依赖）。
- [ ] 6.3 `src/features/params/`、`src/features/tasks/`：MVP 占位页（"MVP 暂未实现"提示 + 可用的少量项：输出模式已在首页，参数页放 qps 输入即可）。
- [ ] 6.4 主题联动 ConfigProvider；语言联动 i18n。
- [ ] 6.5 导入/导出配置（不含 key）命令 `export_providers`/`import_providers`。
- **G6 验证**：主题/语言切换生效；设置重启保留；导出 JSON 不含 api_key 明文。
- **回滚点 R6**：设置覆盖逻辑 bug → 仅影响偏好，不影响翻译主链路。

## 阶段 7：端到端验收（gate G7 = AC1–AC12）

- [ ] 7.1 逐条跑 `prd.md` 的 AC1–AC12，每条记录结果。
- [ ] 7.2 README 写安装/构建/许可证/已知债（Python+BabelDOC 前置、API Key 命令行风险）。
- [ ] 7.3 `pnpm tauri build` 产出 Windows 安装包（验证可打包；不强制分发）。
- **G7 验证**：AC1–AC12 全绿；build 产物生成。
- **回滚点 R7**：build 失败（签名/wix）→ 只保 `dev` 可用，build 问题单独跟进。

## 跨阶段约束

- 每个 gate 前先 `cargo check` + `pnpm tauri dev` 烟测。
- 不在本任务里 `git commit`（Trellis 3.4 阶段统一提交）；但可在 G0 初始化 git 后留本地快照便于回滚。
- 子进程调用的 babeldoc 路径优先 `which("babeldoc")`；失败回退 `python -m babeldoc`（注：BabelDOC 入口是 `babeldoc` console script，`python -m` 需确认模块名，MVP 先只走 `babeldoc`）。
- 前后端共享类型手写镜像（Rust serde struct ↔ `src/types/*.ts`），MVP 不引 codegen。
