# PageWeave

Windows 本地 PDF 翻译桌面工具。拖入 PDF → 选源/目标语言 → 选 AI 服务商 → 调用 [BabelDOC](https://github.com/funstory-ai/BabelDOC) 翻译 → 输出尽量保留原排版的译文 PDF。API Key 只存本地、加密保存。

## 技术栈

- **桌面**：Tauri 2.x + React 19 + TypeScript + Ant Design + Zustand + i18next + Vite
- **后端**：Rust（SQLite via rusqlite，API Key via `keyring` crate = Windows Credential Manager + DPAPI）
- **翻译引擎**：BabelDOC，以**内置 sidecar**（PyInstaller 冻结的独立 exe）形式随应用分发——**用户无需安装 Python**，开箱即用，且已预置 DocLayout-YOLO 模型与字体（首次运行无需联网下载）

## 开发

```bash
pnpm install
pnpm tauri dev     # 启动开发模式（需 Rust 工具链）
```

Rust 工具链：`rustup` + stable + MSVC build tools。Node ≥ 20，pnpm ≥ 9。

## 构建 sidecar（免 Python 的内置翻译引擎）

sidecar 不随 `tauri dev` 自动构建，需单独执行一次：

```bash
bash sidecar/build_sidecar.sh [/path/to/venv/python]
# 默认用 /tmp/bd_venv/Scripts/python.exe（一个已 pip install BabelDOC 的 venv）
```

脚本会：
1. `pip install pyinstaller`（若缺）
2. `babeldoc --warmup` 预下载模型/字体
3. PyInstaller 冻结成 `sidecar/dist/babeldoc-sidecar/`（one-folder，含 `_internal/`）
4. 补拷 hyperscan 的 vendor-hashed MSVC 运行时（PyInstaller 漏检）
5. `--generate-offline-assets` 生成离线资源包 + `--restore-offline-assets` 还原进 `~/.cache/babeldoc`
6. 复制 `babeldoc-sidecar-x86_64-pc-windows-msvc.exe` 供 Tauri `externalBin` 识别
7. 烟测 `--version`

开发期还需把 `sidecar/dist/babeldoc-sidecar/_internal/` 拷到 `src-tauri/target/debug/_internal/`，让 dev 模式下的 `babeldoc-sidecar.exe` 能找到 Python 运行时（Rust 的 `resolve_sidecar()` 会在 exe 同级 + `sidecar/` 子目录里查找）。

正式 `tauri build` 时通过 `tauri.conf.json` 的 `externalBin` 把 sidecar 纳入安装包。

## 打包分发

```bash
pnpm tauri build
```

`tauri.conf.json` 的 `bundle.externalBin` 指向 sidecar exe，Tauri 会按目标 triple 重命名并纳入安装包。安装后用户无需任何前置依赖。

### 应用更新签名

PageWeave 使用 Tauri updater 从 `jhxxr/PageWeave` GitHub Releases 检查更新。发布时必须签名更新包：

```powershell
pnpm tauri signer generate -w updater.key
```

- `updater.key` 是私钥，不要提交到 Git。
- `updater.key.pub` 是公钥备份，也默认不提交；需要把公钥内容写入 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`。
- 在 GitHub 仓库 `Settings -> Secrets and variables -> Actions` 添加 `TAURI_SIGNING_PRIVATE_KEY`，值为 `updater.key` 文件内容。
- 如果生成密钥时设置了密码，再添加 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。
- CI 发布后会上传 Tauri updater 使用的 `latest.json`。

## 安全

- API Key 经 `keyring` 存入 Windows Credential Manager（DPAPI 加密），SQLite 只存 `api_key_id` 引用
- 导出配置默认不含 API Key
- 日志对 `sk-...` 类令牌做掩码
- CSP 显式白名单

## 许可证

AGPL-3.0-or-later。本项目集成 BabelDOC（AGPL-3.0），依其条款须整体开源。

## 已知债 / 后续

- API Key 经 `--openai-api-key` 命令行传给 sidecar（同机其他进程可见进程命令行）。后续改 `--config` toml（权限 600、用完删）。
- 进度靠解析 rich/tqdm 的 stderr ANSI 输出（正则取百分比）。后续可走 BabelDOC Python API `async_translate` 的事件 dict，更干净。
- 仅单文件翻译；批量、任务历史、术语表、OCR 深度等高级参数页为占位，后续迭代开放。
