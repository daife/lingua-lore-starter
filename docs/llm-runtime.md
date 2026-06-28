# LLM Runtime

[**English**](llm-runtime.md) | [**中文**](llm-runtime.zh.md)

---

The model integration uses a DeepSeek-compatible Chat Completions API shape. The HTTP client posts to:

```text
{base_url}/chat/completions
```

The base URL, model, API key, and strict-tools preference come from the latest saved API profile.

## World Draft Request

World draft generation uses:

| Field | Value |
|---|---|
| `tools` | `None` |
| `tool_choice` | `None` |
| `response_format` | `{ "type": "json_object" }` |
| `temperature` | `0.9` |
| `max_tokens` | `1200` |
| `stream` | `false` |

The returned JSON is parsed as `CreateWorldRequest`.

## Story Turn Request

Story turn preview uses:

| Field | Value |
|---|---|
| `tools` | Read-only tool definitions |
| `tool_choice` | `"auto"` |
| `response_format` | `{ "type": "json_object" }` |
| `temperature` | `0.85` |
| `max_tokens` | `4096` |
| `stream` | `false` |

The returned final JSON is parsed as `TurnOutput`.

## Read-Only Tools

Only these tools are registered in the current code:

| Tool | Required arguments | Behavior |
|---|---|---|
| `query_character_profile` | `character_id` | Reads one row from `characters` |
| `query_character_memory` | `character_id`, `query`, `limit` | Reads promoted `memories` for a character with SQL `LIKE` |
| `query_past_events` | `query`, `limit` | Reads `turns.summary` with SQL `LIKE` |

Unknown tool names return an empty result with an error message. Tools never write.

## Tool Loop

When the model returns `tool_calls`, Rust:

1. Counts the tool round and total tool calls.
2. Pushes the assistant message containing tool calls into the message list.
3. Executes each tool against the same world database connection.
4. Serializes each result as a `role: "tool"` message.
5. Calls the model again.

If tool use exceeds the limits, Rust appends:

```text
Stop calling tools. Return the final valid json now.
```

Limits:

- Maximum tool rounds: `3`
- Maximum total tool calls: `8`

## Parsing And Repair

If the model returns empty content, invalid JSON, or JSON that fails validation, Rust appends a repair instruction and retries until the story-turn repair limit is reached.

Validation checks include:

- Non-empty `narration`
- Exactly three choices
- Choice labels exactly `A`, `B`, `C`
- Risk values only `low`, `medium`, or `high`
- Allowed state update keys
- At most three new characters
- Memory importance from `1` to `10`
- Relationship deltas from `-2` to `2`
- No duplicate relationship update for the same character and dimension

## Important Non-Features

The current LLM runtime does not include:

- Streaming responses
- Write tools
- Vector search
- Embedding retrieval
- A `query_world_lore` tool
- Grammar/vocabulary teaching in the story prompt
