# Design: markitdown document → Markdown

## 1. Architecture & boundaries

```
┌─────────────────┐     invoke      ┌──────────────────────┐     spawn      ┌─────────────────────────┐
│ React           │ ──────────────► │ Rust convert/*       │ ─────────────► │ markitdown-sidecar.exe  │
│ features/convert│ ◄── events ──── │ commands/runner/args │ ◄── stdout ─── │ (PyInstaller one-folder)│
└─────────────────┘  convert://*    └──────────────────────┘    stderr logs └─────────────────────────┘
         │                                      │
         │  open/reveal (reuse)                 │  no secrets / no provider
         ▼                                      ▼
   translateApi openers              independent ConvertRegistry
```

**Parallel to translate, not inside it.**

| Layer | Translate (existing) | Convert (new) |
|-------|----------------------|---------------|
| Sidecar | `babeldoc-sidecar` | `markitdown-sidecar` |
| Build | `sidecar/build_sidecar.sh` | `sidecar/build_markitdown_sidecar.sh` |
| Rust | `src-tauri/src/translate/` | `src-tauri/src/convert/` |
| Events | `translate://progress` | `convert://progress` |
| UI | `/translate` | `/convert` |
| Secrets | API Key via keyring/settings | **none** |

### Module layout

```
sidecar/
  markitdown_entry.py
  markitdown_sidecar.spec
  build_markitdown_sidecar.sh
  dist/markitdown-sidecar/          # gitignored, like babeldoc

src-tauri/src/convert/
  mod.rs
  model.rs          # ConvertRequest, ConvertEvent, ConvertInfo
  args.rs           # whitelist + CLI argv + output path resolve
  state.rs          # ConvertRegistry (single running task)
  runner.rs         # spawn, log drain, cancel, success path
  commands.rs       # start/cancel/probe (+ optional reuse open via translate cmds)

src/features/convert/
  ConvertPage.tsx
src/stores/convertStore.ts
src/services/api.ts                 # convertApi namespace
src/app/routes.tsx                  # /convert
src/App.tsx                         # sider menu item
src/i18n/locales/{zh,en}.ts
src/types/index.ts                  # Convert* types
```

### Optional thin shared util (only if DRY wins)

`src-tauri/src/process_util.rs`（或 `sidecar_util`）：`hide_child_console`、UTF-8 env、decode_process_output。  
**禁止** convert 调用 translate 内部 runner 逻辑；剥离时 process_util 可保留给 translate。

## 2. Sidecar contract

### Entry

`markitdown_entry.py` 薄封装：

- `multiprocessing.freeze_support()`
- 调用 `markitdown.__main__.main` 或等价 CLI，**保持与上游 CLI 一致**：
  - `markitdown <input> -o <output.md>`
  - `markitdown --version`
- 不启用 plugins；不传 Azure/LLM 参数。

### Dependencies (pinned for MVP)

```
markitdown[pdf,docx,pptx,xlsx,xls]==0.1.6
```

（实现期若 0.1.6 有阻断 bug 可小幅升补丁，写入 implement 记录。）

### Bundle shape

- PyInstaller **one-folder**（与 babeldoc 同理：启动快、路径稳定）。
- 输出：`sidecar/dist/markitdown-sidecar/markitdown-sidecar.exe`
- Tauri triple 副本：`markitdown-sidecar-x86_64-pc-windows-msvc.exe`
- `tauri.conf.json`：

```json
"externalBin": [
  "../sidecar/dist/babeldoc-sidecar/babeldoc-sidecar",
  "../sidecar/dist/markitdown-sidecar/markitdown-sidecar"
],
"resources": {
  "../sidecar/dist/babeldoc-sidecar/": "babeldoc-sidecar/",
  "../sidecar/dist/markitdown-sidecar/": "markitdown-sidecar/"
}
```

### Resolve order (`resolve_markitdown_sidecar`)

镜像 babeldoc：

1. resource dir `markitdown-sidecar/`
2. exe 同级
3. 同级 `sidecar/` 子目录
4. dev：`CARGO_MANIFEST_DIR/../sidecar/dist/markitdown-sidecar/...`

`probe`：运行 `--version`，解析 stdout；失败则 `installed: false` + 重装提示。

## 3. Rust contracts

### ConvertRequest (frontend → backend)

```ts
{
  input_path: string;   // absolute local path
  output_dir: string;   // absolute dir
}
```

### Output path resolution (R13)

1. Validate `input_path` extension ∈ `{pdf,docx,pptx,xlsx,xls}` (case-insensitive).
2. `stem = file_stem(input_path)`.
3. `candidate = output_dir / "{stem}.md"`.
4. If exists → `output_dir / "{stem}-{YYYYMMDD}-{HHMMSS}.md"` using **local time**（与用户文件资源管理器一致）。
5. Ensure `output_dir` exists（create_dir_all）或返回明确错误。

