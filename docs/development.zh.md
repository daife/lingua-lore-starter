# 开发

[**English**](development.md) | [**中文**](development.zh.md)

---

## 安装依赖

在仓库根目录运行：

```powershell
npm install
```

## 本地开发

运行 Tauri 开发应用：

```powershell
npm run dev
```

它会委托到：

```powershell
npm --workspace @lingua-lore/desktop run tauri:dev
```

## 检查

检查前端类型：

```powershell
npm run typecheck
```

检查 Rust 后端：

```powershell
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
```

## 桌面构建

构建 Tauri 桌面 bundle：

```powershell
npm --workspace @lingua-lore/desktop run tauri -- build
```

更快的编译检查，不生成安装包：

```powershell
npm --workspace @lingua-lore/desktop run tauri -- build --debug --no-bundle
```

Windows bundle 输出位置：

```text
apps/desktop/src-tauri/target/release/bundle/
```

## Android 构建前置条件

安装：

- Android Studio
- Android SDK Platform Tools
- Android SDK Build Tools
- Android SDK Platform，当前 `android-36`
- Android NDK，当前 `27.0.12077973`
- Rust Android targets

添加 Rust Android targets：

```powershell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

Windows 推荐环境变量：

```powershell
$env:ANDROID_HOME="$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT="$env:LOCALAPPDATA\Android\Sdk"
$env:NDK_HOME="$env:ANDROID_HOME\ndk\27.0.12077973"
```

首次初始化 Tauri Android 项目：

```powershell
npm --workspace @lingua-lore/desktop run tauri -- android init
```

构建 release APK：

```powershell
npm --workspace @lingua-lore/desktop run tauri -- android build --apk --target aarch64
```

APK 输出位置：

```text
apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/
```
