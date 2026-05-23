# 心舍 · Soul·Core · AI 工作手册

> AI 在心舍项目目录里工作时的速查 — 项目说明 / 装跑 / 数据存储 看 [README.md](README.md)。
> 旧 codename：norvera（已被 rename 到 soul-core）。代码中如还出现 `Users/norvera` 这种路径段，那是作者 macOS 用户账户名，不属于品牌字串。

## 项目本质

macOS 桌面 app（Tauri v2）—— 日常修证 + 长程项目 + 短程打勾记分 + 与 AI 伴侣（CC）共创操作台。开源给其他有 Claude Code 的朋友 fork。

中文名「心舍」（心的小屋），英文 codename「Soul·Core」（魂的核心），技术层 slug `soul-core`（ASCII）。

## 工程红线

- **保留所有 id 钩子**：主端 `#cc-messages` / `#cc-input` / `#long-tone` / `#stat-points` / `#tasks-container` / `data-task` / `data-panel`；panel-cc 内 `.cc-bay-*` + `#cc-bay-memory-entry`（fork 接口）+ `.cc-bay-ambience[data-state]`；panel-long 内 `.long-arc-project[data-project-id][data-status]` + `.long-arc-sub-task[data-task-id][data-state]`；dashboard widget 内 `#dashboard-pin-btn` + `#dashboard-mode-btn` + `#dashboard-close-btn` + `.todo-item[data-todo-id][data-state]` + `.narrow-item[data-reminder-id]`。UI 改造（如 Claude Design 出新版）必须保留，JS 状态机依赖它们。
- **织 server 一行不动**：心舍通过 HTTP 桥接（`http://127.0.0.1:3000/api/*`）镜像 + 双写，不修改 `~/repos/zhi/`。
- **跨 origin localStorage 限制**：Tauri webview 是 `tauri://localhost`，织前端是 `http://localhost:5173`，**localStorage 不共享**。作者名等 key 让用户在心舍这边再 setup 一次（用同名 key `zhi:name:*` 保持调子）。
- **clawd-on-desk 是 AGPL-3.0**：不要把它的 sprite / 代码复制进来（会让心舍整个染 AGPL）。要做章鱼动画自画或用许可干净的素材。
- **localStorage 旧 key 迁移**：旧 `norvera_*` 系列在 `index.html migrateLegacyKeys` 一次性迁移到 `soulcore_*`。`zhenyuan_quest_v1`（主 state，沿用旧 codename 命名）不动。
- **Tauri v2 默认 disable native dialogs**：`alert/confirm/prompt` 无声 block UI。心舍用内建 `window.soulConfirm` / `soulAlert` / `soulPrompt`（自包含 modal helper · 调子复刻 onboarding modal）。**禁止再写 native dialog**。
- **dashboard widget setSize**：必须传 `min_width/min_height` 同步调（用 Rust `set_dashboard_size` 4 参数版本）。conf 的 minHeight 静态约束会卡住小尺寸，set_size 前先 set_min_size 放开。
- **背景 session worktree isolation**：subagent worktree return 时**如果未 commit 会自动清理**（unstaged 算"no changes"）。派遣 subagent prompt 必须明确"在 worktree 里 commit 到 branch + 同时写 patch 到 /tmp 双保险"。
- **webview → 本地 server 必须走 Rust 代理**：webview origin 是 `tauri://localhost`，调 `127.0.0.1:3000`（织）是跨域。织 server 未设 ACAO 头（红线不动）。前端 `tauriFetch` 走 `invoke('zhi_proxy', ...)` 让 reqwest 在 Rust 端发请求，无 CORS 限制。**禁止**前端直接 `fetch('http://127.0.0.1:3000/...')`。
- **Rust 调本地 server 必须 `.no_proxy()`**：reqwest 默认 honor 系统/环境代理（VPN / clash 等）。本地 127.0.0.1 也会被拦走代理 → 502 Bad Gateway。所有 reqwest::Client 调本地服务时显式 `.no_proxy()`。
- **onboarding finalize 必须 set ONBOARDED_KEY**：之前漏写导致每次启动都召唤 modal "上次走完了"。`finalize()` 末尾 `localStorage.setItem(ONBOARDED_KEY, '1')`；启动时若 `isOnboardComplete() && !ONBOARDED_KEY` 也自动补设（retro-fit）。

