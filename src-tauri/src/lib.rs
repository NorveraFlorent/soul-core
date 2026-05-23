use tokio::io::AsyncWriteExt;
use tokio::process::Command;

// 心舍 / Soul·Core 内 CC 对话 cwd
// 默认 (main) -> ~/.soul-core/cc/                  （主对话，panel-cc）
// onboarding -> ~/.soul-core/cc-onboarding/        （初遇引导，独立 session 不污染主）
fn soul_core_cc_cwd(subdir: &str) -> Result<std::path::PathBuf, String> {
    // 白名单防注入 / 防误传
    let leaf = match subdir {
        "main" | "" => "cc",
        "onboarding" => "cc-onboarding",
        _ => return Err(format!("invalid cc subdir: {}", subdir)),
    };
    let home = std::env::var("HOME").map_err(|e| format!("no HOME: {}", e))?;
    let dir = std::path::PathBuf::from(home).join(".soul-core").join(leaf);
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {}", e))?;
    Ok(dir)
}

// 校验 session_id 是不是合法 UUID（防止 shell injection）
fn is_uuid(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 36 { return false; }
    bytes.iter().enumerate().all(|(i, &b)| match i {
        8 | 13 | 18 | 23 => b == b'-',
        _ => b.is_ascii_hexdigit(),
    })
}

// 诊断 log · 写 /tmp/soul-core-cc-diag.log，每次 cc_chat 留痕
fn log_diag(msg: &str) {
    use std::io::Write;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/soul-core-cc-diag.log")
    {
        let _ = writeln!(f, "[t={}] {}", ts, msg);
    }
}

