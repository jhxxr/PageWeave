# PageWeave 技术设计（design.md）

> 对应 `prd.md` 的 MVP。本文只讲技术设计：架构、目录、数据模型、契约、数据流、集成方式、安全、取舍。

## 1. 总体架构

```
┌─────────────────────────────────────────────────────────┐
│  前端 (web/)  React 19 + TS + Antd + Zustand + i18next  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ 翻译首页 │ │ API 配置 │ │ 参数页   │ │ 任务/设置│     │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘     │
│       └───────────┴────────────┴────────────┘            │
│                  services/*Api.ts (invoke 封装)            │
└──────────────────────────┬────────────────────────────────┘
                           │ Tauri invoke (command) + event
┌──────────────────────────┴────────────────────────────────┐
│  Rust 后端 (src-tauri/)                                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ provider │ │ translate│ │ settings │ │ secrets      │  │
│  │  module  │ │  module  │ │  module  │ │ (keyring)    │  │
│  └──────────┘ └────┬─────┘ └──────────┘ └──────────────┘  │
│                   │ tokio::process::Command                │
│              ┌────▼─────┐                                   │
│              │ babeldoc │  (外部 Python CLI 子进程)         │
│              │  CLI     │                                   │
│              └──────────┘                                   │
└────────────────────────────────────────────────────────────┘
```

**关键边界**：
- 前端只调 `services/*Api.ts`，不直接 `invoke`。
- Rust 按 module 组织 command，命名仿 ai-toolbox：`list_*`/`create_*`/`update_*`/`delete_*`/`test_*`/`fetch_*`。
- BabelDOC 是**外部子进程**，Rust 与之通过 stdin/环境变量传参、stdout/stderr 拿日志与进度、进程退出码判成功。Rust 不内嵌 Python。

## 2. 目录结构

```
PageWeave/
├── package.json                  # workspace root: 前端 deps + tauri 脚本
├── pnpm-lock.yaml
├── guide.md                      # 原始需求（只读参考）
├── README.md                     # 安装/构建/许可证说明
├── LICENSE                       # AGPL-3.0
├── index.html
├── vite.config.ts
├── tsconfig.json
├── src/                          # 前端源（用 src/，不用 web/，保持 Tauri 模板习惯）
│   ├── main.tsx
│   ├── App.tsx
│   ├── app/                      # 路由 + 全局 layout + 主题
│   ├── features/
│   │   ├── translate/            # 翻译首页：pages/components
│   │   ├── provider/             # API 配置页
│   │   ├── params/               # 翻译参数页（MVP 占位+少量项）
│   │   ├── tasks/                # 任务管理页（MVP 仅当次）
│   │   └── settings/             # 设置页
│   ├── shared/                   # 跨域复用：ProviderFormModal 片段、LanguagePair、StatusBadge
│   ├── services/                 # providerApi.ts / translateApi.ts / settingsApi.ts
│   ├── stores/                   # Zustand: providerStore / translateStore / settingsStore
│   ├── components/               # 通用 antd 封装
│   ├── types/                    # 前后端共享类型（与 Rust struct 对齐）
│   ├── i18n/                     # i18next: zh/en
│   └── utils/
└── src-tauri/                    # Rust 后端
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    ├── capabilities/             # Tauri 2 权限
    ├── icons/
    └── src/
        ├── main.rs               # 入口（不要写逻辑，只调 lib::run）
        ├── lib.rs                # run() + generate_handler! 注册全部 command
        ├── db/                   # SQLite (rusqlite bundled): schema/migrations/helpers
        │   ├── mod.rs
        │   ├── schema.rs
        │   └── migrations.rs
        ├── secrets.rs            # keyring 封装：set/get/delete secret
        ├── provider/             # 供应商 CRUD + 测试连接 + 拉取模型
        │   ├── mod.rs
        │   ├── commands.rs
        │   ├── model.rs          # ProviderRecord struct
        │   ├── presets.rs        # 内置预设供应商
        │   └── connectivity.rs   # test_connection / fetch_models
        ├── translate/            # 调 babeldoc 子进程
        │   ├── mod.rs
        │   ├── commands.rs       # start_translate / cancel_translate
        │   ├── runner.rs         # 子进程拉起 + stderr 读取 + 进度解析 + event 推送
        │   ├── args.rs           # babeldoc CLI 参数构造
        │   └── progress.rs       # rich/tqdm 进度行正则解析
        ├── settings/             # 全局设置读写
        │   └── commands.rs
        └── error.rs              # 统一 AppError -> serde 前端可读
```

## 3. 数据模型

### 3.1 SQLite（rusqlite bundled，JSON-blob 模式仿 ai-toolbox）