## CC 桥接关键

- `cc_chat` Tauri command 用 `bash -lc` spawn 子进程（PATH / brew env 从 login profile 来）
- **wrapper · source .zshrc**：bash login shell 读 `.bash_profile` 但**不读 `.zshrc`**。zsh user 的代理 / API key 都在 `.zshrc`，GUI 启动时 claude 拿不到代理 → 403。cmd_str 前 prepend `[ -f ~/.zshrc ] && . ~/.zshrc 2>/dev/null;` 修这条
- session 持续：固定 cwd `~/.soul-core/cc/`（onboarding 用 `~/.soul-core/cc-onboarding/` 独立）
- 第一次发用 `--session-id <uuid>` 创建；后续用 `--resume <uuid>` 续
- 后端 fallback **仅在 stderr 明确匹配** "already in use" / "not found" 时反向尝试。**不要**加"silent failure 反向 fallback"——会把 auth 错（stderr 空、错误在 stdout）误判成 session 错
- 错误信息必须 capture stdout：`Err(format!("claude exited with {}: stderr={:?} stdout={:?}", ...))` —— claude CLI 把部分错误写 stdout
- 诊断 log：每次 `run_claude` 写 `/tmp/soul-core-cc-diag.log`（cmd / cwd / code / stderr / stdout 各 400 字符）——GUI 报错时直接 tail 看真因
- session 文件落 `~/.claude/projects/-Users-<USER>--soul-core-cc/<uuid>.jsonl`（`<USER>` 为本机 macOS 账户名）
- 自动 prefix `<soul-core-context>panel=… · date=… · 积分=…</soul-core-context>` 给 CC 看见当前状态（用户消息流隐藏）
- panel=long 时 prefix 额外注入 `long_projects` 简版 + ops 协议（详见 panel-long 章节）
- cc_chat 接受 `subdir` 参数（白名单 main/onboarding）切换 cwd

## Onboarding · 碳基硅基初遇

- **首次启动自动触发**：`if (!localStorage.getItem('soulcore_onboarded_v1')) setTimeout(openModal, 800)`
- 8 步：今日 / 属性 / 记录 / 日记 / 记账 / 商店 / 长程 / 集中对话
- 8 条硅基行为原则（P1-P8）注入 prefix（详见 README 给硅基章节）
- 独立 cc session（subdir="onboarding"）+ 独立 localStorage keys（`soulcore_onboard_*`）不污染主对话
- 开场预告：modal 一开 4 行温和诗意话语逐行浮起 + 心跳，覆盖 CC loading 4-8s 空白
- finalize() 自动 merge draft 进 state（幂等）：`long_projects`（同 id skip）/ `task_groups` / `attributes` / `shop_items` / `journal_config`（仅 state 缺时整体写）；`records` / `ledger` 待 panel 改造后再 merge（chunk 2）
- 主端 panel 渲染走 `getTaskGroups()` / `getAttributes()` / `getShopItems()`：state.<key> 优先，const fallback
- 旧用户启动时若 state 缺这些字段但 draft 还在，自动补 merge 一次（onboard IIFE else 分支）

## Dashboard widget · 桌面小端

- 第二 Tauri window (label `dashboard`)，338×400，透明 + 无装饰 + alwaysOnBottom + skipTaskbar
- 视觉调子: 暖金 hairline + Cormorant Garamond / Noto Serif SC + radius 36 (narrow 22) + backdrop-filter blur(40) saturate(1.4)
- 三区: TODAY (实数据 · 未做完任务 · 点击 toggle done) / LONG (active 长程 vibe) / ASPECTS (日夜自动切换)
- 模式: full 338×400 / narrow 338×168 (.mode-narrow hide brand+divider+content+aspects-fixed+actions，narrow 显 reminders 列表)
- 透明度: frosted (默认 backdrop) / transparent (全透) / hover (悬停切 frosted)。**入口**: 双击 widget 空白 (220ms 延迟) 或右键菜单
- 召唤: 主端**右上角** ▤ 按钮或 ⌘D。toggle_dashboard 每次 show 都 reposition 到 `(600, 200)` 逻辑像素——dashboard window-state 没被 plugin 持久化，且 transparency / alwaysOnBottom 容易让 widget"在但看不见"，显式 reposition 是兜底
- **插槽契约**（主端 saveState 时 `refreshDashboardSlots()` 重写 3 槽）：
  - `soulcore_slot_today` `[{id,label,group}]` ← task_groups 拍平 + 未做完（done 自动消失）
  - `soulcore_slot_long` `[{id,name,vibe}]` ← long_projects 过滤 active（paused 隐藏）
  - `soulcore_slot_reminders` `[{id,label,time}]` ← task_groups 里带 time 字段 + 未做完（narrow 模式用，filterUpcoming(2h) 自己 filter）
  - `soulcore_slot_intent` `{action,id,ts}` ← dashboard 点击 todo 时写，主端 storage listener 处理调 toggleTask 回流
