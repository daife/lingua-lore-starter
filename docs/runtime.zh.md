# 运行机制

[**English**](runtime.md) | [**中文**](runtime.zh.md)

---

本文按当前代码描述运行机制，不沿用旧 README 的概括性说法。

![语境传说运行总览](images/runtime-overview.png)

## 端到端流程

代码里有四个明确分开的操作：

1. `generate_world_draft`：让模型生成世界表单草稿。
2. `create_world`：创建本地存储并 seed 世界数据库。
3. `preview_story_turn`：生成并校验故事回合，但不提交。
4. `commit_story_turn_preview`：把已接受的 preview 提交进世界数据库。

preview 和 commit 的分离是有意设计的。它允许快速模式预取分支，也保证 Rust 在任何状态变更前先校验模型输出。

## 世界草稿生成

“AI 填写”路径调用 `world_commands.rs` 中的 `generate_world_draft`。

- 从 `app.db` 读取最新 API 配置。
- 使用兼容 DeepSeek 的 Chat Completions 请求。
- 将 `response_format` 设为 `json_object`。
- 不暴露任何工具。
- 要求只生成一个玩家视角角色。
- 校验草稿必须有且只有一个玩家角色，并且必填字段完整。
- 规范化选中的类型、目标语言、语言等级、叙事风格和角色标记。

草稿只会返回给前端表单。它本身不会创建世界。

## 世界初始化

`create_world` 会持久化表单数据：

- 创建新的 `world_<uuid>` id。
- 向 `app.db.worlds` 插入记录。
- 创建 `worlds/<world_id>/world.db`。
- 执行世界数据库 migration。
- 插入 `world_profile`。
- 插入一个名为 `Opening Scene` 的开场场景。
- 将 `story_state.scene.current` 设为开场场景 id。
- 要求严格一个玩家角色，并存为 `char_player`。
- 可选非玩家初始角色会带有默认 `trust = 0`。

## 打开世界

`get_world_bootstrap` 会读取：

- 来自 `app.db` 的世界记录。
- 来自 `story_state.scene.current` 的当前场景 id；如果没有则退回第一个场景。
- 来自 `turns`、`messages` 和 `branch_choices` 的历史回合。

前端把 bootstrap 结果放进 app state，并渲染阅读器。

## 故事回合运行

![语境传说故事回合运行](images/story-turn-runtime.png)

阅读器的第一条动作是自由文本：

```text
Initialize the story with a vivid opening scene.
```

后续动作可能是：

- `{ kind: "choice", choice_id }`
- `{ kind: "free_text", text }`

`preview_story_turn` 随后会：

1. 打开世界数据库。
2. 调用 `load_context`。
3. 调用 `build_messages`。
4. 注册只读工具。
5. 以 `tool_choice: "auto"` 和 `response_format: json_object` 调用模型。
6. 如果模型请求工具，则执行工具并追加 `role: "tool"` 消息。
7. 将最终正文解析为 `TurnOutput`。
8. 校验输出。
9. 返回包含原始 JSON 和解析结果的 preview。

如果模型返回非法内容，Rust 会追加修复提示并重试本回合。

## 运行限制

| 区域 | 限制 |
|---|---:|
| 模型请求重试 | 4 |
| 世界草稿修复尝试 | 4 |
| 故事回合修复尝试 | 4 |
| 单次故事 preview 工具轮数 | 3 |
| 单次故事 preview 工具调用总数 | 8 |
| 装载最近消息 | 12 |
| 装载最近摘要 | 8 |
| 写入 prompt 的角色数 | 12 |
| 装载故事状态行数 | 80 |
| 装载关系状态行数 | 80 |

## 上下文装载

`load_context` 读取：

- `world_profile`
- 当前 `scenes` 行
- 最多 12 个角色，玩家角色排在前面
- 最多 80 行 `story_state`
- 最多 80 行 `relationship_state`
- 当前场景最近 12 条消息
- 当前场景最近 8 条摘要
- 当前用户行动，也就是自由文本或选中的 `branch_choices` 行

## 提交

`commit_story_turn_preview` 会再次校验，然后调用 `commit_turn`。提交发生在一个 SQLite 事务中。

它写入：

- 用户消息和助手消息
- `turns` 行
- 更新后的场景地点、氛围、目标和摘要
- 带稳定 id 的新分支选项
- 故事状态更新和日志
- 新非玩家角色
- 记忆候选和已晋升记忆
- 关系状态变化和日志

模型仍然不能直接写库。它只提出结构化输出；Rust 决定什么有效、什么会被持久化。

## 记忆系统

当前记忆系统基于 SQLite：

- 模型输出 `memory_candidates`。
- 每条候选都会写入 `memory_candidates`。
- 只有 `importance >= 7` 且 `character_id` 存在时，候选才会晋升到 `memories`。
- `query_character_memory` 之后通过 SQL `LIKE` 查询已晋升记忆，并按重要度和时间排序。

当前实现没有向量检索或 embedding 索引。

## 快速模式

快速模式是前端预取行为。它不会改变模型、prompt、temperature 或校验规则。

当快速模式开启且当前回合有 choice id 时，前端会为每个可选分支调用 `preview_story_turn`，并缓存 promise/result。如果玩家选择了已经预取的分支，前端会把该缓存 preview 发送给 `commit_story_turn_preview`。

这能让选择后的响应更快，但可能消耗更多模型请求，因为未选择的分支也可能已经生成。

## 划词翻译

划词翻译位于故事循环之外。被选中的文本会带着源语言、目标语言和可选上下文发给有道 provider。结果显示在浮层中，不进入故事 prompt，也不修改世界状态。