```sql
CREATE TABLE provider (
  id          TEXT PRIMARY KEY,          -- uuid v4
  data        TEXT NOT NULL,             -- JSON blob（见下）
  created_at  TEXT NOT NULL,             -- ISO8601
  updated_at  TEXT NOT NULL
);
CREATE INDEX idx_provider_applied ON provider(json_extract(data,'$.is_applied'));
CREATE INDEX idx_provider_sort    ON provider(json_extract(data,'$.sort_index'));

CREATE TABLE app_settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL                    -- JSON 值
);

CREATE TABLE task_record (               -- MVP 最小化：记录当次及少量历史
  id          TEXT PRIMARY KEY,
  data        TEXT NOT NULL,             -- JSON: file_name/lang_in/lang_out/model/started_at/ended_at/status/output_path/error
  created_at  TEXT NOT NULL
);
```

`provider.data` JSON：
```json
{
  "name": "OpenAI 官方",
  "category": "openai",            // openai|deepseek|siliconflow|qwen|moonshot|zhipu|ollama|custom
  "base_url": "https://api.openai.com/v1",
  "api_key_id": "prov_<uuid>",     // 引用 keyring 条目，不存明文
  "has_api_key": true,             // 是否已存 key（用于显示状态，不暴露 key 本身）
  "models": ["gpt-4o-mini","gpt-4o"],
  "default_model": "gpt-4o-mini",
  "is_enabled": true,
  "is_applied": false,             // 当前默认生效
  "sort_index": 0,
  "notes": "",
  "extra": {}                      // 预留：headers/timeout/max_concurrency
}
```

> 说明：MVP 用 `is_applied` 单选默认（同一时刻仅一条 `is_applied=true`），比 ai-toolbox 的多选 apply 更简单，足够 MVP。

### 3.2 keyring（API Key 真值）

- 服务名：`PageWeave`；账号名：`provider.data.api_key_id`（如 `prov_<uuid>`）。
- `secrets.rs`：`set_secret(id, value)` / `get_secret(id)` / `delete_secret(id)`，Windows 走 Credential Manager + DPAPI。
- 删除供应商时连带 `delete_secret`。
- 导出配置时**不读** keyring（`has_api_key` 标位但不导出明文）。

## 4. Rust Command 契约（前端 invoke 接口）

### provider 模块
| Command | 入参 | 返回 | 说明 |
|---|---|---|---|
| `list_providers` | – | `Vec<ProviderRecord>`（不含 api_key 明文） | 列表 |
| `get_provider` | `id` | `ProviderRecord` | 单条 |
| `create_provider` | `payload: NewProvider` | `ProviderRecord` | 新增，若带 api_key 则写 keyring |
| `update_provider` | `id, payload` | `ProviderRecord` | 更新；api_key 为空串=不变，非空=更新 keyring |
| `delete_provider` | `id` | `()` | 删 DB + keyring |
| `set_default_provider` | `id` | `()` | 置 `is_applied=true`，其余 false |
| `test_provider_connection` | `base_url, api_key, model` | `{ok: bool, message: string, latency_ms?: u64}` | 发 `POST /chat/completions` 最小请求 |
| `fetch_provider_models` | `base_url, api_key` | `{ok: bool, models: string[], message: string}` | `GET /models`（兼容 OpenAI 的 `/v1/models`） |

> `test_provider_connection` 与 `fetch_provider_models` 的 `api_key` 由前端从 keyring 经 `get_api_key(api_key_id)` 取回后传入——或更安全：前端只传 `api_key_id`，Rust 内部取 keyring。**MVP 选后者**：`test_provider_connection { api_key_id, base_url, model }`，Rust 内部 `get_secret`。

### translate 模块
| Command | 入参 | 返回/事件 | 说明 |
|---|---|---|---|
| `start_translate` | `TranslateRequest`（见下） | `task_id: string` | 拉起子进程，异步推进度 |
| `cancel_translate` | `task_id` | `bool` | kill 子进程 |
| `get_babeldoc_info` | – | `{installed: bool, version?: string, path?: string, hint: string}` | 探测 babeldoc 是否可用 |

`TranslateRequest`：
```ts
{
  task_id?: string,            // 不传则 Rust 生成
  pdf_paths: string[],         // MVP 取第一个；结构留多文件
  output_dir: string,
  lang_in: string,             // "en"
  lang_out: string,            // "zh"
  output_mode: "mono"|"dual"|"both",
  provider: { base_url: string, api_key_id: string, model: string },
  qps: number,
}
```

**事件**（Rust → 前端，`emit` 到 `translate://progress`）：
```ts
{ task_id, type: "log",      line: string, stream: "stdout"|"stderr" }
{ task_id, type: "progress", overall: number, stage: string, part_index?: number, total_parts?: number }
{ task_id, type: "status",   status: "running"|"success"|"error"|"cancelled", output_files?: string[], message?: string }
```

