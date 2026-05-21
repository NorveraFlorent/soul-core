# 心舍 · Soul·Core · AI 工作手册

> AI 在心舍项目目录里工作时的速查 — 项目说明 / 装跑 / 数据存储 看 [README.md](README.md)；后续推进 brief 看 `~/Desktop/心舍-后续工作.md`。
> 旧 codename：norvera（已被 rename 到 soul-core，仅 macOS user home 路径段 `Users/norvera` 是用户账户名保留）。

## 项目本质

macOS 桌面 app（Tauri v2）—— 贞元的日常修证 + 长程项目 + 短程打勾记分 + 与 AI 伴侣（CC）共创操作台。后续要脱敏出 public 模板让其他有 Claude Code 的朋友能用。

中文名「心舍」（心的小屋），英文 codename「Soul·Core」（魂的核心），技术层 slug `soul-core`（ASCII）。

## 工程红线

- **保留所有 id 钩子**：`#cc-messages` / `#cc-input` / `#long-tone` / `#stat-points` / `#tasks-container` / `data-task` / `data-panel` 等。UI 改造（如 Claude Design 出新版）必须保留，JS 状态机依赖它们。
- **织 server 一行不动**：心舍通过 HTTP 桥接（`http://127.0.0.1:3000/api/*`）镜像 + 双写，不修改 `~/repos/zhi/`。
- **跨 origin localStorage 限制**：Tauri webview 是 `tauri://localhost`，织前端是 `http://localhost:5173`，**localStorage 不共享**。作者名等 key 让用户在心舍这边再 setup 一次（用同名 key `zhi:name:*` 保持调子）。
- **clawd-on-desk 是 AGPL-3.0**：不要把它的 sprite / 代码复制进来（会让心舍整个染 AGPL）。要做章鱼动画自画或用许可干净的素材。
- **localStorage 旧 key 迁移**：旧 `norvera_*` 系列在 `index.html migrateLegacyKeys` 一次性迁移到 `soulcore_*`。`zhenyuan_quest_v1`（贞元个人主 state）不动。
- **Tauri v2 默认 disable native dialogs**：`alert/confirm/prompt` 无声 block UI，要么自定义 modal UI，要么 capabilities 加 dialog permission。

## CC 桥接关键

- `cc_chat` Tauri command 用 `bash -lc "claude -p"`（让 user shell PATH 生效找到 homebrew claude）
- session 持续：固定 cwd `~/.soul-core/cc/`（onboarding 用 `~/.soul-core/cc-onboarding/` 独立）
- 第一次发用 `--session-id <uuid>` 创建；后续用 `--resume <uuid>` 续
- 后端有 fallback：create 失败（`already in use`）自动切 resume；resume 失败（`not found`）自动切 create
- session 文件落 `~/.claude/projects/-Users-norvera--soul-core-cc/<uuid>.jsonl`
- 自动 prefix `<soul-core-context>panel=… · date=… · 积分=…</soul-core-context>` 给 CC 看见当前状态（用户消息流隐藏）
- cc_chat 接受 `subdir` 参数（白名单 main/onboarding）切换 cwd

## Onboarding · 碳基硅基初遇

- 8 步：今日 / 属性 / 记录 / 日记 / 记账 / 商店 / 长程 / 集中对话
- 8 条硅基行为原则（P1-P8）注入 prefix：详见 `~/Desktop/心舍-onboarding-schema.md`（v3）
- 独立 cc session + 独立 localStorage keys（`soulcore_onboard_*`）不污染主对话
- 开场预告：modal 一开 4 行温和诗意话语逐行浮起 + 心跳，覆盖 CC loading 4-8s 空白
- ⊕ debug 触发器在右下角（chunk C 阶段，正式发布前会改成首次启动自动）

## 调子提醒

- UI 大改 give Claude Design（视觉），调子定下后 CC 接工程层。CD 额度贵省着用
- 不要堆砌：列选项让贞元选；工程细节 CC 直接做
- 不要假装"做完了"：未验证的不算 done
- 长 session 末尾建议 break + 开新会话推新 vision，比硬塞效率高

## 关键路径速查

```
src/index.html                       # 全部 UI + JS（单文件，~3400 行）
src-tauri/src/lib.rs                 # Rust 后端 (cc_chat, cc_session_dir, soul_core_cc_cwd)
src-tauri/src/main.rs                # soul_core_lib::run()
src-tauri/tauri.conf.json            # productName=Soul-Core / identifier=com.norvera.soulcore / 毛玻璃
src-tauri/tauri.test.conf.json       # productName=Soul-Core-test / identifier=com.norvera.soulcore.test
src-tauri/capabilities/default.json  # http / window 权限
src-tauri/Cargo.toml                 # crate name=soul-core / lib=soul_core_lib
~/.soul-core/cc/                     # CC 主对话 cwd
~/.soul-core/cc-onboarding/          # Onboarding CC 独立 cwd
~/Desktop/心舍-后续工作.md             # 下次会话 brief（task + MVP 路径）
~/Desktop/心舍-onboarding-schema.md   # onboarding 设计 schema v3
```

## 深入文档指针

- [README.md](README.md) — 项目说明 / 装 / 跑 / 工程栈 / 数据 / 织联动 / 跨 origin 限制
- `~/Desktop/心舍-后续工作.md` — 优先级 task（onboarding / CC 小基地 / 仪表盘小端）+ MVP 路径
- `~/Desktop/心舍-onboarding-schema.md` — onboarding v3 schema（P1-P8 + 8 步引导路径 + prefix prompt）
