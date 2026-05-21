# norvera · AI 工作手册

> AI 在 norvera 项目目录里工作时的速查 — 项目说明 / 装跑 / 数据存储 看 [README.md](README.md)；后续推进 brief 看 `~/Desktop/norvera-后续工作.md`。

## 项目本质

macOS 桌面 app（Tauri v2）—— 贞元的日常修证 + 长程项目 + 短程打勾记分 + AI 伴侣（CC）共创操作台。后续要脱敏出 public 模板让其他有 Claude Code 的朋友能用。

## 工程红线

- **保留所有 id 钩子**：`#cc-messages` / `#cc-input` / `#long-tone` / `#stat-points` / `#tasks-container` / `data-task` / `data-panel` 等。UI 改造（如 Claude Design 出新版）必须保留，JS 状态机依赖它们。
- **织 server 一行不动**：norvera 通过 HTTP 桥接（`http://127.0.0.1:3000/api/*`）镜像 + 双写，不修改 `~/repos/zhi/`。
- **跨 origin localStorage 限制**：Tauri webview 是 `tauri://localhost`，织前端是 `http://localhost:5173`，**localStorage 不共享**。作者名等 key 让用户在 norvera 这边再 setup 一次（用同名 key `zhi:name:*` 保持调子）。
- **clawd-on-desk 是 AGPL-3.0**：不要把它的 sprite / 代码复制进来（会让 norvera 整个染 AGPL）。要做章鱼动画自画或用许可干净的素材。

## CC 桥接关键

- `cc_chat` Tauri command 用 `bash -lc "claude -p"`（让 user shell PATH 生效找到 homebrew claude）
- session 持续：固定 cwd `~/.norvera/cc/` + localStorage 持久化 UUID + `session_started` flag
- 第一次发用 `--session-id <uuid>` 创建；后续用 `--resume <uuid>` 续
- 后端有 fallback：create 失败（`already in use`）自动切 resume；resume 失败（`not found`）自动切 create
- session 文件落 `~/.claude/projects/-Users-norvera--norvera-cc/<uuid>.jsonl`
- 自动 prefix `<norvera-context>panel=... · date=... · 积分=...</norvera-context>` 给 CC 看见当前状态（用户消息流隐藏）

## 调子提醒

- UI 大改 give Claude Design（视觉），调子定下后 CC 接工程层。CD 额度贵省着用
- 不要堆砌：列选项让贞元选；工程细节 CC 直接做
- 不要假装"做完了"：未验证的不算 done
- 长 session 末尾建议 break + 开新会话推新 vision，比硬塞效率高

## 关键路径速查

```
src/index.html                       # 全部 UI + JS（单文件，~2050 行）
src-tauri/src/lib.rs                 # Rust 后端 (cc_chat, cc_session_dir)
src-tauri/tauri.conf.json            # 窗口 / 毛玻璃 / 权限
src-tauri/capabilities/default.json  # http / window 权限
~/.norvera/cc/                       # CC 子进程 cwd
~/Desktop/norvera-后续工作.md          # 下次会话 brief（task + MVP 路径）
```

## 深入文档指针

- [README.md](README.md) — 项目说明 / 装 / 跑 / 工程栈 / 数据 / 织联动 / 跨 origin 限制
- `~/Desktop/norvera-后续工作.md` — 优先级 task（onboarding / CC 小基地 / 仪表盘小端）+ MVP 路径