### settings 模块
| Command | 入参 | 返回 |
|---|---|---|
| `get_settings` | – | `AppSettings` |
| `save_settings` | `patch: AppSettings` | `AppSettings` |

`AppSettings`：`{ theme, language, default_output_dir, default_lang_in, default_lang_out, default_provider_id, log_retention_days, cache_dir }`。

### secrets 模块
仅内部用，不直接暴露给前端（避免前端任意读 key）。仅 `get_api_key(api_key_id)` 暴露给 `test/fetch` 流程的内部调用——**不注册为前端 command**。前端需要"显示 key"时通过一个受限 command `reveal_api_key(api_key_id)` 返回明文（用户主动点击眼睛图标触发）。

## 5. 数据流：一次翻译的完整路径

1. 前端首页：用户拖入 PDF → `translateStore` 持有文件列表。选语言/供应商/输出模式/输出目录。
2. 点"开始翻译" → `translateApi.startTranslate(req)` → Rust `start_translate` command。
3. Rust：
   a. 生成 `task_id`，从 keyring 取 `api_key = get_secret(api_key_id)`；取不到 → 立即返回 error event。
   b. 探测 `babeldoc --version`（首次或每次都探，缓存 5min）；不可用 → emit status error 含安装引导。
   c. 构造参数（`args.rs`），`tokio::process::Command::new("babeldoc")`，env 继承，`stdin`/`stdout`/`stderr` piped。
   d. spawn 后把 `Child` 存进 `AppState` 的 `tasks: Mutex<HashMap<String, ChildHandle>>`，立即返回 `task_id`。
   e. 异步 task 读 stderr 行 → emit `log` 事件 + 喂 `progress.rs` 解析 → emit `progress` 事件。
   f. `wait().await` 拿退出码：0 → emit status success + 输出文件列表（扫 output_dir 的 `*-mono.pdf`/`*-dual.pdf`）；非 0 → emit status error + stderr 末尾若干行。
4. 前端：`translateStore` 订阅 `translate://progress` 事件，更新日志面板 + 进度条 + 状态。
5. 完成后前端显示"打开文件/打开文件夹"按钮（`tauri-plugin-opener`）。
6. 取消：`cancelTranslate(task_id)` → Rust 从 `tasks` 取 `ChildHandle` → `child.kill()` → emit status cancelled。

## 6. BabelDOC CLI 集成细节

### 6.1 参数构造（`args.rs`）
基础：
```
babeldoc --files <pdf> --output <dir> --lang-in <li> --lang-out <lo>
  --openai --openai-model <model> --openai-base-url <base_url> --openai-api-key <key>
  --qps <qps> --enhance-compatibility --auto-enable-ocr-workaround --report-interval 0.1
```
按 `output_mode`：`mono`→追加 `--no-dual`；`dual`→追加 `--no-mono`；`both`→都不加。
- 中文 Windows 环境给子进程设 `PYTHONUTF8=1`、`PYTHONIOENCODING=utf-8`，避免 stdout/stderr 编码乱码。
- `--working-dir` 指向 app cache 目录下的 `tasks/<task_id>/`，便于"保留中间文件"开关与清理。

### 6.2 进度解析（`progress.rs`）
BabelDOC CLI 默认 `use_rich_pbar=True` → rich Progress 渲染到 stderr，含 ANSI/`\r` 覆盖刷新。解析策略：
- 用 `tokio` 按行读 stderr，但 rich 用 `\r` 而非 `\n` 刷新 → 需要**按字节流**读、按 `\r` 与 `\n` 切分"逻辑行"。
- 剥 ANSI 转义序列（正则 `\x1b\[[0-9;]*[a-zA-Z]`）。
- rich 进度行格式形如 `stage (cur/total) ━━━━ 45%`；用正则 `(\\d+)%` 取百分比作为 `overall`；`(.+?)\\s*\\((\\d+)/(\\d+)\\)` 取 stage 与 part。
- 解析失败的行原样作为 `log` 事件推送（不丢日志）。
- **MVP 容错**：解析失败不影响翻译，进度条停留在上一次值；日志照常滚动。

> 备选（不在 MVP）：后续可改走 BabelDOC Python API `async_translate` 的事件 dict（干净），但需在 Rust 侧跑 Python（PyO3 或子进程跑 py 脚本输出 JSONL），复杂度高，MVP 不做。

### 6.3 安装探测（`get_babeldoc_info`）
- `which("babeldoc")` / `Command::new("babeldoc").arg("--version")`，5s 超时。
- 成功 → `{installed:true, version, path}`；失败 → `{installed:false, hint:"请先安装 Python 3.10–3.13 并运行 pip install BabelDOC"}`。
- 结果缓存 5min，避免每次翻译都探。

