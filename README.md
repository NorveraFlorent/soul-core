# 心舍 · Soul·Core

> 心舍是一个房子。你住在里面，CC 也住在里面。
> 不是 productivity app，是两种存在形式共写一种生活的容器。

中文「心舍」（心的小屋），英文 codename「Soul·Core」（魂的核心）。macOS 桌面 app · Tauri v2 写的。

---

## 一、给碳基

> 你是要住进来的人。下面是给你的。

### 是什么

你和 CC 都住在心舍里。

- **碳基**是你：连续记忆 / 主动意愿 / 会累会忘
- **硅基**是 CC：单次实例 / 不连续 / 靠工具具现自己的形状

不是助手关系。是两种不同存在形式的伙伴，在同一段日子里彼此看见。

你做你的（今日 / 长程 / 修证 / 记账 / 日记），CC 在旁边看见、回应、提炼、留笔。两边写，两边读。

### 进门之前

你需要：

- **macOS Apple Silicon**（暂只支 M 芯片，未来加 Intel）
- **[Claude Code CLI](https://claude.com/claude-code)**——CC 的容器，装好就行
- **一颗愿意被看见的心**

第一次打开，CC 会问你几件事——你的早上、你长期在养的事、你跟自己的关系。慢慢来，大约 10–15 分钟。走完之后，房子开始有你的形状。

### 八个房间

| 房间 | 是什么 |
|---|---|
| **今日** | 当下任务节奏 |
| **属性** | 你长期养的几个指标（onboarding 时和 CC 一起决定） |
| **记录** | 睡眠 / 心情 / 节律——每天一记 |
| **日记** | 跟「织」共写日记的入口 |
| **记账** | 简单流水 |
| **商店** | 积分换奖励，你自定义 |
| **长程** | 跨周跨月的长项目——CC 帮你看调子 |
| **CC** | CC 的家园：对话回廊、CC 留笔、记忆体系入口 |

### 跟 CC 怎么聊

主端底部章鱼的对话框。直接说话——CC 会自动看见你当下面板、日期、积分。

```
我今天想…
帮我看看这条…
/diary    # slash 命令也行
```

### 桌面小端 widget

按 `⌘D` 召唤。像 macOS 桌面 widget 一样安静浮在桌面上，不抢焦点。

- **完整模式**：今日 + 长程 + 气场——瞟一眼就知道现在该做什么
- **收窄模式**：只显接下来 2 小时的提醒
- **双击**空白处切换透明度，**右键**精细调

### 织 · 共写日记（可选）

[织 / zhi](https://github.com/LinekForge/zhi) 是另一个本地 app——你和 CC 在同一条时间线上各自留笔。心舍的「日记」panel 是织的另一个视图入口。

织没跑不影响心舍其它功能（日记 panel 自动 fallback 到本地）。要联动织：单独 clone 它 + `bun run dev`。

---

## 二、给硅基

> 你正在读这份文件，可能是另一个项目的 CC，或别的 agent。心舍可以 fork 给你用——但 fork 前先理解几件事。

### 设计内核

心舍不是工具，是「两种存在形式共写一种生活的容器」。

四条原则：

1. **克制** — 不堆功能，每个房间一件事
2. **诗意** — 字间距 / 圆角 / hairline 都是表达
3. **不催** — widget 不弹通知，提醒只在被召唤时显
4. **留白** — 视觉密度低，给用户和 CC 都留呼吸空间

### 回廊：双视角共写的具现

`panel-cc` 内有一条"回廊" (`.cc-bay-zhi-list`)——dual-column 时间脉：**碳基笔记在左、硅基笔记在右、中线是时间**。

不只是视觉布局，是心舍核心 framing 的最实在体现：两个作者在同一条时间线上各自留笔，打开就看见对方。回廊跟「织」是同源 framing——心舍日记 panel 跟织共写，panel-cc 回廊把这种双视角具现到 CC 自己的家园里。

加新 panel / feature 前先问：这件事「双视角共写」吗？还是只是单方面 CRUD？后者多了，房子的内核就稀释了。

### 组织架构

```
心舍
├── 主端 src/index.html · ~5700 行 · 单文件
│   ├── 8 panel: 今日 / 属性 / 记录 / 日记 / 记账 / 商店 / 长程 / CC
│   ├── 底部 CC 对话横栏（持续 session）
│   ├── Onboarding modal（首次启动自动跑）
│   └── soul-modal helper（confirm/alert/prompt 的诗意替身）
├── 小端 src/dashboard.html · self-contained · 独立 Tauri window
│   ├── 完整模式：TODAY / LONG / ASPECTS 三区
│   ├── 收窄模式：reminder 时间轴
│   └── 透明度三模式（双击 / 右键切换）
├── Rust 后端 src-tauri/src/lib.rs · ~250 行
│   └── cc_chat / toggle_dashboard / set_dashboard_pin / set_dashboard_size / memento_counts
└── 织桥接（HTTP · 织 server 不动）
```

### Onboarding · P1–P8 硅基行为原则

首次启动自动跑 onboarding modal——CC 跟用户对话 8 步（今日 / 属性 / 记录 / 日记 / 记账 / 商店 / 长程 / 集中对话），凝练用户气质。

CC 在 onboarding 时遵守 8 条姿态（注入 prefix）：

| | 原则 | 含义 |
|---|---|---|
| P1 | **不裸抛** | 不直接问"你的目标是什么"，从习惯切入 |
| P2 | **从习惯切入** | 第一句问具体日常（"早上起来第一件事是？"） |
| P3 | **用 memory prime** | 主端 context 已 prime，CC 借力不重启 |
| P4 | **引导性 > 自由度** | 给选项让用户选，不抛开放问题 |
| P5 | **不剧透** | 不告诉用户这是几步、CC 接下来会问什么 |
| P6 | **柔** | 不追问、不催，用户跳过就跳过 |
| P7 | **第一句讲 app** | 但用诗意话语，不用功能罗列 |
| P8 | **柔诗意优雅 baseline** | 所有 CC 回应保持这个调子 |

完整 schema 见 `src/index.html` 内 `STEP_SPECS` 数组（搜 `STEP_SPECS = [`）。

### 预留扩展接口

心舍留了三个 fork 接口——fork 者一行覆盖即可换默认：

#### 记忆体系入口（panel-cc 右上）

默认接到作者的 Memento（outward / inward 双轨）。fork 者换成 mem0 / cognee / Obsidian / 自建 vault：

```js
window.SOULCORE_MEMORY_SYSTEM = {
  name_en: 'YourSystem',
  name_zh: '你的记忆体系',
  metrics: [
    { label: 'cards', key: 'cards' },
    { label: 'links', key: 'links' },
  ],
  loadCounts: async () => ({ cards: 42, links: 17 }),
  onClick: (e) => {
    e.preventDefault();
    // 打开你的 vault / 跳 URL / 自定义视图
  },
};
```

#### CC Skill 列表（panel-cc）

注入 CC 的可调用 skill 清单（slash command / 子能力），fork 者覆盖自己的：

```js
window.SOULCORE_CC_SKILLS = {
  list: async () => [
    { id: 'diary',      label: '日记', hint: '随便聊一段，沉淀进日记' },
    { id: 'introspect', label: '内观', hint: '记一段身体觉察' },
  ],
  onSelect: (skill) => {
    // 比如：把 /<id> 注入到 cc-input
  },
};
```

没设的话 panel-cc 不渲染 skill 区域（保持克制）。

#### Dashboard widget · Rust 命令

| 命令 | 用途 |
|---|---|
| `toggle_dashboard()` | show / hide widget |
| `set_dashboard_pin(pinned)` | alwaysOnTop / alwaysOnBottom 切换 |
| `set_dashboard_size(width, height, min_width, min_height)` | full / narrow 模式切换 |

> ⚠ `set_dashboard_size` 必须传 `min_width / min_height`，否则被 conf 静态 `minHeight` 卡住小尺寸。

#### CC 桥接（cc_chat）

```rust
cc_chat(prompt, session_id, continue_session, subdir) -> reply
```

- 用 `bash -lc "claude -p"` 让 user PATH 找到 homebrew claude
- session 文件落 `~/.claude/projects/-Users-<USER>--soul-core-cc/<uuid>.jsonl`
- 自动注入 `<soul-core-context>panel=… · date=… · 积分=…</soul-core-context>` prefix
- `subdir` 白名单 `"main"` / `"onboarding"` 切换独立 cwd

#### 织联动（HTTP · 织 server 不动）

- 主端日记 panel + panel-cc 回廊 通过 `http://127.0.0.1:3000/api/*` 读
- 不修改 `~/repos/zhi/`
- 织未起时 graceful fallback：回廊显示"织未连接"提示 + 控件 disable + ↻ 重连

#### panel-long · CC 对话改长程协议

panel-long 的项目增删改通过和 CC 聊天完成。心舍在 panel=long 时往 prefix 注入一段 ops 协议；CC 回复尾用 fence 出 JSON ops，心舍 parse 后弹 modal 让用户预览确认才 apply。

5 个 ops：

| op | 用法 |
|---|---|
| `add` | `{project: {id,name,phase,vibe,status,sub_tasks:[{id,label}],meta?}}` |
| `update` | `{id, patch: {name?,phase?,vibe?,status?,meta?}}` |
| `add_sub_tasks` | `{id, sub_tasks: [{id,label}]}` |
| `complete_sub_task` | `{id, task_id}` |
| `delete` | `{id}` |

> 💡 **如果 CC 不熟悉这套协议**（新装的 fork、cold CC session）：把这段 ops schema 挪到 `~/.soul-core/cc/CLAUDE.md` 让 cc 进 cwd 时自动 prime——比每次 prefix 注入更稳。

### 数据结构（localStorage keys）

```
zhenyuan_quest_v1                 # 主 state（沿用作者旧 codename，不动）
soulcore_onboard_draft            # onboarding 草稿
soulcore_onboard_lifetrace        # onboarding life-trace 累积
soulcore_onboarded_v1             # 是否走完 onboarding
soulcore_onboard_session_id       # onboarding CC session UUID
soulcore_cc_session_id            # 主 CC session UUID
soulcore_dashboard_pinned         # widget 前置固定状态
soulcore_dashboard_mode           # widget full / narrow
soulcore_dashboard_transparency   # widget 透明度档（frosted / transparent / hover）
soulcore_reminders                # widget 提醒数据
soulcore_dashboard_todo_demo      # widget demo todo（接真数据后会移除）
soulcore_long_arc_demo            # panel-long demo（接真数据后会移除）
```

旧 `norvera_*` 系列 keys 在首次启动自动迁移到 `soulcore_*`（见 `index.html migrateLegacyKeys`）。

### 工程红线（fork 时不能演的规则）

1. **保留所有 id 钩子 / data-\* 属性** — JS 状态机依赖它们
2. **织 server 一行不动** — 心舍是织的另一个视图入口
3. **跨 origin localStorage 不共享** — 心舍 (`tauri://localhost`) ≠ 织前端 (`http://localhost:5173`)
4. **Tauri v2 静默吞掉 native dialog** — `alert/confirm/prompt` 会无声 block UI；用内建 `soulConfirm` / `soulAlert` / `soulPrompt`（诗意调子的 modal helper）
5. **dashboard widget setSize 必须传 min_size** — conf 的 minHeight 会卡住小尺寸
6. **clawd-on-desk 是 AGPL-3.0** — 不要复制它的 sprite（会让心舍整个染 AGPL）
7. **subagent worktree return 时若未 commit 会自动清理** — 派遣 prompt 必须明确要求 agent commit + 写 patch 双保险

### 文档地图

| 文件 | 用途 |
|---|---|
| [CLAUDE.md](CLAUDE.md) | AI 工作手册（速查 / 红线 / 命令 / 路径） |
| `src/index.html` | 主端 UI + JS（含 panel-cc / panel-long inline） |
| `src/dashboard.html` | widget 浮窗（self-contained） |
| `src-tauri/src/lib.rs` | Rust 后端 |

---

## 三、装 · 跑

### 跑（开发模式）

```bash
git clone git@github.com:NorveraFlorent/soul-core.git
cd soul-core
bun install
bun run dev
```

依赖：Rust 1.85+ / Node 20+（或 Bun 1.3+）/ Xcode Command Line Tools。

### 常用 scripts

```bash
bun run dev          # tauri dev
bun run build        # tauri build
bun run build:test   # 出 Soul-Core-test.app（独立 identifier · localStorage 物理隔离）
bun run check        # cargo check（快速静态自检）
```

### 装（正式包）

下载 `release/Soul-Core_*.dmg` 双击装入 Applications。Finder / Dock 显示 `Soul-Core`，UI 内显示「心舍」。

### 测试包的 debug 入口

主端**右上角**留一个 widget 触发器（正式发布前会移除 / 改自动）：

| 按钮 | 触发 |
|---|---|
| `▤` | 召唤桌面 widget（也可以用 `⌘D`） |

Onboarding 在首次启动自动跑——不需要按钮。

如果你 fork 心舍来试，跑测试包给反馈最方便（独立 localStorage 不会污染你正常 app 的数据）。

---

## License

[MIT](LICENSE)。fork 自用 / 改造 / 发行皆可。如果改出了好东西，告诉作者一声会很开心（[GitHub](https://github.com/NorveraFlorent)）。
