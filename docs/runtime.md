# Runtime Mechanism

[**English**](runtime.md) | [**中文**](runtime.zh.md)

---

This page describes the runtime according to the current code, not old README wording.

![Lingua Lore runtime overview](images/runtime-overview.png)

## End-To-End Flow

There are four distinct operations:

1. `generate_world_draft`: asks the model for a draft world form.
2. `create_world`: creates local storage and seeds a world database.
3. `preview_story_turn`: generates and validates a story turn without committing it.
4. `commit_story_turn_preview`: commits an accepted preview into the world database.

The separation between preview and commit is deliberate. It lets quick mode prefetch branches and lets Rust validate the generated output before any state mutation happens.

## World Draft Generation

The "AI fill" path calls `generate_world_draft` in `world_commands.rs`.

- It loads the latest API profile from `app.db`.
- It uses a DeepSeek-compatible chat completion request.
- It sets `response_format` to `json_object`.
- It does not expose tools.
- It asks for exactly one player viewpoint character.
- It validates that the draft has exactly one player character with required fields.
- It normalizes the selected genre, target language, language level, narrative style, and character flags.

The draft is returned to the frontend form. It does not create a world by itself.

## World Initialization

`create_world` persists the form data:

- Creates a new `world_<uuid>` id.
- Inserts a row into `app.db.worlds`.
- Creates `worlds/<world_id>/world.db`.
- Applies the world migration.
- Inserts `world_profile`.
- Inserts one opening scene named `Opening Scene`.
- Sets `story_state.scene.current` to the opening scene id.
- Requires exactly one player character and stores it as `char_player`.
- Adds optional non-player seed characters with default `trust = 0`.

## Opening A World

`get_world_bootstrap` loads:

- The world record from `app.db`.
- The current scene id from `story_state.scene.current`, falling back to the first scene.
- Prior story turns from `turns`, `messages`, and `branch_choices`.

The frontend stores this bootstrap result in app state and renders the reader.

## Story Turn Runtime

![Lingua Lore story turn runtime](images/story-turn-runtime.png)

The first reader action is a free-text action:

```text
Initialize the story with a vivid opening scene.
```

Later actions are either:

- `{ kind: "choice", choice_id }`
- `{ kind: "free_text", text }`

`preview_story_turn` then:

1. Opens the world database.
2. Calls `load_context`.
3. Calls `build_messages`.
4. Registers the read-only tools.
5. Calls the model with `tool_choice: "auto"` and `response_format: json_object`.
6. Executes requested tool calls, if any, and appends `role: "tool"` messages.
7. Parses the final content as `TurnOutput`.
8. Validates the output.
9. Returns a preview containing the raw JSON and parsed output.

If the model returns invalid content, Rust appends a repair instruction and retries the turn.

## Runtime Limits

| Area | Limit |
|---|---:|
| Model request retries | 4 |
| World draft repair attempts | 4 |
| Story turn repair attempts | 4 |
| Tool rounds per story preview | 3 |
| Total tool calls per story preview | 8 |
| Recent messages loaded | 12 |
| Recent summaries loaded | 8 |
| Characters loaded into prompt | 12 |
| Story state rows loaded | 80 |
| Relationship rows loaded | 80 |

## Context Loading

`load_context` reads:

- `world_profile`
- Current `scenes` row
- Up to 12 characters, player first
- Up to 80 `story_state` rows
- Up to 80 `relationship_state` rows
- Up to 12 recent messages for the current scene
- Up to 8 recent summaries for the current scene
- The current user action, resolved from free text or the selected `branch_choices` row

## Commit

`commit_story_turn_preview` validates again and calls `commit_turn`. The commit happens in one SQLite transaction.

It writes:

- User and assistant messages
- `turns` row
- Updated scene location, mood, objective, and summary
- New branch choices with stable ids
- Story state updates plus logs
- New non-player characters
- Memory candidates and promoted memories
- Relationship state deltas plus logs

The model still does not write directly. It proposes structured output; Rust decides what is valid and what gets persisted.

## Memory System

The current memory system is SQLite-based:

- The model emits `memory_candidates`.
- Every candidate is stored in `memory_candidates`.
- A candidate is promoted into `memories` only if `importance >= 7` and `character_id` exists.
- `query_character_memory` later retrieves promoted memories with SQL `LIKE`, ordered by importance and recency.

There is no vector search or embedding index in the current implementation.

## Quick Mode

Quick mode is a frontend prefetch behavior. It does not change the model, prompt, temperature, or validation rules.

When quick mode is enabled and the current turn has choice ids, the frontend calls `preview_story_turn` for each available choice and caches the promises/results. If the player selects a prefetched choice, the frontend sends that cached preview to `commit_story_turn_preview`.

This can make selection feel faster, but it may use more model requests because unselected branches can be generated.

## Selection Translation

Selection translation is outside the story loop. Highlighted text is sent to the Youdao provider with source/target language and optional context. The result is shown in a popover and does not enter the story prompt or mutate world state.