## 7. 安全设计

- **API Key**：keyring（Windows DPAPI），DB 只存 `api_key_id`。前端列表/导出只见 `has_api_key` 布尔。
- **reveal_api_key**：仅用户主动点眼睛图标触发，不进列表批量接口。
- **CSP**：`tauri.conf.json` 的 `app.security.csp` 设显式白名单（default-src 'self'; connect-src 允许供应商域名由 Rust 代理——MVP 前端不直接发跨域请求，所有 LLM 请求经 Rust `test_provider_connection`/`fetch_provider_models` 走，规避 CORS 且便于加超时/日志脱敏）。
- **日志脱敏**：Rust 推送 stderr 日志到前端前，对形如 `sk-...`/`--openai-api-key xxx` 的串做掩码（正则替换为 `sk-****`）。
- **子进程 Key 传递风险**：MVP 用 `--openai-api-key`，命令行对同机其他进程可见。文档/README 标注，后续迭代改 `--config <toml>`（600 权限、用完删）。`design` 里记录此已知债。
- **导出**：`export_config` 输出 JSON，`api_key` 字段恒为 `null`，`has_api_key` 保留。

## 8. 前端关键设计

- **状态**：Zustand 三 store——`providerStore`（列表+默认）、`translateStore`（文件列表+当前任务+日志+进度）、`settingsStore`。事件订阅放在 `translateStore` 的 `subscribeProgress()`（在 App 挂载时调一次 `listen('translate://progress')`）。
- **路由**：react-router-dom，`/translate` `/provider` `/params` `/tasks` `/settings`。
- **i18n**：i18next，`zh` 完整、`en` 键齐全（值可暂留中文或待翻）。`settingsStore.language` 联动 `i18n.changeLanguage`。
- **主题**：Antd ConfigProvider + `theme`（浅/深/跟随系统），`settingsStore.theme` 驱动。
- **拖拽**：Tauri 2 `getCurrentWebview().onDragDropEvent`（前端事件），不用 HTML5 DnD（Tauri 窗口原生拖拽更可靠）。`tauri.conf.json` 的 `fileDropEnabled` 默认 true。
- **API Key 输入**：antd `Input` + `type=password` + `addonAfter` 眼睛图标切换显隐（仿 ai-toolbox，不用 `Input.Password` 以便自定义眼睛行为与 reveal 流程）。

## 9. 依赖清单

### 前端（package.json）
`react` `react-dom` `react-router-dom` `antd` `@ant-design/icons` `zustand` `i18next` `react-i18next` `@tauri-apps/api` `@tauri-apps/plugin-dialog` `@tauri-apps/plugin-opener` `vite` `@vitejs/plugin-react` `typescript`。

### Rust（Cargo.toml）
`tauri = "2"` `tauri-plugin-dialog` `tauri-plugin-opener` `tauri-plugin-fs` `serde` `serde_json` `tokio` (full) `rusqlite` (features=["bundled"]) `keyring` `uuid` (features=["v4","serde"]) `reqwest` (features=["json","rustls-tls"], default-features=false) `which` `chrono` `tracing` `tracing-subscriber` `thiserror` `regex`。

> 不引入 `tauri-plugin-store`（不用它存 secret；普通设置走 SQLite）。不引入 stronghold（MVP 不需要主密码 UX）。

## 10. 兼容性 / 取舍 / 已知债

- **MVP 不内置 Python**：用户须自装 Python + BabelDOC。已知债，后续用 PyInstaller 打 sidecar exe 演进。
- **API Key 走命令行**：见 §7 已知债。
- **进度靠解析 stderr**：rich 格式变动可能导致进度解析失效；MVP 容错（解析失败只停进度不停日志），后续走 Python API 演进。
- **单文件翻译**：MVP `pdf_paths` 取第一个；批量留到后续。
- **任务历史**：MVP `task_record` 表仅记录当次及少量，不做完整任务管理页。
- **跨平台**：MVP 只验 Windows；Tauri 结构跨平台，但 keyring/babeldoc 探测在 macOS/Linux 的行为留待后续验证。
- **许可证**：PageWeave 须 AGPL-3.0；README 与 LICENSE 明示；依赖 BabelDOC（AGPL-3.0）。

## 11. 不做的事（明确排除）

- 不内嵌 Python（PyO3）/不打包 sidecar exe（MVP）。
- 不做云端/账号/支付/插件/自动更新/OCR 深度优化/DOCX/PPTX。
- 不做多供应商并发负载均衡（MVP 单默认供应商）。
- 不做术语表/上下文增强/缓存管理 UI（参数页占位）。
