# Development

[**English**](development.md) | [**中文**](development.zh.md)

---

## Install

From the repository root:

```powershell
npm install
```

## Local Development

Run the Tauri development app:

```powershell
npm run dev
```

This delegates to:

```powershell
npm --workspace @lingua-lore/desktop run tauri:dev
```

## Checks

Type-check the frontend:

```powershell
npm run typecheck
```

Check the Rust backend:

```powershell
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
```

## Android Build Prerequisites

Install:

- Android Studio
- Android SDK Platform Tools
- Android SDK Build Tools
- Android SDK Platform, currently `android-36`
- Android NDK, currently `27.0.12077973`
- Rust Android targets

Add Rust Android targets:

```powershell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

Recommended environment variables on Windows:

```powershell
$env:ANDROID_HOME="$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT="$env:LOCALAPPDATA\Android\Sdk"
$env:NDK_HOME="$env:ANDROID_HOME\ndk\27.0.12077973"
```

Initialize the Tauri Android project once:

```powershell
npm --workspace @lingua-lore/desktop run tauri -- android init
```

Build a release APK:

```powershell
npm --workspace @lingua-lore/desktop run tauri -- android build --apk --target aarch64
```

APK output:

```text
apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/
```
