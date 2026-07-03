# ai-toolbox 调研报告

> 调研对象：`coulsontl/ai-toolbox` (GitHub)
> 调研目的：参考其 AI API 配置管理界面与 Tauri+React 桌面架构，为新 Windows 本地 PDF 翻译工具的配置管理模块提供设计借鉴。
> 调研时间：2026-07-01

## 一、项目概览

| 维度 | 内容 |
|---|---|
| 定位 | "个人 AI 工具箱"——跨平台桌面应用，一站式管理多种 AI 编程助手的供应商与模型配置、MCP 服务器、Skills、Prompt |
| 平台 | Windows / macOS / Linux（Windows 上还支持同步配置到 WSL） |
| 桌面框架 | Tauri 2.x（`tauri` crate 2.9.5） |
| 前端 | React 19.2 + TypeScript 5.9 |
| UI 库 | Ant Design 6.4（antd）+ `@ant-design/icons`、`lucide-react`、`@lobehub/icons` |
| 状态管理 | Zustand 5.0（`web/stores/` 下 7 个 store 文件） |
| 构建工具 | Vite 7.3 + `@vitejs/plugin-react` |
| 包管理 | pnpm 9.15 |
| i18n | i18next 25.7 + react-i18next（中/英） |
| 路由 | react-router-dom 7.12 |
| 数据库 | SQLite（rusqlite 0.39，bundled）——已从早期的 SurrealDB 迁移到 SQLite |
| 拖拽 | @dnd-kit/core + sortable（用于供应商排序、列表重排） |
| 活跃度 | 高——962 stars / 71 forks，707 commits，90 个 release，最新 v0.9.9（2026-06-21） |

## 二、许可证

**MIT License**——可自由参考、复用、改造。对"借鉴设计但不抄代码"目标完全没有法律障碍。

## 三、AI API 配置管理设计

### 1. 供应商数据模型（JSON-blob 表模式）

表只有 4 个真列 + JSON blob 存业务字段：
```sql
CREATE TABLE claude_provider (
  id TEXT PRIMARY KEY,
  data BLOB NOT NULL CHECK (json_valid(data,4)),
  created_at TEXT,
  updated_at TEXT
);
-- 仅为 is_applied、sort_index 建 json_extract 索引
```

业务字段定义在 Rust 结构体（以 ClaudeCodeProviderRecord 为例）：`id`、`name`、`category`（official/custom）、`settings_config`（JSON 串，装 baseUrl/apiKey/apiFormat/models）、`extra_settings_config`、`source_provider_id`、`website_url`/`notes`/`icon`/`icon_color`、`sort_index`、`meta`、`is_applied`、`is_disabled`、`created_at`/`updated_at`。

### 2. 前端表单字段
- `category`（Radio：official / custom）
- `name`、`baseUrl`
- `apiKey`（用普通 `Input` + `type=password` 手动切换显隐，`addonAfter` 放眼睛图标）
- `apiFormat`（Select：anthropic / openai_chat / openai_responses / gemini_native）
- 模型映射网格 + 兜底 model
- 高级设置：JSON 文本编辑器
- `notes`

三种模式：manual（手填）/ import（从其它来源导入，字段禁用）/ official（隐藏 baseUrl/apiKey/apiFormat）。

### 3. 预设供应商与模型列表
- 后端 `preset_models.rs`，命令 `fetch_remote_preset_models` / `load_cached_preset_models`——远程拉取并缓存预设模型清单。
- "Fetch Models" 按钮调用 Tauri command `fetch_provider_models`，下拉可切换 'native' / 'openai_compat' 两种拉取方式。
- "Quick Set Models" 一键把第一个模型填到所有角色位。

### 4. 测试连接 / 拉取模型
- 共享组件 `web/features/coding/shared/providerConnectivity/`：`ProviderConnectivityTestModal`（单供应商测试弹窗）、`ProviderConnectivityStatus`（状态徽标）、`batchTest`（批量并发测试）。
- 独立命令 `test_provider_model_connectivity`、`fetch_provider_models`、`get_provider_models`。

### 5. 导入导出
- 单供应商级：`ImportFromAllApiHubModal`、`ImportConflictDialog`（冲突处理）。
- 数据库级：`backup_database` / `restore_database`（本地 ZIP），`backup_to_webdav` / `restore_from_webdav` / `list_webdav_backups`（WebDAV 云备份）。
- Skills 库存级：先 preview 再 apply 的两阶段导入。

