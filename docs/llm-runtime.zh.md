# LLM 运行时

[**English**](llm-runtime.md) | [**中文**](llm-runtime.zh.md)

---

模型集成使用兼容 DeepSeek 的 Chat Completions 请求结构。HTTP 客户端会 POST 到：

```text
{base_url}/chat/completions
```

base URL、model、API key 和 strict-tools 偏好来自最新保存的 API 配置。

## 世界草稿请求

世界草稿生成使用：

| 字段 | 值 |
|---|---|
| `tools` | `None` |
| `tool_choice` | `None` |
| `response_format` | `{ "type": "json_object" }` |
| `temperature` | `0.9` |
| `max_tokens` | `1200` |
| `stream` | `false` |

返回 JSON 会被解析为 `CreateWorldRequest`。

## 故事回合请求

故事回合 preview 使用：

| 字段 | 值 |
|---|---|
| `tools` | 只读工具定义 |
| `tool_choice` | `"auto"` |
| `response_format` | `{ "type": "json_object" }` |
| `temperature` | `0.85` |
| `max_tokens` | `4096` |
| `stream` | `false` |

最终返回 JSON 会被解析为 `TurnOutput`。

## 只读工具

当前代码只注册这些工具：

| 工具 | 必填参数 | 行为 |
|---|---|---|
| `query_character_profile` | `character_id` | 从 `characters` 读取一个角色 |
| `query_character_memory` | `character_id`、`query`、`limit` | 用 SQL `LIKE` 查询某角色已晋升的 `memories` |
| `query_past_events` | `query`、`limit` | 用 SQL `LIKE` 查询 `turns.summary` |

未知工具名会返回空结果和错误消息。工具永远不写库。

## 工具循环

当模型返回 `tool_calls` 时，Rust 会：

1. 统计工具轮数和工具调用总数。
2. 把包含 tool calls 的 assistant message 放入消息列表。
3. 在同一个世界数据库连接上执行每个工具。
4. 把每个结果序列化为 `role: "tool"` 消息。
5. 再次调用模型。

如果工具使用超过限制，Rust 会追加：

```text
Stop calling tools. Return the final valid json now.
```

限制：

- 最大工具轮数：`3`
- 最大工具调用总数：`8`

## 解析与修复

如果模型返回空内容、非法 JSON，或 JSON 未通过校验，Rust 会追加修复提示并重试，直到达到故事回合修复上限。

校验包括：

- `narration` 非空
- 严格三个选择
- 选择 label 必须是 `A`、`B`、`C`
- risk 只能是 `low`、`medium` 或 `high`
- 状态更新 key 必须在允许范围内
- 新角色最多三个
- 记忆重要度必须在 `1` 到 `10`
- 关系 delta 必须在 `-2` 到 `2`
- 同一角色同一维度不能重复更新

## 当前没有的能力

当前 LLM 运行时不包含：

- 流式响应
- 写工具
- 向量检索
- embedding 检索
- `query_world_lore` 工具
- 故事 prompt 中的语法/词汇教学