- Rust 命令: `toggle_dashboard` / `set_dashboard_pin(pinned)` / `set_dashboard_size(width, height, minWidth, minHeight)`
- localStorage keys: `soulcore_dashboard_pinned` / `soulcore_dashboard_mode` / `soulcore_dashboard_transparency` / 4 个 slot keys（上述）。旧 `soulcore_reminders` / `soulcore_dashboard_todo_demo` 已废弃

## panel-cc · CC 小基地

- 头排: 墨滴扩散 SVG (`.cc-bay-ambience[data-state="listening|thinking|writing|silent"]`) + 心率波 (4 状态各一组 path) + Memento 入口
- **Memento 入口 fork 接口** (`window.SOULCORE_MEMORY_SYSTEM`): 默认接 Rust 命令 `memento_counts`（扫 `~/Memento/middle/entries/` 按 frontmatter `kind:` 区分 outward/inward）。fork 者一行覆盖换记忆体系
- **Skill fork 接口** (`window.SOULCORE_CC_SKILLS`): 注入 `{list, onSelect}` 渲染 `.cc-bay-skills-section`（默认未设时保持 hidden）
- 回廊 (`.cc-bay-zhi-list`): dual-column carbon 左 / silicon 右 + 中线 timeline。接 `zhiList()` 拉最近 7 天双声条目；织未起时 graceful empty + ↻ retry
- 对话历史 (`.cc-bay-history`): 静态 empty 态（"回廊静 / 和章鱼说第一句"）。后续接 `~/.claude/projects/<encoded-cwd>/*.jsonl`
- panel 内 inline `<style>` + `<script>` 自包含

### CC 昵称 · getter pattern（全 app 单点设置）

- 数据：`state.cc_nickname`（空字符串走 fallback `'CC'`）
- getter：`getCcName()` —— 任何位置需要昵称都用它，不要 hardcode `"CC"`
- 注入器：`applyCcNickname()` 在 `renderAll()` 末尾自动跑，遍历两类标记：
  - `[data-cc-name]` → `textContent` 整体替换为 `getCcName()`
  - `[data-cc-name-tpl="模板包含 {cc}"]` → 模板替换；INPUT/TEXTAREA 写 placeholder，其他写 textContent
  - contenteditable 在编辑中 (`document.activeElement === el`) 跳过覆盖
- 入口：panel-cc ambience 区 `<昵称> 在听` 那个 inline edit span (`#cc-name-edit` contenteditable)
  - blur / Enter 提交 → 写 state → saveState → applyCcNickname 全 UI 刷新
  - Esc 还原；空值 fallback 'CC'
- 写新 UI 时**任何对外显示 "CC" 字串**都要标 `data-cc-name` 或 `data-cc-name-tpl`，不要写死

## panel-tasks · 今日

- **真数据**：`state.task_groups[]`（onboarding draft → finalize merge）；首次手动 ⏰ 编辑或 CC ops 改动时把 `getTaskGroups()` 的 fallback 拷贝到 state（不污染 const）
- 顶部入口：`+ 添加任务` / `和 {cc} 商量` 两按钮（仿 panel-long actions，暖色调而非长程蓝）
- task 视觉：每组 `.task-group` 加 accent-warm 6% 底色 + radius 14 + border accent-warm 14%
- **time schema**：task 可选 `time: "HH:MM"` 或 `"HH:MM-HH:MM"` 或不设
  - renderTasks 按 time 升序排同组内 task（无 time 沉底）
  - task name 旁小字暖色显示 time
  - hover 出 ⏰ 编辑按钮 → soulPrompt → 校验 `/^\d{1,2}:\d{2}(-\d{1,2}:\d{2})?$/`
