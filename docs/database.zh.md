# 数据库结构

[**English**](database.md) | [**中文**](database.zh.md)

---

语境传说使用两级 SQLite：一个应用数据库，以及每个世界一个独立数据库。

## 存储根目录

`AppState::initialize` 使用 Tauri 的应用数据目录，并创建：

```text
<app-data>/
  app.db
  worlds/
    <world_id>/
      world.db
  logs/
```

具体基础路径取决于操作系统和 Tauri 对 app data 目录的解析。

## 应用数据库

Migration：`apps/desktop/src-tauri/migrations/app/001_init.sql`

| 表 | 用途 |
|---|---|
| `worlds` | 世界库记录，包含标题、简介、存储路径、目标语言、等级和时间戳 |
| `api_profiles` | 已保存的模型 API 配置，API key 会加密保存 |
| `global_settings` | 键值设置 |

## 世界数据库

Migration：`apps/desktop/src-tauri/migrations/world/001_init.sql`

| 表 | 用途 |
|---|---|
| `world_profile` | 描述世界前提、类型、目标语言、等级和叙事风格 |
| `characters` | 玩家和 NPC 档案；玩家角色固定为 `char_player` |
| `scenes` | 场景标题、地点、氛围、摘要、当前目标和状态 |
| `messages` | 与回合关联的用户消息和助手消息 |
| `turns` | 故事回合元数据、摘要、模型原始 JSON 和消息链接 |
| `branch_choices` | 每回合生成的下一组选项；被选择的选项会被标记 |
| `story_state` | 持久键值状态，例如 `scene.current`、flag、inventory、quest 状态和场景字段 |
| `state_update_logs` | 已应用状态更新的旧值、新值、原因和回合 id |
| `memories` | 已晋升的持久记忆，可被只读工具查询 |
| `memory_candidates` | 模型提出的全部记忆候选，包括未晋升候选 |
| `relationship_state` | 按角色和维度存储的当前关系值 |
| `relationship_update_logs` | 已应用关系变化量和原因 |

Migration 还会删除遗留的 `chapters` 和 `world_lore` 表。当前代码不使用 lore 表、词汇表、向量数据库或 embedding 索引。

## 初始化

`create_world` 会向 `app.db.worlds` 插入一条记录，创建世界目录，打开 `world.db`，执行 migration，然后 seed：

- 来自创建请求的 `world_profile`。
- 一个名为 `Opening Scene` 的 `scenes` 行。
- `story_state.scene.current`，指向开场场景。
- 严格一个玩家角色，存为 `char_player`。
- 可选的非玩家初始角色，关系 `trust = 0`。

## 导入与导出

世界导出会创建一个 ZIP，包含：

```text
manifest.json
world.db
```

导入会读取 manifest 和 `world.db`，分配新的 world id，更新 `world_profile.id`，并插入对应的 `app.db.worlds` 记录。
