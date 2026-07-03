# PageWeave sidecar —— 内置 BabelDOC 翻译引擎（免 Python）

PageWeave 通过 PyInstaller 把 BabelDOC（Python 库）冻结成一个独立的 `babeldoc-sidecar.exe`，作为 Tauri 的外部二进制随应用分发。这样**终端用户无需安装 Python 或任何依赖**，应用自带完整的 PDF 翻译能力。

## 文件

- `babeldoc_entry.py` — sidecar 入口。薄封装 `babeldoc.main:cli`，保留与 `babeldoc` CLI 完全一致的参数语义，供 Rust runner 直接调用。
- `babeldoc_sidecar.spec` — PyInstaller spec。用 `collect_all` 把 babeldoc 全部 165 个子模块 + 155 个数据文件 + 各重型依赖（onnxruntime / cv2 / pymupdf / scipy / scikit-image / scikit-learn / rtree / freetype / uharfbuzz / pyzstd / Levenshtein / rapidfuzz / bitstring / tiktoken 等）的原生二进制一并打入。
- `build_sidecar.sh` — 一键构建脚本。

## 为什么是 one-folder 而非 onefile

onefile 每次启动都把 ~200MB 解压到临时目录，慢且会破坏 BabelDOC 的缓存目录假设。one-folder 直接运行，启动快、缓存稳定。

## 关键坑（已解决）

1. **PyInstaller spec 的 `SPECPATH` 是 spec 所在目录（`sidecar/`），不是项目根**。spec 里 `ROOT = Path(SPECPATH).resolve().parent` 才能正确定位 `sidecar/babeldoc_entry.py`。
2. **`bitstring` 的子模块 `bitstring.bitstore_bitarray` 被动态导入，PyInstaller 默认漏检**。spec 用 `collect_all("bitstring")` 兜底。
3. **hyperscan 的 `_hs_ext.pyd` 依赖一个 vendor-hashed 的 `msvcp140-<hash>.dll`，它住在 `hyperscan.libs/` 而非 pyd 旁边，PyInstaller 不收集**。`build_sidecar.sh` 在冻结后手动把该 DLL 拷进 `_internal/`，否则 sidecar 启动报 `DLL load failed while importing _hs_ext`。hyperscan 在 BabelDOC 中仅 `glossary.py` 用到（术语表多模式正则匹配），但 import 链上必须能加载。
4. **dev 模式下 `babeldoc-sidecar.exe` 必须能找到同级的 `_internal/` 目录**（含 `python311.dll` 等）。dev 时需手动把 `sidecar/dist/babeldoc-sidecar/_internal/` 拷到 `src-tauri/target/debug/_internal/`；`tauri build` 时由 `externalBin` 配置统一处理。

## 离线资源（模型 + 字体）

BabelDOC 首次翻译需下载 DocLayout-YOLO ONNX 模型（~72MB）与字体（~254MB）。`build_sidecar.sh` 在冻结后运行 `--generate-offline-assets` 打包、再 `--restore-offline-assets` 还原进 `~/.cache/babeldoc`。

离线资源包不进入 Git 仓库。CI 发布时会把 `sidecar/assets/offline_assets_*.zip` 作为可选 GitHub Release 附件上传。应用设置页提供两种安装方式：

1. 在线安装：从 `jhxxr/PageWeave` 最新 Release 中查找并下载 `offline_assets_*.zip`。
2. 本地安装：用户提前下载该 zip 后，在设置页选择本地文件安装。

未安装离线资源时，应用仍可运行；BabelDOC 首次翻译可能按自身逻辑联网下载资源。

## Rust 侧调用

`src-tauri/src/translate/runner.rs` 的 `resolve_sidecar()` 按以下顺序查找 sidecar exe：
1. 当前 exe 同级目录（`babeldoc-sidecar.exe` / `babeldoc-sidecar`）—— dev 的 `target/debug/` 与正式安装的 app 目录都满足
2. 同级 `sidecar/` 子目录
3. `CARGO_MANIFEST_DIR/../sidecar/dist/babeldoc-sidecar/babeldoc-sidecar.exe`（dev 便利）

`probe_babeldoc()` 同样优先用 sidecar，回退到 PATH 上的 `babeldoc`，再回退 `python -m babeldoc`。

## 构建

见 `build_sidecar.sh` 顶部注释。前置：一个已 `pip install BabelDOC` 的 Python 3.10–3.13 venv。脚本会自检并补装 PyInstaller。

## 版本管理策略

- `sidecar/build/` 是 PyInstaller `--workpath` 中间产物，只用于本地构建缓存，不提交。
- `sidecar/dist/` 是 `build_sidecar.sh` 生成的 one-folder sidecar 输出，也不提交；发布/打包前在本机或 CI 重新生成。
- Tauri `bundle.externalBin` 指向 `../sidecar/dist/babeldoc-sidecar/babeldoc-sidecar`，构建时会自动查找 `babeldoc-sidecar-<target-triple>.exe`。Windows 下脚本会生成 `babeldoc-sidecar-x86_64-pc-windows-msvc.exe`。
- `sidecar/assets/offline_assets_*.zip` 是离线模型/字体资源包；文件超过 GitHub 普通 Git blob 限制，不提交到仓库，只作为 Release 附件分发。
