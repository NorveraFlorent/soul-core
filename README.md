# 心舍 · Soul·Core

一个 macOS 桌面 app —— 日常修证 + 长程项目 + 短程打勾记分 + 与 AI 伴侣（CC）共创的综合操作台。

> 心舍 = 心的小屋；Soul·Core = 魂的核心。
> 旧 codename：norvera（Nova 新星 + Vera 真实）。

## 这是什么

- **大端**（当前）：方形 macOS 窗口（1120×840 起步），三栏布局 + 毛玻璃融入桌面
  - **今日 · 任务**：日常节奏 / 工作 / 修行 / 扣分项，打勾记分
  - **属性**：身 / 清明 / 智识 / 凝视 / 轻盈，5 个修证维度
  - **记录**：晚餐 / 月相 / 近 30 日历史
  - **日记 · 记账 · 商店**：积分兑奖励
  - **长程**：商业策划这类长项目的调子 + 子任务（占位）
  - **CC 桥接**：底部章鱼伴侣对话区，调本地 `claude` CLI（用户 ~/.claude/ 全配置）
  - **碳基硅基初遇**（onboarding）：首次启动 modal 8 步对话，CC 半开放引导 + 完全开放集中对话凝练用户气质
- **小端**（后续）：长条便签独立窗口，常驻桌面打勾

## 调子

- 五种调色板可切换（玄金 / 青瓷 / 朱玄 / 月白 / 纯心）—— 切换 `<html data-palette="…">`
- 两种吉祥物可切换（章鱼 octopus / 麻薯 mochi）—— 切换 `<html data-mascot="…">`
- 八房撞色：每个 panel 自动换调，无需手动配置

## 装

下载 `release/Soul-Core_*.dmg`，双击装入 Applications。Finder / Dock 显示名为 `Soul-Core`，UI 内一律显示「心舍」。

## 跑（开发模式）

```bash
git clone git@github.com:NorveraFlorent/soul-core.git
cd soul-core
bun install
bun run tauri dev
```

依赖：Rust 1.85+ / Node 20+（或 Bun 1.3+）/ Xcode Command Line Tools。

测试包（独立 webview localStorage / 不污染主 app）：

```bash
bun run tauri build -c src-tauri/tauri.test.conf.json
# 出 Soul-Core-test.app
```

## 工程栈

- **Tauri v2**（Rust 壳 + WKWebView）
- **Vanilla HTML / CSS / JS**（无构建步骤，单文件 src/index.html）
- **localStorage 持久化**（数据全在用户端，不上云）
- **macOS hudWindow 毛玻璃**（NSVisualEffectView vibrancy）+ Overlay traffic light + 无边框窗口

## 数据

所有状态（任务进度 / 积分 / 日记 / 记账 / 商店）存浏览器 `localStorage`，键 `zhenyuan_quest_v1`（沿用作者旧 codename 的主 state key，未来可能改）。Onboarding 用独立 keys `soulcore_onboard_*`。可在大端底部 **导出存档** 备份 JSON。

旧 `norvera_*` 系列 keys（旧 codename 时期）在首次启动时自动迁移到 `soulcore_*`（参见 `index.html migrateLegacyKeys`）。

## CC 桥接关键

- `cc_chat` Tauri command 用 `bash -lc "claude -p"`（让 user shell PATH 生效找到 homebrew claude）
- session 持续：固定 cwd `~/.soul-core/cc/`（onboarding 用 `~/.soul-core/cc-onboarding/` 独立）
- 第一次发用 `--session-id <uuid>` 创建，后续 `--resume <uuid>` 续，有 fallback
- session 文件落 `~/.claude/projects/-Users-<USER>--soul-core-cc/<uuid>.jsonl`（`<USER>` 为本机 macOS 账户名）
- 自动 prefix `<soul-core-context>panel=… · date=… · 积分=…</soul-core-context>` 给 CC 看见当前状态

## 织（Zhi）联动（可选）

日记 panel 通过 HTTP 桥接共写到[织](https://github.com/zhenyuan/zhi)（共写日记 app，跑在 `127.0.0.1:3000`）。**织一行代码不动**，心舍只是织的另一个视图入口。需要这块功能时，需要单独 clone 织并启动其 dev server（见下方"织 server 需要在跑"）；不启动也能用心舍其它功能，日记 panel 会 graceful fallback 到本地写入。

- **双写**：写日记时本地 + 织都存一份，本地记录 `zhiId` 防止重复迁移
- **合并展示**：渲染时把本地日记 + 织里所有条目（含 silicon 的批注）按日期分组展示
- **气泡云日历**：日期作为浮动气泡，条目越多气泡越大，hover 时晃动
- **迁移**：把现有本地日记一次性同步到织（zhi-bar 的「迁移本地日记 →」按钮）
- **graceful fallback**：织未起时 zhi-bar 显「织 · 未起 ✕」+ 双写/显示/迁移控件 disable + ↻ 重连恢复

### 作者名（跨 origin 限制）

织的作者显示名存在它前端的 `localStorage`（key 命名空间 `zhi:name:carbon` / `zhi:name:silicon`）。心舍是 Tauri webview（origin `tauri://localhost`），织前端是 web app（origin `http://localhost:5173` 或 `:3000`），**浏览器 localStorage 是 origin-scoped 的，心舍没法直接 fetch 织前端的设置**。

所以心舍提供独立的「作者名」按钮（zhi-bar 右侧），在心舍这边再设一次（用同样的 key 命名空间）。默认 fallback 是 `Carbon` / `Silicon`。

### 织 server 需要在跑

```bash
cd ~/repos/zhi && bun run dev
```

跑了才能联动。zhi-bar 顶部的 "织 · 在线（N 条）" / "织 · 未起 ✕" 显示当前状态，点 ↻ 可重试。

## 设计 / 工程分工

- UI 视觉：[Claude Design](https://claude.ai)（三栏 / 八房撞色 / 五调色板 / 章鱼麻薯吉祥物）
- 工程包装：Project author × Claude Code（Tauri 配置 / 毛玻璃融入 / 拖动 / GitHub / 织桥接 / CC 桥接 / 心舍 onboarding）
- 状态机 JS：Project author × Claude Code
- 现有 license：私有项目 / 私有 repo
