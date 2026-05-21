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

// 在 cwd 下用 bash -lc 跑一次 claude，返回 (stdout, stderr, exit_code)
async fn run_claude(
    cmd_str: &str,
    prompt: &str,
    cwd: &std::path::Path,
) -> Result<(String, String, i32), String> {
    let mut child = Command::new("/bin/bash")
        .current_dir(cwd)
        .arg("-lc")
        .arg(cmd_str)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn claude failed: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .await
            .map_err(|e| format!("write stdin failed: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("wait failed: {}", e))?;

    Ok((
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code().unwrap_or(-1),
    ))
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

    // Fallback: 如果有 sid，错误可能是 "already in use"(创建已存在) 或 "not found"(resume 不存在)
    // 自动换另一种姿态再试一次
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
            let (stdout2, stderr2, code2) = run_claude(&alt_cmd, &prompt, &cwd).await?;
            if code2 == 0 {
                return Ok(stdout2);
            }
            return Err(format!(
                "claude failed (primary exit {}: {}) (alt exit {}: {})",
                code,
                stderr.trim(),
                code2,
                stderr2.trim()
            ));
        }
    }

    Err(format!("claude exited with {}: {}", code, stderr.trim()))
}

// 返回 norvera CC 的 session 存储目录路径（让 UI 能告诉用户文件在哪）
#[tauri::command]
fn cc_session_dir(subdir: Option<String>) -> Result<String, String> {
    let sd = subdir.as_deref().unwrap_or("main");
    let cwd = soul_core_cc_cwd(sd)?;
    Ok(cwd.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![cc_chat, cc_session_dir])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
