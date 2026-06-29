# 发布流程

[**English**](release.md) | [**中文**](release.zh.md)

---

发布版本从本地 Android 构建产物创建。本 starter 仓库只发布 Android APK，不依赖 GitHub Actions 远程构建来打包 release。

## 版本文件

准备发布时，需要一起更新所有相关版本字段：

| 文件 | 字段 |
|---|---|
| `package.json` | `version` |
| `apps/desktop/package.json` | `version` |
| `apps/desktop/src-tauri/Cargo.toml` | `version` |
| `apps/desktop/src-tauri/tauri.conf.json` | `version` |
| `apps/desktop/src-tauri/gen/android/app/tauri.properties` | `versionName` 和 `versionCode` |

`tauri.properties` 由 Android 项目生成，但 release 构建前仍可能需要直接编辑它。

## 发布前检查

```powershell
npm run typecheck
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
```

## 构建产物

构建 Android APK：

```powershell
npm --workspace @lingua-lore/desktop run tauri -- android build --apk --target aarch64
```

常见产物位置：

```text
apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/
```

## Android 签名

Release APK 签名配置位于：

```text
apps/desktop/src-tauri/gen/android/app/build.gradle.kts
```

具体 keystore 和凭据属于本地发布事项，除非明确使用非机密测试 keystore，否则不应提交。

## 打标签并发布

提交版本号和发布相关修改后：

```powershell
git tag v0.1.x
git push origin v0.1.x
```

用明确的本地产物路径创建 GitHub release，例如：

```powershell
$apk = "apps/desktop/src-tauri/gen/android/app/build/outputs/apk/universal/release/Lingua Lore_0.1.x_android-arm64.apk"
gh release create v0.1.x --title "Lingua Lore v0.1.x" --notes "Local release notes." "$apk"
```

GitHub release asset 的下载文件名可能会把空格显示为点号，例如
`Lingua.Lore_0.1.x_android-arm64.apk`。这是 GitHub 对下载链接的规范化结果，发布链接保留点号即可。