### 6. 默认供应商 / 模型的设计思路
- 不用布尔 `is_default`，而是用 **`is_applied`（当前生效）+ `sort_index`（排序）**。
- `select_claude_provider` / `apply_claude_config` 命令把选中项写入目标 CLI 的真实配置文件。
- `toggle_claude_code_provider_disabled` 控制启用/禁用；`reorder_claude_providers` 调整排序。
- 通用配置 `ClaudeCommonConfig`（全局兜底）与单供应商配置（逐条覆盖）分离。

## 四、API Key 安全存储方式

### ai-toolbox 实际用的方式
**关键发现：ai-toolbox 没有用任何加密存储。** `Cargo.toml` 里没有 `keyring`、没有 `tauri-plugin-stronghold`。API Key 作为普通字符串，序列化进 `settings_config` JSON blob，**明文存在 SQLite 数据库文件里**。备份出来的 ZIP / WebDAV 包也是明文。这是它的设计取舍：本地优先、便利优先。**对新项目不建议照搬**——存有金额敞口的 Key 明文落盘风险偏高。

### Tauri 2.x 当前可用的 API Key 存储方案对比

| 方案 | 加密方式 | Windows 表现 | 是否要主密码 | 打包影响 | 适合存 API Key |
|---|---|---|---|---|---|
| `keyring` crate（社区） | OS 原生——Windows 走 Credential Manager + DPAPI | 原生支持，无需额外运行时 | 否 | 几乎为零 | **首选** |
| tauri-plugin-stronghold（Tauri 官方） | IOTA Stronghold 加密 vault 文件 | 完全支持 | 是（vault 需密码解锁） | 二进制体积增大 | 适合需要文件级强加密的场景 |
| tauri-plugin-store（Tauri 官方） | 无加密——明文 JSON 文件 | 全平台 | 否 | 轻量 | **禁用于 secret** |

## 五、Tauri+React 架构要点

### 1. 目录结构
```
ai-toolbox/
├── web/                       # React 前端（不是 src/）
│   ├── app/                   # 路由 / 全局 layout
│   ├── features/              # 按业务域分模块（feature-driven）
│   │   ├── coding/claudecode/ # 每个 CLI 一个子目录：pages/components/utils/index.ts
│   │   └── shared/            # 跨域复用：providerConfig/providerConnectivity/...
│   ├── services/              # 23 个 *Api.ts 文件——Tauri invoke 封装层
│   ├── stores/                # Zustand stores
│   └── components/ constants/ types/ utils/ i18n/ assets/
├── tauri/                     # Rust 后端（不是 src-tauri/）
│   ├── Cargo.toml  tauri.conf.json  build.rs
│   ├── capabilities/          # Tauri 2 权限/capability 配置
│   └── src/
│       ├── main.rs lib.rs     # generate_handler! 注册 300+ command
│       ├── db.rs  db/         # SQLite schema/migrations/helpers/backup
│       └── coding/            # 按 CLI 工具分模块（与前端 features/coding 对称）
```

### 2. Rust 后端 command 组织
- `lib.rs` 的 `.setup()` 里：开 SQLite、`app.manage(db_state)` 注入状态。
- 插件链：`single_instance` + `opener` + `os` + `dialog` + `fs` + `shell` + `updater`。**没挂** stronghold/store/sql 插件——数据库直接用 `rusqlite` crate 自己管。
- `generate_handler!` 注册超 300 个 command，按模块路径命名（如 `coding::claude_code::list_claude_providers`）。
- 命名约定一致：`list_*` / `create_*` / `update_*` / `delete_*` / `reorder_*` / `select_*` / `apply_*` / `toggle_*_disabled` / `get_*_common_config` / `save_*_common_config`。

### 3. 前后端通信
- **invoke 为主**：前端 `web/services/*.ts` 把每个 Tauri command 包成 async 函数，组件只调 service 不直接 `invoke`。
- **event 用于推送**：数据库变更通过事件通知前端（`refreshStore.ts` 负责刷新）。
- Tauri 2 的 capability/权限在 `tauri/capabilities/` 里按域配置。

