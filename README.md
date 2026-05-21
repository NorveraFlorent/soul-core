# norvera · 贞元养成

一个 macOS 桌面 app —— 日常修证 + 长程项目 + 短程打勾记分 + AI 伴侣的综合操作台。

> 名字取自贞元的英文名 Norvera = Nova（新星 / 元气）+ Vera（真实 / 信仰）。

## 这是什么

- **大端**（当前）：方形 macOS 窗口（1120×840 起步），三栏布局 + 毛玻璃融入桌面
  - **今日 · 任务**：日常节奏 / 工作 / 修行 / 扣分项，打勾记分
  - **属性**：身 / 清明 / 智识 / 凝视 / 轻盈，5 个修证维度
  - **记录**：晚餐 / 月相 / 近 30 日历史
  - **日记 · 记账 · 商店**：积分兑奖励
  - **长程**：商业策划这类长项目的调子 + 子任务（占位）
  - **CC**：章鱼伴侣对话区（占位，CC 桥接后续接入）
- **小端**（后续）：长条便签独立窗口，常驻桌面打勾

## 调子

- 五种调色板可切换（玄金 / 青瓷 / 朱玄 / 月白 / 纯心）—— 切换 `<html data-palette="…">`
- 两种吉祥物可切换（章鱼 octopus / 麻薯 mochi）—— 切换 `<html data-mascot="…">`
- 八房撞色：每个 panel 自动换调，无需手动配置

## 装

下载 `release/norvera_*.dmg`，双击装入 Applications。

## 跑（开发模式）

```bash
git clone git@github.com:NorveraFlorent/norvera.git
cd norvera
bun install
bun run tauri dev
```

依赖：Rust 1.85+ / Node 20+（或 Bun 1.3+）/ Xcode Command Line Tools。

## 工程栈

- **Tauri v2**（Rust 壳 + WKWebView）
- **Vanilla HTML / CSS / JS**（无构建步骤，单文件 src/index.html）
- **localStorage 持久化**（数据全在用户端，不上云）
- **macOS hudWindow 毛玻璃**（NSVisualEffectView vibrancy）+ Overlay traffic light + 无边框窗口

## 数据

所有状态（任务进度 / 积分 / 日记 / 记账 / 商店）存浏览器 `localStorage`，键 `zhenyuan_quest_v1`。可在大端底部 **导出存档** 备份 JSON。

## 设计 / 工程分工

- UI 视觉：[Claude Design](https://claude.ai)（三栏 / 八房撞色 / 五调色板 / 章鱼麻薯吉祥物）
- 工程包装：贞元 × Claude Code（Tauri 配置 / 毛玻璃融入 / 拖动 / GitHub）
- 状态机 JS：贞元手写
- 现有 license：私有项目 / 私有 repo