// 在 cwd 下用 bash -lc 跑一次 claude，返回 (stdout, stderr, exit_code)
async fn run_claude(
    cmd_str: &str,
    prompt: &str,
    cwd: &std::path::Path,
) -> Result<(String, String, i32), String> {
    // GUI 启动时 launchctl env 不读 ~/.zshrc，导致代理 / API key / claude auth 相关 env 缺失。
    // /bin/bash -lc 是 login bash——读 .bash_profile 但不读 .zshrc。
    // 用户默认 shell 多是 zsh，关键 env（proxy / brew shellenv / etc）在 .zshrc 里
    // ——所以包一层 source，让 child env 拿到。POSIX export 语句 bash 完全能解析。
    let wrapped_cmd = format!(
        "[ -f \"$HOME/.zshrc\" ] && . \"$HOME/.zshrc\" 2>/dev/null; {}",
        cmd_str
    );
    log_diag(&format!("--- run_claude START cwd={:?} cmd={:?} prompt_len={}", cwd, cmd_str, prompt.len()));

    let mut child = Command::new("/bin/bash")
        .current_dir(cwd)
        .arg("-lc")
        .arg(&wrapped_cmd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            log_diag(&format!("spawn claude failed: {}", e));
            format!("spawn claude failed: {}", e)
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .await
            .map_err(|e| {
                log_diag(&format!("write stdin failed: {}", e));
                format!("write stdin failed: {}", e)
            })?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| {
            log_diag(&format!("wait failed: {}", e));
            format!("wait failed: {}", e)
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    log_diag(&format!(
        "--- run_claude END code={} stderr_len={} stdout_len={} stderr={:?} stdout_head={:?}",
        code,
        stderr.len(),
        stdout.len(),
        if stderr.len() > 400 { &stderr[..400] } else { stderr.as_str() },
        if stdout.len() > 400 { &stdout[..400] } else { stdout.as_str() }
    ));

    Ok((stdout, stderr, code))
}

// 向用户本地 Claude Code CLI 发 prompt 拿响应（print mode）。
// 用 bash -lc 让 user PATH 生效。session_id 让所有调用接到同一对话上下文。
// 先按 continue_session 试一次；如果失败原因是 session 已存在 / 不存在，自动 fallback。
#[tauri::command]
async fn cc_chat(
    prompt: String,
    session_id: Option<String>,
    continue_session: Option<bool>,
    subdir: Option<String>,
) -> Result<String, String> {
    let sd = subdir.as_deref().unwrap_or("main");
    let cwd = soul_core_cc_cwd(sd)?;
    let sid = session_id.as_deref().filter(|s| is_uuid(s));
    let cont = continue_session.unwrap_or(false);

    let primary_cmd = match (sid, cont) {
        (Some(s), true) => format!("claude -p --resume {}", s),
        (Some(s), false) => format!("claude -p --session-id {}", s),
        (None, _) => "claude -p".to_string(),
    };

    let (stdout, stderr, code) = run_claude(&primary_cmd, &prompt, &cwd).await?;
    if code == 0 {
        return Ok(stdout);
    }

    // Fallback: 只在能明确判断"session state 不匹配"时反向尝试。
    //   1. stderr 含 "already in use" → 当前 --session-id 失败，转 --resume
    //   2. stderr 含 "not found" → 当前 --resume 失败，转 --session-id
    // 注意：曾加过"stderr 空就反向 fallback"——但实测它会把 auth 错（stdout="403"，stderr 空）
    // 误判成 session 错，反向尝试反而创建/破坏 session。撤掉了。
    if let Some(s) = sid {
        let lower = stderr.to_lowercase();
        let try_alt = if !cont && (lower.contains("already") || lower.contains("in use") || lower.contains("exists")) {
            Some(format!("claude -p --resume {}", s))
        } else if cont && (lower.contains("not found") || lower.contains("does not exist") || lower.contains("no such")) {
            Some(format!("claude -p --session-id {}", s))
        } else {
            None
        };
        if let Some(alt_cmd) = try_alt {
            // 让上一个失败的 claude 子进程退出状态完全稳定，避免和 fallback 起 race
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            log_diag(&format!("primary failed, try alt: {}", alt_cmd));
            let (stdout2, stderr2, code2) = run_claude(&alt_cmd, &prompt, &cwd).await?;
            if code2 == 0 {
                return Ok(stdout2);
            }
            return Err(format!(
                "claude failed (primary exit {}: stderr={:?} stdout={:?}) (alt exit {}: stderr={:?} stdout={:?})",
                code,
                stderr.trim(),
                stdout.trim(),
                code2,
                stderr2.trim(),
                stdout2.trim(),
            ));
        }
    }

    Err(format!(
        "claude exited with {}: stderr={:?} stdout={:?}",
        code,
        stderr.trim(),
        stdout.trim(),
    ))
}

// 返回心舍 / Soul·Core CC 的 session 存储目录路径（让 UI 能告诉用户文件在哪）
#[tauri::command]
fn cc_session_dir(subdir: Option<String>) -> Result<String, String> {
    let sd = subdir.as_deref().unwrap_or("main");
    let cwd = soul_core_cc_cwd(sd)?;
    Ok(cwd.to_string_lossy().to_string())
}

/// 默认 Memento 计数器：扫 ~/Memento/middle/entries/ 下所有 .md 文件
/// 按 frontmatter `kind:` 字段区分 outward / inward。
/// 返回 { outward, inward, available } —— available=false 表示 Memento 目录不存在
/// （fork 者通常会用 window.SOULCORE_MEMORY_SYSTEM 覆盖 loadCounts，这是默认接到作者 Memento 的实现）
#[tauri::command]
fn memento_counts() -> Result<serde_json::Value, String> {
    let home = std::env::var("HOME").map_err(|e| format!("no HOME: {}", e))?;
    let base = std::path::PathBuf::from(&home)
        .join("Memento")
        .join("middle")
        .join("entries");
    if !base.exists() {
        return Ok(serde_json::json!({ "outward": 0, "inward": 0, "available": false }));
    }
    let mut outward: u32 = 0;
    let mut inward: u32 = 0;
    fn walk(dir: &std::path::Path, outward: &mut u32, inward: &mut u32) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                walk(&p, outward, inward);
            } else if p.extension().map_or(false, |x| x == "md") {
                let Ok(s) = std::fs::read_to_string(&p) else { continue };
                // 只看前 ~30 行的 frontmatter
                for line in s.lines().take(30) {
                    let t = line.trim();
                    if let Some(rest) = t.strip_prefix("kind:") {
                        let v = rest.trim().trim_matches('"').trim_matches('\'');
                        if v == "outward" { *outward += 1; }
                        else if v == "inward" { *inward += 1; }
                        break;
                    }
                }
            }
        }
    }
    walk(&base, &mut outward, &mut inward);
    Ok(serde_json::json!({ "outward": outward, "inward": inward, "available": true }))
}