### 4. 窗口与托盘
- `tauri.conf.json` 里 `"windows": []`——窗口在代码里动态创建（`WebviewWindowBuilder`）。
- `tray.rs` 实现系统托盘菜单；`single_instance.rs` 防多开；`auto_launch.rs` 开机自启。
- `bundle.createUpdaterArtifacts=true` + `plugins.updater` 指向 GitHub releases 的 `latest.json`，签名校验用 minisign。
- **注意**：`tauri.conf.json` 里 `app.security.csp = null`——CSP 关闭，新项目应开 CSP。

## 六、可复用部分（借鉴设计，非抄代码）

1. **JSON-blob 表 + 4 真列** 的数据库模式：schema 演进极灵活（加字段不改表），适合配置管理这种字段会频繁变的场景。
2. **per-CLI 模块对称结构**：前后端都按"工具域"切分，新增一个翻译引擎就照抄一套，互不污染。
3. **shared/ 跨域复用层**：把供应商表单、连通性测试、导入冲突处理等抽到 `shared/`，所有模块复用。
4. **services 分层**：`web/services/*Api.ts` 统一封装 `invoke`，组件不直接 `invoke`。
5. **供应商表单交互模式**：category Radio、API Key 用 Input+眼睛切换、模型用 AutoComplete + Fetch Models 按钮、高级设置用 JSON 文本编辑器、备注用 Collapse 折叠。
6. **生效机制**：`is_applied` + `sort_index` 而非 `is_default`；"选中→apply→写入真实配置文件"两步走。
7. **测试连接**：单供应商 TestModal + 批量 batchTest + 状态徽标三件套。
8. **导入导出**："先 preview 再 apply"的两阶段导入能避免脏数据覆盖。
9. **托盘 + 单实例 + 自启**三件套。

## 七、对本项目的借鉴建议

### 配置管理模块设计

**数据模型（SQLite + JSON blob 模式）**：
```sql
CREATE TABLE translation_provider (
  id TEXT PRIMARY KEY,
  data BLOB NOT NULL CHECK (json_valid(data,4)),
  created_at TEXT, updated_at TEXT
);
CREATE INDEX idx_tp_applied ON translation_provider(json_extract(data,'$.is_applied'));
CREATE INDEX idx_tp_sort    ON translation_provider(json_extract(data,'$.sort_index'));
```

`data` JSON 字段：`name`、`category`、`base_url`、`api_format`、`api_key_id`（引用 keyring 条目，不存明文）、`models`、`default_model`、`prompt_template`、`extra_settings`、`icon`/`icon_color`、`notes`、`sort_index`、`is_applied`、`is_disabled`。

**Rust 后端 command**：`list_providers` / `create_provider` / `update_provider` / `delete_provider` / `reorder_providers` / `select_provider` / `toggle_provider_disabled` / `test_provider_connectivity` / `fetch_provider_models` / `get_common_config` / `save_common_config` / `export_config` / `import_config`。

**前端结构（feature-driven）**：
```
web/features/translation/
  ├── pages/         # 供应商管理页
  ├── components/    # ProviderFormModal / ProviderCard / ImportConflictDialog
  └── index.ts
web/features/shared/
  ├── providerConfig/
  ├── providerConnectivity/  # TestModal + batchTest + Status
  └── languagePair/
web/services/translationApi.ts
web/stores/translationStore.ts   # Zustand
```

### API Key 存储

**不要照搬 ai-toolbox 的明文 JSON。** 推荐方案：
1. **首选：`keyring` crate（OS 原生凭据库）**——Windows 上走 Credential Manager + DPAPI，无需主密码，UX 无摩擦。供应商表里的 `api_key_id` 字段只存引用 ID，真实 Key 走 `keyring::Entry::new("MyApp", &api_key_id)` 存取。打包无额外负担。
2. **次选：`tauri-plugin-stronghold`**——仅当产品定位需要"vault 文件可导出/迁移 + 强文件加密"时才用。代价：用户要设主密码。
3. **绝对不用**：`tauri-plugin-store` 存 Key（明文 JSON）。

**配套安全措施**：
- `tauri.conf.json` 里 `app.security.csp` 设成显式白名单（不要学 ai-toolbox 关掉 CSP）。
- 日志里对 Key 做掩码（`sk-...xxxx`）。
- 备份导出时给 Key 单独选项"导出时包含 Key 是/否"，默认否；包含时给备份包加密码。

**推荐组合**：SQLite（存配置元数据 + `api_key_id` 引用）+ `keyring` crate（存真实 Key）+ Zustand（前端状态）+ antd ProviderFormModal 交互。
