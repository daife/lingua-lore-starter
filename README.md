# Lingua Lore

[**English**](README.md) | [**中文**](README.zh.md)

---

A desktop and mobile app for immersive foreign-language story reading.

### Product Names

| Language | Name |
|---|---|
| English | Lingua Lore |
| Chinese (简体中文) | 语境传说 |
| Japanese (日本語) | 言の葉ロア |

## Why Lingua Lore?

Every interactive story engine faces the same wall: **the LLM forgets**. Characters lose their accent mid-conversation, the plot derails, the world's own rules dissolve into thin air.

Lingua Lore was built from the ground up to solve this — not with fragile prompt hacks, but with a **persistent memory architecture** baked into every story turn.

### 🧠 Persistent Memory & Anti-Amnesia Core

- **Structured memory candidates**: Every turn surfaces memory candidates — key events, character observations, world-state changes — that Rust validates and commits in a single ACID transaction.
- **Relationship tracking with delta system**: Character relationships evolve dimensionally (trust, familiarity, affection). Each interaction records a delta + reason, so the LLM never has to guess who trusts whom or why.
- **Scene-aware context loading**: The runtime doesn't dump the entire history — it loads only what's relevant to the current scene, keeping context windows lean and responses sharp.
- **Turn-summary anchoring**: Every response includes a compressed `turn_summary` that future turns consume as anchoring context, creating a closed-loop memory chain.

### 👥 Character System That Lives

Characters aren't decorative tags. Each one carries:
- **Personality, background, speaking style** — defining how they react, not just what they say
- **Dynamic relationship dimensions** that change with player choices
- **Memory of past interactions** — referenced by the LLM across turns
- **Player-character support** — step into the story as yourself, not a puppet

Your choices leave real traces. Characters remember what you did. The world bends around your decisions, not the other way around.

### 🎮 Immersive on Purpose

- **Choice-driven narrative** with exactly three curated options per turn, each tagged with intent and risk level
- **Free-text input** for when the presets don't fit — the LLM interprets your action in-world
- **Selection translation** uses Youdao's public dictionary endpoint for Chinese, English, Japanese, and Korean dictionary pairs, and never pollutes or inflates the LLM story context
- **Quick mode** for deeper, more coherent generation at higher token cost
- **Auto version check** on startup — never miss an update

Lingua Lore isn't a chat wrapper with a fantasy skin. It's a **stateful narrative engine** where every turn strengthens the story's internal consistency.

## Roadmap

### ✅ Completed

- [x] World creation, opening, deletion, export, and import
- [x] AI-assisted world draft generation
- [x] Immersive story reading experience
- [x] Branching choice-driven narrative
- [x] Free-text action input
- [x] Independent multilingual selection translation (Youdao dictionary pairs for Chinese, English, Japanese, and Korean)
- [x] World export / import (ZIP format)
- [x] Multiple API profile support
- [x] Quick mode (higher quality, higher token cost)
- [x] Multilingual UI (English / 中文 / 日本語)
- [x] Windows (MSI + NSIS) and Android (APK) builds
- [x] Automatic version update check on startup

### 🚧 In Progress

- [ ] Character relationship viewer
- [ ] Thinking mode support
- [ ] Reference mode (upload novel as reference material)
- [ ] Custom character cards
- [ ] Progress rollback

## Stack

- Tauri + Rust backend
- React + Vite frontend
- SQLite storage
- DeepSeek Chat Completions with an OpenAI-compatible API shape
- Youdao public dictionary endpoint for independent selection translation

## Core Runtime

- LLM story generation uses JSON Output.
- Tool calls are optional and read-only.
- Every story turn must return narration, dialogues, summary, scene status, exactly three choices, state update candidates, memory candidates, and relationship updates.
- Rust validates final JSON and commits all writes in one transaction.
- Selection translation never enters LLM context.
- World export/import uses a zip package containing `manifest.json` and `world.db`.

## Setup

```powershell
npm install
```

For Android builds, also install:

- Android Studio
- Android SDK Platform Tools
- Android SDK Build Tools
- Android SDK Platform, currently `android-36`
- Android NDK, currently `27.0.12077973`
- Rust Android targets:

```powershell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

Recommended Android environment variables:

```powershell
$env:ANDROID_HOME="$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT="$env:LOCALAPPDATA\Android\Sdk"
$env:NDK_HOME="$env:ANDROID_HOME\ndk\27.0.12077973"
```

## Development

```powershell
npm run dev
npm run typecheck
```

## Windows Build

Build the Windows app locally:

```powershell
npm --workspace @lingua-lore/desktop run tauri -- build
```

Useful outputs are written under:

```text
apps/desktop/src-tauri/target/release/bundle/
```

For a fast local compile check without packaging installers:

```powershell
npm --workspace @lingua-lore/desktop run tauri -- build --debug --no-bundle
```

## Android Build

Initialize the Tauri Android project once:

```powershell
npm --workspace @lingua-lore/desktop run tauri -- android init
```

Build a release APK locally:

```powershell
npm --workspace @lingua-lore/desktop run tauri -- android build --apk --target aarch64
```

The APK is written under:

```text
apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/
```

## Local Release

Releases are published from local build artifacts. GitHub Actions remote builds are intentionally not used.

1. **Make code changes and bump versions**. Update all version files:

   | File | Field |
   |---|---|
   | `package.json` | `version` |
   | `apps/desktop/package.json` | `version` |
   | `apps/desktop/src-tauri/Cargo.toml` | `version` |
   | `apps/desktop/src-tauri/tauri.conf.json` | `version` |
   | `apps/desktop/src-tauri/gen/android/app/tauri.properties` | `versionName` + `versionCode` |

   > ⚠️ Android `tauri.properties` is auto-generated — edit it directly before the Android build.

2. **Configure Android APK signing** (one-time). Add a `signingConfigs` block to `apps/desktop/src-tauri/gen/android/app/build.gradle.kts`:

   ```kotlin
   signingConfigs {
       create("release") {
           storeFile = file("../lingua-lore-test.keystore")
           storePassword = "android"
           keyAlias = "lingua-lore-test"
           keyPassword = "android"
       }
   }
   ```

   Then reference it in the `release` build type:

   ```kotlin
   getByName("release") {
       signingConfig = signingConfigs.getByName("release")
       // ...
   }
   ```

3. **Run checks:**

   ```powershell
   npm run typecheck
   cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
   ```

4. **Commit everything** (code changes + version bumps in one commit):

   ```powershell
   git add .
   git commit -m "feat: your feature description"
   git push origin main
   ```

5. **Build Windows and Android locally:**

   ```powershell
   npm --workspace @lingua-lore/desktop run tauri -- build
   npm --workspace @lingua-lore/desktop run tauri -- android build --apk --target aarch64
   ```

6. **Rename the Android APK** to the release naming convention:

   ```powershell
   copy apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk "apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/Lingua Lore_0.1.x_android-arm64.apk"
   ```

7. **Tag and push:**

   ```powershell
   git tag v0.1.x
   git push origin v0.1.x
   ```

8. **Create the GitHub release** with explicit artifact paths:

   ```powershell
   $msi = "apps/desktop/src-tauri/target/release/bundle/msi/Lingua Lore_0.1.x_x64_en-US.msi"
   $exe = "apps/desktop/src-tauri/target/release/bundle/nsis/Lingua Lore_0.1.x_x64-setup.exe"
   $apk = "apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/Lingua Lore_0.1.x_android-arm64.apk"
   gh release create v0.1.x --title "Lingua Lore v0.1.x" --notes "Local release notes." "$msi" "$exe" "$apk"
   ```