/// 织 server 通用代理（绕过 webview 跨域 CORS）
/// - webview origin 是 tauri://localhost，向 http://127.0.0.1:3000 发 fetch 会被浏览器 CORS 拦
/// - zhi server 未设 Access-Control-Allow-Origin，所以前端 fetch 拿不到响应
/// - 用 Rust reqwest 走原生网络栈，没有 CORS 限制
/// 参数：method (GET/POST/PATCH/DELETE)、path（如 "/api/health"）、body（可选 JSON 字符串）
/// 返回：{status: u16, body: string} —— body 是原始响应字符串，前端自己 JSON.parse
#[tauri::command]
async fn zhi_proxy(
    method: String,
    path: String,
    body: Option<String>,
) -> Result<serde_json::Value, String> {
    let base = "http://127.0.0.1:3000";
    let url = format!("{}{}", base, path);
    // 显式 no_proxy：贞元机器有系统代理（VPN / clash 等），reqwest 默认 honor HTTP_PROXY
    // 系统代理会拦截到 127.0.0.1:3000 的请求返回 502。本地织 server 无需走代理。
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .no_proxy()
        .build()
        .map_err(|e| format!("build client: {}", e))?;

    let m = method.to_uppercase();
    let mut req = match m.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PATCH" => client.patch(&url),
        "DELETE" => client.delete(&url),
        "PUT" => client.put(&url),
        _ => return Err(format!("unsupported method: {}", method)),
    };
    if let Some(b) = body {
        req = req.header("content-type", "application/json").body(b);
    }
    let resp = req.send().await.map_err(|e| format!("request error: {}", e))?;
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    Ok(serde_json::json!({ "status": status, "body": text }))
}

/// 启动织 dev server (`~/repos/zhi` · `bun run dev`)
/// - cwd: ~/repos/zhi（GUI 启动时 PATH 缺 /opt/homebrew/bin，显式注入）
/// - detach: 不 wait，心舍关掉了 zhi server 还在跑（用户自己 kill）
/// - 仓库不存在 / bun 不在 PATH 时返回错误供前端展示
#[tauri::command]
fn start_zhi_dev() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|e| format!("no HOME: {}", e))?;
    let zhi_dir = std::path::PathBuf::from(&home).join("repos").join("zhi");
    if !zhi_dir.exists() {
        return Err(format!("~/repos/zhi 不存在（{}）—— 请先 git clone 织仓库", zhi_dir.display()));
    }

    // 注入 brew PATH（GUI 启动环境缺）
    let extra = "/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin";
    let path = std::env::var("PATH").unwrap_or_default();
    let new_path = if path.is_empty() { extra.to_string() } else { format!("{}:{}", extra, path) };

    // detach spawn：stdout/stderr 丢到 /dev/null，避免 zombie
    let dev_null_out = std::fs::File::open("/dev/null").map_err(|e| format!("open /dev/null: {}", e))?;
    let dev_null_err = std::fs::File::open("/dev/null").map_err(|e| format!("open /dev/null: {}", e))?;

    let child = std::process::Command::new("bun")
        .args(["run", "dev"])
        .current_dir(&zhi_dir)
        .env("PATH", new_path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(dev_null_out))
        .stderr(std::process::Stdio::from(dev_null_err))
        .spawn()
        .map_err(|e| format!("spawn bun 失败（bun 可能不在 PATH）: {}", e))?;

    // 不等 wait()，让进程独立跑
    Ok(format!("已启动织 dev server (pid={})，约 2-3 秒后健康检查应该通过", child.id()))
}

/// macOS · 把 dashboard window 设成"桌面图标层"，跟原生桌面 widget 同层级
/// NSDesktopIconWindowLevel ≈ -2147483603（桌面图标所在层，wallpaper 之上、所有 app 之下）
/// 注意：那层 macOS 默认不接收 mouse events——若要让 widget 可交互必须自定义 canBecomeKey
/// 目前未使用（保留以备实验），widget 体验由 alwaysOnBottom + transparent + decorations:false 组合实现
#[cfg(target_os = "macos")]
#[allow(dead_code)]
#[allow(deprecated)]
fn set_dashboard_to_widget_level(win: &tauri::WebviewWindow) -> Result<(), String> {
    use cocoa::appkit::NSWindow;
    use cocoa::base::id;
    let ns_window_ptr = win.ns_window().map_err(|e| e.to_string())?;
    let ns_window: id = ns_window_ptr as id;
    unsafe {
        NSWindow::setLevel_(ns_window, -2147483603);
    }
    Ok(())
}

