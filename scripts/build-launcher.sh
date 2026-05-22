#!/bin/bash
# 心舍 · Soul-Core 启动器生成脚本
# ────────────────────────────────────────────
# 在 ~/Desktop 生成 心舍.app launcher bundle:
#  - 杀残留进程 (修 Cmd+Q 后僵尸进程占住导致再启动失败)
#  - 显式注入 brew PATH (GUI 启动 environment 缺 /opt/homebrew/bin 导致 CC CLI 调用失败)
#  - 用项目 icon.icns 作为图标
#  - LSUIElement: true 不抢 dock 位置
# 
# 用法: bash scripts/build-launcher.sh
# 注意: 需要先 `cargo build` 出 binary

set -e

REPO="$(cd "$(dirname "$0")/.." && pwd)"
BINARY="$REPO/src-tauri/target/debug/soul-core"
ICON="$REPO/src-tauri/icons/icon.icns"
APP_NAME="心舍"
APP_DIR="$HOME/Desktop/$APP_NAME.app"

if [ ! -f "$ICON" ]; then
  echo "✗ icon 缺失: $ICON"
  exit 1
fi

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cat > "$APP_DIR/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>launcher</string>
  <key>CFBundleIdentifier</key>
  <string>com.soulcore.launcher</string>
  <key>CFBundleName</key>
  <string>心舍</string>
  <key>CFBundleDisplayName</key>
  <string>心舍</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleVersion</key>
  <string>1.0</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>LSArchitecturePriority</key>
  <array>
    <string>arm64</string>
  </array>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>CFBundleSupportedPlatforms</key>
  <array>
    <string>MacOSX</string>
  </array>
</dict>
</plist>
PLIST

cat > "$APP_DIR/Contents/MacOS/launcher" <<LAUNCHER
#!/bin/bash
# 心舍 launcher
BINARY="$BINARY"

# 显式注入 brew path 让 binary 内 \`bash -lc "claude"\` 能找到 claude CLI
# GUI 双击启动 environment 缺 /opt/homebrew/bin, 需要 launcher 帮注入
export PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:\$PATH"

# 杀残留 (Cmd+Q 不真退出时累积的僵尸进程)
pkill -9 -f "\$BINARY" 2>/dev/null
sleep 0.3

if [ ! -x "\$BINARY" ]; then
  osascript -e 'display dialog "找不到 binary。请先 cd \$(dirname "\$BINARY") && cargo build" buttons {"OK"} default button "OK"'
  exit 1
fi

exec "\$BINARY"
LAUNCHER

chmod +x "$APP_DIR/Contents/MacOS/launcher"
cp "$ICON" "$APP_DIR/Contents/Resources/AppIcon.icns"

# 让 Finder 重新加载 icon
touch "$APP_DIR"
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$APP_DIR" 2>/dev/null

echo "✓ launcher 生成: $APP_DIR"
echo "  双击桌面「$APP_NAME」启动 app"
