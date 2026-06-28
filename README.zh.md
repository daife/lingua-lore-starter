# 语境传说

[**English**](README.md) | [**中文**](README.zh.md)

---

一款用于沉浸式外语故事阅读的桌面和移动端应用。

### 产品名称

| 语言 | 名称 |
|---|---|
| English | Lingua Lore |
| 简体中文 | 语境传说 |
| 日本語 | 言の葉ロア |

## 为什么选择语境传说？

每一个互动故事引擎都面临着同一堵墙：**LLM 会失忆**。角色说着说着就变了口音，剧情走到半路突然跑偏，世界设定在几句话后灰飞烟灭。

语境传说从一开始就是为了解决这个问题而构建的——不是靠脆弱的提示词补丁，而是靠一道**持久化记忆架构**深植于每一个故事回合之中。

### 🧠 持久记忆与反失忆核心

- **结构化记忆候选**：每一回合都会产出一组记忆候选——关键事件、角色观察、世界状态变化——由 Rust 端校验后在一个 ACID 事务中全部提交。
- **维度化关系追踪**：角色关系沿多个维度（信任、熟悉度、好感度）动态演变。每次交互记录变化量 + 原因，LLM 再也不用猜测谁信任谁、为什么信任。
- **场景感知上下文加载**：运行时不会倾泻整个历史——只加载当前场景相关的内容，保持上下文窗口精悍、回复犀利。
- **回合摘要锚定**：每次回复都包含一份压缩的 `turn_summary`，作为后续回合的锚定上下文，形成闭环记忆链。

### 👥 活着的人物系统

角色不是装饰标签。每个角色都携带：
- **性格、背景、说话风格**——定义了他们的反应方式，而不仅仅是说了什么
- **随玩家选择动态变化的关系维度**
- **对过往交互的记忆**——LLM 在跨回合之间可以引用
- **玩家角色支持**——以你自己的身份踏入故事，而不是提线木偶

你的选择会留下真正的痕迹。角色记得你做过什么。世界因你的决策而改变，而不是反过来。

### 🎮 全方位沉浸体验

- **分支选择叙事**：每回合精确给出三个选项，每个都标注意图和风险等级
- **自由文本输入**：当预设选项不够用时，LLM 会在世界中解释你的行动
- **独立划词翻译**：使用有道词典公共接口，支持中英日韩词典语言对，翻译在 LLM 上下文之外运行，绝不污染或膨胀故事状态
- **快速模式**：消耗更多 token，换取更深层、更连贯的生成
- **启动版本检测**：永远不错过任何更新

语境传说不是给聊天框套了一层奇幻皮肤。它是一个**有状态的故事引擎**，每一个回合都在强化故事的内在一致性。

## 开发路线图

### ✅ 已完成

- [x] 世界创建、打开、删除与导入 / 导出
- [x] AI 辅助世界草稿生成
- [x] 沉浸式故事阅读体验
- [x] 分支选择式互动叙事
- [x] 自由文本行动输入
- [x] 独立多语种划词翻译（有道词典：中英日韩词典语言对）
- [x] 世界导出 / 导入（ZIP 格式）
- [x] 多 API 配置支持
- [x] 快速模式（更高品质，更高 token 消耗）
- [x] 多语言界面（简体中文 / English / 日本語）
- [x] Windows（MSI + NSIS）及 Android（APK）构建
- [x] 启动时自动版本更新检测

### 🚧 规划中

- [ ] 人物关系查看
- [ ] 思考模式支持
- [ ] 参考模式支持（上传小说作为参考素材）
- [ ] 自定义角色卡片
- [ ] 进度回退

## 技术栈

- Tauri + Rust 后端
- React + Vite 前端
- SQLite 存储
- DeepSeek Chat Completions（兼容 OpenAI API）
- 有道词典公共接口（独立划词翻译）

## 核心运行机制

- LLM 故事生成使用 JSON Output 模式
- 工具调用为可选且只读
- 每个故事回合必须返回：叙述、对话、摘要、场景状态、三个选择、状态更新候选、记忆候选、关系更新
- Rust 端校验最终 JSON 并在单个事务内提交所有写入
- 划词翻译不进入 LLM 上下文
- 世界导出/导入使用 ZIP 包，包含 `manifest.json` 和 `world.db`

## 环境配置

```powershell
npm install
```

Android 构建需要额外安装：

- Android Studio
- Android SDK Platform Tools
- Android SDK Build Tools
- Android SDK Platform，当前 `android-36`
- Android NDK，当前 `27.0.12077973`
- Rust Android 目标：

```powershell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

推荐 Android 环境变量：

```powershell
$env:ANDROID_HOME="$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT="$env:LOCALAPPDATA\Android\Sdk"
$env:NDK_HOME="$env:ANDROID_HOME\ndk\27.0.12077973"
```

## 开发

```powershell
npm run dev
npm run typecheck
```

## Windows 构建

本地构建 Windows 应用：

```powershell
npm --workspace @lingua-lore/desktop run tauri -- build
```

输出文件位于：

```text
apps/desktop/src-tauri/target/release/bundle/
```

快速编译检查（不打包安装程序）：

```powershell
npm --workspace @lingua-lore/desktop run tauri -- build --debug --no-bundle
```

## Android 构建

初始化 Tauri Android 项目（仅需一次）：

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

## 本地发布

发布版本直接从本地构建产物发布，不使用 GitHub Actions 远程构建。

1. **完成代码修改并更新版本号**。需要更新的文件：

   | 文件 | 字段 |
   |---|---|
   | `package.json` | `version` |
   | `apps/desktop/package.json` | `version` |
   | `apps/desktop/src-tauri/Cargo.toml` | `version` |
   | `apps/desktop/src-tauri/tauri.conf.json` | `version` |
   | `apps/desktop/src-tauri/gen/android/app/tauri.properties` | `versionName` + `versionCode` |

   > ⚠️ Android `tauri.properties` 是自动生成文件，需要在构建前手动编辑。

2. **配置 Android APK 签名**（一次性）。在 `apps/desktop/src-tauri/gen/android/app/build.gradle.kts` 中添加：

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

   然后在 `release` 构建类型中引用：

   ```kotlin
   getByName("release") {
       signingConfig = signingConfigs.getByName("release")
       // ...
   }
   ```

3. **运行检查：**

   ```powershell
   npm run typecheck
   cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
   ```

4. **提交所有修改**（代码 + 版本号在同一 commit）：

   ```powershell
   git add .
   git commit -m "feat: your feature description"
   git push origin main
   ```

5. **本地构建 Windows 和 Android：**

   ```powershell
   npm --workspace @lingua-lore/desktop run tauri -- build
   npm --workspace @lingua-lore/desktop run tauri -- android build --apk --target aarch64
   ```

6. **重命名 Android APK** 为发布命名规范：

   ```powershell
   copy apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk "apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/Lingua Lore_0.1.x_android-arm64.apk"
   ```

7. **打标签并推送：**

   ```powershell
   git tag v0.1.x
   git push origin v0.1.x
   ```

8. **从本地产物创建 GitHub Release**（使用显式路径）：

   ```powershell
   $msi = "apps/desktop/src-tauri/target/release/bundle/msi/Lingua Lore_0.1.x_x64_en-US.msi"
   $exe = "apps/desktop/src-tauri/target/release/bundle/nsis/Lingua Lore_0.1.x_x64-setup.exe"
   $apk = "apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/Lingua Lore_0.1.x_android-arm64.apk"
   gh release create v0.1.x --title "Lingua Lore v0.1.x" --notes "Local release notes." "$msi" "$exe" "$apk"
   ```
