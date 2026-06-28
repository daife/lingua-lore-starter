# 架构

[**English**](architecture.md) | [**中文**](architecture.zh.md)

---

语境传说是一个 Tauri 应用，前端使用 React/Vite，后端使用 Rust。后端负责持久化、模型调用、翻译调用、校验和故事回合提交。

## 高层分层

| 层级 | 主要文件 | 职责 |
|---|---|---|
| React UI | `apps/desktop/src/pages/` | 世界库、阅读器、设置、划词翻译 UI |
| Tauri bridge | `apps/desktop/src/lib/tauri.ts` | 前端对 Tauri commands 的 typed wrapper |
| Commands | `apps/desktop/src-tauri/src/commands/` | 前端可调用的命令入口 |
| Storage | `apps/desktop/src-tauri/src/storage/mod.rs` | 应用数据库、世界数据库创建、导入导出、API 配置 |
| Story runtime | `apps/desktop/src-tauri/src/story_runtime/` | 上下文装载、prompt 构建、LLM 回合预览、输出解析和校验 |
| Tool runtime | `apps/desktop/src-tauri/src/tool_runtime/` | 只读工具定义和 SQLite 工具执行 |
| Turn commit | `apps/desktop/src-tauri/src/turn_commit/` | 已接受故事 preview 的单事务写入 |
| DeepSeek client | `apps/desktop/src-tauri/src/deepseek/` | 兼容 OpenAI 的请求/响应类型和 HTTP 客户端 |
| Translation | `apps/desktop/src-tauri/src/translation/` | 独立有道划词翻译 |

## 运行模式

故事引擎遵循这个模式：

```text
React 输入
  -> preview_story_turn
  -> 装载世界上下文
  -> 构建 prompt
  -> DeepSeek JSON 输出，可选调用只读工具
  -> 解析并校验 TurnOutput
  -> 返回 preview，不写库
  -> commit_story_turn_preview
  -> 在一个 SQLite 事务中提交所有写入
```

模型不能直接写库。它只能返回最终 JSON，或请求某个只读工具。Rust 负责校验并应用最终输出。

## 前端流程

- `WorldLibraryPage.tsx` 负责世界列表、创建世界、导入导出世界，以及 AI 世界草稿生成。
- `ReaderPage.tsx` 负责开始开场回合、发送自由文本行动、选择分支、快速模式预取分支 preview，以及划词翻译。
- `SettingsPage.tsx` 保存模型功能使用的 API 配置。

## 后端边界

- `generate_world_draft` 调用不带工具的模型请求，返回草稿表单数据。
- `create_world` 执行本地存储写入，并 seed 新的 `world.db`。
- `preview_story_turn` 可以调用模型和只读工具，但不会持久化新回合。
- `commit_story_turn_preview` 是故事回合唯一写入路径。
- 划词翻译位于故事循环之外，不修改世界状态。