### CLI argv

```
[input_path, "-o", output_path]
```

不使用 stdin pipe 作为主路径（大文件与 Windows 编码更省事）。

### Events (`convert://progress`)

对齐 translate 形状，便于前端复用模式：

```ts
type ConvertEvent =
  | { type: "status"; task_id: string; status: "running"|"success"|"error"|"cancelled";
      output_file?: string; message?: string }
  | { type: "log"; task_id: string; line: string; stream: "stdout"|"stderr" };
```

无百分比字段（R14）。

### Commands

| Command | Behavior |
|---------|----------|
| `get_markitdown_info` | probe sidecar |
| `start_convert` | if registry busy → error；else spawn，return `task_id` |
| `cancel_convert` | kill via ConvertRegistry |

打开文件/文件夹：**复用**已有 `open_file_path` / `reveal_file_path`（属通用能力，不绑 convert）。

### Concurrency (D6 / R12)

- `ConvertRegistry`：最多 1 个 running entry（或 start 前 `is_busy()`）。
- **不**检查 translate `TaskRegistry`。
- translate 不检查 convert。

### Cancellation

与 translate 相同：`mpsc` cancel → `child.start_kill()` → wait；status `cancelled`。

### Env when spawning

```
PYTHONUTF8=1
PYTHONIOENCODING=utf-8
NO_COLOR=1
FORCE_COLOR=0
```

+ `CREATE_NO_WINDOW` on Windows。

### History / DB

MVP：**不**写入 translate 的 task_records 表（避免污染翻译历史与剥离耦合）。  
会话内状态由 `convertStore` 持有即可。

## 4. Frontend UX

- 路由 `/convert`，侧栏图标与文案 i18n（zh/en）。
- 页内容：
  - 拖拽/选择（dialog filter = 白名单扩展名）
  - 输出目录（默认 `settings.default_output_dir`，可改）
  - 开始 / 取消
  - 状态 + 日志面板（可简化复用 ProgressLogPanel 模式或轻量列表）
  - 成功：`output_file` + 打开文件 / 打开文件夹
- 页脚/说明：非排版级还原（R15）。
- 不展示 provider/API 相关 UI。

## 5. Security

- 仅本地绝对路径；拒绝 `http(s):` 等 scheme。
- 扩展名白名单前后端双重校验。
- 不把用户路径拼进 shell；`Command::new(exe).args([...])`。
- markitdown 自身可触网能力（requests）MVP 不暴露 URL 输入；仍应避免把任意 URI 传给 `convert()`。

## 6. Packaging & build ops

- `sidecar/dist/markitdown-sidecar/` 不提交 git（同 babeldoc）。
- 开发：先 `bash sidecar/build_markitdown_sidecar.sh [python]`。
- CI/release：在现有 sidecar 构建步骤旁增加 markitdown 构建（implement 阶段改 workflow 若存在）。
- 体积：显著小于 babeldoc；仍需在 README 注明双 sidecar。

## 7. Peel-off checklist（AC3）

删除或回滚以下项后，翻译应仍可用：

1. `sidecar/markitdown_entry.py`、`markitdown_sidecar.spec`、`build_markitdown_sidecar.sh`、`dist/markitdown-sidecar/`
2. `src-tauri/src/convert/` 与 `lib.rs` 中 `mod convert` + convert commands 注册
3. `tauri.conf.json` 中 markitdown 的 `externalBin` / `resources` 项
4. `src/features/convert/`、`convertStore`、`convertApi`、routes/menu/i18n/types 中 convert 字段
5. README / sidecar README 中 markitdown 章节

**保留**：`open_file_path`、`reveal_file_path`、settings 默认输出目录、可选 `process_util`。

## 8. Trade-offs

| Choice | Why | Cost |
|--------|-----|------|
| 独立 sidecar | 可剥离、依赖隔离 | 安装包多一块磁盘 |
| CLI 子进程 vs 内嵌 Python | 与现架构一致、崩溃隔离 | 启动冷启动成本（one-folder 可接受） |
| 无任务落库 | 模块薄 | 无跨会话 convert 历史 |
| 时间戳防覆盖 | 安全 | 用户目录可能多文件 |

## 9. Rollback

- 开发中：`git revert` convert 相关提交；去掉 tauri externalBin 第二项即可回到单 sidecar。
- 发布后：发不含 markitdown 资源的版本；用户侧无迁移数据（无 DB 表）。

## 10. Testing strategy

- Sidecar 烟测：`--version`；各格式一份 fixture → 非空 md。
- Rust：扩展名校验与重名路径纯函数单测（若易测）；否则手工清单。
- UI：手动走通选文件 → 成功 → 打开；取消；错误扩展名；busy 二次 start。
- 并存：translate running 时 start convert 成功（或反之）。