/// 切换 dashboard widget 前置固定状态
/// pinned=true → alwaysOnTop（始终前置，像 mac 便签 Float on Top）
/// pinned=false → alwaysOnBottom（widget 默认行为，在所有 app 后面）
#[tauri::command]
async fn set_dashboard_pin(app: tauri::AppHandle, pinned: bool) -> Result<bool, String> {
    use tauri::Manager;
    let win = app
        .get_webview_window("dashboard")
        .ok_or_else(|| "dashboard window not found".to_string())?;
    if pinned {
        win.set_always_on_bottom(false).map_err(|e| e.to_string())?;
        win.set_always_on_top(true).map_err(|e| e.to_string())?;
    } else {
        win.set_always_on_top(false).map_err(|e| e.to_string())?;
        win.set_always_on_bottom(true).map_err(|e| e.to_string())?;
    }
    Ok(pinned)
}

/// 切换 dashboard widget 尺寸（完整模式 / 收窄提醒模式）
/// width/height 单位为逻辑像素（Tauri LogicalSize），不受 DPR 影响
/// min_width/min_height 同步调整，避免 conf 的 minSize 卡住小尺寸（如 narrow 120 高被 conf minHeight 340 阻挡）
#[tauri::command]
async fn set_dashboard_size(
    app: tauri::AppHandle,
    width: f64,
    height: f64,
    min_width: Option<f64>,
    min_height: Option<f64>,
) -> Result<(), String> {
    use tauri::Manager;
    let win = app
        .get_webview_window("dashboard")
        .ok_or_else(|| "dashboard window not found".to_string())?;
    // 先放开 min_size，再 set_size，否则 set_size 被旧 min 卡住
    let mw = min_width.unwrap_or(width.min(240.0));
    let mh = min_height.unwrap_or(height.min(100.0));
    win.set_min_size(Some(tauri::LogicalSize::new(mw, mh)))
        .map_err(|e| e.to_string())?;
    win.set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn toggle_dashboard(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri::Manager;
    let win = app
        .get_webview_window("dashboard")
        .ok_or_else(|| "dashboard window not found".to_string())?;
    let visible = win.is_visible().map_err(|e| e.to_string())?;

    // diag · 记录 dashboard 当前状态到 /tmp/soul-core-cc-diag.log
    {
        use std::io::Write;
        let pos = win.outer_position().ok();
        let size = win.outer_size().ok();
        let always_on_top = win.is_always_on_top().ok();
        let minimized = win.is_minimized().ok();
        let msg = format!(
            "[toggle_dashboard] visible={} pos={:?} size={:?} always_on_top={:?} minimized={:?}",
            visible, pos, size, always_on_top, minimized
        );
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true).append(true)
            .open("/tmp/soul-core-cc-diag.log")
        {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs()).unwrap_or(0);
            let _ = writeln!(f, "[t={}] {}", ts, msg);
        }
    }

    if visible {
        win.hide().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        // 强制重新摆位 + 临时 always_on_top，破解"在屏幕外 / 在主端后面"问题
        // 600,200 是一个稳定可见的初始位置
        let _ = win.set_position(tauri::LogicalPosition::new(600.0_f64, 200.0_f64));
        let _ = win.set_always_on_bottom(false);
        let _ = win.set_always_on_top(true);
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        // 2 秒后恢复 alwaysOnBottom（默认行为）
        let win_clone = win.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = win_clone.set_always_on_top(false);
            let _ = win_clone.set_always_on_bottom(true);
        });
        Ok(true)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_filter(|label| label == "main")
                .build(),
        )
        .setup(|_app| {
            // 撤回 NSDesktopIconWindowLevel: 那层 macOS 默认不接收 mouse events
            // 改用 alwaysOnBottom (在 tauri.conf.json) — widget 仍在所有 app 后面但能交互
            Ok(())
        })
        .on_window_event(|window, event| {
            // 主窗口关闭（点红点 / Cmd+Q）→ 整个 app 退出
            // 修 transparent + macOSPrivateApi + 无装饰窗口下 Cmd+Q 没真正 terminate 的 bug
            // dashboard widget 独立关闭不触发 app 退出（保持 widget 可作为常驻浮窗的语义）
            use tauri::Manager;
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![cc_chat, cc_session_dir, toggle_dashboard, set_dashboard_pin, set_dashboard_size, memento_counts, start_zhi_dev, zhi_proxy])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // macOS: dock icon click / cmd-tab activate → unminimize + show + focus main window
            // 缺这个 handler，主窗口 minimize 后 dock 点击不会重新显示（特别是 transparent+macOSPrivateApi 下）
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { has_visible_windows, .. } = event {
                use tauri::Manager;
                if !has_visible_windows {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.unminimize();
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
        });
}
