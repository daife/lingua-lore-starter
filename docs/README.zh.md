# 文档

[**English**](README.md) | [**中文**](README.zh.md)

---

这里保存项目的详细文档。根目录 README 故意保持简短，具体机制放在本目录中。

## 推荐阅读顺序

1. [运行机制](runtime.zh.md)：世界草稿、世界初始化、故事预览、工具调用、提交、记忆和快速模式的端到端流程。
2. [架构](architecture.zh.md)：前端、Tauri 命令、Rust 模块、存储边界和运行时职责。
3. [数据库结构](database.zh.md)：当前 SQLite 数据库和各表职责。
4. [LLM 运行时](llm-runtime.zh.md)：DeepSeek 兼容请求、JSON 输出、只读工具、重试和校验。
5. [提示词设计](prompt-design.zh.md)：故事 prompt 包含什么，以及明确禁止什么。
6. [开发](development.zh.md)：依赖安装、本地开发、检查和构建命令。
7. [发布流程](release.zh.md)：本地发布步骤和版本文件。

## 图片

- [运行总览](images/runtime-overview.png)
- [故事回合运行](images/story-turn-runtime.png)

## 文档维护说明

- `deepseek-json-mode.md` 和 `deepseek-tool-calls.md` 已合并到 [LLM 运行时](llm-runtime.zh.md)，因为拆分文档存在重复，并且旧工具文档列出了当前代码不存在的工具。
- 更新数据库和运行机制文档前，应对照 `apps/desktop/src-tauri/migrations/`、`story_runtime/`、`tool_runtime/` 和 `turn_commit/`。
