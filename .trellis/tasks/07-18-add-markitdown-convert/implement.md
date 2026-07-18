# Implement: markitdown document → Markdown

## Preconditions

- [ ] User reviewed `prd.md` + `design.md`
- [ ] `task.py start` only after review approval
- [ ] Local Python 3.10–3.13 venv available for sidecar freeze
- [ ] BabelDOC path remains green（改动不碰 translate runner 逻辑除非抽 shared util）

## Ordered checklist

### Phase S — Sidecar

1. [ ] 添加 `sidecar/markitdown_entry.py`（freeze_support + CLI dispatch）
2. [ ] 添加 `sidecar/markitdown_sidecar.spec`（collect 必要依赖；one-folder）
3. [ ] 添加 `sidecar/build_markitdown_sidecar.sh`（PyInstaller + triple rename + `--version` smoke）
4. [ ] venv：`pip install 'markitdown[pdf,docx,pptx,xlsx,xls]==0.1.6' pyinstaller`
5. [ ] 构建并烟测：`--version`；对 5 类扩展名各转一份 fixture 到临时目录

### Phase R — Rust convert module

6. [ ] `src-tauri/src/convert/mod.rs` + `model` / `args` / `state` / `runner` / `commands`
7. [ ] 扩展名白名单 + 输出路径解析（stem + 时间戳防覆盖，本地时间）
8. [ ] `resolve_markitdown_sidecar` + `probe` / `start` / `cancel`
9. [ ] `ConvertRegistry` 单任务 busy 拒绝
10. [ ] `lib.rs` 注册 module 与 commands
11. [ ] （可选）抽取 `hide_child_console` / decode 到 shared util，translate 改引用时保持行为不变

### Phase P — Packaging

12. [ ] `tauri.conf.json` 增加 markitdown `externalBin` + `resources`
13. [ ] `.gitignore` 确认 `sidecar/dist/` 已忽略
14. [ ] 更新 `sidecar/README.md` 与根 `README.md`（构建步骤、双引擎）

### Phase F — Frontend

15. [ ] `types` + `convertApi` + `convertStore`
16. [ ] `features/convert/ConvertPage.tsx`（选文件、输出目录、开始/取消、日志、打开）
17. [ ] routes + App 侧栏 + i18n zh/en
18. [ ] 监听 `convert://progress`（在 App 或 ConvertPage 挂载处，避免与 translate 串线）

### Phase V — Validation

19. [ ] `pnpm tauri dev`：5 格式各成功一例；取消；非法扩展名；busy 二次 start
20. [ ] 与 translate 并存：一边翻译一边 convert（或 mock 长任务）
21. [ ] 剥离清单桌面走查：对照 design §7 列表确认无交叉引用

## Validation commands

```bash
# Sidecar
bash sidecar/build_markitdown_sidecar.sh /path/to/venv/python
sidecar/dist/markitdown-sidecar/markitdown-sidecar-x86_64-pc-windows-msvc.exe --version
# Manual convert smoke (example)
# ...exe path/to/sample.pdf -o /tmp/sample.md

# App
pnpm install
pnpm tauri dev
# optional full package
pnpm tauri build
```

## Risky files / rollback points

| Risk | Files | Rollback |
|------|-------|----------|
| tauri 双 externalBin 缺文件导致 build 失败 | `tauri.conf.json` | 暂时注释 markitdown 项 |
| 抽 shared util 回归翻译 | `translate/runner.rs` | 不抽 util，convert 复制 hide/decode |
| PyInstaller 漏 DLL/依赖 | `markitdown_sidecar.spec` | 按 warn 补 hiddenimports/collect_all |
| 事件名冲突 | frontend listeners | 严格 `convert://` 前缀 |

## implement.jsonl / check.jsonl

- 填入 backend/frontend quality + directory specs，及本任务 design/prd。
- 无独立 research 文件时以 design 为技术真相源；实现前若升 markitdown 版本，在 `research/` 记一笔。

## Exit criteria before claiming done

- PRD AC1–AC12 可勾选或注明延期理由
- 剥离清单完整
- 翻译回归：至少一次 PDF 翻译启动/取消仍正常