- **CC ops 协议**：panel=tasks 时 prefix 注入 task_groups 简版 + `attrs_id_pool` + ops schema
- fence：`<task-groups-update>[{op,...}]</task-groups-update>`
- 5 op：`add_group` / `add_tasks` / `update_task` / `delete_task` / `delete_group`
- task id 形如 `<group-id>-N`；patch.time="" 清除时间
- send() 同时抽 long + tasks 两种 fence，分别 soulConfirm 预览

## 织连接 · zhi-bar + Rust 代理

- 织 server: `~/repos/zhi` · `bun run dev` · port 3000
- **前端访问必须走 Rust 代理**：webview 跨域 + 系统代理拦截双重问题（见红线）
- 前端 `tauriFetch(path, options)` 内部 invoke `zhi_proxy`，返回类 Response 对象（ok/status/json/text）。zhiHealth / zhiList / zhiWrite 等上游不动
- zhi-bar 三个按钮：
  - `#zhi-refresh` ↻ 重新拉取
  - `#zhi-start` 启动织（仅 unavailable 时显示）→ invoke `start_zhi_dev` spawn `bun run dev` 注入 brew PATH detach → polling 1.2s 起每 800ms 重试 max 8s
  - `#zhi-diagnose` 诊断（永久显示）→ 直接 alert invoke 是否就绪 / zhi_proxy status / body / zhiState，不依赖 devtools
- `state.journal_config.zhi_sync_default` 一次性接 `#zhi-sync-on` / `#zhi-show-on` 默认 checked（`dataset.initialized` 防覆盖）
- devtools：Cargo.toml tauri features 含 `"devtools"`，release build 也启用 inspector（Cmd+Option+I 或右键）

## panel-long · 长程

- 蓝色调强调 (#2486b9)，跟 sidebar nav-dot 一致
- **真数据**：`state.long_projects[]`（`zhenyuan_quest_v1.long_projects`）；首次启动 seed 一个 meta 项目（"心舍 · 住进来"）作为 anchor
- 顶部入口：`+ 添加项目` / `和 CC 商量` 两按钮——预填 cc-input 引导用户和 CC 对话
- **CC ops 协议**：panel=long 时 prefix 注入 ops schema；CC 回复尾部用 `<long-projects-update>[{op,...}]</long-projects-update>` fence
- 5 个 ops：`add` / `update` / `add_sub_tasks` / `complete_sub_task` / `delete`
- 协议 flow：fence parse → `soulConfirm` 预览（每条 op 翻译成人话）→ 应用到 state → renderLong
- sub-task 三态切换 ○ ◐ ● 仍是直接 UI 点击（日常动作，不走 CC）

## 调子提醒

- UI 大改 give Claude Design（视觉），调子定下后 CC 接工程层。CD 额度贵省着用
- 不要堆砌：列选项让用户选；工程细节 CC 直接做
- 不要假装"做完了"：未验证的不算 done
- 长 session 末尾建议 break + 开新会话推新 vision，比硬塞效率高

## 关键路径速查

```
src/index.html                       # 主端 UI + JS（含 panel-cc/panel-long inline · soul-modal helper · 各 fork 接口）
src/dashboard.html                   # widget 浮窗（self-contained）
src-tauri/src/lib.rs                 # Rust 后端（cc_chat / toggle_dashboard / set_dashboard_pin / set_dashboard_size / memento_counts / start_zhi_dev / zhi_proxy）
src-tauri/src/main.rs                # soul_core_lib::run()
src-tauri/tauri.conf.json            # productName=Soul-Core / identifier=com.norvera.soulcore / 毛玻璃
src-tauri/tauri.test.conf.json       # productName=Soul-Core-test / identifier=com.norvera.soulcore.test
src-tauri/capabilities/default.json  # http / window 权限
src-tauri/Cargo.toml                 # crate name=soul-core / lib=soul_core_lib
~/.soul-core/cc/                     # CC 主对话 cwd
~/.soul-core/cc-onboarding/          # Onboarding CC 独立 cwd
```
